//! REL005: Temp file with predictable name
//!
//! **Rule**: Detect hardcoded temp file paths like `/tmp/foo` instead of `mktemp`
//!
//! **Why this matters**:
//! Hardcoded temp file names (e.g., `/tmp/foo`, `/tmp/myapp.log`) are
//! predictable, creating security risks (symlink attacks) and reliability
//! issues (multiple instances overwrite each other). Use `mktemp` for
//! unique, unpredictable temp files.
//!
//! **Auto-fix**: Safe - suggest replacing with `mktemp`
//!
//! ## Examples
//!
//! Bad (predictable, vulnerable to symlink attacks):
//! ```bash
//! echo "data" > /tmp/myapp.log
//! tmpfile=/tmp/output.txt
//! ```
//!
//! Good (unique, secure):
//! ```bash
//! tmpfile=$(mktemp)
//! echo "data" > "$tmpfile"
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for hardcoded temp file paths
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Match hardcoded /tmp/ paths in assignments or redirections
    // But not /tmp itself or /tmp/ alone
    let pattern = Regex::new(r"/tmp/[a-zA-Z_][a-zA-Z0-9_.\-]*").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Skip lines that already use mktemp
        if line.contains("mktemp") {
            continue;
        }

        // Skip trap cleanup lines (they reference temp files legitimately)
        if trimmed.starts_with("trap ") {
            continue;
        }

        for m in pattern.find_iter(line) {
            let path = m.as_str();

            // Skip /tmp/. and /tmp/.. path components
            if path == "/tmp/." || path == "/tmp/.." {
                continue;
            }

            let start_col = m.start() + 1;
            let end_col = m.end() + 1;

            let diagnostic = Diagnostic::new(
                "REL005",
                Severity::Warning,
                format!(
                    "Predictable temp file `{}`. Use `mktemp` for unique, secure temp files.",
                    path
                ),
                Span::new(line_num + 1, start_col, line_num + 1, end_col),
            )
            .with_fix(Fix::new("$(mktemp)".to_string()));

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rel005_detects_hardcoded_tmp() {
        let script = "tmpfile=/tmp/output.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "REL005");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_rel005_provides_fix() {
        let script = "tmpfile=/tmp/output.txt";
        let result = check(script);
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "$(mktemp)");
    }

    #[test]
    fn test_rel005_no_flag_with_mktemp() {
        let script = "tmpfile=$(mktemp /tmp/myapp.XXXXXX)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel005_no_false_positive_comment() {
        let script = "# tmpfile=/tmp/output.txt";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel005_detects_redirect_to_tmp() {
        let script = "echo data > /tmp/myapp.log";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_rel005_no_flag_trap_cleanup() {
        let script = "trap 'rm -f /tmp/mylock' EXIT";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_rel005_detects_multiple_tmp_files() {
        let script = "a=/tmp/foo\nb=/tmp/bar";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 2);
    }
}
