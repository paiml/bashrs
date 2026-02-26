//! Mock Command Executor for Testing Quality Gates
//!
//! This module provides infrastructure for mocking external command execution
//! to enable unit testing of quality gate logic without running actual commands.
//!
//! # Design
//!
//! Uses trait-based dependency injection to allow:
//! - Real execution in production (`RealCommandExecutor`)
//! - Mock execution in tests (`MockCommandExecutor`)
//!
//! # Example
//!
//! ```ignore
//! let mut mock = MockCommandExecutor::new();
//! mock.register("cargo", &["clippy"], CommandResult::success("Clippy passed"));
//! let result = mock.execute("cargo", &["clippy"]);
//! assert!(result.success);
//! ```

use std::collections::HashMap;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

/// Result of command execution
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Whether the command succeeded (exit code 0)
    pub success: bool,
    /// Exit code (0 for success)
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
}

impl CommandResult {
    /// Create a successful result
    pub fn success(stdout: &str) -> Self {
        Self {
            success: true,
            exit_code: 0,
            stdout: stdout.to_string(),
            stderr: String::new(),
        }
    }

    /// Create a failed result
    pub fn failure(stderr: &str, exit_code: i32) -> Self {
        Self {
            success: false,
            exit_code,
            stdout: String::new(),
            stderr: stderr.to_string(),
        }
    }

    /// Create a result with both stdout and stderr
    pub fn with_output(success: bool, stdout: &str, stderr: &str) -> Self {
        Self {
            success,
            exit_code: i32::from(!success),
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
        }
    }
}

impl From<Output> for CommandResult {
    fn from(output: Output) -> Self {
        Self {
            success: output.status.success(),
            exit_code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }
}

/// Trait for executing external commands
///
/// Implementations can execute real commands or return mock results.
pub trait CommandExecutor: Send + Sync {
    /// Execute a command with arguments
    fn execute(&self, program: &str, args: &[&str]) -> Result<CommandResult, String>;
}

/// Real command executor using std::process::Command
#[derive(Debug, Default)]
pub struct RealCommandExecutor;

impl CommandExecutor for RealCommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<CommandResult, String> {
        Command::new(program)
            .args(args)
            .output()
            .map(CommandResult::from)
            .map_err(|e| format!("Failed to execute {}: {}", program, e))
    }
}

/// Key for identifying a command (program + args combination)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandKey {
    pub program: String,
    pub args: Vec<String>,
}

impl CommandKey {
    pub fn new(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Mock command executor for testing
///
/// Allows registering expected commands and their results.
#[derive(Debug, Default, Clone)]
pub struct MockCommandExecutor {
    /// Registered command results
    results: Arc<Mutex<HashMap<CommandKey, CommandResult>>>,
    /// Fallback result for unregistered commands
    fallback: Option<CommandResult>,
    /// Track executed commands for verification
    executions: Arc<Mutex<Vec<CommandKey>>>,
}

impl MockCommandExecutor {
    /// Create a new mock executor
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a mock executor with a fallback result
    pub fn with_fallback(result: CommandResult) -> Self {
        Self {
            fallback: Some(result),
            ..Default::default()
        }
    }

    /// Register an expected command and its result
    pub fn register(&mut self, program: &str, args: &[&str], result: CommandResult) {
        let key = CommandKey::new(program, args);
        if let Ok(mut guard) = self.results.lock() {
            guard.insert(key, result);
        }
    }

    /// Register a command that matches any args for the program
    pub fn register_program(&mut self, program: &str, result: CommandResult) {
        // Register with empty args as a wildcard
        let key = CommandKey::new(program, &[]);
        if let Ok(mut guard) = self.results.lock() {
            guard.insert(key, result);
        }
    }

    /// Get all executed commands (for test verification)
    pub fn get_executions(&self) -> Vec<CommandKey> {
        self.executions
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    /// Verify a command was executed
    pub fn was_executed(&self, program: &str, args: &[&str]) -> bool {
        let key = CommandKey::new(program, args);
        self.executions
            .lock()
            .map(|guard| guard.contains(&key))
            .unwrap_or(false)
    }
}

impl CommandExecutor for MockCommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<CommandResult, String> {
        let key = CommandKey::new(program, args);

        // Record execution
        if let Ok(mut guard) = self.executions.lock() {
            guard.push(key.clone());
        }

        // Try exact match first
        let results = match self.results.lock() {
            Ok(guard) => guard,
            Err(_) => return Err("Failed to acquire lock".to_string()),
        };
        if let Some(result) = results.get(&key) {
            return Ok(result.clone());
        }

        // Try program-only match (wildcard)
        let program_key = CommandKey::new(program, &[]);
        if let Some(result) = results.get(&program_key) {
            return Ok(result.clone());
        }

        // Use fallback if available
        if let Some(ref fallback) = self.fallback {
            return Ok(fallback.clone());
        }

        Err(format!("No mock registered for: {} {:?}", program, args))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_result_success() {
        let result = CommandResult::success("output");
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "output");
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn test_command_result_failure() {
        let result = CommandResult::failure("error message", 1);
        assert!(!result.success);
        assert_eq!(result.exit_code, 1);
        assert!(result.stdout.is_empty());
        assert_eq!(result.stderr, "error message");
    }

    #[test]
    fn test_command_result_with_output() {
        let result = CommandResult::with_output(true, "stdout", "stderr");
        assert!(result.success);
        assert_eq!(result.stdout, "stdout");
        assert_eq!(result.stderr, "stderr");
    }

    #[test]
    fn test_mock_executor_register_and_execute() {
        let mut mock = MockCommandExecutor::new();
        mock.register("echo", &["hello"], CommandResult::success("hello\n"));

        let result = mock.execute("echo", &["hello"]).expect("should succeed");
        assert!(result.success);
        assert_eq!(result.stdout, "hello\n");
    }

    #[test]
    fn test_mock_executor_unregistered_command() {
        let mock = MockCommandExecutor::new();
        let result = mock.execute("unknown", &["arg"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No mock registered"));
    }

    #[test]
    fn test_mock_executor_fallback() {
        let mock = MockCommandExecutor::with_fallback(CommandResult::success("fallback"));
        let result = mock.execute("any", &["command"]).expect("should succeed");
        assert!(result.success);
        assert_eq!(result.stdout, "fallback");
    }

    #[test]
    fn test_mock_executor_execution_tracking() {
        let mock = MockCommandExecutor::with_fallback(CommandResult::success("ok"));

        mock.execute("cmd1", &["a", "b"]).expect("should succeed");
        mock.execute("cmd2", &["c"]).expect("should succeed");

        assert!(mock.was_executed("cmd1", &["a", "b"]));
        assert!(mock.was_executed("cmd2", &["c"]));
        assert!(!mock.was_executed("cmd3", &[]));
    }

    #[test]
    fn test_mock_executor_program_wildcard() {
        let mut mock = MockCommandExecutor::new();
        mock.register_program("cargo", CommandResult::success("cargo output"));

        // Should match any cargo command
        let result = mock.execute("cargo", &["build"]).expect("should succeed");
        assert!(result.success);

        let result = mock.execute("cargo", &["test"]).expect("should succeed");
        assert!(result.success);
    }

    #[test]
    fn test_command_key_equality() {
        let key1 = CommandKey::new("cargo", &["test"]);
        let key2 = CommandKey::new("cargo", &["test"]);
        let key3 = CommandKey::new("cargo", &["build"]);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_real_executor_echo() {
        let executor = RealCommandExecutor;
        let result = executor.execute("echo", &["test"]);

        // This actually runs echo
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("test"));
    }
}
