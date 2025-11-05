// Exit code tests for bashrs lint command (Issue #6)
//
// Expected behavior (aligned with shellcheck, eslint, gcc):
// - Exit 0: No errors found (warnings/info are OK)
// - Exit 1: Errors found (actual lint failures)
// - Exit 2: Tool failure (invalid arguments, file not found)
//
// EXTREME TDD: Test-driven development for Issue #6
// https://github.com/paiml/bashrs/issues/6

use assert_cmd::Command;
use std::io::Write;
use tempfile::NamedTempFile;

/// Helper function to create bashrs command
fn bashrs_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
}

// ============================================================================
// RED Phase: Test_Issue_006_* - Exit Code Tests
// ============================================================================

/// Test: Exit 0 when no issues found
#[test]
fn test_issue_006_exit_0_no_issues() {
    // ARRANGE: Clean bash script with no issues
    let bash_code = r#"#!/bin/bash
# Clean script
echo "Hello, World"
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(bash_code.as_bytes()).unwrap();

    // ACT & ASSERT: Should exit 0 (success)
    bashrs_cmd()
        .arg("lint")
        .arg(file.path())
        .assert()
        .success(); // success() checks exit code 0
}

/// Test: Exit 0 when only warnings (no errors)
#[test]
fn test_issue_006_exit_0_warnings_only() {
    // ARRANGE: Script with warning (SC2086 - unquoted variable)
    // This should produce WARNING, not ERROR
    let bash_code = r#"#!/bin/bash
var="test"
echo $var
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(bash_code.as_bytes()).unwrap();

    // ACT & ASSERT: Should exit 0 (warnings are non-blocking)
    bashrs_cmd()
        .arg("lint")
        .arg(file.path())
        .assert()
        .success(); // Exit 0 for warnings only
}

/// Test: Exit 0 when only info messages (no errors)
#[test]
fn test_issue_006_exit_0_info_only() {
    // ARRANGE: Script that might produce INFO-level diagnostics
    let bash_code = r#"#!/bin/bash
# Script with potential style issues
echo "test"
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(bash_code.as_bytes()).unwrap();

    // ACT & ASSERT: Should exit 0 (info is non-blocking)
    bashrs_cmd()
        .arg("lint")
        .arg(file.path())
        .assert()
        .success(); // Exit 0 for info only
}

/// Test: Exit 1 when errors found
#[test]
fn test_issue_006_exit_1_errors_found() {
    // ARRANGE: Script with actual ERROR (SC2188: Redirection without command)
    let bash_code = r#"#!/bin/bash
# SC2188: Redirection without command (ERROR severity)
> output.txt
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(bash_code.as_bytes()).unwrap();

    // ACT & ASSERT: Should exit 1 (errors found)
    bashrs_cmd()
        .arg("lint")
        .arg(file.path())
        .assert()
        .failure() // Exit non-zero
        .code(1); // Specifically exit code 1
}

/// Test: Exit 1 when multiple errors found
#[test]
fn test_issue_006_exit_1_multiple_errors() {
    // ARRANGE: Script with multiple errors (SC2188)
    let bash_code = r#"#!/bin/bash
# Multiple redirection errors
> output1.txt
> output2.txt
echo $y  # WARNING (unquoted variable)
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(bash_code.as_bytes()).unwrap();

    // ACT & ASSERT: Should exit 1 (errors found, even with warnings)
    bashrs_cmd()
        .arg("lint")
        .arg(file.path())
        .assert()
        .failure()
        .code(1);
}

/// Test: Exit 1 when errors AND warnings (errors take precedence)
#[test]
fn test_issue_006_exit_1_errors_and_warnings() {
    // ARRANGE: Script with both errors and warnings
    let bash_code = r#"#!/bin/bash
var="test"
echo $var  # WARNING (unquoted variable)

> error.log  # ERROR (SC2188: Redirection without command)
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(bash_code.as_bytes()).unwrap();

    // ACT & ASSERT: Should exit 1 (errors present)
    bashrs_cmd()
        .arg("lint")
        .arg(file.path())
        .assert()
        .failure()
        .code(1);
}

/// Test: Exit 2 for tool failure (file not found)
#[test]
fn test_issue_006_exit_2_file_not_found() {
    // ARRANGE: Non-existent file

    // ACT & ASSERT: Should exit 2 (tool failure)
    bashrs_cmd()
        .arg("lint")
        .arg("/nonexistent/path/to/file.sh")
        .assert()
        .failure()
        .code(2); // Exit code 2 for tool failure
}

/// Test: Exit 2 for tool failure (invalid format argument)
#[test]
fn test_issue_006_exit_2_invalid_format() {
    // ARRANGE: Create a valid file
    let bash_code = "#!/bin/bash\necho 'test'\n";
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(bash_code.as_bytes()).unwrap();

    // ACT & ASSERT: Invalid --format argument should be tool failure
    bashrs_cmd()
        .arg("lint")
        .arg("--format")
        .arg("invalid-format")
        .arg(file.path())
        .assert()
        .failure()
        .code(2); // Exit code 2 for invalid arguments
}

// ============================================================================
// CI/CD Integration Tests
// ============================================================================

/// Test: CI/CD pipeline with warnings should pass (exit 0)
/// This is the critical use case from Issue #6
#[test]
fn test_issue_006_ci_cd_warnings_pass() {
    // ARRANGE: Typical CI/CD script with minor warnings
    let bash_code = r#"#!/bin/bash
# CI/CD deployment script
VERSION="1.0.0"
echo $VERSION  # WARNING: unquoted variable
deploy_to_production
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(bash_code.as_bytes()).unwrap();

    // ACT & ASSERT: CI/CD should pass with only warnings
    let output = bashrs_cmd()
        .arg("lint")
        .arg(file.path())
        .output()
        .unwrap();

    // Should exit 0 (warnings don't block CI/CD)
    assert_eq!(
        output.status.code(),
        Some(0),
        "CI/CD should pass with warnings. Exit code should be 0, got: {:?}",
        output.status.code()
    );
}

/// Test: CI/CD pipeline with errors should fail (exit 1)
#[test]
fn test_issue_006_ci_cd_errors_fail() {
    // ARRANGE: CI/CD script with actual errors
    let bash_code = r#"#!/bin/bash
# Broken deployment with ERROR
VERSION="1.0.0"
> deploy.log  # ERROR (SC2188: Redirection without command)
"#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(bash_code.as_bytes()).unwrap();

    // ACT & ASSERT: CI/CD should fail with errors
    let output = bashrs_cmd()
        .arg("lint")
        .arg(file.path())
        .output()
        .unwrap();

    // Should exit 1 (errors block CI/CD)
    assert_eq!(
        output.status.code(),
        Some(1),
        "CI/CD should fail with errors. Exit code should be 1, got: {:?}",
        output.status.code()
    );
}

// ============================================================================
// Property Tests (EXTREME TDD)
// ============================================================================

/// Property: Any script without errors should exit 0
#[test]
fn test_issue_006_property_no_errors_means_exit_0() {
    // Test multiple clean scripts
    let clean_scripts = vec![
        "#!/bin/bash\necho 'hello'\n",
        "#!/bin/bash\ntrue\n",
        "#!/bin/bash\n# Just a comment\n",
        "#!/bin/bash\nVAR=\"test\"\necho \"$VAR\"\n", // Properly quoted
    ];

    for (idx, script) in clean_scripts.iter().enumerate() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(script.as_bytes()).unwrap();

        let output = bashrs_cmd()
            .arg("lint")
            .arg(file.path())
            .output()
            .unwrap();

        assert_eq!(
            output.status.code(),
            Some(0),
            "Clean script {} should exit 0, got: {:?}",
            idx,
            output.status.code()
        );
    }
}

/// Property: File not found should always exit 2
#[test]
fn test_issue_006_property_file_not_found_exit_2() {
    let nonexistent_paths = vec![
        "/tmp/nonexistent_file_12345.sh",
        "/does/not/exist.bash",
        "~/fake_script.sh",
    ];

    for path in nonexistent_paths {
        let output = bashrs_cmd()
            .arg("lint")
            .arg(path)
            .output()
            .unwrap();

        assert_eq!(
            output.status.code(),
            Some(2),
            "File not found should exit 2 for path: {}",
            path
        );
    }
}
