// Rich error diagnostics for Rash transpiler
// Testing Spec Section 1.6: Error message quality ≥0.7
//
// Provides structured, helpful error messages with:
// - Source location (file:line:column)
// - Error category and explanation
// - Helpful suggestions
// - Related information

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

impl Diagnostic {
    /// Create a diagnostic from an error with context
    pub fn from_error(error: &Error, file: Option<String>) -> Self {
        let (category, note, help) = Self::categorize_error(error);

        // Note: syn::Error already includes location in its Display impl
        // We don't extract it separately as proc_macro2::Span doesn't expose line/column
        let (line, column) = (None, None);

        Self {
            error: error.to_string(),
            file,
            line,
            column,
            category,
            note,
            help,
            snippet: None,
        }
    }

    /// Categorize error and provide helpful context
    fn categorize_error(error: &Error) -> (ErrorCategory, Option<String>, Option<String>) {
        match error {
            Error::Parse(_) => (
                ErrorCategory::Syntax,
                Some("Rash uses a subset of Rust syntax for transpilation to shell scripts.".to_string()),
                Some("Ensure your code uses supported Rust syntax. See docs/user-guide.md for details.".to_string()),
            ),

            Error::Validation(msg) if msg.contains("Only functions") => (
                ErrorCategory::UnsupportedFeature,
                Some("Rash only supports function definitions at the top level.".to_string()),
                Some("Remove struct, trait, impl, or other definitions. Only 'fn' declarations are allowed.".to_string()),
            ),

            Error::Validation(msg) if msg.contains("Unsupported") => (
                ErrorCategory::UnsupportedFeature,
                Some("This Rust feature is not supported for shell script transpilation.".to_string()),
                Some("Check the user guide for supported features, or file an issue for feature requests.".to_string()),
            ),

            Error::Validation(msg) => (
                ErrorCategory::Validation,
                Some(format!("Validation failed: {}", msg)),
                Some("Review the error message and ensure your code follows Rash constraints.".to_string()),
            ),

            Error::IrGeneration(msg) => (
                ErrorCategory::Transpilation,
                Some(format!("Failed to generate intermediate representation: {}", msg)),
                Some("This is likely a transpiler bug. Please report this issue.".to_string()),
            ),

            Error::Io(_) => (
                ErrorCategory::Io,
                Some("Failed to read or write files.".to_string()),
                Some("Check file paths and permissions.".to_string()),
            ),

            Error::Unsupported(feature) => (
                ErrorCategory::UnsupportedFeature,
                Some(format!("The feature '{}' is not yet supported for transpilation.", feature)),
                Some("See docs/user-guide.md for supported features, or use a workaround.".to_string()),
            ),

            _ => (
                ErrorCategory::Internal,
                Some("An internal error occurred during transpilation.".to_string()),
                Some("This may be a bug. Please report this with a minimal reproduction.".to_string()),
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

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Error header with location
        write!(f, "error")?;

        if let Some(file) = &self.file {
            write!(f, " in {}", file)?;
            if let Some(line) = self.line {
                write!(f, ":{}", line)?;
                if let Some(col) = self.column {
                    write!(f, ":{}", col)?;
                }
            }
        }

        writeln!(f, ": {}", self.error)?;

        // Code snippet (if available)
        if let Some(snippet) = &self.snippet {
            writeln!(f)?;
            writeln!(f, "{}", snippet)?;
            if let Some(col) = self.column {
                // Add caret indicator
                writeln!(f, "{}^", " ".repeat(col.saturating_sub(1)))?;
            }
        }

        // Note (explanation)
        if let Some(note) = &self.note {
            writeln!(f)?;
            writeln!(f, "note: {}", note)?;
        }

        // Help (suggestion)
        if let Some(help) = &self.help {
            writeln!(f)?;
            writeln!(f, "help: {}", help)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let output = format!("{}", diag);
        assert!(output.contains("error in main.rs:5:10"));
        assert!(output.contains("note: Expected a semicolon"));
        assert!(output.contains("help: Add ';'"));
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

        let output = format!("{}", diag);
        assert!(output.contains("error: parse error"));
        assert!(!output.contains("in "));
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

        let output = format!("{}", diag);
        assert!(output.contains("error in test.rs"));
        assert!(!output.contains(":0")); // No line number
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

        let output = format!("{}", diag);
        assert!(output.contains("error in test.rs:42"));
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
            snippet: Some("let x = foo(".to_string()),
        };

        let output = format!("{}", diag);
        assert!(output.contains("let x = foo("));
        assert!(output.contains("^")); // Caret indicator
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
            snippet: Some("bad code".to_string()),
        };

        let output = format!("{}", diag);
        assert!(output.contains("bad code"));
        assert!(output.contains("^")); // Caret at column 0
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
            "Score with snippet should be >0.9, got {}",
            score
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
    fn test_categorize_validation_unsupported() {
        let error = Error::Validation("Unsupported expression type".to_string());
        let diag = Diagnostic::from_error(&error, None);

        assert_eq!(diag.category, ErrorCategory::UnsupportedFeature);
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
        assert!(diag.help.as_ref().unwrap().contains("permissions"));
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
        let debug_str = format!("{:?}", cat);
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

        let debug_str = format!("{:?}", diag);
        assert!(debug_str.contains("Diagnostic"));
        assert!(debug_str.contains("test"));
    }
}
