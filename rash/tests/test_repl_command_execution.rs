//! REPL Command Execution Tests
//!
//! Task: REPL-008-001 - Normal mode command execution
//! Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//!
//! Quality targets:
//! - Integration tests: 15+ scenarios
//! - Command execution workflows verified
//! - Exit code handling tested
//! - Output capture verified

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper function to create bashrs REPL command
fn bashrs_repl() -> Command {
    let mut cmd = Command::cargo_bin("bashrs").expect("Failed to find bashrs binary");
    cmd.arg("repl");
    cmd
}

// ===== BASIC COMMAND EXECUTION TESTS =====

/// Test: REPL-008-001-001 - Execute simple echo command
#[test]
fn test_repl_008_001_echo_simple() {
    bashrs_repl()
        .write_stdin("echo hello\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

/// Test: REPL-008-001-002 - Execute command with arguments
#[test]
fn test_repl_008_001_echo_multiple_args() {
    bashrs_repl()
        .write_stdin("echo hello world\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

/// Test: REPL-008-001-003 - Execute pwd command
#[test]
fn test_repl_008_001_pwd() {
    bashrs_repl()
        .write_stdin("pwd\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("/"));
}

/// Test: REPL-008-001-004 - Execute date command
#[test]
fn test_repl_008_001_date() {
    bashrs_repl()
        .write_stdin("date +%Y\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("202")); // Year starts with 202
}

// ===== VARIABLE EXPANSION IN EXECUTION =====

/// Test: REPL-008-001-005 - Execute command with variable expansion
#[test]
fn test_repl_008_001_execute_with_variables() {
    bashrs_repl()
        .write_stdin("greeting=hello\necho $greeting\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Variable set"))
        .stdout(predicate::str::contains("hello"));
}

/// Test: REPL-008-001-006 - Execute command with multiple variables
#[test]
fn test_repl_008_001_execute_multiple_variables() {
    bashrs_repl()
        .write_stdin("x=foo\ny=bar\necho $x $y\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("foo bar"));
}

/// Test: REPL-008-001-007 - Execute command with braced variable
#[test]
fn test_repl_008_001_execute_braced_variable() {
    bashrs_repl()
        .write_stdin("name=Alice\necho Hello ${name}!\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello Alice!"));
}

// ===== ERROR HANDLING TESTS =====

/// Test: REPL-008-001-008 - Execute non-existent command
#[test]
fn test_repl_008_001_nonexistent_command() {
    bashrs_repl()
        .write_stdin("nonexistent_command_xyz\nquit\n")
        .assert()
        .success() // REPL itself succeeds
        .stdout(predicate::str::contains("not found").or(predicate::str::contains("No such")));
}

/// Test: REPL-008-001-009 - Execute command with error exit code
#[test]
fn test_repl_008_001_command_error() {
    bashrs_repl()
        .write_stdin("ls /nonexistent_directory_xyz\nquit\n")
        .assert()
        .success() // REPL itself succeeds
        .stdout(predicate::str::contains("No such file"));
}

// ===== MULTI-LINE AND COMPLEX COMMANDS =====

/// Test: REPL-008-001-010 - Execute multiple commands sequentially
#[test]
fn test_repl_008_001_multiple_commands() {
    bashrs_repl()
        .write_stdin("echo first\necho second\necho third\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("first"))
        .stdout(predicate::str::contains("second"))
        .stdout(predicate::str::contains("third"));
}

/// Test: REPL-008-001-011 - Execute command with pipe
#[test]
fn test_repl_008_001_pipe_command() {
    bashrs_repl()
        .write_stdin("echo hello world | head -1\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

// ===== MODE SWITCHING WITH EXECUTION =====

/// Test: REPL-008-001-012 - Switch between normal and other modes
#[test]
fn test_repl_008_001_mode_switching() {
    bashrs_repl()
        .write_stdin(":mode normal\necho test\n:mode purify\nmkdir /tmp/test\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("Purified"));
}

/// Test: REPL-008-001-013 - Execute in normal mode after lint mode
#[test]
fn test_repl_008_001_normal_after_lint() {
    bashrs_repl()
        .write_stdin(":mode lint\ncat file | grep x\n:mode normal\necho back to normal\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("back to normal"));
}

// ===== OUTPUT CAPTURE TESTS =====

/// Test: REPL-008-001-014 - Capture stdout properly
#[test]
fn test_repl_008_001_stdout_capture() {
    bashrs_repl()
        .write_stdin("echo 'Line 1'\necho 'Line 2'\necho 'Line 3'\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Line 1"))
        .stdout(predicate::str::contains("Line 2"))
        .stdout(predicate::str::contains("Line 3"));
}

/// Test: REPL-008-001-015 - Handle empty output
#[test]
fn test_repl_008_001_empty_output() {
    bashrs_repl().write_stdin("true\nquit\n").assert().success();
}

// ===== WORKFLOW INTEGRATION TESTS =====

/// Test: REPL-008-001-016 - Complete workflow: variables + execution
#[test]
fn test_repl_008_001_complete_workflow() {
    bashrs_repl()
        .write_stdin("version=1.0.0\necho Release: $version\n:vars\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Variable set"))
        .stdout(predicate::str::contains("Release: 1.0.0"))
        .stdout(predicate::str::contains("version"));
}

/// Test: REPL-008-001-017 - Execute after history viewing
#[test]
fn test_repl_008_001_execute_after_history() {
    bashrs_repl()
        .write_stdin("echo test1\necho test2\n:history\necho test3\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("test1"))
        .stdout(predicate::str::contains("test2"))
        .stdout(predicate::str::contains("Command History"))
        .stdout(predicate::str::contains("test3"));
}

/// Test: REPL-008-001-018 - Execute commands with special characters
#[test]
fn test_repl_008_001_special_characters() {
    bashrs_repl()
        .write_stdin("echo 'hello!@#$%'\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello!@#$%"));
}
