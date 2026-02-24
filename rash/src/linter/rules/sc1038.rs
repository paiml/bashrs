//! SC1038: Use `< <(cmd)` for process substitution, not `<<(cmd)`
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! while read line; do echo "$line"; done <<(grep pattern file)
//! ```
//!
//! Good:
//! ```bash
//! while read line; do echo "$line"; done < <(grep pattern file)
//! ```
//!
//! # Rationale
//!
//! `<<(cmd)` is not valid syntax. The intended construct is `< <(cmd)`, which
//! redirects stdin from a process substitution. The space between `<` and `<(`
//! is required.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for `<<(` that should be `< <(`
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        // Search for all occurrences of <<( on this line
        let bytes = line.as_bytes();
        let mut i = 0;
        while i + 2 < bytes.len() {
            if bytes[i] == b'<' && bytes[i + 1] == b'<' && bytes[i + 2] == b'(' {
                // Check this is not a heredoc delimiter that happens to start with (
                // Heredocs use <<WORD or <<-WORD, not <<(
                // Also skip if preceded by another < (like <<<)
                let preceded_by_lt = i > 0 && bytes[i - 1] == b'<';
                if !preceded_by_lt {
                    let col = i + 1; // 1-indexed
                    let span = Span::new(line_num, col, line_num, col + 3);
                    let diag = Diagnostic::new(
                        "SC1038",
                        Severity::Error,
                        "Shells are space sensitive: use '< <(cmd)', not '<<(cmd)'",
                        span,
                    );
                    result.add(diag);
                }
                i += 3;
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
    fn test_sc1038_detects_missing_space() {
        let script = "while read line; do echo \"$line\"; done <<(grep pattern file)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1038");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1038_no_false_positive_correct_syntax() {
        let script = "while read line; do echo \"$line\"; done < <(grep pattern file)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1038_no_false_positive_heredoc() {
        // Normal heredoc should not trigger
        let script = "cat <<EOF\nhello\nEOF";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1038_no_false_positive_comment() {
        let script = "# <<(this is a comment)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1038_multiple_on_same_line() {
        let script = "cmd <<(a) <<(b)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc1038_span_location() {
        let script = "done <<(cmd)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        let span = result.diagnostics[0].span;
        assert_eq!(span.start_line, 1);
        assert_eq!(span.start_col, 6); // "done " is 5 chars, << starts at col 6
    }
}
