/// Test test_command with a shell script that has test functions.
#[test]
fn test_cov_test_command_with_tests() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test_suite.sh");
    fs::write(
        &file,
        r#"#!/bin/sh
# TEST: Basic echo test
# GIVEN: A simple command
# WHEN: We run it
# THEN: It should succeed
test_echo() {
    result=$(echo hello)
    [ "$result" = "hello" ]
}

# TEST: Another test
test_true() {
    true
}
"#,
    )
    .unwrap();

    let result = test_command(
        &file,
        crate::cli::args::TestOutputFormat::Human,
        false,
        None,
    );
    let _ = result;
}

/// Test test_command with detailed output.
#[test]
fn test_cov_test_command_detailed() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test_suite.sh");
    fs::write(
        &file,
        r#"#!/bin/sh
# TEST: Echo test
test_echo() {
    echo hello
}
"#,
    )
    .unwrap();

    let result = test_command(&file, crate::cli::args::TestOutputFormat::Human, true, None);
    let _ = result;
}

/// Test test_command with JSON output.
#[test]
fn test_cov_test_command_json() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test_suite.sh");
    fs::write(
        &file,
        r#"#!/bin/sh
test_basic() {
    true
}
"#,
    )
    .unwrap();

    let result = test_command(&file, crate::cli::args::TestOutputFormat::Json, false, None);
    let _ = result;
}

/// Test test_command with JUnit output.
#[test]
fn test_cov_test_command_junit() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test_suite.sh");
    fs::write(
        &file,
        r#"#!/bin/sh
test_basic() {
    true
}
"#,
    )
    .unwrap();

    let result = test_command(
        &file,
        crate::cli::args::TestOutputFormat::Junit,
        false,
        None,
    );
    let _ = result;
}

/// Test test_command with pattern filter.
#[test]
fn test_cov_test_command_with_pattern() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test_suite.sh");
    fs::write(
        &file,
        r#"#!/bin/sh
test_echo() {
    echo hello
}
test_math() {
    [ "$(expr 1 + 1)" = "2" ]
}
"#,
    )
    .unwrap();

    let result = test_command(
        &file,
        crate::cli::args::TestOutputFormat::Human,
        false,
        Some("echo"),
    );
    let _ = result;
}

/// Test test_command with pattern that matches nothing.
#[test]
fn test_cov_test_command_pattern_no_match() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test_suite.sh");
    fs::write(
        &file,
        r#"#!/bin/sh
test_echo() {
    echo hello
}
"#,
    )
    .unwrap();

    let result = test_command(
        &file,
        crate::cli::args::TestOutputFormat::Human,
        false,
        Some("nonexistent_pattern"),
    );
    // Should succeed (0 tests is not an error, just a warning)
    assert!(result.is_ok());
}

/// Test test_command with script that has no tests.
#[test]
fn test_cov_test_command_no_tests() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("empty.sh");
    fs::write(&file, "#!/bin/sh\necho 'no tests here'\n").unwrap();

    let result = test_command(
        &file,
        crate::cli::args::TestOutputFormat::Human,
        false,
        None,
    );
    assert!(result.is_ok());
}

/// Test test_command with nonexistent file.
#[test]
fn test_cov_test_command_nonexistent_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("nonexistent.sh");

    let result = test_command(
        &file,
        crate::cli::args::TestOutputFormat::Human,
        false,
        None,
    );
    assert!(result.is_err());
}

/// Test print_human_test_results directly.
#[test]
fn test_cov_print_human_test_results_empty() {
    let report = crate::bash_quality::testing::TestReport {
        tests: vec![],
        results: vec![],
        duration_ms: 0,
    };
    super::test_commands::print_human_test_results(&report, false);
}

/// Test print_json_test_results directly.
#[test]
fn test_cov_print_json_test_results_empty() {
    let report = crate::bash_quality::testing::TestReport {
        tests: vec![],
        results: vec![],
        duration_ms: 0,
    };
    super::test_commands::print_json_test_results(&report);
}

/// Test print_junit_test_results directly.
#[test]
fn test_cov_print_junit_test_results_empty() {
    let report = crate::bash_quality::testing::TestReport {
        tests: vec![],
        results: vec![],
        duration_ms: 0,
    };
    super::test_commands::print_junit_test_results(&report);
}

/// Test print_test_summary directly.
#[test]
fn test_cov_print_test_summary_empty() {
    let report = crate::bash_quality::testing::TestReport {
        tests: vec![],
        results: vec![],
        duration_ms: 42,
    };
    super::test_commands::print_test_summary(&report);
}

/// Test print_test_detail with nonexistent test name.
#[test]
fn test_cov_print_test_detail_missing() {
    let report = crate::bash_quality::testing::TestReport {
        tests: vec![],
        results: vec![],
        duration_ms: 0,
    };
    // Should be a no-op (test not found)
    super::test_commands::print_test_detail(&report, "nonexistent", true);
}

// ============================================================================
// Additional Lint Edge Cases
// ============================================================================

/// Test lint_command with .bashrsignore file.
#[test]
fn test_cov_lint_with_bashrsignore() {
    let dir = TempDir::new().unwrap();

    // Create a script with only warning-level issues (not errors),
    // so even if the ignore pattern doesn't match the full path,
    // CI mode with fail_on=Error won't call process::exit.
    let file = dir.path().join("test.sh");
    fs::write(&file, "#!/bin/sh\necho $HOME\n").unwrap();

    // Create a .bashrsignore that ignores this file
    let ignore_file = dir.path().join(".bashrsignore");
    fs::write(&ignore_file, "test.sh\n").unwrap();

    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Human,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: false,
        ignore_file_path: Some(&ignore_file),
        quiet: false,
        level: crate::cli::args::LintLevel::Info,
        ignore_rules: None,
        exclude_rules: None,
        citl_export_path: None,
        profile: crate::cli::args::LintProfileArg::Standard,
        ci: true,
        fail_on: crate::cli::args::LintLevel::Error,
    };
    let _ = lint_command(opts);
}

/// Test lint_command with devcontainer profile.
#[test]
fn test_cov_lint_devcontainer_profile() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("Dockerfile");
    fs::write(&file, "FROM ubuntu:22.04\nRUN apt-get update\n").unwrap();

    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Human,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: true,
        ignore_file_path: None,
        quiet: false,
        level: crate::cli::args::LintLevel::Info,
        ignore_rules: None,
        exclude_rules: None,
        citl_export_path: None,
        profile: crate::cli::args::LintProfileArg::DevContainer,
        ci: true,
        fail_on: crate::cli::args::LintLevel::Error,
    };
    let _ = lint_command(opts);
}
