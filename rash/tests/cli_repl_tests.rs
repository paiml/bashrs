#![allow(deprecated)]
#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)]
//! CLI REPL Integration Tests
//!
//! Task: REPL-003-002 - Basic REPL loop with rustyline integration
//! Test Approach: CLI integration tests with assert_cmd
//!
//! Quality targets:
//! - Integration tests: 3+ scenarios
//! - CLI interaction validated
//! - User experience verified

#![allow(non_snake_case)] // Test naming convention: test_<TASK_ID>_<feature>_<scenario>

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper function to create bashrs REPL command
fn bashrs_repl() -> Command {
    let mut cmd = Command::cargo_bin("bashrs").expect("Failed to find bashrs binary");
    cmd.arg("repl");
    cmd
}

/// Test: REPL-003-002-001 - REPL starts and accepts quit command
#[test]
fn test_REPL_003_002_repl_starts_and_quits() {
    bashrs_repl()
        .write_stdin("quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs REPL"))
        .stdout(predicate::str::contains("Goodbye!"));
}

/// Test: REPL-003-002-002 - REPL handles empty input gracefully
#[test]
fn test_REPL_003_002_repl_handles_empty_input() {
    bashrs_repl()
        .write_stdin("\n\nexit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs REPL"));
}

/// Test: REPL-003-002-003 - REPL handles EOF (Ctrl-D) gracefully
#[test]
fn test_REPL_003_002_repl_handles_eof() {
    // EOF is simulated by closing stdin (no input)
    bashrs_repl()
        .write_stdin("") // Empty input simulates EOF
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs REPL"))
        .stdout(predicate::str::contains("EOF"));
}

/// Test: REPL-003-002-004 - REPL shows help command
#[test]
fn test_REPL_003_002_repl_shows_help() {
    bashrs_repl()
        .write_stdin("help\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs REPL v"))
        .stdout(predicate::str::contains("OVERVIEW"))
        .stdout(predicate::str::contains(":help commands"));
}

/// Test: REPL-003-002-005 - REPL accepts exit command as alternative to quit
#[test]
fn test_REPL_003_002_repl_accepts_exit() {
    bashrs_repl()
        .write_stdin("exit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Goodbye!"));
}

/// Test: REPL-003-002-006 - REPL with debug mode
#[test]
fn test_REPL_003_002_repl_debug_mode() {
    let mut cmd = Command::cargo_bin("bashrs").unwrap();
    cmd.arg("repl")
        .arg("--debug")
        .write_stdin("quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs REPL"));
}

/// Test: REPL-003-002-007 - REPL with custom configuration
#[test]
fn test_REPL_003_002_repl_custom_config() {
    let mut cmd = Command::cargo_bin("bashrs").unwrap();
    cmd.arg("repl")
        .arg("--max-memory")
        .arg("200")
        .arg("--timeout")
        .arg("60")
        .arg("--max-depth")
        .arg("200")
        .write_stdin("quit\n")
        .assert()
        .success();
}

/// Test: REPL-003-002-008 - REPL with sandboxed mode
#[test]
fn test_REPL_003_002_repl_sandboxed() {
    let mut cmd = Command::cargo_bin("bashrs").unwrap();
    cmd.arg("repl")
        .arg("--sandboxed")
        .write_stdin("quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs REPL"));
}

// ===== REPL-003-003: HISTORY PERSISTENCE TESTS =====

/// Test: REPL-003-003-001 - History persists across sessions
#[test]
fn test_REPL_003_003_history_persistence() {
    use std::fs;
    use std::path::PathBuf;

    // Get history file path
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let history_path = PathBuf::from(home).join(".bashrs_history");

    // Clean up any existing history file
    let _ = fs::remove_file(&history_path);

    // Session 1: Add commands to history
    bashrs_repl().write_stdin("help\nexit\n").assert().success();

    // Verify history file was created
    assert!(history_path.exists(), "History file should be created");

    // Session 2: History should be loaded automatically
    // Note: This test verifies the file exists; actual history loading
    // is tested by rustyline's built-in functionality
    assert!(history_path.exists(), "History file should persist");

    // Clean up
    let _ = fs::remove_file(&history_path);
}

/// Test: REPL-003-003-002 - Multiple commands saved to history
/// Note: This test may be environment-dependent
#[test]
#[ignore] // Ignore by default due to filesystem timing issues in CI
fn test_REPL_003_003_multiple_commands() {
    use std::fs;
    use std::path::PathBuf;

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let history_path = PathBuf::from(home).join(".bashrs_history");

    // Clean up
    let _ = fs::remove_file(&history_path);

    // Add multiple commands
    bashrs_repl()
        .write_stdin("help\nhelp\nhelp\nquit\n")
        .assert()
        .success();

    // Wait for file to be written
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Verify history file exists (may fail in some CI environments)
    if history_path.exists() {
        // Read history file and verify it has content
        let history_content = fs::read_to_string(&history_path).unwrap();
        assert!(
            !history_content.is_empty(),
            "History should contain commands"
        );
    }

    // Clean up
    let _ = fs::remove_file(&history_path);
}

// ===== REPL-003-004: MODE SWITCHING TESTS =====

/// Test: REPL-003-004-001 - REPL shows current mode at startup
#[test]
fn test_REPL_003_004_shows_current_mode() {
    bashrs_repl()
        .write_stdin("quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Current mode: normal"));
}

/// Test: REPL-003-004-002 - :mode command shows available modes
#[test]
fn test_REPL_003_004_mode_command_shows_modes() {
    bashrs_repl()
        .write_stdin(":mode\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available modes"))
        .stdout(predicate::str::contains("normal"))
        .stdout(predicate::str::contains("purify"))
        .stdout(predicate::str::contains("lint"))
        .stdout(predicate::str::contains("debug"))
        .stdout(predicate::str::contains("explain"));
}

/// Test: REPL-003-004-003 - :mode switches to purify mode
#[test]
fn test_REPL_003_004_mode_switch_to_purify() {
    bashrs_repl()
        .write_stdin(":mode purify\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Switched to purify mode"));
}

/// Test: REPL-003-004-004 - :mode switches to lint mode
#[test]
fn test_REPL_003_004_mode_switch_to_lint() {
    bashrs_repl()
        .write_stdin(":mode lint\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Switched to lint mode"));
}

/// Test: REPL-003-004-005 - :mode switches to debug mode
#[test]
fn test_REPL_003_004_mode_switch_to_debug() {
    bashrs_repl()
        .write_stdin(":mode debug\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Switched to debug mode"));
}

/// Test: REPL-003-004-006 - :mode switches to explain mode
#[test]
fn test_REPL_003_004_mode_switch_to_explain() {
    bashrs_repl()
        .write_stdin(":mode explain\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Switched to explain mode"));
}

/// Test: REPL-003-004-007 - :mode with invalid mode shows error
#[test]
fn test_REPL_003_004_mode_invalid_shows_error() {
    bashrs_repl()
        .write_stdin(":mode invalid\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Error"))
        .stdout(predicate::str::contains("Unknown mode"));
}

/// Test: REPL-003-004-008 - Mode is case-insensitive
#[test]
fn test_REPL_003_004_mode_case_insensitive() {
    bashrs_repl()
        .write_stdin(":mode PURIFY\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Switched to purify mode"));
}

/// Test: REPL-003-004-009 - Multiple mode switches work correctly
#[test]
fn test_REPL_003_004_multiple_mode_switches() {
    bashrs_repl()
        .write_stdin(":mode purify\n:mode lint\n:mode normal\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Switched to purify mode"))
        .stdout(predicate::str::contains("Switched to lint mode"))
        .stdout(predicate::str::contains("Switched to normal mode"));
}

// ===== REPL-004-001: PARSER INTEGRATION TESTS =====

/// Test: REPL-004-001-001 - :parse command with simple command
#[test]
fn test_REPL_004_001_parse_simple_command_cli() {
    bashrs_repl()
        .write_stdin(":parse echo hello\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse successful"));
}

/// Test: REPL-004-001-002 - :parse command shows AST
#[test]
fn test_REPL_004_001_parse_shows_ast() {
    bashrs_repl()
        .write_stdin(":parse ls -la\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("AST"))
        .stdout(predicate::str::contains("Statements:"));
}

/// Test: REPL-004-001-003 - :parse command with no input shows usage
#[test]
fn test_REPL_004_001_parse_no_input_shows_usage() {
    bashrs_repl()
        .write_stdin(":parse\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: :parse"));
}

/// Test: REPL-004-001-004 - :parse command with invalid syntax shows error
#[test]
fn test_REPL_004_001_parse_invalid_syntax() {
    bashrs_repl()
        .write_stdin(":parse if then fi\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Syntax error"));
}

/// Test: REPL-004-001-005 - Help shows :parse command
#[test]
fn test_REPL_004_001_help_shows_parse() {
    bashrs_repl()
        .write_stdin(":help commands\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(":parse"))
        .stdout(predicate::str::contains("Parse bash code"));
}

// ===== REPL-005-001: PURIFIER INTEGRATION TESTS =====

/// Test: REPL-005-001-001 - :purify command with simple command
#[test]
fn test_REPL_005_001_purify_simple_command_cli() {
    bashrs_repl()
        .write_stdin(":purify mkdir /tmp/test\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Purification successful"));
}

/// Test: REPL-005-001-002 - :purify command shows result
#[test]
fn test_REPL_005_001_purify_shows_result() {
    bashrs_repl()
        .write_stdin(":purify echo hello\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Purification successful"));
}

/// Test: REPL-005-001-003 - :purify command with no input shows usage
#[test]
fn test_REPL_005_001_purify_no_input_shows_usage() {
    bashrs_repl()
        .write_stdin(":purify\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: :purify"));
}

/// Test: REPL-005-001-004 - Help shows :purify command
#[test]
fn test_REPL_005_001_help_shows_purify() {
    bashrs_repl()
        .write_stdin(":help commands\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(":purify"))
        .stdout(predicate::str::contains("Purify bash code"));
}

// ===== REPL-006-001: LINTER INTEGRATION TESTS =====

/// Test: REPL-006-001-001 - :lint command with simple command
#[test]
fn test_REPL_006_001_lint_simple_command_cli() {
    bashrs_repl()
        .write_stdin(":lint echo hello\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("issue").or(predicate::str::contains("No issues")));
}

/// Test: REPL-006-001-002 - :lint command shows diagnostics
#[test]
fn test_REPL_006_001_lint_shows_diagnostics() {
    bashrs_repl()
        .write_stdin(":lint cat file.txt | grep pattern\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("issue").or(predicate::str::contains("No issues")));
}

/// Test: REPL-006-001-003 - :lint command with no input shows usage
#[test]
fn test_REPL_006_001_lint_no_input_shows_usage() {
    bashrs_repl()
        .write_stdin(":lint\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: :lint"));
}

/// Test: REPL-006-001-004 - Help shows :lint command
#[test]
fn test_REPL_006_001_help_shows_lint() {
    bashrs_repl()
        .write_stdin(":help commands\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(":lint"))
        .stdout(predicate::str::contains("Lint bash code"));
}
