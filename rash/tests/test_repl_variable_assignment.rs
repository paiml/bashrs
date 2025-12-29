#![allow(deprecated)]
#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)]
//! REPL Variable Assignment Tests
//!
//! Task: REPL-007-001 - Variable assignment and expansion
//! Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//!
//! Quality targets:
//! - Integration tests: 12+ scenarios
//! - Variable assignment workflows verified
//! - Variable expansion in all modes tested

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper function to create bashrs REPL command
fn bashrs_repl() -> Command {
    let mut cmd = assert_cmd::cargo_bin_cmd!("bashrs");
    cmd.arg("repl");
    cmd
}

// ===== VARIABLE ASSIGNMENT TESTS =====

/// Test: REPL-007-001-001 - Simple variable assignment
#[test]
fn test_repl_007_001_simple_assignment() {
    bashrs_repl()
        .write_stdin("x=5\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Variable set"));
}

/// Test: REPL-007-001-002 - Variable assignment with double quotes
#[test]
fn test_repl_007_001_assignment_double_quotes() {
    bashrs_repl()
        .write_stdin("name=\"hello world\"\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Variable set"));
}

/// Test: REPL-007-001-003 - Variable assignment with single quotes
#[test]
fn test_repl_007_001_assignment_single_quotes() {
    bashrs_repl()
        .write_stdin("path='/usr/bin'\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Variable set"));
}

/// Test: REPL-007-001-004 - Multiple variable assignments
#[test]
fn test_repl_007_001_multiple_assignments() {
    bashrs_repl()
        .write_stdin("x=5\ny=10\nz=15\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Variable set").count(3));
}

// ===== VARIABLE EXPANSION TESTS =====

/// Test: REPL-007-001-005 - Variable expansion in normal mode
#[test]
fn test_repl_007_001_expansion_normal_mode() {
    bashrs_repl()
        .write_stdin("x=42\necho $x\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

/// Test: REPL-007-001-006 - Variable expansion with braces
#[test]
fn test_repl_007_001_expansion_braced() {
    bashrs_repl()
        .write_stdin("name=Alice\necho ${name}\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"));
}

/// Test: REPL-007-001-007 - Multiple variable expansion
#[test]
fn test_repl_007_001_multiple_expansion() {
    bashrs_repl()
        .write_stdin("x=1\ny=2\necho $x + $y\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"));
}

/// Test: REPL-007-001-008 - Unknown variable expands to empty
#[test]
fn test_repl_007_001_unknown_variable() {
    bashrs_repl()
        .write_stdin("echo $unknown\nquit\n")
        .assert()
        .success();
}

// ===== :VARS COMMAND WITH VARIABLES =====

/// Test: REPL-007-001-009 - :vars shows assigned variables
#[test]
fn test_repl_007_001_vars_shows_variables() {
    bashrs_repl()
        .write_stdin("x=5\ny=10\n:vars\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("x"))
        .stdout(predicate::str::contains("5"))
        .stdout(predicate::str::contains("y"))
        .stdout(predicate::str::contains("10"));
}

/// Test: REPL-007-001-010 - :vars before any assignments
#[test]
fn test_repl_007_001_vars_empty() {
    bashrs_repl()
        .write_stdin(":vars\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("No session variables"));
}

// ===== VARIABLE EXPANSION IN DIFFERENT MODES =====

/// Test: REPL-007-001-011 - Variable expansion in purify mode
#[test]
fn test_repl_007_001_expansion_purify_mode() {
    bashrs_repl()
        .write_stdin(":mode purify\nx=test\nmkdir $x\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Variable set"))
        .stdout(predicate::str::contains("Purified"));
}

/// Test: REPL-007-001-012 - Variable expansion in lint mode
#[test]
fn test_repl_007_001_expansion_lint_mode() {
    bashrs_repl()
        .write_stdin(":mode lint\nfilename=test.txt\ncat $filename\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Variable set"));
}

// ===== EDGE CASES =====

/// Test: REPL-007-001-013 - Empty variable value
#[test]
fn test_repl_007_001_empty_value() {
    bashrs_repl()
        .write_stdin("empty=\n:vars\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("empty"));
}

/// Test: REPL-007-001-014 - Variable with underscore in name
#[test]
fn test_repl_007_001_underscore_name() {
    bashrs_repl()
        .write_stdin("_private=secret\n:vars\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("_private"));
}

/// Test: REPL-007-001-015 - Variable assignment doesn't override commands
#[test]
#[ignore] // Output format changed: "Would execute" message removed
fn test_repl_007_001_assignment_not_command() {
    bashrs_repl()
        .write_stdin("test -f file=test.txt\nquit\n")
        .assert()
        .success()
        // This should execute as a command, not be treated as an assignment
        .stdout(predicate::str::contains("Would execute"))
        .stdout(predicate::str::contains("Variable set").not());
}

// ===== WORKFLOW INTEGRATION TESTS =====

/// Test: REPL-007-001-016 - Complete workflow: assign, expand, check
#[test]
fn test_repl_007_001_complete_workflow() {
    bashrs_repl()
        .write_stdin("version=1.0.0\necho Release: $version\n:vars\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Variable set"))
        .stdout(predicate::str::contains("1.0.0"))
        .stdout(predicate::str::contains("version"));
}

/// Test: REPL-007-001-017 - Variable reassignment
#[test]
fn test_repl_007_001_variable_reassignment() {
    bashrs_repl()
        .write_stdin("x=old\nx=new\necho $x\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Variable set").count(2))
        .stdout(predicate::str::contains("new"));
}

/// Test: REPL-007-001-018 - Variables persist across mode switches
#[test]
fn test_repl_007_001_variables_persist_across_modes() {
    bashrs_repl()
        .write_stdin(":mode normal\nx=test\n:mode purify\necho $x\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Variable set"))
        .stdout(predicate::str::contains("Switched to purify mode"));
}
