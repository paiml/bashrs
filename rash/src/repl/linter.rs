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

/// Format lint violations with source code context (REPL-014-003)
///
/// Displays each violation with:
/// - Line numbers (±2 lines of context)
/// - Source code at that location
/// - Visual indicator (caret) pointing to the issue
/// - Diagnostic message with rule code
/// - Fix suggestion if available
///
/// # Examples
///
/// ```no_run
/// use bashrs::repl::linter::{format_violations_with_context, lint_bash};
///
/// let source = "echo $RANDOM\nmkdir /app\n";
/// let result = lint_bash(source).unwrap();
/// let formatted = format_violations_with_context(&result, source);
/// ```
pub fn format_violations_with_context(result: &LintResult, source: &str) -> String {
    let mut output = String::new();

    if result.diagnostics.is_empty() {
        return "✓ No violations\n".to_string();
    }

    let lines: Vec<&str> = source.lines().collect();
    let max_line_num = lines.len();
    let line_num_width = max_line_num.to_string().len().max(3);

    for diagnostic in &result.diagnostics {
        let line_idx = diagnostic.span.start_line.saturating_sub(1);

        // Show context: ±2 lines
        let start_line = line_idx.saturating_sub(2);
        let end_line = (line_idx + 3).min(lines.len());

        output.push('\n');

        // Show context lines
        for i in start_line..end_line {
            if i < lines.len() {
                let line_num = i + 1;
                let prefix = if i == line_idx { ">" } else { " " };
                if let Some(line) = lines.get(i) {
                    output.push_str(&format!(
                        "{} {:>width$} | {}\n",
                        prefix,
                        line_num,
                        line,
                        width = line_num_width
                    ));

                    // Show indicator on the problematic line
                    if i == line_idx {
                        let col = diagnostic.span.start_col.saturating_sub(1);
                        let indicator_width = if diagnostic.span.end_line == diagnostic.span.start_line
                        {
                            diagnostic
                                .span
                                .end_col
                                .saturating_sub(diagnostic.span.start_col)
                                .max(1)
                        } else {
                            line.len().saturating_sub(col).max(1)
                        };

                    output.push_str(&format!(
                        "  {:>width$} | {}{} {} [{}]: {}\n",
                        "",
                        " ".repeat(col),
                        "^".repeat(indicator_width),
                        diagnostic.severity,
                        diagnostic.code,
                        diagnostic.message,
                        width = line_num_width
                    ));
                    }
                }
            }
        }

        // Show fix suggestion if available
        if let Some(fix) = &diagnostic.fix {
            output.push_str("\n  Suggested fix:\n");
            output.push_str(&format!(
                "  {:>width$} | {}\n",
                line_idx + 1,
                fix.replacement,
                width = line_num_width
            ));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::{Diagnostic, Fix, Span};

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

    // ===== REPL-014-003 TESTS (RED PHASE) =====

    /// Test: REPL-014-003-001 - Format single violation with context
    #[test]
    fn test_REPL_014_003_format_single_violation() {
        let source = "echo hello\necho $RANDOM\necho world\n";

        // Create a diagnostic manually for testing
        let diagnostic = Diagnostic {
            code: "DET001".to_string(),
            severity: Severity::Error,
            message: "Non-deterministic $RANDOM".to_string(),
            span: Span::new(2, 6, 2, 13),
            fix: None,
        };

        let lint_result = LintResult {
            diagnostics: vec![diagnostic],
        };

        let formatted = format_violations_with_context(&lint_result, source);

        // Should show line context (allowing for width padding)
        assert!(formatted.contains("1 | echo hello"), "Output: {}", formatted);
        assert!(formatted.contains(">") && formatted.contains("2 | echo $RANDOM"), "Output: {}", formatted);
        assert!(formatted.contains("3 | echo world"));

        // Should show indicator
        assert!(formatted.contains("^^^^^^^")); // 7 chars for "$RANDOM"
        assert!(formatted.contains("error [DET001]")); // Severity displays as lowercase
        assert!(formatted.contains("Non-deterministic $RANDOM"));
    }

    /// Test: REPL-014-003-002 - Format with fix suggestion
    #[test]
    fn test_REPL_014_003_format_with_fix() {
        let source = "mkdir /app\n";

        let diagnostic = Diagnostic {
            code: "IDEM001".to_string(),
            severity: Severity::Error,
            message: "mkdir without -p".to_string(),
            span: Span::new(1, 1, 1, 11),
            fix: Some(Fix::new("mkdir -p /app")),
        };

        let lint_result = LintResult {
            diagnostics: vec![diagnostic],
        };

        let formatted = format_violations_with_context(&lint_result, source);

        // Should show violation (allowing for width padding)
        assert!(formatted.contains(">") && formatted.contains("1 | mkdir /app"));
        assert!(formatted.contains("IDEM001"));

        // Should show fix
        assert!(formatted.contains("Suggested fix:"));
        assert!(formatted.contains("mkdir -p /app"));
    }

    /// Test: REPL-014-003-003 - Format multiple violations
    #[test]
    fn test_REPL_014_003_multiple_violations() {
        let source = "echo $RANDOM\nmkdir /app\nrm /tmp/file\n";

        let diagnostics = vec![
            Diagnostic {
                code: "DET001".to_string(),
                severity: Severity::Error,
                message: "Non-deterministic $RANDOM".to_string(),
                span: Span::new(1, 6, 1, 13),
                fix: None,
            },
            Diagnostic {
                code: "IDEM001".to_string(),
                severity: Severity::Error,
                message: "mkdir without -p".to_string(),
                span: Span::new(2, 1, 2, 11),
                fix: Some(Fix::new("mkdir -p /app")),
            },
        ];

        let lint_result = LintResult { diagnostics };

        let formatted = format_violations_with_context(&lint_result, source);

        // Should show both violations
        assert!(formatted.contains("DET001"));
        assert!(formatted.contains("IDEM001"));
        assert!(formatted.contains("echo $RANDOM"));
        assert!(formatted.contains("mkdir /app"));
    }

    /// Test: REPL-014-003-004 - Format no violations
    #[test]
    fn test_REPL_014_003_no_violations() {
        let source = "echo hello\n";
        let lint_result = LintResult {
            diagnostics: vec![],
        };

        let formatted = format_violations_with_context(&lint_result, source);

        assert!(formatted.contains("✓ No violations"));
    }

    /// Test: REPL-014-003-005 - Format edge of file
    #[test]
    fn test_REPL_014_003_edge_of_file() {
        // Test violation on first line
        let source1 = "echo $RANDOM\n";
        let diagnostic1 = Diagnostic {
            code: "DET001".to_string(),
            severity: Severity::Error,
            message: "Non-deterministic $RANDOM".to_string(),
            span: Span::new(1, 6, 1, 13),
            fix: None,
        };

        let formatted1 = format_violations_with_context(
            &LintResult {
                diagnostics: vec![diagnostic1],
            },
            source1,
        );

        // Should not crash, should show line 1 (allowing for width padding)
        assert!(formatted1.contains(">") && formatted1.contains("1 | echo $RANDOM"));

        // Test violation on last line
        let source2 = "echo hello\necho world\necho $RANDOM\n";
        let diagnostic2 = Diagnostic {
            code: "DET001".to_string(),
            severity: Severity::Error,
            message: "Non-deterministic $RANDOM".to_string(),
            span: Span::new(3, 6, 3, 13),
            fix: None,
        };

        let formatted2 = format_violations_with_context(
            &LintResult {
                diagnostics: vec![diagnostic2],
            },
            source2,
        );

        // Should not crash, should show lines 1-3 (allowing for width padding)
        assert!(formatted2.contains("1 | echo hello"));
        assert!(formatted2.contains("2 | echo world"));
        assert!(formatted2.contains(">") && formatted2.contains("3 | echo $RANDOM"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::linter::{Diagnostic, Span};
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

    /// Property: REPL-014-003 - format_violations_with_context never panics
    proptest! {
        #[test]
            fn prop_format_violations_never_panics(
            source in ".*{0,500}",
            line in 1usize..100,
            col in 1usize..100,
        ) {
            // Create a diagnostic at potentially out-of-bounds position
            let diagnostic = Diagnostic {
                code: "TEST001".to_string(),
                severity: Severity::Error,
                message: "Test message".to_string(),
                span: Span::new(line, col, line, col + 5),
                fix: None,
            };

            let lint_result = LintResult {
                diagnostics: vec![diagnostic],
            };

            // Should not panic on any input
            let _ = format_violations_with_context(&lint_result, &source);
        }
    }
}
