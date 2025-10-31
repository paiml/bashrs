// REPL-015-003: Better Error Messages
//
// Provides structured, actionable error messages with source context
//
// Quality gates:
// - EXTREME TDD: RED → GREEN → REFACTOR → PROPERTY → MUTATION
// - Unit tests: 6+ scenarios
// - Integration tests: REPL workflow
// - Property tests: 1+ generators
// - Complexity: <10 per function

use crate::linter::{Diagnostic, Severity as LintSeverity};

/// Structured error message
#[derive(Debug, Clone)]
pub struct ErrorMessage {
    /// Error type (Parse, Lint, Command, Runtime)
    pub error_type: ErrorType,

    /// Error code (optional, e.g., "DET001", "E001")
    pub code: Option<String>,

    /// Severity level
    pub severity: Severity,

    /// Main error message
    pub message: String,

    /// Source code context (line with error)
    pub context: Option<SourceContext>,

    /// Detailed explanation (optional)
    pub explanation: Option<String>,

    /// Suggested fix (optional)
    pub suggestion: Option<Suggestion>,

    /// Related help topics (optional)
    pub help_topics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    Parse,
    Lint,
    Command,
    Runtime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct SourceContext {
    /// Line number (1-indexed)
    pub line: usize,

    /// Column number (1-indexed)
    pub column: usize,

    /// Source code line
    pub source_line: String,

    /// Length of problematic section
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    /// Description of the fix
    pub description: String,

    /// Fixed code (if applicable)
    pub fixed_code: Option<String>,

    /// Auto-fix available flag
    pub auto_fixable: bool,
}

/// Format error message for display
pub fn format_error(_error: &ErrorMessage) -> String {
    unimplemented!("REPL-015-003: Not implemented")
}

/// Create error message for parse errors
pub fn format_parse_error(_error_msg: &str, _line: usize, _column: usize, _source: &str) -> ErrorMessage {
    unimplemented!("REPL-015-003: Not implemented")
}

/// Create error message for lint violations
pub fn format_lint_error(_diagnostic: &Diagnostic, _source: &str) -> ErrorMessage {
    unimplemented!("REPL-015-003: Not implemented")
}

/// Create error message for unknown commands
pub fn format_command_error(_command: &str, _available_commands: &[&str]) -> ErrorMessage {
    unimplemented!("REPL-015-003: Not implemented")
}

/// Suggest similar command using edit distance
pub fn suggest_command(_input: &str, _commands: &[&str]) -> Option<String> {
    unimplemented!("REPL-015-003: Not implemented")
}

/// Format source context with caret indicator
pub fn format_source_context(_context: &SourceContext) -> String {
    unimplemented!("REPL-015-003: Not implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::{Fix, Span};

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: REPL-015-003-001 - Format parse error
    #[test]
    #[should_panic(expected = "not implemented")]
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
    #[should_panic(expected = "not implemented")]
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
    #[should_panic(expected = "not implemented")]
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
    #[should_panic(expected = "not implemented")]
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
    #[should_panic(expected = "not implemented")]
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
    #[should_panic(expected = "not implemented")]
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
}

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Property: Error formatting never panics
    #[test]
    #[should_panic(expected = "not implemented")]
    fn prop_error_formatting_never_panics() {
        // Should never panic, even with invalid input
        let long_message = "a".repeat(1000);
        let test_cases = vec![
            ("", 1, 1),
            ("short", 1, 100), // column beyond line length
            (long_message.as_str(), 1000, 1000),
        ];

        for (message, _line, _column) in test_cases {
            let error = ErrorMessage {
                error_type: ErrorType::Runtime,
                code: None,
                severity: Severity::Error,
                message: message.to_string(),
                context: None,
                explanation: None,
                suggestion: None,
                help_topics: vec![],
            };

            // Should never panic
            let _ = format_error(&error);
        }
    }
}
