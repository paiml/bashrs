// SC2138: Function defined with incorrect syntax or in incorrect context
//
// Functions should be defined at the top level, not inside other constructs.
// Also, function definitions have specific syntax requirements.
//
// Examples:
// Bad:
//   if true; then function foo() { :; } fi    // Function inside if
//   for i in 1; do function bar() { :; } done // Function inside loop
//   function() { echo "test"; }               // 'function' as name
//
// Good:
//   function foo() { :; }                     // Top-level definition
//   foo() { :; }                              // POSIX style
//   my_function() { echo "test"; }            // Valid name
//
// Impact: Portability issues, may not work in all shells

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static FUNCTION_IN_IF: Lazy<Regex> = Lazy::new(|| {
    // Match: if/elif/else ... function name() {
    Regex::new(r"\b(if|elif|else)\b[^;]*;\s*then[^\n]*\bfunction\b").unwrap()
});

static FUNCTION_IN_LOOP: Lazy<Regex> = Lazy::new(|| {
    // Match: for/while/until ... function name() {
    Regex::new(r"\b(for|while|until)\b[^;]*;\s*do[^\n]*\bfunction\b").unwrap()
});

static FUNCTION_AS_NAME: Lazy<Regex> = Lazy::new(|| {
    // Match: function() { ... } - 'function' used as function name
    Regex::new(r"\bfunction\s*\(\s*\)\s*\{").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for function defined inside if/elif/else
        if FUNCTION_IN_IF.is_match(line) {
            if let Some(mat) = FUNCTION_IN_IF.find(line) {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2138",
                    Severity::Warning,
                    "Functions should be defined at top level, not inside if statements"
                        .to_string(),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }

        // Check for function defined inside loops
        if FUNCTION_IN_LOOP.is_match(line) {
            if let Some(mat) = FUNCTION_IN_LOOP.find(line) {
                let start_col = mat.start() + 1;
                let end_col = mat.end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2138",
                    Severity::Warning,
                    "Functions should be defined at top level, not inside loops".to_string(),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
            }
        }

        // Check for 'function' used as function name
        for mat in FUNCTION_AS_NAME.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2138",
                Severity::Error,
                "'function' is a keyword and cannot be used as a function name".to_string(),
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
    fn test_sc2138_function_in_if() {
        let code = "if true; then function foo() { :; } fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("top level"));
    }

    #[test]
    fn test_sc2138_function_in_loop() {
        let code = "for i in 1; do function bar() { :; } done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("loops"));
    }

    #[test]
    fn test_sc2138_function_as_name() {
        let code = "function() { echo \"test\"; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("keyword"));
    }

    #[test]
    fn test_sc2138_top_level_ok() {
        let code = "function foo() { :; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2138_posix_style_ok() {
        let code = "foo() { :; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2138_comment_ok() {
        let code = "# if true; then function foo() { :; } fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2138_while_loop() {
        let code = "while true; do function test() { echo hi; } done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2138_elif() {
        let code = "if false; then :; elif true; then function baz() { :; } fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2138_valid_name() {
        let code = "my_function() { echo \"test\"; }";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2138_multiline() {
        let code = r#"
if true; then function foo() { :; } fi
for i in 1; do function bar() { :; } done
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
