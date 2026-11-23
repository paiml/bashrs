//! Faithful syntax tree for `bashrs parse`.
//!
//! The older `BashAst` in `ast.rs` is built for transpilation and is lossy: it
//! drops here-documents, keeps quoted words as bare literals, and flattens
//! `$(...)` to a string. This tree keeps every construct that can run a command,
//! so a consumer can ask "which commands does this script run?" and get an answer
//! derived from the tree rather than from a second hand-kept list.

use serde::{Serialize, Serializer};
use std::cell::RefCell;
use std::rc::Rc;

/// A part of the input the parser could not model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unparsed {
    pub line: usize,
    pub reason: String,
}

/// A slot filled after the node holding it is built (here-document bodies are
/// read at the next newline, after the redirect that names them).
#[derive(Debug, Clone)]
pub struct Shared<T>(pub Rc<RefCell<T>>);

impl<T: Serialize> Serialize for Shared<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.borrow().serialize(s)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Word {
    /// The word exactly as written in the source.
    pub text: String,
    pub line: usize,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Part {
    /// Unquoted literal with escapes removed. `glob` is set when the shell would
    /// expand it (`*`, `?`, `[`, brace lists, leading `~`).
    Lit {
        value: String,
        glob: bool,
    },
    Single {
        value: String,
    },
    Double {
        parts: Vec<Part>,
    },
    AnsiC {
        raw: String,
    },
    /// `$x`, `${...}`; `inner` holds expansions nested inside the braces.
    Param {
        raw: String,
        inner: Vec<Part>,
    },
    Arith {
        raw: String,
        inner: Vec<Part>,
    },
    CmdSubst {
        raw: String,
        backtick: bool,
        body: Vec<Node>,
    },
    ProcSubst {
        raw: String,
        dir: String,
        body: Vec<Node>,
    },
}

impl Part {
    pub fn is_static(&self) -> bool {
        match self {
            Part::Lit { glob, .. } => !glob,
            Part::Single { .. } => true,
            Part::Double { parts } => parts.iter().all(Part::is_static),
            _ => false,
        }
    }

    fn value_into(&self, out: &mut String) {
        match self {
            Part::Lit { value, .. } | Part::Single { value } => out.push_str(value),
            Part::Double { parts } => parts.iter().for_each(|p| p.value_into(out)),
            _ => {}
        }
    }

    fn template_into(&self, out: &mut String) {
        match self {
            Part::Lit { value, .. } | Part::Single { value } => out.push_str(value),
            Part::Double { parts } => parts.iter().for_each(|p| p.template_into(out)),
            Part::AnsiC { raw }
            | Part::Param { raw, .. }
            | Part::Arith { raw, .. }
            | Part::CmdSubst { raw, .. }
            | Part::ProcSubst { raw, .. } => out.push_str(raw),
        }
    }
}

impl Word {
    pub fn is_static(&self) -> bool {
        self.parts.iter().all(Part::is_static)
    }

    /// The word after quote removal, when nothing in it expands.
    pub fn value(&self) -> Option<String> {
        if !self.is_static() {
            return None;
        }
        let mut s = String::new();
        self.parts.iter().for_each(|p| p.value_into(&mut s));
        Some(s)
    }

    /// The text a nested shell would receive: quotes removed, expansions kept
    /// as written so the nested parse still sees them.
    pub fn template(&self) -> String {
        let mut s = String::new();
        self.parts.iter().for_each(|p| p.template_into(&mut s));
        s
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Node {
    #[serde(flatten)]
    pub kind: Kind,
    pub line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Kind {
    Simple(Simple),
    Pipeline {
        negated: bool,
        timed: bool,
        stages: Vec<Node>,
    },
    AndOr {
        op: String,
        left: Box<Node>,
        right: Box<Node>,
    },
    Background {
        body: Box<Node>,
    },
    Subshell {
        body: Vec<Node>,
    },
    Group {
        body: Vec<Node>,
    },
    If {
        cond: Vec<Node>,
        then_body: Vec<Node>,
        elifs: Vec<Elif>,
        else_body: Option<Vec<Node>>,
    },
    Loop {
        until: bool,
        cond: Vec<Node>,
        body: Vec<Node>,
    },
    For {
        var: String,
        items: Option<Vec<Word>>,
        body: Vec<Node>,
    },
    Select {
        var: String,
        items: Option<Vec<Word>>,
        body: Vec<Node>,
    },
    Case {
        word: Word,
        arms: Vec<Arm>,
    },
    Function {
        name: String,
        body: Box<Node>,
    },
    Coproc {
        name: Option<String>,
        body: Box<Node>,
    },
    ArithCmd {
        inner: Vec<Part>,
    },
    ArithFor {
        inner: Vec<Part>,
        body: Vec<Node>,
    },
    Cond {
        words: Vec<Word>,
    },
    Redirected {
        body: Box<Node>,
        redirects: Vec<Redirect>,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct Elif {
    pub cond: Vec<Node>,
    pub body: Vec<Node>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Arm {
    pub patterns: Vec<Word>,
    pub body: Vec<Node>,
    pub terminator: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Simple {
    pub argv: Vec<Word>,
    pub assignments: Vec<Assign>,
    pub redirects: Vec<Redirect>,
    /// Leading wrappers (`sudo`, `env`, ...) peeled off `argv`.
    pub wrappers: Vec<Wrapper>,
    /// Index into `argv` of the command the wrappers run.
    pub exec_index: usize,
    /// Scripts the command passes to a nested shell (`sh -c`, `eval`), parsed.
    pub embedded: Vec<Embedded>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Embedded {
    pub kind: String,
    pub shell: String,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Wrapper {
    pub name: String,
    pub args: Vec<String>,
    pub assignments: Vec<String>,
    /// The wrapper appends words read from stdin to the command (`xargs`).
    pub stdin_args: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Assign {
    pub name: String,
    pub append: bool,
    pub value: Option<Word>,
    pub elements: Option<Vec<Word>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Redirect {
    pub fd: Option<String>,
    pub op: String,
    pub target: Option<Word>,
    pub heredoc: Option<Heredoc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Heredoc {
    pub delimiter: String,
    pub expand: bool,
    pub strip_tabs: bool,
    pub body: Shared<Option<HeredocBody>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeredocBody {
    pub line: usize,
    pub lines: usize,
    pub parts: Vec<Part>,
}
