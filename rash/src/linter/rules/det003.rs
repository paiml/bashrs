//! DET003: Unordered wildcard usage
//!
//! **Rule**: Detect wildcards without sorting for deterministic results
//!
//! **Why this matters**:
//! File glob results vary by filesystem and can change between runs,
//! breaking determinism.
//!
//! **Auto-fix**: Wrap wildcard with sort
//!
//! ## Examples
//!
//! ❌ **BAD** (non-deterministic):
//! ```bash
//! FILES=$(ls *.txt)
//! for f in *.c; do echo $f; done
//! ```
//!
//! ✅ **GOOD** (deterministic):
//! ```bash
//! FILES=$(ls *.txt | sort)
//! for f in $(ls *.c | sort); do echo $f; done
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for unordered wildcard usage
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Simple heuristic: look for * in variable assignments or command substitutions
        // without | sort
        if line.contains('*') && !line.contains("| sort") {
            // Check if it's in a potentially problematic context
            if line.contains("$(ls") || line.contains("for ") && line.contains(" in ") {
                if let Some(col) = line.find('*') {
                    let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 2);

                    let diag = Diagnostic::new(
                        "DET003",
                        Severity::Warning,
                        "Unordered wildcard - results may vary",
                        span,
                    )
                    .with_fix(Fix::new("| sort"));

                    result.add(diag);
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
    fn test_DET003_detects_ls_wildcard() {
        let script = "FILES=$(ls *.txt)";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "DET003");
        assert_eq!(diag.severity, Severity::Warning);
    }

    #[test]
    fn test_DET003_detects_for_loop_wildcard() {
        let script = "for f in *.c; do echo $f; done";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_DET003_no_warning_with_sort() {
        let script = "FILES=$(ls *.txt | sort)";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_DET003_provides_fix() {
        let script = "FILES=$(ls *.txt)";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        assert_eq!(fix.replacement, "| sort");
    }
}
