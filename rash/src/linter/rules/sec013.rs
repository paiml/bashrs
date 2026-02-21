//! SEC013: Insecure Temporary File Creation
//!
//! **Rule**: Detect insecure temporary file creation patterns
//!
//! **Why this matters**:
//! Using predictable temporary file paths (e.g., /tmp/myfile) creates race conditions
//! (TOCTOU) and symlink attacks. An attacker can create a symlink at the predicted path
//! to overwrite arbitrary files.
//!
//! **Auto-fix**: Replace with mktemp
//!
//! ## Examples
//!
//! Bad:
//! ```bash
//! echo data > /tmp/myapp.log
//! TMPFILE=/tmp/output.$$
//! cat foo > /tmp/bar
//! ```
//!
//! Good:
//! ```bash
//! TMPFILE=$(mktemp)
//! echo data > "$TMPFILE"
//! TMPDIR=$(mktemp -d)
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Predictable temp path patterns (prefix matches)
const UNSAFE_TMP_PATTERNS: &[&str] = &["/tmp/", "/var/tmp/"];

/// Check whether a line writes to or assigns a temp path
fn is_tmp_write_or_assignment(trimmed: &str) -> bool {
    let is_write = trimmed.contains('>')
        || trimmed.contains('=')
        || trimmed.starts_with("touch ")
        || trimmed.starts_with("cat ")
        || trimmed.starts_with("cp ")
        || trimmed.starts_with("mv ");

    let is_assignment = trimmed.contains("=/tmp/") || trimmed.contains("=/var/tmp/");

    is_write || is_assignment
}

/// Check a single line for insecure temp file usage, returning a diagnostic if found
fn check_line(line: &str, line_num: usize) -> Option<Diagnostic> {
    let trimmed = line.trim();

    if trimmed.starts_with('#') || trimmed.is_empty() || trimmed.contains("mktemp") {
        return None;
    }

    for pattern in UNSAFE_TMP_PATTERNS {
        if let Some(pos) = trimmed.find(pattern) {
            if is_tmp_write_or_assignment(trimmed) {
                let span = Span::new(line_num + 1, pos + 1, line_num + 1, line.len());
                let mut diag = Diagnostic::new(
                    "SEC013",
                    Severity::Warning,
                    format!(
                        "Insecure temporary file: hardcoded {} path is vulnerable to symlink attacks - use mktemp instead",
                        pattern.trim_end_matches('/')
                    ),
                    span,
                );
                diag.fix = Some(Fix::new("$(mktemp)"));
                return Some(diag);
            }
        }
    }

    None
}

/// Check for insecure temporary file creation
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if let Some(diag) = check_line(line, line_num) {
            result.add(diag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sec013_detects_hardcoded_tmp_write() {
        let script = "echo data > /tmp/myapp.log";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SEC013");
    }

    #[test]
    fn test_sec013_detects_tmp_assignment() {
        let script = "TMPFILE=/tmp/output.$$";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SEC013");
    }

    #[test]
    fn test_sec013_detects_var_tmp() {
        let script = "LOG=/var/tmp/build.log";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sec013_safe_mktemp() {
        let script = "TMPFILE=$(mktemp)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec013_safe_mktemp_dir() {
        let script = "TMPDIR=$(mktemp -d)";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec013_ignores_comments() {
        let script = "# echo data > /tmp/myapp.log";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec013_has_autofix() {
        let script = "echo data > /tmp/myapp.log";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert!(result.diagnostics[0]
            .fix
            .as_ref()
            .is_some_and(|f| f.replacement.contains("mktemp")));
    }

    #[test]
    fn test_sec013_empty_input() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sec013_touch_tmp() {
        let script = "touch /tmp/lockfile";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sec013_cp_to_tmp() {
        let script = "cp config.yml /tmp/config.bak";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]

        #[test]
        fn prop_sec013_never_panics(s in ".*") {
            let _ = check(&s);
        }

        #[test]
        fn prop_sec013_mktemp_always_safe(
            var in "[A-Z_]{1,10}",
            flags in "(-d|-t|-p /tmp){0,1}",
        ) {
            let script = format!("{}=$(mktemp {})", var, flags);
            let result = check(&script);
            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
