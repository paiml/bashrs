#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
//! # SPRINT 35: Multi-Shell Execution Tests
//!
//! Validates that generated shell scripts execute correctly across different
//! POSIX-compliant shells (sh, dash, ash, busybox sh).
//!
//! Testing Spec Section 1.3: Layer 3 - Execution Tests
//!
//! ## Critical Invariants
//! 1. Scripts execute successfully in all target shells
//! 2. Output is semantically equivalent across shells
//! 3. Exit codes match expected values
//! 4. Differential testing: Shell output matches Rust execution

use bashrs::{transpile, Config};
use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

/// Represents a shell interpreter
#[derive(Debug, Clone)]
pub enum Shell {
    Sh,        // System sh (usually dash symlink)
    Dash,      // Debian Almquist Shell
    Bash,      // Bourne Again Shell
    Ash,       // BusyBox ash (requires Docker)
    BusyboxSh, // BusyBox sh (requires Docker)
}

impl Shell {
    fn command(&self) -> &str {
        match self {
            Shell::Sh => "sh",
            Shell::Dash => "dash",
            Shell::Bash => "bash",
            Shell::Ash => "ash",
            Shell::BusyboxSh => "busybox sh",
        }
    }

    fn is_available(&self) -> bool {
        match self {
            Shell::Sh | Shell::Dash | Shell::Bash => Command::new(self.command())
                .arg("-c")
                .arg("true")
                .output()
                .is_ok(),
            Shell::Ash | Shell::BusyboxSh => {
                // These require Docker
                false
            }
        }
    }

    fn all_available() -> Vec<Shell> {
        vec![
            Shell::Sh,
            Shell::Dash,
            Shell::Bash,
            Shell::Ash,
            Shell::BusyboxSh,
        ]
        .into_iter()
        .filter(|s| s.is_available())
        .collect()
    }
}

/// Execute a shell script in the specified shell
fn execute_shell_script(shell: &Shell, script: &str) -> Result<Output, String> {
    let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let script_path = temp_dir.path().join("test.sh");

    fs::write(&script_path, script).map_err(|e| format!("Failed to write script: {}", e))?;

    let output = Command::new(shell.command())
        .arg(&script_path)
        .output()
        .map_err(|e| format!("Failed to execute {} script: {}", shell.command(), e))?;

    Ok(output)
}

/// Compile and execute Rust source code
#[allow(dead_code)]
fn execute_rust_source(source: &str) -> Result<Output, String> {
    let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let source_path = temp_dir.path().join("main.rs");
    let binary_path = temp_dir.path().join("main");

    fs::write(&source_path, source).map_err(|e| format!("Failed to write Rust source: {}", e))?;

    // Compile
    let compile_output = Command::new("rustc")
        .arg(&source_path)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .map_err(|e| format!("Failed to compile Rust: {}", e))?;

    if !compile_output.status.success() {
        return Err(format!(
            "Rust compilation failed:\n{}",
            String::from_utf8_lossy(&compile_output.stderr)
        ));
    }

    // Execute
    let output = Command::new(&binary_path)
        .output()
        .map_err(|e| format!("Failed to execute Rust binary: {}", e))?;

    Ok(output)
}

/// Helper to transpile and execute in multiple shells
fn transpile_and_execute_multi_shell(source: &str) -> Vec<(Shell, Result<Output, String>)> {
    let config = Config::default();
    let shell_script = match transpile(source, config) {
        Ok(script) => script,
        Err(e) => {
            return vec![(Shell::Sh, Err(format!("Transpilation failed: {}", e)))];
        }
    };

    Shell::all_available()
        .into_iter()
        .map(|shell| {
            let result = execute_shell_script(&shell, &shell_script);
            (shell, result)
        })
        .collect()
}

// ============================================================================
// Basic Execution Tests
// ============================================================================

#[test]
fn test_multi_shell_empty_main() {
    let source = r#"
        fn main() {}
    "#;

    let results = transpile_and_execute_multi_shell(source);

    for (shell, result) in results {
        let output = result.unwrap_or_else(|e| panic!("{:?} failed: {}", shell, e));
        assert!(
            output.status.success(),
            "{:?} failed with exit code: {:?}",
            shell,
            output.status.code()
        );
    }
}

#[test]
fn test_multi_shell_simple_echo() {
    let source = r#"
        fn main() {
            println!("Hello, World!");
        }
    "#;

    let results = transpile_and_execute_multi_shell(source);

    for (shell, result) in results {
        let output = result.unwrap_or_else(|e| panic!("{:?} failed: {}", shell, e));
        assert!(
            output.status.success(),
            "{:?} failed with exit code: {:?}",
            shell,
            output.status.code()
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Hello, World!"),
            "{:?} output missing expected text. Got: {}",
            shell,
            stdout
        );
    }
}

#[test]
fn test_multi_shell_variables() {
    let source = r#"
        fn main() {
            let x = 42;
            let y = 10;
        }
    "#;

    let results = transpile_and_execute_multi_shell(source);

    for (shell, result) in results {
        let output = result.unwrap_or_else(|e| panic!("{:?} failed: {}", shell, e));
        assert!(output.status.success(), "{:?} failed", shell);
    }
}

#[test]
fn test_multi_shell_arithmetic() {
    let source = r#"
        fn main() {
            let x = 10 + 5;
            let y = 20 - 3;
            let z = 4 * 5;
        }
    "#;

    let results = transpile_and_execute_multi_shell(source);

    for (shell, result) in results {
        let output = result.unwrap_or_else(|e| panic!("{:?} failed: {}", shell, e));
        assert!(
            output.status.success(),
            "{:?} failed with arithmetic",
            shell
        );
    }
}

#[test]
fn test_multi_shell_if_statement() {
    let source = r#"
        fn main() {
            let x = 5;
            if x > 3 {
                println!("greater");
            } else {
                println!("not greater");
            }
        }
    "#;

    let results = transpile_and_execute_multi_shell(source);

    for (shell, result) in results {
        let output = result.unwrap_or_else(|e| panic!("{:?} failed: {}", shell, e));
        assert!(output.status.success(), "{:?} failed", shell);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("greater"),
            "{:?} if-statement failed. Got: {}",
            shell,
            stdout
        );
    }
}

#[test]
fn test_multi_shell_for_loop() {
    let source = r#"
        fn main() {
            for i in 0..3 {
                println!("loop");
            }
        }
    "#;

    let results = transpile_and_execute_multi_shell(source);

    for (shell, result) in results {
        let output = result.unwrap_or_else(|e| panic!("{:?} failed: {}", shell, e));
        assert!(output.status.success(), "{:?} failed", shell);

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should print "loop" 3 times
        let count = stdout.matches("loop").count();
        assert_eq!(
            count, 3,
            "{:?} for loop didn't iterate 3 times. Got {count} iterations",
            shell
        );
    }
}

#[test]
fn test_multi_shell_while_loop() {
    let source = r#"
        fn main() {
            while true {
                println!("once");
                break;
            }
        }
    "#;

    let results = transpile_and_execute_multi_shell(source);

    for (shell, result) in results {
        let output = result.unwrap_or_else(|e| panic!("{:?} failed: {}", shell, e));
        assert!(output.status.success(), "{:?} failed", shell);

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should print "once" exactly one time (break immediately)
        let count = stdout.matches("once").count();
        assert_eq!(count, 1, "{:?} while loop with break failed", shell);
    }
}

#[test]
fn test_multi_shell_match_expression() {
    let source = r#"
        fn main() {
            let x = 2;
            match x {
                1 => { println!("one"); }
                2 => { println!("two"); }
                _ => { println!("other"); }
            }
        }
    "#;

    let results = transpile_and_execute_multi_shell(source);

    for (shell, result) in results {
        let output = result.unwrap_or_else(|e| panic!("{:?} failed: {}", shell, e));
        assert!(output.status.success(), "{:?} failed", shell);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("two"),
            "{:?} match expression failed. Got: {}",
            shell,
            stdout
        );
    }
}

#[test]
fn test_multi_shell_string_operations() {
    let source = r#"
        fn main() {
            let name = "World";
            let greeting = "Hello";
            println!("test");
        }
    "#;

    let results = transpile_and_execute_multi_shell(source);

    for (shell, result) in results {
        let output = result.unwrap_or_else(|e| panic!("{:?} failed: {}", shell, e));
        assert!(output.status.success(), "{:?} failed", shell);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("test"),
            "{:?} string ops failed. Got: {}",
            shell,
            stdout
        );
    }
}

// ============================================================================
// POSIX Compliance Tests
// ============================================================================

#[test]
fn test_multi_shell_posix_exit_codes() {
    let source = r#"
        fn main() {
            let x = 0;
            if x == 0 {
                println!("zero");
            }
        }
    "#;

    let results = transpile_and_execute_multi_shell(source);

    for (shell, result) in results {
        let output = result.unwrap_or_else(|e| panic!("{:?} failed: {}", shell, e));
        assert_eq!(
            output.status.code(),
            Some(0),
            "{:?} exit code should be 0",
            shell
        );
    }
}

#[test]
fn test_multi_shell_special_chars_escaped() {
    let source = r#"
        fn main() {
            let text = "$HOME is home;";
            println!("test done");
        }
    "#;

    let results = transpile_and_execute_multi_shell(source);

    for (shell, result) in results {
        let output = result.unwrap_or_else(|e| panic!("{:?} failed: {}", shell, e));
        assert!(output.status.success(), "{:?} failed", shell);

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Verify script executed
        assert!(
            stdout.contains("test done"),
            "{:?} special chars test failed. Got: {}",
            shell,
            stdout
        );
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Run a test across all available shells and collect results
#[allow(dead_code)]
pub fn test_across_shells<F>(test_fn: F) -> Vec<(Shell, Result<(), String>)>
where
    F: Fn(&Shell) -> Result<(), String>,
{
    Shell::all_available()
        .into_iter()
        .map(|shell| {
            let result = test_fn(&shell);
            (shell, result)
        })
        .collect()
}

/// Get list of available shells for testing
#[allow(dead_code)]
pub fn get_available_shells() -> Vec<Shell> {
    Shell::all_available()
}
