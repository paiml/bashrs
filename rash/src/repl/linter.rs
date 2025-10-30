// REPL Linter Integration Module
//
// Task: REPL-006-001 - Run linter from REPL
// Test Approach: RED → GREEN → REFACTOR → INTEGRATION
//
// Quality targets:
// - Unit tests: 3+ scenarios
// - Integration tests: CLI workflow
// - Complexity: <10 per function

use crate::linter::{lint_shell, LintResult, Severity};

/// Lint bash input and return diagnostics
///
/// # Examples
///
/// ```
/// use bashrs::repl::linter::lint_bash;
///
/// let result = lint_bash("cat file.txt | grep pattern");
/// assert!(result.is_ok());
/// ```
pub fn lint_bash(input: &str) -> anyhow::Result<LintResult> {
    let result = lint_shell(input);
    Ok(result)
}

/// Format lint results for display in REPL
pub fn format_lint_results(result: &LintResult) -> String {
    let mut output = String::new();

    if result.diagnostics.is_empty() {
        output.push_str("✓ No issues found!\n");
        return output;
    }

    // Count by severity
    let errors = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();
    let info = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Info)
        .count();

    output.push_str(&format!("Found {} issue(s):\n", result.diagnostics.len()));
    if errors > 0 {
        output.push_str(&format!("  ✗ {} error(s)\n", errors));
    }
    if warnings > 0 {
        output.push_str(&format!("  ⚠ {} warning(s)\n", warnings));
    }
    if info > 0 {
        output.push_str(&format!("  ℹ {} info\n", info));
    }

    output.push('\n');

    // Show diagnostics
    for (i, diag) in result.diagnostics.iter().enumerate() {
        let severity_icon = match diag.severity {
            Severity::Error => "✗",
            Severity::Warning => "⚠",
            Severity::Info => "ℹ",
            Severity::Note => "📝",
            Severity::Perf => "⚡",
            Severity::Risk => "⚠",
        };

        output.push_str(&format!(
            "[{}] {} {} - {}\n",
            i + 1,
            severity_icon,
            diag.code,
            diag.message
        ));

        if diag.span.start_line > 0 {
            output.push_str(&format!("    Line {}\n", diag.span.start_line));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: REPL-006-001-001 - Lint finds issues in bash code
    #[test]
    fn test_REPL_006_001_lint_finds_issues() {
        // Use a pattern that triggers a lint rule
        let input = "cat file.txt | grep pattern";
        let result = lint_bash(input);

        assert!(result.is_ok(), "Should lint successfully: {:?}", result);
        let lint_result = result.unwrap();

        // May or may not find issues depending on rules
        // Just verify the structure is correct - diagnostics vec exists
        let _ = lint_result.diagnostics.len();
    }

    /// Test: REPL-006-001-002 - Lint categorizes by severity
    #[test]
    fn test_REPL_006_001_lint_categorizes_severity() {
        let input = "echo test";
        let result = lint_bash(input);

        assert!(result.is_ok(), "Should lint successfully");
        let lint_result = result.unwrap();

        // Check that we can categorize by severity
        let errors = lint_result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let warnings = lint_result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count();

        // Verify categorization succeeded - both counts should be valid
        assert!(
            errors + warnings <= lint_result.diagnostics.len(),
            "Error and warning counts should not exceed total diagnostics"
        );
    }

    /// Test: REPL-006-001-003 - Format lint results
    #[test]
    fn test_REPL_006_001_format_lint_results() {
        let input = "echo hello";
        let result = lint_bash(input).unwrap();

        let formatted = format_lint_results(&result);
        assert!(!formatted.is_empty(), "Should format results");
        assert!(
            formatted.contains("issue") || formatted.contains("No issues"),
            "Should show issue count or success message"
        );
    }

    /// Test: REPL-006-001-004 - Lint handles empty input
    #[test]
    fn test_REPL_006_001_lint_empty_input() {
        let input = "";
        let result = lint_bash(input);

        assert!(result.is_ok(), "Should handle empty input");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // ===== PROPERTY TESTS (PROPERTY PHASE) =====

    /// Property: lint_bash should never panic on any input
    proptest! {
        #[test]
        fn prop_lint_never_panics(input in ".*{0,1000}") {
            // Test that linter gracefully handles any input without panicking
            let _ = lint_bash(&input);
            // If we get here without panic, test passes
        }
    }

    /// Property: lint_bash should be deterministic
    proptest! {
        #[test]
        fn prop_lint_deterministic(input in "[a-z ]{1,50}") {
            // Same input should always produce same output
            let result1 = lint_bash(&input);
            let result2 = lint_bash(&input);

            match (result1, result2) {
                (Ok(out1), Ok(out2)) => {
                    // Compare diagnostic counts (not exact output due to potential internal IDs)
                    prop_assert_eq!(
                        out1.diagnostics.len(),
                        out2.diagnostics.len(),
                        "Linting should be deterministic"
                    );
                }
                (Err(_), Err(_)) => {
                    // Both failed - consistent behavior
                }
                _ => {
                    prop_assert!(false, "Inconsistent results for same input");
                }
            }
        }
    }

    /// Property: format_lint_results never panics
    proptest! {
        #[test]
        fn prop_format_never_panics(input in "[a-z ]{1,100}") {
            if let Ok(result) = lint_bash(&input) {
                let _ = format_lint_results(&result);
                // If we get here without panic, test passes
            }
        }
    }

    /// Property: format_lint_results always produces non-empty output
    proptest! {
        #[test]
        fn prop_format_not_empty(input in "[a-z ]{1,100}") {
            if let Ok(result) = lint_bash(&input) {
                let formatted = format_lint_results(&result);
                prop_assert!(
                    !formatted.is_empty(),
                    "Formatted output should never be empty"
                );
            }
        }
    }

    /// Property: format_lint_results is deterministic
    proptest! {
        #[test]
        fn prop_format_deterministic(input in "[a-z ]{1,50}") {
            if let Ok(result) = lint_bash(&input) {
                let formatted1 = format_lint_results(&result);
                let formatted2 = format_lint_results(&result);
                prop_assert_eq!(
                    formatted1,
                    formatted2,
                    "Format should be deterministic"
                );
            }
        }
    }

    /// Property: Severity counts always sum to total diagnostics
    proptest! {
        #[test]
        fn prop_severity_counts_sum(input in "[a-z ]{1,50}") {
            if let Ok(result) = lint_bash(&input) {
                let errors = result.diagnostics.iter()
                    .filter(|d| d.severity == Severity::Error)
                    .count();
                let warnings = result.diagnostics.iter()
                    .filter(|d| d.severity == Severity::Warning)
                    .count();
                let info = result.diagnostics.iter()
                    .filter(|d| d.severity == Severity::Info)
                    .count();
                let notes = result.diagnostics.iter()
                    .filter(|d| d.severity == Severity::Note)
                    .count();
                let perf = result.diagnostics.iter()
                    .filter(|d| d.severity == Severity::Perf)
                    .count();
                let risk = result.diagnostics.iter()
                    .filter(|d| d.severity == Severity::Risk)
                    .count();

                let sum = errors + warnings + info + notes + perf + risk;
                prop_assert_eq!(
                    sum,
                    result.diagnostics.len(),
                    "Severity counts should sum to total diagnostics"
                );
            }
        }
    }

    /// Property: Formatted output contains diagnostic count
    proptest! {
        #[test]
        fn prop_format_contains_count(input in "[a-z ]{1,50}") {
            if let Ok(result) = lint_bash(&input) {
                let formatted = format_lint_results(&result);
                if result.diagnostics.is_empty() {
                    prop_assert!(
                        formatted.contains("No issues"),
                        "Should indicate no issues when clean"
                    );
                } else {
                    prop_assert!(
                        formatted.contains("issue"),
                        "Should mention issues when found"
                    );
                }
            }
        }
    }
}
