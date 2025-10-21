// SC2057: Unknown binary operator
//
// Test commands support specific binary operators for string and numeric comparisons.
// Using invalid operators causes syntax errors or unexpected behavior.
//
// Examples:
// Bad:
//   [ "$a" === "$b" ]        // Invalid operator (=== is not valid)
//   [ $x =! $y ]             // Wrong syntax (should be !=)
//   [ $a => $b ]             // Invalid operator (should be -ge)
//   [ "$str" <> "$other" ]   // Not a valid shell operator
//
// Good:
//   [ "$a" = "$b" ]          // String equality
//   [ "$a" == "$b" ]         // String equality (bash)
//   [ "$a" != "$b" ]         // String inequality
//   [ $x -eq $y ]            // Numeric equality
//   [ $x -ne $y ]            // Numeric inequality
//   [ $a -lt $b ]            // Less than (numeric)
//   [ $a -gt $b ]            // Greater than (numeric)
//
// Valid operators:
//   String: =, ==, !=, <, > (in [[]])
//   Numeric: -eq, -ne, -lt, -le, -gt, -ge
//
// Note: = and == are equivalent in [[]] but only = is POSIX in [].

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static INVALID_OPERATORS: Lazy<Regex> = Lazy::new(|| {
    // Match common invalid operators
    Regex::new(r"\[\s+[^\]]*\s+(===|=!|<>|=>|=<)\s+[^\]]*\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for invalid operators in test commands
        for cap in INVALID_OPERATORS.captures_iter(line) {
            let operator = cap.get(1).unwrap().as_str();
            let full_match = cap.get(0).unwrap().as_str();
            let pos = line.find(full_match).unwrap_or(0);

            let start_col = pos + 1;
            let end_col = start_col + full_match.len();

            let suggestion = match operator {
                "===" => "= or ==",
                "=!" => "!=",
                "=>" => "-ge (for numeric) or use [[ ]]",
                "=<" => "-le (for numeric) or use [[ ]]",
                "<>" => "!=",
                _ => "a valid test operator",
            };

            let diagnostic = Diagnostic::new(
                "SC2057",
                Severity::Error,
                format!("Unknown binary operator '{}'. Use {}", operator, suggestion),
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
    fn test_sc2057_triple_equals() {
        let code = r#"[ "$a" === "$b" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2057");
        assert!(result.diagnostics[0].message.contains("==="));
    }

    #[test]
    fn test_sc2057_wrong_not_equal() {
        let code = r#"[ $x =! $y ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("!="));
    }

    #[test]
    fn test_sc2057_greater_equal_wrong() {
        let code = r#"[ $a => $b ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2057_less_equal_wrong() {
        let code = r#"[ $a =< $b ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2057_diamond_operator() {
        let code = r#"[ "$str" <> "$other" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2057_single_equals_ok() {
        let code = r#"[ "$a" = "$b" ]"#;
        let result = check(code);
        // Single = is valid
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2057_double_equals_ok() {
        let code = r#"[ "$a" == "$b" ]"#;
        let result = check(code);
        // Double == is valid in bash
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2057_not_equal_ok() {
        let code = r#"[ "$a" != "$b" ]"#;
        let result = check(code);
        // != is valid
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2057_numeric_operators_ok() {
        let code = r#"[ $x -eq $y ] && [ $a -lt $b ]"#;
        let result = check(code);
        // Numeric operators are valid
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2057_comment_ok() {
        let code = r#"# [ "$a" === "$b" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
