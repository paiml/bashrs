//! SC1020: Missing space before closing `]`
//!
//! Detects `[ expr]` without a space before the closing `]` in test
//! commands. The `]` must be a separate argument to `[`.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! [ -f file.txt]
//! [ "$x" = "y"]
//! ```
//!
//! Good:
//! ```bash
//! [ -f file.txt ]
//! [ "$x" = "y" ]
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Find test command patterns: [ ... ]
        // We look for `[ ` to start and then check if `]` is preceded by non-space
        let bytes = line.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            // Find opening `[ ` (single bracket test, not `[[`)
            if bytes[i] == b'['
                && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric())
                && (i + 1 >= bytes.len() || bytes[i + 1] != b'[')
            {
                // Check it's not an array subscript: $arr[idx]
                if i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'}') {
                    i += 1;
                    continue;
                }

                // Find the matching ]
                if let Some(close_pos) = line[i + 1..].find(']') {
                    let abs_close = i + 1 + close_pos;
                    // Check if character before ] is not a space
                    if abs_close > 0 && bytes[abs_close - 1] != b' ' && bytes[abs_close - 1] != b'\t' {
                        // Make sure there's actual content between [ and ] (not empty)
                        let inner = line[i + 1..abs_close].trim();
                        if !inner.is_empty() {
                            let col = abs_close + 1;
                            result.add(Diagnostic::new(
                                "SC1020",
                                Severity::Error,
                                "Missing space before closing ] in test expression",
                                Span::new(line_num, col, line_num, col + 1),
                            ));
                        }
                    }
                    i = abs_close + 1;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1020_missing_space_before_close() {
        let result = check("[ -f file.txt]");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1020");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1020_proper_spacing_ok() {
        let result = check("[ -f file.txt ]");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1020_string_comparison() {
        let result = check(r#"[ "$x" = "y"]"#);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1020_array_subscript_not_flagged() {
        let result = check("echo ${arr[0]}");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1020_double_bracket_not_matched() {
        // Double brackets have different parsing rules
        let result = check("[[ -f file.txt ]]");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1020_comment_not_flagged() {
        let result = check("# [ -f file.txt]");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1020_if_context() {
        let result = check("if [ -f file.txt]; then");
        assert_eq!(result.diagnostics.len(), 1);
    }
}
