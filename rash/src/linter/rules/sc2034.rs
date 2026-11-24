//! SC2034: Variable assigned but never used
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! unused_var="value"
//! echo "Hello"
//! ```
//!
//! Good:
//! ```bash
//! used_var="value"
//! echo "$used_var"
//! ```
//!
//! # Rationale
//!
//! Variables that are assigned but never used may indicate:
//! - Dead code
//! - Typos in variable names
//! - Forgotten cleanup
//!
//! # Auto-fix
//!
//! Warning only - may be intentional (exported vars, etc.)

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Check for variables assigned but never used
static ASSIGN_PATTERN: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)=").unwrap());
static USE_PATTERN: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?").unwrap());

/// Incremental tokeniser state for `split_commands`.
#[derive(Default)]
struct Lexer {
    cmds: Vec<Vec<String>>,
    words: Vec<String>,
    cur: String,
    in_word: bool,
    /// The next word is a redirection target and is dropped.
    skip_next: bool,
}

impl Lexer {
    fn flush_word(&mut self) {
        if self.in_word {
            let w = std::mem::take(&mut self.cur);
            if std::mem::take(&mut self.skip_next) {
                // redirection target: not an operand
            } else {
                self.words.push(w);
            }
            self.in_word = false;
        }
    }

    fn end_command(&mut self) {
        self.flush_word();
        self.skip_next = false;
        if !self.words.is_empty() {
            self.cmds.push(std::mem::take(&mut self.words));
        }
    }

    fn push_char(&mut self, c: char) {
        self.cur.push(c);
        self.in_word = true;
    }

    /// An unquoted command separator. `&` right after a redirection operator
    /// (`>&2`) belongs to the redirection.
    fn separator(&mut self, c: char) {
        if c == '&' && self.skip_next && self.cur.is_empty() {
            return;
        }
        self.end_command();
    }

    /// An unquoted `<` or `>` starts a redirection: drop an fd prefix such as
    /// `2` and the target word that follows.
    fn start_redirect(&mut self) {
        if self.in_word && self.cur.chars().all(|c| c.is_ascii_digit()) {
            self.cur.clear();
            self.in_word = false;
        } else {
            self.flush_word();
        }
        self.skip_next = true;
    }
}

type Chars<'a> = std::iter::Peekable<std::str::Chars<'a>>;

/// Consume a quoted span (opening quote already read) into the current word.
fn lex_quoted(q: char, chars: &mut Chars<'_>, lx: &mut Lexer) {
    lx.in_word = true;
    while let Some(c) = chars.next() {
        if c == q {
            return;
        }
        if c == '\\' && q == '"' {
            if let Some(n) = chars.next() {
                lx.cur.push(n);
            }
        } else {
            lx.cur.push(c);
        }
    }
}

/// Consume a `$( ... )` span (the `$` already read, `(` next) verbatim into
/// the current word, balancing nested parentheses.
fn lex_subst(chars: &mut Chars<'_>, lx: &mut Lexer) {
    lx.push_char('$');
    let mut depth = 0usize;
    while let Some(c) = chars.next() {
        lx.cur.push(c);
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return;
                }
            }
            '\\' => {
                if let Some(n) = chars.next() {
                    lx.cur.push(n);
                }
            }
            _ => {}
        }
    }
}

/// Consume a backtick span (opening backtick already read) verbatim.
fn lex_backtick(chars: &mut Chars<'_>, lx: &mut Lexer) {
    lx.push_char('`');
    while let Some(c) = chars.next() {
        lx.cur.push(c);
        match c {
            '`' => return,
            '\\' => {
                if let Some(n) = chars.next() {
                    lx.cur.push(n);
                }
            }
            _ => {}
        }
    }
}

/// Split a line into simple commands, each a list of words with quotes
/// removed. Command substitutions stay inside their word and redirection
/// targets are dropped. Stops at an unquoted `#` comment; splits at unquoted
/// `;`, `&`, `|`, `(` and `)`.
fn split_commands(line: &str) -> Vec<Vec<String>> {
    let mut lx = Lexer::default();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' | '"' => lex_quoted(c, &mut chars, &mut lx),
            '\\' => {
                if let Some(n) = chars.next() {
                    lx.push_char(n);
                }
            }
            '$' if chars.peek() == Some(&'(') => lex_subst(&mut chars, &mut lx),
            '`' => lex_backtick(&mut chars, &mut lx),
            '#' if !lx.in_word => break,
            '<' | '>' => lx.start_redirect(),
            ';' | '&' | '|' | '(' | ')' => lx.separator(c),
            c if c.is_whitespace() => lx.flush_word(),
            _ => lx.push_char(c),
        }
    }
    lx.end_command();
    lx.cmds
}

/// Option words are the leading `-x` arguments up to `--` or the first
/// operand; later words are operands, never flags.
fn has_flag(args: &[String], flag: char) -> bool {
    args.iter()
        .take_while(|w| w.starts_with('-') && w.as_str() != "--")
        .any(|w| !w.starts_with("--") && w.contains(flag))
}

fn is_assignment_word(w: &str) -> bool {
    let Some(eq) = w.find('=') else { return false };
    let name = w[..eq].trim_end_matches('+');
    let mut cs = name.chars();
    matches!(cs.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && cs.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Index of the command word, skipping shell keywords, `!`, `{` and
/// prefix assignments (`FOO=bar export x`).
fn command_index(cmd: &[String]) -> usize {
    cmd.iter()
        .position(|w| {
            !matches!(
                w.as_str(),
                "if" | "then" | "else" | "elif" | "do" | "while" | "until" | "!" | "{" | "time"
            ) && !is_assignment_word(w)
        })
        .unwrap_or(cmd.len())
}

/// Names passed to `export` (or `declare -x` / `typeset -x`) count as used
/// (shellcheck parity). `readonly`, `export -n` and `export -f` do not
/// export a variable.
fn mark_exported(line: &str, used: &mut HashSet<String>) {
    for cmd in split_commands(line) {
        let idx = command_index(&cmd);
        let Some(name) = cmd.get(idx) else { continue };
        let args = &cmd[idx + 1..];
        if exports_variables(name, args) {
            mark_operands(args, used);
        }
    }
}

fn exports_variables(name: &str, args: &[String]) -> bool {
    match name {
        "export" => !has_flag(args, 'n') && !has_flag(args, 'f'),
        "declare" | "typeset" => has_flag(args, 'x'),
        _ => false,
    }
}

fn mark_operands(args: &[String], used: &mut HashSet<String>) {
    for word in args.iter().filter(|w| !w.starts_with('-')) {
        let name = word.split('=').next().unwrap_or("").trim_end_matches('+');
        if !name.is_empty() {
            used.insert(name.to_string());
        }
    }
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Collect assigned variables
    let assign_pattern = &*ASSIGN_PATTERN;
    // Collect used variables
    let use_pattern = &*USE_PATTERN;

    let mut assigned: HashMap<String, usize> = HashMap::new();
    let mut used: HashSet<String> = HashSet::new();

    // First pass: collect assignments and uses
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Find assignments
        for cap in assign_pattern.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str().to_string();
            assigned.insert(var_name, line_num);
        }

        mark_exported(line, &mut used);

        // Find uses
        for cap in use_pattern.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str().to_string();
            used.insert(var_name);
        }
    }

    // Second pass: find unused variables
    for (var_name, line_num) in &assigned {
        if !used.contains(var_name) {
            // Skip common patterns that are intentionally unused
            if var_name.starts_with('_') || var_name.to_uppercase() == *var_name {
                continue; // Skip _ prefixed and ALL_CAPS (often exported)
            }

            let diagnostic = Diagnostic::new(
                "SC2034",
                Severity::Info,
                format!("Variable '{}' is assigned but never used", var_name),
                Span::new(*line_num, 1, *line_num, var_name.len() + 1),
            );

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2034_basic_detection() {
        let script = r#"
unused_var="value"
echo "Hello"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2034");
    }

    #[test]
    fn test_sc2034_variable_used() {
        let script = r#"
used_var="value"
echo "$used_var"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2034_multiple_unused() {
        let script = r#"
unused1="value1"
unused2="value2"
echo "Hello"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2034_skip_underscore_prefix() {
        let script = r#"
_private="value"
echo "Hello"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0); // _ prefix skipped
    }

    #[test]
    fn test_sc2034_skip_all_caps() {
        let script = r#"
EXPORTED_VAR="value"
echo "Hello"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0); // ALL_CAPS skipped
    }

    #[test]
    fn test_sc2034_braced_usage() {
        let script = r#"
used_var="value"
echo "${used_var}"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2034_used_in_condition() {
        let script = r#"
check_var="test"
if [ "$check_var" = "test" ]; then
    echo "yes"
fi
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2034_mixed_used_and_unused() {
        let script = r#"
used="value1"
unused="value2"
echo "$used"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2034_used_in_assignment() {
        let script = r#"
var1="value"
var2="$var1"
echo "$var2"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2034_no_false_positive_in_comment() {
        let script = r#"
# unused_var="value"
echo "Hello"
"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2034_exported_variable_is_used() {
        let script = "x=1\nexport x";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2034_export_does_not_hide_other_unused() {
        let script = "x=1\ny=2\nexport x";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("'y'"));
    }

    fn unused_names(script: &str) -> Vec<String> {
        let mut v: Vec<String> = check(script)
            .diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect();
        v.sort();
        v
    }

    #[test]
    fn test_sc2034_export_quoted_value_does_not_leak() {
        let r = unused_names("y=2\nexport x=\"hello y world\"\n");
        assert!(r.iter().any(|m| m.contains("'y'")), "{r:?}");
    }

    #[test]
    fn test_sc2034_export_inline_comment_does_not_leak() {
        let r = unused_names("y=2\nexport x=1 # y\n");
        assert!(r.iter().any(|m| m.contains("'y'")), "{r:?}");
    }

    #[test]
    fn test_sc2034_export_chained_command_does_not_leak() {
        let r = unused_names("y=2\nexport x; y\n");
        assert!(r.iter().any(|m| m.contains("'y'")), "{r:?}");
        let r = unused_names("x=1\ny=2\nexport x && true\n");
        assert!(r.iter().any(|m| m.contains("'y'")), "{r:?}");
        assert!(!r.iter().any(|m| m.contains("'x'")), "{r:?}");
    }

    #[test]
    fn test_sc2034_readonly_is_not_export() {
        let r = unused_names("x=1\nreadonly x\n");
        assert!(r.iter().any(|m| m.contains("'x'")), "{r:?}");
    }

    #[test]
    fn test_sc2034_export_n_is_not_export() {
        let r = unused_names("x=1\nexport -n x\n");
        assert!(r.iter().any(|m| m.contains("'x'")), "{r:?}");
    }

    #[test]
    fn test_sc2034_export_still_exempts() {
        assert!(unused_names("x=1\nexport x\n").is_empty());
        assert!(unused_names("x=1\nexport -p x\n").is_empty());
    }

    #[test]
    fn test_sc2034_export_cmd_subst_does_not_leak() {
        let r = unused_names("env=prod\nexport API_URL=$(build_url env us-east)\n");
        assert!(r.iter().any(|m| m.contains("'env'")), "{r:?}");
        let r = unused_names("y=2\nexport x=`cmd y z`\n");
        assert!(r.iter().any(|m| m.contains("'y'")), "{r:?}");
        let r = unused_names("y=2\nexport x=$(a $(b y) z)\n");
        assert!(r.iter().any(|m| m.contains("'y'")), "{r:?}");
    }

    #[test]
    fn test_sc2034_export_redirect_does_not_leak() {
        let r = unused_names("out=1\nexport -p > out\n");
        assert!(r.iter().any(|m| m.contains("'out'")), "{r:?}");
        let r = unused_names("out=1\nexport -p 2>out\n");
        assert!(r.iter().any(|m| m.contains("'out'")), "{r:?}");
        let r = unused_names("out=1\nexport -p >&out\n");
        assert!(r.iter().any(|m| m.contains("'out'")), "{r:?}");
        assert!(unused_names("x=1\nexport x > /dev/null\n").is_empty());
    }

    #[test]
    fn test_sc2034_export_compound_prefixed_append() {
        assert!(unused_names("x=1\nif true; then export x; fi\n").is_empty());
        assert!(unused_names("x=1\n{ export x; }\n").is_empty());
        assert!(unused_names("x=1\n(export x)\n").is_empty());
        assert!(unused_names("x=1\nFOO=bar export x\n").is_empty());
        assert!(unused_names("x=1\nexport x+=2\n").is_empty());
        assert!(unused_names("x=1\nexport x=2\n").is_empty());
    }

    #[test]
    fn test_sc2034_export_f_does_not_mark_variable() {
        let r = unused_names("x=1\nexport -f x\n");
        assert!(r.iter().any(|m| m.contains("'x'")), "{r:?}");
    }

    #[test]
    fn test_sc2034_flags_only_from_option_args() {
        assert!(unused_names("x=1\nexport x=$(echo -n hi)\n").is_empty());
        let r = unused_names("x=1\nfoo=2\ndeclare x=$(grep -x foo bar)\n");
        assert_eq!(r.len(), 2, "{r:?}");
        assert!(unused_names("name=1\nexport name\n").is_empty());
        let r = unused_names("x=1\nexport -- -n x\n");
        assert!(r.is_empty(), "{r:?}");
        let r = unused_names("x=1\nexport --nope x\n");
        assert!(r.is_empty(), "{r:?}");
    }

    #[test]
    fn test_sc2034_declare_typeset_x() {
        assert!(unused_names("x=1\ndeclare -x x\n").is_empty());
        assert!(unused_names("x=1\ntypeset -x x\n").is_empty());
        let r = unused_names("x=1\ndeclare x\n");
        assert!(r.iter().any(|m| m.contains("'x'")), "{r:?}");
        let r = unused_names("x=1\ntypeset x\n");
        assert!(r.iter().any(|m| m.contains("'x'")), "{r:?}");
    }

    #[test]
    fn test_sc2034_lexer_quotes_escapes_pipe() {
        let r = unused_names("y=2\nexport x='a; y'\n");
        assert!(r.iter().any(|m| m.contains("'y'")), "{r:?}");
        let r = unused_names("y=2\nexport x=\"a \\\" y\"\n");
        assert!(r.iter().any(|m| m.contains("'y'")), "{r:?}");
        let r = unused_names("y=2\nexport x=a\\ y\n");
        assert!(r.iter().any(|m| m.contains("'y'")), "{r:?}");
        let r = unused_names("y=2\nexport x | y\n");
        assert!(r.iter().any(|m| m.contains("'y'")), "{r:?}");
        let r = unused_names("x=1\ny=2\nexport x=a#b y\n");
        assert!(r.iter().any(|m| m.contains("'x'")) == false, "{r:?}");
    }
}
