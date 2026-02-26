// SC2169: In dash, [ ... -eq ... ] is undefined for strings
//
// The -eq, -ne, -lt, -le, -gt, -ge operators are for numeric comparison only.
// Using them with non-numeric strings is undefined behavior in dash/POSIX sh.
//
// Examples:
// Bad:
//   if [ "$var" -eq "string" ]; then
//
// Good:
//   if [ "$var" = "string" ]; then  # String comparison
//   if [ "$num" -eq 42 ]; then     # Numeric comparison

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static NUMERIC_OP_WITH_STRING: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [ ... -eq "string" ] or similar
    Regex::new(r#"\[\s+[^\]]*(-eq|-ne|-lt|-le|-gt|-ge)\s+"[A-Za-z_][^\]"]*""#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip double brackets (bash extension)
        if line.contains("[[") {
            continue;
        }

        for cap in NUMERIC_OP_WITH_STRING.captures_iter(line) {
            if let Some(operator) = cap.get(1) {
                let start_col = operator.start() + 1;
                let end_col = operator.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2169",
                    Severity::Warning,
                    format!(
                        "In dash, '{}' is undefined for non-numeric strings. Use '=' or '!=' for string comparison",
                        operator.as_str()
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
    fn test_sc2169_eq_with_string() {
        let code = r#"if [ "$var" -eq "string" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2169");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("-eq"));
    }

    #[test]
    fn test_sc2169_ne_with_string() {
        let code = r#"if [ "$status" -ne "running" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2169_lt_with_string() {
        let code = r#"if [ "$val" -lt "high" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2169_numeric_comparison_ok() {
        let code = r#"if [ "$count" -eq 10 ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2169_string_comparison_ok() {
        let code = r#"if [ "$var" = "string" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2169_variable_vs_number_ok() {
        let code = r#"if [ "$num" -eq 42 ]; then"#;
        let result = check(code);
        // Variable vs number is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2169_double_bracket_ok() {
        let code = r#"if [[ "$var" -eq "string" ]]; then"#;
        let result = check(code);
        // Double brackets are bash extension, allowed
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2169_not_equal() {
        let code = r#"if [ "$state" != "active" ]; then"#;
        let result = check(code);
        // != is string comparison, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2169_multiple_on_line() {
        let code = r#"if [ "$a" -eq "x" ] && [ "$b" -ne "y" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2169_no_quotes_ok() {
        let code = r#"if [ $num -eq $other ]; then"#;
        let result = check(code);
        // No quoted strings, OK
        assert_eq!(result.diagnostics.len(), 0);
    }
}
