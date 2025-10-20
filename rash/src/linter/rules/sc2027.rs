//! SC2027: Wrong quoting in printf format strings
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! printf "Hello $name\n"
//! ```
//!
//! Good:
//! ```bash
//! printf "Hello %s\n" "$name"
//! printf 'Hello %s\n' "$name"
//! ```
//!
//! # Rationale
//!
//! Printf format strings should use format specifiers, not direct variable expansion:
//! - Prevents injection attacks
//! - Handles special characters correctly
//! - Follows printf conventions
//!
//! # Auto-fix
//!
//! Suggest using printf format specifiers

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Check for wrong quoting in printf format strings
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: printf "...$var..." or printf '...$var...'
    let pattern =
        Regex::new(r#"printf\s+["']([^"']*\$\{?[A-Za-z_][A-Za-z0-9_]*\}?[^"']*)["']"#).unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let format_str = cap.get(1).unwrap().as_str();

            // Skip if it's already using format specifiers (%s, %d, etc.)
            if format_str.contains("%s") || format_str.contains("%d") || format_str.contains("%f") {
                continue;
            }

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2027",
                Severity::Warning,
                "The surrounding quotes actually unquote this. Use printf '%s\\n' \"$var\" instead",
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
    fn test_sc2027_basic_detection() {
        let script = r#"printf "Hello $name\n""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2027");
    }

    #[test]
    fn test_sc2027_variable_in_format() {
        let script = r#"printf "Value: $value\n""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2027_braced_variable() {
        let script = r#"printf "Name is ${name}\n""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2027_false_positive_correct_format() {
        let script = r#"printf "Hello %s\n" "$name""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2027_false_positive_single_quotes() {
        let script = r#"printf 'Hello %s\n' "$name""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2027_false_positive_in_comment() {
        let script = r#"# printf "Hello $name\n""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2027_multiple_variables() {
        let script = r#"printf "Name: $name, Age: $age\n""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2027_with_integer_format() {
        let script = r#"printf "Count %d\n" "$count""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2027_with_float_format() {
        let script = r#"printf "Value %f\n" "$val""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2027_literal_only() {
        let script = r#"printf "Hello World\n""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
