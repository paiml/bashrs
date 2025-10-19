//! SEC006: Unsafe Temporary File Creation
//!
//! **Rule**: Detect predictable temporary file names (race condition vulnerability)
//!
//! **Why this matters**:
//! Predictable temp file names enable symlink attacks and race conditions.
//! Attackers can predict the filename and create malicious symlinks.
//!
//! **Auto-fix**: Safe (suggest mktemp)
//!
//! ## Examples
//!
//! ❌ **PREDICTABLE TEMP FILES** (race condition):
//! ```bash
//! TMPFILE="/tmp/myapp.$$"
//! TMPFILE="/tmp/script_temp"
//! TMP_DIR="/tmp/build_cache"
//! ```
//!
//! ✅ **SECURE TEMP FILES** (auto-fixable):
//! ```bash
//! TMPFILE="$(mktemp)"
//! TMPDIR="$(mktemp -d)"
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for unsafe temporary file creation
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Look for predictable /tmp/ assignments
        if line.contains("/tmp/") && !line.contains("mktemp") {
            // Check if it's an assignment (VAR="/tmp/...")
            if let Some(eq_pos) = line.find('=') {
                let after_eq = &line[eq_pos + 1..].trim_start();

                // Check for quoted /tmp/ path (predictable filename)
                if (after_eq.starts_with("\"/tmp/") || after_eq.starts_with("'/tmp/")) {
                    // Check if it uses $$ (process ID) - still vulnerable
                    if let Some(col) = line.find("/tmp/") {
                        let span = Span::new(
                            line_num + 1,
                            col + 1,
                            line_num + 1,
                            col + 6,  // "/tmp/" is 5 chars
                        );

                        let diag = Diagnostic::new(
                            "SEC006",
                            Severity::Warning,
                            "Unsafe temp file - use mktemp for secure random names",
                            span,
                        )
                        .with_fix(Fix::new("$(mktemp)"));

                        result.add(diag);
                    }
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
    fn test_SEC006_detects_predictable_tmp() {
        let script = r#"TMPFILE="/tmp/myapp.$$""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC006");
        assert_eq!(diag.severity, Severity::Warning);
    }

    #[test]
    fn test_SEC006_detects_static_tmp() {
        let script = "TMP_DIR=\"/tmp/build_cache\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_SEC006_no_warning_mktemp() {
        let script = "TMPFILE=\"$(mktemp)\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC006_no_warning_mktemp_dir() {
        let script = "TMPDIR=\"$(mktemp -d)\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC006_provides_fix() {
        let script = "TMPFILE=\"/tmp/script_temp\"";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "$(mktemp)");
    }
}
