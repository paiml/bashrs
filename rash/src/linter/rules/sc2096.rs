// SC2096: Redirections override previously specified redirections
//
// When multiple redirections of the same type are specified for the same file
// descriptor, only the last one takes effect. Earlier redirections are silently
// ignored, which is often unintentional.
//
// Examples:
// Bad:
//   command > file1.txt > file2.txt
//   # Only file2.txt is written, file1.txt is untouched
//
//   command 2> err1.log 2> err2.log
//   # Only err2.log gets stderr
//
// Good:
//   command > file1.txt
//   # Single output
//
//   command > stdout.txt 2> stderr.txt
//   # Different streams to different files
//
// bashrs#431: this rule used to count `>` CHARACTERS on the line, so the `>`
// inside `printf '%s\n' "a <N>m b N>0" >&2` was read as a second stdout
// redirection. It now counts redirection OPERATORS, read from tokens. The
// dispatcher hands it the masked source (it is in `QUOTE_SENSITIVE_RULES`), so
// a string literal or a heredoc body is never a token at all, and the lexer
// below steps over the places where `>` is not a redirection in code either:
// `$(( ))`, `(( ))`, `[[ ]]`, `${...}`, `\>` and comments.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

const STDOUT_MESSAGE: &str =
    "Multiple stdout redirections specified. Only the last one will be used, earlier ones are ignored";
const STDERR_MESSAGE: &str =
    "Multiple stderr redirections specified. Only the last one will be used, earlier ones are ignored";
const APPEND_MESSAGE: &str =
    "Multiple append redirections specified. Only the last one will be used, earlier ones are ignored";

/// What a redirection operator does to its file descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    /// `>` and `>|`: open the target, truncating it.
    Truncate,
    /// `>>`: open the target for appending.
    Append,
    /// `>&` and `<&`: copy (or close) a descriptor.
    Dup,
    /// `&>`: stdout and stderr to one file.
    Both,
    /// `&>>`: stdout and stderr appended to one file.
    BothAppend,
    /// `<`, `<<`, `<<-`, `<<<` and `<>`: never an override of an output.
    Input,
}

/// One redirection operator. `fd` is `None` when the descriptor is not a
/// fixed number: `&>`, `{var}>`, or a number too large to be one.
#[derive(Clone, Copy, Debug)]
struct Redir {
    fd: Option<u32>,
    kind: Kind,
}

/// True when two redirections that `counts` selects follow one another with no
/// descriptor copy between them. A copy (`2>&1`, `>&2`) may read the earlier
/// target — `> a 2>&1 > b` still sends stderr to `a` — so it ends the run.
fn overridden(redirs: &[Redir], counts: impl Fn(&Redir) -> bool) -> bool {
    let mut run = 0;
    for redir in redirs {
        if redir.kind == Kind::Dup {
            run = 0;
        } else if counts(redir) {
            run += 1;
            if run > 1 {
                return true;
            }
        }
    }
    false
}

/// The descriptors an append redirection writes to.
fn appended_fds(redir: &Redir) -> [Option<u32>; 2] {
    match redir.kind {
        Kind::Append => [redir.fd, None],
        Kind::BothAppend => [Some(1), Some(2)],
        _ => [None, None],
    }
}

/// Two appends to the SAME descriptor. `>> a 2>> b` appends stdout and stderr
/// to two files and overrides nothing.
fn overrides_append(redirs: &[Redir]) -> bool {
    redirs
        .iter()
        .flat_map(appended_fds)
        .flatten()
        .any(|fd| overridden(redirs, |redir| appended_fds(redir).contains(&Some(fd))))
}

/// The messages one command's redirections earn, in the rule's fixed order.
fn findings(redirs: &[Redir]) -> Vec<&'static str> {
    let stdout = overridden(redirs, |r| r.kind == Kind::Truncate && r.fd == Some(1));
    let stderr = overridden(redirs, |r| {
        r.fd == Some(2) && matches!(r.kind, Kind::Truncate | Kind::Append)
    });
    [
        (stdout, STDOUT_MESSAGE),
        (stderr, STDERR_MESSAGE),
        (overrides_append(redirs), APPEND_MESSAGE),
    ]
    .into_iter()
    .filter_map(|(hit, message)| hit.then_some(message))
    .collect()
}

/// `>>`, `>&`, `>|` or `>`, and how many bytes it takes.
fn out_op(rest: &[u8]) -> (Kind, usize) {
    match rest.get(1) {
        Some(b'>') => (Kind::Append, 2),
        Some(b'&') => (Kind::Dup, 2),
        Some(b'|') => (Kind::Truncate, 2),
        _ => (Kind::Truncate, 1),
    }
}

/// `<<<`, `<<-`, `<<`, `<>`, `<&` or `<`, and how many bytes it takes.
fn in_op(rest: &[u8]) -> (Kind, usize) {
    match (rest.get(1), rest.get(2)) {
        (Some(b'<'), Some(b'<' | b'-')) => (Kind::Input, 3),
        (Some(b'<' | b'>'), _) => (Kind::Input, 2),
        (Some(b'&'), _) => (Kind::Dup, 2),
        _ => (Kind::Input, 1),
    }
}

/// The index just past the bracket that closes the one at `start`, or the end
/// of the line when it does not close on it.
fn skip_balanced(b: &[u8], start: usize, open: u8, close: u8) -> usize {
    let mut depth = 0usize;
    for (offset, &c) in b[start..].iter().enumerate() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return start + offset + 1;
            }
        }
    }
    b.len()
}

/// The redirections of one level of nesting: the line itself, or a `$( )`,
/// `( )`, backtick or process substitution opened on it.
struct Frame {
    backtick: bool,
    redirs: Vec<Redir>,
}

/// Splits one line of (masked) shell into commands and collects each
/// command's redirection operators, in order.
///
/// Quote characters are ordinary word bytes here. That is only right because
/// the dispatcher masks every literal first, which is why this rule is in
/// `QUOTE_SENSITIVE_RULES`: one quoting scanner for the whole linter (GH-272),
/// not a second one in this file.
struct Lexer<'a> {
    b: &'a [u8],
    i: usize,
    /// Where the current word started, if the lexer is inside one.
    word: Option<usize>,
    frames: Vec<Frame>,
    done: Vec<Vec<Redir>>,
}

impl<'a> Lexer<'a> {
    fn new(b: &'a [u8]) -> Self {
        Self {
            b,
            i: 0,
            word: None,
            frames: vec![Frame {
                backtick: false,
                redirs: Vec::new(),
            }],
            done: Vec::new(),
        }
    }

    fn run(mut self) -> Vec<Vec<Redir>> {
        while let Some(&c) = self.b.get(self.i) {
            self.step(c);
        }
        while self.frames.len() > 1 {
            self.pop();
        }
        self.end_command();
        self.done
    }

    /// Every branch advances `i`, so `run` always terminates.
    fn step(&mut self, c: u8) {
        if c.is_ascii_whitespace() {
            self.word = None;
            self.i += 1;
        } else if !self.operator(c) && !self.grouping(c) && !self.structure(c) {
            self.in_word();
            self.i += 1;
        }
    }

    fn peek(&self, ahead: usize) -> Option<u8> {
        self.b.get(self.i + ahead).copied()
    }

    /// Mark the current byte as part of a word, starting one if needed.
    fn in_word(&mut self) {
        if self.word.is_none() {
            self.word = Some(self.i);
        }
    }

    fn operator(&mut self, c: u8) -> bool {
        match c {
            b'>' | b'<' => self.angle(c),
            b'&' => self.ampersand(),
            b'|' | b';' => self.separator(),
            _ => return false,
        }
        true
    }

    fn grouping(&mut self, c: u8) -> bool {
        match c {
            b'(' => self.open_paren(),
            b')' => self.close_paren(),
            b'`' => self.backtick(),
            _ => return false,
        }
        true
    }

    fn structure(&mut self, c: u8) -> bool {
        match c {
            b'\\' => self.escaped(),
            b'$' => self.dollar(),
            b'#' if self.word.is_none() => self.i = self.b.len(),
            b'[' if self.word.is_none() && self.at_double_bracket() => self.double_bracket(),
            _ => return false,
        }
        true
    }

    fn angle(&mut self, c: u8) {
        if self.peek(1) == Some(b'(') {
            // `>(cmd)` / `<(cmd)`: process substitution, an argument.
            self.push(false, 2);
            return;
        }
        let rest = &self.b[self.i..];
        let ((kind, len), default_fd) = if c == b'>' {
            (out_op(rest), 1)
        } else {
            (in_op(rest), 0)
        };
        let fd = self.fd_before(default_fd);
        self.add(Redir { fd, kind }, len);
    }

    fn ampersand(&mut self) {
        match (self.peek(1), self.peek(2)) {
            (Some(b'>'), Some(b'>')) => self.add(
                Redir {
                    fd: None,
                    kind: Kind::BothAppend,
                },
                3,
            ),
            (Some(b'>'), _) => self.add(
                Redir {
                    fd: None,
                    kind: Kind::Both,
                },
                2,
            ),
            // `&&`, or a command sent to the background.
            _ => self.separator(),
        }
    }

    /// The descriptor an operator applies to: a number written directly
    /// before it (`2>`), none for `{var}>`, and the default otherwise — in
    /// `echo file2>out`, `file2` is an argument, not a descriptor.
    fn fd_before(&self, default_fd: u32) -> Option<u32> {
        let Some(start) = self.word else {
            return Some(default_fd);
        };
        let word = &self.b[start..self.i];
        if word.first() == Some(&b'{') && word.last() == Some(&b'}') {
            return None;
        }
        if !word.is_empty() && word.iter().all(u8::is_ascii_digit) {
            return std::str::from_utf8(word).ok()?.parse().ok();
        }
        Some(default_fd)
    }

    fn add(&mut self, redir: Redir, len: usize) {
        if let Some(frame) = self.frames.last_mut() {
            frame.redirs.push(redir);
        }
        self.word = None;
        self.i += len;
    }

    fn separator(&mut self) {
        self.end_command();
        self.word = None;
        self.i += 1;
    }

    fn end_command(&mut self) {
        let redirs = self
            .frames
            .last_mut()
            .map(|frame| std::mem::take(&mut frame.redirs))
            .unwrap_or_default();
        self.finish(redirs);
    }

    fn finish(&mut self, redirs: Vec<Redir>) {
        if !redirs.is_empty() {
            self.done.push(redirs);
        }
    }

    fn push(&mut self, backtick: bool, len: usize) {
        self.frames.push(Frame {
            backtick,
            redirs: Vec::new(),
        });
        self.word = None;
        self.i += len;
    }

    fn pop(&mut self) {
        if let Some(frame) = self.frames.pop() {
            self.finish(frame.redirs);
        }
    }

    fn top_is_backtick(&self) -> bool {
        self.frames.last().is_some_and(|frame| frame.backtick)
    }

    fn escaped(&mut self) {
        self.in_word();
        self.i = (self.i + 2).min(self.b.len());
    }

    fn dollar(&mut self) {
        self.in_word();
        match (self.peek(1), self.peek(2)) {
            // `$(( a > b ))`: a comparison, not a redirection.
            (Some(b'('), Some(b'(')) => self.i = skip_balanced(self.b, self.i + 1, b'(', b')'),
            (Some(b'('), _) => self.push(false, 2),
            // `${x//>/y}`: a pattern, not a redirection.
            (Some(b'{'), _) => self.i = skip_balanced(self.b, self.i + 1, b'{', b'}'),
            _ => self.i += 1,
        }
    }

    fn open_paren(&mut self) {
        if self.word.is_none() && self.peek(1) == Some(b'(') {
            // `(( a > b ))`: an arithmetic command.
            self.i = skip_balanced(self.b, self.i, b'(', b')');
        } else {
            self.push(false, 1);
        }
    }

    fn close_paren(&mut self) {
        if self.frames.len() > 1 && !self.top_is_backtick() {
            self.pop();
            self.word = Some(self.i);
        } else {
            // A `case` pattern's `)`, or a group this line did not open.
            self.end_command();
            self.word = None;
        }
        self.i += 1;
    }

    fn backtick(&mut self) {
        if self.frames.len() > 1 && self.top_is_backtick() {
            self.pop();
            self.word = Some(self.i);
            self.i += 1;
        } else {
            self.push(true, 1);
        }
    }

    fn at_double_bracket(&self) -> bool {
        self.peek(1) == Some(b'[') && self.peek(2).is_none_or(|c| c == b' ' || c == b'\t')
    }

    /// `[[ $a > $b ]]`: a string comparison, not a redirection.
    fn double_bracket(&mut self) {
        let from = self.i + 2;
        self.i = self.b[from..]
            .windows(2)
            .position(|pair| pair == b"]]")
            .map_or(self.b.len(), |at| from + at + 2);
    }
}

/// Create a diagnostic for SC2096
fn create_diagnostic(message: String, line_num: usize, line_len: usize) -> Diagnostic {
    Diagnostic::new(
        "SC2096",
        Severity::Warning,
        message,
        Span::new(line_num, 1, line_num, line_len),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (index, line) in source.lines().enumerate() {
        for redirs in Lexer::new(line.as_bytes()).run() {
            for message in findings(&redirs) {
                result.add(create_diagnostic(
                    message.to_string(),
                    index + 1,
                    line.len(),
                ));
            }
        }
    }

    result
}

#[cfg(test)]
#[path = "sc2096_tests_prop_sc2096.rs"]
mod tests_extracted;
