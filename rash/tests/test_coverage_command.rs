#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
                               // test_coverage_command.rs - Coverage tracking tests
                               // Testing bashrs coverage command (v6.13.0 - Bash Quality Tools)

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

// RED TEST 1: Coverage command exists
#[test]
fn test_coverage_001_command_exists() {
    rash_cmd()
        .arg("coverage")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate coverage report"));
}

// RED TEST 2: Coverage on script with tests
#[test]
fn test_coverage_002_basic_coverage_report() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "coverage_test.sh",
        r#"#!/bin/bash
set -e

greet() {
    echo "Hello, $1"
}

farewell() {
    echo "Goodbye, $1"
}

test_greet() {
    result=$(greet "World")
    [[ "$result" == "Hello, World" ]] || return 1
}

# Main - greet is tested, farewell is not
greet "Test"
"#,
    );

    rash_cmd()
        .arg("coverage")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains("Coverage Report"))
        .stdout(predicate::str::contains("Lines:"))
        .stdout(predicate::str::contains("Functions:"));
}

// RED TEST 3: Coverage shows percentages
#[test]
fn test_coverage_003_shows_percentages() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "percentages.sh",
        r#"#!/bin/bash
covered_func() {
    echo "covered"
}

uncovered_func() {
    echo "never called"
}

test_covered() {
    covered_func
}

covered_func
"#,
    );

    rash_cmd()
        .arg("coverage")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains("%"));
}

// RED TEST 4: Coverage JSON output
#[test]
fn test_coverage_004_json_format() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "json_test.sh",
        r#"#!/bin/bash
func() { echo "test"; }
test_func() { func; }
func
"#,
    );

    rash_cmd()
        .arg("coverage")
        .arg(&script)
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("coverage"))
        .stdout(predicate::str::contains("lines"))
        .stdout(predicate::str::contains("functions"));
}

// RED TEST 5: Coverage minimum enforcement
#[test]
fn test_coverage_005_min_coverage_fails() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "low_coverage.sh",
        r#"#!/bin/bash
covered() { echo "covered"; }
uncovered1() { echo "not covered"; }
uncovered2() { echo "not covered"; }
uncovered3() { echo "not covered"; }

test_one() { covered; }
covered
"#,
    );

    rash_cmd()
        .arg("coverage")
        .arg(&script)
        .arg("--min")
        .arg("80")
        .assert()
        .failure()
        .stderr(predicate::str::contains("below minimum").or(predicate::str::contains("Coverage")));
}

// RED TEST 6: Coverage minimum enforcement passes
#[test]
fn test_coverage_006_min_coverage_passes() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "high_coverage.sh",
        r#"#!/bin/bash
set -e

func1() { echo "1"; }
func2() { echo "2"; }
func3() { echo "3"; }

test_all() {
    func1
    func2
    func3
}

func1
func2
func3
"#,
    );

    rash_cmd()
        .arg("coverage")
        .arg(&script)
        .arg("--min")
        .arg("50")
        .assert()
        .success();
}

// RED TEST 7: Coverage shows uncovered lines
#[test]
fn test_coverage_007_shows_uncovered_lines() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "uncovered.sh",
        r#"#!/bin/bash
covered_line() {
    echo "This is covered"
}

uncovered_line() {
    echo "This is NOT covered"
}

test_covered() {
    covered_line
}

covered_line
"#,
    );

    rash_cmd()
        .arg("coverage")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains("Uncovered").or(predicate::str::contains("Not covered")));
}

// RED TEST 8: Coverage nonexistent file fails
#[test]
fn test_coverage_008_nonexistent_file_error() {
    rash_cmd()
        .arg("coverage")
        .arg("nonexistent.sh")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error").or(predicate::str::contains("No such file")));
}

// RED TEST 9: Coverage with no tests shows zero coverage
#[test]
fn test_coverage_009_no_tests_zero_coverage() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "no_tests.sh",
        r#"#!/bin/bash
func1() { echo "1"; }
func2() { echo "2"; }
"#,
    );

    rash_cmd()
        .arg("coverage")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains("0%").or(predicate::str::contains("No tests")));
}

// RED TEST 10: Coverage detailed output
#[test]
fn test_coverage_010_detailed_output() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "detailed.sh",
        r#"#!/bin/bash
set -e

math_add() {
    echo $(($1 + $2))
}

math_sub() {
    echo $(($1 - $2))
}

test_math_add() {
    result=$(math_add 2 3)
    [[ "$result" == "5" ]] || return 1
}

math_add 1 2
"#,
    );

    rash_cmd()
        .arg("coverage")
        .arg(&script)
        .arg("--detailed")
        .assert()
        .success()
        .stdout(predicate::str::contains("Function").or(predicate::str::contains("Line")))
        .stdout(predicate::str::contains("coverage").or(predicate::str::contains("Coverage")));
}

// RED TEST 11: Coverage LCOV format
#[test]
fn test_coverage_011_lcov_format() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "lcov_test.sh",
        r#"#!/bin/bash
func() { echo "test"; }
test_func() { func; }
func
"#,
    );

    rash_cmd()
        .arg("coverage")
        .arg(&script)
        .arg("--format")
        .arg("lcov")
        .assert()
        .success()
        .stdout(predicate::str::contains("TN:").or(predicate::str::contains("SF:")));
}

// RED TEST 12: Coverage shows function names
#[test]
fn test_coverage_012_function_coverage_by_name() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "func_names.sh",
        r#"#!/bin/bash
covered_function() {
    echo "This function is covered"
}

uncovered_function() {
    echo "This function is NOT covered"
}

test_covered_function() {
    covered_function
}

covered_function
"#,
    );

    rash_cmd()
        .arg("coverage")
        .arg(&script)
        .arg("--detailed")
        .assert()
        .success()
        .stdout(predicate::str::contains("covered_function"))
        .stdout(predicate::str::contains("uncovered_function"));
}
