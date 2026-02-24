// SC1036: `(` is invalid here (expected command)
//
// A standalone `(` in a position where a command is expected is likely a
// syntax error, possibly from trying to use C/Python-style syntax.
//
// Examples:
// Bad:
//   then(
//   ;(echo hello)
//   do(
//
// Good:
//   then echo hello
//   ; echo hello
//   do echo hello

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

/// Matches keywords immediately followed by `(` (no space before paren)
/// e.g. `then(`, `do(`, `else(`
static KEYWORD_PAREN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(then|do|else)\(").unwrap()
});

/// Matches `;\(` - semicolon followed directly by open paren
static SEMI_PAREN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r";\(").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        for mat in KEYWORD_PAREN.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC1036",
                Severity::Error,
                "`(` is invalid here. Add a space or use proper shell syntax.".to_string(),
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        for mat in SEMI_PAREN.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC1036",
                Severity::Error,
                "`(` is invalid here. Remove `;` before `(` or fix syntax.".to_string(),
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
    fn test_sc1036_then_paren() {
        let code = "if true; then(echo hello); fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1036");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1036_do_paren() {
        let code = "for i in 1 2; do(echo $i); done";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1036_semi_paren() {
        let code = "echo hello;(echo world)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1036_normal_then_ok() {
        let code = "if true; then echo hello; fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1036_subshell_ok() {
        let code = "( echo hello )";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1036_comment_ok() {
        let code = "# then(echo hello)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
