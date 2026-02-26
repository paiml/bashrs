// SC2055: You probably wanted && here, not -a
//
// The -a operator in test commands is deprecated and non-portable.
// It combines multiple test conditions but is obsolete in modern shells.
// Use && to chain test commands or [[ ]] for complex conditions.
//
// Examples:
// Bad:
//   [ $a -eq 1 -a $b -eq 2 ]         // Deprecated -a operator
//   [ -f file -a -r file ]           // Hard to read, deprecated
//   test $x -gt 0 -a $x -lt 10       // POSIX discourages this
//
// Good:
//   [ $a -eq 1 ] && [ $b -eq 2 ]     // Chained tests (POSIX)
//   [[ $a -eq 1 && $b -eq 2 ]]       // Bash native && (preferred)
//   [ -f file ] && [ -r file ]       // Clear intent
//
// Rationale:
//   - POSIX marks -a/-o as obsolete
//   - Precedence rules are confusing
//   - && is clearer and more portable
//   - [[ ]] provides better operators in bash/ksh
//
// Note: -a for file existence ([ -a file ]) is different and correct.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static TEST_WITH_AND: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [ ... -a ... ] or test ... -a ...
    // Look for -a operator between test conditions
    Regex::new(r"\[\s+[^\]]*\s+-a\s+[^\]]*\]|test\s+[^\n]*\s+-a\s+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip [[ ]] tests (they use && natively)
        if line.contains("[[") {
            continue;
        }

        // Look for -a operator in test commands
        for mat in TEST_WITH_AND.find_iter(line) {
            let matched = mat.as_str();

            // Verify it's the logical -a, not file test -a
            // File test would be: [ -a filename ]
            // Logical would be: [ condition -a condition ]
            if matched.contains("-a ") && !matched.starts_with("[ -a ") {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2055",
                    Severity::Warning,
                    "You probably wanted && here, not -a (which is obsolete). Use [ cond ] && [ cond ]".to_string(),
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
    fn test_sc2055_and_operator_in_test() {
        let code = r#"[ $a -eq 1 -a $b -eq 2 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2055");
    }

    #[test]
    fn test_sc2055_file_tests_with_and() {
        let code = r#"[ -f file -a -r file ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2055_test_command() {
        let code = r#"test $x -gt 0 -a $x -lt 10"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2055_chained_tests_ok() {
        let code = r#"[ $a -eq 1 ] && [ $b -eq 2 ]"#;
        let result = check(code);
        // Chained with &&, correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2055_double_bracket_ok() {
        let code = r#"[[ $a -eq 1 && $b -eq 2 ]]"#;
        let result = check(code);
        // [[ ]] uses native &&, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2055_file_existence_ok() {
        let code = r#"[ -a file ]"#;
        let result = check(code);
        // -a for file existence (not logical AND), OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2055_comment_ok() {
        let code = r#"# [ $a -eq 1 -a $b -eq 2 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2055_multiple_conditions() {
        let code = r#"[ $a -eq 1 -a $b -eq 2 -a $c -eq 3 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2055_if_statement() {
        let code = r#"if [ $status -eq 0 -a $ready = "yes" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2055_separate_tests_ok() {
        let code = r#"[ -f file ] && [ -r file ]"#;
        let result = check(code);
        // Separate tests chained with &&, correct
        assert_eq!(result.diagnostics.len(), 0);
    }
}
