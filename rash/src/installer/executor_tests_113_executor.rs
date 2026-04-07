
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
    use crate::installer::spec::InstallerSpec;

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

    let result = executor
        .execute_step(&spec.step[0])
        .expect("Should execute");

    assert!(result.success);
    assert!(result.stdout.contains("step executed"));
}

#[test]
fn test_113_execute_step_file_write() {
    use crate::installer::spec::InstallerSpec;

    let test_path = "/tmp/bashrs_test_113_step_file.txt";
    let _ = std::fs::remove_file(test_path);

    let toml = format!(
        r#"
[installer]
name = "test"
version = "1.0.0"

[[step]]
id = "test-file"
name = "Test File Write"
action = "file-write"
path = "{}"
content = "step file content"
"#,
        test_path
    );

    let spec = InstallerSpec::parse(&toml).expect("Valid TOML");
    let executor = StepExecutor::new();

    let result = executor
        .execute_step(&spec.step[0])
        .expect("Should execute");

    assert!(result.success);
    assert!(Path::new(test_path).exists());

    let content = std::fs::read_to_string(test_path).expect("Should read");
    assert_eq!(content, "step file content");

    let _ = std::fs::remove_file(test_path);
}

#[test]
fn test_113_execute_step_unknown_action() {
    use crate::installer::spec::InstallerSpec;

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

    let result = executor
        .execute_step(&spec.step[0])
        .expect("Should not error");

    assert!(!result.success);
    assert!(result.stderr.contains("Unknown action"));
}

#[test]
fn test_113_postcondition_file_exists() {
    use crate::installer::spec::InstallerSpec;

    let test_path = "/tmp/bashrs_test_113_postcond.txt";

    // Create the file first
    std::fs::write(test_path, "test").expect("Should write");

    let toml = format!(
        r#"
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
"#,
        test_path
    );

    let spec = InstallerSpec::parse(&toml).expect("Valid TOML");
    let executor = StepExecutor::new();

    let result = executor
        .execute_step(&spec.step[0])
        .expect("Should execute");

    assert!(result.success);
    assert!(!result.postcondition_results.is_empty());
    assert!(result.postcondition_results[0].passed);

    let _ = std::fs::remove_file(test_path);
}

#[test]
fn test_113_postcondition_fails() {
    use crate::installer::spec::InstallerSpec;

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

    let result = executor
        .execute_step(&spec.step[0])
        .expect("Should execute");

    // Step should fail because postcondition fails
    assert!(!result.success);
    assert!(!result.postcondition_results.is_empty());
    assert!(!result.postcondition_results[0].passed);
}
