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
}

impl Lexer {
    fn flush_word(&mut self) {
        if self.in_word {
            self.words.push(std::mem::take(&mut self.cur));
            self.in_word = false;
        }
    }

    fn end_command(&mut self) {
        self.flush_word();
        if !self.words.is_empty() {
            self.cmds.push(std::mem::take(&mut self.words));
        }
    }

    fn push_char(&mut self, c: char) {
        self.cur.push(c);
        self.in_word = true;
    }
}

/// Consume a quoted span (opening quote already read) into the current word.
fn lex_quoted(q: char, chars: &mut std::str::Chars<'_>, lx: &mut Lexer) {
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

/// Split a line into simple commands, each a list of words with quotes
/// removed. Stops at an unquoted `#` comment; splits at unquoted `;`, `&`, `|`.
fn split_commands(line: &str) -> Vec<Vec<String>> {
    let mut lx = Lexer::default();
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match c {
            '\'' | '"' => lex_quoted(c, &mut chars, &mut lx),
            '\\' => {
                if let Some(n) = chars.next() {
                    lx.push_char(n);
                }
            }
            '#' if !lx.in_word => break,
            ';' | '&' | '|' => lx.end_command(),
            c if c.is_whitespace() => lx.flush_word(),
            _ => lx.push_char(c),
        }
    }
    lx.end_command();
    lx.cmds
}

fn has_flag(args: &[String], flag: char) -> bool {
    args.iter()
        .any(|w| w.starts_with('-') && !w.starts_with("--") && w.contains(flag))
}

/// Names passed to `export` (or `declare -x` / `typeset -x`) count as used
/// (shellcheck parity). `readonly` and `export -n` do not export.
fn mark_exported(line: &str, used: &mut HashSet<String>) {
    for cmd in split_commands(line) {
        let args = &cmd[1..];
        let exports = match cmd[0].as_str() {
            "export" => !has_flag(args, 'n'),
            "declare" | "typeset" => has_flag(args, 'x'),
            _ => false,
        };
        if !exports {
            continue;
        }
        for word in args.iter().filter(|w| !w.starts_with('-')) {
            let name = word.split('=').next().unwrap_or("");
            if !name.is_empty() {
                used.insert(name.to_string());
            }
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
}
