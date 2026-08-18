//! Word / command-position analysis shared by lint rules (GH-228, GH-229).
//!
//! Many rules need to answer two questions about a physical line that a raw
//! `line.contains("docker")` or a quote-parity heuristic cannot answer:
//!
//! 1. **Is this word in command position, or is it an argument?**
//!    `$sh_c 'docker version'` mentions `docker`, but `docker` is *argument text*;
//!    the command is the variable. (GH-229)
//! 2. **Is this `$` really unquoted?** A command substitution starts a *fresh*
//!    quoting context (POSIX 2.6.3), so in `out="$(curl "$url")"` the `$url` is
//!    quoted even though the outer word opened a double quote. (GH-228)
//!
//! This module answers both by scanning the line as a byte slice with an explicit
//! quoting state, splitting it into simple commands, and recursing into `$( … )`
//! and `` ` … ` `` as independent command contexts.
//!
//! ## Design constraints
//!
//! * **Total and panic-free.** A lint rule must never abort on malformed input, so
//!   there is no `Result`: unbalanced quotes / unterminated `$(` are scanned to end
//!   of line and yield whatever was recognised. Every byte read goes through
//!   `bytes.get()`, and every slice through `str::get(..)`.
//! * **Byte columns.** Every column is a 1-indexed *byte* offset into the physical
//!   line, because that is what the autofix splicer
//!   (`linter::autofix_apply::apply_single_fix`) and SC2086/SC2183 use. Char-indexed
//!   columns make `--fix` panic on non-ASCII lines.
//! * **Bounded recursion.** Nesting is capped at [`MAX_SUB_DEPTH`] so a pathological
//!   line cannot exhaust the stack.
//!
//! It deliberately does *not* reuse `bash_parser::lexer`: that lexer collapses
//! `'x'` and `"x"` into the same token (erasing the single-vs-double distinction
//! this analysis is built on) and returns `Err` on unterminated strings.

/// Where a word sits inside its simple command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordRole {
    /// A `NAME=value` prefix appearing before the command name.
    AssignPrefix,
    /// A shell reserved word, or a wrapper command and its own operands.
    Reserved,
    /// The command name itself.
    CommandName,
    /// Any word after the command name.
    Argument,
    /// The word immediately following a `<` or `>` operator.
    RedirectTarget,
}

/// One `$`-expansion inside a word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expansion {
    /// 1-indexed **byte** column of the `$`, relative to the physical line.
    pub col: usize,
    /// 1-indexed byte column one past the end of the expansion text.
    /// `$URL` in `curl $URL` gives `col = 6`, `end_col = 10`.
    pub end_col: usize,
    /// The expansion exactly as written: `$URL`, `${URL}`, `${URL:-x}`.
    /// Quoting this text is always a semantics-preserving fix.
    pub text: String,
    /// Variable name without `$`, braces or modifiers: `URL` for all three above.
    pub name: String,
    /// True for the `${NAME…}` form.
    pub braced: bool,
    /// True when the `$` sits inside `'…'` or `"…"` **at its own nesting level**.
    /// A command substitution resets quoting (POSIX 2.6.3), so the `$url` in
    /// `x="$(curl $url)"` is NOT quoted while the one in `x="$(curl "$url")"` is.
    pub quoted: bool,
}

/// One shell word: a maximal run of characters between unquoted blanks.
#[derive(Debug, Clone)]
pub struct ShellWord {
    /// 1-indexed byte column of the word's first character.
    pub col: usize,
    /// The word exactly as written, quotes and all. Used for assignment-prefix
    /// detection, where a valid `NAME` cannot contain a quote or a `$`.
    pub raw: String,
    /// The word with quotes removed and backslash escapes resolved, and with
    /// expansions omitted. `'docker version'` -> `docker version`, `-d' '` -> `-d `.
    pub literal: String,
    /// The word's position within its simple command.
    pub role: WordRole,
    /// Expansions in this word, in ascending column order.
    pub expansions: Vec<Expansion>,
}

/// A simple command: the words between two control operators.
#[derive(Debug, Clone)]
pub struct SimpleCommand {
    /// `basename` of the command name, when that word is fully literal (no
    /// expansion). `None` when the name is `$var`, absent, or unresolvable —
    /// which is exactly the GH-229 dispatcher case.
    pub name: Option<String>,
    /// Every word of the command, in source order, each tagged with its role.
    pub words: Vec<ShellWord>,
}

/// Reserved words that never name an external command; the word after them is
/// still in command position.
const RESERVED_WORDS: &[&str] = &[
    "if", "then", "else", "elif", "fi", "while", "until", "do", "done", "case", "esac", "for",
    "select", "in", "function", "coproc", "!", "{", "}", "[[", "]]",
];

/// Words that delegate to another command; the real command name follows.
const WRAPPER_COMMANDS: &[&str] = &[
    "sudo", "doas", "env", "command", "exec", "nohup", "nice", "ionice", "setsid", "stdbuf",
    "time", "timeout", "busybox", "xargs",
];

/// Upper bound on words skipped while resolving through wrappers. Bounds the
/// false-positive surface; termination does not depend on it.
const MAX_WRAPPER_SKIP: u8 = 8;

/// Upper bound on `$( … )` / `` ` … ` `` nesting we descend into. Termination does
/// not depend on it (the lexer always advances), but the stack depth does.
pub const MAX_SUB_DEPTH: u8 = 32;

/// Split one physical line into simple commands, recursing into `$( … )` and
/// `` ` … ` `` as fresh command contexts.
///
/// Total and panic-free: malformed input (unbalanced quotes, unterminated `$(`)
/// is scanned to end of line and yields whatever words were recognised. All
/// columns are 1-indexed **byte** offsets into `line`, valid for nested words too.
///
/// # Examples
///
/// ```
/// use bashrs::linter::shell_words::{simple_commands, WordRole};
///
/// let cmds = simple_commands(r#"out="$(curl -sSfL $url)""#);
/// // The outer word is an assignment prefix, so the outer command has no name.
/// // The inner command substitution is analysed as its own command.
/// let curl = cmds.iter().find(|c| c.name.as_deref() == Some("curl")).unwrap();
/// let arg = curl.words.iter().find(|w| w.role == WordRole::Argument).unwrap();
/// assert_eq!(arg.expansions[0].text, "$url");
/// assert!(!arg.expansions[0].quoted);
/// ```
pub fn simple_commands(line: &str) -> Vec<SimpleCommand> {
    let mut out = Vec::new();
    collect(line, 0, 0, &mut out);
    out
}

fn collect(text: &str, base: usize, depth: u8, out: &mut Vec<SimpleCommand>) {
    let toks = WordLexer::new(text, base).run();
    build_commands(text, base, &toks, out);
    if depth >= MAX_SUB_DEPTH {
        return;
    }
    for tok in &toks {
        if let Tok::Word(w) = tok {
            for &(s, e) in &w.subs {
                if let Some(inner) = text.get(s..e) {
                    collect(inner, base + s, depth + 1, out);
                }
            }
        }
    }
}

// ===================================================================
// Lexing
// ===================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum Quote {
    Bare,
    Single,
    Double,
}

#[derive(Default)]
struct RawWord {
    start: usize,
    end: usize,
    literal: Vec<u8>,
    expansions: Vec<Expansion>,
    /// Byte ranges (into the lexer's `text`) of command-substitution bodies.
    subs: Vec<(usize, usize)>,
}

enum Tok {
    Word(RawWord),
    /// A control operator: `;`, `&`, `|`, newline, `(`, `)`.
    Sep,
    /// A redirection operator: `<` or `>`.
    Redir,
}

struct WordLexer<'a> {
    text: &'a str,
    bytes: &'a [u8],
    base: usize,
    i: usize,
    quote: Quote,
    cur: Option<RawWord>,
    toks: Vec<Tok>,
    done: bool,
}

impl<'a> WordLexer<'a> {
    fn new(text: &'a str, base: usize) -> Self {
        Self {
            text,
            bytes: text.as_bytes(),
            base,
            i: 0,
            quote: Quote::Bare,
            cur: None,
            toks: Vec::new(),
            done: false,
        }
    }

    fn run(mut self) -> Vec<Tok> {
        while self.i < self.bytes.len() && !self.done {
            self.step();
        }
        self.flush();
        self.toks
    }

    fn step(&mut self) {
        let b = match self.bytes.get(self.i) {
            Some(&b) => b,
            None => {
                self.i += 1;
                return;
            }
        };
        match self.quote {
            Quote::Bare => self.step_bare(b),
            Quote::Single => self.step_single(b),
            Quote::Double => self.step_double(b),
        }
    }

    fn step_bare(&mut self, b: u8) {
        if self.bare_separator(b) || self.bare_quote(b) {
            return;
        }
        match b {
            // A `#` only starts a comment where a word could start.
            b'#' if self.cur.is_none() => self.done = true,
            b'\\' => self.escape_bare(),
            b'$' => self.read_dollar(),
            b'`' => self.read_backtick(),
            _ => {
                self.push_literal(b);
                self.i += 1;
            }
        }
    }

    fn bare_separator(&mut self, b: u8) -> bool {
        match b {
            b' ' | b'\t' | b'\r' => self.emit_sep(None),
            b';' | b'&' | b'|' | b'\n' | b'(' | b')' => self.emit_sep(Some(Tok::Sep)),
            b'<' | b'>' => self.emit_sep(Some(Tok::Redir)),
            _ => false,
        }
    }

    fn emit_sep(&mut self, tok: Option<Tok>) -> bool {
        self.flush();
        if let Some(t) = tok {
            self.toks.push(t);
        }
        self.i += 1;
        true
    }

    fn bare_quote(&mut self, b: u8) -> bool {
        let next = match b {
            b'\'' => Quote::Single,
            b'"' => Quote::Double,
            _ => return false,
        };
        self.start_word();
        self.quote = next;
        self.i += 1;
        true
    }

    fn step_single(&mut self, b: u8) {
        if b == b'\'' {
            self.quote = Quote::Bare;
        } else {
            self.push_literal(b);
        }
        self.i += 1;
    }

    fn step_double(&mut self, b: u8) {
        match b {
            b'"' => {
                self.quote = Quote::Bare;
                self.i += 1;
            }
            b'\\' => self.escape_double(),
            b'$' => self.read_dollar(),
            b'`' => self.read_backtick(),
            _ => {
                self.push_literal(b);
                self.i += 1;
            }
        }
    }

    fn escape_bare(&mut self) {
        self.start_word();
        match self.bytes.get(self.i + 1) {
            Some(&nb) => {
                self.push_literal(nb);
                self.i += 2;
            }
            None => self.i += 1,
        }
    }

    fn escape_double(&mut self) {
        // Inside `"…"` a backslash only escapes `$`, backtick, `"` and `\`.
        match self.bytes.get(self.i + 1) {
            Some(&nb) if matches!(nb, b'$' | b'`' | b'"' | b'\\') => {
                self.push_literal(nb);
                self.i += 2;
            }
            _ => {
                self.push_literal(b'\\');
                self.i += 1;
            }
        }
    }

    fn read_dollar(&mut self) {
        self.start_word();
        let d = self.i;
        match (self.bytes.get(d + 1), self.bytes.get(d + 2)) {
            // `$(( … ))` arithmetic: no word splitting, nothing to report.
            (Some(b'('), Some(b'(')) => {
                self.i = find_close(self.bytes, d + 1, b'(', b')') + 1;
            }
            (Some(b'('), _) => {
                let end = find_close(self.bytes, d + 1, b'(', b')');
                self.push_sub(d + 2, end);
                self.i = end + 1;
            }
            (Some(b'{'), _) => {
                let end = find_close(self.bytes, d + 1, b'{', b'}');
                self.push_brace(d, end);
                self.i = end + 1;
            }
            (Some(&c), _) if is_name_byte(c) => self.read_name(d),
            _ => {
                self.push_literal(b'$');
                self.i += 1;
            }
        }
    }

    fn read_backtick(&mut self) {
        self.start_word();
        let start = self.i;
        let mut j = start + 1;
        while let Some(&b) = self.bytes.get(j) {
            if b == b'\\' {
                j += 2;
                continue;
            }
            if b == b'`' {
                break;
            }
            j += 1;
        }
        let end = j.min(self.bytes.len());
        self.push_sub(start + 1, end);
        self.i = end + 1;
    }

    fn read_name(&mut self, dollar: usize) {
        let mut j = dollar + 1;
        while self.bytes.get(j).is_some_and(|&c| is_name_byte(c)) {
            j += 1;
        }
        let name = self.text.get(dollar + 1..j).unwrap_or("").to_string();
        self.add_expansion(dollar, j, name, false);
        self.i = j;
    }

    fn push_brace(&mut self, dollar: usize, close: usize) {
        let body = self.text.get(dollar + 2..close).unwrap_or("");
        if let Some(name) = brace_var_name(body) {
            let name = name.to_string();
            // `close` is the index of `}`, or `bytes.len()` when unterminated.
            let end = if close < self.bytes.len() {
                close + 1
            } else {
                close
            };
            self.add_expansion(dollar, end, name, true);
        }
    }

    fn add_expansion(&mut self, dollar: usize, end: usize, name: String, braced: bool) {
        let quoted = self.quote == Quote::Double;
        let text = self.text.get(dollar..end).unwrap_or("").to_string();
        let (col, end_col) = (self.base + dollar + 1, self.base + end + 1);
        if let Some(w) = self.cur.as_mut() {
            w.expansions.push(Expansion {
                col,
                end_col,
                text,
                name,
                braced,
                quoted,
            });
        }
    }

    fn push_sub(&mut self, start: usize, end: usize) {
        if start > end {
            return;
        }
        if let Some(w) = self.cur.as_mut() {
            w.subs.push((start, end));
        }
    }

    fn start_word(&mut self) {
        if self.cur.is_none() {
            self.cur = Some(RawWord {
                start: self.i,
                ..RawWord::default()
            });
        }
    }

    fn push_literal(&mut self, b: u8) {
        self.start_word();
        if let Some(w) = self.cur.as_mut() {
            w.literal.push(b);
        }
    }

    fn flush(&mut self) {
        if let Some(mut w) = self.cur.take() {
            w.end = self.i;
            self.toks.push(Tok::Word(w));
        }
    }
}

// ===================================================================
// Byte-level helpers (all clamped, none can panic)
// ===================================================================

fn is_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Byte index of the `close` byte matching the `open` byte at `start`, honouring
/// quotes and backslash escapes. Returns `bytes.len()` when unbalanced.
fn find_close(bytes: &[u8], start: usize, open: u8, close: u8) -> usize {
    let mut depth = 1usize;
    let mut i = start + 1;
    while let Some(&b) = bytes.get(i) {
        if b == b'\'' || b == b'"' {
            i = skip_quoted(bytes, i, b);
            continue;
        }
        if b == b'\\' {
            i += 2;
            continue;
        }
        if b == open {
            depth += 1;
        } else if b == close {
            depth -= 1;
            if depth == 0 {
                return i;
            }
        }
        i += 1;
    }
    bytes.len()
}

/// Byte index just past the closing `quote` byte. `"` honours `\` escapes; `'` does
/// not (POSIX). Returns `bytes.len()` when the quote is never closed.
fn skip_quoted(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut i = start + 1;
    while let Some(&b) = bytes.get(i) {
        if quote == b'"' && b == b'\\' {
            i += 2;
            continue;
        }
        if b == quote {
            return i + 1;
        }
        i += 1;
    }
    bytes.len()
}

/// Leading variable name inside a `${ … }` body, ignoring a leading `#` or `!`.
/// `URL` for `URL`, `URL:-x`, `#URL`; `None` for `!*` or an empty body.
fn brace_var_name(body: &str) -> Option<&str> {
    let s = body
        .strip_prefix('#')
        .or_else(|| body.strip_prefix('!'))
        .unwrap_or(body);
    let end = s
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(s.len());
    if end == 0 {
        None
    } else {
        s.get(..end)
    }
}

/// `NAME` when `raw` starts with an unquoted POSIX assignment prefix `NAME=`.
/// A valid NAME contains no quote and no `$`, so matching on `raw` is exact:
/// `FOO=$X` -> `Some("FOO")`, `myapp=myapp:$V` -> `Some("myapp")`,
/// `-d'='` -> `None`, `"docker"` -> `None`.
fn assignment_name(raw: &str) -> Option<&str> {
    let bytes = raw.as_bytes();
    let first = *bytes.first()?;
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return None;
    }
    let mut i = 1;
    while let Some(&b) = bytes.get(i) {
        if b == b'=' {
            return raw.get(..i);
        }
        if !is_name_byte(b) {
            return None;
        }
        i += 1;
    }
    None
}

/// A wrapper's own option or numeric operand: `-E`, `5`, `30s`, `1.5m`.
/// Must reject command names: `ssh` -> strip trailing `h` -> `ss` -> not numeric.
fn is_wrapper_operand(lit: &str) -> bool {
    if lit.starts_with('-') {
        return true;
    }
    let core = match lit.as_bytes().last() {
        // A bare duration suffix: `30s`, `1.5m`, `2h`, `1d`.
        Some(b's' | b'm' | b'h' | b'd') => lit.get(..lit.len() - 1).unwrap_or(""),
        _ => lit,
    };
    !core.is_empty() && core.bytes().all(|c| c.is_ascii_digit() || c == b'.')
}

/// `/usr/bin/curl` -> `curl`.
fn basename(s: &str) -> &str {
    match s.rfind('/') {
        Some(p) => s.get(p + 1..).unwrap_or(s),
        None => s,
    }
}

// ===================================================================
// Role assignment
// ===================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Prefix,
    Args,
}

struct CmdState {
    name: Option<String>,
    words: Vec<ShellWord>,
    stage: Stage,
    wrappers: u8,
    redirect: bool,
}

impl CmdState {
    fn new() -> Self {
        Self {
            name: None,
            words: Vec::new(),
            stage: Stage::Prefix,
            wrappers: 0,
            redirect: false,
        }
    }

    fn role_for(&mut self, w: &ShellWord) -> WordRole {
        if self.redirect {
            self.redirect = false;
            return WordRole::RedirectTarget;
        }
        if self.stage == Stage::Args {
            return WordRole::Argument;
        }
        if assignment_name(&w.raw).is_some() {
            return WordRole::AssignPrefix;
        }
        if w.expansions.is_empty() && RESERVED_WORDS.contains(&w.literal.as_str()) {
            return WordRole::Reserved;
        }
        // `$sh_c 'docker version'`: the name is a variable, so it is unresolvable.
        if !w.expansions.is_empty() {
            return WordRole::CommandName;
        }
        if self.consume_wrapper(&w.literal) {
            return WordRole::Reserved;
        }
        WordRole::CommandName
    }

    /// `wrappers > 0` gates the operand rule deliberately: without it a numbered
    /// documentation line (`  18 kubectl set image …`) would treat `18` as an
    /// operand and promote `kubectl` to the command name.
    fn consume_wrapper(&mut self, lit: &str) -> bool {
        if self.wrappers >= MAX_WRAPPER_SKIP {
            return false;
        }
        let hit = (self.wrappers > 0 && is_wrapper_operand(lit))
            || WRAPPER_COMMANDS.contains(&basename(lit));
        if hit {
            self.wrappers += 1;
        }
        hit
    }

    fn push_word(&mut self, mut w: ShellWord) {
        w.role = self.role_for(&w);
        if w.role == WordRole::CommandName {
            self.name = command_name_of(&w);
            self.stage = Stage::Args;
        }
        self.words.push(w);
    }

    fn finish(&mut self, out: &mut Vec<SimpleCommand>) {
        if !self.words.is_empty() {
            out.push(SimpleCommand {
                name: self.name.take(),
                words: std::mem::take(&mut self.words),
            });
        }
        *self = CmdState::new();
    }
}

fn command_name_of(w: &ShellWord) -> Option<String> {
    if w.expansions.is_empty() {
        Some(basename(&w.literal).to_string())
    } else {
        None
    }
}

fn to_shell_word(text: &str, base: usize, w: &RawWord) -> ShellWord {
    ShellWord {
        col: base + w.start + 1,
        raw: text.get(w.start..w.end).unwrap_or("").to_string(),
        literal: String::from_utf8_lossy(&w.literal).into_owned(),
        role: WordRole::Argument,
        expansions: w.expansions.clone(),
    }
}

fn build_commands(text: &str, base: usize, toks: &[Tok], out: &mut Vec<SimpleCommand>) {
    let mut st = CmdState::new();
    for tok in toks {
        match tok {
            Tok::Sep => st.finish(out),
            Tok::Redir => st.redirect = true,
            Tok::Word(w) => st.push_word(to_shell_word(text, base, w)),
        }
    }
    st.finish(out);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd_names(line: &str) -> Vec<Option<String>> {
        simple_commands(line).into_iter().map(|c| c.name).collect()
    }

    fn all_expansions(line: &str) -> Vec<Expansion> {
        simple_commands(line)
            .into_iter()
            .flat_map(|c| c.words.into_iter().flat_map(|w| w.expansions))
            .collect()
    }

    #[test]
    fn test_SW_001_words_and_columns() {
        let cmds = simple_commands("curl -sSfL $u");
        assert_eq!(cmds.len(), 1);
        let cols: Vec<usize> = cmds[0].words.iter().map(|w| w.col).collect();
        assert_eq!(cols, vec![1, 6, 12]);
        assert_eq!(cmds[0].name.as_deref(), Some("curl"));
    }

    #[test]
    fn test_SW_002_single_quotes_join_into_one_word() {
        let cmds = simple_commands("cut -d' ' -f1");
        assert_eq!(cmds[0].words.len(), 3);
        assert_eq!(cmds[0].words[1].literal, "-d ");
        assert_eq!(cmds[0].words[1].raw, "-d' '");
    }

    #[test]
    fn test_SW_003_adjacent_quoting_concatenates() {
        let cmds = simple_commands(r#"curl pre"$url"post"#);
        assert_eq!(cmds[0].words.len(), 2);
        let w = &cmds[0].words[1];
        assert_eq!(w.literal, "prepost");
        assert_eq!(w.expansions.len(), 1);
        assert!(w.expansions[0].quoted);
    }

    #[test]
    fn test_SW_004_cmdsub_yields_two_commands() {
        let names = cmd_names(r#"x="$(curl $u | cut -f1)""#);
        assert!(names.contains(&Some("curl".to_string())));
        assert!(names.contains(&Some("cut".to_string())));
    }

    #[test]
    fn test_SW_005_cmdsub_columns_are_absolute() {
        let e = all_expansions(r#"x="$(curl $u)""#);
        assert_eq!(e.len(), 1);
        assert_eq!((e[0].col, e[0].end_col), (11, 13));
        assert_eq!(e[0].text, "$u");
    }

    #[test]
    fn test_SW_006_cmdsub_resets_quoting() {
        let quoted = all_expansions(r#"x="$(curl "$u")""#);
        assert_eq!(quoted.len(), 1);
        assert!(quoted[0].quoted, "inner \"$u\" is quoted at its own level");

        let bare = all_expansions(r#"x="$(curl $u)""#);
        assert_eq!(bare.len(), 1);
        assert!(!bare[0].quoted, "outer quote must not leak into the sub");
    }

    #[test]
    fn test_SW_007_nested_cmdsub() {
        let names = cmd_names("a=$(echo $(curl $u))");
        assert!(names.contains(&Some("echo".to_string())));
        assert!(names.contains(&Some("curl".to_string())));
    }

    #[test]
    fn test_SW_008_roles_assignment_name_argument() {
        let cmds = simple_commands("FOO=1 curl -s $U");
        assert_eq!(cmds.len(), 1);
        let roles: Vec<WordRole> = cmds[0].words.iter().map(|w| w.role).collect();
        assert_eq!(
            roles,
            vec![
                WordRole::AssignPrefix,
                WordRole::CommandName,
                WordRole::Argument,
                WordRole::Argument
            ]
        );
        assert_eq!(cmds[0].name.as_deref(), Some("curl"));
    }

    #[test]
    fn test_SW_009_wrapper_chain_sudo_env_timeout() {
        assert_eq!(
            simple_commands("sudo env timeout 30s curl $U")[0]
                .name
                .as_deref(),
            Some("curl")
        );
        // Beyond MAX_WRAPPER_SKIP the chain stops resolving rather than looping.
        let many = "sudo -a -b -c -d -e -f -g -h -i curl $U";
        assert_ne!(simple_commands(many)[0].name.as_deref(), Some("curl"));
    }

    #[test]
    fn test_SW_010_redirect_target_role() {
        let cmds = simple_commands("curl $A > $B");
        let roles: Vec<WordRole> = cmds[0].words.iter().map(|w| w.role).collect();
        assert_eq!(
            roles,
            vec![
                WordRole::CommandName,
                WordRole::Argument,
                WordRole::RedirectTarget
            ]
        );
    }

    #[test]
    fn test_SW_011_unbalanced_paren_returns_end_of_line() {
        let b = b"$(abc";
        assert_eq!(find_close(b, 1, b'(', b')'), b.len());
        // And the whole scan still terminates and finds the inner command.
        assert!(cmd_names("x=$(curl $u").contains(&Some("curl".to_string())));
    }

    #[test]
    fn test_SW_012_find_close_ignores_parens_inside_quotes() {
        let b = b"$(echo \"(\")";
        assert_eq!(find_close(b, 1, b'(', b')'), b.len() - 1);
    }

    #[test]
    fn test_SW_013_brace_var_name_strips_hash_and_modifiers() {
        assert_eq!(brace_var_name("a"), Some("a"));
        assert_eq!(brace_var_name("#a"), Some("a"));
        assert_eq!(brace_var_name("a:-b"), Some("a"));
        assert_eq!(brace_var_name("a##*/"), Some("a"));
        assert_eq!(brace_var_name(""), None);
        assert_eq!(brace_var_name("!*"), None);
    }

    #[test]
    fn test_SW_014_assignment_name_rejects_quoted_and_dashed() {
        assert_eq!(assignment_name("FOO=$X"), Some("FOO"));
        assert_eq!(assignment_name("myapp=myapp:$V"), Some("myapp"));
        assert_eq!(assignment_name("_x=1"), Some("_x"));
        assert_eq!(assignment_name("-d'='"), None);
        assert_eq!(assignment_name("\"docker\""), None);
        assert_eq!(assignment_name("18"), None);
        assert_eq!(assignment_name("curl"), None);
    }

    #[test]
    fn test_SW_015_is_wrapper_operand_rejects_command_names() {
        assert!(is_wrapper_operand("-E"));
        assert!(is_wrapper_operand("5"));
        assert!(is_wrapper_operand("30s"));
        assert!(is_wrapper_operand("1.5m"));
        for name in ["ssh", "scp", "sh", "dd", "curl", "git"] {
            assert!(!is_wrapper_operand(name), "{} must not be an operand", name);
        }
    }

    #[test]
    fn test_SW_016_multibyte_columns_are_byte_offsets() {
        let line = "curl é$U";
        let e = all_expansions(line);
        assert_eq!(e.len(), 1);
        // 'é' is 2 bytes, so `$` is at byte column 8, not char column 7.
        assert_eq!(e[0].col, 8);
        assert!(line.is_char_boundary(e[0].col - 1));
        assert!(line.is_char_boundary(e[0].end_col - 1));
    }

    #[test]
    fn test_SW_017_comment_stops_the_scan() {
        let names = cmd_names("curl $a # docker run $b");
        assert_eq!(names, vec![Some("curl".to_string())]);
    }

    #[test]
    fn test_SW_018_braced_expansion_text_is_verbatim() {
        let e = all_expansions("curl ${URL:-https://d}");
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].text, "${URL:-https://d}");
        assert_eq!(e[0].name, "URL");
        assert!(e[0].braced);
    }

    #[test]
    fn test_SW_019_deep_nesting_is_bounded_and_terminates() {
        let s = format!("curl {}$u{}", "$(".repeat(200), ")".repeat(200));
        let cmds = simple_commands(&s);
        assert!(cmds.len() <= MAX_SUB_DEPTH as usize + 2);
    }

    #[test]
    fn test_SW_020_arithmetic_is_not_an_expansion() {
        assert!(all_expansions("curl $(( x + $y ))").is_empty());
    }
}
