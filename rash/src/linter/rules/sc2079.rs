// SC2079: (( )) doesn't support decimals. Use bc or awk for floating point
//
// Shell arithmetic expansion $(( )) only supports integer arithmetic.
// Decimal numbers are truncated or cause errors.
//
// Examples:
// Bad:
//   result=$((3.14 * 2))       // Decimals not supported
//   if (( value > 2.5 )); then // Error or wrong result
//
// Good:
//   result=$(echo "3.14 * 2" | bc)         // Use bc
//   result=$(awk "BEGIN {print 3.14 * 2}") // Use awk
//
// Impact: Syntax errors, wrong calculations

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static DECIMAL_IN_ARITHMETIC: Lazy<Regex> = Lazy::new(|| {
    // Match: $(( ... 3.14 ... )) or (( ... 2.5 ... ))
    Regex::new(r"\$?\(\(\s*[^)]*[0-9]+\.[0-9]+[^)]*\)\)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in DECIMAL_IN_ARITHMETIC.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2079",
                Severity::Error,
                "(( )) doesn't support decimals. Use bc or awk for floating point arithmetic"
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
    fn test_sc2079_decimal_in_arithmetic() {
        let code = r#"result=$((3.14 * 2))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2079_decimal_comparison() {
        let code = r#"if (( value > 2.5 )); then echo "yes"; fi"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2079_decimal_pi() {
        let code = r#"circumference=$((2 * 3.14159 * radius))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2079_integer_ok() {
        let code = r#"result=$((3 * 2))"#;
        let result = check(code);
        // Integer arithmetic is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2079_bc_ok() {
        let code = r#"result=$(echo "3.14 * 2" | bc)"#;
        let result = check(code);
        // Using bc for decimals is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2079_awk_ok() {
        let code = r#"result=$(awk 'BEGIN {print 3.14 * 2}')"#;
        let result = check(code);
        // Using awk for decimals is correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2079_comment_ok() {
        let code = r#"# result=$((3.14 * 2))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2079_multiple_decimals() {
        let code = r#"result=$((1.5 + 2.5))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2079_double_paren() {
        let code = r#"(( x = 1.5 ))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2079_version_string() {
        let code = r#"version="1.2.3""#;
        let result = check(code);
        // String assignment, not arithmetic
        assert_eq!(result.diagnostics.len(), 0);
    }
}
