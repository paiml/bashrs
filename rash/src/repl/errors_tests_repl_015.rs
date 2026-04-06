
use super::*;
use crate::linter::{Fix, Span};

// ===== UNIT TESTS (RED PHASE) =====

/// Test: REPL-015-003-001 - Format parse error
#[test]
fn test_REPL_015_003_001_format_parse_error() {
    let source = "if [ -f test";
    let error = format_parse_error("Expected 'then'", 1, 13, source);

    assert_eq!(error.error_type, ErrorType::Parse);
    assert_eq!(error.severity, Severity::Error);
    assert!(error.message.contains("Expected"));

    let formatted = format_error(&error);
    assert!(formatted.contains("✗"));
    assert!(formatted.contains("if [ -f test"));
    assert!(formatted.contains("^"));
}

/// Test: REPL-015-003-002 - Format lint error with fix
#[test]
fn test_REPL_015_003_002_format_lint_error() {
    let diagnostic = Diagnostic {
        code: "IDEM001".to_string(),
        severity: LintSeverity::Error,
        message: "mkdir without -p".to_string(),
        span: Span::new(1, 1, 1, 11),
        fix: Some(Fix::new("mkdir -p /tmp")),
    };

    let source = "mkdir /tmp";
    let error = format_lint_error(&diagnostic, source);

    assert_eq!(error.error_type, ErrorType::Lint);
    assert_eq!(error.code, Some("IDEM001".to_string()));
    assert!(error.suggestion.is_some());
    assert!(error.suggestion.as_ref().unwrap().auto_fixable);

    let formatted = format_error(&error);
    assert!(formatted.contains("IDEM001"));
    assert!(formatted.contains("mkdir /tmp"));
    assert!(formatted.contains("Auto-fix"));
}

/// Test: REPL-015-003-003 - Format command error with suggestion
#[test]
fn test_REPL_015_003_003_format_command_error() {
    let commands = vec!["purify", "lint", "quit", "ast"];
    let error = format_command_error("purfy", &commands);

    assert_eq!(error.error_type, ErrorType::Command);
    assert!(error.message.contains("Unknown command"));
    assert!(error.suggestion.is_some());

    let formatted = format_error(&error);
    assert!(formatted.contains("purfy"));
    assert!(formatted.contains("Did you mean"));
    assert!(formatted.contains("purify"));
}

/// Test: REPL-015-003-004 - Command suggestion with edit distance
#[test]
fn test_REPL_015_003_004_suggest_command() {
    let commands = vec!["purify", "lint", "quit", "ast", "help"];

    // Close matches
    assert_eq!(
        suggest_command("purfy", &commands),
        Some("purify".to_string())
    );
    assert_eq!(suggest_command("lnt", &commands), Some("lint".to_string()));
    assert_eq!(suggest_command("qit", &commands), Some("quit".to_string()));

    // No close matches
    assert_eq!(suggest_command("foobar", &commands), None);
    assert_eq!(suggest_command("xyz123", &commands), None);
}

/// Test: REPL-015-003-005 - Source context extraction
#[test]
fn test_REPL_015_003_005_source_context() {
    let context = SourceContext {
        line: 2,
        column: 13,
        source_line: "if [ -f test".to_string(),
        length: 1,
    };

    let formatted = format_source_context(&context);
    assert!(formatted.contains("2 |"));
    assert!(formatted.contains("if [ -f test"));
    assert!(formatted.contains("^"));
}

/// Test: REPL-015-003-006 - Error severity formatting
#[test]
fn test_REPL_015_003_006_severity_formatting() {
    let error = ErrorMessage {
        error_type: ErrorType::Lint,
        code: Some("PERF001".to_string()),
        severity: Severity::Warning,
        message: "Inefficient code".to_string(),
        context: None,
        explanation: None,
        suggestion: None,
        help_topics: vec![],
    };

    let formatted = format_error(&error);
    assert!(formatted.contains("⚠")); // Warning icon
    assert!(formatted.contains("Warning"));
}
