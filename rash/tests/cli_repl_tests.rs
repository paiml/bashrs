//! CLI REPL Integration Tests
//!
//! Task: REPL-003-002 - Basic REPL loop with rustyline integration
//! Test Approach: CLI integration tests with assert_cmd
//!
//! Quality targets:
//! - Integration tests: 3+ scenarios
//! - CLI interaction validated
//! - User experience verified

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
        .write_stdin("")  // Empty input simulates EOF
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
        .stdout(predicate::str::contains("bashrs REPL Commands"))
        .stdout(predicate::str::contains("help"))
        .stdout(predicate::str::contains("quit"));
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
