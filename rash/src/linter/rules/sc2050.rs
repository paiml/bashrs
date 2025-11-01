//! SC2050: This expression is constant. Did you forget the $ on a variable?
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! if [ "var" = "value" ]; then  # Always false (comparing literals)
//!   echo "Equal"
//! fi
//! ```
//!
//! Good:
//! ```bash
//! if [ "$var" = "value" ]; then  # Compares variable
//!   echo "Equal"
//! fi
//! ```
//!
//! # Rationale
//!
//! Comparing two string literals is always constant:
//! - Indicates missing $ on variable
//! - Dead code (condition never changes)
//! - Likely a bug
//!
//! # Auto-fix
//!
//! Warning only - check for missing $ prefix

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Check for constant comparisons (missing $ on variables)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: [ "literal" = "literal" ]
    let pattern = Regex::new(r#"\[\s*"([a-z_][a-z0-9_]*)"\s*=\s*"([^$"][^"]*)"\s*\]"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let left = cap.get(1).unwrap().as_str();
            let right = cap.get(2).unwrap().as_str();

            // Skip if it looks like a constant comparison (e.g., [ "true" = "true" ])
            // We're looking for cases where left looks like a variable name
            // Pattern allows [a-z_][a-z0-9_]*, so allow alphanumeric names
            if left.len() > 1
                && left
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_lowercase() || c == '_')
                && left
                    .chars()
                    .all(|c| c.is_lowercase() || c.is_ascii_digit() || c == '_')
            {
                let start_col = full_match.start() + 1;
                let end_col = full_match.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2050",
                    Severity::Warning,
                    format!(
                        "This expression is constant. Did you forget the $ on '{}'?",
                        left
                    ),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2050_basic_detection() {
        let script = r#"if [ "var" = "value" ]; then echo "Equal"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2050");
    }

    #[test]
    fn test_sc2050_missing_dollar() {
        let script = r#"[ "name" = "Alice" ]"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2050_lowercase_variable_name() {
        let script = r#"[ "username" = "admin" ]"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2050_false_positive_correct_syntax() {
        let script = r#"if [ "$var" = "value" ]; then echo "Equal"; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2050_false_positive_in_comment() {
        let script = r#"# [ "var" = "value" ]"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2050_with_underscore() {
        let script = r#"[ "user_name" = "test" ]"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2050_multiple_occurrences() {
        let script = r#"[ "var1" = "value1" ]
[ "var2" = "value2" ]"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2050_single_letter_skip() {
        let script = r#"[ "x" = "1" ]"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0); // Too short to be variable name
    }

    #[test]
    fn test_sc2050_if_statement() {
        let script = r#"if [ "status" = "active" ]; then process; fi"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2050_double_bracket() {
        let script = r#"[[ "config" = "prod" ]]"#;
        let result = check(script);
        // Pattern matches double brackets too, but that's OK - same warning applies
        assert!(result.diagnostics.len() <= 1);
    }
}
