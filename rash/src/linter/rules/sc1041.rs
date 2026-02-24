//! SC1041: Found `EOF` on the same line as `<<`
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! cat <<EOF hello EOF
//! ```
//!
//! Good:
//! ```bash
//! cat <<EOF
//! hello
//! EOF
//! ```
//!
//! # Rationale
//!
//! If the heredoc delimiter appears both after `<<` and again on the same line,
//! it likely means the user expected the heredoc content to be on the same line.
//! Heredocs always start on the *next* line after the `<<DELIM` token.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

/// Regex to match heredoc start and capture the delimiter (no backreferences)
#[allow(clippy::expect_used)]
static HEREDOC_START: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"<<-?\s*\\?(?:'(\w+)'|"(\w+)"|(\w+))"#).expect("valid heredoc start regex")
});

/// Extract the delimiter from captures (whichever alternative matched)
fn extract_delimiter<'a>(caps: &'a regex::Captures<'a>) -> Option<&'a str> {
    caps.get(1)
        .or_else(|| caps.get(2))
        .or_else(|| caps.get(3))
        .map(|m| m.as_str())
}

/// Check for heredoc delimiter appearing on the same line as content
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        if let Some(caps) = HEREDOC_START.captures(line) {
            if let Some(delimiter) = extract_delimiter(&caps) {
                let full_match = caps.get(0).unwrap();
                let after_heredoc = &line[full_match.end()..];

                // Check if the delimiter appears again in the remainder of the line
                if after_heredoc.contains(delimiter) {
                    let col = full_match.start() + 1;
                    let span = Span::new(line_num, col, line_num, col + full_match.len());
                    let diag = Diagnostic::new(
                        "SC1041",
                        Severity::Error,
                        format!(
                            "Found '{delimiter}' further on the same line as the << token; \
                             the heredoc body starts on the next line"
                        ),
                        span,
                    );
                    result.add(diag);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1041_detects_delimiter_on_same_line() {
        let script = "cat <<EOF hello EOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1041");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1041_no_false_positive_normal_heredoc() {
        let script = "cat <<EOF\nhello\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1041_no_false_positive_comment() {
        let script = "# cat <<EOF hello EOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1041_detects_with_dash_heredoc() {
        let script = "cat <<-MARKER content MARKER";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1041_span_points_to_heredoc_token() {
        let script = "cat <<EOF text EOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        assert_eq!(span.start_line, 1);
        assert_eq!(span.start_col, 5); // "cat " is 4 chars, <<EOF starts at 5
    }
}
