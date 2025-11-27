//! Shell error categories for ML classification.

use serde::{Deserialize, Serialize};

/// Shell error categories for ML classification.
///
/// 24 categories covering:
/// - Syntax errors (0-9)
/// - Command errors (10-19)
/// - File errors (20-29)
/// - Variable errors (30-39)
/// - Process errors (40-49)
/// - Pipe/Redirect errors (50-59)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ErrorCategory {
    // Syntax errors (0-9)
    /// Quote mismatch (' or ")
    SyntaxQuoteMismatch = 0,
    /// Bracket mismatch ([], {}, ())
    SyntaxBracketMismatch = 1,
    /// Unexpected token
    SyntaxUnexpectedToken = 2,
    /// Missing operand
    SyntaxMissingOperand = 3,

    // Command errors (10-19)
    /// Command not found (exit 127)
    CommandNotFound = 10,
    /// Permission denied (exit 126)
    CommandPermissionDenied = 11,
    /// Invalid option
    CommandInvalidOption = 12,
    /// Missing required argument
    CommandMissingArgument = 13,

    // File errors (20-29)
    /// No such file or directory
    FileNotFound = 20,
    /// Permission denied for file
    FilePermissionDenied = 21,
    /// Target is a directory
    FileIsDirectory = 22,
    /// Target is not a directory
    FileNotDirectory = 23,
    /// Too many open files
    FileTooManyOpen = 24,

    // Variable errors (30-39)
    /// Unbound variable
    VariableUnbound = 30,
    /// Readonly variable
    VariableReadonly = 31,
    /// Bad substitution
    VariableBadSubstitution = 32,

    // Process errors (40-49)
    /// Process killed by signal
    ProcessSignaled = 40,
    /// Non-zero exit code
    ProcessExitNonZero = 41,
    /// Process timeout
    ProcessTimeout = 42,

    // Pipe/Redirect errors (50-59)
    /// Broken pipe (SIGPIPE)
    PipeBroken = 50,
    /// Redirect failed
    RedirectFailed = 51,
    /// Unterminated here-doc
    HereDocUnterminated = 52,

    // Unknown (fallback)
    /// Unknown/uncategorized error
    Unknown = 255,
}

impl ErrorCategory {
    /// Number of categories (for ML output layer).
    pub const COUNT: usize = 24;

    /// Convert from u8 class label.
    #[must_use]
    pub fn from_label(label: u8) -> Self {
        match label {
            0 => Self::SyntaxQuoteMismatch,
            1 => Self::SyntaxBracketMismatch,
            2 => Self::SyntaxUnexpectedToken,
            3 => Self::SyntaxMissingOperand,
            10 => Self::CommandNotFound,
            11 => Self::CommandPermissionDenied,
            12 => Self::CommandInvalidOption,
            13 => Self::CommandMissingArgument,
            20 => Self::FileNotFound,
            21 => Self::FilePermissionDenied,
            22 => Self::FileIsDirectory,
            23 => Self::FileNotDirectory,
            24 => Self::FileTooManyOpen,
            30 => Self::VariableUnbound,
            31 => Self::VariableReadonly,
            32 => Self::VariableBadSubstitution,
            40 => Self::ProcessSignaled,
            41 => Self::ProcessExitNonZero,
            42 => Self::ProcessTimeout,
            50 => Self::PipeBroken,
            51 => Self::RedirectFailed,
            52 => Self::HereDocUnterminated,
            _ => Self::Unknown,
        }
    }

    /// Convert to ML label index (0-23 for training).
    #[must_use]
    pub fn to_label_index(&self) -> usize {
        match self {
            Self::SyntaxQuoteMismatch => 0,
            Self::SyntaxBracketMismatch => 1,
            Self::SyntaxUnexpectedToken => 2,
            Self::SyntaxMissingOperand => 3,
            Self::CommandNotFound => 4,
            Self::CommandPermissionDenied => 5,
            Self::CommandInvalidOption => 6,
            Self::CommandMissingArgument => 7,
            Self::FileNotFound => 8,
            Self::FilePermissionDenied => 9,
            Self::FileIsDirectory => 10,
            Self::FileNotDirectory => 11,
            Self::FileTooManyOpen => 12,
            Self::VariableUnbound => 13,
            Self::VariableReadonly => 14,
            Self::VariableBadSubstitution => 15,
            Self::ProcessSignaled => 16,
            Self::ProcessExitNonZero => 17,
            Self::ProcessTimeout => 18,
            Self::PipeBroken => 19,
            Self::RedirectFailed => 20,
            Self::HereDocUnterminated => 21,
            Self::Unknown => 22,
        }
    }

    /// Convert from ML label index (0-23).
    #[must_use]
    pub fn from_label_index(idx: usize) -> Self {
        match idx {
            0 => Self::SyntaxQuoteMismatch,
            1 => Self::SyntaxBracketMismatch,
            2 => Self::SyntaxUnexpectedToken,
            3 => Self::SyntaxMissingOperand,
            4 => Self::CommandNotFound,
            5 => Self::CommandPermissionDenied,
            6 => Self::CommandInvalidOption,
            7 => Self::CommandMissingArgument,
            8 => Self::FileNotFound,
            9 => Self::FilePermissionDenied,
            10 => Self::FileIsDirectory,
            11 => Self::FileNotDirectory,
            12 => Self::FileTooManyOpen,
            13 => Self::VariableUnbound,
            14 => Self::VariableReadonly,
            15 => Self::VariableBadSubstitution,
            16 => Self::ProcessSignaled,
            17 => Self::ProcessExitNonZero,
            18 => Self::ProcessTimeout,
            19 => Self::PipeBroken,
            20 => Self::RedirectFailed,
            21 => Self::HereDocUnterminated,
            _ => Self::Unknown,
        }
    }

    /// Human-readable fix suggestion.
    #[must_use]
    pub fn fix_suggestion(&self) -> &'static str {
        match self {
            Self::SyntaxQuoteMismatch => "Check for unmatched quotes (' or \")",
            Self::SyntaxBracketMismatch => "Check for unmatched brackets ([], {}, ())",
            Self::SyntaxUnexpectedToken => "Review syntax near the reported token",
            Self::SyntaxMissingOperand => "Add missing operand to the expression",
            Self::CommandNotFound => "Check PATH or install the missing command",
            Self::CommandPermissionDenied => "Use chmod +x or run with sudo",
            Self::CommandInvalidOption => "Check command documentation for valid options",
            Self::CommandMissingArgument => "Provide required argument to the command",
            Self::FileNotFound => "Verify the file path exists",
            Self::FilePermissionDenied => "Check file permissions with ls -la",
            Self::FileIsDirectory => "Use a file path, not a directory",
            Self::FileNotDirectory => "Use a directory path, not a file",
            Self::FileTooManyOpen => "Close unused file descriptors or increase ulimit",
            Self::VariableUnbound => "Initialize variable or use ${VAR:-default}",
            Self::VariableReadonly => "Cannot modify readonly variable",
            Self::VariableBadSubstitution => "Fix parameter expansion syntax",
            Self::ProcessSignaled => "Process was killed by signal",
            Self::ProcessExitNonZero => "Check command exit status",
            Self::ProcessTimeout => "Increase timeout or optimize command",
            Self::PipeBroken => "Check if downstream process exited early",
            Self::RedirectFailed => "Verify target path is writable",
            Self::HereDocUnterminated => "Add terminating delimiter for here-doc",
            Self::Unknown => "Review the error message for details",
        }
    }

    /// Get human-readable category name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::SyntaxQuoteMismatch => "Syntax: Quote Mismatch",
            Self::SyntaxBracketMismatch => "Syntax: Bracket Mismatch",
            Self::SyntaxUnexpectedToken => "Syntax: Unexpected Token",
            Self::SyntaxMissingOperand => "Syntax: Missing Operand",
            Self::CommandNotFound => "Command: Not Found",
            Self::CommandPermissionDenied => "Command: Permission Denied",
            Self::CommandInvalidOption => "Command: Invalid Option",
            Self::CommandMissingArgument => "Command: Missing Argument",
            Self::FileNotFound => "File: Not Found",
            Self::FilePermissionDenied => "File: Permission Denied",
            Self::FileIsDirectory => "File: Is Directory",
            Self::FileNotDirectory => "File: Not Directory",
            Self::FileTooManyOpen => "File: Too Many Open",
            Self::VariableUnbound => "Variable: Unbound",
            Self::VariableReadonly => "Variable: Readonly",
            Self::VariableBadSubstitution => "Variable: Bad Substitution",
            Self::ProcessSignaled => "Process: Signaled",
            Self::ProcessExitNonZero => "Process: Non-Zero Exit",
            Self::ProcessTimeout => "Process: Timeout",
            Self::PipeBroken => "Pipe: Broken",
            Self::RedirectFailed => "Redirect: Failed",
            Self::HereDocUnterminated => "HereDoc: Unterminated",
            Self::Unknown => "Unknown",
        }
    }

    /// All categories for iteration.
    #[must_use]
    pub fn all() -> &'static [ErrorCategory] {
        &[
            Self::SyntaxQuoteMismatch,
            Self::SyntaxBracketMismatch,
            Self::SyntaxUnexpectedToken,
            Self::SyntaxMissingOperand,
            Self::CommandNotFound,
            Self::CommandPermissionDenied,
            Self::CommandInvalidOption,
            Self::CommandMissingArgument,
            Self::FileNotFound,
            Self::FilePermissionDenied,
            Self::FileIsDirectory,
            Self::FileNotDirectory,
            Self::FileTooManyOpen,
            Self::VariableUnbound,
            Self::VariableReadonly,
            Self::VariableBadSubstitution,
            Self::ProcessSignaled,
            Self::ProcessExitNonZero,
            Self::ProcessTimeout,
            Self::PipeBroken,
            Self::RedirectFailed,
            Self::HereDocUnterminated,
            Self::Unknown,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_roundtrip() {
        for cat in ErrorCategory::all() {
            let idx = cat.to_label_index();
            let restored = ErrorCategory::from_label_index(idx);
            assert_eq!(*cat, restored, "Roundtrip failed for {cat:?}");
        }
    }

    #[test]
    fn test_all_categories_count() {
        // 23 defined + Unknown = 23 in all() (Unknown included)
        assert_eq!(ErrorCategory::all().len(), 23);
    }

    #[test]
    fn test_fix_suggestions_not_empty() {
        for cat in ErrorCategory::all() {
            assert!(
                !cat.fix_suggestion().is_empty(),
                "Empty fix suggestion for {cat:?}"
            );
        }
    }

    #[test]
    fn test_names_not_empty() {
        for cat in ErrorCategory::all() {
            assert!(!cat.name().is_empty(), "Empty name for {cat:?}");
        }
    }

    #[test]
    fn test_from_label_unknown() {
        assert_eq!(ErrorCategory::from_label(200), ErrorCategory::Unknown);
        assert_eq!(ErrorCategory::from_label(100), ErrorCategory::Unknown);
    }
}
