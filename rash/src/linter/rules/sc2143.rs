// SC2143: Use grep -q instead of comparing grep output to empty string.
//
// Comparing the output of grep to an empty string is inefficient because grep
// processes the entire file. Using grep -q exits on first match.
//
// Examples:
// Bad:
//   [ -z "$(grep pattern file)" ]    // Processes entire file
//   [ -n "$(grep pattern file)" ]    // Processes entire file
//   if [ "$(grep foo bar)" ]; then   // Inefficient
//
// Good:
//   grep -q pattern file              // Exits on first match
//   ! grep -q pattern file            // Negation with -q
//   if grep -q foo bar; then          // Direct conditional
//
// Impact: Performance - grep -q is much faster for large files

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static GREP_IN_TEST_Z: Lazy<Regex> = Lazy::new(|| {
    // Match: [ -z "$(grep ...)" ]
    Regex::new(r#"\[\s+-z\s+"?\$\(grep\s+"#).unwrap()
});

static GREP_IN_TEST_N: Lazy<Regex> = Lazy::new(|| {
    // Match: [ -n "$(grep ...)" ]
    Regex::new(r#"\[\s+-n\s+"?\$\(grep\s+"#).unwrap()
});

static GREP_IN_TEST_DIRECT: Lazy<Regex> = Lazy::new(|| {
    // Match: [ "$(grep ...)" ] or if [ "$(grep ...)" ]
    Regex::new(r#"\[\s+"?\$\(grep\s+"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for [ -z "$(grep ...)" ]
        for mat in GREP_IN_TEST_Z.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2143",
                Severity::Info,
                "Use 'grep -q' instead of comparing output to empty string".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for [ -n "$(grep ...)" ]
        for mat in GREP_IN_TEST_N.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2143",
                Severity::Info,
                "Use 'grep -q' instead of comparing output to empty string".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for [ "$(grep ...)" ]
        // But skip if it's already caught by -z or -n
        if !GREP_IN_TEST_Z.is_match(line) && !GREP_IN_TEST_N.is_match(line) {
            for mat in GREP_IN_TEST_DIRECT.find_iter(line) {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2143",
                    Severity::Info,
                    "Use 'grep -q' directly in conditional instead of capturing output".to_string(),
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
    fn test_sc2143_grep_with_z_flag() {
        let code = r#"[ -z "$(grep pattern file)" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("grep -q"));
    }

    #[test]
    fn test_sc2143_grep_with_n_flag() {
        let code = r#"[ -n "$(grep pattern file)" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("grep -q"));
    }

    #[test]
    fn test_sc2143_grep_direct_test() {
        let code = r#"if [ "$(grep foo bar)" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("grep -q"));
    }

    #[test]
    fn test_sc2143_grep_q_ok() {
        let code = "grep -q pattern file";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2143_grep_q_conditional_ok() {
        let code = "if grep -q pattern file; then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2143_grep_q_negated_ok() {
        let code = "if ! grep -q pattern file; then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2143_comment_ok() {
        let code = r#"# [ -z "$(grep pattern file)" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2143_grep_without_test_ok() {
        let code = r#"result="$(grep pattern file)""#;
        let result = check(code);
        // Assignment is OK, not testing
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2143_multiple() {
        let code = r#"
[ -z "$(grep foo bar)" ]
[ -n "$(grep baz qux)" ]
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2143_without_quotes() {
        let code = r#"[ -z $(grep pattern file) ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
