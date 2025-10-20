//! IDEM002: Non-idempotent rm
//!
//! **Rule**: Detect `rm` without `-f` flag
//!
//! **Why this matters**:
//! `rm` without `-f` fails if file doesn't exist, making scripts non-idempotent.
//! Re-running the script will fail instead of succeeding.
//!
//! **Auto-fix**: Add `-f` flag
//!
//! ## Examples
//!
//! ❌ **BAD** (non-idempotent):
//! ```bash
//! rm /app/current
//! ```
//!
//! ✅ **GOOD** (idempotent):
//! ```bash
//! rm -f /app/current
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for rm without -f flag
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Look for rm without -f (but allow -rf, -fr)
        if line.contains("rm ") && !line.contains("rm -") {
            if let Some(col) = line.find("rm ") {
                let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 3);

                let fix = Fix::new_with_assumptions(
                    "rm -f",
                    vec!["Missing file is not an error condition".to_string()],
                );

                let diag = Diagnostic::new(
                    "IDEM002",
                    Severity::Warning,
                    "Non-idempotent rm - add -f flag (SAFE-WITH-ASSUMPTIONS)",
                    span,
                )
                .with_fix(fix);

                result.add(diag);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_IDEM002_detects_rm_without_f() {
        let script = "rm /app/current";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "IDEM002");
        assert_eq!(diag.severity, Severity::Warning);
    }

    #[test]
    fn test_IDEM002_no_warning_with_f_flag() {
        let script = "rm -f /app/current";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_IDEM002_no_warning_with_rf() {
        let script = "rm -rf /app/releases";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_IDEM002_provides_fix() {
        let script = "rm /tmp/foo";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "rm -f");
    }
}
