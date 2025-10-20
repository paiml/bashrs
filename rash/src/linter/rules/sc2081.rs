//! SC2081: Expressions don't expand in single quotes, use double quotes for that
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! echo 'Value: $var'  # Prints literal $var
//! echo 'Result: $(cmd)'  # Prints literal $(cmd)
//! ```
//!
//! Good:
//! ```bash
//! echo "Value: $var"  # Expands $var
//! echo "Result: $(cmd)"  # Expands $(cmd)
//! echo 'Literal $var'  # OK if literal is intended
//! ```
//!
//! # Rationale
//!
//! Single quotes preserve everything literally:
//! - $var is not expanded
//! - $(cmd) is not executed
//! - \n is literal backslash-n
//!
//! This is often not what the user intended.
//!
//! # Auto-fix
//!
//! Suggest using double quotes if expansion is intended

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for variable/command expansion in single quotes
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: '...$var...' or '...$(cmd)...'
    let pattern =
        Regex::new(r#"'([^']*(?:\$\{?[A-Za-z_][A-Za-z0-9_]*\}?|\$\([^)]+\))[^']*)'"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let content = cap.get(1).unwrap().as_str();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            // Suggest double quotes as fix
            let fix_text = format!("\"{}\"", content);

            let diagnostic = Diagnostic::new(
                "SC2081",
                Severity::Info,
                "Expressions don't expand in single quotes, use double quotes for that",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new(fix_text));

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2081_basic_detection() {
        let script = r#"echo 'Value: $var'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2081");
    }

    #[test]
    fn test_sc2081_autofix() {
        let script = r#"echo 'Value: $var'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "\"Value: $var\""
        );
    }

    #[test]
    fn test_sc2081_command_substitution() {
        let script = r#"echo 'Result: $(date)'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2081_braced_variable() {
        let script = r#"echo 'Name: ${username}'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2081_false_positive_double_quotes() {
        let script = r#"echo "Value: $var""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2081_false_positive_literal_only() {
        let script = r#"echo 'Hello World'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2081_false_positive_in_comment() {
        let script = r#"# echo 'Value: $var'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2081_multiple_variables() {
        let script = r#"echo 'Name: $name, Age: $age'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2081_in_assignment() {
        let script = r#"msg='Error: $error_code'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2081_mixed_content() {
        let script = r#"echo 'Path is $HOME/bin'"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "\"Path is $HOME/bin\""
        );
    }
}
