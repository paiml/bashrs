//! SC1101: Trailing spaces after `\` line continuation
//!
//! Detects lines that end with `\` followed by trailing whitespace.
//! The continuation won't work because the backslash escapes the space
//! or tab, not the newline. This is a silent and dangerous bug.
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! echo hello \
//!     world
//! ```
//! (Note: trailing spaces after `\` above)
//!
//! Good:
//! ```bash
//! echo hello \
//!     world
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        // Skip comment lines
        if line.trim().starts_with('#') {
            continue;
        }

        // Check if line has trailing whitespace after a backslash
        // Pattern: `\` followed by one or more spaces/tabs at end of line
        let trimmed_end = line.trim_end();
        if trimmed_end.ends_with('\\') && line.len() > trimmed_end.len() {
            // There is whitespace after the final `\`
            let backslash_pos = trimmed_end.len(); // 1-indexed will be +1
            let col = backslash_pos; // position of the backslash (0-indexed)
            let end_col = line.len();

            result.add(Diagnostic::new(
                "SC1101",
                Severity::Error,
                "Trailing spaces after \\ will break line continuation",
                Span::new(line_num, col + 1, line_num, end_col + 1),
            ));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc1101_trailing_space_after_backslash() {
        let script = "echo hello \\   \n    world";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC1101");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc1101_trailing_tab_after_backslash() {
        let script = "echo hello \\\t\n    world";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc1101_proper_continuation_ok() {
        let script = "echo hello \\\n    world";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1101_no_backslash_ok() {
        let script = "echo hello   ";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1101_backslash_in_middle_ok() {
        let script = "echo hello\\ world";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1101_comment_not_flagged() {
        let script = "# echo hello \\   ";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc1101_multiple_lines() {
        let script = "cmd1 \\  \ncmd2 \\  \ncmd3";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
