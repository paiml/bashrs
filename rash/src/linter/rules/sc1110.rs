//! SC1110: Unicode (smart/curly) double quotes detected
//!
//! Detects Unicode left/right double quotation marks (\u{201c}, \u{201d})
//! which are not valid shell syntax. Use ASCII double quotes instead.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for Unicode smart/curly double quotes
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        // Check for left double quotation mark \u{201c} (bytes: e2 80 9c)
        // and right double quotation mark \u{201d} (bytes: e2 80 9d)
        for (byte_idx, ch) in line.char_indices() {
            if ch == '\u{201c}' || ch == '\u{201d}' {
                let col = byte_idx + 1;
                let description = if ch == '\u{201c}' {
                    "Unicode left double quotation mark detected. Use ASCII double quote (\") instead"
                } else {
                    "Unicode right double quotation mark detected. Use ASCII double quote (\") instead"
                };
                let diagnostic = Diagnostic::new(
                    "SC1110",
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
    fn test_sc1110_left_curly_double_quote() {
        let script = "echo \u{201c}hello\u{201d}";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
        assert_eq!(result.diagnostics[0].code, "SC1110");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1110_no_smart_quotes() {
        let script = r#"echo "hello world""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1110_mixed_with_normal() {
        let script = "echo \u{201c}hello\"";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("left"));
    }

    #[test]
    fn test_sc1110_right_only() {
        let script = "echo hello\u{201d}";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("right"));
    }
}
