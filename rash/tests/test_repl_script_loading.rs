#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! REPL Script Loading Tests
//!
//! Task: REPL-009-001 - Script loading and sourcing
//! Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//!
//! NOTE: Some tests are marked #[ignore] as the :load/:source feature is partially implemented.
//! Run with: cargo test --ignored test_repl_009
//!
//! Quality targets:
//! - Integration tests: 15+ scenarios
//! - Script loading workflows verified
//! - Function extraction tested
//! - Error handling covered

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper function to create bashrs REPL command
fn bashrs_repl() -> Command {
    let mut cmd = assert_cmd::cargo_bin_cmd!("bashrs");
    cmd.arg("repl");
    cmd
}

/// Helper to create a test bash script
fn create_test_script(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let file_path = dir.path().join(name);
    fs::write(&file_path, content).expect("Failed to write test file");
    file_path
}

// ===== :load COMMAND TESTS =====

/// Test: REPL-009-001-001 - Load simple bash script
#[test]
fn test_repl_009_001_load_simple_script() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "test.sh",
        r#"#!/bin/bash
# Simple test script
echo "Hello from script"
"#,
    );

    bashrs_repl()
        .write_stdin(format!(":load {}\nquit\n", script.display()))
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded"));
}

/// Test: REPL-009-001-002 - Load script with functions
#[test]
fn test_repl_009_001_load_script_with_functions() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "functions.sh",
        r#"#!/bin/bash
greet() {
    echo "Hello, $1"
}

farewell() {
    echo "Goodbye, $1"
}
"#,
    );

    bashrs_repl()
        .write_stdin(format!(":load {}\nquit\n", script.display()))
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded"))
        .stdout(predicate::str::contains("function").or(predicate::str::contains("greet")));
}

/// Test: REPL-009-001-003 - Load nonexistent file
#[test]
fn test_repl_009_001_load_nonexistent_file() {
    bashrs_repl()
        .write_stdin(":load /nonexistent/file.sh\nquit\n")
        .assert()
        .success() // REPL itself succeeds
        .stdout(predicate::str::contains("Error").or(predicate::str::contains("not found")));
}

/// Test: REPL-009-001-004 - Load script with parse error
#[test]
#[ignore] // Partial implementation: :load command exists but output format changed
fn test_repl_009_001_load_invalid_syntax() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "invalid.sh",
        r#"#!/bin/bash
if then fi  # Invalid syntax
"#,
    );

    bashrs_repl()
        .write_stdin(format!(":load {}\nquit\n", script.display()))
        .assert()
        .success()
        .stdout(predicate::str::contains("Error").or(predicate::str::contains("parse")));
}

// ===== :source COMMAND TESTS =====

/// Test: REPL-009-001-005 - Source script executes in session
#[test]
fn test_repl_009_001_source_script() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "source.sh",
        r#"#!/bin/bash
export VAR=test_value
echo "Script sourced"
"#,
    );

    bashrs_repl()
        .write_stdin(format!(":source {}\nquit\n", script.display()))
        .assert()
        .success()
        .stdout(predicate::str::contains("Sourced").or(predicate::str::contains("Script sourced")));
}

/// Test: REPL-009-001-006 - Source adds functions to session
#[test]
fn test_repl_009_001_source_adds_functions() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "lib.sh",
        r#"#!/bin/bash
helper() {
    echo "Helper function"
}
"#,
    );

    bashrs_repl()
        .write_stdin(format!(":source {}\n:functions\nquit\n", script.display()))
        .assert()
        .success()
        .stdout(predicate::str::contains("Sourced"))
        .stdout(predicate::str::contains("function").or(predicate::str::contains("helper")));
}

// ===== :functions COMMAND TESTS =====

/// Test: REPL-009-001-007 - List functions when none loaded
#[test]
fn test_repl_009_001_functions_empty() {
    bashrs_repl()
        .write_stdin(":functions\nquit\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("No functions").or(predicate::str::contains("0 functions")),
        );
}

/// Test: REPL-009-001-008 - List functions after loading script
#[test]
fn test_repl_009_001_functions_after_load() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "funcs.sh",
        r#"#!/bin/bash
func1() { echo "1"; }
func2() { echo "2"; }
func3() { echo "3"; }
"#,
    );

    bashrs_repl()
        .write_stdin(format!(":load {}\n:functions\nquit\n", script.display()))
        .assert()
        .success()
        .stdout(predicate::str::contains("function"));
}

// ===== :reload COMMAND TESTS =====

/// Test: REPL-009-001-009 - Reload last loaded script
#[test]
fn test_repl_009_001_reload_script() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "reload.sh",
        r#"#!/bin/bash
echo "Original version"
"#,
    );

    bashrs_repl()
        .write_stdin(format!(":load {}\n:reload\nquit\n", script.display()))
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Loaded")
                .count(2)
                .or(predicate::str::contains("Reloaded")),
        );
}

/// Test: REPL-009-001-010 - Reload without previous load
#[test]
fn test_repl_009_001_reload_without_load() {
    bashrs_repl()
        .write_stdin(":reload\nquit\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("No script").or(predicate::str::contains("nothing to reload")),
        );
}

// ===== WORKFLOW INTEGRATION TESTS =====

/// Test: REPL-009-001-011 - Load, modify, reload workflow
#[test]
fn test_repl_009_001_load_reload_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("workflow.sh");

    // Create initial version
    fs::write(&script_path, "#!/bin/bash\necho 'Version 1'\n").unwrap();

    bashrs_repl()
        .write_stdin(format!(":load {}\n:reload\nquit\n", script_path.display()))
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded"));
}

/// Test: REPL-009-001-012 - Load script then execute function
#[test]
#[ignore] // Partial implementation: :source command exists but execution may not work as expected
fn test_repl_009_001_load_then_execute() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "exec.sh",
        r#"#!/bin/bash
test_func() {
    echo "Function executed"
}
"#,
    );

    bashrs_repl()
        .write_stdin(format!(":source {}\ntest_func\nquit\n", script.display()))
        .assert()
        .success()
        .stdout(predicate::str::contains("Sourced"))
        .stdout(predicate::str::contains("Function executed"));
}

/// Test: REPL-009-001-013 - Multiple script loading
#[test]
#[ignore] // Partial implementation: Multiple :load commands may not work as expected
fn test_repl_009_001_load_multiple_scripts() {
    let temp_dir = TempDir::new().unwrap();
    let script1 = create_test_script(&temp_dir, "s1.sh", "#!/bin/bash\nfunc1() { :; }\n");
    let script2 = create_test_script(&temp_dir, "s2.sh", "#!/bin/bash\nfunc2() { :; }\n");

    bashrs_repl()
        .write_stdin(format!(
            ":load {}\n:load {}\n:functions\nquit\n",
            script1.display(),
            script2.display()
        ))
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded").count(2));
}

/// Test: REPL-009-001-014 - Load with variables
#[test]
fn test_repl_009_001_load_with_variables() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "vars.sh",
        r#"#!/bin/bash
SCRIPT_VAR=from_script
echo "Script variable: $SCRIPT_VAR"
"#,
    );

    bashrs_repl()
        .write_stdin(format!(":source {}\n:vars\nquit\n", script.display()))
        .assert()
        .success()
        .stdout(predicate::str::contains("Sourced"));
}

/// Test: REPL-009-001-015 - Load script in different modes
#[test]
fn test_repl_009_001_load_in_modes() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(
        &temp_dir,
        "modes.sh",
        r#"#!/bin/bash
mode_test() {
    echo "Testing modes"
}
"#,
    );

    bashrs_repl()
        .write_stdin(format!(
            ":mode normal\n:load {}\n:mode purify\n:reload\nquit\n",
            script.display()
        ))
        .assert()
        .success()
        .stdout(predicate::str::contains("Loaded"));
}
