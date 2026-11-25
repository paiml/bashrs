//! SC2012: Use find instead of ls to better handle non-alphanumeric filenames
//!
//! Using ls in loops or with while read breaks on filenames with newlines or special characters.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! ls | while read file; do ...; done
//! ls -l | wc -l
//! for file in $(ls); do ...; done
//! ```
//!
//! Good:
//! ```bash
//! find . -maxdepth 1 -print0 | while IFS= read -r -d '' file; do ...; done
//! for file in *; do [ -e "$file" ] || continue; ...; done
//! ```
//!
//! # Compatibility
//!
//! Universal

use crate::linter::diagnostic::FixSafetyLevel;
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Keywords after which the next word is still in command position.
const LEADING_KEYWORDS: [&[u8]; 9] = [
    b"if", b"then", b"do", b"else", b"elif", b"while", b"until", b"!", b"time",
];

/// An `ls` command that feeds a pipe: byte offsets of `ls` and of the end of the pipe operator.
struct Hit {
    ls_start: usize,
    pipe_end: usize,
}

/// Per simple-command state while scanning.
struct Cmd {
    /// Next word is in command position.
    start: bool,
    /// Start offset of an `ls` command word seen in this simple command.
    ls: Option<usize>,
    /// Next word is a redirection target, not a command word.
    redir: bool,
}

impl Cmd {
    fn fresh() -> Self {
        Cmd {
            start: true,
            ls: None,
            redir: false,
        }
    }
}

struct Scanner<'a> {
    b: &'a [u8],
    i: usize,
    hits: Vec<Hit>,
}

fn is_blank(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

fn is_word_break(c: u8) -> bool {
    c.is_ascii_whitespace() || b"|&;()<>'\"`\\".contains(&c)
}

fn is_delim(c: u8) -> bool {
    c.is_ascii_whitespace() || b"|&;()<>".contains(&c)
}

fn is_assignment(w: &[u8]) -> bool {
    match w.iter().position(|&c| c == b'=') {
        Some(p) if p > 0 => {
            !w[0].is_ascii_digit()
                && w[..p]
                    .iter()
                    .all(|&c| c.is_ascii_alphanumeric() || c == b'_')
        }
        _ => false,
    }
}

impl<'a> Scanner<'a> {
    fn peek(&self, off: usize) -> Option<u8> {
        self.b.get(self.i + off).copied()
    }

    /// Scan a command list until `close` (or end of input).
    fn list(&mut self, close: u8) {
        let mut st = Cmd::fresh();
        while self.i < self.b.len() {
            let c = self.b[self.i];
            if c == close {
                self.i += 1;
                return;
            }
            self.step(c, &mut st);
        }
    }

    fn step(&mut self, c: u8, st: &mut Cmd) {
        match c {
            b' ' | b'\t' | b'\r' => self.i += 1,
            b'\n' => self.separator(st),
            b'#' if self.at_word_start() => self.skip_comment(),
            b'\'' => self.quoted(st, Self::skip_single),
            b'"' => self.quoted(st, Self::skip_double),
            b'`' => self.substitution(st, 1, b'`'),
            b'\\' => {
                self.i += 2;
                st.start = false;
            }
            b'$' if self.peek(1) == Some(b'(') => self.substitution(st, 2, b')'),
            b'|' => self.pipe(st),
            b'&' if self.peek(1) == Some(b'>') => self.i += 1,
            b';' | b'&' | b'(' | b')' => self.separator(st),
            b'<' | b'>' => self.redirect(st),
            _ => self.word(st),
        }
    }

    fn at_word_start(&self) -> bool {
        self.i == 0 || is_delim(self.b[self.i - 1])
    }

    fn skip_comment(&mut self) {
        while self.i < self.b.len() && self.b[self.i] != b'\n' {
            self.i += 1;
        }
    }

    fn quoted(&mut self, st: &mut Cmd, skip: fn(&mut Self)) {
        skip(self);
        self.finish_word(st);
    }

    fn substitution(&mut self, st: &mut Cmd, open_len: usize, close: u8) {
        self.i += open_len;
        self.list(close);
        self.finish_word(st);
    }

    /// A non-keyword word was consumed.
    fn finish_word(&mut self, st: &mut Cmd) {
        st.start = false;
        st.redir = false;
    }

    fn skip_single(&mut self) {
        self.i += 1;
        while self.i < self.b.len() && self.b[self.i] != b'\'' {
            self.i += 1;
        }
        self.i += 1;
    }

    fn skip_double(&mut self) {
        self.i += 1;
        while self.i < self.b.len() {
            match self.b[self.i] {
                b'"' => {
                    self.i += 1;
                    return;
                }
                b'\\' => self.i += 2,
                b'`' => {
                    self.i += 1;
                    self.list(b'`');
                }
                b'$' if self.peek(1) == Some(b'(') => {
                    self.i += 2;
                    self.list(b')');
                }
                _ => self.i += 1,
            }
        }
    }

    fn separator(&mut self, st: &mut Cmd) {
        self.i += 1;
        *st = Cmd::fresh();
    }

    fn pipe(&mut self, st: &mut Cmd) {
        self.i += 1;
        if self.peek(0) == Some(b'|') {
            self.i += 1;
            *st = Cmd::fresh();
            return;
        }
        if self.peek(0) == Some(b'&') {
            self.i += 1;
        }
        if let Some(ls_start) = st.ls {
            self.hits.push(Hit {
                ls_start,
                pipe_end: self.i,
            });
        }
        *st = Cmd::fresh();
    }

    fn redirect(&mut self, st: &mut Cmd) {
        self.i += 1;
        if self.peek(0) == Some(b'(') {
            self.i += 1;
            self.list(b')');
        } else {
            while matches!(self.peek(0), Some(b'<' | b'>' | b'&' | b'|')) {
                self.i += 1;
            }
            st.redir = true;
        }
    }

    fn word(&mut self, st: &mut Cmd) {
        let s = self.i;
        while self.i < self.b.len() && !is_word_break(self.b[self.i]) {
            if self.b[self.i] == b'$' && self.peek(1) == Some(b'(') {
                break;
            }
            self.skip_param_expansion();
        }
        if self.i == s {
            self.i += 1;
            return;
        }
        self.classify(s, st);
    }

    /// Advance one byte, skipping a whole `${...}` expansion when one starts here.
    fn skip_param_expansion(&mut self) {
        if self.b[self.i] == b'$' && self.peek(1) == Some(b'{') {
            while self.i < self.b.len() && self.b[self.i] != b'}' {
                self.i += 1;
            }
        }
        self.i += 1;
    }

    fn classify(&mut self, s: usize, st: &mut Cmd) {
        let w = &self.b[s..self.i];
        let is_fd = w.iter().all(u8::is_ascii_digit) && matches!(self.peek(0), Some(b'<' | b'>'));
        if is_fd {
            return;
        }
        if st.redir {
            st.redir = false;
            return;
        }
        let keeps_start = st.start && (LEADING_KEYWORDS.contains(&w) || is_assignment(w));
        if keeps_start {
            return;
        }
        if st.start && w == b"ls" && self.peek(0).is_none_or(is_delim) {
            st.ls = Some(s);
        }
        st.start = false;
    }

    /// End offset of `while read` directly after `from`, if present.
    fn while_read_end(&self, from: usize) -> Option<usize> {
        let mut p = self.skip_blanks(from);
        p = self.keyword_end(p, b"while")?;
        p = self.skip_blanks(p);
        self.keyword_end(p, b"read")
    }

    fn skip_blanks(&self, mut p: usize) -> usize {
        while self.b.get(p).copied().is_some_and(is_blank) {
            p += 1;
        }
        p
    }

    fn keyword_end(&self, p: usize, kw: &[u8]) -> Option<usize> {
        let end = p + kw.len();
        if self.b.get(p..end)? != kw {
            return None;
        }
        match self.b.get(end) {
            None => Some(end),
            Some(&c) if is_delim(c) => Some(end),
            Some(_) => None,
        }
    }
}

fn line_col(source: &str, offset: usize) -> (usize, usize) {
    let before = &source[..offset];
    let line = before.matches('\n').count() + 1;
    let line_start = before.rfind('\n').map_or(0, |p| p + 1);
    (line, source[line_start..offset].chars().count() + 1)
}

fn make_diagnostic(source: &str, scanner: &Scanner<'_>, hit: &Hit) -> Diagnostic {
    let read_end = scanner.while_read_end(hit.pipe_end);
    let end = read_end.unwrap_or(hit.pipe_end);
    let (start_line, start_col) = line_col(source, hit.ls_start);
    let (end_line, end_col) = line_col(source, end);
    let fix = read_end.map(|_| Fix {
        replacement: "find . -maxdepth 1 -print0 | while IFS= read -r -d '' file".to_string(),
        safety_level: FixSafetyLevel::SafeWithAssumptions,
        assumptions: vec!["find -print0 is available".to_string()],
        suggested_alternatives: vec![
            "for file in *; do [ -e \"$file\" ] || continue; ...; done".to_string()
        ],
    });
    Diagnostic {
        code: "SC2012".to_string(),
        severity: Severity::Warning,
        message: "Use find instead of ls to better handle non-alphanumeric filenames.".to_string(),
        span: Span {
            start_line,
            end_line,
            start_col,
            end_col,
        },
        fix,
    }
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let mut scanner = Scanner {
        b: source.as_bytes(),
        i: 0,
        hits: Vec::new(),
    };
    scanner.list(0);
    for hit in &scanner.hits {
        result
            .diagnostics
            .push(make_diagnostic(source, &scanner, hit));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(source: &str) -> usize {
        check(source).diagnostics.len()
    }

    #[test]
    fn test_sc2012_basic() {
        let source = "ls | while read file; do echo $file; done";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2012");
    }

    #[test]
    fn test_sc2012_find_usage() {
        let source = "find . -print0 | while IFS= read -r -d '' file; do echo $file; done";
        let result = check(source);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2012_ls_long_wc() {
        assert_eq!(count("ls -l | wc -l"), 1);
        assert_eq!(count("ls | wc -l"), 1);
        assert_eq!(count("ls -l|wc -l"), 1);
        assert_eq!(count("ls | grep x"), 1);
    }

    #[test]
    fn test_sc2012_ls_while_read_variants() {
        assert_eq!(count("ls -l | while read -r l; do :; done"), 1);
        assert_eq!(count("ls\t|\twhile\tread f; do :; done"), 1);
    }

    #[test]
    fn test_sc2012_whitespace_and_indent() {
        assert_eq!(count("  ls | wc -l"), 1);
        assert_eq!(count("\tls\t|\twc -l"), 1);
        assert_eq!(count("ls   -l   |   wc -l"), 1);
    }

    #[test]
    fn test_sc2012_span_columns_chars() {
        let r = check("\tls | wc -l");
        assert_eq!(r.diagnostics[0].span.start_col, 2);
        assert_eq!(r.diagnostics[0].span.start_line, 1);
        let r = check("a=1\n  ls | wc -l");
        assert_eq!(r.diagnostics[0].span.start_line, 2);
        assert_eq!(r.diagnostics[0].span.start_col, 3);
    }

    #[test]
    fn test_sc2012_no_pipe_not_flagged() {
        assert_eq!(count("ls"), 0);
        assert_eq!(count("ls -l"), 0);
        assert_eq!(count("ls > out"), 0);
        assert_eq!(count("ls foo; wc -l"), 0);
        assert_eq!(count("ls && wc -l"), 0);
        assert_eq!(count("ls a || wc -l"), 0);
        assert_eq!(count("for f in $(ls); do :; done"), 0);
    }

    #[test]
    fn test_sc2012_not_command_position() {
        assert_eq!(count("echo ls | wc -l"), 0);
        assert_eq!(count("echo ls | while read x; do :; done"), 0);
        assert_eq!(count("sudo ls | wc -l"), 0);
        assert_eq!(count("lsof | wc -l"), 0);
        assert_eq!(count("ls.sh | wc -l"), 0);
        assert_eq!(count("my-ls | wc -l"), 0);
        assert_eq!(count("echo \"ls | wc -l\""), 0);
        assert_eq!(count("echo 'ls | wc -l'"), 0);
        assert_eq!(count("ls -l # | wc"), 0);
        assert_eq!(count("# ls | wc -l"), 0);
    }

    #[test]
    fn test_sc2012_quoted_pipe_in_args() {
        assert_eq!(count("ls \"a | b\"; wc"), 0);
        assert_eq!(count("ls -l \"a|b\" | wc -l"), 1);
        assert_eq!(count("ls 'a|b' | wc -l"), 1);
        assert_eq!(count("ls ./x\\ y | wc -l"), 1);
        assert_eq!(count("ls a\\|b; wc"), 0);
    }

    #[test]
    fn test_sc2012_substitution_in_args_no_leak() {
        // pipe inside $(...) belongs to the inner command, not to ls
        assert_eq!(count("ls $(echo a | tr a b); wc"), 0);
        assert_eq!(count("ls $(echo a | tr a b) | wc -l"), 1);
        assert_eq!(count("ls \"$(pwd)\" | wc -l"), 1);
        assert_eq!(count("ls `pwd` | wc -l"), 1);
        assert_eq!(count("ls $x | wc -l"), 1);
        assert_eq!(count("ls ${x:-a} | wc -l"), 1);
        assert_eq!(count("ls `echo a | tr a b`; wc"), 0);
    }

    #[test]
    fn test_sc2012_redirections() {
        assert_eq!(count("ls 2>&1 | wc -l"), 1);
        assert_eq!(count("ls > out | wc -l"), 1);
        assert_eq!(count("ls < in | wc -l"), 1);
        assert_eq!(count("ls &> out | wc -l"), 1);
        assert_eq!(count("ls |& wc -l"), 1);
        assert_eq!(count("ls >out; wc"), 0);
        assert_eq!(count(">out ls | wc -l"), 1);
        assert_eq!(count("2>err ls | wc -l"), 1);
        assert_eq!(count("ls > >(cat) | wc -l"), 1);
    }

    #[test]
    fn test_sc2012_nested_contexts() {
        assert_eq!(count("x=$(ls | wc -l)"), 1);
        assert_eq!(count("x=`ls | wc -l`"), 1);
        assert_eq!(count("x=\"$(ls | wc -l)\""), 1);
        assert_eq!(count("cd x && ls | wc -l"), 1);
        assert_eq!(count("(ls | wc -l)"), 1);
        assert_eq!(count("if ls | wc -l; then :; fi"), 1);
        assert_eq!(count("! ls | wc -l"), 1);
        assert_eq!(count("time ls | wc -l"), 1);
        assert_eq!(count("foo | ls | wc -l"), 1);
        assert_eq!(count("ls | sort | wc -l"), 1);
        assert_eq!(count("ls || ls | wc -l"), 1);
        assert_eq!(count("FOO=1 ls | wc -l"), 1);
    }

    #[test]
    fn test_sc2012_multiline() {
        assert_eq!(count("echo a\nls -l | wc -l\necho b\n"), 1);
        assert_eq!(count("echo \"a\nls | wc -l\"\n"), 0);
        assert_eq!(count("ls a \\\n  | wc -l"), 1);
        assert_eq!(count("ls | wc -l\nls | wc -l"), 2);
    }

    #[test]
    fn test_sc2012_fix_only_for_while_read() {
        let r = check("ls | while read file; do :; done");
        assert!(r.diagnostics[0].fix.is_some());
        let r = check("ls -l | wc -l");
        assert!(r.diagnostics[0].fix.is_none());
    }

    #[test]
    fn test_sc2012_unterminated_inputs_do_not_panic() {
        for s in [
            "ls \"abc", "ls 'abc", "ls $(", "ls `", "ls \\", "ls |", "$(", "\"$(",
        ] {
            let _ = check(s);
        }
    }
}
