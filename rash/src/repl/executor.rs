//! REPL Command Execution Module
//!
//! Task: REPL-008-001 - Normal mode command execution
//! Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//!
//! Quality targets:
//! - Unit tests: 12+ scenarios
//! - Integration tests: Command execution workflows
//! - Mutation score: ≥90%
//! - Complexity: <10 per function

use std::process::{Command, Stdio};

/// Result of executing a bash command
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResult {
    /// Standard output from the command
    pub stdout: String,
    /// Standard error from the command
    pub stderr: String,
    /// Exit code (0 = success)
    pub exit_code: i32,
    /// Whether execution was successful
    pub success: bool,
}

impl ExecutionResult {
    /// Format the execution result for display
    pub fn format(&self) -> String {
        let mut output = String::new();

        // Add stdout if present
        if !self.stdout.is_empty() {
            output.push_str(&self.stdout);
            if !self.stdout.ends_with('\n') {
                output.push('\n');
            }
        }

        // Add stderr if present
        if !self.stderr.is_empty() {
            output.push_str(&self.stderr);
            if !self.stderr.ends_with('\n') {
                output.push('\n');
            }
        }

        // Add exit code if non-zero
        if !self.success {
            output.push_str(&format!("Exit code: {}\n", self.exit_code));
        }

        output
    }
}

/// Execute a bash command in a shell
///
/// Executes the command using `bash -c` and captures stdout, stderr, and exit code.
///
/// # Safety
/// This function executes arbitrary shell commands. It should only be used in
/// interactive contexts where the user controls the input.
///
/// # Examples
///
/// ```rust,no_run
/// use bashrs::repl::executor::execute_command;
///
/// let result = execute_command("echo hello");
/// assert!(result.success);
/// assert_eq!(result.stdout.trim(), "hello");
/// ```
pub fn execute_command(command: &str) -> ExecutionResult {
    // Use bash to execute the command
    let output = Command::new("bash")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);
            let success = output.status.success();

            ExecutionResult {
                stdout,
                stderr,
                exit_code,
                success,
            }
        }
        Err(e) => {
            // Failed to execute bash itself
            ExecutionResult {
                stdout: String::new(),
                stderr: format!("Failed to execute command: {}", e),
                exit_code: -1,
                success: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== RED PHASE: Unit Tests (These should PASS with implementation) =====

    #[test]
    fn test_REPL_008_001_execute_echo_simple() {
        let result = execute_command("echo hello");

        assert!(result.success);
        assert_eq!(result.stdout.trim(), "hello");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_REPL_008_001_execute_echo_multiple_words() {
        let result = execute_command("echo hello world");

        assert!(result.success);
        assert_eq!(result.stdout.trim(), "hello world");
    }

    #[test]
    fn test_REPL_008_001_execute_pwd() {
        let result = execute_command("pwd");

        assert!(result.success);
        assert!(!result.stdout.is_empty());
        assert!(result.stdout.contains('/'));
    }

    #[test]
    fn test_REPL_008_001_execute_true_command() {
        let result = execute_command("true");

        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "");
    }

    #[test]
    fn test_REPL_008_001_execute_false_command() {
        let result = execute_command("false");

        assert!(!result.success);
        assert_eq!(result.exit_code, 1);
    }

    #[test]
    fn test_REPL_008_001_execute_nonexistent_command() {
        let result = execute_command("nonexistent_command_xyz_12345");

        assert!(!result.success);
        assert!(!result.stderr.is_empty());
        assert!(result.stderr.contains("not found") || result.stderr.contains("command not found"));
    }

    #[test]
    fn test_REPL_008_001_execute_with_pipe() {
        let result = execute_command("echo hello world | head -1");

        assert!(result.success);
        assert_eq!(result.stdout.trim(), "hello world");
    }

    #[test]
    fn test_REPL_008_001_execute_with_error_output() {
        let result = execute_command("ls /nonexistent_directory_xyz_12345");

        assert!(!result.success);
        assert!(!result.stderr.is_empty());
        assert!(result.stderr.contains("No such file"));
    }

    #[test]
    fn test_REPL_008_001_format_success_output() {
        let result = ExecutionResult {
            stdout: "hello\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        };

        let formatted = result.format();
        assert_eq!(formatted, "hello\n");
    }

    #[test]
    fn test_REPL_008_001_format_error_output() {
        let result = ExecutionResult {
            stdout: String::new(),
            stderr: "error message\n".to_string(),
            exit_code: 1,
            success: false,
        };

        let formatted = result.format();
        assert!(formatted.contains("error message"));
        assert!(formatted.contains("Exit code: 1"));
    }

    #[test]
    fn test_REPL_008_001_format_mixed_output() {
        let result = ExecutionResult {
            stdout: "stdout line\n".to_string(),
            stderr: "stderr line\n".to_string(),
            exit_code: 2,
            success: false,
        };

        let formatted = result.format();
        assert!(formatted.contains("stdout line"));
        assert!(formatted.contains("stderr line"));
        assert!(formatted.contains("Exit code: 2"));
    }

    #[test]
    fn test_REPL_008_001_execute_date_command() {
        let result = execute_command("date +%Y");

        assert!(result.success);
        assert!(result.stdout.trim().starts_with("202")); // Year 202X
    }

    #[test]
    fn test_REPL_008_001_execute_empty_command() {
        let result = execute_command("");

        assert!(result.success); // Empty command succeeds
        assert_eq!(result.stdout, "");
    }

    #[test]
    fn test_REPL_008_001_execute_special_characters() {
        let result = execute_command("echo 'hello!@#$%'");

        assert!(result.success);
        assert_eq!(result.stdout.trim(), "hello!@#$%");
    }
}
