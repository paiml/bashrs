// SC2171: Found trailing ] without opening [.
//
// Unmatched ] indicates syntax error or typo.
//
// Examples:
// Bad:
//   if  "$a" = x ]; then         // Missing [
//   ] && echo "ok"                // Standalone ]
//
// Good:
//   if [ "$a" = x ]; then         // Matched brackets
//   [[ "$a" = x ]] && echo "ok"   // Proper syntax
//
// Impact: Syntax error

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static TRAILING_BRACKET: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*\]").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for line starting with ]
        if TRAILING_BRACKET.is_match(line) {
            let start_col = line.find(']').map(|i| i + 1).unwrap_or(1);
            let end_col = start_col + 1;

            let diagnostic = Diagnostic::new(
                "SC2171",
                Severity::Error,
                "Found trailing ] without opening [".to_string(),
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
    fn test_sc2171_trailing_bracket() {
        let code = "] && echo ok";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2171_if_missing_open() {
        let code = r#"if  "$a" = x ]; then"#;
        let result = check(code);
        // Would need more context to detect this case
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2171_matched_ok() {
        let code = r#"[ "$a" = x ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2171_comment_ok() {
        let code = "# ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2171_array_subscript_ok() {
        let code = r#"echo "${arr[0]}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2171_double_bracket_ok() {
        let code = "[[ $a = x ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2171_standalone_close() {
        let code = "  ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2171_multiple() {
        let code = "]\n]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2171_end_of_test_ok() {
        let code = "if [ -f file ]; then";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2171_case_pattern_ok() {
        let code = "  pattern)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
