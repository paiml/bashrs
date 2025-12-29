#![allow(deprecated)]
#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)]
//! CLI tests for `bashrs config` commands
//!
//! Tests following EXTREME TDD - these tests are written FIRST (RED phase)
//! before implementing the CLI commands.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to create bashrs command
#[allow(deprecated)]
fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

/// Helper to create a messy .bashrc fixture
fn create_messy_bashrc(dir: &TempDir) -> std::path::PathBuf {
    let bashrc_path = dir.path().join(".bashrc");
    let content = r#"# Messy .bashrc with duplicates
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH"
export EDITOR=vim
alias ll='ls -la'
"#;
    fs::write(&bashrc_path, content).expect("Failed to write fixture");
    bashrc_path
}

// =============================================================================
// TEST: bashrs config analyze
// =============================================================================

#[test]
fn test_config_analyze_basic() {
    // ARRANGE: Create temp directory with messy .bashrc
    let temp_dir = TempDir::new().unwrap();
    let bashrc = create_messy_bashrc(&temp_dir);

    // ACT: Run bashrs config analyze
    bashrs_cmd()
        .arg("config")
        .arg("analyze")
        .arg(&bashrc)
        .assert()
        .success()
        .stdout(predicate::str::contains("Analysis:"))
        .stdout(predicate::str::contains("CONFIG-001")); // Should detect duplicates
}

#[test]
fn test_config_analyze_shows_issues_count() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let bashrc = create_messy_bashrc(&temp_dir);

    // ACT
    bashrs_cmd()
        .arg("config")
        .arg("analyze")
        .arg(&bashrc)
        .assert()
        .success()
        .stdout(predicate::str::contains("Issues Found:")); // Should show count
}

#[test]
fn test_config_analyze_shows_path_entries() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let bashrc = create_messy_bashrc(&temp_dir);

    // ACT
    bashrs_cmd()
        .arg("config")
        .arg("analyze")
        .arg(&bashrc)
        .assert()
        .success()
        .stdout(predicate::str::contains("/usr/local/bin"))
        .stdout(predicate::str::contains("/opt/homebrew/bin"));
}

#[test]
fn test_config_analyze_nonexistent_file() {
    // ACT: Try to analyze non-existent file
    bashrs_cmd()
        .arg("config")
        .arg("analyze")
        .arg("/nonexistent/.bashrc")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("No such file")));
}

// =============================================================================
// TEST: bashrs config lint
// =============================================================================

#[test]
fn test_config_lint_detects_duplicates() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let bashrc = create_messy_bashrc(&temp_dir);

    // ACT
    bashrs_cmd()
        .arg("config")
        .arg("lint")
        .arg(&bashrc)
        .assert()
        .code(1) // Should exit with code 1 (warnings found)
        .stdout(predicate::str::contains("CONFIG-001"))
        .stdout(predicate::str::contains("Duplicate PATH entry"));
}

#[test]
fn test_config_lint_clean_file_exits_zero() {
    // ARRANGE: Create clean .bashrc with no duplicates
    let temp_dir = TempDir::new().unwrap();
    let bashrc = temp_dir.path().join(".bashrc");
    fs::write(
        &bashrc,
        r#"export PATH="/usr/local/bin:$PATH"
export EDITOR=vim
"#,
    )
    .unwrap();

    // ACT
    bashrs_cmd()
        .arg("config")
        .arg("lint")
        .arg(&bashrc)
        .assert()
        .success() // No issues = exit 0
        .stdout(predicate::str::contains("No issues found"));
}

#[test]
fn test_config_lint_json_format() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let bashrc = create_messy_bashrc(&temp_dir);

    // ACT
    bashrs_cmd()
        .arg("config")
        .arg("lint")
        .arg(&bashrc)
        .arg("--format=json")
        .assert()
        .code(1)
        .stdout(predicate::str::contains(r#""rule_id""#))
        .stdout(predicate::str::contains("CONFIG-001"));
}

// =============================================================================
// TEST: bashrs config purify
// =============================================================================

#[test]
fn test_config_purify_dry_run() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let bashrc = create_messy_bashrc(&temp_dir);

    // ACT: Run with --dry-run (default)
    bashrs_cmd()
        .arg("config")
        .arg("purify")
        .arg(&bashrc)
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview of changes"))
        .stdout(predicate::str::contains("CONFIG-001")); // Should show what would be fixed

    // ASSERT: File should NOT be modified
    let content = fs::read_to_string(&bashrc).unwrap();
    assert!(content.contains("export PATH=\"/usr/local/bin:$PATH\""));
    // Should still have duplicates
    let path_count = content
        .lines()
        .filter(|l| l.contains("export PATH="))
        .count();
    assert_eq!(path_count, 3);
}

#[test]
fn test_config_purify_with_fix() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let bashrc = create_messy_bashrc(&temp_dir);
    let _backup_pattern = format!("{}.bak.*", bashrc.display());

    // ACT: Run with --fix
    bashrs_cmd()
        .arg("config")
        .arg("purify")
        .arg(&bashrc)
        .arg("--fix")
        .assert()
        .success()
        .stdout(predicate::str::contains("Applying"))
        .stdout(predicate::str::contains("fixes"))
        .stdout(predicate::str::contains("Backup:"));

    // ASSERT: File should be modified
    let content = fs::read_to_string(&bashrc).unwrap();

    // Should have fewer PATH entries
    let path_count = content
        .lines()
        .filter(|l| l.contains("export PATH="))
        .count();
    assert_eq!(path_count, 2, "Should have only 2 unique PATH entries");

    // Backup should exist
    let backup_files: Vec<_> = fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_str().unwrap().contains(".bak"))
        .collect();

    assert_eq!(backup_files.len(), 1, "Should create exactly one backup");
}

#[test]
fn test_config_purify_no_backup_flag() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let bashrc = create_messy_bashrc(&temp_dir);

    // ACT: Run with --fix --no-backup
    bashrs_cmd()
        .arg("config")
        .arg("purify")
        .arg(&bashrc)
        .arg("--fix")
        .arg("--no-backup")
        .assert()
        .success();

    // ASSERT: No backup should be created
    let backup_files: Vec<_> = fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_str().unwrap().contains(".bak"))
        .collect();

    assert_eq!(backup_files.len(), 0, "Should not create backup");
}

#[test]
fn test_config_purify_output_to_stdout() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let bashrc = create_messy_bashrc(&temp_dir);

    // ACT: Run without --fix, output should go to stdout
    bashrs_cmd()
        .arg("config")
        .arg("purify")
        .arg(&bashrc)
        .arg("--output")
        .arg("-")
        .assert()
        .success()
        .stdout(predicate::str::contains("export PATH="));

    // Original file should be unchanged
    let content = fs::read_to_string(&bashrc).unwrap();
    let path_count = content
        .lines()
        .filter(|l| l.contains("export PATH="))
        .count();
    assert_eq!(path_count, 3, "Original file should be unchanged");
}

#[test]
fn test_config_purify_output_to_file() {
    // ARRANGE
    let temp_dir = TempDir::new().unwrap();
    let bashrc = create_messy_bashrc(&temp_dir);
    let output_file = temp_dir.path().join(".bashrc.purified");

    // ACT: Run with --output
    bashrs_cmd()
        .arg("config")
        .arg("purify")
        .arg(&bashrc)
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();

    // ASSERT: Output file should exist with purified content
    assert!(output_file.exists(), "Output file should be created");

    let content = fs::read_to_string(&output_file).unwrap();
    let path_count = content
        .lines()
        .filter(|l| l.contains("export PATH="))
        .count();
    assert_eq!(path_count, 2, "Output should have deduplicated PATH");

    // Original should be unchanged
    let original = fs::read_to_string(&bashrc).unwrap();
    let original_count = original
        .lines()
        .filter(|l| l.contains("export PATH="))
        .count();
    assert_eq!(original_count, 3, "Original should be unchanged");
}

// =============================================================================
// TEST: Integration with real fixture
// =============================================================================

#[test]
fn test_config_with_real_fixture() {
    // ARRANGE: Use the real fixture from tests/fixtures
    let fixture_path = "tests/fixtures/configs/messy-bashrc.sh";

    // Skip if fixture doesn't exist (CI environment)
    if !std::path::Path::new(fixture_path).exists() {
        eprintln!("Skipping test - fixture not found: {}", fixture_path);
        return;
    }

    // ACT: Analyze the real fixture
    bashrs_cmd()
        .arg("config")
        .arg("analyze")
        .arg(fixture_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("CONFIG-001"))
        .stdout(predicate::str::contains("Issues Found:"));

    // ACT: Lint the real fixture
    bashrs_cmd()
        .arg("config")
        .arg("lint")
        .arg(fixture_path)
        .assert()
        .code(1) // Should have warnings
        .stdout(predicate::str::contains("CONFIG-001"));
}

// =============================================================================
// TEST: Error handling
// =============================================================================

#[test]
fn test_config_missing_subcommand() {
    // ACT: Run config without subcommand
    bashrs_cmd()
        .arg("config")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("COMMAND")));
}

#[test]
fn test_config_invalid_subcommand() {
    // ACT
    bashrs_cmd()
        .arg("config")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized").or(predicate::str::contains("invalid")));
}

#[test]
fn test_config_help() {
    // ACT
    bashrs_cmd()
        .arg("config")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("analyze"))
        .stdout(predicate::str::contains("lint"))
        .stdout(predicate::str::contains("purify"));
}
