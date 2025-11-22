//! REPL Test Harness
//!
//! Task: REPL-002-003 - Create REPL test harness with assert_cmd
//!
//! This file provides reusable test infrastructure for REPL testing.
//! All REPL CLI tests should use these patterns.
//!
//! Quality Standards:
//! - MANDATORY: Use assert_cmd::Command for all CLI tests
//! - Test naming: test_<TASK_ID>_<feature>_<scenario>
//! - Property tests: 100+ cases per component
//! - Mutation score: ≥90% kill rate
//!
//! References:
//! - CLAUDE.md - CLI Testing Protocol (MANDATORY)
//! - REPL-DEBUGGER-ROADMAP.yaml - REPL-002-003

#![allow(non_snake_case)] // Test naming convention uses TASK_ID format

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper function to create bashrs REPL command
///
/// # Example
///
/// ```no_run
/// use assert_cmd::Command;
///
/// fn bashrs_repl() -> Command {
///     Command::cargo_bin("bashrs").expect("Failed to find bashrs binary").arg("repl")
/// }
///
/// #[test]
/// fn test_REPL_002_003_repl_starts() {
///     bashrs_repl()
///         .write_stdin(":quit\n")
///         .assert()
///         .success();
/// }
/// ```
pub fn bashrs_repl() -> Command {
    let mut cmd = Command::cargo_bin("bashrs").expect("Failed to find bashrs binary");
    cmd.arg("repl");
    cmd
}

/// Helper function to create bashrs REPL with debug mode
pub fn bashrs_repl_debug() -> Command {
    let mut cmd = Command::cargo_bin("bashrs").expect("Failed to find bashrs binary");
    cmd.arg("repl").arg("--debug");
    cmd
}

/// Helper function to create bashrs REPL with custom config
pub fn bashrs_repl_custom(max_memory: &str, timeout: &str, max_depth: &str) -> Command {
    let mut cmd = Command::cargo_bin("bashrs").expect("Failed to find bashrs binary");
    cmd.arg("repl")
        .arg("--max-memory")
        .arg(max_memory)
        .arg("--timeout")
        .arg(timeout)
        .arg("--max-depth")
        .arg(max_depth);
    cmd
}

// ===== REPL-002-003: Test Harness Verification =====

/// Test: REPL-002-003-001 - Verify REPL starts successfully
#[test]
fn test_REPL_002_003_repl_starts() {
    bashrs_repl()
        .write_stdin("quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs"));
}

/// Test: REPL-002-003-002 - Verify debug mode works
#[test]
fn test_REPL_002_003_repl_debug_mode() {
    bashrs_repl_debug()
        .write_stdin("quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs"));
}

/// Test: REPL-002-003-003 - Verify custom config works
#[test]
fn test_REPL_002_003_repl_custom_config() {
    bashrs_repl_custom("200", "60", "200")
        .write_stdin("quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs"));
}

/// Test: REPL-002-003-004 - Verify REPL handles quit command
#[test]
fn test_REPL_002_003_repl_quit() {
    bashrs_repl()
        .write_stdin("quit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Goodbye!"));
}

/// Test: REPL-002-003-005 - Verify REPL handles exit command
#[test]
fn test_REPL_002_003_repl_exit() {
    bashrs_repl()
        .write_stdin("exit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Goodbye!"));
}

/// Test: REPL-002-003-006 - Verify REPL handles EOF
#[test]
fn test_REPL_002_003_repl_eof() {
    bashrs_repl()
        .write_stdin("") // Empty input simulates EOF
        .assert()
        .success()
        .stdout(predicate::str::contains("EOF"));
}

/// Test: REPL-002-003-007 - Verify REPL mode switching
///
/// NOTE: This is a template test demonstrating mode switching verification.
/// Actual mode switching implementation is tracked in separate sprint tasks.
#[test]
#[ignore = "Template test - mode switching tested in REPL-003-004"]
fn test_REPL_002_003_mode_switching() {
    bashrs_repl()
        .write_stdin(":mode parse\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("parse"));
}

/// Test: REPL-002-003-008 - Verify REPL help command
#[test]
fn test_REPL_002_003_help_command() {
    bashrs_repl()
        .write_stdin("help\nquit\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("OVERVIEW"));
}

// ===== Property Tests (using example from roadmap) =====

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Property test: REPL always exits cleanly with valid quit commands
    proptest! {
        #[test]
        fn prop_REPL_002_003_quit_commands_always_work(
            quit_cmd in prop::sample::select(vec!["quit", "exit", ":quit", ":exit"])
        ) {
            let cmd_with_newline = format!("{}\n", quit_cmd);
            bashrs_repl()
                .write_stdin(cmd_with_newline.as_str())
                .assert()
                .success();
        }
    }

    // Property test: REPL handles empty lines gracefully
    proptest! {
        #[test]
        fn prop_REPL_002_003_empty_input_handled(
            empty_count in 1usize..10
        ) {
            let empty_input = "\n".repeat(empty_count) + "quit\n";
            bashrs_repl()
                .write_stdin(empty_input.as_str())
                .assert()
                .success();
        }
    }
}

// ===== Documentation Examples =====

/// Example: Basic REPL test
///
/// ```no_run
/// use assert_cmd::Command;
/// use predicates::prelude::*;
///
/// #[test]
/// fn test_example_basic_repl() {
///     let mut cmd = Command::cargo_bin("bashrs").unwrap();
///     cmd.arg("repl")
///         .write_stdin("quit\n")
///         .assert()
///         .success()
///         .stdout(predicate::str::contains("bashrs"));
/// }
/// ```
#[allow(dead_code)]
fn example_basic_repl_test() {}

/// Example: REPL with mode switching
///
/// ```no_run
/// use assert_cmd::Command;
/// use predicates::prelude::*;
///
/// #[test]
/// fn test_example_mode_switching() {
///     let mut cmd = Command::cargo_bin("bashrs").unwrap();
///     cmd.arg("repl")
///         .write_stdin(":mode purify\nmkdir /tmp/test\nquit\n")
///         .assert()
///         .success()
///         .stdout(predicate::str::contains("purify"))
///         .stdout(predicate::str::contains("mkdir -p"));
/// }
/// ```
#[allow(dead_code)]
fn example_mode_switching_test() {}
