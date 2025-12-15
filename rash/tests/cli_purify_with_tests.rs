#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
// Tests can use unwrap() for simplicity
// CLI Integration Tests for bashrs purify --with-tests Command
// EXTREME TDD: RED phase - These tests will FAIL until feature is implemented
//
// Test Naming Convention: test_WITH_TESTS_<ID>_<feature>_<scenario>
//
// Task IDs:
// - WITH_TESTS_001: Basic test generation
// - WITH_TESTS_002: Determinism test generation
// - WITH_TESTS_003: Idempotency test generation
// - WITH_TESTS_004: POSIX compliance test generation
// - WITH_TESTS_005: Property-based test generation
// - WITH_TESTS_006: Test execution verification
#![allow(non_snake_case)] // Test naming convention: test_<TASK_ID>_<feature>_<scenario>

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a bashrs command (MANDATORY pattern per CLAUDE.md)
fn bashrs_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
}

/// Create a temporary bash script with given content
fn create_temp_bash_script(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");
    file
}

// ============================================================================
// Test: WITH_TESTS_001 - Basic Test Generation
// ============================================================================

#[test]
fn test_WITH_TESTS_001_generates_test_file() {
    let bash_script = r#"#!/bin/bash
# Simple script
x=42
echo $x
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("script.sh");
    let test_file = output_dir.path().join("script_test.sh");

    // RED: This will fail until --with-tests is implemented
    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--with-tests")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    // Verify test file was created
    assert!(
        test_file.exists(),
        "Test file should be generated at {}",
        test_file.display()
    );

    // Verify test file has POSIX shebang
    let test_content = fs::read_to_string(&test_file).expect("Failed to read test file");
    assert!(
        test_content.starts_with("#!/bin/sh"),
        "Test file should have POSIX shebang"
    );
}

#[test]
fn test_WITH_TESTS_001_test_file_naming_convention() {
    let bash_script = "#!/bin/bash\nx=1";
    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("my_script.sh");

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--with-tests")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    // Test file should be named <script>_test.sh
    let test_file = output_dir.path().join("my_script_test.sh");
    assert!(
        test_file.exists(),
        "Test file should follow <script>_test.sh naming"
    );
}

// ============================================================================
// Test: WITH_TESTS_002 - Determinism Test Generation
// ============================================================================

#[test]
fn test_WITH_TESTS_002_generates_determinism_test() {
    let bash_script = r#"#!/bin/bash
echo "Hello, World!"
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("script.sh");
    let test_file = output_dir.path().join("script_test.sh");

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--with-tests")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let test_content = fs::read_to_string(&test_file).expect("Failed to read test file");

    // Verify test contains determinism test
    assert!(
        test_content.contains("test_determinism") || test_content.contains("determinism"),
        "Test file should contain determinism test"
    );

    // Verify test runs script twice to compare outputs
    assert!(
        test_content.contains("output1") && test_content.contains("output2"),
        "Determinism test should run script twice and compare outputs"
    );
}

#[test]
fn test_WITH_TESTS_002_determinism_test_passes() {
    let bash_script = r#"#!/bin/bash
echo "constant"
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("script.sh");
    let test_file = output_dir.path().join("script_test.sh");

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--with-tests")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    // Make generated files executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&output_file).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&output_file, perms).unwrap();

        let mut test_perms = fs::metadata(&test_file).unwrap().permissions();
        test_perms.set_mode(0o755);
        fs::set_permissions(&test_file, test_perms).unwrap();
    }

    // Run generated tests - should pass
    let output = Command::new("sh")
        .arg(&test_file)
        .current_dir(&output_dir)
        .output()
        .expect("Failed to run generated tests");

    assert!(
        output.status.success(),
        "Generated determinism test should pass for deterministic script"
    );
}

// ============================================================================
// Test: WITH_TESTS_003 - Idempotency Test Generation
// ============================================================================

#[test]
fn test_WITH_TESTS_003_generates_idempotency_test() {
    let bash_script = r#"#!/bin/bash
mkdir -p /tmp/test_dir
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("script.sh");
    let test_file = output_dir.path().join("script_test.sh");

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--with-tests")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let test_content = fs::read_to_string(&test_file).expect("Failed to read test file");

    // Verify test contains idempotency test
    assert!(
        test_content.contains("test_idempotency") || test_content.contains("idempotent"),
        "Test file should contain idempotency test"
    );

    // Verify test runs script multiple times
    assert!(
        test_content.matches("./").count() >= 2,
        "Idempotency test should run script multiple times"
    );
}

#[test]
fn test_WITH_TESTS_003_idempotency_test_detects_issues() {
    // Script with non-idempotent operation (mkdir without -p)
    let bash_script = r#"#!/bin/bash
mkdir /tmp/test_dir_$RANDOM
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("script.sh");

    // Purify should fix this (add -p, remove $RANDOM)
    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--with-tests")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    // Purified version should be idempotent
    let purified = fs::read_to_string(&output_file).expect("Failed to read purified");
    assert!(
        purified.contains("mkdir -p"),
        "Purified script should use mkdir -p"
    );
    assert!(
        !purified.contains("$RANDOM"),
        "Purified script should not contain $RANDOM"
    );
}

// ============================================================================
// Test: WITH_TESTS_004 - POSIX Compliance Test Generation
// ============================================================================

#[test]
fn test_WITH_TESTS_004_generates_posix_compliance_test() {
    let bash_script = r#"#!/bin/bash
echo "test"
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("script.sh");
    let test_file = output_dir.path().join("script_test.sh");

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--with-tests")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let test_content = fs::read_to_string(&test_file).expect("Failed to read test file");

    // Verify test contains POSIX compliance check
    assert!(
        test_content.contains("shellcheck") || test_content.contains("POSIX"),
        "Test file should contain POSIX compliance test"
    );
}

// ============================================================================
// Test: WITH_TESTS_005 - Property-Based Test Generation
// ============================================================================

#[test]
fn test_WITH_TESTS_005_property_tests_flag() {
    let bash_script = r#"#!/bin/bash
echo "$1"
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("script.sh");
    let test_file = output_dir.path().join("script_test.sh");

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--with-tests")
        .arg("--property-tests")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let test_content = fs::read_to_string(&test_file).expect("Failed to read test file");

    // Verify test contains property testing logic
    assert!(
        test_content.contains("for")
            || test_content.contains("while")
            || test_content.contains("seq"),
        "Property tests should iterate over multiple test cases"
    );

    // Verify test runs many cases (look for numbers like 100)
    assert!(
        test_content.contains("100") || test_content.contains("50"),
        "Property tests should run many cases (e.g., 100)"
    );
}

// ============================================================================
// Test: WITH_TESTS_006 - Test Execution Verification
// ============================================================================

#[test]
fn test_WITH_TESTS_006_generated_tests_are_executable() {
    let bash_script = r#"#!/bin/bash
x=42
echo $x
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("script.sh");
    let test_file = output_dir.path().join("script_test.sh");

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--with-tests")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    // Make test file executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_file).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_file, perms).unwrap();

        // Make purified script executable
        let mut script_perms = fs::metadata(&output_file).unwrap().permissions();
        script_perms.set_mode(0o755);
        fs::set_permissions(&output_file, script_perms).unwrap();
    }

    // Verify test file is valid sh
    let output = Command::new("sh")
        .arg("-n") // Syntax check only
        .arg(&test_file)
        .output()
        .expect("Failed to check test file syntax");

    assert!(
        output.status.success(),
        "Generated test file should have valid sh syntax"
    );
}

#[test]
fn test_WITH_TESTS_006_all_tests_pass_for_valid_script() {
    let bash_script = r#"#!/bin/bash
# Deterministic, idempotent script
echo "Hello, World!"
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("script.sh");
    let test_file = output_dir.path().join("script_test.sh");

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--with-tests")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    // Make files executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut test_perms = fs::metadata(&test_file).unwrap().permissions();
        test_perms.set_mode(0o755);
        fs::set_permissions(&test_file, test_perms).unwrap();

        let mut script_perms = fs::metadata(&output_file).unwrap().permissions();
        script_perms.set_mode(0o755);
        fs::set_permissions(&output_file, script_perms).unwrap();
    }

    // Run generated tests
    let output = Command::new("sh")
        .arg(&test_file)
        .current_dir(&output_dir)
        .output()
        .expect("Failed to run generated tests");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Generated tests should pass for valid purified script.\nStdout: {}\nStderr: {}",
        stdout,
        stderr
    );
}

// ============================================================================
// Test: Error Handling
// ============================================================================

#[test]
fn test_WITH_TESTS_error_missing_output() {
    let bash_script = "#!/bin/bash\necho test";
    let input_file = create_temp_bash_script(bash_script);

    // Should fail: --with-tests requires -o flag
    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--with-tests")
        .assert()
        .failure()
        .stderr(predicate::str::contains("output").or(predicate::str::contains("-o")));
}

// ============================================================================
// Test: Help and Documentation
// ============================================================================

#[test]
fn test_WITH_TESTS_help_flag() {
    bashrs_cmd()
        .arg("purify")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--with-tests"))
        .stdout(predicate::str::contains("Generate test suite"));
}
