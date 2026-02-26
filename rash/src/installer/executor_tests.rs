#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for installer/executor.rs
//! Targets uncovered branches in: StepExecutionResult, PostconditionResult,
//! ExecutorConfig, StepExecutor dry-run paths, execute_step dispatch,
//! and postcondition checking.

use super::*;
use crate::installer::spec::InstallerSpec;
use std::collections::HashMap;

#[test]
fn test_COV_EXEC_001_step_execution_result_fields() {
    let ok = StepExecutionResult {
        step_id: "s1".to_string(),
        success: true,
        exit_code: Some(0),
        stdout: "done".to_string(),
        stderr: String::new(),
        duration_ms: 1234,
        postcondition_results: vec![],
    };
    assert!(ok.success && ok.exit_code == Some(0));

    let fail = StepExecutionResult {
        step_id: "s2".to_string(),
        success: false,
        exit_code: Some(127),
        stdout: String::new(),
        stderr: "not found".to_string(),
        duration_ms: 50,
        postcondition_results: vec![],
    };
    assert!(!fail.success);

    let no_code = StepExecutionResult {
        step_id: "s3".to_string(),
        success: false,
        exit_code: None,
        stdout: String::new(),
        stderr: "signal".to_string(),
        duration_ms: 0,
        postcondition_results: vec![],
    };
    assert!(no_code.exit_code.is_none());
}

#[test]
fn test_COV_EXEC_002_postcondition_result() {
    let pass = PostconditionResult {
        check_type: "file_exists".to_string(),
        passed: true,
        details: "exists".to_string(),
    };
    assert!(pass.passed);

    let fail = PostconditionResult {
        check_type: "command_succeeds".to_string(),
        passed: false,
        details: "exit 1".to_string(),
    };
    assert!(!fail.passed);
}

#[test]
fn test_COV_EXEC_003_executor_config() {
    let default = ExecutorConfig::default();
    assert!(!default.dry_run && !default.use_sudo && default.timeout_secs == 0);

    let custom = ExecutorConfig {
        dry_run: false,
        use_sudo: true,
        environment: {
            let mut e = HashMap::new();
            e.insert("K".to_string(), "V".to_string());
            e
        },
        working_dir: Some("/tmp".to_string()),
        timeout_secs: 300,
    };
    assert!(custom.use_sudo && custom.environment.len() == 1);
}

#[test]
fn test_COV_EXEC_004_executor_creation() {
    assert!(!StepExecutor::default().config.dry_run);
    let cfg = ExecutorConfig {
        dry_run: true,
        use_sudo: true,
        ..Default::default()
    };
    let ex = StepExecutor::with_config(cfg);
    assert!(ex.config.dry_run && ex.config.use_sudo);
}

#[test]
fn test_COV_EXEC_005_dry_run_script() {
    let ex = StepExecutor::with_config(ExecutorConfig {
        dry_run: true,
        ..Default::default()
    });
    let r = ex.execute_script("s1", "bash", "apt-get update").unwrap();
    assert!(r.success && r.stdout.contains("[DRY-RUN]") && r.stdout.contains("bash"));
    assert_eq!(r.step_id, "s1");
    assert_eq!(r.duration_ms, 0);
}

#[test]
fn test_COV_EXEC_006_dry_run_apt_install() {
    let ex = StepExecutor::with_config(ExecutorConfig {
        dry_run: true,
        ..Default::default()
    });
    let pkgs = vec!["curl".to_string(), "wget".to_string()];
    let r = ex.execute_apt_install("inst", &pkgs).unwrap();
    assert!(r.stdout.contains("curl") && r.stdout.contains("wget"));

    // Empty packages: no dry run needed
    let r2 = StepExecutor::new()
        .execute_apt_install("empty", &[])
        .unwrap();
    assert!(r2.success && r2.stdout.contains("No packages"));
}

#[test]
fn test_COV_EXEC_007_dry_run_file_write() {
    let ex = StepExecutor::with_config(ExecutorConfig {
        dry_run: true,
        ..Default::default()
    });
    let content = "[server]\nport = 8080";
    let r = ex
        .execute_file_write("w", "/etc/app.conf", content)
        .unwrap();
    assert!(
        r.stdout.contains("[DRY-RUN]") && r.stdout.contains(&format!("{} bytes", content.len()))
    );

    let r2 = ex.execute_file_write("w2", "/tmp/e.txt", "").unwrap();
    assert!(r2.stdout.contains("0 bytes"));
}

#[test]
fn test_COV_EXEC_008_dry_run_user_group() {
    let ex = StepExecutor::with_config(ExecutorConfig {
        dry_run: true,
        ..Default::default()
    });
    let r = ex.execute_user_group("ug", "deploy", "www-data").unwrap();
    assert!(r.stdout.contains("deploy") && r.stdout.contains("www-data"));
}

#[test]
fn test_COV_EXEC_009_execute_step_script_dry_run() {
    let toml = r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "s"
name = "S"
action = "script"
[step.script]
interpreter = "sh"
content = "echo ok"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let ex = StepExecutor::with_config(ExecutorConfig {
        dry_run: true,
        ..Default::default()
    });
    let r = ex.execute_step(&spec.step[0]).unwrap();
    assert!(r.success && r.stdout.contains("[DRY-RUN]"));
}

#[test]
fn test_COV_EXEC_010_execute_step_missing_script() {
    let toml = r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "s"
name = "S"
action = "script"
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let r = StepExecutor::new().execute_step(&spec.step[0]).unwrap();
    assert!(!r.success && r.stderr.contains("script content"));
}

#[test]
fn test_COV_EXEC_011_execute_step_apt_dry_run() {
    let toml = r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "s"
name = "S"
action = "apt-install"
packages = ["git"]
"#;
    let spec = InstallerSpec::parse(toml).unwrap();
    let ex = StepExecutor::with_config(ExecutorConfig {
        dry_run: true,
        ..Default::default()
    });
    assert!(ex.execute_step(&spec.step[0]).unwrap().success);
}

#[test]
fn test_COV_EXEC_012_execute_step_file_write_missing_fields() {
    // Missing path
    let toml1 = r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "s"
name = "S"
action = "file-write"
content = "x"
"#;
    let r1 = StepExecutor::new()
        .execute_step(&InstallerSpec::parse(toml1).unwrap().step[0])
        .unwrap();
    assert!(!r1.success && r1.stderr.contains("path and content"));

    // Missing content
    let toml2 = r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "s"
name = "S"
action = "file-write"
path = "/tmp/t.txt"
"#;
    let r2 = StepExecutor::new()
        .execute_step(&InstallerSpec::parse(toml2).unwrap().step[0])
        .unwrap();
    assert!(!r2.success && r2.stderr.contains("path and content"));
}

#[test]
fn test_COV_EXEC_013_execute_step_user_group_missing() {
    let toml1 = r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "s"
name = "S"
action = "user-add-to-group"
group = "docker"
"#;
    let r1 = StepExecutor::new()
        .execute_step(&InstallerSpec::parse(toml1).unwrap().step[0])
        .unwrap();
    assert!(r1.stderr.contains("user and group"));

    let toml2 = r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "s"
name = "S"
action = "user-add-to-group"
user = "deploy"
"#;
    let r2 = StepExecutor::new()
        .execute_step(&InstallerSpec::parse(toml2).unwrap().step[0])
        .unwrap();
    assert!(r2.stderr.contains("user and group"));
}

#[test]
fn test_COV_EXEC_014_execute_step_unknown_action() {
    let toml = r#"
[installer]
name = "t"
version = "1.0.0"
[[step]]
id = "s"
name = "S"
action = "deploy-to-mars"
"#;
    let r = StepExecutor::new()
        .execute_step(&InstallerSpec::parse(toml).unwrap().step[0])
        .unwrap();
    assert!(
        !r.success && r.stderr.contains("Unknown action") && r.stderr.contains("deploy-to-mars")
    );
}

#[test]
fn test_COV_EXEC_015_postcondition_file_exists_pass() {
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
"#,
        path
    );
    let spec = InstallerSpec::parse(&toml).unwrap();
    let r = StepExecutor::new().execute_step(&spec.step[0]).unwrap();
    assert!(r.success && r.postcondition_results[0].passed);
}

#[test]
fn test_COV_EXEC_016_postcondition_command_passes_and_fails() {
    let toml_pass = r#"
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
    let rp = StepExecutor::new()
        .execute_step(&InstallerSpec::parse(toml_pass).unwrap().step[0])
        .unwrap();
    assert!(rp.success);

    let toml_fail = r#"
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
command_succeeds = "test -f /nonexistent_xyz"
"#;
    let rf = StepExecutor::new()
        .execute_step(&InstallerSpec::parse(toml_fail).unwrap().step[0])
        .unwrap();
    assert!(!rf.success && rf.stderr.contains("Postcondition"));
}

#[test]
fn test_COV_EXEC_017_check_file_exists_messages() {
    let ex = StepExecutor::new();
    assert!(ex.check_file_exists("/tmp").details.contains("File exists"));
    assert!(ex
        .check_file_exists("/no/such")
        .details
        .contains("does not exist"));
}

#[test]
fn test_COV_EXEC_018_executor_with_env_dry_run() {
    let mut env = HashMap::new();
    env.insert("MY_VAR".to_string(), "val".to_string());
    let ex = StepExecutor::with_config(ExecutorConfig {
        dry_run: true,
        environment: env,
        working_dir: Some("/tmp".to_string()),
        ..Default::default()
    });
    assert!(
        ex.execute_script("e", "sh", "echo $MY_VAR")
            .unwrap()
            .success
    );
}
