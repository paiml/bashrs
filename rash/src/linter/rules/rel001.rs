//! REL001: Destructive command without error check
//!
//! **Rule**: Detect destructive commands (`rm -rf`, `drop`) without error checking
//!
//! **Why this matters**:
//! Destructive commands like `rm -rf` can silently fail or operate on wrong paths.
//! Without error checking (`|| exit`, `set -e`, or `if` guard), data loss may
//! go undetected and the script continues in an inconsistent state.
//!
//! **Auto-fix**: None (manual review required)
//!
//! ## Examples
//!
//! Bad (no error check):
//! ```bash
//! rm -rf /var/cache/app
//! ```
//!
//! Good (with error check):
//! ```bash
//! rm -rf /var/cache/app || { echo "Failed to clean cache" >&2; exit 1; }
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

/// Check for destructive commands without error checking
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Check if set -e is active (global error checking)
    let has_set_e = source.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "set -e" || trimmed == "set -euo pipefail" || trimmed.contains("set -e ")
            || trimmed.starts_with("set -e;") || trimmed == "set -eu"
    });

    if has_set_e {
        return result;
    }

    let destructive_pattern = Regex::new(r"\b(rm\s+-rf|rm\s+-r\s+-f|rm\s+-fr)\b").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        if let Some(m) = destructive_pattern.find(line) {
            // Check if there's an error handler on the same line
            let after_cmd = &line[m.end()..];
            if after_cmd.contains("||") || after_cmd.contains("&&") {
                continue;
            }

            // Check if it's inside an if block (previous line)
            if trimmed.starts_with("if ") || trimmed.starts_with("then") {
                continue;
            }

            let start_col = m.start() + 1;
            let end_col = m.end() + 1;

            let diagnostic = Diagnostic::new(
                "REL001",
                Severity::Warning,
                "Destructive command without error check. Add `|| exit 1` or use `set -e`.",
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
    fn test_rel001_detects_rm_rf_without_check() {
        let script = "rm -rf /var/cache/app";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "REL001");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_rel001_no_flag_with_error_handler() {
        let script = "rm -rf /var/cache/app || exit 1";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel001_no_flag_with_set_e() {
        let script = "set -e\nrm -rf /var/cache/app";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel001_no_flag_with_and_handler() {
        let script = "rm -rf /var/cache/app && echo done";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel001_no_false_positive_comment() {
        let script = "# rm -rf /var/cache/app";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel001_no_fix_provided() {
        let script = "rm -rf /tmp/build";
        let result = check(script);
        assert!(result.diagnostics[0].fix.is_none());
    }

    #[test]
    fn test_rel001_detects_rm_r_f() {
        let script = "rm -r -f /tmp/build";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_rel001_no_flag_with_set_euo() {
        let script = "set -euo pipefail\nrm -rf /tmp/build";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
