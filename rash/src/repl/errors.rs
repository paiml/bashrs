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

/// Structured error message with source context and actionable suggestions.
///
/// `ErrorMessage` provides rich, user-friendly error messages for REPL interactions.
/// Each error includes:
/// - **Type and severity**: Categorize the error for appropriate handling
/// - **Source context**: Show exactly where the error occurred
/// - **Explanation**: Describe why it's a problem
/// - **Suggestion**: Provide actionable fixes
/// - **Help topics**: Link to relevant documentation
///
/// # Examples
///
/// ## Parse error with source context
///
/// ```
/// use bashrs::repl::errors::{ErrorMessage, ErrorType, Severity, SourceContext};
///
/// let error = ErrorMessage {
///     error_type: ErrorType::Parse,
///     code: Some("P001".to_string()),
///     severity: Severity::Error,
///     message: "Expected 'then' after condition".to_string(),
///     context: Some(SourceContext {
///         line: 1,
///         column: 13,
///         source_line: "if [ -f file".to_string(),
///         length: 1,
///     }),
///     explanation: None,
///     suggestion: None,
///     help_topics: vec![],
/// };
///
/// assert_eq!(error.error_type, ErrorType::Parse);
/// ```
///
/// ## Format error for display
///
/// ```
/// use bashrs::repl::errors::{format_parse_error, format_error};
///
/// let error = format_parse_error("Expected 'then'", 1, 13, "if [ -f file");
/// let formatted = format_error(&error);
///
/// assert!(formatted.contains("Parse Error"));
/// assert!(formatted.contains("if [ -f file"));
/// ```
#[derive(Debug, Clone)]
pub struct ErrorMessage {
    /// Error type (Parse, Lint, Command, Runtime).
    pub error_type: ErrorType,

    /// Error code (e.g., "DET001", "P001").
    ///
    /// Codes enable users to search for specific errors in documentation.
    pub code: Option<String>,

    /// Severity level (Error, Warning, Info).
    pub severity: Severity,

    /// Main error message.
    ///
    /// Should be concise and describe what went wrong.
    pub message: String,

    /// Source code context showing where the error occurred.
    ///
    /// Includes line number, column, and the problematic source line.
    pub context: Option<SourceContext>,

    /// Detailed explanation of why this is a problem.
    ///
    /// Helps users understand the underlying issue.
    pub explanation: Option<String>,

    /// Suggested fix with optional corrected code.
    pub suggestion: Option<Suggestion>,

    /// Related help topics.
    ///
    /// Users can type `:help <topic>` for more information.
    pub help_topics: Vec<String>,
}

/// Error type categorization for REPL errors.
///
/// # Variants
///
/// - **Parse**: Syntax errors in bash scripts
/// - **Lint**: Code quality and safety violations
/// - **Command**: Unknown or invalid REPL commands
/// - **Runtime**: Execution failures
///
/// # Examples
///
/// ```
/// use bashrs::repl::errors::ErrorType;
///
/// let parse_error = ErrorType::Parse;
/// assert_eq!(parse_error, ErrorType::Parse);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    /// Parse error - syntax error in bash script.
    Parse,

    /// Lint error - code quality or safety violation.
    Lint,

    /// Command error - unknown or invalid REPL command.
    Command,

    /// Runtime error - execution failure.
    Runtime,
}

/// Severity level for errors and warnings.
///
/// # Examples
///
/// ```
/// use bashrs::repl::errors::Severity;
///
/// let error = Severity::Error;
/// let warning = Severity::Warning;
/// let info = Severity::Info;
///
/// assert_eq!(error, Severity::Error);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    /// Error - must be fixed.
    Error,

    /// Warning - should be fixed.
    Warning,

    /// Info - informational message.
    Info,
}

/// Source code context for error messages.
///
/// Provides the exact location and excerpt of problematic code.
///
/// # Examples
///
/// ```
/// use bashrs::repl::errors::SourceContext;
///
/// let context = SourceContext {
///     line: 5,
///     column: 10,
///     source_line: "mkdir /tmp/foo".to_string(),
///     length: 5,
/// };
///
/// assert_eq!(context.line, 5);
/// assert_eq!(context.column, 10);
/// ```
#[derive(Debug, Clone)]
pub struct SourceContext {
    /// Line number (1-indexed).
    pub line: usize,

    /// Column number (1-indexed).
    pub column: usize,

    /// Source code line containing the error.
    pub source_line: String,

    /// Length of the problematic section.
    ///
    /// Used to draw caret indicators (^^^) under the error.
    pub length: usize,
}

/// Suggested fix for an error.
///
/// Provides actionable guidance to resolve the issue.
///
/// # Examples
///
/// ```
/// use bashrs::repl::errors::Suggestion;
///
/// let suggestion = Suggestion {
///     description: "Use -p flag for idempotent directory creation".to_string(),
///     fixed_code: Some("mkdir -p /tmp/foo".to_string()),
///     auto_fixable: true,
/// };
///
/// assert!(suggestion.auto_fixable);
/// assert!(suggestion.fixed_code.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// Description of the suggested fix.
    pub description: String,

    /// Fixed code (if applicable).
    ///
    /// Shows the corrected version of the problematic code.
    pub fixed_code: Option<String>,

    /// Auto-fix available flag.
    ///
    /// If true, bashrs can automatically apply the fix via `:purify`.
    pub auto_fixable: bool,
}

/// Formats an error message for display in the terminal.
///
/// Produces a user-friendly, color-coded error message with:
/// - Icon and severity (✗, ⚠, ℹ)
/// - Error type and code
/// - Source context with caret indicators
/// - Explanation and suggestions
/// - Related help topics
///
/// # Arguments
///
/// * `error` - The error message to format
///
/// # Returns
///
/// Formatted string ready for terminal display
///
/// # Examples
///
/// ```
/// use bashrs::repl::errors::{ErrorMessage, ErrorType, Severity, format_error};
///
/// let error = ErrorMessage {
///     error_type: ErrorType::Command,
///     code: None,
///     severity: Severity::Error,
///     message: "Unknown command 'foo'".to_string(),
///     context: None,
///     explanation: Some("Command not recognized".to_string()),
///     suggestion: None,
///     help_topics: vec!["commands".to_string()],
/// };
///
/// let formatted = format_error(&error);
/// assert!(formatted.contains("Unknown command"));
/// assert!(formatted.contains(":help commands"));
/// ```
pub fn format_error(error: &ErrorMessage) -> String {
    let mut output = String::new();

    // Header with icon, severity, and type
    let icon = match error.severity {
        Severity::Error => "✗",
        Severity::Warning => "⚠",
        Severity::Info => "ℹ",
    };

    let severity_str = match error.severity {
        Severity::Error => "Error",
        Severity::Warning => "Warning",
        Severity::Info => "Info",
    };

    let type_str = match error.error_type {
        ErrorType::Parse => "Parse",
        ErrorType::Lint => "Lint",
        ErrorType::Command => "Command",
        ErrorType::Runtime => "Runtime",
    };

    // Format header
    if let Some(code) = &error.code {
        output.push_str(&format!(
            "{} {} {} [{}]\n\n",
            icon, type_str, severity_str, code
        ));
    } else {
        output.push_str(&format!("{} {} {}\n\n", icon, type_str, severity_str));
    }

    // Add source context if available
    if let Some(context) = &error.context {
        output.push_str(&format_source_context(context));
        output.push('\n');
    }

    // Add main message
    output.push_str(&format!("  {}\n", error.message));

    // Add explanation if available
    if let Some(explanation) = &error.explanation {
        output.push_str(&format!("\n  Problem: {}\n", explanation));
    }

    // Add suggestion if available
    if let Some(suggestion) = &error.suggestion {
        output.push_str(&format!("\n  Suggestion: {}\n", suggestion.description));

        if let Some(fixed) = &suggestion.fixed_code {
            output.push_str(&format!("  Try: {}\n", fixed));
        }

        if suggestion.auto_fixable {
            output.push_str("\n  Auto-fix available: Run ':purify' to see safe version\n");
        }
    }

    // Add help topics if available
    if !error.help_topics.is_empty() {
        output.push_str("\n  For more help, try:\n");
        for topic in &error.help_topics {
            output.push_str(&format!("    :help {}\n", topic));
        }
    }

    output
}

/// Creates an error message for parse errors (syntax errors).
///
/// Analyzes the parse error and constructs a helpful error message with
/// source context and common fix suggestions.
///
/// # Arguments
///
/// * `error_msg` - The parser's error message
/// * `line` - Line number where error occurred (1-indexed)
/// * `column` - Column number where error occurred (1-indexed)
/// * `source` - The source code being parsed
///
/// # Returns
///
/// Structured `ErrorMessage` with code P001
///
/// # Examples
///
/// ```
/// use bashrs::repl::errors::{format_parse_error, ErrorType};
///
/// let error = format_parse_error(
///     "Expected 'then'",
///     1,
///     13,
///     "if [ -f test"
/// );
///
/// assert_eq!(error.error_type, ErrorType::Parse);
/// assert_eq!(error.code, Some("P001".to_string()));
/// assert!(error.context.is_some());
/// ```

include!("errors_incl2.rs");
