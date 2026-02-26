//! REL003: `read` without `-t` timeout
//!
//! **Rule**: Detect `read` commands without `-t` timeout in scripts
//!
//! **Why this matters**:
//! A `read` without timeout will block indefinitely if stdin is closed or
//! no input arrives. In automated scripts (cron, CI, systemd), this causes
//! the script to hang forever. Adding `-t` ensures the script continues.
//!
//! **Auto-fix**: None (timeout value depends on context)
//!
//! ## Examples
//!
//! Bad (blocks forever if no input):
//! ```bash
//! read -p "Enter value: " val
//! ```
//!
//! Good (times out after 30 seconds):
//! ```bash
//! read -t 30 -p "Enter value: " val
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Check for read without -t timeout
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Match: read at start of command, possibly with flags, but not -t
    let read_pattern = Regex::new(r"\bread\b").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Skip lines inside here-docs (very simplified check)
        if trimmed.starts_with("<<") {
            continue;
        }

        if let Some(m) = read_pattern.find(line) {
            let after = &line[m.end()..];

            // Must be followed by space, end of line, or flags
            if !after.is_empty() && !after.starts_with(' ') && !after.starts_with('\t') {
                continue;
            }

            // Check if -t is already present
            if after.contains(" -t ") || after.contains(" -t") || after.starts_with(" -t") {
                continue;
            }

            // Check if it's a while read loop (reading from pipe is OK)
            if trimmed.starts_with("while")
                || trimmed.contains("| read")
                || trimmed.contains("|read")
            {
                continue;
            }

            let start_col = m.start() + 1;
            let end_col = m.end() + 1;

            let diagnostic = Diagnostic::new(
                "REL003",
                Severity::Info,
                "`read` without `-t` timeout may block indefinitely. Consider adding `-t <seconds>`.",
                Span::new(line_num + 1, start_col, line_num + 1, end_col),
            );

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rel003_detects_read_without_timeout() {
        let script = r#"read -p "Enter value: " val"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "REL003");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
    }

    #[test]
    fn test_rel003_no_flag_with_timeout() {
        let script = r#"read -t 30 -p "Enter value: " val"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel003_no_false_positive_comment() {
        let script = "# read val";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel003_no_flag_while_read_loop() {
        let script = "while read line; do echo $line; done < file.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel003_no_fix_provided() {
        let script = "read val";
        let result = check(script);
        assert!(result.diagnostics[0].fix.is_none());
    }

    #[test]
    fn test_rel003_no_flag_pipe_read() {
        let script = "echo hello | read val";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel003_detects_bare_read() {
        let script = "read";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
