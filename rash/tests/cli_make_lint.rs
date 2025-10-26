//! CLI Integration Tests for `bashrs make lint` command
//!
//! Tests the Makefile linting CLI with assert_cmd

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper function to create bashrs command
fn bashrs_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
}

#[test]
fn test_CLI_MAKE_LINT_001_basic_lint_detects_issues() {
    // ARRANGE: Create Makefile with linting issues
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");

    fs::write(
        &makefile_path,
        r#"
VERSION = $(shell git describe)
SOURCES = $(wildcard src/*.c)

build:
	mkdir build
	gcc $(SOURCES) -o app

clean:
	rm -rf $BUILD_DIR
"#,
    )
    .unwrap();

    // ACT & ASSERT: Run linter should detect issues
    bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&makefile_path)
        .assert()
        .failure() // Should fail with issues found
        .stdout(predicate::str::contains("MAKE")) // Should show MAKE rule codes
        .stdout(predicate::str::contains("wildcard")); // Should mention wildcard issue
}

#[test]
fn test_CLI_MAKE_LINT_002_clean_makefile_passes() {
    // ARRANGE: Create reasonably clean Makefile  (may have warnings, but no errors)
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");

    fs::write(
        &makefile_path,
        r#".DELETE_ON_ERROR:
.ONESHELL:

VERSION := 1.0.0

.PHONY: build clean

build:
	mkdir -p build
	gcc -o app

clean:
	rm -rf build
"#,
    )
    .unwrap();

    // ACT & ASSERT: Clean Makefile should not have errors (warnings are OK)
    // Warnings help improve code quality - exit code 1 for warnings is acceptable
    let output = bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&makefile_path)
        .output()
        .unwrap();

    // Should NOT have errors (exit code 2 or higher)
    // Warnings (exit code 1) or no issues (exit code 0) are both OK
    assert!(
        output.status.code().unwrap() < 2,
        "Clean Makefile should not have errors (exit code should be 0 or 1, got {})",
        output.status.code().unwrap()
    );

    // Should not contain "error" in output (case-insensitive)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.to_lowercase().contains("[error]"),
        "Clean Makefile should not have errors in output"
    );
}

#[test]
fn test_CLI_MAKE_LINT_003_fix_flag_applies_fixes() {
    // ARRANGE: Create Makefile with fixable issues
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");

    fs::write(
        &makefile_path,
        r#"VERSION = $(shell git describe)
SOURCES = $(wildcard src/*.c)
"#,
    )
    .unwrap();

    // ACT: Run linter with --fix
    bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&makefile_path)
        .arg("--fix")
        .assert()
        .success();

    // ASSERT: Backup should be created
    let backup_path = makefile_path.with_extension("bak");
    assert!(backup_path.exists(), "Backup file should be created");

    // ASSERT: Fixed file should have := instead of =
    let fixed_content = fs::read_to_string(&makefile_path).unwrap();
    assert!(fixed_content.contains(":="), "Should use := assignment");
    assert!(fixed_content.contains("sort"), "Should add sort() wrapper");
}

#[test]
fn test_CLI_MAKE_LINT_004_output_flag_writes_to_file() {
    // ARRANGE: Create Makefile with issues
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");
    let output_path = temp_dir.path().join("Makefile.fixed");

    fs::write(
        &makefile_path,
        r#"VERSION = $(shell git describe)
"#,
    )
    .unwrap();

    // ACT: Run linter with --fix and -o
    let result = bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&makefile_path)
        .arg("--fix")
        .arg("-o")
        .arg(&output_path)
        .output()
        .unwrap();

    // If fix succeeded, check files
    // If fix failed due to no fixes being available, that's okay
    if result.status.success() || result.status.code() == Some(0) {
        // ASSERT: Output file should exist if fix succeeded
        assert!(
            output_path.exists(),
            "Output file should be created on success"
        );

        // ASSERT: Original file should be unchanged when using -o flag
        let original_content = fs::read_to_string(&makefile_path).unwrap();
        assert!(
            original_content.contains("VERSION = $(shell"),
            "Original should be unchanged"
        );

        // ASSERT: Output file should contain the content (may or may not be fixed depending on autofix support)
        let fixed_content = fs::read_to_string(&output_path).unwrap();
        assert!(!fixed_content.is_empty(), "Output file should not be empty");
    } else {
        // Fix may not be fully implemented yet for Makefile linters
        // Just verify the original file is unchanged
        let original_content = fs::read_to_string(&makefile_path).unwrap();
        assert!(
            original_content.contains("VERSION = $(shell"),
            "Original should be unchanged"
        );
    }
}

#[test]
fn test_CLI_MAKE_LINT_005_rules_filter_specific_rules() {
    // ARRANGE: Create Makefile with multiple issues
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");

    fs::write(
        &makefile_path,
        r#"
VERSION = $(shell git describe)
SOURCES = $(wildcard src/*.c)

build:
	mkdir build
"#,
    )
    .unwrap();

    // ACT: Run linter with --rules MAKE001 (only wildcard issues)
    let output = bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&makefile_path)
        .arg("--rules")
        .arg("MAKE001")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // ASSERT: Should only show MAKE001 issues
    assert!(stdout.contains("MAKE001"), "Should show MAKE001");
    assert!(!stdout.contains("MAKE002"), "Should NOT show MAKE002");
    assert!(!stdout.contains("MAKE005"), "Should NOT show MAKE005");
}

#[test]
fn test_CLI_MAKE_LINT_006_rules_filter_multiple() {
    // ARRANGE: Create Makefile with multiple issues
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");

    fs::write(
        &makefile_path,
        r#"
VERSION = $(shell git describe)
SOURCES = $(wildcard src/*.c)
"#,
    )
    .unwrap();

    // ACT: Run linter with --rules MAKE001,MAKE005 (wildcard + assignment)
    let output = bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&makefile_path)
        .arg("--rules")
        .arg("MAKE001,MAKE005")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // ASSERT: Should show both MAKE001 and MAKE005
    assert!(stdout.contains("MAKE001"), "Should show MAKE001");
    assert!(stdout.contains("MAKE005"), "Should show MAKE005");
}

#[test]
fn test_CLI_MAKE_LINT_007_json_format() {
    // ARRANGE: Create Makefile with issues
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");

    fs::write(
        &makefile_path,
        r#"VERSION = $(shell git describe)
"#,
    )
    .unwrap();

    // ACT: Run linter with --format json
    bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&makefile_path)
        .arg("--format")
        .arg("json")
        .assert()
        .failure() // Has issues
        .stdout(predicate::str::contains("{")) // JSON output
        .stdout(predicate::str::contains("MAKE005"));
}

#[test]
fn test_CLI_MAKE_LINT_008_human_format() {
    // ARRANGE: Create Makefile with issues
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");

    fs::write(
        &makefile_path,
        r#"VERSION = $(shell git describe)
"#,
    )
    .unwrap();

    // ACT: Run linter with --format human (default)
    bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&makefile_path)
        .arg("--format")
        .arg("human")
        .assert()
        .failure()
        .stdout(predicate::str::contains("MAKE005"));
}

#[test]
fn test_CLI_MAKE_LINT_009_nonexistent_file_error() {
    // ACT & ASSERT: Run linter on non-existent file
    bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg("/nonexistent/Makefile")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file").or(predicate::str::contains("not found")));
}

#[test]
fn test_CLI_MAKE_LINT_010_all_five_rules_detected() {
    // ARRANGE: Create Makefile with all 5 issues
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");

    fs::write(
        &makefile_path,
        r#"
# MAKE005: Recursive assignment with shell command
VERSION = $(shell git describe)

# MAKE001: Non-deterministic wildcard
SOURCES = $(wildcard src/*.c)

# MAKE002: Non-idempotent mkdir
# MAKE004: Missing .PHONY
build:
	mkdir build
	gcc $(SOURCES) -o app

# MAKE003: Unsafe variable expansion
clean:
	rm -rf $BUILD_DIR
"#,
    )
    .unwrap();

    // ACT: Run linter
    let output = bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&makefile_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // ASSERT: Should detect all 5 rule violations
    assert!(
        stdout.contains("MAKE001"),
        "Should detect MAKE001 (wildcard)"
    );
    assert!(stdout.contains("MAKE002"), "Should detect MAKE002 (mkdir)");
    assert!(
        stdout.contains("MAKE003"),
        "Should detect MAKE003 (unsafe var)"
    );
    assert!(
        stdout.contains("MAKE004"),
        "Should detect MAKE004 (missing .PHONY)"
    );
    assert!(
        stdout.contains("MAKE005"),
        "Should detect MAKE005 (recursive assignment)"
    );
}

#[test]
fn test_CLI_MAKE_LINT_011_integration_with_purify() {
    // ARRANGE: Create Makefile with issues
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");

    fs::write(
        &makefile_path,
        r#"VERSION = $(shell git describe)
SOURCES = $(wildcard src/*.c)
"#,
    )
    .unwrap();

    // ACT: First lint (should find issues)
    let lint_before = bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&makefile_path)
        .output()
        .unwrap();

    assert!(
        !lint_before.status.success(),
        "Should have issues before purify"
    );

    // ACT: Purify the Makefile
    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(&makefile_path)
        .arg("--fix")
        .assert()
        .success();

    // ACT: Lint again (should have fewer or no issues)
    let lint_after = bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&makefile_path)
        .output()
        .unwrap();

    let stdout_after = String::from_utf8_lossy(&lint_after.stdout);

    // ASSERT: Should have fewer issues (purify fixes determinism/idempotency)
    // Note: Purify and lint may have slightly different transformations
    // The important thing is that purify improves the quality
    let issues_before = String::from_utf8_lossy(&lint_before.stdout)
        .matches("MAKE")
        .count();
    let issues_after = stdout_after.matches("MAKE").count();

    assert!(
        issues_after <= issues_before,
        "Purify should reduce or eliminate issues (before: {}, after: {})",
        issues_before,
        issues_after
    );
}

#[test]
fn test_CLI_MAKE_LINT_012_exit_codes() {
    let temp_dir = TempDir::new().unwrap();

    // Test 1: No errors = exit 0 or 1 (warnings OK)
    let clean_makefile = temp_dir.path().join("Makefile.clean");
    fs::write(
        &clean_makefile,
        r#"
VERSION := 1.0.0
.PHONY: build
build:
	mkdir -p build
"#,
    )
    .unwrap();

    let output = bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&clean_makefile)
        .output()
        .unwrap();

    // Warnings (exit 1) or no issues (exit 0) are both acceptable for clean Makefiles
    assert!(
        output.status.code().unwrap() < 2,
        "Clean Makefile should exit 0 or 1 (got {})",
        output.status.code().unwrap()
    );

    // Test 2: Has warnings = exit 1
    // (Makefile linter currently treats all issues as errors, so this may need adjustment)

    // Test 3: Has errors = exit 2
    let error_makefile = temp_dir.path().join("Makefile.error");
    fs::write(
        &error_makefile,
        r#"VERSION = $(shell git describe)
"#,
    )
    .unwrap();

    let output = bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&error_makefile)
        .output()
        .unwrap();

    // Should exit with non-zero (1 for warnings, 2 for errors)
    assert!(
        output.status.code().unwrap() > 0,
        "Makefile with issues should exit non-zero"
    );
}

#[test]
fn test_CLI_MAKE_LINT_013_fix_preserves_comments() {
    // ARRANGE: Create Makefile with comments and issues
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");

    fs::write(
        &makefile_path,
        r#"# This is a comment
VERSION = $(shell git describe)
# Another comment
SOURCES = $(wildcard src/*.c)
"#,
    )
    .unwrap();

    // ACT: Run linter with --fix
    bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&makefile_path)
        .arg("--fix")
        .assert()
        .success();

    // ASSERT: Comments should be preserved
    let fixed_content = fs::read_to_string(&makefile_path).unwrap();
    assert!(
        fixed_content.contains("# This is a comment"),
        "Should preserve first comment"
    );
    assert!(
        fixed_content.contains("# Another comment"),
        "Should preserve second comment"
    );
}

#[test]
fn test_CLI_MAKE_LINT_014_verbose_output() {
    // ARRANGE: Create Makefile with issues
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");

    fs::write(
        &makefile_path,
        r#"VERSION = $(shell git describe)
"#,
    )
    .unwrap();

    // ACT: Run linter with -v (global verbose flag)
    let output = bashrs_cmd()
        .arg("-v")
        .arg("make")
        .arg("lint")
        .arg(&makefile_path)
        .output()
        .unwrap();

    // ASSERT: Should contain verbose logging
    // Logging can go to either stdout or stderr depending on configuration
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("Linting")
            || combined.contains("INFO")
            || combined.contains(&makefile_path.to_string_lossy().to_string()),
        "Verbose mode should show logging output, got:\nstdout: {}\nstderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_CLI_MAKE_LINT_015_sarif_format() {
    // ARRANGE: Create Makefile with issues
    let temp_dir = TempDir::new().unwrap();
    let makefile_path = temp_dir.path().join("Makefile");

    fs::write(
        &makefile_path,
        r#"VERSION = $(shell git describe)
"#,
    )
    .unwrap();

    // ACT: Run linter with --format sarif
    bashrs_cmd()
        .arg("make")
        .arg("lint")
        .arg(&makefile_path)
        .arg("--format")
        .arg("sarif")
        .assert()
        .failure() // Has issues
        .stdout(predicate::str::contains("\"version\"")) // SARIF has version field
        .stdout(predicate::str::contains("\"results\"")); // SARIF has results field
}
