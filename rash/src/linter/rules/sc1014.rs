// SC1014: Use `if cmd; then` not `if (cmd)` to check exit code
//
// Using parentheses around a command in an if statement creates a subshell,
// which is rarely the intended behavior. Use `if cmd; then` instead.
//
// Examples:
// Bad:
//   if (grep -q foo file); then echo found; fi
//   if(true); then echo yes; fi
//
// Good:
//   if grep -q foo file; then echo found; fi
//   if (( x > 0 )); then echo positive; fi   # arithmetic is fine

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Matches `if (` or `if(` - we'll manually exclude `if ((`
static IF_PAREN: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\bif\s*\(").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        for mat in IF_PAREN.find_iter(line) {
            let end = mat.end();
            // Skip if next char is also `(` → `if ((...))` arithmetic
            if end < line.len() && line.as_bytes()[end] == b'(' {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = end + 1;

            let diagnostic = Diagnostic::new(
                "SC1014",
                Severity::Warning,
                "Use `if cmd; then` instead of `if (cmd)` to check exit code".to_string(),
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
    fn test_sc1014_if_paren_cmd() {
        let code = "if (grep -q foo file); then echo found; fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1014");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc1014_if_no_space_paren() {
        let code = "if(true); then echo yes; fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1014_arithmetic_ok() {
        let code = "if (( x > 0 )); then echo positive; fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1014_normal_if_ok() {
        let code = "if grep -q foo file; then echo found; fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1014_comment_ok() {
        let code = "# if (cmd); then echo found; fi";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
