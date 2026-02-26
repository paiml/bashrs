//! REL004: TOCTOU lock file race condition
//!
//! **Rule**: Detect `if [ ! -f lockfile ]; then touch lockfile` pattern
//!
//! **Why this matters**:
//! The pattern `if [ ! -f file ]; then touch file; fi` has a Time-of-Check
//! to Time-of-Use (TOCTOU) race condition. Between the check and the touch,
//! another process may create the file. Use atomic operations like `mkdir`
//! or `ln` for lock files instead.
//!
//! **Auto-fix**: None (requires architectural change)
//!
//! ## Examples
//!
//! Bad (TOCTOU race):
//! ```bash
//! if [ ! -f /tmp/lock ]; then
//!     touch /tmp/lock
//! fi
//! ```
//!
//! Good (atomic lock):
//! ```bash
//! mkdir /tmp/lock.d 2>/dev/null || { echo "locked"; exit 1; }
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for TOCTOU lock file patterns
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let lines: Vec<&str> = source.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Pattern 1: if [ ! -f FILE ]; then ... touch FILE
        // Look for the check pattern
        let is_file_check = (trimmed.contains("[ ! -f ") || trimmed.contains("[ ! -e "))
            && (trimmed.contains("; then") || trimmed.ends_with("then"));

        if is_file_check {
            // Look in the next few lines for a touch/creation of the same file
            let check_end = line_num + 5; // Look ahead up to 5 lines
            let max_line = check_end.min(lines.len());

            for (next_line_num, next_line) in
                lines.iter().enumerate().take(max_line).skip(line_num + 1)
            {
                let next_trimmed = next_line.trim();

                if next_trimmed.contains("touch ") || next_trimmed.contains("> ") {
                    let span =
                        Span::new(line_num + 1, 1, next_line_num + 1, next_trimmed.len() + 1);

                    let diagnostic = Diagnostic::new(
                        "REL004",
                        Severity::Warning,
                        "TOCTOU race condition: check-then-create pattern for lock file. Use `mkdir` for atomic locking.",
                        span,
                    );

                    result.add(diagnostic);
                    break;
                }

                // Stop at fi/else/done
                if next_trimmed == "fi" || next_trimmed == "else" || next_trimmed == "done" {
                    break;
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
    fn test_rel004_detects_toctou_pattern() {
        let script = "if [ ! -f /tmp/lock ]; then\n    touch /tmp/lock\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "REL004");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_rel004_no_flag_without_touch() {
        let script = "if [ ! -f /tmp/lock ]; then\n    echo locked\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel004_no_false_positive_comment() {
        let script = "# if [ ! -f /tmp/lock ]; then";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel004_no_fix_provided() {
        let script = "if [ ! -f /tmp/lock ]; then\n    touch /tmp/lock\nfi";
        let result = check(script);
        assert!(result.diagnostics[0].fix.is_none());
    }

    #[test]
    fn test_rel004_detects_with_e_flag() {
        let script = "if [ ! -e /tmp/lock ]; then\n    touch /tmp/lock\nfi";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_rel004_no_flag_mkdir_pattern() {
        // mkdir is atomic, so not a TOCTOU
        let script = "mkdir /tmp/lock.d 2>/dev/null || exit 1";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
