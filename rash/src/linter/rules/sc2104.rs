// SC2104: Missing space before ]
//
// Detects test expressions missing required whitespace before closing bracket.
// In POSIX shell, [ is a command and ] is its final argument, so spaces are required.
//
// Examples:
// Bad:
//   if [ "$var" = "value"]; then
//
// Good:
//   if [ "$var" = "value" ]; then

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static MISSING_SPACE_BEFORE_BRACKET: Lazy<Regex> = Lazy::new(|| {
    // Match: anything followed by ] without space before it
    Regex::new(r"[^\s\[]\]").unwrap()
});

static TEST_COMMAND: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[\s+").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (i, line) in source.lines().enumerate() {
        let line_num = i + 1;

        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Only check lines with test commands
        if !TEST_COMMAND.is_match(line) {
            continue;
        }

        // Skip double brackets [[...]]
        if line.contains("[[") {
            continue;
        }

        // Find missing spaces before ]
        for mat in MISSING_SPACE_BEFORE_BRACKET.find_iter(line) {
            let match_str = mat.as_str();

            // Skip if next char is ] (this is ]])
            if mat.end() < line.len() && line.chars().nth(mat.end()) == Some(']') {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            // Auto-fix: insert space before ]
            // Match is like "value]" - we need to insert space before ]
            let fixed_match = format!("{} ]", &match_str[..match_str.len() - 1]);
            let fixed_line = format!(
                "{}{}{}",
                &line[..mat.start()],
                fixed_match,
                &line[mat.end()..]
            );

            let diagnostic = Diagnostic::new(
                "SC2104",
                Severity::Error,
                "Missing space before ]",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new(fixed_line));

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2104_missing_space_basic() {
        let code = r#"if [ "$var" = "value"]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2104");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].fix.is_some());
    }

    #[test]
    fn test_sc2104_autofix() {
        let code = r#"if [ "$var" = "value"]; then"#;
        let result = check(code);
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains(" ]"));
        assert!(!fix.replacement.contains("\"]\"")); // Should not have "]" without space
        assert!(fix.replacement.contains("\" ]")); // Should have " ]" with space
    }

    #[test]
    fn test_sc2104_correct_spacing_ok() {
        let code = r#"if [ "$var" = "value" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2104_double_bracket_ok() {
        let code = r#"if [[ "$var" = "value"]]; then"#;
        let result = check(code);
        // Should not trigger on [[...]] (bash extended test)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2104_numeric_comparison() {
        let code = r#"if [ "$count" -eq 10]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2104_string_comparison() {
        let code = r#"if [ "$str" != "test"]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2104_file_test() {
        let code = r#"if [ -f "$file"]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2104_multiple_conditions() {
        let code = r#"if [ "$a" = "1"] && [ "$b" = "2" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2104_no_test_command() {
        let code = r#"echo "array[0]""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2104_array_subscript_ok() {
        let code = r#"echo "${array[0]}""#;
        let result = check(code);
        // Should not trigger on array subscripts
        assert_eq!(result.diagnostics.len(), 0);
    }
}
