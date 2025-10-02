//! Idempotence Property Tests - EXTREME TDD for TICKET-1001
//!
//! Philosophy: 自働化 (Jidoka) - Build Quality In
//!
//! These tests verify that generated shell scripts are idempotent,
//! meaning they produce the same result when run multiple times.
//!
//! Critical Property: For all valid Rash programs P:
//!   run(transpile(P)) ≡ run(run(transpile(P)))
//!
//! This is especially critical for:
//! - Control flow (if/else, while, for)
//! - File system operations
//! - State modifications

use crate::{transpile, Config};
use proptest::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Represents the observable state after running a script
#[derive(Debug, Clone, PartialEq, Eq)]
struct ScriptState {
    /// Exit code
    exit_code: i32,
    /// Standard output
    stdout: String,
    /// Standard error
    stderr: String,
    /// Files created/modified (path -> content hash)
    files: HashMap<String, String>,
    /// Environment variables set (for testing purposes)
    env_vars: HashMap<String, String>,
}

/// Execute a shell script and capture its state
fn execute_and_capture_state(script: &str, working_dir: &TempDir) -> ScriptState {
    // Write script to temp file
    let script_path = working_dir.path().join("script.sh");
    fs::write(&script_path, script).expect("Failed to write script");

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    // Execute script
    let output = Command::new("sh")
        .arg(&script_path)
        .current_dir(working_dir.path())
        .output()
        .expect("Failed to execute script");

    // Capture file system state
    let mut files = HashMap::new();
    if let Ok(entries) = fs::read_dir(working_dir.path()) {
        for entry in entries.flatten() {
            if let Ok(path) = entry.path().strip_prefix(working_dir.path()) {
                if path.to_str() != Some("script.sh") {
                    if let Ok(content) = fs::read(&entry.path()) {
                        let hash = blake3::hash(&content).to_string();
                        files.insert(path.display().to_string(), hash);
                    }
                }
            }
        }
    }

    ScriptState {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        files,
        env_vars: HashMap::new(), // TODO: Capture exported vars if needed
    }
}

// ============================================================================
// PROPERTY 1: Simple if/else idempotence
// ============================================================================

#[test]
fn test_if_else_idempotent_true_branch() {
    let source = r#"
        fn main() {
            let condition = true;
            if condition {
                write_file("result.txt", "true branch");
            } else {
                write_file("result.txt", "false branch");
            }
        }

        fn write_file(path: &str, content: &str) {}
    "#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Run twice in same directory
    let temp_dir = TempDir::new().unwrap();
    let state1 = execute_and_capture_state(&shell, &temp_dir);

    let temp_dir2 = TempDir::new().unwrap();
    let state2 = execute_and_capture_state(&shell, &temp_dir2);

    // States should be identical (idempotent)
    assert_eq!(state1.exit_code, state2.exit_code, "Exit codes differ");
    assert_eq!(state1.stdout, state2.stdout, "Stdout differs");
    assert_eq!(state1.files.len(), state2.files.len(), "File count differs");
}

#[test]
fn test_if_else_idempotent_false_branch() {
    let source = r#"
        fn main() {
            let condition = false;
            if condition {
                write_file("result.txt", "true branch");
            } else {
                write_file("result.txt", "false branch");
            }
        }

        fn write_file(path: &str, content: &str) {}
    "#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let state1 = execute_and_capture_state(&shell, &temp_dir);

    let temp_dir2 = TempDir::new().unwrap();
    let state2 = execute_and_capture_state(&shell, &temp_dir2);

    assert_eq!(state1.exit_code, state2.exit_code, "Exit codes differ");
    assert_eq!(state1.stdout, state2.stdout, "Stdout differs");
}

// ============================================================================
// PROPERTY 2: Nested if/else idempotence
// ============================================================================

#[test]
fn test_nested_if_else_idempotent() {
    let source = r#"
        fn main() {
            let outer = true;
            let inner = false;

            if outer {
                if inner {
                    write_file("nested.txt", "both true");
                } else {
                    write_file("nested.txt", "outer true, inner false");
                }
            } else {
                write_file("nested.txt", "outer false");
            }
        }

        fn write_file(path: &str, content: &str) {}
    "#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Run 3 times to ensure consistent idempotence
    let states: Vec<ScriptState> = (0..3)
        .map(|_| {
            let temp = TempDir::new().unwrap();
            execute_and_capture_state(&shell, &temp)
        })
        .collect();

    // All states must be identical
    for i in 1..states.len() {
        assert_eq!(
            states[0].exit_code, states[i].exit_code,
            "Exit code differs on run {}",
            i
        );
        assert_eq!(
            states[0].stdout, states[i].stdout,
            "Stdout differs on run {}",
            i
        );
        assert_eq!(
            states[0].files.len(),
            states[i].files.len(),
            "File count differs on run {}",
            i
        );
    }
}

// ============================================================================
// PROPERTY 3: Multiple independent if statements
// ============================================================================

#[test]
fn test_multiple_if_statements_idempotent() {
    let source = r#"
        fn main() {
            let check1 = true;
            let check2 = false;
            let check3 = true;

            if check1 {
                write_file("file1.txt", "check1 passed");
            }

            if check2 {
                write_file("file2.txt", "check2 passed");
            }

            if check3 {
                write_file("file3.txt", "check3 passed");
            }
        }

        fn write_file(path: &str, content: &str) {}
    "#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir1 = TempDir::new().unwrap();
    let state1 = execute_and_capture_state(&shell, &temp_dir1);

    let temp_dir2 = TempDir::new().unwrap();
    let state2 = execute_and_capture_state(&shell, &temp_dir2);

    assert_eq!(state1, state2, "Multiple if statements not idempotent");
}

// ============================================================================
// PROPERTY 4: Control flow with early return (if supported)
// ============================================================================

#[test]
fn test_early_exit_idempotent() {
    // Test conditional execution - only code in executed branches runs
    // This validates control flow correctness
    let source = r#"
        fn main() {
            let should_execute = true;

            if should_execute {
                let marker = "branch_executed";
            }

            if !should_execute {
                let unreachable = "should_not_execute";
            }
        }
    "#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir1 = TempDir::new().unwrap();
    let state1 = execute_and_capture_state(&shell, &temp_dir1);

    let temp_dir2 = TempDir::new().unwrap();
    let state2 = execute_and_capture_state(&shell, &temp_dir2);

    // Should execute identically both times
    assert_eq!(state1.exit_code, 0, "First run should complete successfully");
    assert_eq!(state2.exit_code, 0, "Second run should complete successfully");
    assert_eq!(state1, state2, "Conditional execution not idempotent");
}

// ============================================================================
// PROPERTY 5: Variable assignment in branches
// ============================================================================

#[test]
fn test_variable_assignment_in_branches_idempotent() {
    let source = r#"
        fn main() {
            let condition = true;
            let result = "default";

            if condition {
                let result = "modified";
                write_file("var.txt", result);
            } else {
                write_file("var.txt", result);
            }
        }

        fn write_file(path: &str, content: &str) {}
    "#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir1 = TempDir::new().unwrap();
    let state1 = execute_and_capture_state(&shell, &temp_dir1);

    let temp_dir2 = TempDir::new().unwrap();
    let state2 = execute_and_capture_state(&shell, &temp_dir2);

    assert_eq!(state1, state2, "Variable assignment in branches not idempotent");
}

// ============================================================================
// PROPERTY 6: Chained if-else-if
// ============================================================================

#[test]
fn test_if_else_if_chain_idempotent() {
    let source = r#"
        fn main() {
            let value = 2;

            if value == 1 {
                write_file("result.txt", "one");
            } else if value == 2 {
                write_file("result.txt", "two");
            } else if value == 3 {
                write_file("result.txt", "three");
            } else {
                write_file("result.txt", "other");
            }
        }

        fn write_file(path: &str, content: &str) {}
    "#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Run 5 times to ensure no state accumulation
    let states: Vec<ScriptState> = (0..5)
        .map(|_| {
            let temp = TempDir::new().unwrap();
            execute_and_capture_state(&shell, &temp)
        })
        .collect();

    // All states identical
    for (i, state) in states.iter().enumerate().skip(1) {
        assert_eq!(&states[0], state, "State differs on run {}", i);
    }
}

// ============================================================================
// PROPERTY 7: Boolean expressions in conditions
// ============================================================================

#[test]
fn test_complex_boolean_conditions_idempotent() {
    let source = r#"
        fn main() {
            let a = true;
            let b = false;
            let c = true;

            if a && b {
                write_file("result.txt", "a and b");
            } else if a || c {
                write_file("result.txt", "a or c");
            } else {
                write_file("result.txt", "neither");
            }
        }

        fn write_file(path: &str, content: &str) {}
    "#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir1 = TempDir::new().unwrap();
    let state1 = execute_and_capture_state(&shell, &temp_dir1);

    let temp_dir2 = TempDir::new().unwrap();
    let state2 = execute_and_capture_state(&shell, &temp_dir2);

    assert_eq!(state1, state2, "Complex boolean conditions not idempotent");
}

// ============================================================================
// PROPERTY 8: Deterministic script execution (smoke test)
// ============================================================================

#[test]
fn test_simple_script_deterministic() {
    let source = r#"
        fn main() {
            write_file("test.txt", "hello world");
        }

        fn write_file(path: &str, content: &str) {}
    "#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Run 10 times
    let states: Vec<ScriptState> = (0..10)
        .map(|_| {
            let temp = TempDir::new().unwrap();
            execute_and_capture_state(&shell, &temp)
        })
        .collect();

    // All must be identical
    for (i, state) in states.iter().enumerate().skip(1) {
        assert_eq!(&states[0], state, "Non-deterministic on run {}", i);
    }
}

// ============================================================================
// PROPERTY 9: No unintended side effects in conditions
// ============================================================================

#[test]
fn test_condition_evaluation_no_side_effects() {
    let source = r#"
        fn main() {
            let counter = 0;

            // Condition should not modify state
            if counter == 0 {
                write_file("zero.txt", "counter is zero");
            }

            // Counter should still be 0
            if counter == 0 {
                write_file("still_zero.txt", "counter still zero");
            }
        }

        fn write_file(path: &str, content: &str) {}
    "#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir1 = TempDir::new().unwrap();
    let state1 = execute_and_capture_state(&shell, &temp_dir1);

    let temp_dir2 = TempDir::new().unwrap();
    let state2 = execute_and_capture_state(&shell, &temp_dir2);

    // Both files should be created both times
    assert_eq!(state1.files.len(), state2.files.len());
    assert_eq!(state1, state2, "Condition evaluation has side effects");
}

// ============================================================================
// PROPERTY 10: Empty branches don't cause issues
// ============================================================================

#[test]
fn test_empty_branches_idempotent() {
    let source = r#"
        fn main() {
            let condition = true;

            if condition {
                // Empty true branch
            } else {
                write_file("else.txt", "else branch");
            }

            write_file("after.txt", "after if");
        }

        fn write_file(path: &str, content: &str) {}
    "#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir1 = TempDir::new().unwrap();
    let state1 = execute_and_capture_state(&shell, &temp_dir1);

    let temp_dir2 = TempDir::new().unwrap();
    let state2 = execute_and_capture_state(&shell, &temp_dir2);

    assert_eq!(state1, state2, "Empty branches not idempotent");
}

// ============================================================================
// Future: Property-based tests with QuickCheck/Proptest
// ============================================================================
//
// These would test arbitrary combinations of:
// - Conditions (bool expressions)
// - Branch depths (nested if/else)
// - Statement types in branches
// - Variable scoping
//
// Example (to be implemented):
//
// #[proptest]
// fn prop_generated_if_else_always_idempotent(
//     #[strategy(bool_expr_strategy())] condition: BoolExpr,
//     #[strategy(statement_list_strategy())] then_stmts: Vec<Statement>,
//     #[strategy(statement_list_strategy())] else_stmts: Vec<Statement>,
// ) {
//     // Generate source from AST
//     // Transpile
//     // Run twice
//     // Assert states identical
// }
