//! SC1040: With `<<-`, you can only indent with tabs
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! cat <<-EOF
//!     indented with spaces
//!     EOF
//! ```
//!
//! Good:
//! ```bash
//! cat <<-EOF
//!     indented with tabs
//!     EOF
//! ```
//!
//! # Rationale
//!
//! The `<<-` heredoc operator strips leading *tabs* from the body and closing
//! delimiter. It does not strip spaces. If you indent with spaces, they will
//! appear in the output, and the closing delimiter won't be recognized if
//! it's indented with spaces.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

/// Regex to match `<<-` heredoc start with unquoted, single-quoted, or double-quoted delimiter
#[allow(clippy::expect_used)]
static HEREDOC_STRIP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"<<-\s*\\?(?:'(\w+)'|"(\w+)"|(\w+))"#).expect("valid heredoc strip regex")
});

/// Extract the delimiter from captures (whichever alternative matched)
fn extract_delimiter<'a>(caps: &'a regex::Captures<'a>) -> Option<&'a str> {
    caps.get(1)
        .or_else(|| caps.get(2))
        .or_else(|| caps.get(3))
        .map(|m| m.as_str())
}

/// Check for space indentation in `<<-` heredoc bodies
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            i += 1;
            continue;
        }

        if let Some(caps) = HEREDOC_STRIP.captures(line) {
            if let Some(delimiter) = extract_delimiter(&caps) {
                // Scan the heredoc body
                let mut j = i + 1;
                while j < lines.len() {
                    let body_line = lines[j];
                    // Check if this is the closing delimiter (possibly indented with tabs)
                    if body_line.trim() == delimiter {
                        break;
                    }
                    // Flag lines that start with spaces (not tabs, not empty)
                    if !body_line.is_empty() && body_line.starts_with(' ') {
                        let line_num = j + 1;
                        let span = Span::new(line_num, 1, line_num, 1);
                        let diag = Diagnostic::new(
                            "SC1040",
                            Severity::Warning,
                            "When using <<-, you can only indent with tabs",
                            span,
                        );
                        result.add(diag);
                    }
                    j += 1;
                }
                i = j + 1;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1040_detects_space_indentation() {
        let script = "cat <<-EOF\n    hello world\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1040");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc1040_no_flag_for_tab_indentation() {
        let script = "cat <<-EOF\n\thello world\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1040_no_flag_for_regular_heredoc() {
        // Regular << (without dash) is not affected
        let script = "cat <<EOF\n    hello world\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1040_multiple_space_lines() {
        let script = "cat <<-END\n  line1\n  line2\n  line3\nEND";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 3);
    }

    #[test]
    fn test_sc1040_mixed_tabs_and_spaces() {
        let script = "cat <<-EOF\n\tgood line\n  bad line\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].span.start_line, 3);
    }

    #[test]
    fn test_sc1040_no_false_positive_comment() {
        let script = "# cat <<-EOF\n    not a heredoc\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
