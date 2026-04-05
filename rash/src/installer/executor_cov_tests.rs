//! Additional coverage tests for installer/executor.rs — targeting remaining uncovered branches
//!
//! Focuses on:
//! - execute_apt_install: real execution (non-dry-run) with sudo and without sudo
//! - execute_user_group: real execution with sudo and without sudo
//! - execute_file_write: real execution with parent dir creation, error on bad path
//! - check_command_succeeds: success details message, failure details message
//! - check_service_active: both pass and fail scenarios, error case
//! - check_postconditions: multiple postconditions, packages_absent
//! - run_command: with environment variables and working directory
//! - execute_step: file-write dry-run, user-add-to-group dry-run,
//!   postconditions with command_succeeds and service_active

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::installer::spec::InstallerSpec;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// execute_script: with environment variables
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_001_script_with_env() {
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "test_value".to_string());
    let executor = StepExecutor::with_config(ExecutorConfig {
        environment: env,
        ..Default::default()
    });
    let result = executor
        .execute_script("env-test", "sh", "echo $TEST_VAR")
        .unwrap();
    assert!(result.success, "Should succeed: {:?}", result);
    assert!(
        result.stdout.contains("test_value"),
        "Env var should be set: {}",
        result.stdout
    );
}

// ---------------------------------------------------------------------------
// execute_script: with working directory
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_002_script_with_working_dir() {
    let executor = StepExecutor::with_config(ExecutorConfig {
        working_dir: Some("/tmp".to_string()),
        ..Default::default()
    });
    let result = executor.execute_script("wd-test", "sh", "pwd").unwrap();
    assert!(result.success, "Should succeed: {:?}", result);
    assert!(
        result.stdout.contains("/tmp"),
        "Should be in /tmp: {}",
        result.stdout
    );
}

// ---------------------------------------------------------------------------
// execute_script: real execution that outputs to stderr
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_003_script_stderr_output() {
    let executor = StepExecutor::new();
    let result = executor
        .execute_script("stderr-test", "sh", "echo 'error msg' >&2")
        .unwrap();
    assert!(result.success);
    assert!(
        result.stderr.contains("error msg"),
        "Stderr: {}",
        result.stderr
    );
}

// ---------------------------------------------------------------------------
// execute_file_write: with nested parent directory creation
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_004_file_write_creates_parent_dirs() {
    let test_dir = "/tmp/bashrs_excov_004_nested/sub/dir";
    let test_path = format!("{}/test.txt", test_dir);

    // Clean up before test
    let _ = std::fs::remove_dir_all("/tmp/bashrs_excov_004_nested");

    let executor = StepExecutor::new();
    let result = executor
        .execute_file_write("nested-write", &test_path, "nested content")
        .unwrap();

    assert!(result.success, "Should succeed: {:?}", result);
    assert!(
        std::path::Path::new(&test_path).exists(),
        "File should exist"
    );
    let content = std::fs::read_to_string(&test_path).unwrap();
    assert_eq!(content, "nested content");

    // Clean up
    let _ = std::fs::remove_dir_all("/tmp/bashrs_excov_004_nested");
}

// ---------------------------------------------------------------------------
// execute_file_write: with empty parent (root-level file)
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_005_file_write_empty_parent() {
    // Writing to a path with no parent directory component
    let test_path = "/tmp/bashrs_excov_005.txt";
    let _ = std::fs::remove_file(test_path);

    let executor = StepExecutor::new();
    let result = executor
        .execute_file_write("root-write", test_path, "root content")
        .unwrap();

    assert!(result.success);
    assert!(result.stdout.contains("12 bytes"));

    let _ = std::fs::remove_file(test_path);
}

// ---------------------------------------------------------------------------
// check_command_succeeds: success message details
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_006_check_command_succeeds_details() {
    let executor = StepExecutor::new();

    let pass = executor.check_command_succeeds("true");
    assert!(pass.passed);
    assert!(
        pass.details.contains("Command succeeded"),
        "Details: {}",
        pass.details
    );

    let fail = executor.check_command_succeeds("false");
    assert!(!fail.passed);
    assert!(
        fail.details.contains("Command failed"),
        "Details: {}",
        fail.details
    );
}

// ---------------------------------------------------------------------------
// check_command_succeeds: with exit code in details
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_007_check_command_exit_code_in_details() {
    let executor = StepExecutor::new();
    let result = executor.check_command_succeeds("exit 42");
    assert!(!result.passed);
    assert!(
        result.details.contains("exit"),
        "Should mention exit: {}",
        result.details
    );
}

// ---------------------------------------------------------------------------
// check_service_active: inactive service
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_008_check_service_inactive() {
    let executor = StepExecutor::new();
    // Use a nonexistent service name to test the inactive path
    let result = executor.check_service_active("nonexistent_service_xyz_12345");
    assert!(!result.passed);
    assert_eq!(result.check_type, "service_active");
    assert!(
        result.details.contains("not active") || result.details.contains("Failed"),
        "Details: {}",
        result.details
    );
}

// ---------------------------------------------------------------------------
// check_postconditions: packages_absent
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_009_postconditions_packages_absent() {
    let toml = r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "s"
name = "S"
action = "script"
[step.script]
content = "true"
[step.postconditions]
packages_absent = ["nonexistent_pkg_xyz_12345"]
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let executor = StepExecutor::new();
    let result = executor.execute_step(&spec.step[0]).unwrap();
    // Package should be absent (not installed)
    assert!(result.success, "Should succeed since package is absent");
    let absent_check = result
        .postcondition_results
        .iter()
        .find(|r| r.check_type == "package_absent");
    assert!(absent_check.is_some(), "Should have package_absent check");
    assert!(
        absent_check.unwrap().passed,
        "Non-existent package should be absent"
    );
}

// ---------------------------------------------------------------------------
// execute_step: file-write dry-run via spec
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_010_execute_step_file_write_dry_run() {
    let toml = r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "fw"
name = "File Write"
action = "file-write"
path = "/tmp/bashrs_excov_010.txt"
content = "dry run content"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let executor = StepExecutor::with_config(ExecutorConfig {
        dry_run: true,
        ..Default::default()
    });
    let result = executor.execute_step(&spec.step[0]).unwrap();
    assert!(result.success);
    assert!(result.stdout.contains("[DRY-RUN]"));
}

// ---------------------------------------------------------------------------
// execute_step: user-add-to-group dry-run via spec
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_011_execute_step_user_group_dry_run() {
    let toml = r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "ug"
name = "User Group"
action = "user-add-to-group"
user = "testuser"
group = "docker"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let executor = StepExecutor::with_config(ExecutorConfig {
        dry_run: true,
        ..Default::default()
    });
    let result = executor.execute_step(&spec.step[0]).unwrap();
    assert!(result.success);
    assert!(result.stdout.contains("[DRY-RUN]"));
    assert!(result.stdout.contains("testuser"));
    assert!(result.stdout.contains("docker"));
}

// ---------------------------------------------------------------------------
// execute_step: postcondition command_succeeds via spec
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_012_execute_step_postcondition_command_succeeds() {
    let toml = r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "s"
name = "S"
action = "script"
[step.script]
content = "true"
[step.postconditions]
command_succeeds = "test -d /tmp"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let executor = StepExecutor::new();
    let result = executor.execute_step(&spec.step[0]).unwrap();
    assert!(result.success);
    let cmd_check = result
        .postcondition_results
        .iter()
        .find(|r| r.check_type == "command_succeeds");
    assert!(cmd_check.is_some());
    assert!(cmd_check.unwrap().passed);
}

// ---------------------------------------------------------------------------
// execute_step: postcondition command_succeeds fails → step fails
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_013_postcondition_command_fails_step_fails() {
    let toml = r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "s"
name = "S"
action = "script"
[step.script]
content = "true"
[step.postconditions]
command_succeeds = "test -f /nonexistent_xyz_12345"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let executor = StepExecutor::new();
    let result = executor.execute_step(&spec.step[0]).unwrap();
    assert!(!result.success, "Step should fail: {:?}", result);
    assert!(
        result.stderr.contains("Postcondition"),
        "Stderr: {}",
        result.stderr
    );
}

// ---------------------------------------------------------------------------
// execute_step: multiple postconditions (file_exists + command_succeeds)
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_014_multiple_postconditions() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    let path = temp.path().to_str().unwrap();
    let toml = format!(
        r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "s"
name = "S"
action = "script"
[step.script]
content = "true"
[step.postconditions]
file_exists = "{}"
command_succeeds = "true"
"#,
        path
    );
    let spec = InstallerSpec::parse(&toml).unwrap();
    let executor = StepExecutor::new();
    let result = executor.execute_step(&spec.step[0]).unwrap();
    assert!(result.success);
    assert_eq!(result.postcondition_results.len(), 2);
    assert!(result.postcondition_results.iter().all(|r| r.passed));
}

// ---------------------------------------------------------------------------
// execute_apt_install: real execution (without sudo) will fail (no apt-get usually for tests)
// but we test the non-dry-run path structure
// ---------------------------------------------------------------------------

#[test]

include!("executor_cov_tests_tests_EXCOV.rs");
