// SC2160: Instead of '! [ -e x ]', use '[ ! -e x ]'
//
// Negation should be inside the test brackets for better portability and clarity.
// Placing ! outside the brackets can lead to issues with subshell execution and quoting.
//
// Examples:
// Bad:
//   if ! [ -e file ]; then
//   if ! [ -f "$path" ]; then
//
// Good:
//   if [ ! -e file ]; then
//   if [ ! -f "$path" ]; then

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static NEGATION_OUTSIDE_BRACKETS: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: ! [ -e x ] or ! [ -f x ] etc.
    Regex::new(r"!\s+\[").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip double brackets (bash-specific, different semantics)
        if line.contains("[[") {
            continue;
        }

        for mat in NEGATION_OUTSIDE_BRACKETS.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2160",
                Severity::Info,
                "Instead of '! [ ... ]', use '[ ! ... ]' for better portability",
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
    fn test_sc2160_negation_outside() {
        let code = r#"if ! [ -e file ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2160");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("[ !"));
    }

    #[test]
    fn test_sc2160_negation_inside_ok() {
        let code = r#"if [ ! -e file ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2160_multiple_tests() {
        let code = r#"if ! [ -f "$a" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2160_double_bracket_ok() {
        let code = r#"if ! [[ -e file ]]; then"#;
        let result = check(code);
        // Double brackets have different semantics, skip
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2160_negated_command_ok() {
        let code = r#"if ! grep pattern file; then"#;
        let result = check(code);
        // Not a test bracket, just negated command
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2160_string_test() {
        let code = r#"if ! [ -z "$var" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2160_numeric_test() {
        let code = r#"if ! [ "$count" -eq 0 ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2160_multiple_on_line() {
        let code = r#"if ! [ -e a ] && ! [ -e b ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2160_extra_spaces() {
        let code = r#"if !  [ -f file ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2160_no_negation_ok() {
        let code = r#"if [ -e file ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
