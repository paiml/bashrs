// SC1065: Don't declare function parameters in shell
//
// Shell functions don't support declaring parameters in parentheses like
// other languages. Parameters are accessed via $1, $2, etc.
//
// Examples:
// Bad:
//   function greet(name) { echo "Hello $name"; }
//   add(a, b) { echo $((a + b)); }
//   myfunc(x) { echo "$x"; }
//
// Good:
//   greet() { echo "Hello $1"; }
//   add() { echo $(($1 + $2)); }
//   function myfunc { echo "$1"; }

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

/// Matches function declarations with non-empty parentheses content.
/// Handles both `function name(args)` and `name(args)` styles.
/// Ignores empty parens `()`.
static FUNC_WITH_PARAMS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:function\s+)?\w+\s*\(([^)]+)\)\s*\{").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        for caps in FUNC_WITH_PARAMS.captures_iter(line) {
            let params = caps.get(1).unwrap().as_str().trim();
            // Only flag if the content looks like parameter names (not empty/whitespace)
            if params.is_empty() {
                continue;
            }
            // Check that it contains word characters (parameter names), not just operators
            if params.chars().any(|c| c.is_alphabetic()) {
                let full = caps.get(0).unwrap();
                let start_col = full.start() + 1;
                let end_col = full.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC1065",
                    Severity::Error,
                    "Don't declare function parameters in shell. Use $1, $2, etc.".to_string(),
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
    fn test_sc1065_function_with_params() {
        let code = "greet(name) {";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1065");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1065_function_keyword_with_params() {
        let code = "function add(a, b) {";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1065_single_param() {
        let code = "myfunc(x) {";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1065_empty_parens_ok() {
        let code = "greet() {";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1065_function_keyword_no_parens_ok() {
        let code = "function greet {";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1065_comment_ok() {
        let code = "# greet(name) {";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
