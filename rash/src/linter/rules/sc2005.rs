// SC2005: Useless echo instead of bare command
//
// `echo $(cmd)` prints what `cmd` prints (minus trailing newlines and word
// splitting), so it can usually be written `cmd`. ShellCheck 0.8.0 reports this
// when `echo`, as the command's own first word, has exactly ONE argument and
// that argument is exactly one command substitution -- `$(...)`, `"$(...)"`,
// `` `...` `` or `` "`...`" `` -- whose body is a single command (a pipeline,
// `&&` list or compound command counts as one).
//
// The `$(echo x)` form is a different check, ShellCheck SC2116 (see sc2116.rs).
//
// Examples:
// Bad:
//   echo $(ls)                      // just: ls
//   echo "$(date +%s)"              // just: date +%s
//   echo `ls`                       // just: ls
//
// Good:
//   echo $(ls) foo                  // more than one argument
//   echo "$(ls) x"                  // the argument is not only the substitution
//   echo -n $(ls)                   // flag is a second argument
//   echo $(cd d; ls)                // two commands in the body
//   /bin/echo $(ls)                 // not the shell builtin by that name
//
// Not detected: a `case` pattern inside the body (the word lexer ends `$(` at
// the pattern's `)`), and (the word cannot be seen whole on one physical line) a
// substitution whose closing `)` or backtick is on a later line.

use crate::linter::shell_words::{
    simple_commands, Expansion, ExpansionKind, ShellWord, SimpleCommand, WordRole,
};
use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Reserved words after which a command follows. Wrappers (`sudo`, `env`, ...)
/// are deliberately absent: ShellCheck does not see through them for SC2005.
const LEADING_KEYWORDS: &[&str] = &[
    "if", "then", "else", "elif", "while", "until", "do", "!", "{",
];

/// Is this word exactly the literal text `name`, with no expansion in it?
fn is_literal(word: &ShellWord, name: &str) -> bool {
    word.literal == name && word.expansions.is_empty() && word.substitutions.is_empty()
}

/// The command's own first word: not an assignment, a redirect target or a
/// leading reserved word.
fn head_index(cmd: &SimpleCommand) -> Option<usize> {
    cmd.words.iter().position(|w| match w.role {
        WordRole::AssignPrefix | WordRole::RedirectTarget => false,
        _ => !LEADING_KEYWORDS.iter().any(|k| is_literal(w, k)),
    })
}

/// A file-descriptor number glued to a redirection (`2` in `2>err`, `2>&1`).
/// The lexer leaves it as its own word, so the line decides.
fn is_fd_prefix(word: &ShellWord, line: &str) -> bool {
    let end = word.col - 1 + word.raw.len();
    !word.raw.is_empty()
        && word.raw.bytes().all(|c| c.is_ascii_digit())
        && matches!(line.as_bytes().get(end), Some(b'<' | b'>'))
}

/// The one argument of `echo`, when it has exactly one (redirections are not
/// arguments).
fn sole_echo_argument<'a>(cmd: &'a SimpleCommand, line: &str) -> Option<&'a ShellWord> {
    let head = head_index(cmd)?;
    if !is_literal(cmd.words.get(head)?, "echo") {
        return None;
    }
    let mut args = cmd.words[head + 1..]
        .iter()
        .filter(|w| w.role != WordRole::RedirectTarget && !is_fd_prefix(w, line));
    let only = args.next()?;
    args.next().is_none().then_some(only)
}

/// The command substitution that makes up the WHOLE word, optionally inside one
/// pair of double quotes.
fn whole_substitution(word: &ShellWord) -> Option<&Expansion> {
    let len = word.raw.len();
    let quoted = word.raw.len() >= 2 && word.raw.starts_with('"') && word.raw.ends_with('"');
    let (from, to) = if quoted {
        (word.col + 1, word.col + len - 1)
    } else {
        (word.col, word.col + len)
    };
    word.substitutions.iter().find(|sub| {
        matches!(sub.kind, ExpansionKind::CommandSubstitution { .. })
            && sub.col == from
            && sub.end_col == to
    })
}

/// The text between the delimiters of a whole-word command substitution, when
/// `text` really is one `$( ... )` or one `` ` ... ` `` and nothing else.
fn substitution_body(text: &str) -> Option<&str> {
    if let Some(rest) = text.strip_prefix("$(") {
        let close = skip_balanced(text.as_bytes(), 1, b'(', b')')?;
        return (close == text.len()).then(|| &rest[..rest.len() - 1]);
    }
    let inner = text.strip_prefix('`')?.strip_suffix('`')?;
    // A backslash-escaped backtick nests a substitution; not analysed.
    (!inner.contains("\\`") && !inner.contains('`')).then_some(inner)
}

/// Index just past the `close` that matches the `open` at `from`, skipping
/// quoted text and escapes.
fn skip_balanced(b: &[u8], from: usize, open: u8, close: u8) -> Option<usize> {
    let (mut i, mut depth) = (from, 0usize);
    while let Some(&c) = b.get(i) {
        match c {
            b'\\' => i += 2,
            b'\'' => i = skip_single(b, i)?,
            b'"' => i = skip_double(b, i)?,
            b'`' => i = skip_backtick(b, i)?,
            _ => {
                depth = depth_after(c, open, close, depth);
                i += 1;
                if c == close && depth == 0 {
                    return Some(i);
                }
            }
        }
    }
    None
}

fn depth_after(c: u8, open: u8, close: u8, depth: usize) -> usize {
    if c == open {
        depth + 1
    } else if c == close {
        depth.saturating_sub(1)
    } else {
        depth
    }
}

/// Index past the `'` that closes the one at `i`.
fn skip_single(b: &[u8], i: usize) -> Option<usize> {
    let end = b.get(i + 1..)?.iter().position(|&c| c == b'\'')?;
    Some(i + 1 + end + 1)
}

/// Index past the `"` that closes the one at `i`.
fn skip_double(b: &[u8], i: usize) -> Option<usize> {
    let mut k = i + 1;
    while let Some(&c) = b.get(k) {
        match c {
            b'\\' => k += 2,
            b'"' => return Some(k + 1),
            b'$' if b.get(k + 1) == Some(&b'(') => k = skip_balanced(b, k + 1, b'(', b')')?,
            b'`' => k = skip_backtick(b, k)?,
            _ => k += 1,
        }
    }
    None
}

/// Index past the backtick that closes the one at `i`.
fn skip_backtick(b: &[u8], i: usize) -> Option<usize> {
    let mut k = i + 1;
    while let Some(&c) = b.get(k) {
        match c {
            b'\\' => k += 2,
            b'`' => return Some(k + 1),
            _ => k += 1,
        }
    }
    None
}

/// How the scanner reads one command-substitution body: counts the top-level
/// commands, so `a; b`, `a & b` and a newline make two while a pipeline, an
/// `&&` list and `if ...; then ...; fi` make one.
struct BodyScan<'a> {
    b: &'a [u8],
    i: usize,
    /// Open `( ... )` groups.
    parens: usize,
    /// Open `if`/`case`/loop/`{ }` compounds.
    compounds: usize,
    count: usize,
    /// A command has started since the last separator.
    started: bool,
    /// The next word is in command position.
    cmd_start: bool,
    /// A `|`, `&&` or `||` still waits for its right-hand side.
    dangling: bool,
    /// A construct that is a syntax error (`;;` outside a `case`).
    invalid: bool,
    /// Text of the word being read, and whether it is entirely unquoted.
    word: Vec<u8>,
    in_word: bool,
    plain: bool,
}

/// How many top-level commands `body` holds; `None` when it cannot be told
/// (unbalanced, a comment, a dangling operator), which ShellCheck also does not
/// report.
fn command_count(body: &str) -> Option<usize> {
    let mut scan = BodyScan::new(body.as_bytes());
    while scan.i < scan.b.len() {
        scan.step()?;
    }
    scan.end_word();
    scan.finish_command();
    let complete = scan.parens == 0 && scan.compounds == 0;
    (complete && !scan.dangling && !scan.invalid).then_some(scan.count)
}

impl<'a> BodyScan<'a> {
    fn new(b: &'a [u8]) -> Self {
        Self {
            b,
            i: 0,
            parens: 0,
            compounds: 0,
            count: 0,
            started: false,
            cmd_start: true,
            dangling: false,
            invalid: false,
            word: Vec::new(),
            in_word: false,
            plain: true,
        }
    }

    fn at(&self, k: usize) -> Option<u8> {
        self.b.get(k).copied()
    }

    fn step(&mut self) -> Option<()> {
        let c = self.at(self.i)?;
        match c {
            b' ' | b'\t' | b'\r' => self.blank(),
            b'\n' => self.newline(),
            b'#' if !self.in_word => return None,
            b'\\' | b'\'' | b'"' | b'`' | b'$' => return self.quoted(c),
            b'(' | b')' => return self.paren(c),
            b';' | b'&' | b'|' => self.operator(c),
            _ => self.word_byte(c),
        }
        Some(())
    }

    fn blank(&mut self) {
        self.end_word();
        self.i += 1;
    }

    fn newline(&mut self) {
        self.end_word();
        self.separator();
        self.i += 1;
    }

    fn word_byte(&mut self, c: u8) {
        self.in_word = true;
        self.started = true;
        self.dangling = false;
        self.word.push(c);
        self.i += 1;
    }

    /// A construct whose inside is opaque to command counting.
    fn quoted(&mut self, c: u8) -> Option<()> {
        let next = match c {
            b'\\' if self.at(self.i + 1) == Some(b'\n') => {
                self.i += 2;
                return Some(());
            }
            b'\\' => self.i + 2,
            b'\'' => skip_single(self.b, self.i)?,
            b'"' => skip_double(self.b, self.i)?,
            b'`' => skip_backtick(self.b, self.i)?,
            _ => return self.dollar(),
        };
        self.opaque(next);
        Some(())
    }

    fn dollar(&mut self) -> Option<()> {
        let next = match self.at(self.i + 1) {
            Some(b'(') => skip_balanced(self.b, self.i + 1, b'(', b')')?,
            Some(b'{') => skip_balanced(self.b, self.i + 1, b'{', b'}')?,
            _ => {
                self.word_byte(b'$');
                return Some(());
            }
        };
        self.opaque(next);
        Some(())
    }

    fn opaque(&mut self, next: usize) {
        self.in_word = true;
        self.started = true;
        self.dangling = false;
        self.plain = false;
        self.i = next;
    }

    fn paren(&mut self, c: u8) -> Option<()> {
        self.end_word();
        self.i += 1;
        if c == b'(' {
            self.parens += 1;
            self.started = true;
            self.cmd_start = true;
        } else if self.parens > 0 {
            self.parens -= 1;
            self.cmd_start = false;
        } else if self.compounds > 0 {
            // The `)` ending a `case` pattern.
            self.cmd_start = true;
        } else {
            return None;
        }
        Some(())
    }

    fn operator(&mut self, c: u8) {
        self.end_word();
        let (prev, next) = (
            self.i.checked_sub(1).and_then(|k| self.at(k)),
            self.at(self.i + 1),
        );
        match (c, next) {
            // `;;` and `;&` end a `case` clause.
            (b';', Some(b';' | b'&')) => {
                self.invalid |= self.compounds == 0;
                self.skip_operator(2, false)
            }
            (b';', _) => {
                self.separator();
                self.i += 1;
            }
            // `>&`, `<&` and `&>` are redirections, not control operators.
            (b'&', n) if matches!(prev, Some(b'>' | b'<')) || n == Some(b'>') => {
                self.word_byte(b'&')
            }
            (b'&', Some(b'&')) => self.skip_operator(2, true),
            (b'&', _) => {
                self.separator();
                self.i += 1;
            }
            (_, Some(b'|' | b'&')) => self.skip_operator(2, true),
            _ => self.skip_operator(1, true),
        }
    }

    fn skip_operator(&mut self, len: usize, dangling: bool) {
        self.i += len;
        self.cmd_start = true;
        self.dangling = dangling;
    }

    fn separator(&mut self) {
        if self.parens == 0 && self.compounds == 0 {
            self.finish_command();
        }
        self.cmd_start = true;
    }

    fn finish_command(&mut self) {
        if self.started {
            self.count += 1;
            self.started = false;
        }
    }

    /// A word just ended: if it is a compound-command keyword in command
    /// position, track it.
    fn end_word(&mut self) {
        if !self.in_word {
            return;
        }
        let at_cmd = self.cmd_start;
        let plain_word = std::mem::take(&mut self.word);
        let keyword = (at_cmd && self.plain).then_some(plain_word.as_slice());
        self.in_word = false;
        self.plain = true;
        self.cmd_start = false;
        let Some(kw) = keyword else { return };
        match kw {
            b"if" | b"while" | b"until" | b"{" => {
                self.compounds += 1;
                self.cmd_start = true;
            }
            b"case" | b"for" | b"select" => self.compounds += 1,
            b"fi" | b"esac" | b"done" | b"}" => self.compounds = self.compounds.saturating_sub(1),
            b"then" | b"do" | b"else" | b"elif" | b"!" | b"time" => self.cmd_start = true,
            _ => {}
        }
    }
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cmd in simple_commands(line) {
            let Some(sub) = sole_echo_argument(&cmd, line).and_then(whole_substitution) else {
                continue;
            };
            let single = substitution_body(&sub.text).and_then(command_count) == Some(1);
            if single {
                result.add(Diagnostic::new(
                    "SC2005",
                    Severity::Info,
                    "Useless echo? Instead of 'echo $(cmd)', just use 'cmd'".to_string(),
                    Span::new(line_num, sub.col, line_num, sub.end_col),
                ));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(code: &str) -> usize {
        check(code).diagnostics.len()
    }

    // ===== the reported form: echo $(cmd) =====

    #[test]
    fn test_sc2005_echo_command_substitution() {
        let result = check("echo $(ls)");
        assert_eq!(result.diagnostics.len(), 1);
        let d = &result.diagnostics[0];
        assert_eq!(d.code, "SC2005");
        assert_eq!(d.severity, Severity::Info);
        assert!(d.message.contains("Useless echo"));
        // Span names the substitution: columns 6..=10, end exclusive.
        assert_eq!((d.span.start_col, d.span.end_col), (6, 11));
    }

    #[test]
    fn test_sc2005_double_quoted_and_backtick_forms() {
        assert_eq!(count(r#"echo "$(ls)""#), 1);
        assert_eq!(count("echo `ls`"), 1);
        assert_eq!(count(r#"echo "`ls`""#), 1);
    }

    #[test]
    fn test_sc2005_span_of_quoted_and_backtick_forms() {
        let d = &check(r#"echo "$(ls)""#).diagnostics[0];
        assert_eq!((d.span.start_col, d.span.end_col), (7, 12));
        let d = &check("echo `ls`").diagnostics[0];
        assert_eq!((d.span.start_col, d.span.end_col), (6, 10));
    }

    #[test]
    fn test_sc2005_whitespace_variants() {
        assert_eq!(count("    echo $(ls)"), 1);
        assert_eq!(count("\techo $(ls)"), 1);
        assert_eq!(count("echo\t$(ls)"), 1);
        assert_eq!(count("echo     $(ls)"), 1);
        assert_eq!(count("echo $( ls )"), 1);
        assert_eq!(count("echo $(\tls\t)"), 1);
    }

    #[test]
    fn test_sc2005_single_command_bodies() {
        assert_eq!(count("echo $(ls -l)"), 1);
        assert_eq!(count("echo $(ls | wc -l)"), 1);
        assert_eq!(count("echo $(a && b)"), 1);
        assert_eq!(count("echo $(a || b)"), 1);
        assert_eq!(count("echo $(! a)"), 1);
        assert_eq!(count("echo $(ls;)"), 1);
        assert_eq!(count("echo $(ls &)"), 1);
        assert_eq!(count("echo $(ls >f)"), 1);
        assert_eq!(count("echo $(ls 2>&1)"), 1);
        assert_eq!(count("echo $(ls &>f)"), 1);
        assert_eq!(count("echo $(f $(g))"), 1);
    }

    #[test]
    fn test_sc2005_compound_bodies_are_one_command() {
        assert_eq!(count("echo $(if a; then b; fi)"), 1);
        // Known gap: the lexer closes `$(` at the `)` of the case pattern.
        assert_eq!(count("echo $(case x in a) b;; esac)"), 0);
        assert_eq!(count("echo $(for i in a; do b; done)"), 1);
        assert_eq!(count("echo $(while a; do b; done)"), 1);
        assert_eq!(count("echo $( (a;b) )"), 1);
        assert_eq!(count("echo $({ a;b; })"), 1);
    }

    #[test]
    fn test_sc2005_separators_inside_quotes_do_not_count() {
        assert_eq!(count(r#"echo $(ls "a;b")"#), 1);
        assert_eq!(count("echo $(ls 'a;b')"), 1);
        assert_eq!(count(r"echo $(ls \; x)"), 1);
        assert_eq!(count("echo $(ls ${x:-a;b})"), 1);
        assert_eq!(count("echo $(ls $(a;b))"), 1);
        assert_eq!(count("echo $(ls `a;b`)"), 1);
        assert_eq!(count("echo $(ls $# $?)"), 1);
    }

    #[test]
    fn test_sc2005_keyword_not_in_command_position_is_a_word() {
        assert_eq!(count("echo $(ls if fi)"), 1);
        assert_eq!(count("echo $(ls done; x)"), 0);
    }

    #[test]
    fn test_sc2005_context_of_the_echo() {
        assert_eq!(count("echo $(ls) | cat"), 1);
        assert_eq!(count("echo $(ls) && ls"), 1);
        assert_eq!(count("echo $(ls) 2>&1"), 1);
        assert_eq!(count("echo $(ls) >f 2>&1"), 1);
        assert_eq!(count("echo $(ls) 2> err"), 1);
        assert_eq!(count("echo $(ls) 2>err"), 1);
        assert_eq!(count("echo $(ls) &>err"), 1);
        assert_eq!(count("echo $(ls) > out"), 1);
        assert_eq!(count("echo $(ls) < in"), 1);
        assert_eq!(count("echo $(ls) x 2>&1"), 0);
        assert_eq!(count("echo 2 $(ls)"), 0);
        assert_eq!(count(r#"echo "2>&1" $(ls)"#), 0);
        assert_eq!(count("echo >f $(ls)"), 1);
        assert_eq!(count("FOO=1 echo $(ls)"), 1);
        assert_eq!(count("if echo $(ls); then :; fi"), 1);
        assert_eq!(count("! echo $(ls)"), 1);
        assert_eq!(count("x=$(echo $(ls))"), 1);
        assert_eq!(count("echo $(ls) # $(a;b)"), 1);
    }

    #[test]
    fn test_sc2005_two_echoes_on_one_line() {
        assert_eq!(count("echo $(a); echo $(b)"), 2);
    }

    // ===== not reported =====

    #[test]
    fn test_sc2005_other_argument_shapes_not_reported() {
        assert_eq!(count("echo $(ls) foo"), 0);
        assert_eq!(count("echo foo $(ls)"), 0);
        assert_eq!(count(r#"echo "$(ls)" "x""#), 0);
        assert_eq!(count(r#"echo "$(ls) x""#), 0);
        assert_eq!(count(r#"echo "$(ls)"x"#), 0);
        assert_eq!(count(r#"echo x"$(ls)""#), 0);
        assert_eq!(count(r#"echo "a$(ls)""#), 0);
        assert_eq!(count("echo $(ls) $(ls)"), 0);
        assert_eq!(count("echo $(a)$(b)"), 0);
        assert_eq!(count("echo -n $(ls)"), 0);
        assert_eq!(count("echo"), 0);
    }

    #[test]
    fn test_sc2005_multi_command_bodies_not_reported() {
        assert_eq!(count("echo $(cd d; ls)"), 0);
        assert_eq!(count("echo $(a & b)"), 0);
        assert_eq!(count("echo $(a;b)"), 0);
        assert_eq!(count("echo $(a ; b)"), 0);
        assert_eq!(count("echo $(a;;b)"), 0);
        assert_eq!(count("echo `a; b`"), 0);
    }

    #[test]
    fn test_sc2005_unreadable_bodies_not_reported() {
        assert_eq!(count("echo $()"), 0);
        assert_eq!(count("echo $(  )"), 0);
        assert_eq!(count("echo $(ls #c)"), 0);
        assert_eq!(count("echo $(ls | )"), 0);
        assert_eq!(count("echo $(a &&)"), 0);
        assert_eq!(count("echo $(if a; then b)"), 0);
        assert_eq!(count("echo $(ls"), 0);
        assert_eq!(count("echo `ls"), 0);
    }

    #[test]
    fn test_sc2005_multiline_substitution_not_seen_whole() {
        assert_eq!(count("echo $(\n  ls\n)"), 0);
        assert_eq!(count("echo $(a\nb)"), 0);
    }

    #[test]
    fn test_sc2005_not_arithmetic_or_parameter() {
        assert_eq!(count("echo $((1+2))"), 0);
        assert_eq!(count("echo ${x}"), 0);
        assert_eq!(count("echo $x"), 0);
    }

    #[test]
    fn test_sc2005_only_the_builtin_name_echo() {
        assert_eq!(count("/bin/echo $(ls)"), 0);
        assert_eq!(count("command echo $(ls)"), 0);
        assert_eq!(count("sudo echo $(ls)"), 0);
        assert_eq!(count("env echo $(ls)"), 0);
        assert_eq!(count("nohup echo $(ls)"), 0);
        assert_eq!(count("xargs echo $(ls)"), 0);
        assert_eq!(count("exec echo $(ls)"), 0);
        assert_eq!(count("ECHO $(ls)"), 0);
        assert_eq!(count("echoo $(ls)"), 0);
        assert_eq!(count("$e $(ls)"), 0);
        assert_eq!(count("printf %s $(ls)"), 0);
        assert_eq!(count("foo echo $(ls)"), 0);
    }

    #[test]
    fn test_sc2005_comments_skipped() {
        assert_eq!(count("# echo $(ls)"), 0);
        assert_eq!(count("  # echo $(ls)"), 0);
    }

    // ===== $(echo x) is SC2116, never SC2005 =====

    #[test]
    fn test_sc2005_dollar_echo_form_is_sc2116_territory() {
        assert_eq!(count("result=$(echo $value)"), 0);
        assert_eq!(count(r#"output=$(echo "$var")"#), 0);
        assert_eq!(count("file=$(echo /path/to/file)"), 0);
        assert_eq!(count("for i in $(echo a); do :; done"), 0);
        assert_eq!(count("x=`echo a`"), 0);
        assert_eq!(count("cmd $(echo a b)"), 0);
    }

    #[test]
    fn test_sc2005_echo_of_echo_substitution_reports_once() {
        // ShellCheck: SC2005 on the outer echo (and SC2116 on the inner).
        assert_eq!(count("echo $(echo a)"), 1);
    }

    // ===== unit tests of the helpers =====

    #[test]
    fn test_sc2005_command_count_direct() {
        assert_eq!(command_count(""), Some(0));
        assert_eq!(command_count("ls"), Some(1));
        assert_eq!(command_count("a\nb"), Some(2));
        assert_eq!(command_count("a\n\nb"), Some(2));
        assert_eq!(command_count("a \\\n b"), Some(1));
        assert_eq!(command_count("a;b;c"), Some(3));
        assert_eq!(command_count("(a;b)"), Some(1));
        assert_eq!(command_count("a)"), None);
        assert_eq!(command_count("(a"), None);
        assert_eq!(command_count("a |"), None);
        assert_eq!(command_count("a 'b"), None);
        assert_eq!(command_count("a \"b"), None);
        assert_eq!(command_count("a $(b"), None);
        assert_eq!(command_count("a ${b"), None);
        assert_eq!(command_count("a `b"), None);
    }

    #[test]
    fn test_sc2005_substitution_body_direct() {
        assert_eq!(substitution_body("$(ls)"), Some("ls"));
        assert_eq!(substitution_body("`ls`"), Some("ls"));
        assert_eq!(substitution_body("$(ls) x"), None);
        assert_eq!(substitution_body("$(a $(b))"), Some("a $(b)"));
        assert_eq!(substitution_body("`a\\`b`"), None);
        assert_eq!(substitution_body("x"), None);
        assert_eq!(substitution_body("$(unclosed"), None);
    }
}
