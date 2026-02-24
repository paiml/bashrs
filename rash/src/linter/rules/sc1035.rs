//! SC1035: Missing space after certain keywords
//!
//! Detects missing spaces after shell keywords like `then`, `do`, `else`,
//! `elif`, `fi`, `done`, `while`, `until`, `for`, `case`, `esac` when they
//! are immediately followed by a non-space character.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! if true; then(echo hi)
//! for i in 1 2; do{echo $i;}
//! ```
//!
//! Good:
//! ```bash
//! if true; then echo hi
//! for i in 1 2; do echo $i; done
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

const KEYWORDS: &[&str] = &[
    "then", "do", "else", "elif", "fi", "done", "while", "until", "for", "case", "esac", "in",
];

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        for kw in KEYWORDS {
            // Search for keyword followed immediately by a non-whitespace, non-semicolon char
            let kw_len = kw.len();
            let mut search_start = 0;

            while let Some(pos) = trimmed[search_start..].find(kw) {
                let abs_pos = search_start + pos;
                let after = abs_pos + kw_len;

                // Verify it's a word boundary before the keyword
                let is_word_start = abs_pos == 0
                    || !trimmed.as_bytes()[abs_pos - 1].is_ascii_alphanumeric()
                        && trimmed.as_bytes()[abs_pos - 1] != b'_';

                if is_word_start && after < trimmed.len() {
                    let next_char = trimmed.as_bytes()[after];
                    // Must be followed by something that's not whitespace, semicolon, newline, or #
                    if next_char != b' '
                        && next_char != b'\t'
                        && next_char != b';'
                        && next_char != b'\n'
                        && next_char != b'#'
                        && next_char != b'\r'
                        // Also skip if next char is alphanumeric (it's part of a longer word, e.g. "done_flag")
                        && !(next_char.is_ascii_alphanumeric() || next_char == b'_')
                    {
                        let line_offset = line.find(trimmed).unwrap_or(0);
                        let col = line_offset + abs_pos + 1;
                        result.add(Diagnostic::new(
                            "SC1035",
                            Severity::Error,
                            format!("Missing space after '{}' keyword", kw),
                            Span::new(line_num, col, line_num, col + kw_len),
                        ));
                    }
                }

                search_start = abs_pos + kw_len;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1035_then_no_space() {
        let result = check("if true; then(echo hi)");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1035");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1035_do_no_space() {
        let result = check("for i in 1 2; do{echo $i;}");
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1035_proper_spacing_ok() {
        let result = check("if true; then echo hi; fi");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1035_keyword_as_part_of_word_ok() {
        // "done_flag" should not be flagged (done is part of a larger word)
        let result = check("done_flag=1");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1035_semicolon_after_keyword_ok() {
        let result = check("then;");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1035_comment_not_flagged() {
        let result = check("# then(echo)");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1035_else_no_space() {
        let result = check("else(echo fallback)");
        assert_eq!(result.diagnostics.len(), 1);
    }
}
