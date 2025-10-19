//! DET001: Non-deterministic $RANDOM usage
//!
//! **Rule**: Detect usage of `$RANDOM` which produces non-deterministic output
//!
//! **Why this matters**:
//! Scripts using `$RANDOM` will produce different output on each run,
//! breaking determinism and making testing/debugging harder.
//!
//! **Auto-fix**: Suggest replacing with version-based identifier
//!
//! ## Examples
//!
//! ❌ **BAD** (non-deterministic):
//! ```bash
//! SESSION_ID=$RANDOM
//! ```
//!
//! ✅ **GOOD** (deterministic):
//! ```bash
//! SESSION_ID="session-${VERSION}"
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for $RANDOM usage in shell script
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if let Some(col) = line.find("$RANDOM") {
            let span = Span::new(
                line_num + 1,  // start_line
                col + 1,       // start_col
                line_num + 1,  // end_line
                col + 8,       // end_col ($RANDOM is 7 chars, +1 for 1-indexed)
            );

            let fix = Fix::new_unsafe(vec![
                "Option 1: Use version/build ID: SESSION_ID=\"session-${VERSION}\"".to_string(),
                "Option 2: Use timestamp as argument: SESSION_ID=\"$1\"".to_string(),
                "Option 3: Use hash of input: SESSION_ID=$(echo \"$INPUT\" | sha256sum | cut -c1-8)".to_string(),
            ]);

            let diag = Diagnostic::new(
                "DET001",
                Severity::Error,
                "Non-deterministic $RANDOM usage - requires manual fix (UNSAFE)",
                span,
            )
            .with_fix(fix);

            result.add(diag);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_DET001_detects_random_usage() {
        let script = "SESSION_ID=$RANDOM";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "DET001");
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "Non-deterministic $RANDOM usage - requires manual fix (UNSAFE)");
    }

    #[test]
    fn test_DET001_provides_fix() {
        let script = "ID=$RANDOM";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        // UNSAFE fix: no automatic replacement, provides suggestions
        assert_eq!(fix.replacement, "");
        assert!(fix.is_unsafe());
        assert!(!fix.suggested_alternatives.is_empty());
        assert!(fix.suggested_alternatives.len() >= 3);
    }

    #[test]
    fn test_DET001_no_false_positive() {
        let script = "ID=\"session-${VERSION}\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_DET001_multiple_random() {
        let script = "A=$RANDOM\nB=$RANDOM";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 2);
    }
}
