//! Recursive-descent parser for the command structure of POSIX/bash scripts.
//!
//! It models exactly what can run a command. Anything it recognises but does
//! not model (`((`, `{fd}>`, extglob, ...) is an `Unparsed` error, never a
//! silent skip: the caller must treat the whole input as unparsed.

use super::shell_ast::*;
use super::shell_wrap;
use std::cell::RefCell;
use std::rc::Rc;

pub type R<T> = Result<T, Unparsed>;

const CLOSERS: [&str; 7] = ["then", "else", "elif", "fi", "do", "done", "esac"];
const DECLS: [&str; 5] = ["declare", "local", "typeset", "readonly", "export"];
const COMPOUND_WORDS: [&str; 8] = [
    "if", "while", "until", "for", "select", "case", "[[", "function",
];

struct Pending {
    delim: String,
    strip: bool,
    expand: bool,
    slot: Shared<Option<HeredocBody>>,
}

pub struct Parser {
    s: Vec<char>,
    i: usize,
    line: usize,
    pending: Vec<Pending>,
}

fn is_name(s: &str) -> bool {
    let mut it = s.chars();
    matches!(it.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && it.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl Parser {
    pub fn new(src: &str, first_line: usize) -> Self {
        Parser {
            s: src.chars().collect(),
            i: 0,
            line: first_line,
            pending: Vec::new(),
        }
    }

    // ---- cursor helpers -------------------------------------------------

    fn err<T>(&self, reason: impl Into<String>) -> R<T> {
        Err(Unparsed {
            line: self.line,
            reason: reason.into(),
        })
    }

    fn peek(&self) -> Option<char> {
        self.s.get(self.i).copied()
    }

    fn at(&self, n: usize) -> Option<char> {
        self.s.get(self.i + n).copied()
    }

    fn eof(&self) -> bool {
        self.i >= self.s.len()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.i += 1;
        if c == '\n' {
            self.line += 1;
        }
        Some(c)
    }

    fn adv(&mut self, n: usize) {
        for _ in 0..n {
            self.bump();
        }
    }

    fn looking_at(&self, pat: &str) -> bool {
        pat.chars().enumerate().all(|(k, c)| self.at(k) == Some(c))
    }

    fn slice(&self, from: usize) -> String {
        self.s[from..self.i].iter().collect()
    }

    fn describe(&self) -> String {
        match self.peek() {
            None => "end of input".to_string(),
            Some(c) => format!("`{c}`"),
        }
    }

    fn expect_char(&mut self, c: char) -> R<()> {
        self.skip_blanks();
        if self.peek() == Some(c) {
            self.bump();
            Ok(())
        } else {
            self.err(format!("expected `{c}`, found {}", self.describe()))
        }
    }

    fn skip_blanks(&mut self) {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') => {
                    self.bump();
                }
                Some('\\') if self.at(1) == Some('\n') => self.adv(2),
                Some('#') => {
                    while !matches!(self.peek(), None | Some('\n')) {
                        self.bump();
                    }
                }
                _ => return,
            }
        }
    }

    fn skip_blanks_and_newlines(&mut self) -> R<()> {
        loop {
            self.skip_blanks();
            if self.peek() == Some('\n') {
                self.bump();
                self.read_heredocs()?;
            } else {
                return Ok(());
            }
        }
    }

    /// The next token if it is a plain unquoted word, without consuming it.
    fn peek_word_text(&self) -> Option<String> {
        let mut out = String::new();
        let mut j = self.i;
        while let Some(&c) = self.s.get(j) {
            if matches!(
                c,
                ' ' | '\t' | '\n' | '|' | '&' | ';' | '(' | ')' | '<' | '>'
            ) {
                break;
            }
            if matches!(c, '\'' | '"' | '`' | '$' | '\\') {
                return None;
            }
            out.push(c);
            j += 1;
        }
        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    fn is_closer_word(&self) -> bool {
        match self.peek_word_text() {
            Some(w) => CLOSERS.contains(&w.as_str()) || w == "}",
            None => false,
        }
    }

    fn expect_word(&mut self, w: &str) -> R<()> {
        self.skip_blanks();
        if self.peek_word_text().as_deref() == Some(w) {
            self.adv(w.chars().count());
            Ok(())
        } else {
            self.err(format!("expected `{w}`, found {}", self.describe_word()))
        }
    }

    fn describe_word(&self) -> String {
        match self.peek_word_text() {
            Some(w) => format!("`{w}`"),
            None => self.describe(),
        }
    }

    // ---- entry ----------------------------------------------------------

    pub fn parse_program(&mut self) -> R<Vec<Node>> {
        let body = self.parse_list()?;
        self.skip_blanks();
        if !self.eof() {
            return self.err(format!("unexpected {}", self.describe_word()));
        }
        if !self.pending.is_empty() {
            return self.err("here-document has no body");
        }
        Ok(body)
    }

    // ---- lists ----------------------------------------------------------

    /// After a command in a list: consumes `;` or `&` and wraps `&`.
    fn list_terminator(&mut self, node: Node) -> Node {
        self.skip_blanks();
        match self.peek() {
            Some(';') if !(self.looking_at(";;") || self.looking_at(";&")) => {
                self.bump();
                node
            }
            Some('&') if !self.looking_at("&&") => {
                self.bump();
                let (line, end_line) = (node.line, self.line);
                Node {
                    kind: Kind::Background {
                        body: Box::new(node),
                    },
                    line,
                    end_line,
                }
            }
            _ => node,
        }
    }

    /// Whether the list ends here; consumes blank lines first.
    fn list_ends(&mut self) -> R<bool> {
        loop {
            self.skip_blanks();
            match self.peek() {
                None | Some(')') => return Ok(true),
                Some('\n') => {
                    self.bump();
                    self.read_heredocs()?;
                }
                Some(';') => {
                    if self.looking_at(";;") || self.looking_at(";&") {
                        return Ok(true);
                    }
                    return self.err("unexpected `;`");
                }
                _ => return Ok(self.is_closer_word()),
            }
        }
    }

    fn parse_list(&mut self) -> R<Vec<Node>> {
        let mut out = Vec::new();
        while !self.list_ends()? {
            let start = self.i;
            let node = self.parse_and_or()?;
            if self.i == start {
                return self.err(format!("unexpected {}", self.describe_word()));
            }
            out.push(self.list_terminator(node));
        }
        Ok(out)
    }

    fn parse_and_or(&mut self) -> R<Node> {
        let mut left = self.parse_pipeline()?;
        loop {
            self.skip_blanks();
            let op = if self.looking_at("&&") {
                "&&"
            } else if self.looking_at("||") {
                "||"
            } else {
                break;
            };
            self.adv(2);
            self.skip_blanks_and_newlines()?;
            let right = self.parse_pipeline()?;
            let line = left.line;
            left = Node {
                kind: Kind::AndOr {
                    op: op.to_string(),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                line,
                end_line: self.line,
            };
        }
        Ok(left)
    }

    /// Leading `!` and `time [-p]`: `(negated, timed)`.
    fn pipeline_prefix(&mut self) -> (bool, bool) {
        let (mut negated, mut timed) = (false, false);
        loop {
            self.skip_blanks();
            match self.peek_word_text().as_deref() {
                Some("!") => {
                    negated = !negated;
                    self.adv(1);
                }
                Some("time") => {
                    self.adv(4);
                    self.skip_blanks();
                    if self.peek_word_text().as_deref() == Some("-p") {
                        self.adv(2);
                    }
                    timed = true;
                }
                _ => return (negated, timed),
            }
        }
    }

    /// Consumes `|` or `|&` (not `||`).
    fn pipe_operator(&mut self) -> bool {
        self.skip_blanks();
        if self.looking_at("||") {
            return false;
        }
        if self.looking_at("|&") {
            self.adv(2);
            return true;
        }
        if self.peek() == Some('|') {
            self.adv(1);
            return true;
        }
        false
    }

    fn parse_pipeline(&mut self) -> R<Node> {
        self.skip_blanks();
        let line = self.line;
        let (negated, timed) = self.pipeline_prefix();
        let mut stages = vec![self.parse_command()?];
        while self.pipe_operator() {
            self.skip_blanks_and_newlines()?;
            stages.push(self.parse_command()?);
        }
        if stages.len() == 1 && !negated && !timed {
            if let Some(only) = stages.pop() {
                return Ok(only);
            }
        }
        Ok(Node {
            kind: Kind::Pipeline {
                negated,
                timed,
                stages,
            },
            line,
            end_line: self.line,
        })
    }

    // ---- commands -------------------------------------------------------

    fn parse_command(&mut self) -> R<Node> {
        self.skip_blanks();
        let line = self.line;
        let kind = if self.peek() == Some('(') {
            match if self.at(1) == Some('(') {
                self.scan_arith(2)
            } else {
                None
            } {
                Some(inner) => Kind::ArithCmd { inner },
                None => {
                    self.bump();
                    let body = self.parse_list()?;
                    self.expect_char(')')?;
                    Kind::Subshell { body }
                }
            }
        } else if self.peek() == Some('{')
            && matches!(self.at(1), None | Some(' ') | Some('\t') | Some('\n'))
        {
            self.bump();
            let body = self.parse_list()?;
            self.expect_word("}")?;
            Kind::Group { body }
        } else {
            match self.peek_word_text().as_deref() {
                Some("if") => self.parse_if()?,
                Some(w @ ("while" | "until")) => {
                    let until = w == "until";
                    self.adv(w.len());
                    let cond = self.parse_list()?;
                    self.expect_word("do")?;
                    let body = self.parse_list()?;
                    self.expect_word("done")?;
                    Kind::Loop { until, cond, body }
                }
                Some("for") => self.parse_for(false)?,
                Some("select") => self.parse_for(true)?,
                Some("case") => self.parse_case()?,
                Some("function") => self.parse_function_kw()?,
                Some("coproc") => self.parse_coproc()?,
                Some("[[") => self.parse_cond()?,
                Some(w) if CLOSERS.contains(&w) || w == "}" => {
                    return self.err(format!("unexpected `{w}`"));
                }
                _ => return self.parse_simple(),
            }
        };
        let body = Node {
            kind,
            line,
            end_line: self.line,
        };
        self.wrap_redirects(body)
    }

    fn wrap_redirects(&mut self, body: Node) -> R<Node> {
        let mut redirects = Vec::new();
        loop {
            self.skip_blanks();
            if self.is_redirect_start() {
                redirects.push(self.parse_redirect()?);
            } else {
                break;
            }
        }
        if redirects.is_empty() {
            return Ok(body);
        }
        let line = body.line;
        Ok(Node {
            kind: Kind::Redirected {
                body: Box::new(body),
                redirects,
            },
            line,
            end_line: self.line,
        })
    }

    fn parse_if(&mut self) -> R<Kind> {
        self.adv(2);
        let cond = self.parse_list()?;
        self.expect_word("then")?;
        let then_body = self.parse_list()?;
        let mut elifs = Vec::new();
        let mut else_body = None;
        loop {
            self.skip_blanks();
            match self.peek_word_text().as_deref() {
                Some("elif") => {
                    self.adv(4);
                    let cond = self.parse_list()?;
                    self.expect_word("then")?;
                    let body = self.parse_list()?;
                    elifs.push(Elif { cond, body });
                }
                Some("else") => {
                    self.adv(4);
                    else_body = Some(self.parse_list()?);
                }
                _ => break,
            }
        }
        self.expect_word("fi")?;
        Ok(Kind::If {
            cond,
            then_body,
            elifs,
            else_body,
        })
    }

    /// Skips `;` and newlines (reading any pending here-documents).
    fn skip_separators(&mut self) -> R<()> {
        loop {
            self.skip_blanks();
            match self.peek() {
                Some(';') => {
                    self.bump();
                }
                Some('\n') => {
                    self.bump();
                    self.read_heredocs()?;
                }
                _ => return Ok(()),
            }
        }
    }

    /// `do LIST done`, after the loop header.
    fn parse_do_done(&mut self) -> R<Vec<Node>> {
        self.skip_separators()?;
        self.expect_word("do")?;
        let body = self.parse_list()?;
        self.expect_word("done")?;
        Ok(body)
    }

    /// Words after `in` up to `;` or a newline.
    fn read_item_words(&mut self) -> R<Vec<Word>> {
        let mut words = Vec::new();
        loop {
            self.skip_blanks();
            if matches!(self.peek(), None | Some('\n') | Some(';')) {
                return Ok(words);
            }
            let start = self.i;
            let w = self.read_word()?;
            if self.i == start {
                return self.err(format!("unexpected {}", self.describe()));
            }
            words.push(w);
        }
    }

    fn parse_for(&mut self, select: bool) -> R<Kind> {
        self.adv(if select { 6 } else { 3 });
        self.skip_blanks();
        if self.looking_at("((") && !select {
            let inner = match self.scan_arith(2) {
                Some(i) => i,
                None => return self.err("malformed `for ((` loop header"),
            };
            let body = self.parse_do_done()?;
            return Ok(Kind::ArithFor { inner, body });
        }
        let var = match self.peek_word_text() {
            Some(v) if is_name(&v) => v,
            _ => return self.err("expected a loop variable name"),
        };
        self.adv(var.chars().count());
        self.skip_blanks_and_newlines()?;
        let mut items = None;
        if self.peek_word_text().as_deref() == Some("in") {
            self.adv(2);
            items = Some(self.read_item_words()?);
        }
        let body = self.parse_do_done()?;
        Ok(if select {
            Kind::Select { var, items, body }
        } else {
            Kind::For { var, items, body }
        })
    }

    fn word_ends_pattern(&self, len: usize) -> bool {
        let mut j = self.i + len;
        while matches!(self.s.get(j), Some(' ') | Some('\t')) {
            j += 1;
        }
        matches!(self.s.get(j), Some(')') | Some('|'))
    }

    /// One arm's patterns: `[(] pat ( | pat )* )`.
    fn parse_case_patterns(&mut self) -> R<Vec<Word>> {
        if self.peek() == Some('(') {
            self.bump();
            self.skip_blanks();
        }
        let mut patterns = Vec::new();
        loop {
            self.skip_blanks();
            let s = self.i;
            let w = self.read_word()?;
            if self.i == s {
                return self.err(format!(
                    "expected a case pattern, found {}",
                    self.describe()
                ));
            }
            patterns.push(w);
            self.skip_blanks();
            if self.peek() != Some('|') {
                break;
            }
            self.bump();
        }
        self.expect_char(')')?;
        Ok(patterns)
    }

    /// `;;`, `;&` or `;;&`, if present.
    fn case_terminator(&mut self) -> Option<String> {
        self.skip_blanks();
        for t in [";;&", ";;", ";&"] {
            if self.looking_at(t) {
                self.adv(t.len());
                return Some(t.to_string());
            }
        }
        None
    }

    /// `esac` closes the `case` (a first-arm pattern `esac)` does not).
    fn at_esac(&self, first: bool) -> bool {
        self.peek_word_text().as_deref() == Some("esac") && !(first && self.word_ends_pattern(4))
    }

    fn parse_case(&mut self) -> R<Kind> {
        self.adv(4);
        self.skip_blanks();
        let start = self.i;
        let word = self.read_word()?;
        if self.i == start {
            return self.err("expected a word after `case`");
        }
        self.skip_blanks_and_newlines()?;
        self.expect_word("in")?;
        let mut arms = Vec::new();
        let mut first = true;
        loop {
            self.skip_blanks_and_newlines()?;
            if self.eof() {
                return self.err("unterminated `case`");
            }
            if self.at_esac(first) {
                self.adv(4);
                break;
            }
            first = false;
            let patterns = self.parse_case_patterns()?;
            let body = self.parse_list()?;
            let terminator = self.case_terminator();
            let last = terminator.is_none();
            arms.push(Arm {
                patterns,
                body,
                terminator,
            });
            if last {
                self.skip_blanks_and_newlines()?;
                if self.peek_word_text().as_deref() != Some("esac") {
                    return self.err("expected `;;` or `esac` after a case arm");
                }
            }
        }
        Ok(Kind::Case { word, arms })
    }

    fn parse_function_kw(&mut self) -> R<Kind> {
        self.adv(8);
        self.skip_blanks();
        let name = match self.peek_word_text() {
            Some(n) => n,
            None => return self.err("expected a function name"),
        };
        self.adv(name.chars().count());
        self.skip_blanks();
        if self.peek() == Some('(') {
            self.bump();
            self.expect_char(')')?;
        }
        self.function_body(name)
    }

    fn function_body(&mut self, name: String) -> R<Kind> {
        self.skip_blanks_and_newlines()?;
        let body = self.parse_command()?;
        if matches!(body.kind, Kind::Simple(_)) {
            return self.err("a function body must be a compound command");
        }
        Ok(Kind::Function {
            name,
            body: Box::new(body),
        })
    }

    fn at_compound_start(&self) -> bool {
        match self.peek() {
            Some('(') => true,
            Some('{') => matches!(self.at(1), None | Some(' ') | Some('\t') | Some('\n')),
            _ => self
                .peek_word_text()
                .map(|w| COMPOUND_WORDS.contains(&w.as_str()))
                .unwrap_or(false),
        }
    }

    fn parse_coproc(&mut self) -> R<Kind> {
        self.adv(6);
        self.skip_blanks();
        let mut name = None;
        if !self.at_compound_start() {
            if let Some(w) = self.peek_word_text() {
                if is_name(&w) {
                    let save = (self.i, self.line, self.pending.len());
                    self.adv(w.chars().count());
                    self.skip_blanks();
                    if self.at_compound_start() {
                        name = Some(w);
                    } else {
                        self.i = save.0;
                        self.line = save.1;
                        self.pending.truncate(save.2);
                    }
                }
            }
        }
        let body = self.parse_command()?;
        Ok(Kind::Coproc {
            name,
            body: Box::new(body),
        })
    }

    /// One `[[ ... ]]` token that is not a plain word; `None` for a word.
    fn cond_operator(&mut self) -> Option<String> {
        match self.peek() {
            Some(c @ ('(' | ')')) => {
                self.bump();
                Some(c.to_string())
            }
            Some(c @ ('<' | '>')) if self.at(1) != Some('(') => {
                self.bump();
                Some(c.to_string())
            }
            Some(c @ ('&' | '|')) => {
                if self.at(1) == Some(c) {
                    self.adv(2);
                    Some(format!("{c}{c}"))
                } else {
                    self.bump();
                    Some(c.to_string())
                }
            }
            _ => None,
        }
    }

    /// A plain word inside `[[`; `None` at the closing `]]`.
    fn cond_word(&mut self) -> R<Option<Word>> {
        if self.peek_word_text().as_deref() == Some("]]") {
            self.adv(2);
            return Ok(None);
        }
        let s = self.i;
        let w = self.read_word()?;
        if self.i == s {
            return self.err(format!("unexpected {} inside `[[`", self.describe()));
        }
        Ok(Some(w))
    }

    fn parse_cond(&mut self) -> R<Kind> {
        self.adv(2);
        let mut words = Vec::new();
        loop {
            self.skip_blanks();
            let line = self.line;
            match self.peek() {
                None => return self.err("unterminated `[[`"),
                Some('\n') => {
                    self.bump();
                    continue;
                }
                Some(';') => return self.err("unexpected `;` inside `[[`"),
                _ => {}
            }
            if let Some(op) = self.cond_operator() {
                words.push(Word {
                    text: op.clone(),
                    line,
                    parts: vec![Part::Lit {
                        value: op,
                        glob: false,
                    }],
                });
                continue;
            }
            match self.cond_word()? {
                Some(w) => words.push(w),
                None => break,
            }
        }
        Ok(Kind::Cond { words })
    }

    fn at_function_parens(&self) -> bool {
        let mut j = self.i;
        while matches!(self.s.get(j), Some(' ') | Some('\t')) {
            j += 1;
        }
        if self.s.get(j) != Some(&'(') {
            return false;
        }
        j += 1;
        while matches!(self.s.get(j), Some(' ') | Some('\t')) {
            j += 1;
        }
        self.s.get(j) == Some(&')')
    }

    /// `name() body` once `name` has been read.
    fn parse_function_def(&mut self, w: &Word, line: usize) -> R<Node> {
        let name = match w.value() {
            Some(n) if !n.is_empty() => n,
            _ => return self.err("function name must be a plain word"),
        };
        self.skip_blanks();
        self.bump();
        self.expect_char(')')?;
        let kind = self.function_body(name)?;
        Ok(Node {
            kind,
            line,
            end_line: self.line,
        })
    }

    fn reject_fd_redirect(&self, w: &Word) -> R<()> {
        if matches!(self.peek(), Some('<') | Some('>'))
            && w.text.starts_with('{')
            && w.text.ends_with('}')
            && is_name(&w.text[1..w.text.len() - 1])
        {
            return self.err("named file-descriptor redirect `{fd}>` is not modelled");
        }
        Ok(())
    }

    fn at_simple_end(&self) -> bool {
        matches!(
            self.peek(),
            None | Some('\n') | Some(';') | Some('&') | Some('|') | Some(')')
        )
    }

    /// One element of a simple command. `Ok(None)` continues, `Ok(Some(n))`
    /// ends the command (`n` is `Some` for a function definition).
    fn simple_step(&mut self, acc: &mut SimpleAcc, line: usize) -> R<Option<Option<Node>>> {
        self.skip_blanks();
        if self.is_redirect_start() {
            acc.redirects.push(self.parse_redirect()?);
            return Ok(None);
        }
        if self.at_simple_end() {
            return Ok(Some(None));
        }
        if acc.argv.is_empty() {
            if let Some(a) = self.try_assignment()? {
                acc.assigns.push(a);
                return Ok(None);
            }
        }
        let start = self.i;
        let w = self.read_word()?;
        if self.i == start {
            return self.err(format!("unexpected {}", self.describe()));
        }
        let w = self.decl_array_arg(&acc.argv, w)?;
        self.reject_fd_redirect(&w)?;
        let first = acc.argv.is_empty() && acc.assigns.is_empty() && acc.redirects.is_empty();
        if first && self.at_function_parens() {
            return Ok(Some(Some(self.parse_function_def(&w, line)?)));
        }
        acc.argv.push(w);
        Ok(None)
    }

    fn parse_simple(&mut self) -> R<Node> {
        let line = self.line;
        let mut acc = SimpleAcc::default();
        loop {
            match self.simple_step(&mut acc, line)? {
                None => continue,
                Some(None) => break,
                Some(Some(func)) => return Ok(func),
            }
        }
        let SimpleAcc {
            argv,
            assigns,
            redirects,
        } = acc;
        if argv.is_empty() && assigns.is_empty() && redirects.is_empty() {
            return self.err(format!("empty command before {}", self.describe()));
        }
        let analysis = shell_wrap::analyze(&argv)?;
        let simple = Simple {
            argv,
            assignments: assigns,
            redirects,
            wrappers: analysis.wrappers,
            exec_index: analysis.exec_index,
            embedded: analysis.embedded,
        };
        Ok(Node {
            kind: Kind::Simple(simple),
            line,
            end_line: self.line,
        })
    }

    /// At `(`: reads `( word ... )` of an array assignment.
    fn read_array_elements(&mut self) -> R<Vec<Word>> {
        self.bump();
        let mut elements = Vec::new();
        loop {
            self.skip_blanks();
            match self.peek() {
                None => return self.err("unterminated array assignment"),
                Some('\n') => {
                    self.bump();
                    self.read_heredocs()?;
                }
                Some(')') => {
                    self.bump();
                    return Ok(elements);
                }
                _ => {
                    let s = self.i;
                    let w = self.read_word()?;
                    if self.i == s {
                        return self.err(format!(
                            "unexpected {} in array assignment",
                            self.describe()
                        ));
                    }
                    elements.push(w);
                }
            }
        }
    }

    /// `declare -a x=(...)`: the array argument of a declaration builtin.
    fn decl_array_arg(&mut self, argv: &[Word], w: Word) -> R<Word> {
        let decl = argv
            .first()
            .and_then(|a| a.value())
            .map(|n| DECLS.contains(&n.as_str()))
            .unwrap_or(false);
        let head = w.text.trim_end_matches('+');
        let assign_head = head.ends_with('=')
            && head.len() > 1
            && is_name(head[..head.len() - 1].split('[').next().unwrap_or(""));
        if !(decl && self.peek() == Some('(') && assign_head) {
            return Ok(w);
        }
        let start = self.i;
        let elements = self.read_array_elements()?;
        let mut inner = w.parts.clone();
        inner.retain(|p| !matches!(p, Part::Lit { .. }));
        for e in &elements {
            inner.extend(e.parts.iter().cloned());
        }
        let text = format!("{}{}", w.text, self.slice(start));
        let raw = text.clone();
        Ok(Word {
            text,
            line: w.line,
            parts: vec![Part::Param { raw, inner }],
        })
    }

    /// After the name at `j`: skips a `[subscript]`; `Ok(None)` when it is
    /// not a subscript of an assignment.
    fn assignment_subscript_end(&self, j: usize) -> R<Option<usize>> {
        if self.s.get(j) != Some(&'[') {
            return Ok(Some(j));
        }
        let mut k = j + 1;
        while matches!(self.s.get(k), Some(c) if *c != ']' && *c != '\n' && *c != ' ') {
            k += 1;
        }
        if self.s.get(k) != Some(&']') {
            return Ok(None);
        }
        let sub: String = self.s[j..k].iter().collect();
        if sub.contains("$(") || sub.contains('`') {
            return self.err("assignment subscript with a command substitution is not modelled");
        }
        Ok(Some(k + 1))
    }

    fn try_assignment(&mut self) -> R<Option<Assign>> {
        let mut j = self.i;
        match self.s.get(j) {
            Some(c) if c.is_ascii_alphabetic() || *c == '_' => {}
            _ => return Ok(None),
        }
        while matches!(self.s.get(j), Some(c) if c.is_ascii_alphanumeric() || *c == '_') {
            j += 1;
        }
        j = match self.assignment_subscript_end(j)? {
            Some(k) => k,
            None => return Ok(None),
        };
        let name_end = j;
        let append = self.s.get(j) == Some(&'+');
        if append {
            j += 1;
        }
        if self.s.get(j) != Some(&'=') {
            return Ok(None);
        }
        let name: String = self.s[self.i..name_end].iter().collect();
        self.adv(j + 1 - self.i);
        if self.peek() == Some('(') {
            let elements = Some(self.read_array_elements()?);
            return Ok(Some(Assign {
                name,
                append,
                value: None,
                elements,
            }));
        }
        let value = Some(self.read_word()?);
        Ok(Some(Assign {
            name,
            append,
            value,
            elements: None,
        }))
    }

    // ---- redirects and here-documents ------------------------------------

    fn is_redirect_start(&self) -> bool {
        if self.peek() == Some('&') && self.at(1) == Some('>') {
            return true;
        }
        let mut j = self.i;
        while matches!(self.s.get(j), Some(c) if c.is_ascii_digit()) {
            j += 1;
        }
        match self.s.get(j) {
            Some('<') | Some('>') => !(j == self.i && self.s.get(j + 1) == Some(&'(')),
            _ => false,
        }
    }

    /// The delimiter text of a here-document word, quotes removed.
    fn heredoc_delimiter(word: &Word, quoted: bool) -> String {
        if !quoted {
            return word.text.clone();
        }
        let mut d = String::new();
        for p in &word.parts {
            match p {
                Part::Lit { value, .. } | Part::Single { value } => d.push_str(value),
                Part::Double { parts } => {
                    for q in parts {
                        if let Part::Lit { value, .. } = q {
                            d.push_str(value);
                        }
                    }
                }
                other => {
                    let w = Word {
                        text: String::new(),
                        line: 0,
                        parts: vec![other.clone()],
                    };
                    d.push_str(&w.template());
                }
            }
        }
        d
    }

    fn heredoc_redirect(&mut self, fd: Option<String>, op: &str, word: &Word) -> Redirect {
        let quoted = word.text.contains(['\'', '"', '\\']);
        let delim = Self::heredoc_delimiter(word, quoted);
        let slot = Shared(Rc::new(RefCell::new(None)));
        let strip = op == "<<-";
        self.pending.push(Pending {
            delim: delim.clone(),
            strip,
            expand: !quoted,
            slot: slot.clone(),
        });
        let heredoc = Heredoc {
            delimiter: delim,
            expand: !quoted,
            strip_tabs: strip,
            body: slot,
        };
        Redirect {
            fd,
            op: op.to_string(),
            target: None,
            heredoc: Some(heredoc),
        }
    }

    fn parse_redirect(&mut self) -> R<Redirect> {
        let mut fd = String::new();
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            fd.push(self.bump().unwrap_or('0'));
        }
        const OPS: [&str; 12] = [
            "<<<", "<<-", "<<", "<>", "<&", "<", "&>>", "&>", ">>", ">&", ">|", ">",
        ];
        let op = match OPS.iter().find(|o| self.looking_at(o)) {
            Some(o) => *o,
            None => return self.err("expected a redirection operator"),
        };
        self.adv(op.len());
        self.skip_blanks();
        let start = self.i;
        let word = self.read_word()?;
        if self.i == start {
            return self.err(format!("redirection `{op}` has no target"));
        }
        let fd = if fd.is_empty() { None } else { Some(fd) };
        if op == "<<" || op == "<<-" {
            return Ok(self.heredoc_redirect(fd, op, &word));
        }
        Ok(Redirect {
            fd,
            op: op.to_string(),
            target: Some(word),
            heredoc: None,
        })
    }

    fn read_raw_line(&mut self) -> String {
        let mut l = String::new();
        while let Some(c) = self.bump() {
            if c == '\n' {
                break;
            }
            l.push(c);
        }
        l
    }

    /// Called right after a newline was consumed: reads every pending body.
    fn read_heredocs(&mut self) -> R<()> {
        let pending = std::mem::take(&mut self.pending);
        for p in pending {
            let start_line = self.line;
            let mut text = String::new();
            let mut lines = 0;
            loop {
                if self.eof() {
                    return self.err(format!("unterminated here-document `{}`", p.delim));
                }
                let l = self.read_raw_line();
                let cmp = if p.strip {
                    l.trim_start_matches('\t')
                } else {
                    l.as_str()
                };
                if cmp == p.delim {
                    break;
                }
                text.push_str(&l);
                text.push('\n');
                lines += 1;
            }
            let parts = if p.expand {
                let mut sub = Parser::new(&text, start_line);
                sub.read_dq(true)?
            } else {
                Vec::new()
            };
            *p.slot.0.borrow_mut() = Some(HeredocBody {
                line: start_line,
                lines,
                parts,
            });
        }
        Ok(())
    }

    // ---- words ----------------------------------------------------------

    /// One quoting or expansion construct at `c`; `None` is a lone `$`.
    fn word_expansion(&mut self, c: char) -> R<Option<Part>> {
        Ok(Some(match c {
            '<' | '>' => self.parse_procsub()?,
            '\'' => {
                self.bump();
                let mut value = String::new();
                loop {
                    match self.bump() {
                        None => return self.err("unterminated single quote"),
                        Some('\'') => break,
                        Some(ch) => value.push(ch),
                    }
                }
                Part::Single { value }
            }
            '"' => {
                self.bump();
                Part::Double {
                    parts: self.read_dq(false)?,
                }
            }
            '$' => return self.parse_dollar(false),
            _ => self.parse_backtick(false)?,
        }))
    }

    fn read_word(&mut self) -> R<Word> {
        let start = self.i;
        let line = self.line;
        let mut parts: Vec<Part> = Vec::new();
        let mut lit = String::new();
        let mut glob = false;
        let mut open_brace = false;
        while let Some(c) = self.peek() {
            let ends = matches!(c, ' ' | '\t' | '\n' | '|' | '&' | ';' | '(' | ')');
            if ends || (matches!(c, '<' | '>') && self.at(1) != Some('(')) {
                break;
            }
            match c {
                '\\' => {
                    self.bump();
                    match self.bump() {
                        None => lit.push('\\'),
                        Some('\n') => {}
                        Some(ch) => lit.push(ch),
                    }
                }
                '<' | '>' | '\'' | '"' | '$' | '`' => {
                    flush(&mut parts, &mut lit, &mut glob);
                    match self.word_expansion(c)? {
                        Some(p) => parts.push(p),
                        None => lit.push('$'),
                    }
                }
                _ => {
                    self.bump();
                    let word_start = parts.is_empty() && lit.is_empty();
                    glob |= is_glob_char(c, word_start, open_brace, &lit);
                    open_brace |= c == '{';
                    lit.push(c);
                }
            }
        }
        flush(&mut parts, &mut lit, &mut glob);
        Ok(Word {
            text: self.slice(start),
            line,
            parts,
        })
    }

    /// A backslash inside double quotes (the backslash is not yet consumed).
    fn dq_escape(&mut self, heredoc: bool, lit: &mut String) {
        self.bump();
        match self.peek() {
            Some(c) if matches!(c, '$' | '`' | '\\') || (c == '"' && !heredoc) => {
                self.bump();
                lit.push(c);
            }
            Some('\n') => {
                self.bump();
            }
            _ => lit.push('\\'),
        }
    }

    /// Reads double-quoted content. The opening quote is already consumed; for
    /// a here-document body there is no closing quote and EOF ends the text.
    fn read_dq(&mut self, heredoc: bool) -> R<Vec<Part>> {
        let mut parts = Vec::new();
        let mut lit = String::new();
        loop {
            match self.peek() {
                None if heredoc => break,
                None => return self.err("unterminated double quote"),
                Some('"') if !heredoc => {
                    self.bump();
                    break;
                }
                Some('\\') => self.dq_escape(heredoc, &mut lit),
                Some('$') => {
                    push_lit(&mut parts, &mut lit);
                    match self.parse_dollar(true)? {
                        Some(p) => parts.push(p),
                        None => lit.push('$'),
                    }
                }
                Some('`') => {
                    push_lit(&mut parts, &mut lit);
                    parts.push(self.parse_backtick(true)?);
                }
                Some(c) => {
                    self.bump();
                    lit.push(c);
                }
            }
        }
        push_lit(&mut parts, &mut lit);
        Ok(parts)
    }

    fn parse_ansi_c(&mut self, start: usize) -> R<Part> {
        self.adv(2);
        loop {
            match self.bump() {
                None => return self.err("unterminated $'...' string"),
                Some('\\') => {
                    self.bump();
                }
                Some('\'') => break,
                Some(_) => {}
            }
        }
        Ok(Part::AnsiC {
            raw: self.slice(start),
        })
    }

    /// `$name` or `$<special>` (`consume` chars already positioned).
    fn parse_simple_param(&mut self, start: usize, name: bool) -> Part {
        if name {
            self.bump();
            while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '_') {
                self.bump();
            }
        } else {
            self.adv(2);
        }
        Part::Param {
            raw: self.slice(start),
            inner: Vec::new(),
        }
    }

    /// At `$`. `None` means a lone `$` (already consumed) that is a literal.
    fn parse_dollar(&mut self, in_dq: bool) -> R<Option<Part>> {
        let start = self.i;
        match self.at(1) {
            Some('(') => {
                if self.at(2) == Some('(') {
                    if let Some(p) = self.try_arith(start) {
                        return Ok(Some(p));
                    }
                }
                self.adv(2);
                let body = self.parse_paren_body()?;
                Ok(Some(Part::CmdSubst {
                    raw: self.slice(start),
                    backtick: false,
                    body,
                }))
            }
            Some('{') => {
                self.adv(2);
                let inner = self.scan_param_body(in_dq)?;
                Ok(Some(Part::Param {
                    raw: self.slice(start),
                    inner,
                }))
            }
            Some('\'') if !in_dq => Ok(Some(self.parse_ansi_c(start)?)),
            Some('"') if !in_dq => {
                self.adv(2);
                let parts = self.read_dq(false)?;
                Ok(Some(Part::Double { parts }))
            }
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                Ok(Some(self.parse_simple_param(start, true)))
            }
            Some(c)
                if c.is_ascii_digit() || matches!(c, '@' | '*' | '#' | '?' | '-' | '$' | '!') =>
            {
                Ok(Some(self.parse_simple_param(start, false)))
            }
            _ => {
                self.bump();
                Ok(None)
            }
        }
    }

    /// `$((...))`. `None` (cursor restored) means it is really `$( (...) ...)`.
    fn try_arith(&mut self, start: usize) -> Option<Part> {
        let inner = self.scan_arith(3)?;
        Some(Part::Arith {
            raw: self.slice(start),
            inner,
        })
    }

    /// One arithmetic token. `Some(true)` closed, `Some(false)` malformed.
    fn arith_step(&mut self, depth: &mut usize, inner: &mut Vec<Part>) -> R<Option<bool>> {
        match self.peek() {
            None => return Ok(Some(false)),
            Some('(') => {
                *depth += 1;
                self.bump();
            }
            Some(')') if *depth == 0 => {
                if self.at(1) == Some(')') {
                    self.adv(2);
                    return Ok(Some(true));
                }
                return Ok(Some(false));
            }
            Some(')') => {
                *depth -= 1;
                self.bump();
            }
            Some('$') => {
                if let Some(p) = self.parse_dollar(false)? {
                    inner.push(p);
                }
            }
            Some('`') => inner.push(self.parse_backtick(false)?),
            Some(_) => {
                self.bump();
            }
        }
        Ok(None)
    }

    /// Skips `open` opening chars, then reads to the matching `))`. On failure
    /// the cursor is restored and `None` is returned.
    fn scan_arith(&mut self, open: usize) -> Option<Vec<Part>> {
        let save = (self.i, self.line, self.pending.len());
        self.adv(open);
        let mut depth = 0usize;
        let mut inner = Vec::new();
        let outcome: R<bool> = loop {
            match self.arith_step(&mut depth, &mut inner) {
                Ok(None) => continue,
                Ok(Some(done)) => break Ok(done),
                Err(e) => break Err(e),
            }
        };
        match outcome {
            Ok(true) => Some(inner),
            _ => {
                self.i = save.0;
                self.line = save.1;
                self.pending.truncate(save.2);
                None
            }
        }
    }

    fn skip_single_in_param(&mut self) -> R<()> {
        self.bump();
        loop {
            match self.bump() {
                None => return self.err("unterminated single quote in `${`"),
                Some('\'') => return Ok(()),
                Some(_) => {}
            }
        }
    }

    /// After `${`: reads to the matching `}` and returns nested expansions.
    fn scan_param_body(&mut self, in_dq: bool) -> R<Vec<Part>> {
        let mut inner = Vec::new();
        loop {
            match self.peek() {
                None => return self.err("unterminated `${`"),
                Some('}') => {
                    self.bump();
                    return Ok(inner);
                }
                Some('\\') => self.adv(2),
                Some('$') => {
                    if let Some(p) = self.parse_dollar(in_dq)? {
                        inner.push(p);
                    }
                }
                Some('`') => inner.push(self.parse_backtick(in_dq)?),
                Some('\'') if !in_dq => self.skip_single_in_param()?,
                Some('"') => {
                    self.bump();
                    let parts = self.read_dq(false)?;
                    inner.push(Part::Double { parts });
                }
                Some(_) => {
                    self.bump();
                }
            }
        }
    }

    fn parse_backtick(&mut self, in_dq: bool) -> R<Part> {
        let start = self.i;
        let first_line = self.line;
        self.bump();
        let mut buf = String::new();
        loop {
            match self.bump() {
                None => return self.err("unterminated backtick command substitution"),
                Some('`') => break,
                Some('\\') => match self.peek() {
                    Some(c) if matches!(c, '`' | '\\' | '$') || (in_dq && c == '"') => {
                        self.bump();
                        buf.push(c);
                    }
                    _ => buf.push('\\'),
                },
                Some(c) => buf.push(c),
            }
        }
        let mut sub = Parser::new(&buf, first_line);
        let body = sub.parse_program()?;
        Ok(Part::CmdSubst {
            raw: self.slice(start),
            backtick: true,
            body,
        })
    }

    /// After the opening `(`: parses commands up to and including `)`.
    fn parse_paren_body(&mut self) -> R<Vec<Node>> {
        let body = self.parse_list()?;
        self.skip_blanks();
        if self.peek() != Some(')') {
            return self.err(format!(
                "expected `)` to close `(`, found {}",
                self.describe_word()
            ));
        }
        self.bump();
        Ok(body)
    }

    fn parse_procsub(&mut self) -> R<Part> {
        let start = self.i;
        let dir = self.bump().unwrap_or('<').to_string();
        self.bump();
        let body = self.parse_paren_body()?;
        Ok(Part::ProcSubst {
            raw: self.slice(start),
            dir,
            body,
        })
    }
}

fn push_lit(parts: &mut Vec<Part>, lit: &mut String) {
    if !lit.is_empty() {
        parts.push(Part::Lit {
            value: std::mem::take(lit),
            glob: false,
        });
    }
}

fn flush(parts: &mut Vec<Part>, lit: &mut String, glob: &mut bool) {
    if !lit.is_empty() {
        let g = *glob && lit != "[" && lit != "]";
        parts.push(Part::Lit {
            value: std::mem::take(lit),
            glob: g,
        });
    }
    *glob = false;
}

/// Parses a whole script; any construct it cannot model is an `Unparsed` error.
pub fn parse_script(src: &str) -> R<Vec<Node>> {
    if src.contains('\0') {
        return Err(Unparsed {
            line: 1,
            reason: "input contains a NUL byte".to_string(),
        });
    }
    Parser::new(src, 1).parse_program()
}

/// Whether literal char `c` makes the word a glob or brace pattern.
fn is_glob_char(c: char, word_start: bool, open_brace: bool, lit: &str) -> bool {
    (c == '~' && word_start)
        || matches!(c, '*' | '?' | '[')
        || (open_brace && (c == ',' || (c == '.' && lit.ends_with('.'))))
}

#[derive(Default)]
struct SimpleAcc {
    argv: Vec<Word>,
    assigns: Vec<Assign>,
    redirects: Vec<Redirect>,
}
