//! Enhanced error types for Makefile parser
//!
//! Sprint 73 Phase 5: Error Handling Polish
//!
//! Provides structured error types with:
//! - Source location (file, line, column)
//! - Code snippets
//! - Explanatory notes
//! - Recovery hints
//!
//! Target: Error quality score ≥0.8

use std::fmt;
use thiserror::Error;

/// Source location information for error reporting
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub file: Option<String>,
    pub line: usize,
    pub column: Option<usize>,
    pub source_line: Option<String>,
}

impl SourceLocation {
    pub fn new(line: usize) -> Self {
        Self {
            file: None,
            line,
            column: None,
            source_line: None,
        }
    }

    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }

    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    pub fn with_source_line(mut self, source_line: String) -> Self {
        self.source_line = Some(source_line);
        self
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(file) = &self.file {
            write!(f, "{}:{}", file, self.line)?;
        } else {
            write!(f, "line {}", self.line)?;
        }

        if let Some(col) = self.column {
            write!(f, ":{}", col)?;
        }

        Ok(())
    }
}

/// Enhanced error types for Makefile parsing
#[derive(Error, Debug)]
pub enum MakeParseError {
    #[error("Invalid variable assignment at {location}")]
    InvalidVariableAssignment {
        location: SourceLocation,
        found: String,
    },

    #[error("Empty variable name at {location}")]
    EmptyVariableName { location: SourceLocation },

    #[error("No assignment operator found at {location}")]
    NoAssignmentOperator {
        location: SourceLocation,
        found: String,
    },

    #[error("Invalid include syntax at {location}")]
    InvalidIncludeSyntax {
        location: SourceLocation,
        found: String,
    },

    #[error("Invalid conditional syntax at {location}")]
    InvalidConditionalSyntax {
        location: SourceLocation,
        directive: String,
        found: String,
    },

    #[error("Conditional requires arguments at {location}")]
    MissingConditionalArguments {
        location: SourceLocation,
        directive: String,
        expected_args: usize,
        found_args: usize,
    },

    #[error("Missing variable name in {directive} at {location}")]
    MissingVariableName {
        location: SourceLocation,
        directive: String,
    },

    #[error("Unknown conditional directive at {location}")]
    UnknownConditional {
        location: SourceLocation,
        found: String,
    },

    #[error("Invalid target rule syntax at {location}")]
    InvalidTargetRule {
        location: SourceLocation,
        found: String,
    },

    #[error("Empty target name at {location}")]
    EmptyTargetName { location: SourceLocation },

    #[error("Unterminated define block for variable '{var_name}' at {location}")]
    UnterminatedDefine {
        location: SourceLocation,
        var_name: String,
    },

    #[error("Unexpected end of file")]
    UnexpectedEof,
}

impl MakeParseError {
    /// Get the location information for this error
    pub fn location(&self) -> Option<&SourceLocation> {
        match self {
            Self::InvalidVariableAssignment { location, .. } => Some(location),
            Self::EmptyVariableName { location } => Some(location),
            Self::NoAssignmentOperator { location, .. } => Some(location),
            Self::InvalidIncludeSyntax { location, .. } => Some(location),
            Self::InvalidConditionalSyntax { location, .. } => Some(location),
            Self::MissingConditionalArguments { location, .. } => Some(location),
            Self::MissingVariableName { location, .. } => Some(location),
            Self::UnknownConditional { location, .. } => Some(location),
            Self::InvalidTargetRule { location, .. } => Some(location),
            Self::EmptyTargetName { location } => Some(location),
            Self::UnterminatedDefine { location, .. } => Some(location),
            Self::UnexpectedEof => None,
        }
    }

    /// Get explanatory note for this error
    pub fn note(&self) -> String {
        match self {
            Self::InvalidVariableAssignment { .. } => {
                "Variable assignments must use one of the assignment operators: =, :=, ?=, +=, !=".to_string()
            }
            Self::EmptyVariableName { .. } => {
                "Variable names cannot be empty. A valid variable name must contain at least one character.".to_string()
            }
            Self::NoAssignmentOperator { .. } => {
                "Variable assignments require an assignment operator (=, :=, ?=, +=, or !=)".to_string()
            }
            Self::InvalidIncludeSyntax { .. } => {
                "Include directives must be: 'include file', '-include file', or 'sinclude file'".to_string()
            }
            Self::InvalidConditionalSyntax { directive, .. } => {
                match directive.as_str() {
                    "ifeq" | "ifneq" => {
                        format!("{} requires arguments in parentheses with a comma separator", directive)
                    }
                    "ifdef" | "ifndef" => {
                        format!("{} requires a variable name argument", directive)
                    }
                    _ => "Conditional directives must follow GNU Make syntax".to_string(),
                }
            }
            Self::MissingConditionalArguments { directive, expected_args, found_args, .. } => {
                format!("{} requires {} argument(s), but found {}", directive, expected_args, found_args)
            }
            Self::MissingVariableName { directive, .. } => {
                format!("{} requires a variable name to test", directive)
            }
            Self::UnknownConditional { .. } => {
                "Supported conditional directives are: ifeq, ifneq, ifdef, ifndef".to_string()
            }
            Self::InvalidTargetRule { .. } => {
                "Target rules must have the format: target: prerequisites".to_string()
            }
            Self::EmptyTargetName { .. } => {
                "Target names cannot be empty. A valid target must have a name before the colon.".to_string()
            }
            Self::UnterminatedDefine { .. } => {
                "define blocks must be terminated with 'endef'".to_string()
            }
            Self::UnexpectedEof => {
                "The Makefile ended unexpectedly. Check for unclosed conditional blocks or incomplete rules.".to_string()
            }
        }
    }

    /// Get recovery hint for this error
    pub fn help(&self) -> String {
        match self {
            Self::InvalidVariableAssignment { .. } => {
                "Example: VAR = value\n       VAR := value\n       VAR ?= value".to_string()
            }
            Self::EmptyVariableName { .. } => {
                "Provide a variable name before the assignment operator.\nExample: MY_VAR = value".to_string()
            }
            Self::NoAssignmentOperator { .. } => {
                "Use one of the following assignment operators:\n  =   (recursive expansion)\n  :=  (simple expansion)\n  ?=  (conditional assignment)\n  +=  (append)\n  !=  (shell assignment)".to_string()
            }
            Self::InvalidIncludeSyntax { .. } => {
                "Use: include filename.mk\nOr for optional includes:\n     -include filename.mk\n     sinclude filename.mk".to_string()
            }
            Self::InvalidConditionalSyntax { directive, .. } => {
                match directive.as_str() {
                    "ifeq" => "Use: ifeq ($(VAR),value)\nOr:  ifeq (arg1,arg2)".to_string(),
                    "ifneq" => "Use: ifneq ($(VAR),value)\nOr:  ifneq (arg1,arg2)".to_string(),
                    "ifdef" => "Use: ifdef VARIABLE_NAME".to_string(),
                    "ifndef" => "Use: ifndef VARIABLE_NAME".to_string(),
                    _ => "Check the GNU Make manual for conditional syntax".to_string(),
                }
            }
            Self::MissingConditionalArguments { directive, .. } => {
                match directive.as_str() {
                    "ifeq" | "ifneq" => format!("Use: {} (arg1,arg2)", directive),
                    "ifdef" | "ifndef" => format!("Use: {} VAR_NAME", directive),
                    _ => "Provide the required arguments for the conditional".to_string(),
                }
            }
            Self::MissingVariableName { directive, .. } => {
                format!("Provide a variable name after {}.\nExample: {} DEBUG", directive, directive)
            }
            Self::UnknownConditional { found, .. } => {
                format!("Did you mean one of: ifeq, ifneq, ifdef, ifndef?\nFound: {}", found)
            }
            Self::InvalidTargetRule { .. } => {
                "Use the format: target: prerequisite1 prerequisite2\nFollowed by tab-indented recipe lines".to_string()
            }
            Self::EmptyTargetName { .. } => {
                "Provide a target name before the colon.\nExample: build: main.c\n\t$(CC) -o build main.c".to_string()
            }
            Self::UnterminatedDefine { .. } => {
                "Ensure all define blocks are closed with 'endef'.\nExample:\ndefine VAR_NAME\ncontent\nendef".to_string()
            }
            Self::UnexpectedEof => {
                "Ensure all conditional blocks (ifeq/ifdef/etc.) are closed with 'endif'.\nCheck that all target rules are complete.".to_string()
            }
        }
    }

    /// Convert to a displayable error message with note and help
    pub fn to_detailed_string(&self) -> String {
        let mut output = String::new();

        // Error message
        output.push_str("error: ");
        output.push_str(&self.to_string());
        output.push('\n');

        // Source code snippet (if available)
        if let Some(location) = self.location() {
            if let Some(source_line) = &location.source_line {
                output.push('\n');
                output.push_str(&format!("{} | {}\n", location.line, source_line));

                // Add caret indicator if column is known
                if let Some(col) = location.column {
                    let line_num_width = format!("{}", location.line).len();
                    let spaces = " ".repeat(line_num_width + 3 + col.saturating_sub(1));
                    output.push_str(&format!("{}^\n", spaces));
                }
            }
        }

        // Note (explanation)
        output.push('\n');
        output.push_str("note: ");
        output.push_str(&self.note());
        output.push('\n');

        // Help (recovery hint)
        output.push('\n');
        output.push_str("help: ");
        output.push_str(&self.help());
        output.push('\n');

        output
    }

    /// Calculate quality score for this error
    ///
    /// Score components:
    /// - Error message: 1.0 (always present)
    /// - File location: 1.0
    /// - Line number: 0.25
    /// - Column number: 0.25
    /// - Code snippet: 1.0
    /// - Note: 2.5 (always present)
    /// - Help: 2.5 (always present)
    ///
    /// Max: 8.5 → normalized to 1.0
    pub fn quality_score(&self) -> f32 {
        let mut score = 0.0;

        // Error message (always present)
        score += 1.0;

        // Note (always present)
        score += 2.5;

        // Help (always present)
        score += 2.5;

        // Location-based scores
        if let Some(location) = self.location() {
            if location.file.is_some() {
                score += 1.0;
            }
            // Line always present for located errors
            score += 0.25;

            if location.column.is_some() {
                score += 0.25;
            }

            if location.source_line.is_some() {
                score += 1.0;
            }
        }

        score / 8.5 // Normalize to 0-1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_score_minimum() {
        // UnexpectedEof has no location, so minimal score
        let error = MakeParseError::UnexpectedEof;
        let score = error.quality_score();

        // Score: error(1.0) + note(2.5) + help(2.5) = 6.0 / 8.5 = 0.706
        assert!(score >= 0.7, "Score {} should be ≥0.7", score);
        assert!(score < 0.75, "Score {} should be <0.75", score);
    }

    #[test]
    fn test_quality_score_with_location() {
        let location = SourceLocation::new(15);
        let error = MakeParseError::EmptyVariableName { location };
        let score = error.quality_score();

        // Score: error(1.0) + note(2.5) + help(2.5) + line(0.25) = 6.25 / 8.5 = 0.735
        assert!(score >= 0.73, "Score {} should be ≥0.73", score);
        assert!(score < 0.75, "Score {} should be <0.75", score);
    }

    #[test]
    fn test_quality_score_with_file_and_column() {
        let location = SourceLocation::new(15)
            .with_file("Makefile".to_string())
            .with_column(8);

        let error = MakeParseError::EmptyTargetName { location };
        let score = error.quality_score();

        // Score: error(1.0) + note(2.5) + help(2.5) + file(1.0) + line(0.25) + column(0.25) = 7.5 / 8.5 = 0.882
        assert!(score >= 0.88, "Score {} should be ≥0.88", score);
        assert!(score < 0.89, "Score {} should be <0.89", score);
    }

    #[test]
    fn test_quality_score_with_snippet() {
        let location = SourceLocation::new(15)
            .with_file("Makefile".to_string())
            .with_column(8)
            .with_source_line("ifeq $(VAR) value".to_string());

        let error = MakeParseError::InvalidConditionalSyntax {
            location,
            directive: "ifeq".to_string(),
            found: "$(VAR) value".to_string(),
        };

        let score = error.quality_score();

        // Score: error(1.0) + note(2.5) + help(2.5) + file(1.0) + line(0.25) + column(0.25) + snippet(1.0) = 8.5 / 8.5 = 1.0
        assert_eq!(score, 1.0, "Score should be perfect 1.0");
    }

    #[test]
    fn test_quality_score_target_exceeds_08() {
        // Target: All errors with full context should achieve ≥0.8 quality score
        let location = SourceLocation::new(15)
            .with_file("Makefile".to_string())
            .with_column(8)
            .with_source_line("ifeq $(VAR) value".to_string());

        let error = MakeParseError::InvalidConditionalSyntax {
            location,
            directive: "ifeq".to_string(),
            found: "$(VAR) value".to_string(),
        };

        assert!(
            error.quality_score() >= 0.8,
            "Error quality score {} must be ≥0.8",
            error.quality_score()
        );
    }

    #[test]
    fn test_note_present_for_all_errors() {
        // All error types should have explanatory notes
        let errors = vec![
            MakeParseError::EmptyVariableName {
                location: SourceLocation::new(1),
            },
            MakeParseError::InvalidConditionalSyntax {
                location: SourceLocation::new(1),
                directive: "ifeq".to_string(),
                found: "bad".to_string(),
            },
            MakeParseError::UnexpectedEof,
        ];

        for error in errors {
            let note = error.note();
            assert!(!note.is_empty(), "Note should not be empty for {:?}", error);
            assert!(
                note.len() > 10,
                "Note should be descriptive for {:?}",
                error
            );
        }
    }

    #[test]
    fn test_help_present_for_all_errors() {
        // All error types should have recovery hints
        let errors = vec![
            MakeParseError::EmptyTargetName {
                location: SourceLocation::new(1),
            },
            MakeParseError::InvalidIncludeSyntax {
                location: SourceLocation::new(1),
                found: "bad include".to_string(),
            },
            MakeParseError::UnexpectedEof,
        ];

        for error in errors {
            let help = error.help();
            assert!(!help.is_empty(), "Help should not be empty for {:?}", error);
            assert!(help.len() > 10, "Help should be actionable for {:?}", error);
        }
    }

    #[test]
    fn test_detailed_string_format() {
        let location = SourceLocation::new(15)
            .with_file("Makefile".to_string())
            .with_column(8)
            .with_source_line("ifeq $(VAR) value".to_string());

        let error = MakeParseError::InvalidConditionalSyntax {
            location,
            directive: "ifeq".to_string(),
            found: "$(VAR) value".to_string(),
        };

        let detailed = error.to_detailed_string();

        // Should contain all components
        assert!(detailed.contains("error:"));
        assert!(detailed.contains("15 | ifeq $(VAR) value"));
        assert!(detailed.contains("note:"));
        assert!(detailed.contains("help:"));
        assert!(detailed.contains("^")); // Caret indicator
    }
}
