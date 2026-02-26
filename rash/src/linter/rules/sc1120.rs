//! SC1120: No comments after heredoc token
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! cat <<EOF # this is a comment
//! hello
//! EOF
//! ```
//!
//! Good:
//! ```bash
//! # This is a comment
//! cat <<EOF
//! hello
//! EOF
//! ```
//!
//! # Rationale
//!
//! Comments after the heredoc token are not treated as comments by the shell.
//! Instead, everything after `<<` up to the end of the line (including `#`)
//! becomes part of the delimiter. This means the closing delimiter would need
//! to include the `#` and comment text, which is almost certainly not intended.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Regex to match heredoc token followed by what looks like a comment.
/// Matches: <<EOF # ...  or  <<-EOF # ...  or  <<'EOF' # ...  or  <<"EOF" # ...
#[allow(clippy::expect_used)]
static HEREDOC_WITH_COMMENT: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r#"<<-?\s*\\?(?:'(\w+)'|"(\w+)"|(\w+))\s+#"#).expect("valid heredoc comment regex")
});

/// Check for comments after heredoc tokens
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        if let Some(mat) = HEREDOC_WITH_COMMENT.find(line) {
            let col = mat.start() + 1;
            let span = Span::new(line_num, col, line_num, col + mat.len());
            let diag = Diagnostic::new(
                "SC1120",
                Severity::Error,
                "No comments after the heredoc token: the # becomes part of the delimiter",
                span,
            );
            result.add(diag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1120_detects_comment_after_heredoc() {
        let script = "cat <<EOF # this is a comment\nhello\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1120");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1120_no_false_positive_normal_heredoc() {
        let script = "cat <<EOF\nhello\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1120_no_false_positive_comment_line() {
        let script = "# cat <<EOF # comment";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1120_detects_with_dash_heredoc() {
        let script = "cat <<-EOF # strip tabs\nhello\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1120_no_false_positive_hash_in_body() {
        // Hash inside heredoc body is fine
        let script = "cat <<EOF\n# this is content\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1120_detects_with_quoted_delimiter() {
        let script = "cat <<'EOF' # comment\nhello\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
