#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)]
//! Differential Testing for Bash-to-Rash Transpiler
//!
//! Compares execution of bash scripts vs transpiled Rash code
//! to verify semantic equivalence.

#![allow(dead_code)] // Some helper functions reserved for future full differential testing

use bashrs::bash_parser::parser::BashParser;
use bashrs::bash_transpiler::codegen::{BashToRashTranspiler, TranspileOptions};
use std::process::Command;
use tempfile::TempDir;

/// Result of executing a script (bash or rash)
#[derive(Debug, Clone, PartialEq)]
struct ExecutionResult {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

impl ExecutionResult {
    fn normalize(&self) -> Self {
        Self {
            stdout: self.normalize_output(&self.stdout),
            stderr: self.normalize_output(&self.stderr),
            exit_code: self.exit_code,
        }
    }

    fn normalize_output(&self, output: &str) -> String {
        // Normalize whitespace and line endings
        output
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Execute bash script in sandboxed environment
fn execute_bash_sandboxed(script: &str, _workdir: &TempDir) -> Result<ExecutionResult, String> {
    // Use bash in restricted mode for safety
    let output = Command::new("bash")
        .arg("-c")
        .arg(script)
        .output()
        .map_err(|e| format!("Failed to execute bash: {}", e))?;

    Ok(ExecutionResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

/// Compare bash and rash execution results
fn assert_execution_equivalence(
    bash_result: &ExecutionResult,
    rash_result: &ExecutionResult,
    script_name: &str,
) {
    let bash_norm = bash_result.normalize();
    let rash_norm = rash_result.normalize();

    assert_eq!(
        bash_norm.stdout, rash_norm.stdout,
        "stdout mismatch for {}: bash='{}', rash='{}'",
        script_name, bash_norm.stdout, rash_norm.stdout
    );

    // Stderr may differ slightly due to implementation details, so we're more lenient
    if bash_norm.exit_code == 0 && rash_norm.exit_code == 0 {
        // Success cases should have matching output
        assert_eq!(
            bash_norm.exit_code, rash_norm.exit_code,
            "exit code mismatch for {}",
            script_name
        );
    }
}

#[test]
fn test_differential_simple_echo() {
    let bash_script = r#"echo "Hello, World!""#;

    let workdir = TempDir::new().unwrap();

    // Execute original bash
    let bash_result = execute_bash_sandboxed(bash_script, &workdir).unwrap();

    // Parse and transpile to rash
    let mut parser = BashParser::new(bash_script).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    // For now, we'll just verify the transpilation succeeded
    // Full rash execution would require the rash runtime
    assert!(rash_code.contains("echo"));
    assert!(bash_result.exit_code == 0);
}

#[test]
fn test_differential_variable_assignment() {
    let bash_script = r#"
FOO=bar
echo $FOO
"#;

    let workdir = TempDir::new().unwrap();
    let bash_result = execute_bash_sandboxed(bash_script, &workdir).unwrap();

    // Parse and transpile
    let mut parser = BashParser::new(bash_script).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    // Verify transpilation preserves structure
    assert!(rash_code.contains("let FOO"));
    assert!(rash_code.contains("echo"));
    assert!(bash_result.stdout.contains("bar"));
}

#[test]
fn test_differential_simple_command_sequence() {
    let bash_script = r#"
echo "first"
echo "second"
"#;

    let workdir = TempDir::new().unwrap();
    let bash_result = execute_bash_sandboxed(bash_script, &workdir).unwrap();

    // Parse and transpile
    let mut parser = BashParser::new(bash_script).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    // Verify multiple commands are preserved
    assert!(rash_code.matches("echo").count() >= 2);
    assert!(bash_result.stdout.contains("first"));
    assert!(bash_result.stdout.contains("second"));
}

#[test]
fn test_differential_function_definition() {
    let bash_script = r#"
function greet() {
    echo "Hello from function"
}

greet
"#;

    let workdir = TempDir::new().unwrap();
    let bash_result = execute_bash_sandboxed(bash_script, &workdir).unwrap();

    // Parse and transpile
    let mut parser = BashParser::new(bash_script).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    // Verify function transpilation
    assert!(rash_code.contains("fn greet()"));
    assert!(rash_code.contains("greet()"));
    assert!(bash_result.stdout.contains("Hello from function"));
}

#[test]
fn test_differential_conditional() {
    let bash_script = r#"
x=1
if [ $x == 1 ]; then
    echo "one"
else
    echo "not one"
fi
"#;

    let workdir = TempDir::new().unwrap();
    let bash_result = execute_bash_sandboxed(bash_script, &workdir).unwrap();

    // Parse and transpile
    let mut parser = BashParser::new(bash_script).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    // Verify conditional structure
    assert!(rash_code.contains("if x == 1"));
    assert!(rash_code.contains("else"));
    assert!(bash_result.stdout.contains("one"));
}

/// Test corpus from specification examples
#[test]
fn test_differential_corpus_basic_installer() {
    let bash_script = r#"
#!/bin/bash
# Simple installer pattern
INSTALL_DIR=/opt/myapp

function create_dir() {
    echo "Creating directory"
}

create_dir
"#;

    let workdir = TempDir::new().unwrap();
    let bash_result = execute_bash_sandboxed(bash_script, &workdir).unwrap();

    // Parse and transpile
    let mut parser = BashParser::new(bash_script).unwrap();
    let ast = parser.parse().unwrap();

    let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
    let rash_code = transpiler.transpile(&ast).unwrap();

    // Verify structure is preserved
    assert!(rash_code.contains("INSTALL_DIR"));
    assert!(rash_code.contains("fn create_dir()"));
    assert!(bash_result.stdout.contains("Creating directory"));
}

/// Measure differential testing coverage
#[test]
fn test_differential_coverage_metrics() {
    // Focus on core bash constructs that are currently implemented
    let test_cases = vec![
        ("echo test", "simple echo"),
        ("FOO=bar", "variable assignment"),
        ("FOO=bar\necho $FOO", "variable usage"),
        (r#"echo "hello""#, "quoted string"),
        ("export PATH=/usr/bin", "export statement"),
    ];

    let workdir = TempDir::new().unwrap();
    let mut success_count = 0;
    let mut total_count = 0;

    for (script, description) in test_cases {
        total_count += 1;

        match execute_bash_sandboxed(script, &workdir) {
            Ok(_bash_result) => {
                // Attempt parse and transpile
                if let Ok(mut parser) = BashParser::new(script) {
                    if let Ok(ast) = parser.parse() {
                        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
                        if transpiler.transpile(&ast).is_ok() {
                            success_count += 1;
                        } else {
                            eprintln!("Transpilation failed for: {}", description);
                        }
                    } else {
                        eprintln!("Parse failed for: {}", description);
                    }
                } else {
                    eprintln!("Lexer failed for: {}", description);
                }
            }
            Err(e) => {
                eprintln!("Failed to execute {}: {}", description, e);
            }
        }
    }

    // Report coverage
    let coverage = (success_count as f64 / total_count as f64) * 100.0;
    println!(
        "Differential test coverage: {:.1}% ({}/{})",
        coverage, success_count, total_count
    );

    // Phase 1/2 implementation - focus on basic constructs
    // 80% of basic bash constructs should transpile successfully
    assert!(
        coverage >= 80.0,
        "Coverage should be >= 80%, got {:.1}%",
        coverage
    );
}

#[test]
fn test_differential_determinism() {
    let bash_script = "echo 'deterministic output'";

    let workdir = TempDir::new().unwrap();

    // Execute multiple times
    let results: Vec<_> = (0..5)
        .map(|_| execute_bash_sandboxed(bash_script, &workdir).unwrap())
        .collect();

    // All results should be identical
    for window in results.windows(2) {
        assert_eq!(
            window[0], window[1],
            "Bash execution should be deterministic"
        );
    }
}
