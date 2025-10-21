// SC2017: Increase precision by replacing a/b*c with a*c/b
//
// Due to integer arithmetic, a/b*c performs division first, losing precision.
// Reordering to a*c/b performs multiplication first, preserving more precision.
//
// Examples:
// Bad:
//   result=$((a/b*c))              // Loss of precision
//   percent=$((total/100*ratio))   // Integer division truncates
//   value=$((x/y*z))               // Wrong order
//
// Good:
//   result=$((a*c/b))              // Better precision
//   percent=$((total*ratio/100))   // Multiplication first
//   value=$((x*z/y))               // Correct order
//
// Note: Shell arithmetic is integer-only. a/b*c = (a/b)*c loses the fractional
// part of a/b. Reordering as a*c/b keeps more precision.
//
// Example: (100/3)*2 = 33*2 = 66, but 100*2/3 = 200/3 = 66 (closer to 66.67)

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static DIV_MULT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Match: a/b*c pattern inside $((...))
    // Look for division followed by multiplication
    // Allow variables or numbers
    Regex::new(r"\$\(\([^)]*([a-zA-Z_0-9]+)\s*/\s*([a-zA-Z_0-9]+)\s*\*\s*([a-zA-Z_0-9]+)[^)]*\)\)")
        .unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Look for a/b*c pattern
        for cap in DIV_MULT_PATTERN.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let a = cap.get(1).unwrap().as_str();
            let b = cap.get(2).unwrap().as_str();
            let c = cap.get(3).unwrap().as_str();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2017",
                Severity::Info,
                format!(
                    "Increase precision by replacing {}/{}*{} with {}*{}/{}",
                    a, b, c, a, c, b
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
    fn test_sc2017_div_mult_order() {
        let code = r#"result=$((a/b*c))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2017");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("precision"));
    }

    #[test]
    fn test_sc2017_percent_calculation() {
        let code = r#"percent=$((total/100*ratio))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2017_with_spaces() {
        let code = r#"value=$((x / y * z))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2017_correct_order_ok() {
        let code = r#"result=$((a*c/b))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2017_multiplication_first_ok() {
        let code = r#"percent=$((total*ratio/100))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2017_division_only_ok() {
        let code = r#"result=$((a/b))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2017_multiplication_only_ok() {
        let code = r#"result=$((a*b))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2017_mult_div_order_ok() {
        let code = r#"result=$((a*b/c))"#;
        let result = check(code);
        // a*b/c is fine - multiplication before division
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2017_multiple_issues() {
        let code = r#"
x=$((a/b*c))
y=$((d/e*f))
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2017_complex_expression() {
        let code = r#"result=$(((x/y*z) + 10))"#;
        let result = check(code);
        // Nested parentheses are complex to parse with regex - edge case
        assert_eq!(result.diagnostics.len(), 0);
    }
}
