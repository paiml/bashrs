//! SC1009: Comment detected where command was expected
//!
//! Detects cases where a comment appears immediately after a control
//! structure keyword (`then`, `do`, `else`, `{`) with no command between.
//! The comment is the only thing where a command body is expected.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! if true; then
//!     # TODO: implement
//! fi
//! ```
//!
//! Good:
//! ```bash
//! if true; then
//!     : # TODO: implement
//! fi
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Keywords after which a command is expected on the next line.
const CONTROL_KEYWORDS: &[&str] = &["then", "do", "else", "{"];

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Check if line ends with (or is) a control keyword
        let ends_with_keyword = CONTROL_KEYWORDS
            .iter()
            .any(|kw| trimmed == *kw || trimmed.ends_with(kw));

        if !ends_with_keyword {
            continue;
        }

        // Look at the next non-empty line
        let mut next_idx = i + 1;
        while next_idx < lines.len() && lines[next_idx].trim().is_empty() {
            next_idx += 1;
        }

        if next_idx < lines.len() {
            let next_trimmed = lines[next_idx].trim();
            if next_trimmed.starts_with('#') {
                let line_num = next_idx + 1;
                let col = lines[next_idx].find('#').unwrap_or(0) + 1;
                result.add(Diagnostic::new(
                    "SC1009",
                    Severity::Warning,
                    "Comment here is not a command. Use a no-op `:` if the body is empty",
                    Span::new(line_num, col, line_num, col + next_trimmed.len()),
                ));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1009_comment_after_then() {
        let script = "if true; then\n    # todo\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1009");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc1009_comment_after_do() {
        let script = "for i in 1 2 3; do\n    # process\ndone";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1009_comment_after_else() {
        let script = "if true; then\n    echo ok\nelse\n    # fallback\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1009_command_after_then_ok() {
        let script = "if true; then\n    echo hello\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1009_colon_after_then_ok() {
        let script = "if true; then\n    : # placeholder\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1009_comment_not_after_keyword() {
        let script = "echo hello\n# just a comment\necho world";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1009_blank_lines_between() {
        let script = "if true; then\n\n    # todo\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
