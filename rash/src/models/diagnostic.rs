// Rich error diagnostics for Rash transpiler
// Testing Spec Section 1.6: Error message quality ≥0.7
//
// Provides structured, helpful error messages with:
// - Source location (file:line:column)
// - Error category and explanation
// - Helpful suggestions
// - Related information
// - ANSI-colored rustc-style output

use crate::models::Error;
use std::fmt;

/// Enhanced diagnostic information for errors
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The underlying error
    pub error: String,

    /// Source file path (if available)
    pub file: Option<String>,

    /// Line number (if available)
    pub line: Option<usize>,

    /// Column number (if available)
    pub column: Option<usize>,

    /// Error category (for grouping similar errors)
    pub category: ErrorCategory,

    /// Additional context/explanation
    pub note: Option<String>,

    /// Suggested fix or workaround
    pub help: Option<String>,

    /// Code snippet (if available)
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Syntax/parse errors
    Syntax,

    /// Unsupported Rust features
    UnsupportedFeature,

    /// Validation errors
    Validation,

    /// IR generation errors
    Transpilation,

    /// I/O errors
    Io,

    /// Internal compiler errors
    Internal,
}

impl ErrorCategory {
    /// Short tag for the category, used in `error[tag]:` prefix
    fn tag(&self) -> &'static str {
        match self {
            Self::Syntax => "syntax",
            Self::UnsupportedFeature => "unsupported",
            Self::Validation => "validation",
            Self::Transpilation => "transpile",
            Self::Io => "io",
            Self::Internal => "internal",
        }
    }
}

impl Diagnostic {
    /// Create a diagnostic from an error with context.
    /// If the error is `WithContext`, unwraps it to extract file/source.
    pub fn from_error(error: &Error, file: Option<String>) -> Self {
        match error {
            Error::WithContext {
                inner,
                file: ctx_file,
                source_code,
            } => {
                let effective_file = ctx_file.clone().or(file);
                Self::from_error_with_source(inner, effective_file, source_code.as_deref())
            }
            _ => Self::from_error_with_source(error, file, None),
        }
    }

    /// Create a diagnostic from an error with optional source code for snippet extraction
    pub fn from_error_with_source(
        error: &Error,
        file: Option<String>,
        source_code: Option<&str>,
    ) -> Self {
        let (category, note, help) = Self::categorize_error(error);
        let (line, column) = Self::extract_location(error);
        let snippet = match (source_code, line) {
            (Some(src), Some(ln)) => Some(Self::extract_snippet(src, ln, column)),
            _ => None,
        };

        Self {
            error: Self::extract_message(error),
            file,
            line,
            column,
            category,
            note,
            help,
            snippet,
        }
    }

    /// Extract line/column from syn::Error span (requires proc-macro2 span-locations)
    fn extract_location(error: &Error) -> (Option<usize>, Option<usize>) {
        if let Error::Parse(syn_err) = error {
            let span = syn_err.span();
            let start = span.start();
            // proc-macro2 with span-locations returns real values;
            // without it, line=0, column=0
            if start.line > 0 {
                return (Some(start.line), Some(start.column));
            }
        }
        (None, None)
    }

    /// Extract a cleaner error message from the error, stripping syn's location prefix
    fn extract_message(error: &Error) -> String {
        let msg = error.to_string();
        // syn::Error formats as "cannot parse..." — strip the "Parse error: " prefix
        // from our thiserror wrapper so we don't double-prefix
        if let Error::Parse(_) = error {
            if let Some(stripped) = msg.strip_prefix("Parse error: ") {
                return stripped.to_string();
            }
        }
        msg
    }

    /// Build a 3-line source snippet with line numbers, gutter, and caret
    fn extract_snippet(source: &str, line: usize, column: Option<usize>) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let line_idx = line.saturating_sub(1); // 0-indexed
        let mut result = String::new();

        // Calculate gutter width (widest line number)
        let max_line = (line + 1).min(lines.len());
        let gutter_width = max_line.to_string().len();

        // Line before (if exists)
        if line_idx > 0 {
            let prev = line_idx - 1;
            result.push_str(&format!(
                " {:>width$} | {}\n",
                prev + 1,
                lines[prev],
                width = gutter_width
            ));
        }

        // Error line
        if line_idx < lines.len() {
            result.push_str(&format!(
                " {:>width$} | {}\n",
                line,
                lines[line_idx],
                width = gutter_width
            ));

            // Caret line
            if let Some(col) = column {
                result.push_str(&format!(
                    " {:>width$} | {}^\n",
                    "",
                    " ".repeat(col),
                    width = gutter_width
                ));
            }
        }

        // Line after (if exists)
        if line_idx + 1 < lines.len() {
            result.push_str(&format!(
                " {:>width$} | {}\n",
                line + 1,
                lines[line_idx + 1],
                width = gutter_width
            ));
        }

        result
    }

    /// Categorize error and provide helpful context
    fn categorize_error(error: &Error) -> (ErrorCategory, Option<String>, Option<String>) {
        match error {
            Error::Parse(syn_err) => Self::categorize_parse_error(syn_err),

            Error::Validation(msg) if msg.contains("Only functions") => (
                ErrorCategory::UnsupportedFeature,
                Some("Rash only supports function definitions at the top level.".to_string()),
                Some("Remove struct, trait, impl, or other definitions. Only 'fn' declarations are allowed.".to_string()),
            ),

            Error::Validation(msg) if msg.contains("Unsupported macro") => (
                ErrorCategory::UnsupportedFeature,
                Some("This macro is not supported for shell transpilation.".to_string()),
                Some("Use println!() for output. Only println!, eprintln!, and format! macros are supported.".to_string()),
            ),

            Error::Validation(msg) if msg.contains("No main") || msg.contains("no main") => (
                ErrorCategory::Validation,
                Some("A main() function is required as the entry point.".to_string()),
                Some("Add `fn main() { ... }` to your source file.".to_string()),
            ),

            Error::Validation(msg) if msg.contains("must have initializer") => (
                ErrorCategory::Validation,
                Some("Shell variables must be initialized at declaration.".to_string()),
                Some("Add an initial value: `let x = \"\";` or `let x = 0;`".to_string()),
            ),

            Error::Validation(msg) if msg.contains("Unsupported expression") => (
                ErrorCategory::UnsupportedFeature,
                Some("This expression type cannot be transpiled to shell.".to_string()),
                Some("Simplify the expression. Rash supports literals, variables, function calls, if/else, and loops.".to_string()),
            ),

            Error::Validation(msg) if msg.contains("Unsupported") => (
                ErrorCategory::UnsupportedFeature,
                Some("This Rust feature is not supported for shell script transpilation.".to_string()),
                Some("Check the user guide for supported features, or file an issue for feature requests.".to_string()),
            ),

            Error::Validation(msg) => (
                ErrorCategory::Validation,
                Some(format!("Validation failed: {msg}")),
                Some("Review the error message and ensure your code follows Rash constraints.".to_string()),
            ),

            Error::IrGeneration(msg) => (
                ErrorCategory::Transpilation,
                Some(format!("Failed to generate intermediate representation: {msg}")),
                Some("This is likely a transpiler bug. Please report this issue.".to_string()),
            ),

            Error::Io(io_err) => Self::categorize_io_error(io_err),

            Error::Unsupported(feature) => (
                ErrorCategory::UnsupportedFeature,
                Some(format!("The feature '{feature}' is not yet supported for transpilation.")),
                Some("See docs/user-guide.md for supported features, or use a workaround.".to_string()),
            ),

            Error::WithContext { inner, .. } => Self::categorize_error(inner),

            _ => (
                ErrorCategory::Internal,
                Some("An internal error occurred during transpilation.".to_string()),
                Some("This may be a bug. Please report this with a minimal reproduction.".to_string()),
            ),
        }
    }

    /// Specific categorization for parse errors, extracting detail from syn::Error message
    fn categorize_parse_error(
        syn_err: &syn::Error,
    ) -> (ErrorCategory, Option<String>, Option<String>) {
        let msg = syn_err.to_string();

        // Check "unexpected" patterns first (before "expected", since "unexpected" contains "expected")
        if msg.contains("unexpected eof") || msg.contains("unexpected end") {
            (
                ErrorCategory::Syntax,
                Some("The file ended unexpectedly.".to_string()),
                Some(
                    "Check for missing closing braces `}`, parentheses `)`, or semicolons `;`."
                        .to_string(),
                ),
            )
        } else if msg.contains("unexpected token") {
            (
                ErrorCategory::Syntax,
                Some("An unexpected token was encountered.".to_string()),
                Some(
                    "Check for typos or unsupported syntax near the indicated location.".to_string(),
                ),
            )
        } else if msg.contains("expected") && msg.contains("found") {
            // "expected `;`, found `let`" style
            (
                ErrorCategory::Syntax,
                Some("The Rust parser found unexpected syntax.".to_string()),
                Some(format!("Fix the syntax error: {msg}")),
            )
        } else if msg.contains("expected") {
            // "expected `;`" style
            let help = if msg.contains(';') {
                "Add a semicolon `;` at the end of the statement.".to_string()
            } else {
                format!("Fix the syntax error: {msg}")
            };
            (
                ErrorCategory::Syntax,
                Some("The Rust parser found unexpected syntax.".to_string()),
                Some(help),
            )
        } else {
            (
                ErrorCategory::Syntax,
                Some("The Rust parser could not parse this code.".to_string()),
                Some("Ensure your code is valid Rust syntax. Rash supports a subset of Rust.".to_string()),
            )
        }
    }

    /// Specific categorization for I/O errors
    fn categorize_io_error(
        io_err: &std::io::Error,
    ) -> (ErrorCategory, Option<String>, Option<String>) {
        match io_err.kind() {
            std::io::ErrorKind::NotFound => (
                ErrorCategory::Io,
                Some("The specified file was not found.".to_string()),
                Some("Check that the file path is correct and the file exists.".to_string()),
            ),
            std::io::ErrorKind::PermissionDenied => (
                ErrorCategory::Io,
                Some("Permission denied when accessing the file.".to_string()),
                Some("Check file permissions. You may need to use chmod or run with appropriate privileges.".to_string()),
            ),
            _ => (
                ErrorCategory::Io,
                Some("Failed to read or write files.".to_string()),
                Some("Check file paths and permissions.".to_string()),
            ),
        }
    }

    /// Calculate quality score (0.0 to 1.0)
    pub fn quality_score(&self) -> f32 {
        let mut score = 0.0;

        // Has error prefix (always present)
        score += 1.0;

        // Has source location (file is most important)
        if self.file.is_some() {
            score += 1.0;
        }
        if self.line.is_some() {
            score += 0.25;
        }
        if self.column.is_some() {
            score += 0.25;
        }

        // Has code snippet (nice to have but not always possible)
        if self.snippet.is_some() {
            score += 1.0;
        }

        // Has explanation (note) - CRITICAL for user understanding
        if self.note.is_some() {
            score += 2.5;
        }

        // Has suggestion (help) - CRITICAL for actionability
        if self.help.is_some() {
            score += 2.5;
        }

        score / 8.5 // Normalize to 0-1 (max 8.5 points)
    }
}

/// Check if stderr is a terminal (for ANSI color support)
fn use_color() -> bool {
    std::env::var("NO_COLOR").is_err()
        && std::env::var("TERM").map_or(true, |t| t != "dumb")
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let color = use_color();

        // ANSI codes
        let red = if color { "\x1b[1;31m" } else { "" };
        let blue = if color { "\x1b[1;34m" } else { "" };
        let cyan = if color { "\x1b[1;36m" } else { "" };
        let green = if color { "\x1b[1;32m" } else { "" };
        let bold = if color { "\x1b[1m" } else { "" };
        let reset = if color { "\x1b[0m" } else { "" };

        // Error header: error[tag]: message
        let tag = self.category.tag();
        write!(f, "{red}error[{tag}]{reset}: {bold}{}{reset}", self.error)?;
        writeln!(f)?;

        // Location: --> file:line:col
        if let Some(file) = &self.file {
            write!(f, " {blue}-->{reset} {file}")?;
            if let Some(line) = self.line {
                write!(f, ":{line}")?;
                if let Some(col) = self.column {
                    write!(f, ":{col}")?;
                }
            }
            writeln!(f)?;
        }

        // Code snippet with gutter
        if let Some(snippet) = &self.snippet {
            // The snippet already has line numbers and pipe chars built in.
            // We need to colorize the gutter pipes blue and the caret red.
            for line in snippet.lines() {
                if line.contains('^') {
                    // Caret line: colorize the caret red
                    let (before_caret, from_caret) =
                        line.split_at(line.find('^').unwrap_or(line.len()));
                    // Colorize gutter pipe blue
                    if let Some(pipe_pos) = before_caret.find('|') {
                        let (gutter, rest) = before_caret.split_at(pipe_pos + 1);
                        writeln!(f, "{blue}{gutter}{reset}{rest}{red}{from_caret}{reset}")?;
                    } else {
                        writeln!(f, "{before_caret}{red}{from_caret}{reset}")?;
                    }
                } else if let Some(pipe_pos) = line.find('|') {
                    // Source line: colorize gutter pipe blue
                    let (gutter, code) = line.split_at(pipe_pos + 1);
                    writeln!(f, "{blue}{gutter}{reset}{code}")?;
                } else {
                    writeln!(f, "{line}")?;
                }
            }
        }

        // Note (explanation)
        if let Some(note) = &self.note {
            writeln!(f, "  {cyan}note{reset}: {note}")?;
        }

        // Help (suggestion)
        if let Some(help) = &self.help {
            writeln!(f, "  {green}help{reset}: {help}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Helper to format a diagnostic with colors disabled
    fn format_no_color(diag: &Diagnostic) -> String {
        // SAFETY: Only called from serial tests
        unsafe { std::env::set_var("NO_COLOR", "1") };
        let result = format!("{diag}");
        unsafe { std::env::remove_var("NO_COLOR") };
        result
    }

    #[test]
    fn test_diagnostic_quality_score() {
        let mut diag = Diagnostic {
            error: "test error".to_string(),
            file: None,
            line: None,
            column: None,
            category: ErrorCategory::Syntax,
            note: None,
            help: None,
            snippet: None,
        };

        // Baseline: just error prefix
        assert!(diag.quality_score() < 0.7); // Only error prefix, no context

        // Add location
        diag.file = Some("test.rs".to_string());
        diag.line = Some(10);
        diag.column = Some(5);
        assert!(diag.quality_score() < 0.7); // Missing note+help, below threshold

        // Add note and help (target ≥0.7)
        diag.note = Some("Explanation".to_string());
        diag.help = Some("Suggestion".to_string());
        assert!(diag.quality_score() >= 0.7); // Should exceed 0.7 threshold
    }

    #[test]
    fn test_unsupported_feature_diagnostic() {
        let error = Error::Validation("Only functions are allowed in Rash code".to_string());
        let diag = Diagnostic::from_error(&error, Some("example.rs".to_string()));

        assert_eq!(diag.category, ErrorCategory::UnsupportedFeature);
        assert!(diag.note.is_some());
        assert!(diag.help.is_some());

        // Should achieve ≥0.7 quality score
        assert!(
            diag.quality_score() >= 0.7,
            "Quality score {} should be ≥0.7",
            diag.quality_score()
        );
    }

    #[test]
    fn test_diagnostic_display() {
        let diag = Diagnostic {
            error: "unexpected token".to_string(),
            file: Some("main.rs".to_string()),
            line: Some(5),
            column: Some(10),
            category: ErrorCategory::Syntax,
            note: Some("Expected a semicolon here".to_string()),
            help: Some("Add ';' after the statement".to_string()),
            snippet: None,
        };

        let output = format_no_color(&diag);
        assert!(output.contains("error[syntax]"), "got: {output}");
        assert!(output.contains("--> main.rs:5:10"), "got: {output}");
        assert!(output.contains("note: Expected a semicolon"), "got: {output}");
        assert!(output.contains("help: Add ';'"), "got: {output}");
    }

    // ====== Additional Tests for Coverage ======

    #[test]
    fn test_diagnostic_display_no_file() {
        let diag = Diagnostic {
            error: "parse error".to_string(),
            file: None,
            line: None,
            column: None,
            category: ErrorCategory::Syntax,
            note: None,
            help: None,
            snippet: None,
        };

        let output = format_no_color(&diag);
        assert!(output.contains("error[syntax]: parse error"), "got: {output}");
        assert!(!output.contains("-->"), "got: {output}");
    }

    #[test]
    fn test_diagnostic_display_file_only() {
        let diag = Diagnostic {
            error: "file error".to_string(),
            file: Some("test.rs".to_string()),
            line: None,
            column: None,
            category: ErrorCategory::Io,
            note: None,
            help: None,
            snippet: None,
        };

        let output = format_no_color(&diag);
        assert!(output.contains("--> test.rs"), "got: {output}");
        assert!(!output.contains(":0"), "No line number: {output}");
    }

    #[test]
    fn test_diagnostic_display_file_and_line() {
        let diag = Diagnostic {
            error: "line error".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(42),
            column: None,
            category: ErrorCategory::Validation,
            note: None,
            help: None,
            snippet: None,
        };

        let output = format_no_color(&diag);
        assert!(output.contains("--> test.rs:42"), "got: {output}");
    }

    #[test]
    fn test_diagnostic_display_with_snippet() {
        let diag = Diagnostic {
            error: "syntax error".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(5),
            column: Some(10),
            category: ErrorCategory::Syntax,
            note: None,
            help: None,
            snippet: Some(" 5 | let x = foo(\n   |           ^\n".to_string()),
        };

        let output = format_no_color(&diag);
        assert!(output.contains("let x = foo("), "got: {output}");
        assert!(output.contains("^"), "got: {output}");
    }

    #[test]
    fn test_diagnostic_display_snippet_column_0() {
        let diag = Diagnostic {
            error: "syntax error".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(5),
            column: Some(0),
            category: ErrorCategory::Syntax,
            note: None,
            help: None,
            snippet: Some(" 5 | bad code\n   | ^\n".to_string()),
        };

        let output = format_no_color(&diag);
        assert!(output.contains("bad code"), "got: {output}");
        assert!(output.contains("^"), "got: {output}");
    }

    #[test]
    fn test_quality_score_with_snippet() {
        let diag = Diagnostic {
            error: "test error".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(10),
            column: Some(5),
            category: ErrorCategory::Syntax,
            note: Some("Explanation".to_string()),
            help: Some("Suggestion".to_string()),
            snippet: Some("let x = bad;".to_string()),
        };

        // With snippet, score should be higher
        let score = diag.quality_score();
        assert!(
            score > 0.9,
            "Score with snippet should be >0.9, got {score}",
        );
    }

    #[test]
    fn test_categorize_parse_error() {
        let error = Error::Parse(syn::Error::new(proc_macro2::Span::call_site(), "test"));
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Syntax);
        assert!(diag.note.is_some());
        assert!(diag.help.is_some());
    }

    #[test]
    fn test_categorize_parse_error_expected_found() {
        let error = Error::Parse(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected `;`, found `let`",
        ));
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Syntax);
        assert!(
            diag.help.as_ref().unwrap().contains("expected"),
            "help: {:?}",
            diag.help
        );
    }

    #[test]
    fn test_categorize_parse_error_expected_semicolon() {
        let error = Error::Parse(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected `;`",
        ));
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Syntax);
        assert!(
            diag.help.as_ref().unwrap().contains("semicolon"),
            "help: {:?}",
            diag.help
        );
    }

    #[test]
    fn test_categorize_parse_error_unexpected_eof() {
        let error = Error::Parse(syn::Error::new(
            proc_macro2::Span::call_site(),
            "unexpected eof",
        ));
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Syntax);
        assert!(diag.note.as_ref().unwrap().contains("ended unexpectedly"));
    }

    #[test]
    fn test_categorize_parse_error_unexpected_token() {
        let error = Error::Parse(syn::Error::new(
            proc_macro2::Span::call_site(),
            "unexpected token after this expression",
        ));
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Syntax);
        assert!(diag.note.as_ref().unwrap().contains("unexpected token"));
    }

    #[test]
    fn test_categorize_validation_unsupported() {
        let error = Error::Validation("Unsupported expression type".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::UnsupportedFeature);
    }

    #[test]
    fn test_categorize_validation_unsupported_macro() {
        let error = Error::Validation("Unsupported macro: vec!".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::UnsupportedFeature);
        assert!(diag.help.as_ref().unwrap().contains("println!"));
    }

    #[test]
    fn test_categorize_validation_no_main() {
        let error = Error::Validation("No main function found".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Validation);
        assert!(diag.help.as_ref().unwrap().contains("fn main()"));
    }

    #[test]
    fn test_categorize_validation_must_have_initializer() {
        let error = Error::Validation("Variables must have initializers".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Validation);
        assert!(diag.help.as_ref().unwrap().contains("initial value"));
    }

    #[test]
    fn test_categorize_validation_generic() {
        let error = Error::Validation("Some validation issue".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Validation);
    }

    #[test]
    fn test_categorize_ir_generation() {
        let error = Error::IrGeneration("Failed to generate IR".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Transpilation);
        assert!(diag
            .note
            .as_ref()
            .unwrap()
            .contains("intermediate representation"));
    }

    #[test]
    fn test_categorize_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = Error::Io(io_err);
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Io);
        assert!(diag.help.as_ref().unwrap().contains("exists"));
    }

    #[test]
    fn test_categorize_io_error_permission_denied() {
        let io_err =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let error = Error::Io(io_err);
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Io);
        assert!(diag.help.as_ref().unwrap().contains("chmod"));
    }

    #[test]
    fn test_categorize_unsupported() {
        let error = Error::Unsupported("async functions".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::UnsupportedFeature);
        assert!(diag.note.as_ref().unwrap().contains("async functions"));
    }

    #[test]
    fn test_categorize_internal_error() {
        let error = Error::Internal("unexpected state".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::Internal);
        assert!(diag.help.as_ref().unwrap().contains("bug"));
    }

    #[test]
    fn test_error_category_equality() {
        assert_eq!(ErrorCategory::Syntax, ErrorCategory::Syntax);
        assert_ne!(ErrorCategory::Syntax, ErrorCategory::Io);
        assert_eq!(
            ErrorCategory::UnsupportedFeature,
            ErrorCategory::UnsupportedFeature
        );
        assert_eq!(ErrorCategory::Validation, ErrorCategory::Validation);
        assert_eq!(ErrorCategory::Transpilation, ErrorCategory::Transpilation);
        assert_eq!(ErrorCategory::Internal, ErrorCategory::Internal);
    }

    #[test]
    fn test_diagnostic_clone() {
        let diag = Diagnostic {
            error: "test".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(1),
            column: Some(1),
            category: ErrorCategory::Syntax,
            note: Some("note".to_string()),
            help: Some("help".to_string()),
            snippet: Some("code".to_string()),
        };

        let cloned = diag.clone();
        assert_eq!(diag.error, cloned.error);
        assert_eq!(diag.file, cloned.file);
        assert_eq!(diag.category, cloned.category);
    }

    #[test]
    fn test_error_category_debug() {
        let cat = ErrorCategory::Syntax;
        let debug_str = format!("{cat:?}");
        assert_eq!(debug_str, "Syntax");
    }

    #[test]
    fn test_diagnostic_debug() {
        let diag = Diagnostic {
            error: "test".to_string(),
            file: None,
            line: None,
            column: None,
            category: ErrorCategory::Syntax,
            note: None,
            help: None,
            snippet: None,
        };

        let debug_str = format!("{diag:?}");
        assert!(debug_str.contains("Diagnostic"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_extract_snippet_middle_of_file() {
        let source = "line 1\nline 2\nline 3\nline 4\nline 5";
        let snippet = Diagnostic::extract_snippet(source, 3, Some(2));
        assert!(snippet.contains("line 2"), "snippet: {snippet}");
        assert!(snippet.contains("line 3"), "snippet: {snippet}");
        assert!(snippet.contains("line 4"), "snippet: {snippet}");
        assert!(snippet.contains("^"), "snippet: {snippet}");
    }

    #[test]
    fn test_extract_snippet_first_line() {
        let source = "first line\nsecond line\nthird line";
        let snippet = Diagnostic::extract_snippet(source, 1, Some(5));
        assert!(snippet.contains("first line"), "snippet: {snippet}");
        assert!(snippet.contains("second line"), "snippet: {snippet}");
        assert!(snippet.contains("^"), "snippet: {snippet}");
    }

    #[test]
    fn test_extract_snippet_last_line() {
        let source = "line 1\nline 2\nlast line";
        let snippet = Diagnostic::extract_snippet(source, 3, Some(0));
        assert!(snippet.contains("line 2"), "snippet: {snippet}");
        assert!(snippet.contains("last line"), "snippet: {snippet}");
    }

    #[test]
    fn test_extract_snippet_no_column() {
        let source = "line 1\nline 2\nline 3";
        let snippet = Diagnostic::extract_snippet(source, 2, None);
        assert!(snippet.contains("line 2"), "snippet: {snippet}");
        assert!(!snippet.contains("^"), "no caret without column: {snippet}");
    }

    #[test]
    fn test_from_error_with_source() {
        let source = "fn main() {\n    let x = 42\n    let y = 10;\n}";
        let error = Error::Parse(syn::Error::new(proc_macro2::Span::call_site(), "expected `;`"));
        let diag = Diagnostic::from_error_with_source(
            &error,
            Some("test.rs".to_string()),
            Some(source),
        );

        assert_eq!(diag.category, ErrorCategory::Syntax);
        assert_eq!(diag.file, Some("test.rs".to_string()));
        assert!(diag.help.as_ref().unwrap().contains("semicolon"));
    }

    #[test]
    fn test_from_error_unwraps_with_context() {
        let inner = Error::Parse(syn::Error::new(proc_macro2::Span::call_site(), "test error"));
        let error = Error::WithContext {
            inner: Box::new(inner),
            file: Some("ctx.rs".to_string()),
            source_code: Some("fn main() {}".to_string()),
        };

        let diag = Diagnostic::from_error(&error, None);
        assert_eq!(diag.file, Some("ctx.rs".to_string()));
        assert_eq!(diag.category, ErrorCategory::Syntax);
    }

    #[test]
    fn test_from_error_with_context_prefers_context_file() {
        let inner = Error::Validation("test".to_string());
        let error = Error::WithContext {
            inner: Box::new(inner),
            file: Some("from_context.rs".to_string()),
            source_code: None,
        };

        // Even if we pass a file to from_error, WithContext's file takes precedence
        let diag = Diagnostic::from_error(&error, Some("from_caller.rs".to_string()));
        assert_eq!(diag.file, Some("from_context.rs".to_string()));
    }

    #[test]
    fn test_from_error_with_context_falls_back_to_caller_file() {
        let inner = Error::Validation("test".to_string());
        let error = Error::WithContext {
            inner: Box::new(inner),
            file: None,
            source_code: None,
        };

        let diag = Diagnostic::from_error(&error, Some("fallback.rs".to_string()));
        assert_eq!(diag.file, Some("fallback.rs".to_string()));
    }

    #[test]
    fn test_category_tags() {
        assert_eq!(ErrorCategory::Syntax.tag(), "syntax");
        assert_eq!(ErrorCategory::UnsupportedFeature.tag(), "unsupported");
        assert_eq!(ErrorCategory::Validation.tag(), "validation");
        assert_eq!(ErrorCategory::Transpilation.tag(), "transpile");
        assert_eq!(ErrorCategory::Io.tag(), "io");
        assert_eq!(ErrorCategory::Internal.tag(), "internal");
    }

    #[test]
    fn test_extract_message_strips_parse_prefix() {
        let error = Error::Parse(syn::Error::new(
            proc_macro2::Span::call_site(),
            "cannot parse",
        ));
        let msg = Diagnostic::extract_message(&error);
        // Should strip "Parse error: " prefix
        assert_eq!(msg, "cannot parse");
    }

    #[test]
    fn test_extract_message_keeps_validation_prefix() {
        let error = Error::Validation("something wrong".to_string());
        let msg = Diagnostic::extract_message(&error);
        assert_eq!(msg, "AST validation error: something wrong");
    }
}
