// SC2170: Numerical -gt, -eq, etc. operators only work on integers. Use string operators like = instead.
//
// When comparing strings, use string comparison operators (=, !=, <, >).
// Numeric operators (-eq, -ne, -lt, -le, -gt, -ge) should only be used with integers.
//
// Examples:
// Bad:
//   if [ "$string" -gt "abc" ]; then  # -gt is numeric, not for strings
//   if [ "$version" -eq "1.2.3" ]; then  # Version string, not integer
//
// Good:
//   if [ "$string" = "abc" ]; then    # String comparison
//   if [ "$num" -gt 10 ]; then        # Numeric comparison with integer

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static NUMERIC_OP_LIKELY_STRING: Lazy<Regex> = Lazy::new(|| {
    // Match: numeric operators with quoted strings containing non-digits
    // Looking for patterns like: "string" -gt "other" or "$var" -eq "text"
    Regex::new(r#"(-eq|-ne|-lt|-le|-gt|-ge)\s+"([^"]*[A-Za-z_\.\-][^"]*)""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip double brackets (bash extension with different semantics)
        if line.contains("[[") {
            continue;
        }

        for cap in NUMERIC_OP_LIKELY_STRING.captures_iter(line) {
            if let Some(operator) = cap.get(1) {
                if let Some(value) = cap.get(2) {
                    let val_text = value.as_str();

                    // Skip if it looks like a pure number or variable expansion
                    if val_text.chars().all(|c| c.is_ascii_digit() || c == '-') {
                        continue;
                    }

                    // Skip if starts with ${ or $ (variable expansion)
                    if val_text.starts_with('$') {
                        continue;
                    }

                    let start_col = operator.start() + 1;
                    let end_col = operator.end() + 1;

                    let diagnostic = Diagnostic::new(
                        "SC2170",
                        Severity::Warning,
                        format!(
                            "Numerical {} operator used with string \"{}\". Use string operators like = instead",
                            operator.as_str(),
                            if val_text.len() > 20 {
                                format!("{}...", &val_text[..20])
                            } else {
                                val_text.to_string()
                            }
                        ),
                        Span::new(line_num, start_col, line_num, end_col),
                    );

                    result.add(diagnostic);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2170_string_with_numeric_op() {
        let code = r#"if [ "$var" -gt "abc" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2170");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("-gt"));
        assert!(result.diagnostics[0].message.contains("abc"));
    }

    #[test]
    fn test_sc2170_version_string() {
        let code = r#"if [ "$version" -eq "1.2.3" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("1.2.3"));
    }

    #[test]
    fn test_sc2170_numeric_comparison_ok() {
        let code = r#"if [ "$count" -gt 10 ]; then"#;
        let result = check(code);
        // Pure number is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2170_string_comparison_ok() {
        let code = r#"if [ "$string" = "abc" ]; then"#;
        let result = check(code);
        // String operator used correctly
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2170_variable_expansion_ok() {
        let code = r#"if [ "$a" -gt "$b" ]; then"#;
        let result = check(code);
        // Variable expansion - could be numbers
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2170_double_bracket_ok() {
        let code = r#"if [[ "$var" -gt "text" ]]; then"#;
        let result = check(code);
        // Double brackets have different semantics
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2170_multiple_operators() {
        let code = r#"if [ "$a" -eq "text" ] && [ "$b" -ne "other" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2170_negative_number_ok() {
        let code = r#"if [ "$val" -lt -5 ]; then"#;
        let result = check(code);
        // Negative number is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2170_long_string() {
        let code = r#"if [ "$x" -gt "this is a very long string value" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("..."));
    }

    #[test]
    fn test_sc2170_path_string() {
        let code = r#"if [ "$path" -eq "/usr/bin" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
