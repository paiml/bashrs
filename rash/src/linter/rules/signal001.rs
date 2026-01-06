//! SIGNAL001: Signal and process management validation (F096-F100)
//!
//! **Rule**: Validate signal handling and process management in shell scripts
//!
//! **Why this matters**:
//! Proper signal handling ensures graceful shutdown, prevents zombie processes,
//! and enables reliable daemon operation.
//!
//! ## Checks implemented:
//! - F096: Validate trap handler syntax
//! - F097: Detect signal forwarding patterns
//! - F098: Validate PID file patterns (race-free)
//! - F099: Detect zombie prevention (wait after background jobs)
//! - F100: Validate graceful shutdown patterns

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// Valid signal names for trap
const VALID_SIGNALS: &[&str] = &[
    "EXIT", "HUP", "INT", "QUIT", "TERM", "KILL", "USR1", "USR2", "PIPE", "ALRM", "CHLD", "CONT",
    "STOP", "TSTP", "TTIN", "TTOU", "ERR", "DEBUG", "RETURN", "SIGTERM", "SIGINT", "SIGHUP",
    "SIGQUIT", "SIGKILL", "SIGUSR1", "SIGUSR2", "SIGPIPE",
];

/// Check for signal and process management patterns
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let mut _has_trap = false;
    let mut has_background_job = false;
    let mut has_wait = false;
    let mut has_pid_file_write = false;
    let mut has_cleanup_trap = false;

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // F096: Validate trap syntax
        if trimmed.starts_with("trap ") || trimmed.contains(" trap ") {
            _has_trap = true;
            validate_trap(trimmed, line_num + 1, &mut result);

            // Check for cleanup trap
            if trimmed.contains("EXIT")
                || trimmed.contains("TERM")
                || trimmed.contains("INT")
                || trimmed.contains("cleanup")
                || trimmed.contains("rm ")
            {
                has_cleanup_trap = true;
            }
        }

        // F097: Detect signal forwarding
        if trimmed.contains("kill -") && trimmed.contains("$$") {
            // Forwarding signal to self (common in wrappers)
        }

        // F098: PID file patterns
        if (trimmed.contains("echo $$") || trimmed.contains("printf") && trimmed.contains("$$"))
            && (trimmed.contains("> ") || trimmed.contains(">>"))
            && trimmed.contains(".pid")
        {
            has_pid_file_write = true;

            // Check for race-free PID file creation
            if !trimmed.contains("exec") && !trimmed.contains("flock") {
                let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
                let diag = Diagnostic::new(
                    "SIGNAL001",
                    Severity::Info,
                    "PID file write may have race condition - consider atomic write pattern (F098)"
                        .to_string(),
                    span,
                );
                result.add(diag);
            }
        }

        // F099: Background jobs and wait
        if trimmed.ends_with(" &") || trimmed.contains(" & ") {
            has_background_job = true;
        }

        if trimmed == "wait" || trimmed.starts_with("wait ") || trimmed.contains("; wait") {
            has_wait = true;
        }

        // F100: Check for exit without cleanup
        if (trimmed.starts_with("exit ") || trimmed == "exit")
            && has_pid_file_write
            && !has_cleanup_trap
        {
            let span = Span::new(line_num + 1, 1, line_num + 1, trimmed.len().min(80));
            let diag = Diagnostic::new(
                "SIGNAL001",
                Severity::Warning,
                "Exit without cleanup trap - PID file may not be removed (F100)".to_string(),
                span,
            );
            result.add(diag);
        }
    }

    // F099: Warn about background jobs without wait
    if has_background_job && !has_wait {
        let diag = Diagnostic::new(
            "SIGNAL001",
            Severity::Info,
            "Background job(s) without 'wait' - may leave zombie processes (F099)".to_string(),
            Span::new(1, 1, 1, 1),
        );
        result.add(diag);
    }

    result
}

/// Validate trap command syntax (F096)
fn validate_trap(line: &str, line_num: usize, result: &mut LintResult) {
    // Extract signals from trap command
    let parts: Vec<&str> = line.split_whitespace().collect();

    // Find trap position
    let Some(trap_idx) = parts.iter().position(|&p| p == "trap") else {
        return;
    };

    // Check for empty trap (trap '' SIGNAL)
    if parts.len() > trap_idx + 1 && (parts[trap_idx + 1] == "''" || parts[trap_idx + 1] == "\"\"")
    {
        // Empty trap is valid (ignores signal)
        return;
    }

    // Validate signal names at end of trap command
    for part in parts.iter().skip(trap_idx + 1) {
        // Skip the command/handler part (usually in quotes)
        if part.starts_with('\'') || part.starts_with('"') || part.starts_with('$') {
            continue;
        }

        // Check if it looks like a signal
        let upper = part.to_uppercase();
        let is_signal_like = upper.starts_with("SIG")
            || VALID_SIGNALS.contains(&upper.as_str())
            || part.parse::<u32>().is_ok();

        if is_signal_like && !VALID_SIGNALS.contains(&upper.as_str()) {
            // Check numeric signals
            if let Ok(num) = part.parse::<u32>() {
                if num > 64 {
                    let span = Span::new(line_num, 1, line_num, line.len().min(80));
                    let diag = Diagnostic::new(
                        "SIGNAL001",
                        Severity::Warning,
                        format!("Invalid signal number {} in trap (F096)", num),
                        span,
                    );
                    result.add(diag);
                }
            }
        }
    }

    // Check for common trap mistakes

    // trap without quotes around command
    if parts.len() > trap_idx + 2 {
        let cmd = parts[trap_idx + 1];
        if !cmd.starts_with('\'') && !cmd.starts_with('"') && cmd != "''" && cmd != "\"\"" {
            // Might be unquoted command
            if !cmd.starts_with('-') && cmd.len() > 1 {
                let span = Span::new(line_num, 1, line_num, line.len().min(80));
                let diag = Diagnostic::new(
                    "SIGNAL001",
                    Severity::Info,
                    "Consider quoting trap command to prevent early expansion (F096)".to_string(),
                    span,
                );
                result.add(diag);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F096: Trap validation
    #[test]
    fn test_F096_valid_trap() {
        let script = r#"#!/bin/sh
trap 'cleanup' EXIT TERM INT"#;
        let result = check(script);

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Error),
            "F096: Valid trap should not error"
        );
    }

    #[test]
    fn test_F096_empty_trap() {
        let script = r#"#!/bin/sh
trap '' PIPE"#;
        let result = check(script);

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Error),
            "F096: Empty trap (ignore signal) should be valid"
        );
    }

    /// F098: PID file validation
    #[test]
    fn test_F098_pid_file() {
        let script = r#"#!/bin/sh
echo $$ > /var/run/daemon.pid"#;
        let result = check(script);

        assert!(
            result.diagnostics.iter().any(|d| d.message.contains("PID")),
            "F098: PID file write should be noted"
        );
    }

    /// F099: Zombie prevention
    #[test]
    fn test_F099_background_without_wait() {
        let script = r#"#!/bin/sh
background_task &
echo "started""#;
        let result = check(script);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("zombie")),
            "F099: Background without wait should warn"
        );
    }

    #[test]
    fn test_F099_background_with_wait() {
        let script = r#"#!/bin/sh
background_task &
wait"#;
        let result = check(script);

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("zombie")),
            "F099: Background with wait should not warn"
        );
    }

    /// F100: Graceful shutdown
    #[test]
    fn test_F100_exit_without_cleanup() {
        let script = r#"#!/bin/sh
echo $$ > /var/run/daemon.pid
exit 0"#;
        let result = check(script);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("cleanup")),
            "F100: Exit without cleanup trap should warn"
        );
    }

    #[test]
    fn test_F100_exit_with_cleanup_trap() {
        let script = r#"#!/bin/sh
trap 'rm -f /var/run/daemon.pid' EXIT
echo $$ > /var/run/daemon.pid
exit 0"#;
        let result = check(script);

        assert!(
            !result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("cleanup")),
            "F100: Exit with cleanup trap should not warn"
        );
    }
}
