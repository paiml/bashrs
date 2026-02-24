//! SC1111: Unicode (smart/curly) single quotes detected
//!
//! Detects Unicode left/right single quotation marks (\u{2018}, \u{2019})
//! which are not valid shell syntax. Use ASCII single quotes instead.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for Unicode smart/curly single quotes
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        // Check for left single quotation mark \u{2018} (bytes: e2 80 98)
        // and right single quotation mark \u{2019} (bytes: e2 80 99)
        for (byte_idx, ch) in line.char_indices() {
            if ch == '\u{2018}' || ch == '\u{2019}' {
                let col = byte_idx + 1;
                let description = if ch == '\u{2018}' {
                    "Unicode left single quotation mark detected. Use ASCII single quote (') instead"
                } else {
                    "Unicode right single quotation mark detected. Use ASCII single quote (') instead"
                };
                let diagnostic = Diagnostic::new(
                    "SC1111",
                    Severity::Error,
                    description,
                    Span::new(line_num, col, line_num, col + ch.len_utf8()),
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
    fn test_sc1111_left_curly_single_quote() {
        let script = "echo \u{2018}hello\u{2019}";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
        assert_eq!(result.diagnostics[0].code, "SC1111");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1111_no_smart_quotes() {
        let script = "echo 'hello world'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1111_mixed_with_normal() {
        let script = "echo \u{2018}hello'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("left"));
    }

    #[test]
    fn test_sc1111_right_only() {
        let script = "echo hello\u{2019}";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("right"));
    }

    #[test]
    fn test_sc1111_apostrophe_not_flagged() {
        // ASCII apostrophe should not be flagged
        let script = "echo 'it is fine'";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
