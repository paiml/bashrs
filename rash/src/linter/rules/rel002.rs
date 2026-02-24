//! REL002: mktemp without trap cleanup
//!
//! **Rule**: Detect `mktemp` usage without a corresponding `trap ... EXIT` cleanup
//!
//! **Why this matters**:
//! Temporary files created with `mktemp` will leak if the script exits
//! unexpectedly (error, signal). A `trap` on EXIT ensures cleanup happens
//! regardless of how the script terminates.
//!
//! **Auto-fix**: None (manual - add trap for cleanup)
//!
//! ## Examples
//!
//! Bad (temp file leaks on error):
//! ```bash
//! tmpfile=$(mktemp)
//! # script may exit before cleanup
//! rm "$tmpfile"
//! ```
//!
//! Good (trap ensures cleanup):
//! ```bash
//! tmpfile=$(mktemp)
//! trap 'rm -f "$tmpfile"' EXIT
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Check for mktemp without trap EXIT cleanup
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let has_mktemp = source.contains("mktemp");
    if !has_mktemp {
        return result;
    }

    // Check if there's a trap on EXIT/ERR
    let has_trap_exit = source.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.starts_with('#')
            && trimmed.contains("trap")
            && (trimmed.contains("EXIT") || trimmed.contains("ERR") || trimmed.contains("0"))
    });

    if has_trap_exit {
        return result;
    }

    // Flag each mktemp usage
    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        if let Some(col) = line.find("mktemp") {
            // Verify it's a standalone word
            let before_ok = col == 0 || !line.as_bytes()[col - 1].is_ascii_alphanumeric();
            let after_idx = col + 6;
            let after_ok = after_idx >= line.len()
                || !line.as_bytes()[after_idx].is_ascii_alphanumeric();

            if before_ok && after_ok {
                let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 7);

                let diagnostic = Diagnostic::new(
                    "REL002",
                    Severity::Warning,
                    "mktemp without `trap ... EXIT` cleanup. Temp files may leak on unexpected exit.",
                    span,
                );

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rel002_detects_mktemp_without_trap() {
        let script = "tmpfile=$(mktemp)\necho hello > $tmpfile\nrm $tmpfile";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "REL002");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_rel002_no_flag_with_trap_exit() {
        let script = "tmpfile=$(mktemp)\ntrap 'rm -f \"$tmpfile\"' EXIT";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel002_no_flag_with_trap_err() {
        let script = "tmpfile=$(mktemp)\ntrap 'rm -f \"$tmpfile\"' ERR";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel002_no_false_positive_comment() {
        let script = "# tmpfile=$(mktemp)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel002_no_fix_provided() {
        let script = "tmpfile=$(mktemp)";
        let result = check(script);
        assert!(result.diagnostics[0].fix.is_none());
    }

    #[test]
    fn test_rel002_no_flag_without_mktemp() {
        let script = "echo hello";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel002_no_flag_with_trap_signal_0() {
        let script = "tmpfile=$(mktemp)\ntrap 'rm -f \"$tmpfile\"' 0";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
