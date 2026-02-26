// SC2056: You probably wanted || here, not -o
//
// The -o operator in test commands is deprecated and non-portable.
// It performs logical OR but is obsolete in modern shells.
// Use || to chain test commands or [[ ]] for complex conditions.
//
// Examples:
// Bad:
//   [ $a -eq 1 -o $b -eq 2 ]         // Deprecated -o operator
//   [ ! -f file -o ! -r file ]       // Hard to read, deprecated
//   test $x -lt 0 -o $x -gt 10       // POSIX discourages this
//
// Good:
//   [ $a -eq 1 ] || [ $b -eq 2 ]     // Chained tests (POSIX)
//   [[ $a -eq 1 || $b -eq 2 ]]       // Bash native || (preferred)
//   [ ! -f file ] || [ ! -r file ]   // Clear intent
//
// Rationale:
//   - POSIX marks -a/-o as obsolete
//   - Precedence rules with ! are confusing
//   - || is clearer and more portable
//   - [[ ]] provides better operators in bash/ksh
//
// Note: Don't confuse with shell -o option (set -o, shopt -o).

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static TEST_WITH_OR: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: [ ... -o ... ] or test ... -o ...
    // Look for -o operator between test conditions
    Regex::new(r"\[\s+[^\]]*\s+-o\s+[^\]]*\]|test\s+[^\n]*\s+-o\s+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip [[ ]] tests (they use || natively)
        if line.contains("[[") {
            continue;
        }

        // Skip lines with "set -o" or "shopt -o" (shell options, not test operators)
        if line.contains("set -o") || line.contains("shopt -o") {
            continue;
        }

        // Look for -o operator in test commands
        for mat in TEST_WITH_OR.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2056",
                Severity::Warning,
                "You probably wanted || here, not -o (which is obsolete). Use [ cond ] || [ cond ]"
                    .to_string(),
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
    fn test_sc2056_or_operator_in_test() {
        let code = r#"[ $a -eq 1 -o $b -eq 2 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2056");
    }

    #[test]
    fn test_sc2056_file_tests_with_or() {
        let code = r#"[ ! -f file -o ! -r file ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2056_test_command() {
        let code = r#"test $x -lt 0 -o $x -gt 10"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2056_chained_tests_ok() {
        let code = r#"[ $a -eq 1 ] || [ $b -eq 2 ]"#;
        let result = check(code);
        // Chained with ||, correct
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2056_double_bracket_ok() {
        let code = r#"[[ $a -eq 1 || $b -eq 2 ]]"#;
        let result = check(code);
        // [[ ]] uses native ||, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2056_set_option_ok() {
        let code = r#"set -o errexit"#;
        let result = check(code);
        // set -o is shell option, not test operator, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2056_shopt_ok() {
        let code = r#"shopt -o noclobber"#;
        let result = check(code);
        // shopt -o is shell option, OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2056_comment_ok() {
        let code = r#"# [ $a -eq 1 -o $b -eq 2 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2056_multiple_conditions() {
        let code = r#"[ $a -eq 1 -o $b -eq 2 -o $c -eq 3 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2056_if_statement() {
        let code = r#"if [ $status -ne 0 -o $ready = "no" ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
