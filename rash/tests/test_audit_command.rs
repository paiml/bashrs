#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// test_audit_command.rs - Comprehensive quality audit tests
// Testing bashrs audit command (v6.12.0 - Bash Quality Tools)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper function to create rash command
fn rash_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
}

/// Helper function to create test bash script
fn create_test_script(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let file_path = dir.path().join(name);
    fs::write(&file_path, content).expect("Failed to write test file");
    file_path
}

// RED TEST 1: Audit command exists
#[test]
fn test_audit_001_command_exists() {
    rash_cmd()
        .arg("audit")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Run comprehensive quality audit"));
}

// RED TEST 2: Audit perfect script succeeds
#[test]
fn test_audit_002_perfect_script_passes() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "perfect.sh",
        r#"#!/bin/bash
# Perfect script with all quality features

set -euo pipefail

# Function with proper documentation
# Args: name - The name to greet
greet() {
    local name="$1"
    echo "Hello, ${name}"
}

# TEST: greet function basic
test_greet_basic() {
    result=$(greet "World")
    [[ "$result" == "Hello, World" ]] || return 1
}

# Main execution
main() {
    greet "Rash"
}

main "$@"
"#,
    );

    rash_cmd()
        .arg("audit")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains("Overall:").and(predicate::str::contains("PASS")))
        .stdout(predicate::str::contains("Lint"))
        .stdout(predicate::str::contains("Test"))
        .stdout(predicate::str::contains("Score"));
}

// RED TEST 3: Audit poor script shows failures
#[test]
fn test_audit_003_poor_script_shows_issues() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "poor.sh",
        r#"#!/bin/bash
FILES=$(ls *.txt)
for f in $FILES; do
    echo $f
done
"#,
    );

    rash_cmd()
        .arg("audit")
        .arg(&script)
        .assert()
        .success() // Still succeeds but shows warnings
        .stdout(predicate::str::contains("Lint"))
        .stdout(predicate::str::contains("warnings").or(predicate::str::contains("errors")))
        .stdout(predicate::str::contains("Score"));
}

// RED TEST 4: Audit JSON output format
#[test]
fn test_audit_004_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "test.sh",
        r#"#!/bin/bash
echo "test"
"#,
    );

    rash_cmd()
        .arg("audit")
        .arg(&script)
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("lint"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("score"));
}

// RED TEST 5: Audit strict mode fails on warnings
#[test]
fn test_audit_005_strict_mode_fails_on_warnings() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "warnings.sh",
        r#"#!/bin/bash
FILES=$(ls *.txt)  # SC2012 warning
echo $FILES  # Unquoted variable
"#,
    );

    rash_cmd()
        .arg("audit")
        .arg(&script)
        .arg("--strict")
        .assert()
        .failure()
        .stderr(predicate::str::contains("warnings").or(predicate::str::contains("errors")));
}

// RED TEST 6: Audit shows all check results
#[test]
fn test_audit_006_shows_all_checks() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "basic.sh",
        r#"#!/bin/bash
set -e
greet() {
    echo "Hello, $1"
}
test_greet() {
    result=$(greet "Test")
    [[ "$result" == "Hello, Test" ]] || return 1
}
"#,
    );

    rash_cmd()
        .arg("audit")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse"))
        .stdout(predicate::str::contains("Lint"))
        .stdout(predicate::str::contains("Test"))
        .stdout(predicate::str::contains("Score"));
}

// RED TEST 7: Audit detailed output shows dimension breakdown
#[test]
fn test_audit_007_detailed_output() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "detailed.sh",
        r#"#!/bin/bash
set -euo pipefail
main() {
    echo "test"
}
main "$@"
"#,
    );

    rash_cmd()
        .arg("audit")
        .arg(&script)
        .arg("--detailed")
        .assert()
        .success()
        .stdout(predicate::str::contains("Complexity"))
        .stdout(predicate::str::contains("Safety"))
        .stdout(predicate::str::contains("Maintainability"));
}

// RED TEST 8: Audit nonexistent file fails
#[test]
fn test_audit_008_nonexistent_file_error() {
    rash_cmd()
        .arg("audit")
        .arg("nonexistent.sh")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error").or(predicate::str::contains("No such file")));
}

// RED TEST 9: Audit with minimum grade enforcement
// TODO: Implement --min-grade feature in audit command
#[test]
#[ignore = "Feature not yet implemented - needs GREEN phase implementation"]
fn test_audit_009_min_grade_enforcement() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "low_quality.sh",
        r#"#!/bin/bash
# Very basic, low quality script
FILES=$(ls)
rm $FILES
"#,
    );

    rash_cmd()
        .arg("audit")
        .arg(&script)
        .arg("--min-grade")
        .arg("A")
        .assert()
        .failure()
        .stderr(predicate::str::contains("grade").or(predicate::str::contains("below minimum")));
}

// RED TEST 10: Audit summary shows pass/fail counts
#[test]
fn test_audit_010_summary_counts() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "mixed.sh",
        r#"#!/bin/bash
set -e

test_pass() {
    [[ "1" == "1" ]] || return 1
}

test_fail() {
    [[ "1" == "2" ]] || return 1
}

greet() {
    echo "Hello"
}
"#,
    );

    // Allow failure because test_fail() will intentionally fail
    let output = rash_cmd().arg("audit").arg(&script).assert().failure(); // Expect failure due to failed test

    // But still verify the output shows counts
    output.stdout(predicate::str::contains("passed").or(predicate::str::contains("failed")));
}
