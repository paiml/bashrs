pub fn format_parse_error(
    error_msg: &str,
    line: usize,
    column: usize,
    source: &str,
) -> ErrorMessage {
    let lines: Vec<&str> = source.lines().collect();
    let source_line = if line > 0 && line <= lines.len() {
        lines
            .get(line - 1)
            .map(|s| s.to_string())
            .unwrap_or_default()
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

/// Creates an error message for lint violations (code quality issues).
///
/// Converts a linter diagnostic into a user-friendly error message with
/// explanations and auto-fix suggestions.
///
/// # Arguments
///
/// * `diagnostic` - The linter diagnostic
/// * `source` - The source code being linted
///
/// # Returns
///
/// Structured `ErrorMessage` with lint code (DET*, IDEM*, SEC*)
///
/// # Examples
///
/// ```
/// use bashrs::repl::errors::format_lint_error;
/// use bashrs::linter::{Diagnostic, Severity, Span, Fix};
///
/// let diagnostic = Diagnostic {
///     code: "IDEM001".to_string(),
///     severity: Severity::Error,
///     message: "mkdir without -p".to_string(),
///     span: Span::new(1, 1, 1, 11),
///     fix: Some(Fix::new("mkdir -p /tmp")),
/// };
///
/// let error = format_lint_error(&diagnostic, "mkdir /tmp");
///
/// assert_eq!(error.code, Some("IDEM001".to_string()));
/// assert!(error.suggestion.is_some());
/// ```
pub fn format_lint_error(diagnostic: &Diagnostic, source: &str) -> ErrorMessage {
    let lines: Vec<&str> = source.lines().collect();
    let line_idx = diagnostic.span.start_line.saturating_sub(1);
    let source_line = if line_idx < lines.len() {
        lines
            .get(line_idx)
            .map(|s| s.to_string())
            .unwrap_or_default()
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

/// Creates an error message for unknown REPL commands.
///
/// Uses edit distance to suggest similar commands and lists all available commands.
///
/// # Arguments
///
/// * `command` - The unknown command entered by the user
/// * `available_commands` - List of valid command names
///
/// # Returns
///
/// Structured `ErrorMessage` with command suggestion if similar command found
///
/// # Examples
///
/// ```
/// use bashrs::repl::errors::format_command_error;
///
/// let error = format_command_error(
///     "purfy",
///     &["purify", "lint", "quit", "help"]
/// );
///
/// assert!(error.message.contains("purfy"));
/// assert!(error.suggestion.is_some());
/// assert!(error.explanation.is_some());
/// ```
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
///
/// This is a standard dynamic programming algorithm with guaranteed safe indexing
/// because the matrix is pre-allocated to (len1+1) x (len2+1) dimensions.
#[allow(clippy::indexing_slicing, clippy::needless_range_loop)]
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
            let cost = usize::from(s1_chars[i - 1] != s2_chars[j - 1]);

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
#[path = "errors_tests_extracted.rs"]
mod tests_extracted;
