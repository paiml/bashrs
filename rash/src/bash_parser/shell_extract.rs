//! Derives the flat `commands` list by walking the syntax tree. There is no
//! second list of constructs: a node kind the walker forgets to visit is a
//! compile error (exhaustive matches) or a failing property test.

use super::shell_ast::*;
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize)]
pub struct Command {
    pub argv: Vec<Value>,
    pub line: usize,
    pub context: Vec<String>,
    pub wrappers: Vec<String>,
}

fn arg_json(w: &Word) -> Value {
    match w.value() {
        Some(v) => Value::String(v),
        None => json!({ "dynamic": true, "text": w.text }),
    }
}

struct Walker {
    ctx: Vec<String>,
    out: Vec<Command>,
}

impl Walker {
    fn scoped<F: FnOnce(&mut Self)>(&mut self, name: &str, f: F) {
        self.ctx.push(name.to_string());
        f(self);
        self.ctx.pop();
    }

    fn nodes(&mut self, ns: &[Node]) {
        ns.iter().for_each(|n| self.node(n));
    }

    fn words(&mut self, ws: &[Word]) {
        ws.iter().for_each(|w| self.parts(&w.parts));
    }

    fn parts(&mut self, ps: &[Part]) {
        ps.iter().for_each(|p| self.part(p));
    }

    fn part(&mut self, p: &Part) {
        match p {
            Part::Lit { .. } | Part::Single { .. } | Part::AnsiC { .. } => {}
            Part::Double { parts } => self.parts(parts),
            Part::Param { inner, .. } | Part::Arith { inner, .. } => self.parts(inner),
            Part::CmdSubst { backtick, body, .. } => {
                let name = if *backtick { "backtick" } else { "cmdsubst" };
                self.scoped(name, |w| w.nodes(body));
            }
            Part::ProcSubst { body, .. } => self.scoped("procsub", |w| w.nodes(body)),
        }
    }

    fn redirects(&mut self, rs: &[Redirect]) {
        for r in rs {
            if let Some(t) = &r.target {
                self.parts(&t.parts);
            }
            if let Some(h) = &r.heredoc {
                if let Some(body) = h.body.0.borrow().as_ref() {
                    self.scoped("heredoc", |w| w.parts(&body.parts));
                }
            }
        }
    }

    fn node(&mut self, n: &Node) {
        match &n.kind {
            Kind::Simple(s) => self.simple(n, s),
            Kind::Pipeline { stages, .. } => self.scoped("pipeline", |w| w.nodes(stages)),
            Kind::AndOr { left, right, .. } => {
                self.node(left);
                self.node(right);
            }
            Kind::Background { body } => self.scoped("background", |w| w.node(body)),
            Kind::Subshell { body } => self.scoped("subshell", |w| w.nodes(body)),
            Kind::Group { body } => self.scoped("group", |w| w.nodes(body)),
            Kind::If {
                cond,
                then_body,
                elifs,
                else_body,
            } => self.scoped("if", |w| {
                w.nodes(cond);
                w.nodes(then_body);
                for e in elifs {
                    w.nodes(&e.cond);
                    w.nodes(&e.body);
                }
                if let Some(b) = else_body {
                    w.nodes(b);
                }
            }),
            Kind::Loop { until, cond, body } => {
                let name = if *until { "until" } else { "while" };
                self.scoped(name, |w| {
                    w.nodes(cond);
                    w.nodes(body);
                });
            }
            Kind::For { items, body, .. } => self.scoped("for", |w| {
                if let Some(i) = items {
                    w.words(i);
                }
                w.nodes(body);
            }),
            Kind::Select { items, body, .. } => self.scoped("select", |w| {
                if let Some(i) = items {
                    w.words(i);
                }
                w.nodes(body);
            }),
            Kind::Case { word, arms } => self.scoped("case", |w| {
                w.parts(&word.parts);
                for a in arms {
                    w.words(&a.patterns);
                    w.nodes(&a.body);
                }
            }),
            Kind::Function { body, .. } => self.scoped("function", |w| w.node(body)),
            Kind::Coproc { body, .. } => self.scoped("coproc", |w| w.node(body)),
            Kind::ArithCmd { inner } => self.scoped("arith", |w| w.parts(inner)),
            Kind::ArithFor { inner, body } => self.scoped("for", |w| {
                w.parts(inner);
                w.nodes(body);
            }),
            Kind::Cond { words } => self.scoped("cond", |w| w.words(words)),
            Kind::Redirected { body, redirects } => {
                self.node(body);
                self.redirects(redirects);
            }
        }
    }

    fn simple(&mut self, n: &Node, s: &Simple) {
        if s.exec_index < s.argv.len() {
            self.out.push(Command {
                argv: s.argv[s.exec_index..].iter().map(arg_json).collect(),
                line: n.line,
                context: self.ctx.clone(),
                wrappers: s.wrappers.iter().map(|w| w.name.clone()).collect(),
            });
        }
        for a in &s.assignments {
            if let Some(v) = &a.value {
                self.parts(&v.parts);
            }
            if let Some(es) = &a.elements {
                self.words(es);
            }
        }
        self.words(&s.argv);
        self.redirects(&s.redirects);
        for e in &s.embedded {
            let kind = e.kind.clone();
            self.scoped(&kind, |w| w.nodes(&e.nodes));
        }
    }
}

pub fn extract(program: &[Node]) -> Vec<Command> {
    let mut w = Walker {
        ctx: Vec::new(),
        out: Vec::new(),
    };
    w.nodes(program);
    w.out
}

/// The full `bashrs parse` JSON document for a parsed program.
pub fn document(program: &[Node]) -> Value {
    json!({ "ast": program, "commands": extract(program) })
}
