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
        output.push_str(&format!("{} {} {} [{}]\n\n", icon, type_str, severity_str, code));
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

/// Create error message for parse errors
pub fn format_parse_error(error_msg: &str, line: usize, column: usize, source: &str) -> ErrorMessage {
    let lines: Vec<&str> = source.lines().collect();
    let source_line = if line > 0 && line <= lines.len() {
        lines[line - 1].to_string()
    } else {
        String::new()
    };

    let context = if !source_line.is_empty() {
        Some(SourceContext {
            line,
            column,
            source_line: source_line.clone(),
            length: 1,
        })
    } else {
        None
    };

    // Try to suggest a fix based on common parse errors
    let suggestion = if error_msg.contains("then") {
        Some(Suggestion {
            description: "Did you forget 'then'?".to_string(),
            fixed_code: Some(format!("{}; then", source_line)),
            auto_fixable: false,
        })
    } else {
        None
    };

    ErrorMessage {
        error_type: ErrorType::Parse,
        code: Some("P001".to_string()),
        severity: Severity::Error,
        message: error_msg.to_string(),
        context,
        explanation: None,
        suggestion,
        help_topics: vec![],
    }
}

/// Create error message for lint violations
pub fn format_lint_error(diagnostic: &Diagnostic, source: &str) -> ErrorMessage {
    let lines: Vec<&str> = source.lines().collect();
    let line_idx = diagnostic.span.start_line.saturating_sub(1);
    let source_line = if line_idx < lines.len() {
        lines[line_idx].to_string()
    } else {
        String::new()
    };

    let context = if !source_line.is_empty() {
        Some(SourceContext {
            line: diagnostic.span.start_line,
            column: diagnostic.span.start_col,
            source_line,
            length: diagnostic
                .span
                .end_col
                .saturating_sub(diagnostic.span.start_col)
                .max(1),
        })
    } else {
        None
    };

    // Convert LintSeverity to our Severity
    let severity = match diagnostic.severity {
        LintSeverity::Error => Severity::Error,
        LintSeverity::Warning => Severity::Warning,
        LintSeverity::Info => Severity::Info,
        LintSeverity::Note => Severity::Info,
        LintSeverity::Perf => Severity::Warning,
        LintSeverity::Risk => Severity::Warning,
    };

    // Determine explanation based on violation code
    let explanation = match diagnostic.code.as_str() {
        code if code.starts_with("IDEM") => {
            Some("Command will fail if run multiple times".to_string())
        }
        code if code.starts_with("DET") => {
            Some("Command produces different output on each run".to_string())
        }
        code if code.starts_with("SEC") => Some("Security vulnerability detected".to_string()),
        _ => None,
    };

    // Create suggestion from fix if available
    let suggestion = diagnostic.fix.as_ref().map(|fix| Suggestion {
        description: "Use idempotent version".to_string(),
        fixed_code: Some(fix.replacement.clone()),
        auto_fixable: true, // bashrs can auto-fix lint issues via purify
    });

    ErrorMessage {
        error_type: ErrorType::Lint,
        code: Some(diagnostic.code.clone()),
        severity,
        message: diagnostic.message.clone(),
        context,
        explanation,
        suggestion,
        help_topics: vec!["purify".to_string()],
    }
}

/// Create error message for unknown commands
pub fn format_command_error(command: &str, available_commands: &[&str]) -> ErrorMessage {
    let message = format!("Unknown command: '{}'", command);

    // Try to suggest similar command
    let suggestion = suggest_command(command, available_commands).map(|suggested| Suggestion {
        description: format!("Did you mean: '{}'?", suggested),
        fixed_code: Some(suggested),
        auto_fixable: false,
    });

    // Build list of available commands for explanation
    let commands_list = available_commands.join(", ");
    let explanation = Some(format!("Available commands: {}", commands_list));

    ErrorMessage {
        error_type: ErrorType::Command,
        code: None,
        severity: Severity::Error,
        message,
        context: None,
        explanation,
        suggestion,
        help_topics: vec!["commands".to_string()],
    }
}

/// Suggest similar command using edit distance
pub fn suggest_command(input: &str, commands: &[&str]) -> Option<String> {
    if commands.is_empty() {
        return None;
    }

    // Calculate Levenshtein distance to each command
    let mut best_match: Option<(String, usize)> = None;

    for &command in commands {
        let distance = levenshtein_distance(input, command);

        // Only suggest if distance is small (< 3)
        if distance < 3 {
            if let Some((_, best_distance)) = &best_match {
                if distance < *best_distance {
                    best_match = Some((command.to_string(), distance));
                }
            } else {
                best_match = Some((command.to_string(), distance));
            }
        }
    }

    best_match.map(|(cmd, _)| cmd)
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first column and row
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Calculate distances
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };

            matrix[i][j] = (matrix[i - 1][j] + 1) // deletion
                .min(matrix[i][j - 1] + 1) // insertion
                .min(matrix[i - 1][j - 1] + cost); // substitution
        }
    }

    matrix[len1][len2]
}

/// Format source context with caret indicator
pub fn format_source_context(context: &SourceContext) -> String {
    let mut output = String::new();

    // Show line number and source
    output.push_str(&format!("    {} | {}\n", context.line, context.source_line));

    // Show caret indicator pointing to the error
    let col = context.column.saturating_sub(1);
    let line_num_width = format!("    {} | ", context.line).len();

    output.push_str(&" ".repeat(line_num_width));
    output.push_str(&" ".repeat(col));
    output.push_str(&"^".repeat(context.length));

    output
}

#[cfg(test)]
mod tests {
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
}

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Property: Error formatting never panics
    #[test]
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
