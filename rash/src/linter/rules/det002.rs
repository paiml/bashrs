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
    let lines: Vec<&str> = source.lines().collect();

    // Track if previous lines had an intentional marker
    let mut intentional_context = false;

    for (line_num, line) in lines.iter().enumerate() {
        // Check for marker comments that indicate intentional timestamp usage
        if is_intentional_timestamp_marker(line) {
            intentional_context = true;
            continue;
        }

        // Reset context if we encounter code that's not a comment/assignment
        if !line.trim().is_empty() && !line.trim().starts_with('#') && !is_variable_assignment(line)
        {
            intentional_context = false;
        }

        // Check for various date patterns
        let patterns = [("date +%s", 8), ("$(date", 6), ("`date", 5)];

        for (pattern, len) in patterns {
            if let Some(col) = line.find(pattern) {
                // Skip if this is intentional timestamp for tracking/benchmarking
                if intentional_context && is_timestamp_for_tracking(line) {
                    continue;
                }

                let span = Span::new(line_num + 1, col + 1, line_num + 1, col + len + 1);

                let fix = Fix::new_unsafe(vec![
                    "Option 1: Use version: RELEASE=\"release-${VERSION}\"".to_string(),
                    "Option 2: Use git commit: RELEASE=\"release-$(git rev-parse --short HEAD)\""
                        .to_string(),
                    "Option 3: Pass as argument: RELEASE=\"release-$1\"".to_string(),
                    "Option 4: Use SOURCE_DATE_EPOCH for reproducible builds".to_string(),
                    "Option 5: Mark as intentional: # Intentional: timestamp for result tracking"
                        .to_string(),
                ]);

                let diag = Diagnostic::new(
                    "DET002",
                    Severity::Error,
                    "Non-deterministic timestamp usage - requires manual fix (UNSAFE)",
                    span,
                )
                .with_fix(fix);

                result.add(diag);
                break; // Only report once per line
            }
        }
    }

    result
}

/// Check if a line contains a marker indicating intentional timestamp usage
fn is_intentional_timestamp_marker(line: &str) -> bool {
    let line_lower = line.to_lowercase();
    let markers = [
        "intentional: timestamp",
        "intentional timestamp",
        "timestamp for result tracking",
        "timestamp for tracking",
        "benchmark result",
        "logging timestamp",
        "log timestamp",
    ];

    markers.iter().any(|marker| line_lower.contains(marker))
}

/// Check if timestamp is used for file tracking (not program logic)
fn is_timestamp_for_tracking(line: &str) -> bool {
    // Timestamp used in variable assignment for filenames is tracking
    // Timestamp in conditional logic is NOT tracking
    let line_trimmed = line.trim();

    // Not tracking if used in conditional
    if line_trimmed.starts_with("if ")
        || line_trimmed.starts_with("elif ")
        || line_trimmed.starts_with("while ")
        || line_trimmed.contains("[ $(date")
        || line_trimmed.contains("[[ $(date")
    {
        return false;
    }

    // Tracking if used in variable assignment
    line_trimmed.contains('=') && !line_trimmed.starts_with('[')
}

/// Check if line is a variable assignment
fn is_variable_assignment(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && !trimmed.starts_with('#')
        && trimmed.contains('=')
        && !trimmed.starts_with('[')
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
        // UNSAFE fix: no automatic replacement, provides suggestions
        assert_eq!(fix.replacement, "");
        assert!(fix.is_unsafe());
        assert!(!fix.suggested_alternatives.is_empty());
        assert!(fix.suggested_alternatives.len() >= 3);
    }

    #[test]
    fn test_DET002_no_false_positive() {
        let script = "RELEASE=\"release-${VERSION}\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    // RED TEST: Issue #43 - Allow timestamps for benchmark result tracking
    #[test]
    fn test_DET002_allows_intentional_timestamp_for_benchmarks() {
        let script = r#"#!/bin/bash
# Intentional: timestamp for result tracking
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
RESULT_FILE="benchmarks/results/baseline_$TIMESTAMP.md"
"#;
        let result = check(script);

        // Should NOT flag as error when marked intentional for tracking
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Intentionally marked timestamp for benchmark tracking should not be flagged"
        );
    }

    #[test]
    fn test_DET002_allows_benchmark_result_comment() {
        // Alternative marker: "benchmark result" in comment
        let script = r#"#!/bin/bash
# Generate benchmark result file
RESULT_FILE="results/baseline_$(date +%s).md"
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Timestamp for benchmark/logging should be allowed with marker comment"
        );
    }

    #[test]
    fn test_DET002_still_flags_timestamp_in_logic() {
        // Even with marker, timestamp in program logic should be flagged
        let script = r#"#!/bin/bash
# Intentional: timestamp for result tracking
if [ $(date +%s) -gt 1000 ]; then
    echo "error"
fi
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            1,
            "Timestamp in logic should still be flagged even with marker"
        );
    }
}
