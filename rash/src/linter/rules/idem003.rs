//! IDEM003: Non-idempotent ln
//!
//! **Rule**: Detect `ln -s` without removing existing symlink first
//!
//! **Why this matters**:
//! `ln -s` fails if symlink exists, making scripts non-idempotent.
//! Re-running the script will fail instead of succeeding.
//!
//! **Auto-fix**: Suggest prepending `rm -f`
//!
//! ## Examples
//!
//! ❌ **BAD** (non-idempotent):
//! ```bash
//! ln -s /app/releases/v1.0 /app/current
//! ```
//!
//! ✅ **GOOD** (idempotent):
//! ```bash
//! rm -f /app/current && ln -s /app/releases/v1.0 /app/current
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for ln -s without rm -f first
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Look for ln -s without preceding rm -f
        if line.contains("ln -s") && !line.contains("rm -f") {
            if let Some(col) = line.find("ln -s") {
                let span = Span::new(
                    line_num + 1,
                    col + 1,
                    line_num + 1,
                    col + 6,
                );

                let diag = Diagnostic::new(
                    "IDEM003",
                    Severity::Warning,
                    "Non-idempotent ln - remove target first",
                    span,
                )
                .with_fix(Fix::new("rm -f <target> && ln -s"));

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
    fn test_IDEM003_detects_ln_without_rm() {
        let script = "ln -s /app/releases/v1.0 /app/current";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "IDEM003");
        assert_eq!(diag.severity, Severity::Warning);
    }

    #[test]
    fn test_IDEM003_no_warning_with_rm() {
        let script = "rm -f /app/current && ln -s /app/releases/v1.0 /app/current";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_IDEM003_provides_fix() {
        let script = "ln -s /src /dst";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert!(fix.replacement.contains("rm -f"));
    }
}
