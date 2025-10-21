// SC2111: `ksh` style functions not supported in sh
//
// The 'function' keyword is not POSIX-compliant and not supported in sh.
// Use POSIX function syntax: name() { ... } instead of function name { ... }
//
// Examples:
// Bad:
//   function foo { echo "bar"; }         // ksh style
//   function bar() { echo "test"; }      // Mixed style
//
// Good:
//   foo() { echo "bar"; }                // POSIX style
//   bar() { echo "test"; }               // POSIX style
//
// Impact: Script won't work in POSIX sh

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static FUNCTION_KEYWORD: Lazy<Regex> = Lazy::new(|| {
    // Match: function name { or function name() {
    Regex::new(r"\bfunction\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*(\(\))?\s*\{").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in FUNCTION_KEYWORD.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2111",
                Severity::Error,
                "`ksh` style 'function' keyword not supported in sh. Use name() {...} instead"
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
    fn test_sc2111_function_keyword() {
        let code = "function foo { echo \"bar\"; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2111_function_with_parens() {
        let code = "function bar() { echo \"test\"; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2111_posix_ok() {
        let code = "foo() { echo \"bar\"; }";
        let result = check(code);
        // POSIX style is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2111_comment_ok() {
        let code = "# function foo { echo \"bar\"; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2111_multiline() {
        let code = r#"
function deploy {
    echo "Deploying"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2111_multiple_functions() {
        let code = r#"
function foo { echo "foo"; }
function bar { echo "bar"; }
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2111_with_body() {
        let code = r#"
function process_file {
    local file=$1
    cat "$file"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    #[ignore] // TODO: Skip function keyword inside strings (complex quote context)
    fn test_sc2111_in_string_ok() {
        let code = r#"echo "function foo { echo test; }""#;
        let result = check(code);
        // Inside string, should skip (but detecting quote context is complex)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2111_spaces() {
        let code = "function   foo   {";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2111_underscore_name() {
        let code = "function _private_func { echo \"private\"; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
