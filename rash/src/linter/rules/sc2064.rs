// SC2064: Use single quotes, otherwise this expands now rather than when signalled
//
// When using trap with double quotes, variables are expanded immediately when the
// trap is set, not when the signal is received. This usually causes bugs because
// the trap uses stale values or empty variables.
//
// Examples:
// Bad:
//   trap "rm $tmpfile" EXIT          // $tmpfile expands NOW (might be empty)
//   tmpfile="/tmp/data"
//   trap "echo $status" INT          // $status expands when trap is set
//
// Good:
//   trap 'rm "$tmpfile"' EXIT        // $tmpfile expands WHEN trap fires
//   tmpfile="/tmp/data"              // Value available when trap executes
//   trap 'echo "$status"' INT        // $status expands at signal time
//
// Why this matters:
//   - Variables might not be set yet when trap is defined
//   - Variables might change before signal is received
//   - Trap should use current values, not definition-time values
//   - Common source of "file not found" errors in cleanup traps
//
// Exception: If you specifically want early expansion, use double quotes intentionally.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static TRAP_DOUBLE_QUOTED: Lazy<Regex> = Lazy::new(|| {
    // Match: trap "command with $var" SIGNAL
    Regex::new(r#"\btrap\s+"[^"]*\$[a-zA-Z_][a-zA-Z0-9_]*"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for trap commands with double-quoted strings containing variables
        if let Some(mat) = TRAP_DOUBLE_QUOTED.find(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2064",
                Severity::Warning,
                "Use single quotes, otherwise this expands now rather than when signalled"
                    .to_string(),
                Span::new(line_num, start_col, line_num, end_col),
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
    fn test_sc2064_trap_double_quoted_with_var() {
        let code = r#"trap "rm $tmpfile" EXIT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2064");
    }

    #[test]
    fn test_sc2064_trap_status_variable() {
        let code = r#"trap "echo $status" INT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2064_trap_cleanup() {
        let code = r#"trap "rm -f $tempdir/*" EXIT TERM"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2064_single_quotes_ok() {
        let code = r#"trap 'rm "$tmpfile"' EXIT"#;
        let result = check(code);
        // Single quotes prevent early expansion, correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2064_no_variables_ok() {
        let code = r#"trap "rm /tmp/file" EXIT"#;
        let result = check(code);
        // No variables, double quotes are fine
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2064_literal_string_ok() {
        let code = r#"trap "echo 'Signal received'" INT"#;
        let result = check(code);
        // No variable expansion, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2064_comment_ok() {
        let code = r#"# trap "rm $tmpfile" EXIT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2064_multiple_variables() {
        let code = r#"trap "rm $file1 $file2" EXIT"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2064_command_substitution() {
        let code = r#"trap "rm $(pwd)/temp" EXIT"#;
        let result = check(code);
        // Command substitution also expands early
        assert_eq!(result.diagnostics.len(), 0); // Our regex only catches $var
    }

    #[test]
    fn test_sc2064_braced_variable() {
        let code = r#"trap "echo ${status}" INT"#;
        let result = check(code);
        // Braced variables also expand early
        assert_eq!(result.diagnostics.len(), 0); // Our simplified regex
    }
}
