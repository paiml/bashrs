// SC1028: Parentheses in `[ ]` need escaping
//
// In single-bracket test expressions, parentheses must be escaped with `\`
// or they will be interpreted as subshell syntax.
//
// Examples:
// Bad:
//   [ (expr) ]
//   [ ( -f file ) ]
//
// Good:
//   [ \( expr \) ]
//   [ \( -f file \) ]
//   [[ (expr) ]]   # double brackets handle parens natively

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

/// Matches `[ ` opening a single-bracket test that contains an unescaped `(`
static SINGLE_BRACKET_WITH_PAREN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[\s+.*\(").unwrap()
});

/// Matches `[[ ` double-bracket open (to exclude)
static DOUBLE_BRACKET: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[\[").unwrap()
});

/// Matches escaped paren `\(`
static ESCAPED_PAREN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\\\(").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();

        if trimmed.starts_with('#') {
            continue;
        }

        // Skip lines with [[ ]] (double bracket handles parens fine)
        if DOUBLE_BRACKET.is_match(line) {
            continue;
        }

        for mat in SINGLE_BRACKET_WITH_PAREN.find_iter(line) {
            let matched = mat.as_str();
            // Check if the `(` in this match is escaped
            if ESCAPED_PAREN.is_match(matched) {
                continue;
            }

            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC1028",
                Severity::Error,
                "Parentheses inside `[ ]` need escaping: use `\\(` and `\\)`".to_string(),
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
    fn test_sc1028_unescaped_paren() {
        let code = "[ (expr) ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1028");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1028_unescaped_paren_with_file_test() {
        let code = "[ ( -f file ) ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1028_escaped_paren_ok() {
        let code = r"[ \( -f file \) ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1028_double_bracket_ok() {
        let code = "[[ ( -f file ) ]]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1028_comment_ok() {
        let code = "# [ (expr) ]";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
