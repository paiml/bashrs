//! DOCKER012: STOPSIGNAL validation (F075)
//!
//! **Rule**: Validate STOPSIGNAL directive for proper signal handling
//!
//! **Why this matters**:
//! STOPSIGNAL determines how Docker terminates the container. Using the
//! correct signal ensures graceful shutdown and proper cleanup.
//!
//! ## Examples
//!
//! ✅ **GOOD** (explicit STOPSIGNAL):
//! ```dockerfile
//! FROM python:3.12
//! STOPSIGNAL SIGTERM
//! CMD ["python", "app.py"]
//! ```

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Valid signal names
const VALID_SIGNALS: &[&str] = &[
    "SIGTERM",
    "SIGINT",
    "SIGQUIT",
    "SIGKILL",
    "SIGHUP",
    "SIGUSR1",
    "SIGUSR2",
    "SIGWINCH",
    "SIGABRT",
    "SIGALRM",
    "SIGBUS",
    "SIGCHLD",
    "SIGCONT",
    "SIGFPE",
    "SIGILL",
    "SIGIO",
    "SIGPIPE",
    "SIGPROF",
    "SIGSEGV",
    "SIGSTOP",
    "SIGSYS",
    "SIGTRAP",
    "SIGTSTP",
    "SIGTTIN",
    "SIGTTOU",
    "SIGURG",
    "SIGVTALRM",
    "SIGXCPU",
    "SIGXFSZ",
];

/// Signals without SIG prefix
const VALID_SIGNALS_SHORT: &[&str] = &[
    "TERM", "INT", "QUIT", "KILL", "HUP", "USR1", "USR2", "WINCH", "ABRT", "ALRM", "BUS", "CHLD",
    "CONT", "FPE", "ILL", "IO", "PIPE", "PROF", "SEGV", "STOP", "SYS", "TRAP", "TSTP", "TTIN",
    "TTOU", "URG", "VTALRM", "XCPU", "XFSZ",
];

/// Check for STOPSIGNAL validation
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();

        if upper.starts_with("STOPSIGNAL ") {
            let signal = trimmed[11..].trim().to_uppercase();

            // Check for valid signal
            let is_valid_named = VALID_SIGNALS.contains(&signal.as_str())
                || VALID_SIGNALS_SHORT.contains(&signal.as_str());

            // Check for numeric signal (1-31 typically valid)
            let is_valid_numeric = signal
                .parse::<u32>()
                .map(|n| (1..=31).contains(&n))
                .unwrap_or(false);

            if !is_valid_named && !is_valid_numeric {
                let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
                let diag = Diagnostic::new(
                    "DOCKER012",
                    Severity::Error,
                    format!(
                        "Invalid STOPSIGNAL '{}' - use signal name (SIGTERM) or number (15) (F075)",
                        signal
                    ),
                    span,
                );
                result.add(diag);
            }

            // Warn about SIGKILL (doesn't allow graceful shutdown)
            if signal == "SIGKILL" || signal == "KILL" || signal == "9" {
                let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
                let diag = Diagnostic::new(
                    "DOCKER012",
                    Severity::Warning,
                    "STOPSIGNAL SIGKILL prevents graceful shutdown - consider SIGTERM (F075)"
                        .to_string(),
                    span,
                );
                result.add(diag);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F075: STOPSIGNAL validation
    #[test]
    fn test_F075_valid_sigterm() {
        let dockerfile = r#"FROM python:3.12
STOPSIGNAL SIGTERM
CMD ["python", "app.py"]"#;
        let result = check(dockerfile);

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Error),
            "F075: SIGTERM should be valid"
        );
    }

    #[test]
    fn test_F075_valid_numeric() {
        let dockerfile = r#"FROM python:3.12
STOPSIGNAL 15
CMD ["python", "app.py"]"#;
        let result = check(dockerfile);

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Error),
            "F075: Numeric signal 15 should be valid"
        );
    }

    #[test]
    fn test_F075_invalid_signal() {
        let dockerfile = r#"FROM python:3.12
STOPSIGNAL INVALID
CMD ["python", "app.py"]"#;
        let result = check(dockerfile);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("Invalid STOPSIGNAL")),
            "F075: Invalid signal should be flagged"
        );
    }

    #[test]
    fn test_F075_sigkill_warning() {
        let dockerfile = r#"FROM python:3.12
STOPSIGNAL SIGKILL
CMD ["python", "app.py"]"#;
        let result = check(dockerfile);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("graceful shutdown")),
            "F075: SIGKILL should warn about graceful shutdown"
        );
    }

    #[test]
    fn test_F075_short_signal_name() {
        let dockerfile = r#"FROM python:3.12
STOPSIGNAL TERM
CMD ["python", "app.py"]"#;
        let result = check(dockerfile);

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Error),
            "F075: Short signal name TERM should be valid"
        );
    }
}
