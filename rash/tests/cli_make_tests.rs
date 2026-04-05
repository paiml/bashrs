#![allow(deprecated)]
#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)] // Tests can use expect() for simplicity
//! CLI integration tests for Makefile parsing and purification
//!
//! These tests verify the `bashrs make` subcommands work correctly.
//! All tests use `assert_cmd` as mandated by CLAUDE.md.
//!
//! Test naming convention: test_<TASK_ID>_<feature>_<scenario>

#![allow(non_snake_case)] // Test naming convention: test_<TASK_ID>_<feature>_<scenario>

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

/// Helper function to create bashrs command
#[allow(deprecated)]
fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

/// Helper to create test fixture directory if needed
fn ensure_fixtures_dir() {
    let _ = fs::create_dir_all("tests/fixtures");
}

/// Helper to cleanup test file
fn cleanup(path: &str) {
    let _ = fs::remove_file(path);
}

// ============================================================================
// RED-001: Parse command tests
// ============================================================================

#[test]
fn test_CLI_MAKE_001_parse_basic_makefile() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_simple.mk";
    fs::write(makefile, "target:\n\techo hello").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("Target"));

    cleanup(makefile);
}

#[test]
fn test_CLI_MAKE_001_parse_json_format() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_json.mk";
    fs::write(makefile, "CC := gcc\ntarget:\n\techo hello").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg(makefile)
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("items")); // MakeAst uses Debug format, not Serialize

    cleanup(makefile);
}

// ============================================================================
// RED-002: Purify command (dry-run) tests
// ============================================================================

#[test]
fn test_CLI_MAKE_002_purify_dry_run() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_wildcard.mk";
    fs::write(makefile, "FILES := $(wildcard *.c)").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("$(sort $(wildcard"));

    // Verify original file unchanged
    let content = fs::read_to_string(makefile).unwrap();
    assert_eq!(content, "FILES := $(wildcard *.c)");

    cleanup(makefile);
}

#[test]
fn test_CLI_MAKE_002_purify_no_changes_needed() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_clean.mk";
    fs::write(makefile, "FILES := $(sort $(wildcard *.c))").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("$(sort $(wildcard"));

    cleanup(makefile);
}

// ============================================================================
// RED-003: Purify with --fix (in-place) tests
// ============================================================================

#[test]
fn test_CLI_MAKE_003_purify_fix_inplace() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_wildcard_fix.mk";
    fs::write(makefile, "FILES := $(wildcard *.c)").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--fix")
        .arg(makefile)
        .assert()
        .success();

    // Verify file changed
    let content = fs::read_to_string(makefile).unwrap();
    assert!(content.contains("$(sort $(wildcard"));

    // Verify backup created
    let backup = format!("{}.bak", makefile);
    assert!(Path::new(&backup).exists());
    let backup_content = fs::read_to_string(&backup).unwrap();
    assert_eq!(backup_content, "FILES := $(wildcard *.c)");

    cleanup(makefile);
    cleanup(&backup);
}

#[test]
fn test_CLI_MAKE_003_purify_fix_creates_backup() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_backup_test.mk";
    let original_content = "SOURCES := $(wildcard src/*.c)\nOBJECTS := $(wildcard obj/*.o)";
    fs::write(makefile, original_content).unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--fix")
        .arg(makefile)
        .assert()
        .success();

    // Verify backup exists and has original content
    let backup = format!("{}.bak", makefile);
    assert!(Path::new(&backup).exists());
    let backup_content = fs::read_to_string(&backup).unwrap();
    assert_eq!(backup_content, original_content);

    cleanup(makefile);
    cleanup(&backup);
}

// ============================================================================
// RED-004: Purify with -o (output to new file) tests
// ============================================================================

#[test]
fn test_CLI_MAKE_004_purify_output_file() {
    ensure_fixtures_dir();
    let input = "tests/fixtures/cli_wildcard_input.mk";
    let output = "tests/fixtures/cli_wildcard_output.mk";
    fs::write(input, "FILES := $(wildcard *.c)").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--fix")
        .arg("-o")
        .arg(output)
        .arg(input)
        .assert()
        .success();

    // Verify output file created
    let content = fs::read_to_string(output).unwrap();
    assert!(content.contains("$(sort $(wildcard"));

    // Verify input file unchanged
    let input_content = fs::read_to_string(input).unwrap();
    assert_eq!(input_content, "FILES := $(wildcard *.c)");

    cleanup(input);
    cleanup(output);
}

#[test]
fn test_CLI_MAKE_004_purify_output_preserves_input() {
    ensure_fixtures_dir();
    let input = "tests/fixtures/cli_preserve_input.mk";
    let output = "tests/fixtures/cli_preserve_output.mk";
    let original = "VAR := $(wildcard *.h)\nSRC := $(wildcard *.c)";
    fs::write(input, original).unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--fix")
        .arg("-o")
        .arg(output)
        .arg(input)
        .assert()
        .success();

    // Input should be byte-identical to original
    let input_content = fs::read_to_string(input).unwrap();
    assert_eq!(input_content, original);

    // Output should be purified
    let output_content = fs::read_to_string(output).unwrap();
    assert!(output_content.contains("$(sort $(wildcard"));

    cleanup(input);
    cleanup(output);
}

// ============================================================================
// RED-005: Purify with --report tests
// ============================================================================

#[test]
fn test_CLI_MAKE_005_purify_report() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_wildcard_report.mk";
    fs::write(makefile, "FILES := $(wildcard *.c)").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--report")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transformations Applied"))
        .stdout(predicate::str::contains("wildcard"));

    cleanup(makefile);
}

#[test]

include!("cli_make_tests_tests_CLI.rs");
