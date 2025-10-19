//! DET002: Non-deterministic timestamp usage
//!
//! **Rule**: Detect usage of `date` commands that produce timestamps
//!
//! **Why this matters**:
//! Scripts using `date +%s` or similar will produce different output on each run,
//! breaking determinism and making reproducible builds impossible.
//!
//! **Auto-fix**: Suggest replacing with version-based identifier
//!
//! ## Examples
//!
//! ❌ **BAD** (non-deterministic):
//! ```bash
//! RELEASE="release-$(date +%s)"
//! BUILD_ID=$(date +%Y%m%d%H%M%S)
//! ```
//!
//! ✅ **GOOD** (deterministic):
//! ```bash
//! RELEASE="release-${VERSION}"
//! BUILD_ID="${VERSION}"
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for timestamp usage in shell script
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Check for various date patterns
        let patterns = [
            ("date +%s", 8),
            ("$(date", 6),
            ("`date", 5),
        ];

        for (pattern, len) in patterns {
            if let Some(col) = line.find(pattern) {
                let span = Span::new(
                    line_num + 1,
                    col + 1,
                    line_num + 1,
                    col + len + 1,
                );

                let diag = Diagnostic::new(
                    "DET002",
                    Severity::Error,
                    "Non-deterministic timestamp usage",
                    span,
                )
                .with_fix(Fix::new("${VERSION}"));

                result.add(diag);
                break; // Only report once per line
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_DET002_detects_date_epoch() {
        let script = "RELEASE=\"release-$(date +%s)\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "DET002");
        assert_eq!(diag.severity, Severity::Error);
    }

    #[test]
    fn test_DET002_detects_date_command_sub() {
        let script = "BUILD_ID=$(date +%Y%m%d)";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_DET002_detects_backtick_date() {
        let script = "TIMESTAMP=`date`";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_DET002_provides_fix() {
        let script = "ID=$(date +%s)";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "${VERSION}");
    }

    #[test]
    fn test_DET002_no_false_positive() {
        let script = "RELEASE=\"release-${VERSION}\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }
}
