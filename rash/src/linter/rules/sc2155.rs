// SC2155: Declare and assign separately to avoid masking return values
//
// When using `local var=$(command)`, the return value of command is masked
// by the return value of `local` (which is always 0).
//
// Examples:
// Bad:
//   local result=$(failing_command)
//   # $? is 0 (from local), not the command's exit code
//
// Good:
//   local result
//   result=$(failing_command)
//   # $? is the command's actual exit code

use crate::linter::{Diagnostic, LintResult, Severity, Span};

const DECL_KEYWORDS: [&str; 5] = ["local", "declare", "typeset", "readonly", "export"];
const PREFIX_WORDS: [&str; 5] = ["then", "do", "else", "{", "!"];

/// A shell word with its byte offset in the statement.
type Word<'a> = (usize, &'a str);
/// Declaration keyword offset, keyword and its argument words.
type Decl<'a> = (usize, &'a str, &'a [Word<'a>]);

/// Minimal shell lexer state: quotes, `(...)` nesting and backticks.
#[derive(Default)]
struct Lex {
    depth: i32,
    quote: Option<char>,
    tick: bool,
}

impl Lex {
    fn is_open(&self) -> bool {
        self.quote.is_some() || self.depth > 0 || self.tick
    }

    /// Feed one char; returns true when the next char is escaped (skip it).
    fn feed(&mut self, c: char) -> bool {
        if c == '\\' && self.quote != Some('\'') {
            return true;
        }
        match self.quote {
            Some(q) => self.in_quote(c, q),
            None => self.outside(c),
        }
        false
    }

    fn in_quote(&mut self, c: char, q: char) {
        if c == q {
            self.quote = None;
        }
    }

    fn outside(&mut self, c: char) {
        match c {
            '\'' | '"' => self.quote = Some(c),
            '`' => self.tick = !self.tick,
            '(' => self.depth += 1,
            ')' => self.depth -= 1,
            _ => {}
        }
    }
}

/// Split a line into simple-command segments `(byte_offset, text)` at
/// unquoted `;`, `&` and `|`.
fn split_statements(line: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let (mut start, mut lex) = (0usize, Lex::default());
    let mut chars = line.char_indices();
    while let Some((i, c)) = chars.next() {
        if matches!(c, ';' | '&' | '|') && !lex.is_open() {
            out.push((start, &line[start..i]));
            start = i + 1;
        } else if lex.feed(c) {
            chars.next();
        }
    }
    out.push((start, &line[start..]));
    out
}

/// Split a statement into whitespace-separated words, honouring quotes,
/// `$(...)` and backticks.
fn split_words(stmt: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let (mut start, mut lex) = (None::<usize>, Lex::default());
    let mut chars = stmt.char_indices();
    while let Some((i, c)) = chars.next() {
        if c.is_whitespace() && !lex.is_open() {
            if let Some(s) = start.take() {
                out.push((s, &stmt[s..i]));
            }
            continue;
        }
        start.get_or_insert(i);
        if lex.feed(c) {
            chars.next();
        }
    }
    if let Some(s) = start {
        out.push((s, &stmt[s..]));
    }
    out
}

/// `$(` that opens a command substitution rather than `$((` arithmetic.
fn is_subst_open(rest: &str) -> bool {
    rest.starts_with('(') && !rest.starts_with("((")
}

/// Byte length of the balanced `open ... close` group at the start of `s`
/// (`s` starts with `open`); the whole of `s` when unbalanced.
fn balanced_len(s: &str, open: char, close: char) -> usize {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        depth += i32::from(c == open) - i32::from(c == close);
        if depth == 0 {
            return i + 1;
        }
    }
    s.len()
}

/// Length of a `$((...))` or `${...}` group whose body shellcheck does not
/// inspect for substitutions; `rest` is the text after the `$`.
fn opaque_group_len(rest: &str) -> Option<usize> {
    if rest.starts_with("((") {
        Some(balanced_len(rest, '(', ')'))
    } else if rest.starts_with('{') {
        Some(balanced_len(rest, '{', '}'))
    } else {
        None
    }
}

/// What the substitution scanner does after one character.
enum Step {
    Found,
    SkipTo(usize),
    Next,
}

/// Quote state of the substitution scanner.
#[derive(Default)]
struct Quotes {
    single: bool,
    double: bool,
}

impl Quotes {
    /// Classify the char at byte `i`; toggles quote state as a side effect.
    fn step(&mut self, value: &str, i: usize, c: char) -> Step {
        match c {
            '\'' if !self.double => self.single = !self.single,
            '"' if !self.single => self.double = !self.double,
            '`' if !self.single => return Step::Found,
            '$' if !self.single => return Self::dollar(&value[i + 1..], i + 1),
            _ => {}
        }
        Step::Next
    }

    fn dollar(rest: &str, base: usize) -> Step {
        match opaque_group_len(rest) {
            Some(n) => Step::SkipTo(base + n),
            None if is_subst_open(rest) => Step::Found,
            None => Step::Next,
        }
    }
}

/// True when the value contains an active command substitution
/// (`$(...)` that is not arithmetic, or backticks), outside single quotes
/// and not backslash-escaped. Like shellcheck, ignores substitutions nested
/// inside `$((...))`, `${...}` and array literals `(...)`.
fn has_command_subst(value: &str) -> bool {
    if value.starts_with('(') {
        return false;
    }
    let (mut quotes, mut skip_to) = (Quotes::default(), 0usize);
    let mut chars = value.char_indices();
    while let Some((i, c)) = chars.next() {
        if c == '\\' && !quotes.single {
            chars.next();
            continue;
        }
        if i < skip_to {
            continue;
        }
        match quotes.step(value, i, c) {
            Step::Found => return true,
            Step::SkipTo(n) => skip_to = n,
            Step::Next => {}
        }
    }
    false
}

/// `NAME=value` where NAME is a valid identifier; returns the value.
fn assignment_value(word: &str) -> Option<&str> {
    let (name, value) = word.split_once('=')?;
    let mut cs = name.chars();
    let first = cs.next()?;
    let ident = (first.is_ascii_alphabetic() || first == '_')
        && cs.all(|c| c.is_ascii_alphanumeric() || c == '_');
    ident.then_some(value)
}

/// shellcheck stays silent when `-r` is given to declare/local/typeset.
fn has_readonly_flag(keyword: &str, flags: &[(usize, &str)]) -> bool {
    keyword != "readonly"
        && flags
            .iter()
            .any(|(_, w)| w.starts_with('-') && w.contains('r'))
}

/// Keyword word of a statement (skipping `then`/`do`/`{` prefixes) and the
/// words after it.
fn declaration_of<'a>(words: &'a [Word<'a>]) -> Option<Decl<'a>> {
    let idx = words.iter().position(|(_, w)| !PREFIX_WORDS.contains(w))?;
    let (off, kw) = words[idx];
    DECL_KEYWORDS
        .contains(&kw)
        .then_some((off, kw, &words[idx + 1..]))
}

/// Offset and name of the declaration keyword if the statement masks a
/// command substitution's return value.
fn masked_declaration(stmt: &str) -> Option<(usize, String)> {
    let words = split_words(stmt);
    let end = words
        .iter()
        .position(|(_, w)| w.starts_with('#'))
        .unwrap_or(words.len());
    let (off, kw, args) = declaration_of(&words[..end])?;
    if has_readonly_flag(kw, args) {
        return None;
    }
    args.iter()
        .filter_map(|(_, w)| assignment_value(w))
        .any(has_command_subst)
        .then(|| (off, kw.to_string()))
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for (base, stmt) in split_statements(line) {
            if let Some((off, keyword)) = masked_declaration(stmt) {
                let start_col = base + off + 1;
                let end_col = start_col + keyword.len();
                result.add(Diagnostic::new(
                    "SC2155",
                    Severity::Warning,
                    format!(
                        "Declare and assign separately to avoid masking return values ({keyword})"
                    ),
                    Span::new(line_num, start_col, line_num, end_col),
                ));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2155_local_with_command_subst() {
        let code = r#"local result=$(command)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2155");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc2155_declare_with_command_subst() {
        let code = r#"declare var=$(some_command)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_readonly_with_command_subst() {
        let code = r#"readonly CONST=$(get_value)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_export_with_command_subst() {
        let code = r#"export PATH=$(get_path)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_separate_ok() {
        let code = r#"
local result
result=$(command)
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2155_simple_assignment_ok() {
        let code = r#"local var="value""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2155_variable_expansion_ok() {
        let code = r#"local var="$other""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2155_multiple_on_line() {
        let code = r#"local a=$(cmd1) b=$(cmd2)"#;
        let result = check(code);
        // Should detect at least the first one
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn test_sc2155_function_context() {
        let code = r#"
function test() {
    local result=$(failing_command)
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_no_command_subst() {
        let code = r#"local var=value"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2155_526_readonly_quoted_subst() {
        assert_eq!(check(r#"readonly X="$(id)""#).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_526_local_quoted_then_plain() {
        assert_eq!(check(r#"local a="$(date)" b=2"#).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_526_local_second_assignment_subst() {
        assert_eq!(check(r#"local a=2 b="$(date)""#).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_526_declare_x_unquoted() {
        assert_eq!(check("declare -x y=$(ls)").diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_526_backticks() {
        assert_eq!(check("local a=`date`").diagnostics.len(), 1);
        assert_eq!(check("local a=\"`date`\"").diagnostics.len(), 1);
        assert_eq!(check("readonly X=`id`").diagnostics.len(), 1);
        assert_eq!(check("export X=`id`").diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_526_embedded_subst() {
        assert_eq!(check(r#"local a="pre$(date)post""#).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_526_after_separator() {
        assert_eq!(check("echo hi; local a=$(date)").diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_526_readonly_flag_silent() {
        // shellcheck 0.8.0 stays silent with -r on declare/local
        assert_eq!(check("declare -r y=$(ls)").diagnostics.len(), 0);
        assert_eq!(check("local -r z=$(id)").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2155_526_silent_cases() {
        assert_eq!(check("local x; x=$(cmd)").diagnostics.len(), 0);
        assert_eq!(check("export X=literal").diagnostics.len(), 0);
        assert_eq!(check("local a='$(date)'").diagnostics.len(), 0);
        assert_eq!(check(r#"local a="\$(date)""#).diagnostics.len(), 0);
        assert_eq!(check("local a=$((1+2))").diagnostics.len(), 0);
        assert_eq!(check(r#"echo "local a=$(date)""#).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2155_parity_apostrophe_in_dq_and_whitespace() {
        assert_eq!(check(r#"local a="it's $(date)""#).diagnostics.len(), 1);
        assert_eq!(check("local\ta=$(date)").diagnostics.len(), 1);
        assert_eq!(check("local   a=$(date)").diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2155_parity_nested_silent() {
        assert_eq!(check("local a=$(( $(date) ))").diagnostics.len(), 0);
        assert_eq!(check(r#"local a="${b:-$(date)}""#).diagnostics.len(), 0);
        assert_eq!(check("local -a arr=($(ls))").diagnostics.len(), 0);
        assert_eq!(check("local a=1 # x=$(date)").diagnostics.len(), 0);
        assert_eq!(check("local a='x'$(date)").diagnostics.len(), 1);
    }
}
