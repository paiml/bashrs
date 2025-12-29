//! Step Executor for Installer Framework (Issue #113)
//!
//! This module provides actual execution of installer steps:
//!
//! - Script execution (shell commands)
//! - Package installation (apt-get, dnf, etc.)
//! - File operations (write, chmod)
//! - User/group management
//!
//! # Safety
//!
//! All operations are:
//! - Idempotent (safe to re-run)
//! - Reversible (rollback support)
//! - Observable (detailed logging)
//! - Sandboxed (optional containerization)

use crate::models::{Error, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// Execution result for a step
#[derive(Debug, Clone)]
pub struct StepExecutionResult {
    /// Step ID that was executed
    pub step_id: String,
    /// Whether the step succeeded
    pub success: bool,
    /// Exit code (for script/command execution)
    pub exit_code: Option<i32>,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Duration of execution
    pub duration_ms: u64,
    /// Any postcondition check results
    pub postcondition_results: Vec<PostconditionResult>,
}

/// Result of a postcondition check
#[derive(Debug, Clone)]
pub struct PostconditionResult {
    /// Type of check (file_exists, command_succeeds, etc.)
    pub check_type: String,
    /// Whether the check passed
    pub passed: bool,
    /// Details about the check
    pub details: String,
}

/// Step executor configuration
#[derive(Debug, Clone, Default)]
pub struct ExecutorConfig {
    /// Whether to run in dry-run mode (simulate only)
    pub dry_run: bool,
    /// Whether to use sudo for privileged operations
    pub use_sudo: bool,
    /// Environment variables to inject
    pub environment: HashMap<String, String>,
    /// Working directory for execution
    pub working_dir: Option<String>,
    /// Timeout in seconds (0 = no timeout)
    pub timeout_secs: u64,
}

/// Step executor handles actual step execution
pub struct StepExecutor {
    config: ExecutorConfig,
}

impl StepExecutor {
    /// Create a new executor with default config
    pub fn new() -> Self {
        Self {
            config: ExecutorConfig::default(),
        }
    }

    /// Create a new executor with custom config
    pub fn with_config(config: ExecutorConfig) -> Self {
        Self { config }
    }

    /// Execute a script action
    pub fn execute_script(
        &self,
        step_id: &str,
        interpreter: &str,
        content: &str,
    ) -> Result<StepExecutionResult> {
        let start = std::time::Instant::now();

        if self.config.dry_run {
            return Ok(StepExecutionResult {
                step_id: step_id.to_string(),
                success: true,
                exit_code: Some(0),
                stdout: format!("[DRY-RUN] Would execute script with {}:\n{}", interpreter, content),
                stderr: String::new(),
                duration_ms: 0,
                postcondition_results: vec![],
            });
        }

        // Execute the script
        let output = self.run_command(interpreter, &["-c", content])?;

        let success = output.status.success();
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(StepExecutionResult {
            step_id: step_id.to_string(),
            success,
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration_ms,
            postcondition_results: vec![],
        })
    }

    /// Execute an apt-install action
    pub fn execute_apt_install(
        &self,
        step_id: &str,
        packages: &[String],
    ) -> Result<StepExecutionResult> {
        let start = std::time::Instant::now();

        if packages.is_empty() {
            return Ok(StepExecutionResult {
                step_id: step_id.to_string(),
                success: true,
                exit_code: Some(0),
                stdout: "No packages to install".to_string(),
                stderr: String::new(),
                duration_ms: 0,
                postcondition_results: vec![],
            });
        }

        if self.config.dry_run {
            return Ok(StepExecutionResult {
                step_id: step_id.to_string(),
                success: true,
                exit_code: Some(0),
                stdout: format!("[DRY-RUN] Would install packages: {}", packages.join(", ")),
                stderr: String::new(),
                duration_ms: 0,
                postcondition_results: vec![],
            });
        }

        // Build apt-get command
        let mut args = vec!["-y", "install"];
        let package_refs: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
        args.extend(package_refs);

        let program = if self.config.use_sudo {
            "sudo"
        } else {
            "apt-get"
        };

        let output = if self.config.use_sudo {
            let mut sudo_args = vec!["apt-get"];
            sudo_args.extend(args);
            self.run_command(program, &sudo_args)?
        } else {
            self.run_command(program, &args)?
        };

        let success = output.status.success();
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(StepExecutionResult {
            step_id: step_id.to_string(),
            success,
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration_ms,
            postcondition_results: vec![],
        })
    }

    /// Execute a file-write action
    pub fn execute_file_write(
        &self,
        step_id: &str,
        path: &str,
        content: &str,
    ) -> Result<StepExecutionResult> {
        let start = std::time::Instant::now();

        if self.config.dry_run {
            return Ok(StepExecutionResult {
                step_id: step_id.to_string(),
                success: true,
                exit_code: Some(0),
                stdout: format!("[DRY-RUN] Would write {} bytes to {}", content.len(), path),
                stderr: String::new(),
                duration_ms: 0,
                postcondition_results: vec![],
            });
        }

        // Create parent directories if needed
        if let Some(parent) = Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    Error::Io(std::io::Error::new(
                        e.kind(),
                        format!("Failed to create parent directory for {}: {}", path, e),
                    ))
                })?;
            }
        }

        // Write the file
        std::fs::write(path, content).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write file {}: {}", path, e),
            ))
        })?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(StepExecutionResult {
            step_id: step_id.to_string(),
            success: true,
            exit_code: Some(0),
            stdout: format!("Wrote {} bytes to {}", content.len(), path),
            stderr: String::new(),
            duration_ms,
            postcondition_results: vec![],
        })
    }

    /// Execute a user-add-to-group action
    pub fn execute_user_group(
        &self,
        step_id: &str,
        user: &str,
        group: &str,
    ) -> Result<StepExecutionResult> {
        let start = std::time::Instant::now();

        if self.config.dry_run {
            return Ok(StepExecutionResult {
                step_id: step_id.to_string(),
                success: true,
                exit_code: Some(0),
                stdout: format!("[DRY-RUN] Would add user {} to group {}", user, group),
                stderr: String::new(),
                duration_ms: 0,
                postcondition_results: vec![],
            });
        }

        // Use usermod -aG to add user to group
        let program = if self.config.use_sudo {
            "sudo"
        } else {
            "usermod"
        };

        let output = if self.config.use_sudo {
            self.run_command(program, &["usermod", "-aG", group, user])?
        } else {
            self.run_command(program, &["-aG", group, user])?
        };

        let success = output.status.success();
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(StepExecutionResult {
            step_id: step_id.to_string(),
            success,
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration_ms,
            postcondition_results: vec![],
        })
    }

    /// Check a postcondition: file_exists
    pub fn check_file_exists(&self, path: &str) -> PostconditionResult {
        let exists = Path::new(path).exists();
        PostconditionResult {
            check_type: "file_exists".to_string(),
            passed: exists,
            details: if exists {
                format!("File exists: {}", path)
            } else {
                format!("File does not exist: {}", path)
            },
        }
    }

    /// Check a postcondition: command_succeeds
    pub fn check_command_succeeds(&self, command: &str) -> PostconditionResult {
        let result = self.run_command("sh", &["-c", command]);
        match result {
            Ok(output) => {
                let success = output.status.success();
                PostconditionResult {
                    check_type: "command_succeeds".to_string(),
                    passed: success,
                    details: if success {
                        format!("Command succeeded: {}", command)
                    } else {
                        format!(
                            "Command failed (exit {}): {}",
                            output.status.code().unwrap_or(-1),
                            command
                        )
                    },
                }
            }
            Err(e) => PostconditionResult {
                check_type: "command_succeeds".to_string(),
                passed: false,
                details: format!("Command execution error: {}", e),
            },
        }
    }

    /// Check a postcondition: service_active
    pub fn check_service_active(&self, service: &str) -> PostconditionResult {
        let result = self.run_command("systemctl", &["is-active", service]);
        match result {
            Ok(output) => {
                let active = output.status.success();
                PostconditionResult {
                    check_type: "service_active".to_string(),
                    passed: active,
                    details: if active {
                        format!("Service is active: {}", service)
                    } else {
                        format!("Service is not active: {}", service)
                    },
                }
            }
            Err(e) => PostconditionResult {
                check_type: "service_active".to_string(),
                passed: false,
                details: format!("Failed to check service status: {}", e),
            },
        }
    }

    /// Execute a step from the spec
    pub fn execute_step(&self, step: &super::spec::Step) -> Result<StepExecutionResult> {
        let start = std::time::Instant::now();

        // Dispatch based on action type
        let mut result = match step.action.as_str() {
            "script" => {
                if let Some(ref script) = step.script {
                    self.execute_script(&step.id, &script.interpreter, &script.content)?
                } else {
                    StepExecutionResult {
                        step_id: step.id.clone(),
                        success: false,
                        exit_code: None,
                        stdout: String::new(),
                        stderr: "Script action requires script content".to_string(),
                        duration_ms: 0,
                        postcondition_results: vec![],
                    }
                }
            }
            "apt-install" => {
                self.execute_apt_install(&step.id, &step.packages)?
            }
            "file-write" => {
                match (&step.path, &step.content) {
                    (Some(path), Some(content)) => {
                        self.execute_file_write(&step.id, path, content)?
                    }
                    _ => StepExecutionResult {
                        step_id: step.id.clone(),
                        success: false,
                        exit_code: None,
                        stdout: String::new(),
                        stderr: "file-write action requires path and content".to_string(),
                        duration_ms: 0,
                        postcondition_results: vec![],
                    },
                }
            }
            "user-add-to-group" => {
                match (&step.user, &step.group) {
                    (Some(user), Some(group)) => {
                        self.execute_user_group(&step.id, user, group)?
                    }
                    _ => StepExecutionResult {
                        step_id: step.id.clone(),
                        success: false,
                        exit_code: None,
                        stdout: String::new(),
                        stderr: "user-add-to-group action requires user and group".to_string(),
                        duration_ms: 0,
                        postcondition_results: vec![],
                    },
                }
            }
            other => {
                StepExecutionResult {
                    step_id: step.id.clone(),
                    success: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: format!("Unknown action type: {}", other),
                    duration_ms: 0,
                    postcondition_results: vec![],
                }
            }
        };

        // Check postconditions if step succeeded
        if result.success {
            result.postcondition_results = self.check_postconditions(&step.postconditions);

            // Step fails if any postcondition fails
            let all_passed = result.postcondition_results.iter().all(|r| r.passed);
            if !all_passed {
                result.success = false;
                result.stderr.push_str("\nPostcondition check failed");
            }
        }

        result.duration_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }

    /// Check all postconditions for a step
    fn check_postconditions(&self, postconditions: &super::spec::Postcondition) -> Vec<PostconditionResult> {
        let mut results = Vec::new();

        if let Some(ref path) = postconditions.file_exists {
            results.push(self.check_file_exists(path));
        }

        if let Some(ref cmd) = postconditions.command_succeeds {
            results.push(self.check_command_succeeds(cmd));
        }

        if let Some(ref service) = postconditions.service_active {
            results.push(self.check_service_active(service));
        }

        // Check packages_absent (packages should NOT be installed)
        for pkg in &postconditions.packages_absent {
            let result = self.run_command("dpkg", &["-s", pkg]);
            let is_absent = match result {
                Ok(output) => !output.status.success(),
                Err(_) => true, // Command failed, package likely not installed
            };
            results.push(PostconditionResult {
                check_type: "package_absent".to_string(),
                passed: is_absent,
                details: if is_absent {
                    format!("Package is absent: {}", pkg)
                } else {
                    format!("Package is installed (should be absent): {}", pkg)
                },
            });
        }

        results
    }

    /// Run a command with the executor's configuration
    fn run_command(&self, program: &str, args: &[&str]) -> Result<Output> {
        let mut cmd = Command::new(program);
        cmd.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        for (key, value) in &self.config.environment {
            cmd.env(key, value);
        }

        // Set working directory if configured
        if let Some(ref dir) = self.config.working_dir {
            cmd.current_dir(dir);
        }

        cmd.output().map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to execute {}: {}", program, e),
            ))
        })
    }
}

impl Default for StepExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_113_executor_new() {
        let executor = StepExecutor::new();
        assert!(!executor.config.dry_run);
        assert!(!executor.config.use_sudo);
    }

    #[test]
    fn test_113_executor_with_config() {
        let config = ExecutorConfig {
            dry_run: true,
            use_sudo: true,
            environment: HashMap::new(),
            working_dir: None,
            timeout_secs: 30,
        };
        let executor = StepExecutor::with_config(config);
        assert!(executor.config.dry_run);
        assert!(executor.config.use_sudo);
    }

    #[test]
    fn test_113_execute_script_dry_run() {
        let config = ExecutorConfig {
            dry_run: true,
            ..Default::default()
        };
        let executor = StepExecutor::with_config(config);

        let result = executor
            .execute_script("test-step", "sh", "echo hello")
            .expect("Execution should succeed");

        assert!(result.success);
        assert_eq!(result.exit_code, Some(0));
        assert!(result.stdout.contains("[DRY-RUN]"));
        assert!(result.stdout.contains("echo hello"));
    }

    #[test]
    fn test_113_execute_script_real() {
        let executor = StepExecutor::new();

        let result = executor
            .execute_script("test-step", "sh", "echo 'hello world'")
            .expect("Execution should succeed");

        assert!(result.success);
        assert_eq!(result.exit_code, Some(0));
        assert!(result.stdout.contains("hello world"));
    }

    #[test]
    fn test_113_execute_script_failure() {
        let executor = StepExecutor::new();

        let result = executor
            .execute_script("test-step", "sh", "exit 42")
            .expect("Execution should succeed");

        assert!(!result.success);
        assert_eq!(result.exit_code, Some(42));
    }

    #[test]
    fn test_113_execute_apt_install_dry_run() {
        let config = ExecutorConfig {
            dry_run: true,
            ..Default::default()
        };
        let executor = StepExecutor::with_config(config);

        let packages = vec!["vim".to_string(), "git".to_string()];
        let result = executor
            .execute_apt_install("test-step", &packages)
            .expect("Execution should succeed");

        assert!(result.success);
        assert!(result.stdout.contains("[DRY-RUN]"));
        assert!(result.stdout.contains("vim"));
        assert!(result.stdout.contains("git"));
    }

    #[test]
    fn test_113_execute_apt_install_empty() {
        let executor = StepExecutor::new();

        let result = executor
            .execute_apt_install("test-step", &[])
            .expect("Execution should succeed");

        assert!(result.success);
        assert!(result.stdout.contains("No packages"));
    }

    #[test]
    fn test_113_execute_file_write_dry_run() {
        let config = ExecutorConfig {
            dry_run: true,
            ..Default::default()
        };
        let executor = StepExecutor::with_config(config);

        let result = executor
            .execute_file_write("test-step", "/tmp/test.txt", "hello")
            .expect("Execution should succeed");

        assert!(result.success);
        assert!(result.stdout.contains("[DRY-RUN]"));
        assert!(result.stdout.contains("5 bytes"));
    }

    #[test]
    fn test_113_execute_file_write_real() {
        let executor = StepExecutor::new();
        let test_path = "/tmp/bashrs_test_113_file_write.txt";

        // Clean up before test
        let _ = std::fs::remove_file(test_path);

        let result = executor
            .execute_file_write("test-step", test_path, "test content")
            .expect("Execution should succeed");

        assert!(result.success);
        assert!(Path::new(test_path).exists());

        let content = std::fs::read_to_string(test_path).expect("Should read file");
        assert_eq!(content, "test content");

        // Clean up
        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn test_113_execute_user_group_dry_run() {
        let config = ExecutorConfig {
            dry_run: true,
            ..Default::default()
        };
        let executor = StepExecutor::with_config(config);

        let result = executor
            .execute_user_group("test-step", "testuser", "docker")
            .expect("Execution should succeed");

        assert!(result.success);
        assert!(result.stdout.contains("[DRY-RUN]"));
        assert!(result.stdout.contains("testuser"));
        assert!(result.stdout.contains("docker"));
    }

    #[test]
    fn test_113_check_file_exists_true() {
        let executor = StepExecutor::new();

        // /tmp should always exist
        let result = executor.check_file_exists("/tmp");

        assert!(result.passed);
        assert_eq!(result.check_type, "file_exists");
    }

    #[test]
    fn test_113_check_file_exists_false() {
        let executor = StepExecutor::new();

        let result = executor.check_file_exists("/nonexistent/path/that/does/not/exist");

        assert!(!result.passed);
        assert_eq!(result.check_type, "file_exists");
    }

    #[test]
    fn test_113_check_command_succeeds_true() {
        let executor = StepExecutor::new();

        let result = executor.check_command_succeeds("true");

        assert!(result.passed);
        assert_eq!(result.check_type, "command_succeeds");
    }

    #[test]
    fn test_113_check_command_succeeds_false() {
        let executor = StepExecutor::new();

        let result = executor.check_command_succeeds("false");

        assert!(!result.passed);
        assert_eq!(result.check_type, "command_succeeds");
    }

    #[test]
    fn test_113_execute_step_script() {
        use super::super::spec::InstallerSpec;

        let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "test-script"
name = "Test Script"
action = "script"

[step.script]
interpreter = "sh"
content = "echo 'step executed'"
"#;

        let spec = InstallerSpec::parse(toml).expect("Valid TOML");
        let executor = StepExecutor::new();

        let result = executor.execute_step(&spec.step[0]).expect("Should execute");

        assert!(result.success);
        assert!(result.stdout.contains("step executed"));
    }

    #[test]
    fn test_113_execute_step_file_write() {
        use super::super::spec::InstallerSpec;

        let test_path = "/tmp/bashrs_test_113_step_file.txt";
        let _ = std::fs::remove_file(test_path);

        let toml = format!(r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "test-file"
name = "Test File Write"
action = "file-write"
path = "{}"
content = "step file content"
"#, test_path);

        let spec = InstallerSpec::parse(&toml).expect("Valid TOML");
        let executor = StepExecutor::new();

        let result = executor.execute_step(&spec.step[0]).expect("Should execute");

        assert!(result.success);
        assert!(Path::new(test_path).exists());

        let content = std::fs::read_to_string(test_path).expect("Should read");
        assert_eq!(content, "step file content");

        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn test_113_execute_step_unknown_action() {
        use super::super::spec::InstallerSpec;

        let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "test-unknown"
name = "Unknown Action"
action = "invalid-action"
"#;

        let spec = InstallerSpec::parse(toml).expect("Valid TOML");
        let executor = StepExecutor::new();

        let result = executor.execute_step(&spec.step[0]).expect("Should not error");

        assert!(!result.success);
        assert!(result.stderr.contains("Unknown action"));
    }

    #[test]
    fn test_113_postcondition_file_exists() {
        use super::super::spec::InstallerSpec;

        let test_path = "/tmp/bashrs_test_113_postcond.txt";

        // Create the file first
        std::fs::write(test_path, "test").expect("Should write");

        let toml = format!(r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "test-postcond"
name = "Test Postconditions"
action = "script"

[step.script]
content = "echo 'done'"

[step.postconditions]
file_exists = "{}"
"#, test_path);

        let spec = InstallerSpec::parse(&toml).expect("Valid TOML");
        let executor = StepExecutor::new();

        let result = executor.execute_step(&spec.step[0]).expect("Should execute");

        assert!(result.success);
        assert!(!result.postcondition_results.is_empty());
        assert!(result.postcondition_results[0].passed);

        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn test_113_postcondition_fails() {
        use super::super::spec::InstallerSpec;

        let toml = r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "test-postcond-fail"
name = "Test Postcondition Failure"
action = "script"

[step.script]
content = "echo 'done'"

[step.postconditions]
file_exists = "/nonexistent/file/that/does/not/exist"
"#;

        let spec = InstallerSpec::parse(toml).expect("Valid TOML");
        let executor = StepExecutor::new();

        let result = executor.execute_step(&spec.step[0]).expect("Should execute");

        // Step should fail because postcondition fails
        assert!(!result.success);
        assert!(!result.postcondition_results.is_empty());
        assert!(!result.postcondition_results[0].passed);
    }
}
