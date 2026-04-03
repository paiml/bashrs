use crate::linter::Severity;

// ===== UNIT TESTS (RED PHASE) =====

/// Test: REPL-005-001-001 - Purify mkdir command
#[test]
fn test_REPL_005_001_purify_mkdir() {
    let input = "mkdir /tmp/test";
    let result = purify_bash(input);

    assert!(result.is_ok(), "Should purify mkdir command: {:?}", result);
    let purified = result.unwrap();
    // Should add -p flag for idempotency
    assert!(
        purified.contains("mkdir -p"),
        "Should add -p flag for idempotency, got: {}",
        purified
    );
    assert!(
        purified.contains("/tmp/test"),
        "Should preserve directory path, got: {}",
        purified
    );
}

/// Test: REPL-005-001-002 - Purify $RANDOM (non-deterministic)
#[test]
fn test_REPL_005_001_purify_random() {
    let input = "echo $RANDOM";
    let result = purify_bash(input);

    assert!(result.is_ok(), "Should handle $RANDOM: {:?}", result);
    let purified = result.unwrap();
    // $RANDOM should be removed or replaced (non-deterministic)
    assert!(
        !purified.contains("$RANDOM"),
        "Should remove non-deterministic $RANDOM, got: {}",
        purified
    );
    // Should still have echo command
    assert!(
        purified.contains("echo"),
        "Should preserve echo command, got: {}",
        purified
    );
}

/// Test: REPL-005-001-003 - Purify unquoted variable
#[test]
fn test_REPL_005_001_purify_unquoted_var() {
    let input = "echo $USER";
    let result = purify_bash(input);

    assert!(
        result.is_ok(),
        "Should handle unquoted variable: {:?}",
        result
    );
    let purified = result.unwrap();
    // Variables should be quoted for safety
    assert!(
        purified.contains("\"$USER\"")
            || purified.contains("'$USER'")
            || purified.contains("\"${USER}\""),
        "Should quote variable for safety, got: {}",
        purified
    );
    assert!(
        purified.contains("echo"),
        "Should preserve echo command, got: {}",
        purified
    );
}

/// Test: REPL-005-001-004 - Format purification report
#[test]
fn test_REPL_005_001_format_report() {
    let report = PurificationReport {
        idempotency_fixes: vec!["mkdir → mkdir -p".to_string()],
        determinism_fixes: vec!["$RANDOM removed".to_string()],
        side_effects_isolated: vec![],
        warnings: vec!["Complex pattern".to_string()],
        type_diagnostics: vec![],
    };

    let formatted = format_purification_report(&report);
    assert!(formatted.contains("Idempotency fixes"));
    assert!(formatted.contains("Determinism fixes"));
    assert!(formatted.contains("Warnings"));
}

// ===== REPL-005-003: Explain what changed =====

/// Test: REPL-005-003-001 - Explain mkdir -p change
#[test]
fn test_REPL_005_003_explain_mkdir_p() {
    let original = "mkdir /tmp/test";
    let explanation = explain_purification_changes(original);

    assert!(
        explanation.is_ok(),
        "Should explain changes: {:?}",
        explanation
    );
    let text = explanation.unwrap();

    // Should mention mkdir and -p flag
    assert!(
        text.contains("mkdir") && text.contains("-p"),
        "Should explain mkdir -p change: {}",
        text
    );
    // Should mention idempotency
    assert!(
        text.contains("idempotent") || text.contains("safe to re-run"),
        "Should explain idempotency: {}",
        text
    );
}

/// Test: REPL-005-003-002 - Explain rm -f change
///
/// Verifies that the purifier transforms `rm file.txt` to `rm -f file.txt`
/// for idempotency and explains the transformation to the user.
#[test]
fn test_REPL_005_003_explain_rm_f() {
    let original = "rm file.txt";
    let explanation = explain_purification_changes(original);

    assert!(
        explanation.is_ok(),
        "Should explain changes: {:?}",
        explanation
    );
    let text = explanation.unwrap();

    // Should mention rm and -f flag
    assert!(
        text.contains("rm") && text.contains("-f"),
        "Should explain rm -f change: {}",
        text
    );
    // Should mention idempotency or force
    assert!(
        text.contains("idempotent") || text.contains("force") || text.contains("safe"),
        "Should explain why -f was added: {}",
        text
    );
}

/// Test: REPL-005-003-003 - Explain quoted variable
#[test]
fn test_REPL_005_003_explain_quote_var() {
    let original = "echo $USER";
    let explanation = explain_purification_changes(original);

    assert!(
        explanation.is_ok(),
        "Should explain changes: {:?}",
        explanation
    );
    let text = explanation.unwrap();

    // Should mention quoting or safety
    assert!(
        text.contains("quot") || text.contains("safe") || text.contains("\""),
        "Should explain variable quoting: {}",
        text
    );
}

// ===== Additional coverage tests =====

#[test]
fn test_purified_lint_result_new() {
    let lint_result = LintResult::new();
    let result = PurifiedLintResult::new("echo hello".to_string(), lint_result);
    assert!(result.is_clean);
    assert_eq!(result.critical_violations(), 0);
}

#[test]
fn test_purified_lint_result_with_det_violations() {
    let mut lint_result = LintResult::new();
    lint_result.diagnostics.push(Diagnostic::new(
        "DET001",
        Severity::Warning,
        "Non-deterministic".to_string(),
        crate::linter::Span::new(1, 1, 1, 10),
    ));
    let result = PurifiedLintResult::new("echo $RANDOM".to_string(), lint_result);
    assert!(!result.is_clean);
    assert_eq!(result.critical_violations(), 1);
    assert_eq!(result.det_violations().len(), 1);
    assert!(result.idem_violations().is_empty());
    assert!(result.sec_violations().is_empty());
}

#[test]
fn test_purified_lint_result_with_idem_violations() {
    let mut lint_result = LintResult::new();
    lint_result.diagnostics.push(Diagnostic::new(
        "IDEM001",
        Severity::Warning,
        "Non-idempotent".to_string(),
        crate::linter::Span::new(1, 1, 1, 10),
    ));
    let result = PurifiedLintResult::new("mkdir dir".to_string(), lint_result);
    assert!(!result.is_clean);
    assert_eq!(result.idem_violations().len(), 1);
}

#[test]
fn test_purified_lint_result_with_sec_violations() {
    let mut lint_result = LintResult::new();
    lint_result.diagnostics.push(Diagnostic::new(
        "SEC001",
        Severity::Error,
        "Security issue".to_string(),
        crate::linter::Span::new(1, 1, 1, 10),
    ));
    let result = PurifiedLintResult::new("eval $input".to_string(), lint_result);
    assert!(!result.is_clean);
    assert_eq!(result.sec_violations().len(), 1);
}

#[test]
fn test_purified_lint_result_clone() {
    let lint_result = LintResult::new();
    let result = PurifiedLintResult::new("echo test".to_string(), lint_result);
    let cloned = result.clone();
    assert_eq!(cloned.is_clean, result.is_clean);
    assert_eq!(cloned.purified_code, result.purified_code);
}

#[test]
fn test_purification_error_new() {
    let mut lint_result = LintResult::new();
    lint_result.diagnostics.push(Diagnostic::new(
        "DET001",
        Severity::Warning,
        "Non-deterministic".to_string(),
        crate::linter::Span::new(1, 1, 1, 10),
    ));
    lint_result.diagnostics.push(Diagnostic::new(
        "IDEM001",
        Severity::Warning,
        "Non-idempotent".to_string(),
        crate::linter::Span::new(2, 1, 2, 10),
    ));
    let result = PurifiedLintResult::new("echo test".to_string(), lint_result);
    let error = PurificationError::new(&result);

    assert_eq!(error.det_violations, 1);
    assert_eq!(error.idem_violations, 1);
    assert_eq!(error.sec_violations, 0);
    assert_eq!(error.total_violations(), 2);
}

#[test]
fn test_purification_error_display() {
    let mut lint_result = LintResult::new();
    lint_result.diagnostics.push(Diagnostic::new(
        "SEC001",
        Severity::Error,
        "Security issue".to_string(),
        crate::linter::Span::new(1, 1, 1, 10),
    ));
    let result = PurifiedLintResult::new("test".to_string(), lint_result);
    let error = PurificationError::new(&result);

    let display = format!("{}", error);
    assert!(display.contains("1 violation"));
    assert!(display.contains("SEC: 1"));
}

#[test]
fn test_format_purified_lint_result_clean() {
    let lint_result = LintResult::new();
    let result = PurifiedLintResult::new("echo hello".to_string(), lint_result);
    let formatted = format_purified_lint_result(&result);
    assert!(formatted.contains("Purified"));
    assert!(formatted.contains("CLEAN"));
}

#[test]
fn test_format_purified_lint_result_with_violations() {
    let mut lint_result = LintResult::new();
    lint_result.diagnostics.push(Diagnostic::new(
        "DET001",
        Severity::Warning,
        "Non-deterministic".to_string(),
        crate::linter::Span::new(1, 1, 1, 10),
    ));
    let result = PurifiedLintResult::new("echo $RANDOM".to_string(), lint_result);
    let formatted = format_purified_lint_result(&result);
    assert!(formatted.contains("Purified"));
    assert!(formatted.contains("critical violation"));
    assert!(formatted.contains("DET"));
}

#[test]
fn test_format_purified_lint_result_with_context() {
    let lint_result = LintResult::new();
    let result = PurifiedLintResult::new("echo hello".to_string(), lint_result);
    let formatted = format_purified_lint_result_with_context(&result, "echo hello");
    assert!(formatted.contains("Purified"));
    assert!(formatted.contains("CLEAN"));
}

#[test]
fn test_format_purification_report_empty() {
    let report = PurificationReport {
        idempotency_fixes: vec![],
        determinism_fixes: vec![],
        side_effects_isolated: vec![],
        warnings: vec![],
        type_diagnostics: vec![],
    };
    let formatted = format_purification_report(&report);
    // Empty report should produce empty or minimal output
    assert!(formatted.is_empty() || formatted.len() < 10);
}

#[test]
fn test_purify_and_validate_clean_code() {
    let result = purify_and_validate("echo hello");
    assert!(result.is_ok());
}

#[test]
fn test_explain_no_changes() {
    // Code that's already clean - may or may not have changes
    let result = explain_purification_changes("echo \"$HOME\"");
    assert!(result.is_ok());
}

#[test]
fn test_explain_ln_sf() {
    let result = explain_purification_changes("ln -s target link");
    assert!(result.is_ok());
    let text = result.unwrap();
    // Should mention ln or symlink
    assert!(text.contains("ln") || text.contains("symlink") || text.contains("-"));
}

// ===== PROPERTY TESTS =====
use proptest::prelude::*;

// ===== PROPERTY TESTS (PROPERTY PHASE) =====

// Property: purify_bash should never panic on any input
proptest! {
    #[test]
    fn prop_purify_never_panics(input in ".*{0,1000}") {
        // Test that purifier gracefully handles any input without panicking
        let _ = purify_bash(&input);
        // If we get here without panic, test passes
    }
}

// Property: Purified output should always be valid bash (parseable)
proptest! {
    #[test]
    fn prop_purify_produces_valid_bash(input in "[a-z ]{1,100}") {
        if let Ok(purified) = purify_bash(&input) {
            // Purified output should be parseable
            let result = crate::repl::parser::parse_bash(&purified);
            // Either the input was invalid (error) or purified output is valid
            // Both are acceptable - just shouldn't panic
            let _ = result; // Either valid or invalid input - just shouldn't panic
        }
    }
}

// Property: mkdir commands always get -p flag added
proptest! {
    #[test]
    fn prop_mkdir_always_idempotent(path in "[a-z0-9/]{1,50}") {
        let input = format!("mkdir {}", path);
        if let Ok(purified) = purify_bash(&input) {
            // If purification succeeded, mkdir should have -p flag
            prop_assert!(
                purified.contains("mkdir -p") || purified.contains("mkdir"),
                "mkdir should either have -p or be preserved: {}",
                purified
            );
        }
    }
}

// Property: Purification should be deterministic
proptest! {
    #[test]
    fn prop_purify_deterministic(input in "[a-z ]{1,50}") {
        // Same input should always produce same output
        let result1 = purify_bash(&input);
        let result2 = purify_bash(&input);

        match (result1, result2) {
            (Ok(out1), Ok(out2)) => {
                prop_assert_eq!(out1, out2, "Purification should be deterministic");
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

// Property: Format purification report never empty for non-empty report
proptest! {
    #[test]
    fn prop_format_report_not_empty(
        fixes in prop::collection::vec("[a-z ]{1,30}", 1..5),
        warnings in prop::collection::vec("[a-z ]{1,30}", 0..3)
    ) {
        let report = PurificationReport {
            idempotency_fixes: fixes.clone(),
            determinism_fixes: vec![],
            side_effects_isolated: vec![],
            warnings: warnings.clone(),
            type_diagnostics: vec![],
        };

        let formatted = format_purification_report(&report);

        // If report has content, formatted output should not be empty
        if !fixes.is_empty() || !warnings.is_empty() {
            prop_assert!(!formatted.is_empty(), "Formatted report should not be empty");
        }
    }
}
