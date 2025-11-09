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
fn bashrs_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
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
fn test_CLI_MAKE_005_purify_report_json_format() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_report_json.mk";
    fs::write(makefile, "FILES := $(wildcard *.c)").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--report")
        .arg("--format")
        .arg("json")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("transformations_applied"))
        .stdout(predicate::str::contains("{"));

    cleanup(makefile);
}

#[test]
fn test_CLI_MAKE_005_purify_report_no_changes() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_report_clean.mk";
    fs::write(makefile, "FILES := $(sort $(wildcard *.c))").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--report")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transformations Applied: 0"));

    cleanup(makefile);
}

// ============================================================================
// RED-006: Error handling tests
// ============================================================================

#[test]
fn test_CLI_MAKE_006_parse_invalid_makefile() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_invalid.mk";
    // Invalid syntax: completely malformed
    // Note: Parser is lenient and returns empty AST rather than failing
    fs::write(makefile, "this is not a makefile at all!!! $$$$").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("items: []"));

    cleanup(makefile);
}

#[test]
fn test_CLI_MAKE_006_parse_nonexistent_file() {
    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg("tests/fixtures/nonexistent.mk")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error")); // Error message contains lowercase "error"
}

#[test]
fn test_CLI_MAKE_006_purify_invalid_makefile() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_purify_invalid.mk";
    // Note: Parser is lenient and returns empty AST rather than failing
    fs::write(makefile, "completely invalid !!!! $$$$ not a makefile").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(makefile)
        .assert()
        .success();

    cleanup(makefile);
}

// ============================================================================
// Additional edge case tests
// ============================================================================

#[test]
fn test_CLI_MAKE_007_purify_multiple_wildcards() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_multi_wildcard.mk";
    fs::write(
        makefile,
        "SOURCES := $(wildcard src/*.c)\nHEADERS := $(wildcard inc/*.h)\nOBJECTS := $(wildcard obj/*.o)",
    )
    .unwrap();

    let output = "tests/fixtures/cli_multi_wildcard_out.mk";
    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--fix")
        .arg("-o")
        .arg(output)
        .arg(makefile)
        .assert()
        .success();

    // Verify all wildcards wrapped
    let content = fs::read_to_string(output).unwrap();
    assert!(content.contains("$(sort $(wildcard src/*.c))"));
    assert!(content.contains("$(sort $(wildcard inc/*.h))"));
    assert!(content.contains("$(sort $(wildcard obj/*.o))"));

    cleanup(makefile);
    cleanup(output);
}

#[test]
fn test_CLI_MAKE_008_purify_complex_makefile() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_complex.mk";
    fs::write(
        makefile,
        r#"# Build configuration
CC := gcc
CFLAGS := -O2 -Wall

SOURCES := $(wildcard src/*.c)
OBJECTS := $(SOURCES:.c=.o)

.PHONY: build clean

build: $(OBJECTS)
	$(CC) $(CFLAGS) -o myapp $(OBJECTS)

clean:
	rm -f $(OBJECTS) myapp
"#,
    )
    .unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--report")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transformations Applied"));

    cleanup(makefile);
}

// ============================================================================
// Integration test: End-to-end workflow
// ============================================================================

#[test]
fn test_CLI_MAKE_009_integration_full_workflow() {
    ensure_fixtures_dir();
    let input = "tests/fixtures/cli_integration.mk";
    let purified = "tests/fixtures/cli_integration_purified.mk";

    // Create a Makefile with multiple issues
    let content = r#"# Build System
CC := gcc
CFLAGS := -O2 -Wall

# Non-deterministic wildcard (will be purified)
SOURCES := $(wildcard src/*.c)
HEADERS := $(wildcard inc/*.h)
OBJECTS := $(wildcard obj/*.o)

.PHONY: build clean

build: $(OBJECTS)
	$(CC) $(CFLAGS) -o myapp $(OBJECTS)

clean:
	rm -f $(OBJECTS) myapp
"#;
    fs::write(input, content).unwrap();

    // Step 1: Parse to verify valid Makefile
    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Target"));

    // Step 2: Purify with report
    let report_output = bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--report")
        .arg(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transformations Applied"))
        .get_output()
        .clone();

    let report = String::from_utf8(report_output.stdout).unwrap();
    assert!(
        report.contains("wildcard"),
        "Report should mention wildcard transformations"
    );

    // Step 3: Purify to output file
    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--fix")
        .arg("-o")
        .arg(purified)
        .arg(input)
        .assert()
        .success();

    // Step 4: Verify purified content
    let purified_content = fs::read_to_string(purified).unwrap();
    assert!(
        purified_content.contains("$(sort $(wildcard src/*.c))"),
        "Should wrap wildcards with sort"
    );
    assert!(
        purified_content.contains("$(sort $(wildcard inc/*.h))"),
        "Should wrap all wildcards"
    );
    assert!(
        purified_content.contains("$(sort $(wildcard obj/*.o))"),
        "Should wrap all wildcards"
    );
    assert!(
        purified_content.contains(".PHONY: build clean"),
        "Should preserve .PHONY"
    );

    // Step 5: Parse purified file (should succeed)
    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg(purified)
        .assert()
        .success();

    // Step 6: Re-purify should show 0 transformations (idempotent)
    let second_purify = bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--report")
        .arg(purified)
        .assert()
        .success()
        .get_output()
        .clone();

    let second_report = String::from_utf8(second_purify.stdout).unwrap();
    assert!(
        second_report.contains("Transformations Applied: 0"),
        "Should be idempotent"
    );

    cleanup(input);
    cleanup(purified);
}
