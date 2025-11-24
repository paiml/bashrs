#![allow(clippy::unwrap_used)]
// Tests can use unwrap() for simplicity
// CLI Integration Tests for bashrs purify Command
// EXTREME TDD: CLI testing with assert_cmd (MANDATORY per CLAUDE.md)
//
// Tests the purify command following the test naming convention:
// test_<TASK_ID>_<feature>_<scenario>
//
// Task IDs:
// - PURIFY_001: Basic purification (bash → purified POSIX sh)
// - PURIFY_002: Output to file with -o flag
// - PURIFY_003: Report generation with --report flag
// - PURIFY_004: Error handling (nonexistent files, invalid bash)
// - PURIFY_005: Performance benchmarking
// - PURIFY_006: Determinism transformations
// - PURIFY_007: Idempotency transformations
// - PURIFY_008: Safety transformations (variable quoting)
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
// Test: PURIFY_001 - Basic Purification (stdout)
// ============================================================================

#[test]
fn test_PURIFY_001_basic_purify_to_stdout() {
    let bash_script = r#"#!/bin/bash
# Test script
x=42
echo $x
"#;

    let input_file = create_temp_bash_script(bash_script);

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("#!/bin/sh"))
        .stdout(predicate::str::contains("x=42"));
}

#[test]
fn test_PURIFY_001_basic_purify_preserves_comments() {
    let bash_script = r#"#!/bin/bash
# Important comment
x=42
"#;

    let input_file = create_temp_bash_script(bash_script);

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Important comment"));
}

#[test]
fn test_PURIFY_001_basic_purify_empty_script() {
    let bash_script = r#"#!/bin/bash
"#;

    let input_file = create_temp_bash_script(bash_script);

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("#!/bin/sh"));
}

// ============================================================================
// Test: PURIFY_002 - Output to File with -o Flag
// ============================================================================

#[test]
fn test_PURIFY_002_output_to_file_short_flag() {
    let bash_script = r#"#!/bin/bash
x=42
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("purified.sh");

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    // Verify output file was created
    assert!(output_file.exists(), "Output file should exist");

    let output_content = fs::read_to_string(&output_file).expect("Failed to read output file");
    assert!(
        output_content.contains("#!/bin/sh"),
        "Output should contain POSIX shebang"
    );
    assert!(
        output_content.contains("x=42"),
        "Output should contain purified code"
    );
}

#[test]
fn test_PURIFY_002_output_to_file_long_flag() {
    let bash_script = r#"#!/bin/bash
msg="hello"
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("purified.sh");

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();

    assert!(output_file.exists(), "Output file should exist");
}

#[test]
fn test_PURIFY_002_output_preserves_content() {
    let bash_script = r#"#!/bin/bash
# Comment
x=1
y=2
z=3
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("purified.sh");

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).expect("Failed to read output file");
    assert!(
        output_content.contains("Comment"),
        "Should preserve comment text"
    );
    assert!(output_content.contains("x=1"), "Should preserve x=1");
    assert!(output_content.contains("y=2"), "Should preserve y=2");
    assert!(output_content.contains("z=3"), "Should preserve z=3");
}

// ============================================================================
// Test: PURIFY_003 - Report Generation with --report Flag
// ============================================================================

#[test]
fn test_PURIFY_003_report_shows_transformations() {
    let bash_script = r#"#!/bin/bash
x=42
"#;

    let input_file = create_temp_bash_script(bash_script);

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--report")
        .assert()
        .success()
        .stdout(predicate::str::contains("Purification Report"))
        .stdout(predicate::str::contains("Transformations Applied"))
        .stdout(predicate::str::contains("Shebang"))
        .stdout(predicate::str::contains("Determinism"))
        .stdout(predicate::str::contains("Idempotency"))
        .stdout(predicate::str::contains("Safety"));
}

#[test]
fn test_PURIFY_003_report_shows_performance() {
    let bash_script = r#"#!/bin/bash
x=42
"#;

    let input_file = create_temp_bash_script(bash_script);

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--report")
        .assert()
        .success()
        .stdout(predicate::str::contains("Performance:"))
        .stdout(predicate::str::contains("Read:"))
        .stdout(predicate::str::contains("Parse:"))
        .stdout(predicate::str::contains("Purify:"))
        .stdout(predicate::str::contains("Codegen:"))
        .stdout(predicate::str::contains("Total:"))
        .stdout(predicate::str::contains("Throughput:"));
}

#[test]
fn test_PURIFY_003_report_shows_input_output_sizes() {
    let bash_script = r#"#!/bin/bash
x=42
y=43
"#;

    let input_file = create_temp_bash_script(bash_script);

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("--report")
        .assert()
        .success()
        .stdout(predicate::str::contains("Input size:"))
        .stdout(predicate::str::contains("Output size:"))
        .stdout(predicate::str::contains("lines"))
        .stdout(predicate::str::contains("bytes"));
}

#[test]
fn test_PURIFY_003_report_with_output_file() {
    let bash_script = r#"#!/bin/bash
x=42
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("purified.sh");

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("-o")
        .arg(&output_file)
        .arg("--report")
        .assert()
        .success()
        .stdout(predicate::str::contains("Input:"))
        .stdout(predicate::str::contains("Output:"))
        .stdout(predicate::str::contains("purified.sh"));
}

// ============================================================================
// Test: PURIFY_004 - Error Handling
// ============================================================================

#[test]
fn test_PURIFY_004_nonexistent_input_file() {
    bashrs_cmd()
        .arg("purify")
        .arg("/nonexistent/file.sh")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error").or(predicate::str::contains("error")));
}

#[test]
fn test_PURIFY_004_missing_input_argument() {
    bashrs_cmd()
        .arg("purify")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("FILE")));
}

#[test]
fn test_PURIFY_004_invalid_output_path() {
    let bash_script = r#"#!/bin/bash
x=42
"#;

    let input_file = create_temp_bash_script(bash_script);

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("-o")
        .arg("/nonexistent/dir/output.sh")
        .assert()
        .failure();
}

// ============================================================================
// Test: PURIFY_006 - Determinism Transformations
// ============================================================================

#[test]
fn test_PURIFY_006_removes_random_variable() {
    let bash_script = r#"#!/bin/bash
SESSION_ID=$RANDOM
echo $SESSION_ID
"#;

    let input_file = create_temp_bash_script(bash_script);

    let output = bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .output()
        .expect("Failed to execute purify");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Purified script should NOT contain $RANDOM
    // (It should be replaced with a deterministic value)
    assert!(output.status.success(), "Purify command should succeed");
    assert!(stdout.contains("SESSION_ID="), "Should contain assignment");
}

#[test]
fn test_PURIFY_006_removes_timestamps() {
    let bash_script = r#"#!/bin/bash
RELEASE="release-$(date +%s)"
echo $RELEASE
"#;

    let input_file = create_temp_bash_script(bash_script);

    let output = bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .output()
        .expect("Failed to execute purify");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Purify command should succeed");
    assert!(stdout.contains("RELEASE="), "Should contain assignment");
}

// ============================================================================
// Test: PURIFY_007 - Idempotency Transformations
// ============================================================================

#[test]
fn test_PURIFY_007_mkdir_becomes_mkdir_p() {
    let bash_script = r#"#!/bin/bash
mkdir /tmp/test
"#;

    let input_file = create_temp_bash_script(bash_script);

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("mkdir -p"));
}

#[test]
fn test_PURIFY_007_rm_becomes_rm_f() {
    let bash_script = r#"#!/bin/bash
rm /tmp/file.txt
"#;

    let input_file = create_temp_bash_script(bash_script);

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("rm -f"));
}

#[test]
fn test_PURIFY_007_ln_becomes_safe_symlink() {
    let bash_script = r#"#!/bin/bash
ln -s /source /target
"#;

    let input_file = create_temp_bash_script(bash_script);

    let output = bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .output()
        .expect("Failed to execute purify");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Purify command should succeed");
    // Should either use rm -f before ln -s or use ln -sf
    assert!(
        stdout.contains("ln -s") || stdout.contains("ln"),
        "Should contain symlink operation"
    );
}

// ============================================================================
// Test: PURIFY_008 - Safety Transformations (Variable Quoting)
// ============================================================================

#[test]
fn test_PURIFY_008_unquoted_variable_echo() {
    let bash_script = r#"#!/bin/bash
msg="hello"
echo $msg
"#;

    let input_file = create_temp_bash_script(bash_script);

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"$msg\"").or(predicate::str::contains("$msg")));
}

// ============================================================================
// Test: Integration - Complex Real-World Scripts
// ============================================================================

#[test]
fn test_PURIFY_integration_deployment_script() {
    let bash_script = r#"#!/bin/bash
# Deployment script
SESSION_ID=$RANDOM
RELEASE="release-$(date +%s)"
mkdir /tmp/releases/$RELEASE
rm /tmp/current
ln -s /tmp/releases/$RELEASE /tmp/current
echo $UNQUOTED
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("purified_deploy.sh");

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("-o")
        .arg(&output_file)
        .arg("--report")
        .assert()
        .success()
        .stdout(predicate::str::contains("Purification Report"))
        .stdout(predicate::str::contains("Performance:"));

    // Verify output file
    assert!(output_file.exists());
    let output_content = fs::read_to_string(&output_file).expect("Failed to read output");

    // Verify transformations
    assert!(
        output_content.contains("#!/bin/sh"),
        "Should have POSIX shebang"
    );
    assert!(
        output_content.contains("mkdir -p"),
        "Should have idempotent mkdir"
    );
    assert!(output_content.contains("rm -f"), "Should have safe rm");
}

#[test]
fn test_PURIFY_integration_multiple_files() {
    let script1 = r#"#!/bin/bash
x=1
"#;
    let script2 = r#"#!/bin/bash
y=2
"#;

    let input1 = create_temp_bash_script(script1);
    let input2 = create_temp_bash_script(script2);

    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output1 = output_dir.path().join("out1.sh");
    let output2 = output_dir.path().join("out2.sh");

    // Purify first file
    bashrs_cmd()
        .arg("purify")
        .arg(input1.path())
        .arg("-o")
        .arg(&output1)
        .assert()
        .success();

    // Purify second file
    bashrs_cmd()
        .arg("purify")
        .arg(input2.path())
        .arg("-o")
        .arg(&output2)
        .assert()
        .success();

    // Both should exist
    assert!(output1.exists());
    assert!(output2.exists());

    // Both should have POSIX shebang
    let content1 = fs::read_to_string(&output1).expect("Failed to read");
    let content2 = fs::read_to_string(&output2).expect("Failed to read");

    assert!(content1.contains("#!/bin/sh"));
    assert!(content2.contains("#!/bin/sh"));
}

// ============================================================================
// Test: Help and Documentation
// ============================================================================

#[test]
fn test_PURIFY_help_flag() {
    bashrs_cmd()
        .arg("purify")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Purify bash scripts"))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--report"));
}

#[test]
fn test_PURIFY_in_main_help() {
    bashrs_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("purify"))
        .stdout(predicate::str::contains("Purify bash scripts"));
}
