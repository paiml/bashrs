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

/// Check if a line is a comment
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if line contains double brackets
fn has_double_bracket(line: &str) -> bool {
    line.contains("[[")
}

/// Check if value is a pure number
fn is_pure_number(val: &str) -> bool {
    val.chars().all(|c| c.is_ascii_digit() || c == '-')
}

/// Check if value is a variable expansion
fn is_variable_expansion(val: &str) -> bool {
    val.starts_with('$')
}

/// Format string value with truncation if needed
fn format_string_preview(val: &str) -> String {
    if val.len() > 20 {
        format!("{}...", &val[..20])
    } else {
        val.to_string()
    }
}

/// Create diagnostic for numeric operator with string
fn create_numeric_operator_diagnostic(
    operator: &str,
    value: &str,
    line_num: usize,
    start_col: usize,
    end_col: usize,
) -> Diagnostic {
    Diagnostic::new(
        "SC2170",
        Severity::Warning,
        format!(
            "Numerical {} operator used with string \"{}\". Use string operators like = instead",
            operator,
            format_string_preview(value)
        ),
        Span::new(line_num, start_col, line_num, end_col),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if is_comment_line(line) || has_double_bracket(line) {
            continue;
        }

        for cap in NUMERIC_OP_LIKELY_STRING.captures_iter(line) {
            if let Some(operator) = cap.get(1) {
                if let Some(value) = cap.get(2) {
                    let val_text = value.as_str();

                    if is_pure_number(val_text) || is_variable_expansion(val_text) {
                        continue;
                    }

                    let start_col = operator.start() + 1;
                    let end_col = operator.end() + 1;

                    let diagnostic = create_numeric_operator_diagnostic(
                        operator.as_str(),
                        val_text,
                        line_num,
                        start_col,
                        end_col,
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

    // ===== Manual Property Tests =====

    #[test]
    fn prop_sc2170_comments_never_diagnosed() {
        // Property: Comment lines should never produce diagnostics
        let test_cases = vec![
            "# if [ \"$var\" -gt \"text\" ]; then",
            "  # if [ \"$version\" -eq \"1.2.3\" ]; then",
            "\t# [ \"$x\" -ne \"abc\" ]",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2170_double_brackets_never_diagnosed() {
        // Property: Double brackets [[ ]] should never be diagnosed
        let test_cases = vec![
            "[[ \"$var\" -gt \"text\" ]]",
            "[[ \"$version\" -eq \"1.2.3\" ]]",
            "if [[ \"$x\" -ne \"abc\" ]]; then",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2170_pure_numbers_never_diagnosed() {
        // Property: Pure numeric values should never be diagnosed
        let test_cases = vec![
            "[ \"$count\" -gt 10 ]",
            "[ \"$val\" -eq 42 ]",
            "[ \"$num\" -lt -5 ]",
            "[ \"$x\" -ge 0 ]",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2170_variable_expansions_never_diagnosed() {
        // Property: Variable expansions should never be diagnosed
        let test_cases = vec![
            "[ \"$a\" -gt \"$b\" ]",
            "[ \"$x\" -eq \"$count\" ]",
            "[ \"$val\" -lt \"${max}\" ]",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2170_string_operators_never_diagnosed() {
        // Property: String operators should never be diagnosed
        let test_cases = vec![
            "[ \"$var\" = \"text\" ]",
            "[ \"$x\" != \"abc\" ]",
            "[ \"$string\" = \"value\" ]",
        ];

        for code in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 0);
        }
    }

    #[test]
    fn prop_sc2170_numeric_op_with_strings_always_diagnosed() {
        // Property: Numeric operators with strings should always be diagnosed
        let test_cases = vec![
            ("[ \"$var\" -gt \"abc\" ]", "abc", "-gt"),
            ("[ \"$version\" -eq \"1.2.3\" ]", "1.2.3", "-eq"),
            ("[ \"$path\" -ne \"/usr/bin\" ]", "/usr/bin", "-ne"),
            ("[ \"$x\" -lt \"text\" ]", "text", "-lt"),
        ];

        for (code, string_val, operator) in test_cases {
            let result = check(code);
            assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
            assert!(result.diagnostics[0].message.contains(string_val));
            assert!(result.diagnostics[0].message.contains(operator));
        }
    }

    #[test]
    fn prop_sc2170_multiple_violations_all_diagnosed() {
        // Property: Multiple violations should all be diagnosed
        let code = "[ \"$a\" -eq \"text\" ] && [ \"$b\" -ne \"other\" ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn prop_sc2170_diagnostic_code_always_sc2170() {
        // Property: All diagnostics must have code \"SC2170\"
        let code = "[ \"$a\" -gt \"text\" ] && [ \"$b\" -lt \"other\" ]";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(&diagnostic.code, "SC2170");
        }
    }

    #[test]
    fn prop_sc2170_diagnostic_severity_always_warning() {
        // Property: All diagnostics must be Warning severity
        let code = "[ \"$var\" -eq \"string\" ]";
        let result = check(code);

        for diagnostic in &result.diagnostics {
            assert_eq!(diagnostic.severity, Severity::Warning);
        }
    }

    #[test]
    fn prop_sc2170_empty_source_no_diagnostics() {
        // Property: Empty source should produce no diagnostics
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    // ===== Original Unit Tests =====

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
