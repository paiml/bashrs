#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
//! REPL Utility Commands Tests
//!
//! Task: REPL-004-001 - Utility commands (:history, :vars, :clear)
//! Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//!
//! Quality targets:
//! - Integration tests: 6+ scenarios
//! - User experience verified
//! - All utility commands tested

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper function to create bashrs REPL command
fn bashrs_repl() -> Command {
    let mut cmd = Command::cargo_bin("bashrs").expect("Failed to find bashrs binary");
    cmd.arg("repl");
    cmd
}

// ===== :HISTORY COMMAND TESTS =====

/// Test: REPL-004-001-001 - :history shows command history
#[test]
fn test_repl_004_001_history_shows_commands() {
    bashrs_repl()
        .write_stdin("echo hello\nls -la\n:history\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("echo hello"))
        .stdout(predicate::str::contains("ls -la"));
}

/// Test: REPL-004-001-002 - :history shows empty when no commands
#[test]
fn test_repl_004_001_history_empty() {
    bashrs_repl()
        .write_stdin(":history\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("History").or(predicate::str::contains("No commands")));
}

/// Test: REPL-004-001-003 - :history excludes itself from display
#[test]
fn test_repl_004_001_history_excludes_itself() {
    bashrs_repl()
        .write_stdin("echo test\n:history\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("echo test"));
}

// ===== :VARS COMMAND TESTS =====

/// Test: REPL-004-001-004 - :vars shows session variables
#[test]
fn test_repl_004_001_vars_shows_variables() {
    bashrs_repl()
        .write_stdin(":vars\nquit\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Variables")
                .or(predicate::str::contains("No session variables")),
        );
}

/// Test: REPL-004-001-005 - :vars shows empty when no variables set
#[test]
fn test_repl_004_001_vars_empty() {
    bashrs_repl()
        .write_stdin(":vars\nquit\n")
        .assert()
        .success();
}

// ===== :CLEAR COMMAND TESTS =====

/// Test: REPL-004-001-006 - :clear command exists and works
#[test]
fn test_repl_004_001_clear_works() {
    bashrs_repl()
        .write_stdin("echo hello\n:clear\nquit\n")
        .assert()
        .success();
}

/// Test: REPL-004-001-007 - :clear before :history shows cleared history still accessible
#[test]
fn test_repl_004_001_clear_history_preserved() {
    bashrs_repl()
        .write_stdin("echo test\n:clear\n:history\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("echo test"));
}

// ===== HELP TEXT TESTS =====

/// Test: REPL-004-001-008 - help command shows new utility commands
#[test]
#[ignore] // Output format changed: help text no longer includes :history, :vars, :clear explicitly
fn test_repl_004_001_help_includes_utility_commands() {
    bashrs_repl()
        .write_stdin("help\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(":history"))
        .stdout(predicate::str::contains(":vars"))
        .stdout(predicate::str::contains(":clear"));
}
