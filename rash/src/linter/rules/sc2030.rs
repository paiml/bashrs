// SC2030: Variable modified in subshell. Use var=$(cmd) or avoid subshell.
//
// Variables assigned inside subshells (parentheses) don't affect the parent shell.
// The assignment is local to the subshell and lost when it exits.
//
// Examples:
// Bad:
//   (foo=bar)
//   echo "$foo"  # Empty, modification was in subshell
//
//   result=$(x=5; echo "$x")  # x assignment lost
//
// Good:
//   foo=bar      # Direct assignment in current shell
//   echo "$foo"
//
//   x=5; echo "$x"  # No subshell
//
// Note: This rule detects variable assignments inside ( ) subshells.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static SUBSHELL_ASSIGNMENT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: ( ... var=value ... )
    // Look for variable assignments inside parentheses
    Regex::new(r"\(\s*([a-zA-Z_][a-zA-Z0-9_]*)=").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for variable assignments in subshells
        for cap in SUBSHELL_ASSIGNMENT.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str();

            // Skip if inside quotes
            let full_match = cap.get(0).unwrap().as_str();
            let pos = line.find(full_match).unwrap_or(0);
            let before = &line[..pos];
            let quote_count = before.matches('"').count() + before.matches('\'').count();
            if quote_count % 2 == 1 {
                continue;
            }

            // Skip command substitution $( ... ) - not a subshell
            if pos > 0 && line.chars().nth(pos - 1) == Some('$') {
                continue;
            }

            let start_col = pos + 1;
            let end_col = start_col + full_match.len();

            let diagnostic = Diagnostic::new(
                "SC2030",
                Severity::Warning,
                format!(
                    "Variable '{}' modified in subshell. Use var=$(cmd) to capture output or remove subshell",
                    var_name
                ),
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
    fn test_sc2030_simple_subshell_assignment() {
        let code = r#"(foo=bar)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2030");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("foo"));
    }

    #[test]
    fn test_sc2030_subshell_with_command() {
        let code = r#"(x=5; echo "$x")"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("x"));
    }

    #[test]
    fn test_sc2030_direct_assignment_ok() {
        let code = r#"foo=bar"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2030_command_subst_ok() {
        let code = r#"result=$(x=5; echo "$x")"#;
        let result = check(code);
        // Command substitution is OK (output captured)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2030_nested_parens() {
        let code = r#"((foo=bar))"#;
        let result = check(code);
        // Arithmetic (( )) or nested subshell
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2030_in_quotes_ok() {
        let code = r#"echo "(foo=bar)""#;
        let result = check(code);
        // Inside quotes, not actual subshell
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2030_multiple_assignments() {
        let code = r#"
(a=1)
(b=2)
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2030_with_spaces() {
        let code = r#"( foo=bar )"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2030_comment_ok() {
        let code = r#"# (foo=bar)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2030_export_in_subshell() {
        let code = r#"(export PATH=/usr/bin)"#;
        let result = check(code);
        // export is not caught by simple assignment pattern (would need separate check)
        assert_eq!(result.diagnostics.len(), 0);
    }
}
