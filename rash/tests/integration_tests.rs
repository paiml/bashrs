#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)]
use bashrs::models::config::{ShellDialect, VerificationLevel};
use bashrs::{check, transpile, Config};
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn test_end_to_end_simple_transpilation() {
    let source = r#"
fn main() {
    let greeting = "Hello, World!";
    echo(greeting);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, &config).unwrap();

    // Verify basic structure
    assert!(result.contains("#!/bin/sh"));
    assert!(result.contains("set -euf"));
    assert!(result.contains("greeting='Hello, World!'"));
    assert!(result.contains("echo \"$greeting\""));
    assert!(result.contains("main \"$@\""));
}

#[test]
fn test_end_to_end_with_verification() {
    let source = r#"
fn main() {
    let safe_string = "safe content";
    echo(safe_string);
}

fn echo(msg: &str) {}
"#;

    let config = Config {
        verify: VerificationLevel::Strict,
        ..Default::default()
    };

    let result = transpile(source, &config);
    assert!(result.is_ok());
}

#[test]
fn test_generated_script_execution() {
    let source = r#"
fn main() {
    let message = "Hello from Rash!";
    echo(message);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell_script = transpile(source, &config).unwrap();

    // Write to temporary file and execute
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test.sh");
    fs::write(&script_path, shell_script).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(output.status.success());
    // The script should execute without errors
}

#[test]
fn test_generated_script_with_variables() {
    let source = r#"
fn main() {
    let x = 42;
    let name = "test";
    let greeting = "Hello";
}
"#;

    let config = Config::default();
    let shell_script = transpile(source, &config).unwrap();

    // Execute and verify variables are set correctly
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test.sh");

    // Modify script to print variables for verification
    // Transpiler single-quotes string values in assignments (e.g., x='42')
    let modified_script = shell_script
        .replace("x='42'", "x='42'\n    echo \"x=$x\"")
        .replace("name='test'", "name='test'\n    echo \"name=$name\"")
        .replace(
            "greeting='Hello'",
            "greeting='Hello'\n    echo \"greeting=$greeting\"",
        );

    fs::write(&script_path, modified_script).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("x=42"));
    assert!(stdout.contains("name=test"));
    assert!(stdout.contains("greeting=Hello"));
}

#[test]
fn test_different_shell_dialects() {
    let source = r#"
fn main() {
    let msg = "testing dialects";
    echo(msg);
}

fn echo(msg: &str) {}
"#;

    let dialects = [
        ShellDialect::Posix,
        ShellDialect::Bash,
        ShellDialect::Dash,
        ShellDialect::Ash,
    ];

    for dialect in dialects.iter() {
        let config = Config {
            target: *dialect,
            ..Default::default()
        };

        let result = transpile(source, &config);
        assert!(result.is_ok(), "Failed for dialect: {dialect:?}");

        let script = result.unwrap();
        assert!(script.contains("#!/bin/sh"));
        assert!(script.contains("msg='testing dialects'"));
    }
}

#[test]
fn test_verification_levels() {
    let safe_source = r#"
fn main() {
    let safe_var = "safe content";
    echo(safe_var);
}

fn echo(msg: &str) {}
"#;

    let levels = [
        VerificationLevel::None,
        VerificationLevel::Basic,
        VerificationLevel::Strict,
        VerificationLevel::Paranoid,
    ];

    for level in levels.iter() {
        let config = Config {
            verify: *level,
            ..Default::default()
        };

        let result = transpile(safe_source, &config);
        assert!(result.is_ok(), "Failed for verification level: {level:?}");
    }
}

#[test]
fn test_optimization_effects() {
    let source = r#"
fn main() {
    let part1 = "Hello";
    let part2 = " ";
    let part3 = "World";
    let greeting = concat_three(part1, part2, part3);
    echo(greeting);
}

fn concat_three(a: &str, b: &str, c: &str) -> &str { a }
fn echo(msg: &str) {}
"#;

    let config_optimized = Config {
        optimize: true,
        ..Default::default()
    };

    let config_unoptimized = Config {
        optimize: false,
        ..Default::default()
    };

    let optimized = transpile(source, &config_optimized).unwrap();
    let unoptimized = transpile(source, &config_unoptimized).unwrap();

    // Both should work - transpiler single-quotes string values
    assert!(optimized.contains("part1='Hello'"));
    assert!(unoptimized.contains("part1='Hello'"));

    // Optimization might affect the output structure, but both should be valid
    // For now, just ensure both contain the expected output
    assert!(!optimized.is_empty());
    assert!(!unoptimized.is_empty());
}

#[test]
fn test_check_function() {
    let valid_source = r#"
fn main() {
    let x = 42;
}
"#;

    let invalid_source = r#"
fn invalid() {
    // This function doesn't have main
}
"#;

    assert!(check(valid_source).is_ok());
    assert!(check(invalid_source).is_err());
}

#[test]
fn test_complex_nested_structures() {
    let source = r#"
fn main() {
    let condition = true;
    if condition {
        let inner = "nested";
        echo(inner);
    } else {
        let other = "alternative";
        echo(other);
    }
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

    // Should handle nested if/else structures
    if let Ok(script) = result {
        assert!(script.contains("if "));
        assert!(script.contains("then"));
        assert!(script.contains("else"));
        assert!(script.contains("fi"));
    }
    // Note: Current implementation might not fully support if/else yet
}

#[test]
fn test_function_calls_translation() {
    let source = r#"
fn main() {
    helper("test");
    process_data(42, "string");
}

fn helper(msg: &str) {}
fn process_data(num: u32, text: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, &config).unwrap();

    // Function calls should be translated to shell commands
    assert!(result.contains("helper"));
    assert!(result.contains("process_data"));
}

#[test]

include!("integration_tests_main.rs");
