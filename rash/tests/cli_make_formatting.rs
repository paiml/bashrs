#![allow(deprecated)]
#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)]
//! Tests for Makefile formatting options
//!
//! Tests the --preserve-formatting, --max-line-length, and selective transformation flags
//! added as part of the dogfooding improvements (Issue #1 follow-up).
//!
//! ## Test Coverage
//! - RED Phase: Write failing tests first (EXTREME TDD)
//! - GREEN Phase: Implement feature to make tests pass
//! - REFACTOR Phase: Clean up implementation
//!
//! ## Related
//! - docs/dogfooding/makefile-purification.md
//! - rash/src/cli/args.rs (MakeCommands::Purify)
//! - rash/src/cli/commands.rs (handle_make_purify)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to create test command
#[allow(deprecated)]
fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

/// Sample Makefile with formatting issues
const SAMPLE_MAKEFILE: &str = r#"# Sample Makefile
.PHONY: all

all: build test

build:
	@if command -v cargo >/dev/null 2>&1; then \
		cargo build --release; \
	else \
		echo "cargo not found"; \
	fi
	@echo "Build complete"

test:
	cargo test
"#;

#[test]
fn test_make_formatting_001_preserve_formatting_flag_exists() {
    // RED: This test should FAIL initially because --preserve-formatting doesn't exist yet

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Makefile");
    let output_file = temp_dir.path().join("Makefile.purified");

    fs::write(&input_file, SAMPLE_MAKEFILE).unwrap();

    // Test that --preserve-formatting flag is recognized
    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(&input_file)
        .arg("--preserve-formatting")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success(); // Should succeed (not fail with "unknown argument")
}

#[test]
fn test_make_formatting_002_preserve_formatting_keeps_blank_lines() {
    // RED: This test should FAIL because --preserve-formatting not implemented

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Makefile");
    let output_file = temp_dir.path().join("Makefile.purified");

    let makefile_with_blank_lines = r#"# Section 1
.PHONY: all

all: build

# Section 2

build:
	echo "Building"
"#;

    fs::write(&input_file, makefile_with_blank_lines).unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(&input_file)
        .arg("--preserve-formatting")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    // Check output preserves blank lines
    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should contain blank lines between sections
    assert!(
        output_content.contains("\n\n"),
        "Expected blank lines to be preserved with --preserve-formatting"
    );
}

#[test]
fn test_make_formatting_003_preserve_formatting_keeps_multiline_format() {
    // ✅ FIXED (Issue #2): Parser now tracks line continuation metadata
    // in the AST and generator reconstructs original backslash continuations
    // when --preserve-formatting is used.

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Makefile");
    let output_file = temp_dir.path().join("Makefile.purified");

    fs::write(&input_file, SAMPLE_MAKEFILE).unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(&input_file)
        .arg("--preserve-formatting")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should preserve multi-line if/then/else format (with backslashes)
    assert!(
        output_content.contains(r#"then \"#) || output_content.contains("then\n"),
        "Expected multi-line format to be preserved with --preserve-formatting"
    );
}

#[test]
fn test_make_formatting_004_without_preserve_formatting_compacts() {
    // This should PASS already (existing behavior)

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Makefile");
    let output_file = temp_dir.path().join("Makefile.purified");

    let makefile_with_blank_lines = r#"# Section 1

.PHONY: all

all: build

build:
	echo "Building"
"#;

    fs::write(&input_file, makefile_with_blank_lines).unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(&input_file)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let input_lines = makefile_with_blank_lines.lines().count();
    let output_content = fs::read_to_string(&output_file).unwrap();
    let output_lines = output_content.lines().count();

    // Without --preserve-formatting, should compact (fewer lines)
    assert!(
        output_lines < input_lines,
        "Expected compacted output without --preserve-formatting (output: {}, input: {})",
        output_lines,
        input_lines
    );
}

#[test]
fn test_make_formatting_005_max_line_length_flag_exists() {
    // RED: This test should FAIL initially because --max-line-length doesn't exist yet

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Makefile");
    let output_file = temp_dir.path().join("Makefile.purified");

    fs::write(&input_file, SAMPLE_MAKEFILE).unwrap();

    // Test that --max-line-length flag is recognized
    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(&input_file)
        .arg("--max-line-length")
        .arg("120")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success(); // Should succeed (not fail with "unknown argument")
}

#[test]
fn test_make_formatting_006_max_line_length_breaks_long_lines() {
    // RED: This test should FAIL because --max-line-length not implemented

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Makefile");
    let output_file = temp_dir.path().join("Makefile.purified");

    let makefile_with_long_line = r#"build:
	@if command -v very_long_command_name >/dev/null 2>&1; then very_long_command_name --with-many-flags --and-options --that-make-it-exceed-line-length-limits; else echo "not found"; fi
"#;

    fs::write(&input_file, makefile_with_long_line).unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(&input_file)
        .arg("--max-line-length")
        .arg("80")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Check that no line exceeds 80 characters
    for line in output_content.lines() {
        assert!(
            line.len() <= 80,
            "Line exceeds max length of 80: {} chars: '{}'",
            line.len(),
            line
        );
    }
}

#[test]
fn test_make_formatting_007_skip_blank_line_removal_flag_exists() {
    // RED: This test should FAIL initially because --skip-blank-line-removal doesn't exist yet

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Makefile");
    let output_file = temp_dir.path().join("Makefile.purified");

    fs::write(&input_file, SAMPLE_MAKEFILE).unwrap();

    // Test that --skip-blank-line-removal flag is recognized
    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(&input_file)
        .arg("--skip-blank-line-removal")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success(); // Should succeed (not fail with "unknown argument")
}

#[test]
fn test_make_formatting_008_skip_consolidation_flag_exists() {
    // RED: This test should FAIL initially because --skip-consolidation doesn't exist yet

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Makefile");
    let output_file = temp_dir.path().join("Makefile.purified");

    fs::write(&input_file, SAMPLE_MAKEFILE).unwrap();

    // Test that --skip-consolidation flag is recognized
    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(&input_file)
        .arg("--skip-consolidation")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success(); // Should succeed (not fail with "unknown argument")
}

#[test]
fn test_make_formatting_009_skip_consolidation_preserves_multiline() {
    // ✅ FIXED (Issue #2): Parser now tracks line continuation metadata
    // and generator reconstructs backslash continuations with --skip-consolidation.

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Makefile");
    let output_file = temp_dir.path().join("Makefile.purified");

    fs::write(&input_file, SAMPLE_MAKEFILE).unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(&input_file)
        .arg("--skip-consolidation")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should preserve multi-line format (contains backslash continuation)
    assert!(
        output_content.contains('\\'),
        "Expected backslash continuations with --skip-consolidation"
    );
}

#[test]
fn test_make_formatting_010_help_shows_new_flags() {
    // Verify all new flags appear in help output

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--preserve-formatting"))
        .stdout(predicate::str::contains("--max-line-length"))
        .stdout(predicate::str::contains("--skip-blank-line-removal"))
        .stdout(predicate::str::contains("--skip-consolidation"));
}

#[test]
fn test_make_formatting_011_combined_flags() {
    // Test combining multiple formatting flags

    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("Makefile");
    let output_file = temp_dir.path().join("Makefile.purified");

    fs::write(&input_file, SAMPLE_MAKEFILE).unwrap();

    // Combine --skip-blank-line-removal and --max-line-length
    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(&input_file)
        .arg("--skip-blank-line-removal")
        .arg("--max-line-length")
        .arg("100")
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    let output_content = fs::read_to_string(&output_file).unwrap();

    // Should have blank lines AND respect line length
    assert!(
        output_content.contains("\n\n"),
        "Expected blank lines preserved"
    );

    for line in output_content.lines() {
        assert!(line.len() <= 100, "Line exceeds max length: {}", line.len());
    }
}
