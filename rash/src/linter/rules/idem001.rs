//! IDEM001: Non-idempotent mkdir
//!
//! **Rule**: Detect `mkdir` without `-p` flag
//!
//! **Why this matters**:
//! `mkdir` without `-p` fails if directory exists, making scripts non-idempotent.
//! Re-running the script will fail instead of succeeding.
//!
//! **Auto-fix**: Add `-p` flag
//!
//! ## Examples
//!
//! ❌ **BAD** (non-idempotent):
//! ```bash
//! mkdir /app/releases
//! ```
//!
//! ✅ **GOOD** (idempotent):
//! ```bash
//! mkdir -p /app/releases
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for mkdir without -p flag
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Look for mkdir without -p
        if line.contains("mkdir ") && !line.contains("mkdir -p") {
            if let Some(col) = line.find("mkdir ") {
                let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 6);

                let fix = Fix::new_with_assumptions(
                    "mkdir -p",
                    vec!["Directory creation failure is not a critical error".to_string()],
                );

                let diag = Diagnostic::new(
                    "IDEM001",
                    Severity::Warning,
                    "Non-idempotent mkdir - add -p flag (SAFE-WITH-ASSUMPTIONS)",
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
    fn test_IDEM001_detects_mkdir_without_p() {
        let script = "mkdir /app/releases";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "IDEM001");
        assert_eq!(diag.severity, Severity::Warning);
    }

    #[test]
    fn test_IDEM001_no_warning_with_p_flag() {
        let script = "mkdir -p /app/releases";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_IDEM001_provides_fix() {
        let script = "mkdir /tmp/foo";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "mkdir -p");
    }

    #[test]
    fn test_IDEM001_multiple_mkdir() {
        let script = "mkdir /a\nmkdir /b";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 2);
    }
}
