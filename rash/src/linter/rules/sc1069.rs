//! SC1069: Missing space before `[` in test
//!
//! Detects `if[`, `while[`, `until[` where a keyword is immediately followed
//! by `[` without an intervening space. The `[` is a separate command and
//! requires a space.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! if[ -f file.txt ]; then
//! while[ "$x" -gt 0 ]; do
//! ```
//!
//! Good:
//! ```bash
//! if [ -f file.txt ]; then
//! while [ "$x" -gt 0 ]; do
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
            let pattern = format!("{}[", kw);
            let mut search_start = 0;

            while let Some(pos) = trimmed[search_start..].find(&pattern) {
                let abs_pos = search_start + pos;

                // Ensure it's a word boundary before the keyword
                let is_word_start = abs_pos == 0
                    || (!trimmed.as_bytes()[abs_pos - 1].is_ascii_alphanumeric()
                        && trimmed.as_bytes()[abs_pos - 1] != b'_');

                if is_word_start {
                    let line_offset = line.find(trimmed).unwrap_or(0);
                    let col = line_offset + abs_pos + kw.len() + 1;
                    result.add(Diagnostic::new(
                        "SC1069",
                        Severity::Error,
                        format!("Missing space between '{}' and '['", kw),
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
    fn test_sc1069_if_no_space() {
        let result = check("if[ -f file.txt ]; then");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1069");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1069_while_no_space() {
        let result = check("while[ true ]; do");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1069_until_no_space() {
        let result = check("until[ false ]; do");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1069_elif_no_space() {
        let result = check("elif[ -d /tmp ]; then");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1069_proper_space_ok() {
        let result = check("if [ -f file.txt ]; then");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1069_while_proper_space_ok() {
        let result = check("while [ true ]; do");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1069_comment_not_flagged() {
        let result = check("# if[ -f file.txt ]; then");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1069_not_keyword_prefix() {
        // "elif" as part of a larger word should not match
        let result = check("reif[0]=1");
        assert_eq!(result.diagnostics.len(), 0);
    }
}
