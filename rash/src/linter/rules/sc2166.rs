// SC2166: Prefer [ p ] && [ q ] as [ p -a q ] is not well defined
//
// Detects usage of deprecated -a (and) and -o (or) operators within test commands.
// These operators are obsolete and poorly defined in POSIX. Use separate tests instead.
//
// Examples:
// Bad:
//   if [ "$a" = "1" -a "$b" = "2" ]; then
//   if [ "$x" -eq 1 -o "$y" -eq 2 ]; then
//
// Good:
//   if [ "$a" = "1" ] && [ "$b" = "2" ]; then
//   if [ "$x" -eq 1 ] || [ "$y" -eq 2 ]; then

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static DEPRECATED_AND_OR: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match -a or -o within test commands
    Regex::new(r"\[\s+[^\]]*\s+(-a|-o)\s+[^\]]*\]").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (i, line) in source.lines().enumerate() {
        let line_num = i + 1;

        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip double brackets [[ ]] (they don't use -a/-o)
        if line.contains("[[") {
            continue;
        }

        // Detect -a or -o operators
        if let Some(cap) = DEPRECATED_AND_OR.captures(line) {
            if let Some(operator) = cap.get(1) {
                let start_col = operator.start() + 1;
                let end_col = operator.end() + 1;

                let replacement = if operator.as_str() == "-a" {
                    "] && ["
                } else {
                    "] || ["
                };

                let diagnostic = Diagnostic::new(
                    "SC2166",
                    Severity::Warning,
                    format!(
                        "Prefer separate test commands: {} is deprecated (use '{}' instead)",
                        operator.as_str(),
                        replacement
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
    fn test_sc2166_deprecated_and() {
        let code = r#"if [ "$a" = "1" -a "$b" = "2" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2166");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("-a"));
    }

    #[test]
    fn test_sc2166_deprecated_or() {
        let code = r#"if [ "$a" = "1" -o "$b" = "2" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("-o"));
    }

    #[test]
    fn test_sc2166_separate_tests_ok() {
        let code = r#"if [ "$a" = "1" ] && [ "$b" = "2" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2166_double_bracket_ok() {
        let code = r#"if [[ "$a" = "1" && "$b" = "2" ]]; then"#;
        let result = check(code);
        // Double brackets use && not -a
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2166_file_tests() {
        let code = r#"if [ -f "$file1" -a -r "$file2" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2166_numeric_comparison() {
        let code = r#"if [ "$count" -eq 10 -o "$count" -eq 20 ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2166_single_condition_ok() {
        let code = r#"if [ "$a" = "1" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2166_negation_ok() {
        let code = r#"if [ ! -f "$file" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2166_complex_multiple_and() {
        let code = r#"if [ "$a" = "1" -a "$b" = "2" -a "$c" = "3" ]; then"#;
        let result = check(code);
        // Should detect at least one -a
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn test_sc2166_no_test_command() {
        let code = r#"echo "test -a test""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
