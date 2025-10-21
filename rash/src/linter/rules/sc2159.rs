// SC2159: [ is a command; use [[ ]] or ( ) for grouping.
//
// [ is a command, not grouping syntax. For logical grouping, use [[ ]] or ( ).
//
// Examples:
// Bad:
//   [ "$a" = x ] && [ "$b" = y ]   // OK but verbose
//   [ [ "$a" = x ] ]                // Wrong - [ is not grouping
//
// Good:
//   [[ "$a" = x && "$b" = y ]]      // Proper [[ ]] grouping
//   [ "$a" = x ] && [ "$b" = y ]    // OK but separate tests
//
// Impact: Syntax confusion, potential errors

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static NESTED_SINGLE_BRACKET: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[\s*\[").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in NESTED_SINGLE_BRACKET.find_iter(line) {
            let matched = mat.as_str();

            // Skip if this is actually [[ (double bracket syntax - that's OK!)
            if matched == "[[" {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2159",
                Severity::Error,
                "[ is a command, not grouping syntax. Use [[ ]] for grouping".to_string(),
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
    fn test_sc2159_nested_brackets() {
        let code = r#"[ [ "$a" = x ] ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2159_double_bracket_ok() {
        let code = r#"[[ "$a" = x && "$b" = y ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2159_separate_tests_ok() {
        let code = r#"[ "$a" = x ] && [ "$b" = y ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2159_single_test_ok() {
        let code = r#"[ "$a" = x ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2159_comment_ok() {
        let code = r#"# [ [ "$a" = x ] ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2159_if_nested() {
        let code = r#"if [ [ -f file ] ]; then"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2159_while_ok() {
        let code = r#"while [ -f file ]; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2159_test_command_ok() {
        let code = r#"test "$a" = x"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2159_multiple() {
        let code = "[ [ -f a ] ]\n[ [ -d b ] ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2159_array_subscript_ok() {
        let code = r#"echo "${arr[0]}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
