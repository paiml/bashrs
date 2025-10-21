// SC2113: 'function' keyword is non-standard. Delete it and use () instead
//
// When using the 'function' keyword, don't also add () - it's redundant and non-standard.
// Either use 'function name { }' (ksh/bash) or 'name() { }' (POSIX), but not both.
//
// Examples:
// Bad:
//   function foo() { echo "bar"; }    // Mixing styles
//   function deploy() { ... }         // Redundant ()
//
// Good:
//   foo() { echo "bar"; }             // POSIX style (recommended)
//   function foo { echo "bar"; }      // ksh/bash style (acceptable)
//
// Impact: Portability - mixed syntax is confusing

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static FUNCTION_WITH_PARENS: Lazy<Regex> = Lazy::new(|| {
    // Match: function name() { or function name( ) {
    Regex::new(r"\bfunction\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(\s*\)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in FUNCTION_WITH_PARENS.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2113",
                Severity::Warning,
                "'function' keyword is non-standard. Delete it and use () instead".to_string(),
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
    fn test_sc2113_function_with_parens() {
        let code = "function foo() { echo \"bar\"; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2113_function_only_ok() {
        let code = "function foo { echo \"bar\"; }";
        let result = check(code);
        // function without () is OK (though SC2112 will suggest removal)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2113_parens_only_ok() {
        let code = "foo() { echo \"bar\"; }";
        let result = check(code);
        // POSIX style is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2113_comment_ok() {
        let code = "# function foo() { echo \"bar\"; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2113_multiline() {
        let code = r#"
function deploy() {
    echo "Deploying"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2113_multiple() {
        let code = r#"
function foo() { echo "foo"; }
function bar() { echo "bar"; }
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2113_spaces() {
        let code = "function   test   (  )   {";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2113_underscore() {
        let code = "function _helper() { echo \"help\"; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2113_complex() {
        let code = r#"
function process_data() {
    local input=$1
    echo "$input"
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2113_shebang_aware() {
        let code = r#"#!/bin/bash
function main() { echo "main"; }
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
