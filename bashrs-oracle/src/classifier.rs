//! Keyword-based error classifier (fallback when ML model not trained).

use crate::ErrorCategory;

/// Keyword-based classifier for shell errors.
///
/// Maps error message keywords to the 24 error categories.
/// Used as fallback when ML model is not trained.
pub struct ErrorClassifier {
    /// Keywords mapped to categories.
    keyword_map: Vec<(Vec<&'static str>, ErrorCategory)>,
}

impl Default for ErrorClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorClassifier {
    /// Create a new keyword-based classifier with 24 category mappings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            keyword_map: vec![
                // Syntax errors (0-3)
                (
                    vec![
                        "unexpected EOF",
                        "unmatched",
                        "unterminated",
                        "missing '\"'",
                        "missing \"'\"",
                    ],
                    ErrorCategory::SyntaxQuoteMismatch,
                ),
                (
                    vec![
                        "unexpected ')'",
                        "unexpected ']'",
                        "unexpected '}'",
                        "missing ')'",
                    ],
                    ErrorCategory::SyntaxBracketMismatch,
                ),
                (
                    vec![
                        "unexpected token",
                        "syntax error near",
                        "parse error",
                        "unexpected end",
                    ],
                    ErrorCategory::SyntaxUnexpectedToken,
                ),
                (
                    vec!["operand expected", "missing operand", "expression expected"],
                    ErrorCategory::SyntaxMissingOperand,
                ),
                // Command errors (10-13)
                (
                    vec!["command not found", "not found:", ": not found"],
                    ErrorCategory::CommandNotFound,
                ),
                (
                    vec![
                        "Permission denied",
                        "permission denied",
                        "cannot execute",
                        "not executable",
                    ],
                    ErrorCategory::CommandPermissionDenied,
                ),
                (
                    vec!["invalid option", "unrecognized option", "illegal option"],
                    ErrorCategory::CommandInvalidOption,
                ),
                (
                    vec![
                        "requires an argument",
                        "missing argument",
                        "option requires",
                    ],
                    ErrorCategory::CommandMissingArgument,
                ),
                // File errors (20-24)
                (
                    vec![
                        "No such file",
                        "no such file",
                        "not found",
                        "does not exist",
                    ],
                    ErrorCategory::FileNotFound,
                ),
                (
                    vec!["Permission denied", "cannot access", "EACCES"],
                    ErrorCategory::FilePermissionDenied,
                ),
                (
                    vec!["Is a directory", "is a directory"],
                    ErrorCategory::FileIsDirectory,
                ),
                (
                    vec!["Not a directory", "not a directory"],
                    ErrorCategory::FileNotDirectory,
                ),
                (
                    vec!["Too many open files", "EMFILE", "ENFILE"],
                    ErrorCategory::FileTooManyOpen,
                ),
                // Variable errors (30-32)
                (
                    vec![
                        "unbound variable",
                        "parameter not set",
                        "undefined variable",
                    ],
                    ErrorCategory::VariableUnbound,
                ),
                (
                    vec!["readonly variable", "read-only variable", "cannot assign"],
                    ErrorCategory::VariableReadonly,
                ),
                (
                    vec!["bad substitution", "bad parameter", "invalid substitution"],
                    ErrorCategory::VariableBadSubstitution,
                ),
                // Process errors (40-42)
                (
                    vec!["Killed", "killed", "signal", "SIGKILL", "SIGTERM"],
                    ErrorCategory::ProcessSignaled,
                ),
                (
                    vec!["exit status", "exited with", "returned"],
                    ErrorCategory::ProcessExitNonZero,
                ),
                (
                    vec!["timed out", "timeout", "exceeded time"],
                    ErrorCategory::ProcessTimeout,
                ),
                // Pipe/redirect errors (50-52)
                (
                    vec!["Broken pipe", "broken pipe", "SIGPIPE", "EPIPE"],
                    ErrorCategory::PipeBroken,
                ),
                (
                    vec![
                        "cannot redirect",
                        "redirect failed",
                        "No space left",
                        "ambiguous redirect",
                    ],
                    ErrorCategory::RedirectFailed,
                ),
                (
                    vec![
                        "here-document",
                        "heredoc",
                        "here document",
                        "delimited by end-of-file",
                    ],
                    ErrorCategory::HereDocUnterminated,
                ),
            ],
        }
    }

    /// Classify an error message by keywords.
    #[must_use]
    pub fn classify_by_keywords(&self, message: &str) -> ErrorCategory {
        let message_lower = message.to_lowercase();

        for (keywords, category) in &self.keyword_map {
            for keyword in keywords {
                if message_lower.contains(&keyword.to_lowercase()) {
                    return *category;
                }
            }
        }

        ErrorCategory::Unknown
    }

    /// Calculate confidence based on keyword matches.
    #[must_use]
    pub fn confidence(&self, message: &str, category: ErrorCategory) -> f32 {
        let message_lower = message.to_lowercase();
        let mut matches = 0;
        let mut total_keywords = 0;

        for (keywords, cat) in &self.keyword_map {
            if *cat == category {
                total_keywords = keywords.len();
                for keyword in keywords {
                    if message_lower.contains(&keyword.to_lowercase()) {
                        matches += 1;
                    }
                }
                break;
            }
        }

        if total_keywords == 0 {
            return 0.5; // Default confidence for Unknown
        }

        // Base confidence + bonus for multiple matches
        let base = 0.6;
        let match_bonus = (matches as f32 / total_keywords as f32) * 0.35;
        (base + match_bonus).min(0.95)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_syntax_quote_mismatch() {
        let classifier = ErrorClassifier::new();
        let msg = "unexpected EOF while looking for matching '\"'";
        assert_eq!(
            classifier.classify_by_keywords(msg),
            ErrorCategory::SyntaxQuoteMismatch
        );
    }

    #[test]
    fn test_classify_syntax_bracket_mismatch() {
        let classifier = ErrorClassifier::new();
        let msg = "syntax error near unexpected ')'";
        assert_eq!(
            classifier.classify_by_keywords(msg),
            ErrorCategory::SyntaxBracketMismatch
        );
    }

    #[test]
    fn test_classify_syntax_unexpected_token() {
        let classifier = ErrorClassifier::new();
        let msg = "syntax error near unexpected token 'done'";
        assert_eq!(
            classifier.classify_by_keywords(msg),
            ErrorCategory::SyntaxUnexpectedToken
        );
    }

    #[test]
    fn test_classify_command_not_found() {
        let classifier = ErrorClassifier::new();
        let msg = "bash: foobar: command not found";
        assert_eq!(
            classifier.classify_by_keywords(msg),
            ErrorCategory::CommandNotFound
        );
    }

    #[test]
    fn test_classify_command_permission_denied() {
        let classifier = ErrorClassifier::new();
        let msg = "bash: ./script.sh: Permission denied";
        assert_eq!(
            classifier.classify_by_keywords(msg),
            ErrorCategory::CommandPermissionDenied
        );
    }

    #[test]
    fn test_classify_file_not_found() {
        let classifier = ErrorClassifier::new();
        let msg = "cat: /nonexistent: No such file or directory";
        assert_eq!(
            classifier.classify_by_keywords(msg),
            ErrorCategory::FileNotFound
        );
    }

    #[test]
    fn test_classify_file_is_directory() {
        let classifier = ErrorClassifier::new();
        let msg = "cat: /tmp: Is a directory";
        assert_eq!(
            classifier.classify_by_keywords(msg),
            ErrorCategory::FileIsDirectory
        );
    }

    #[test]
    fn test_classify_variable_unbound() {
        let classifier = ErrorClassifier::new();
        let msg = "bash: VAR: unbound variable";
        assert_eq!(
            classifier.classify_by_keywords(msg),
            ErrorCategory::VariableUnbound
        );
    }

    #[test]
    fn test_classify_variable_readonly() {
        let classifier = ErrorClassifier::new();
        let msg = "bash: PATH: readonly variable";
        assert_eq!(
            classifier.classify_by_keywords(msg),
            ErrorCategory::VariableReadonly
        );
    }

    #[test]
    fn test_classify_variable_bad_substitution() {
        let classifier = ErrorClassifier::new();
        let msg = "bash: ${foo: bad substitution";
        assert_eq!(
            classifier.classify_by_keywords(msg),
            ErrorCategory::VariableBadSubstitution
        );
    }

    #[test]
    fn test_classify_pipe_broken() {
        let classifier = ErrorClassifier::new();
        let msg = "Broken pipe";
        assert_eq!(
            classifier.classify_by_keywords(msg),
            ErrorCategory::PipeBroken
        );
    }

    #[test]
    fn test_classify_redirect_failed() {
        let classifier = ErrorClassifier::new();
        let msg = "bash: /dev/full: No space left on device";
        assert_eq!(
            classifier.classify_by_keywords(msg),
            ErrorCategory::RedirectFailed
        );
    }

    #[test]
    fn test_classify_heredoc_unterminated() {
        let classifier = ErrorClassifier::new();
        let msg = "warning: here-document delimited by end-of-file";
        assert_eq!(
            classifier.classify_by_keywords(msg),
            ErrorCategory::HereDocUnterminated
        );
    }

    #[test]
    fn test_classify_unknown() {
        let classifier = ErrorClassifier::new();
        let msg = "some random error message";
        assert_eq!(classifier.classify_by_keywords(msg), ErrorCategory::Unknown);
    }

    #[test]
    fn test_confidence_high_match() {
        let classifier = ErrorClassifier::new();
        let msg = "bash: foo: command not found";
        let conf = classifier.confidence(msg, ErrorCategory::CommandNotFound);
        assert!(conf > 0.6, "Expected high confidence, got {conf}");
    }

    #[test]
    fn test_confidence_unknown() {
        let classifier = ErrorClassifier::new();
        let msg = "some message";
        let conf = classifier.confidence(msg, ErrorCategory::Unknown);
        assert!(
            (conf - 0.5).abs() < f32::EPSILON,
            "Expected default confidence 0.5, got {conf}"
        );
    }
}
