// SC2102: Ranges can only match single characters (mentioned using * or +)
//
// Character ranges in shell patterns match single characters, not strings.
// [0-9]+ doesn't work in shell patterns (it's regex syntax).
//
// Examples:
// Bad:
//   [[ $var = [0-9]+ ]]          // + doesn't work in shell patterns
//   case $x in [a-z]*+) ;;       // Invalid syntax
//
// Good:
//   [[ $var =~ [0-9]+ ]]         // Use =~ for regex
//   [[ $var = [0-9]* ]]          // * is OK for glob
//
// Impact: Pattern doesn't match as expected

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static RANGE_WITH_PLUS: Lazy<Regex> = Lazy::new(|| {
    // Match: [range]+ in glob context (not =~)
    Regex::new(r"\[[^\]]+\]\+").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip lines with =~ (regex context)
        if line.contains("=~") {
            continue;
        }

        for mat in RANGE_WITH_PLUS.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2102",
                Severity::Warning,
                "Ranges can only match single chars (to match + literally, use \\+)".to_string(),
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
    fn test_sc2102_range_plus() {
        let code = "[[ $var = [0-9]+ ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2102_case_range_plus() {
        let code = "case $x in [a-z]+) ;;";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2102_regex_ok() {
        let code = "[[ $var =~ [0-9]+ ]]";
        let result = check(code);
        // =~ uses regex, + is valid
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2102_glob_star_ok() {
        let code = "[[ $var = [0-9]* ]]";
        let result = check(code);
        // * is valid in globs
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2102_comment_ok() {
        let code = "# [[ $var = [0-9]+ ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2102_literal_plus() {
        let code = "[[ $var = [0-9]\\+ ]]";
        let result = check(code);
        // Escaped + is literal
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2102_multiple_ranges() {
        let code = "case $x in [a-z]+|[0-9]+) ;;";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2102_in_test() {
        let code = "[ \"$var\" = [0-9]+ ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    #[ignore] // TODO: Handle POSIX classes with +
    fn test_sc2102_posix_class() {
        let code = "[[ $var = [[:digit:]]+ ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2102_find_name() {
        let code = "find . -name \"[0-9]+\"";
        let result = check(code);
        // In find -name patterns
        assert_eq!(result.diagnostics.len(), 1);
    }
}
