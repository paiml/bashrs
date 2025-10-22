//! Integration tests for CONFIG-004: Non-Deterministic Constructs

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create bashrs command
fn bashrs_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
}

#[test]
fn test_config_004_detect_random() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test-bashrc");

    let content = r#"export SESSION_ID=$RANDOM
export PATH="/usr/local/bin:$PATH"
export EDITOR=vim"#;

    fs::write(&test_file, content).unwrap();

    // ACT
    let output = bashrs_cmd()
        .arg("config")
        .arg("analyze")
        .arg(&test_file)
        .assert()
        .success();

    // ASSERT
    output
        .stdout(predicate::str::contains("CONFIG-004"))
        .stdout(predicate::str::contains("Non-deterministic"));
}

#[test]
fn test_config_004_detect_timestamp() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test-bashrc");

    let content = r#"export BUILD_TAG="build-$(date +%s)""#;

    fs::write(&test_file, content).unwrap();

    // ACT
    let output = bashrs_cmd()
        .arg("config")
        .arg("analyze")
        .arg(&test_file)
        .assert()
        .success();

    // ASSERT
    output
        .stdout(predicate::str::contains("CONFIG-004"))
        .stdout(predicate::str::contains("Non-deterministic"));
}

#[test]
fn test_config_004_detect_process_id() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test-bashrc");

    let content = r#"export TEMP_DIR="/tmp/work-$$""#;

    fs::write(&test_file, content).unwrap();

    // ACT
    let output = bashrs_cmd()
        .arg("config")
        .arg("analyze")
        .arg(&test_file)
        .assert()
        .success();

    // ASSERT
    output
        .stdout(predicate::str::contains("CONFIG-004"))
        .stdout(predicate::str::contains("Non-deterministic"));
}

#[test]
fn test_config_004_detect_multiple() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test-bashrc");

    let content = r#"export SESSION_ID=$RANDOM
export BUILD_TAG="build-$(date +%s)"
export TEMP_DIR="/tmp/work-$$""#;

    fs::write(&test_file, content).unwrap();

    // ACT
    let output = bashrs_cmd()
        .arg("config")
        .arg("analyze")
        .arg(&test_file)
        .assert()
        .success();

    // ASSERT: Should detect all 3
    output
        .stdout(predicate::str::contains("CONFIG-004"));
}

#[test]
fn test_config_004_lint_shows_issues() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test-bashrc");

    let content = r#"export SESSION_ID=$RANDOM"#;

    fs::write(&test_file, content).unwrap();

    // ACT & ASSERT
    // NOTE: lint exits with code 1 when issues are found
    bashrs_cmd()
        .arg("config")
        .arg("lint")
        .arg(&test_file)
        .assert()
        .failure() // Lint returns non-zero when issues found
        .stdout(predicate::str::contains("CONFIG-004"));
}

#[test]
fn test_config_004_purify_removes_constructs() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test-bashrc");
    let output_file = temp_dir.path().join("output-bashrc");

    let content = r#"export PATH="/usr/local/bin:$PATH"
export SESSION_ID=$RANDOM
export EDITOR=vim"#;

    fs::write(&test_file, content).unwrap();

    // ACT
    bashrs_cmd()
        .arg("config")
        .arg("purify")
        .arg(&test_file)
        .arg("--output")
        .arg(&output_file)
        .arg("--dry-run")
        .assert()
        .success();

    // ASSERT: Check output file content
    let purified = fs::read_to_string(&output_file).unwrap();

    // Should preserve deterministic lines
    assert!(purified.contains("export PATH"));
    assert!(purified.contains("export EDITOR"));

    // Should comment out non-deterministic line
    assert!(purified.contains("# RASH: Non-deterministic construct removed"));
    assert!(purified.contains("# export SESSION_ID=$RANDOM"));
}

#[test]
fn test_config_004_with_fixture_file() {
    // ARRANGE
    // Get absolute path to fixture file from workspace root
    let mut fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixture.push("tests/fixtures/configs/messy-bashrc.sh");

    // ACT
    let output = bashrs_cmd()
        .arg("config")
        .arg("lint")
        .arg(&fixture)
        .assert()
        .failure(); // Lint returns non-zero when issues found

    // ASSERT: Should detect non-deterministic constructs
    output.stdout(predicate::str::contains("CONFIG-004"));
}

#[test]
fn test_config_004_no_issues() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test-bashrc");

    let content = r#"export PATH="/usr/local/bin:$PATH"
export EDITOR=vim
export VERSION="1.0.0"
alias ll='ls -la'"#;

    fs::write(&test_file, content).unwrap();

    // ACT
    let output = bashrs_cmd()
        .arg("config")
        .arg("analyze")
        .arg(&test_file)
        .assert()
        .success();

    // ASSERT: Should not have CONFIG-004 issues
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // If there are CONFIG-004 issues, test fails
    // (There might be other CONFIG issues like PATH dedup, but not CONFIG-004)
    if stdout.contains("CONFIG-004") {
        panic!("Expected no CONFIG-004 issues, but found some");
    }
}

#[test]
fn test_config_004_idempotent_purification() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test-bashrc");
    let output_file1 = temp_dir.path().join("output1-bashrc");
    let output_file2 = temp_dir.path().join("output2-bashrc");

    let content = r#"export SESSION_ID=$RANDOM"#;

    fs::write(&test_file, content).unwrap();

    // ACT: Purify once
    bashrs_cmd()
        .arg("config")
        .arg("purify")
        .arg(&test_file)
        .arg("--output")
        .arg(&output_file1)
        .arg("--dry-run")
        .assert()
        .success();

    // ACT: Purify the purified output
    bashrs_cmd()
        .arg("config")
        .arg("purify")
        .arg(&output_file1)
        .arg("--output")
        .arg(&output_file2)
        .arg("--dry-run")
        .assert()
        .success();

    // ASSERT: Should be idempotent
    let purified1 = fs::read_to_string(&output_file1).unwrap();
    let purified2 = fs::read_to_string(&output_file2).unwrap();

    assert_eq!(purified1, purified2, "Purification should be idempotent");
}
