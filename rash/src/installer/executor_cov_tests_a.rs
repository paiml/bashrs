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
fn test_EXCOV_021_postcondition_service_active() {
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
service_active = "nonexistent_service_xyz"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let executor = StepExecutor::new();
    let result = executor.execute_step(&spec.step[0]).unwrap();
    // Service doesn't exist → postcondition fails → step fails
    assert!(!result.success);
    let svc_check = result
        .postcondition_results
        .iter()
        .find(|r| r.check_type == "service_active");
    assert!(svc_check.is_some());
    assert!(!svc_check.unwrap().passed);
}

// ---------------------------------------------------------------------------
// check_service_active: error case (systemctl might not exist in CI)
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_022_check_service_active_details() {
    let executor = StepExecutor::new();
    let result = executor.check_service_active("nonexistent_svc");
    assert_eq!(result.check_type, "service_active");
    // Either "not active" (systemctl exists) or "Failed" (systemctl doesn't exist)
    assert!(
        result.details.contains("not active") || result.details.contains("Failed"),
        "Details: {}",
        result.details
    );
}

// ---------------------------------------------------------------------------
// Duration tracking
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_023_duration_tracked() {
    let executor = StepExecutor::new();
    let result = executor
        .execute_script("dur-test", "sh", "sleep 0.01")
        .unwrap();
    assert!(result.success);
    // Duration should be > 0 (at least 10ms sleep)
    assert!(
        result.duration_ms > 0,
        "Duration should be tracked: {}",
        result.duration_ms
    );
}

// ---------------------------------------------------------------------------
// Default trait implementation
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_024_default_implementation() {
    // StepExecutor::default() should work without panic
    let executor = StepExecutor::default();
    // Verify it works by running a dry-run-like test (default is not dry_run)
    let result = executor
        .execute_script("default-test", "sh", "echo ok")
        .unwrap();
    assert!(result.success, "Default executor should work");
}

// ---------------------------------------------------------------------------
// Debug trait derivation
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_025_debug_traits() {
    let result = StepExecutionResult {
        step_id: "test".to_string(),
        success: true,
        exit_code: Some(0),
        stdout: "out".to_string(),
        stderr: String::new(),
        duration_ms: 100,
        postcondition_results: vec![],
    };
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("test"), "Debug: {debug_str}");

    let postcond = PostconditionResult {
        check_type: "file_exists".to_string(),
        passed: true,
        details: "exists".to_string(),
    };
    let debug_str = format!("{:?}", postcond);
    assert!(debug_str.contains("file_exists"), "Debug: {debug_str}");

    let config = ExecutorConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("dry_run"), "Debug: {debug_str}");
}

// ---------------------------------------------------------------------------
// Clone trait derivation
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_026_clone_traits() {
    let result = StepExecutionResult {
        step_id: "test".to_string(),
        success: true,
        exit_code: Some(0),
        stdout: "out".to_string(),
        stderr: String::new(),
        duration_ms: 100,
        postcondition_results: vec![PostconditionResult {
            check_type: "file_exists".to_string(),
            passed: true,
            details: "exists".to_string(),
        }],
    };
    let cloned = result.clone();
    assert_eq!(cloned.step_id, result.step_id);
    assert_eq!(cloned.postcondition_results.len(), 1);

    let config = ExecutorConfig {
        dry_run: true,
        use_sudo: true,
        environment: {
            let mut m = HashMap::new();
            m.insert("K".to_string(), "V".to_string());
            m
        },
        working_dir: Some("/tmp".to_string()),
        timeout_secs: 60,
    };
    let cloned = config.clone();
    assert_eq!(cloned.dry_run, config.dry_run);
    assert_eq!(cloned.environment.len(), 1);
}
