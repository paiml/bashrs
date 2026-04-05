fn test_EXCOV_015_apt_install_real_no_sudo() {
    let executor = StepExecutor::new();
    let packages = vec!["nonexistent_package_xyz".to_string()];
    let result = executor.execute_apt_install("apt-test", &packages);
    // May fail due to apt-get not being available or permissions
    // The key is it doesn't panic and returns a valid StepExecutionResult
    assert!(result.is_ok(), "Should return Ok, not panic");
}

// ---------------------------------------------------------------------------
// execute_apt_install: with sudo flag
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_016_apt_install_dry_run_with_sudo() {
    let executor = StepExecutor::with_config(ExecutorConfig {
        dry_run: true,
        use_sudo: true,
        ..Default::default()
    });
    let packages = vec!["vim".to_string()];
    let result = executor.execute_apt_install("sudo-apt", &packages).unwrap();
    assert!(result.success);
    assert!(result.stdout.contains("[DRY-RUN]"));
    assert!(result.stdout.contains("vim"));
}

// ---------------------------------------------------------------------------
// execute_user_group: real execution (will likely fail but tests path)
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_017_user_group_real() {
    let executor = StepExecutor::new();
    let result =
        executor.execute_user_group("ug-test", "nonexistent_user_xyz", "nonexistent_group_xyz");
    // Will fail due to invalid user/group but tests the non-dry-run path
    assert!(result.is_ok(), "Should return Ok, not panic");
    let r = result.unwrap();
    assert!(!r.success, "Should fail for non-existent user");
}

// ---------------------------------------------------------------------------
// execute_user_group: with sudo flag dry-run
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_018_user_group_dry_run_with_sudo() {
    let executor = StepExecutor::with_config(ExecutorConfig {
        dry_run: true,
        use_sudo: true,
        ..Default::default()
    });
    let result = executor
        .execute_user_group("sudo-ug", "deploy", "docker")
        .unwrap();
    assert!(result.success);
    assert!(result.stdout.contains("[DRY-RUN]"));
}

// ---------------------------------------------------------------------------
// execute_step: apt-install with empty packages via spec
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_019_execute_step_apt_empty_packages() {
    let toml = r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "s"
name = "S"
action = "apt-install"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let executor = StepExecutor::new();
    let result = executor.execute_step(&spec.step[0]).unwrap();
    assert!(result.success, "Empty packages should succeed");
    assert!(
        result.stdout.contains("No packages"),
        "Stdout: {}",
        result.stdout
    );
}

// ---------------------------------------------------------------------------
// execute_script: interpreter that doesn't exist
// ---------------------------------------------------------------------------

#[test]
fn test_EXCOV_020_script_bad_interpreter() {
    let executor = StepExecutor::new();
    let result = executor.execute_script("bad-interp", "nonexistent_interpreter_xyz", "echo hi");
    assert!(result.is_err(), "Should error for missing interpreter");
}

// ---------------------------------------------------------------------------
// execute_step: script with postcondition service_active (will fail)
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
