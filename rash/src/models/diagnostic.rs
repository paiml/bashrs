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
                    "Check for typos or unsupported syntax near the indicated location."
                        .to_string(),
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
                Some(
                    "Ensure your code is valid Rust syntax. Rash supports a subset of Rust."
                        .to_string(),
                ),
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
    std::env::var("NO_COLOR").is_err() && std::env::var("TERM").map_or(true, |t| t != "dumb")
}

/// Colorize a snippet line with gutter pipes (blue) and carets (red)
fn write_snippet_line(
    f: &mut fmt::Formatter<'_>,
    line: &str,
    blue: &str,
    red: &str,
    reset: &str,
) -> fmt::Result {
    if line.contains('^') {
        let (before_caret, from_caret) = line.split_at(line.find('^').unwrap_or(line.len()));
        if let Some(pipe_pos) = before_caret.find('|') {
            let (gutter, rest) = before_caret.split_at(pipe_pos + 1);
            writeln!(f, "{blue}{gutter}{reset}{rest}{red}{from_caret}{reset}")
        } else {
            writeln!(f, "{before_caret}{red}{from_caret}{reset}")
        }
    } else if let Some(pipe_pos) = line.find('|') {
        let (gutter, code) = line.split_at(pipe_pos + 1);
        writeln!(f, "{blue}{gutter}{reset}{code}")
    } else {
        writeln!(f, "{line}")
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let color = use_color();

        let red = if color { "\x1b[1;31m" } else { "" };
        let blue = if color { "\x1b[1;34m" } else { "" };
        let cyan = if color { "\x1b[1;36m" } else { "" };
        let green = if color { "\x1b[1;32m" } else { "" };
        let bold = if color { "\x1b[1m" } else { "" };
        let reset = if color { "\x1b[0m" } else { "" };

        let tag = self.category.tag();
        writeln!(f, "{red}error[{tag}]{reset}: {bold}{}{reset}", self.error)?;

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

        if let Some(snippet) = &self.snippet {
            for line in snippet.lines() {
                write_snippet_line(f, line, blue, red, reset)?;
            }
        }

        if let Some(note) = &self.note {
            writeln!(f, "  {cyan}note{reset}: {note}")?;
        }

        if let Some(help) = &self.help {
            writeln!(f, "  {green}help{reset}: {help}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "diagnostic_tests_format_no.rs"]
mod tests_ext;
