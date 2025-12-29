#![allow(deprecated)]
#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)]
//! REPL Mode-Based Command Processing Tests
//!
//! Task: REPL-003-005 - Automatic mode-based command processing
//! Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//!
//! Quality targets:
//! - Integration tests: 8+ scenarios
//! - User experience verified
//! - All modes tested

#![allow(non_snake_case)] // Test naming convention: test_<TASK_ID>_<feature>_<scenario>

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper function to create bashrs REPL command
fn bashrs_repl() -> Command {
    let mut cmd = assert_cmd::cargo_bin_cmd!("bashrs");
    cmd.arg("repl");
    cmd
}

// ===== NORMAL MODE TESTS =====

/// Test: REPL-003-005-001 - Normal mode shows command not implemented
#[test]
fn test_REPL_003_005_normal_mode_command() {
    bashrs_repl()
        .write_stdin("echo hello\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("normal"))
        .stdout(predicate::str::contains("Command not implemented").not());
}

// ===== PURIFY MODE TESTS =====

/// Test: REPL-003-005-002 - Purify mode automatically purifies commands
#[test]
fn test_REPL_003_005_purify_mode_auto_purify() {
    bashrs_repl()
        .write_stdin(":mode purify\nmkdir /tmp/test\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("purify"))
        .stdout(predicate::str::contains("Purified")); // Shows purification happened
}

/// Test: REPL-003-005-003 - Purify mode shows purified output
#[test]
fn test_REPL_003_005_purify_mode_shows_purified() {
    bashrs_repl()
        .write_stdin(":mode purify\nrm /tmp/test\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Purified")); // Shows purification output
}

// ===== LINT MODE TESTS =====

/// Test: REPL-003-005-004 - Lint mode automatically lints commands
#[test]
fn test_REPL_003_005_lint_mode_auto_lint() {
    bashrs_repl()
        .write_stdin(":mode lint\ncat file.txt | grep pattern\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("lint"))
        .stdout(predicate::str::contains("SC").or(predicate::str::contains("Found")));
    // Shows lint results
}

/// Test: REPL-003-005-005 - Lint mode shows clean result for good code
#[test]
fn test_REPL_003_005_lint_mode_clean_code() {
    bashrs_repl()
        .write_stdin(":mode lint\necho \"hello\"\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("lint"));
}

// ===== MODE SWITCHING TESTS =====

/// Test: REPL-003-005-006 - Switching modes changes behavior
#[test]
fn test_REPL_003_005_mode_switching_changes_behavior() {
    bashrs_repl()
        .write_stdin(":mode\n:mode purify\n:mode normal\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("normal"))
        .stdout(predicate::str::contains("purify"));
}

/// Test: REPL-003-005-007 - Mode persists across commands
#[test]
fn test_REPL_003_005_mode_persists() {
    bashrs_repl()
        .write_stdin(":mode purify\nmkdir test1\nmkdir test2\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("✓ Purified:").count(2)); // Purifies both commands
}

// ===== ERROR HANDLING TESTS =====

/// Test: REPL-003-005-008 - Invalid bash handled gracefully in any mode
#[test]
fn test_REPL_003_005_invalid_bash_handled() {
    bashrs_repl()
        .write_stdin(":mode purify\nif then fi\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("error").or(predicate::str::contains("Parse")));
}

/// Test: REPL-003-005-009 - Mode switching with invalid mode shows error
#[test]
fn test_REPL_003_005_invalid_mode_error() {
    bashrs_repl()
        .write_stdin(":mode invalidmode\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Error").or(predicate::str::contains("Unknown mode")));
}

// ===== EXPLICIT COMMANDS STILL WORK =====

/// Test: REPL-003-005-010 - :parse command works in any mode
#[test]
fn test_REPL_003_005_parse_command_in_any_mode() {
    bashrs_repl()
        .write_stdin(":mode purify\n:parse echo hello\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Parse successful"));
}

/// Test: REPL-003-005-011 - :lint command works in any mode
#[test]
fn test_REPL_003_005_lint_command_in_any_mode() {
    bashrs_repl()
        .write_stdin(":mode normal\n:lint echo hello\nquit\n")
        .assert()
        .success();
}

/// Test: REPL-003-005-012 - :purify command works in any mode
#[test]
fn test_REPL_003_005_purify_command_in_any_mode() {
    bashrs_repl()
        .write_stdin(":mode lint\n:purify mkdir test\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Purification"));
}
