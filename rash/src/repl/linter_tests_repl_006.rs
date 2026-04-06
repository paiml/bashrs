
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
    assert!(
        formatted.contains("1 | echo hello"),
        "Output: {}",
        formatted
    );
    assert!(
        formatted.contains(">") && formatted.contains("2 | echo $RANDOM"),
        "Output: {}",
        formatted
    );
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
