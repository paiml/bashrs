//! SC1129: Missing space before `!` in negation
//!
//! Detects `if!` or `while!` where `!` immediately follows a keyword
//! without a space. The `!` is a separate token for negation and requires
//! a space after the keyword.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! if! command; then
//! while! test -f file; do
//! ```
//!
//! Good:
//! ```bash
//! if ! command; then
//! while ! test -f file; do
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

const KEYWORDS: &[&str] = &["if", "while", "until", "elif"];

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        for kw in KEYWORDS {
            let pattern = format!("{}!", kw);
            let mut search_start = 0;

            while let Some(pos) = trimmed[search_start..].find(&pattern) {
                let abs_pos = search_start + pos;

                // Ensure it's a word boundary before the keyword
                let is_word_start = abs_pos == 0
                    || (!trimmed.as_bytes()[abs_pos - 1].is_ascii_alphanumeric()
                        && trimmed.as_bytes()[abs_pos - 1] != b'_');

                // Ensure `!` is not followed by `=` (e.g., `if!=` is not this pattern)
                let after_bang = abs_pos + pattern.len();
                let not_bang_eq =
                    after_bang >= trimmed.len() || trimmed.as_bytes()[after_bang] != b'=';

                if is_word_start && not_bang_eq {
                    let line_offset = line.find(trimmed).unwrap_or(0);
                    let col = line_offset + abs_pos + kw.len() + 1;
                    result.add(Diagnostic::new(
                        "SC1129",
                        Severity::Error,
                        format!("Missing space between '{}' and '!'", kw),
                        Span::new(line_num, col, line_num, col + 1),
                    ));
                }

                search_start = abs_pos + pattern.len();
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1129_if_bang_no_space() {
        let result = check("if! command; then");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1129");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1129_while_bang_no_space() {
        let result = check("while! test -f file; do");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1129_until_bang_no_space() {
        let result = check("until! false; do");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1129_proper_space_ok() {
        let result = check("if ! command; then");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1129_while_proper_space_ok() {
        let result = check("while ! test -f file; do");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1129_comment_not_flagged() {
        let result = check("# if! command; then");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1129_not_keyword_prefix() {
        // "elif" as part of a word should not trigger
        let result = check("motif! something");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1129_elif_bang() {
        let result = check("elif! test; then");
        assert_eq!(result.diagnostics.len(), 1);
    }
}
