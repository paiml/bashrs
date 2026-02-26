// SC2107: Instead of [ a && b ], use [ a ] && [ b ]
//
// Detects usage of && or || inside single brackets, which is not POSIX-compliant.
// In POSIX shell, use separate test commands with shell && or ||.
//
// Examples:
// Bad:
//   if [ "$a" = "1" && "$b" = "2" ]; then
//
// Good:
//   if [ "$a" = "1" ] && [ "$b" = "2" ]; then

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static LOGICAL_IN_SINGLE_BRACKET: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match [ ... && ... ] or [ ... || ... ]
    Regex::new(r"\[\s+[^\]]*(?:&&|\|\|)[^\]]*\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (i, line) in source.lines().enumerate() {
        let line_num = i + 1;

        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip double brackets [[ ]]
        if line.contains("[[") {
            continue;
        }

        // Detect && or || inside single brackets
        if let Some(mat) = LOGICAL_IN_SINGLE_BRACKET.find(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let operator = if mat.as_str().contains("&&") {
                "&&"
            } else {
                "||"
            };

            let diagnostic = Diagnostic::new(
                "SC2107",
                Severity::Error,
                format!(
                    "Use separate test commands: [ a ] {} [ b ] instead of [ a {} b ]",
                    operator, operator
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
    fn test_sc2107_and_in_single_bracket() {
        let code = r#"if [ "$a" = "1" && "$b" = "2" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2107");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("&&"));
    }

    #[test]
    fn test_sc2107_or_in_single_bracket() {
        let code = r#"if [ "$a" = "1" || "$b" = "2" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("||"));
    }

    #[test]
    fn test_sc2107_separate_tests_ok() {
        let code = r#"if [ "$a" = "1" ] && [ "$b" = "2" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2107_double_bracket_ok() {
        let code = r#"if [[ "$a" = "1" && "$b" = "2" ]]; then"#;
        let result = check(code);
        // Double brackets allow && (bash extension)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2107_single_test_ok() {
        let code = r#"if [ "$a" = "1" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2107_complex_condition() {
        let code = r#"if [ "$a" -eq 1 && "$b" -gt 2 ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2107_file_tests() {
        let code = r#"if [ -f "$file1" && -r "$file2" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2107_negation_ok() {
        let code = r#"if [ ! -f "$file" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2107_no_test_command() {
        let code = r#"echo "test && test""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2107_multiple_ands() {
        let code = r#"if [ "$a" = "1" && "$b" = "2" && "$c" = "3" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
