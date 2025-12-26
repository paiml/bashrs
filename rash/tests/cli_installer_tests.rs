//! CLI Integration tests for installer command
//!
//! These tests follow EXTREME TDD principles and use assert_cmd as MANDATORY per CLAUDE.md.
//! Test naming convention: test_<TASK_ID>_<feature>_<scenario>

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper function to create bashrs command (MANDATORY pattern per CLAUDE.md)
fn bashrs_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
}

// =============================================================================
// INSTALLER_CLI_001: Init command tests
// =============================================================================

#[test]
fn test_INSTALLER_CLI_001_init_creates_project() {
    let temp_dir = TempDir::new().unwrap();
    let project_name = "test-installer";
    let project_path = temp_dir.path().join(project_name);

    bashrs_cmd()
        .arg("installer")
        .arg("init")
        .arg(&project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized installer project"))
        .stdout(predicate::str::contains("installer.toml"));

    // Verify project structure
    assert!(project_path.join("installer.toml").exists());
    assert!(project_path.join("tests").exists());
    assert!(project_path.join("tests").join("mod.rs").exists());
    assert!(project_path.join("tests").join("falsification.rs").exists());
    assert!(project_path.join("templates").exists());
}

#[test]
fn test_INSTALLER_CLI_001_init_with_description() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("my-project");

    bashrs_cmd()
        .arg("installer")
        .arg("init")
        .arg(&project_path)
        .arg("--description")
        .arg("My custom installer")
        .assert()
        .success();

    // Verify description in installer.toml
    let content = fs::read_to_string(project_path.join("installer.toml")).unwrap();
    assert!(content.contains("My custom installer"));
}

// =============================================================================
// INSTALLER_CLI_002: Validate command tests
// =============================================================================

#[test]
fn test_INSTALLER_CLI_002_validate_valid_installer() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("valid-installer");

    // First create a valid installer
    bashrs_cmd()
        .arg("installer")
        .arg("init")
        .arg(&project_path)
        .assert()
        .success();

    // Now validate it
    bashrs_cmd()
        .arg("installer")
        .arg("validate")
        .arg(&project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Installer is valid"));
}

#[test]
fn test_INSTALLER_CLI_002_validate_missing_installer() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent");

    bashrs_cmd()
        .arg("installer")
        .arg("validate")
        .arg(&nonexistent)
        .assert()
        .failure();
}

#[test]
fn test_INSTALLER_CLI_002_validate_invalid_toml() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Write invalid TOML
    fs::write(project_path.join("installer.toml"), "INVALID [[[").unwrap();

    bashrs_cmd()
        .arg("installer")
        .arg("validate")
        .arg(project_path)
        .assert()
        .failure();
}

// =============================================================================
// INSTALLER_CLI_003: Run command tests (dry-run mode)
// =============================================================================

#[test]
fn test_INSTALLER_CLI_003_run_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("installer");

    // Create installer first
    bashrs_cmd()
        .arg("installer")
        .arg("init")
        .arg(&project_path)
        .assert()
        .success();

    // Run in dry-run mode
    bashrs_cmd()
        .arg("installer")
        .arg("run")
        .arg(&project_path)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry-run mode"));
}

#[test]
fn test_INSTALLER_CLI_003_run_dry_run_diff() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("installer");

    // Create installer first
    bashrs_cmd()
        .arg("installer")
        .arg("init")
        .arg(&project_path)
        .assert()
        .success();

    // Run in dry-run with diff
    bashrs_cmd()
        .arg("installer")
        .arg("run")
        .arg(&project_path)
        .arg("--dry-run")
        .arg("--diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry-Run Diff Preview"));
}

// =============================================================================
// INSTALLER_CLI_004: Graph command tests
// =============================================================================

#[test]
fn test_INSTALLER_CLI_004_graph_mermaid() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("installer");

    // Create installer first
    bashrs_cmd()
        .arg("installer")
        .arg("init")
        .arg(&project_path)
        .assert()
        .success();

    // Generate mermaid graph
    bashrs_cmd()
        .arg("installer")
        .arg("graph")
        .arg(&project_path)
        .arg("--format")
        .arg("mermaid")
        .assert()
        .success()
        .stdout(predicate::str::contains("```mermaid"))
        .stdout(predicate::str::contains("graph TD"));
}

#[test]
fn test_INSTALLER_CLI_004_graph_dot() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("installer");

    // Create installer first
    bashrs_cmd()
        .arg("installer")
        .arg("init")
        .arg(&project_path)
        .assert()
        .success();

    // Generate DOT graph
    bashrs_cmd()
        .arg("installer")
        .arg("graph")
        .arg(&project_path)
        .arg("--format")
        .arg("dot")
        .assert()
        .success()
        .stdout(predicate::str::contains("digraph installer"));
}

#[test]
fn test_INSTALLER_CLI_004_graph_json() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("installer");

    // Create installer first
    bashrs_cmd()
        .arg("installer")
        .arg("init")
        .arg(&project_path)
        .assert()
        .success();

    // Generate JSON graph
    bashrs_cmd()
        .arg("installer")
        .arg("graph")
        .arg(&project_path)
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"nodes\""));
}

// =============================================================================
// INSTALLER_CLI_005: Lock command tests
// =============================================================================

#[test]
fn test_INSTALLER_CLI_005_lock_generate() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("installer");

    // Create installer first
    bashrs_cmd()
        .arg("installer")
        .arg("init")
        .arg(&project_path)
        .assert()
        .success();

    // Generate lockfile
    bashrs_cmd()
        .arg("installer")
        .arg("lock")
        .arg(&project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Managing lockfile"));
}

// =============================================================================
// INSTALLER_CLI_006: Test matrix command tests
// =============================================================================

#[test]
fn test_INSTALLER_CLI_006_test_matrix() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("installer");

    // Create installer first
    bashrs_cmd()
        .arg("installer")
        .arg("init")
        .arg(&project_path)
        .assert()
        .success();

    // Run tests with matrix
    bashrs_cmd()
        .arg("installer")
        .arg("test")
        .arg(&project_path)
        .arg("--matrix")
        .arg("ubuntu:22.04,debian:12")
        .assert()
        .success()
        .stdout(predicate::str::contains("Container Test Matrix"))
        .stdout(predicate::str::contains("ubuntu:22.04"))
        .stdout(predicate::str::contains("debian:12"));
}

// =============================================================================
// INSTALLER_CLI_007: Keyring command tests
// =============================================================================

#[test]
fn test_INSTALLER_CLI_007_keyring_init() {
    bashrs_cmd()
        .arg("installer")
        .arg("keyring")
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized keyring"));
}

#[test]
fn test_INSTALLER_CLI_007_keyring_list() {
    bashrs_cmd()
        .arg("installer")
        .arg("keyring")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Keyring contents"));
}

// =============================================================================
// INSTALLER_CLI_008: Help command tests
// =============================================================================

#[test]
fn test_INSTALLER_CLI_008_help_shows_subcommands() {
    bashrs_cmd()
        .arg("installer")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("validate"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("graph"));
}

#[test]
fn test_INSTALLER_CLI_008_init_help() {
    bashrs_cmd()
        .arg("installer")
        .arg("init")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("TDD-first test harness"));
}
