//! SC1044: End token not found (unterminated heredoc)
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! cat <<EOF
//! hello world
//! # missing closing EOF
//! ```
//!
//! Good:
//! ```bash
//! cat <<EOF
//! hello world
//! EOF
//! ```
//!
//! # Rationale
//!
//! Every heredoc must be terminated by its delimiter on its own line.
//! If the closing delimiter is never found, the shell will consume the rest
//! of the script as heredoc content, causing confusing errors.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Regex to match heredoc start and capture the delimiter (no backreferences)
#[allow(clippy::expect_used)]
static HEREDOC_START: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r#"<<-?\s*\\?(?:'(\w+)'|"(\w+)"|(\w+))"#).expect("valid heredoc start regex")
});

/// Extract the delimiter from captures (whichever alternative matched)
fn extract_delimiter<'a>(caps: &'a regex::Captures<'a>) -> Option<&'a str> {
    caps.get(1)
        .or_else(|| caps.get(2))
        .or_else(|| caps.get(3))
        .map(|m| m.as_str())
}

/// Check for unterminated heredocs
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

        if let Some(caps) = HEREDOC_START.captures(line) {
            if let Some(delimiter) = extract_delimiter(&caps) {
                let full_match = caps.get(0).unwrap();
                let is_strip = line[full_match.start()..].starts_with("<<-");

                // Search for the closing delimiter
                let mut found = false;
                let mut j = i + 1;
                while j < lines.len() {
                    let candidate = if is_strip {
                        lines[j].trim_start_matches('\t')
                    } else {
                        lines[j]
                    };
                    if candidate.trim() == delimiter {
                        found = true;
                        i = j + 1;
                        break;
                    }
                    j += 1;
                }

                if !found {
                    let line_num = i + 1;
                    let col = full_match.start() + 1;
                    let span = Span::new(line_num, col, line_num, col + full_match.len());
                    let diag = Diagnostic::new(
                        "SC1044",
                        Severity::Error,
                        format!("Couldn't find end token '{delimiter}' for this heredoc"),
                        span,
                    );
                    result.add(diag);
                    i += 1;
                }
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
    fn test_sc1044_detects_unterminated_heredoc() {
        let script = "cat <<EOF\nhello world\n# no closing delimiter";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1044");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("EOF"));
    }

    #[test]
    fn test_sc1044_no_flag_terminated_heredoc() {
        let script = "cat <<EOF\nhello world\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1044_no_flag_strip_heredoc_with_tabs() {
        let script = "cat <<-EOF\n\thello\n\tEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1044_detects_wrong_delimiter() {
        // Opening says EOF but closing says END
        let script = "cat <<EOF\nhello\nEND";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1044_multiple_heredocs_one_unterminated() {
        let script = "cat <<EOF\nhello\nEOF\ncat <<MARKER\nworld";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("MARKER"));
    }

    #[test]
    fn test_sc1044_no_false_positive_comment() {
        let script = "# cat <<EOF\nhello\n# not a heredoc";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
