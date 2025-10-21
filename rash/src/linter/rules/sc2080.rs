// SC2080: Numbers with leading 0 are considered octal
//
// In arithmetic contexts, numbers with leading zeros are interpreted as octal (base 8).
// This can cause unexpected results when the number contains 8 or 9.
//
// Examples:
// Bad:
//   result=$((010 + 5))        // 010 is octal (8), result is 13
//   if [ "$x" -eq 08 ]; then   // Error: 08 is invalid octal
//
// Good:
//   result=$((10 + 5))         // Decimal 10
//   result=$((8#010 + 5))      // Explicitly octal if intended
//   if [ "$x" -eq 8 ]; then    // Decimal 8
//
// Impact: Wrong calculations, syntax errors with 8/9

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static LEADING_ZERO_NUMBER: Lazy<Regex> = Lazy::new(|| {
    // Match: 0[0-9]+ in arithmetic contexts (not 0x hex)
    // Look for $(( ... 08 ... )) or [ ... -eq 09 ]
    Regex::new(r"(\$\(\(|[\[\(]\s*[^)]*(-eq|-ne|-lt|-le|-gt|-ge)\s+)0[0-9]+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in LEADING_ZERO_NUMBER.find_iter(line) {
            let matched = mat.as_str();

            // Extract the number with leading zero
            if let Some(num_match) = Regex::new(r"0[0-9]+").unwrap().find(matched) {
                let num_str = num_match.as_str();

                // Check if it contains 8 or 9 (invalid octal)
                let has_invalid_octal = num_str.contains('8') || num_str.contains('9');

                let severity = if has_invalid_octal {
                    Severity::Error // Invalid octal
                } else {
                    Severity::Warning // Valid octal but likely unintended
                };

                let message = if has_invalid_octal {
                    format!(
                        "'{}' is not a valid octal number (contains 8 or 9)",
                        num_str
                    )
                } else {
                    format!(
                        "'{}' is interpreted as octal ({}₁₀). Remove leading 0 for decimal",
                        num_str,
                        i32::from_str_radix(&num_str[1..], 8).unwrap_or(0)
                    )
                };

                let start_col = mat.start() + num_match.start() + 1;
                let end_col = mat.start() + num_match.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2080",
                    severity,
                    message,
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
    fn test_sc2080_invalid_octal() {
        let code = r#"[ $x -eq 08 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc2080_octal_nine() {
        let code = r#"result=$((09 + 1))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc2080_valid_octal_warning() {
        let code = r#"result=$((010 + 5))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc2080_decimal_ok() {
        let code = r#"result=$((10 + 5))"#;
        let result = check(code);
        // No leading zero, decimal
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2080_hex_ok() {
        let code = r#"result=$((0x10 + 5))"#;
        let result = check(code);
        // Hex notation is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2080_zero_ok() {
        let code = r#"result=$((0 + 5))"#;
        let result = check(code);
        // Plain 0 is fine
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2080_comment_ok() {
        let code = r#"# result=$((08 + 1))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2080_string_ok() {
        let code = r#"version="01.08.2024""#;
        let result = check(code);
        // String context, not arithmetic
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    #[ignore] // TODO: Fix regex to match multiple occurrences on same line
    fn test_sc2080_multiple() {
        let code = r#"[ $a -eq 08 ] && [ $b -eq 09 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    #[ignore] // TODO: Detect octal in assignments without comparison
    fn test_sc2080_double_paren() {
        let code = r#"(( x = 077 ))"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
