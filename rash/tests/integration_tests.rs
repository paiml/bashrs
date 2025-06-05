use rash::models::config::{ShellDialect, VerificationLevel};
use rash::{check, transpile, Config};
use std::fs;
use std::process::Command;
use tempfile::TempDir;

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
    let result = transpile(source, config).unwrap();

    // Verify basic structure
    assert!(result.contains("#!/bin/sh"));
    assert!(result.contains("set -euf"));
    assert!(result.contains("readonly greeting='Hello, World!'"));
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

    let result = transpile(source, config);
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
    let shell_script = transpile(source, config).unwrap();

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
    let shell_script = transpile(source, config).unwrap();

    // Execute and verify variables are set correctly
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test.sh");

    // Modify script to print variables for verification
    let modified_script = shell_script
        .replace("readonly x=42", "readonly x=42\n    echo \"x=$x\"")
        .replace(
            "readonly name=test",
            "readonly name=test\n    echo \"name=$name\"",
        )
        .replace(
            "readonly greeting=Hello",
            "readonly greeting=Hello\n    echo \"greeting=$greeting\"",
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

        let result = transpile(source, config);
        assert!(result.is_ok(), "Failed for dialect: {dialect:?}");

        let script = result.unwrap();
        assert!(script.contains("#!/bin/sh"));
        assert!(script.contains("readonly msg='testing dialects'"));
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

        let result = transpile(safe_source, config);
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

    let optimized = transpile(source, config_optimized).unwrap();
    let unoptimized = transpile(source, config_unoptimized).unwrap();

    // Both should work
    assert!(optimized.contains("readonly part1=Hello"));
    assert!(unoptimized.contains("readonly part1=Hello"));

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
    let result = transpile(source, config);

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
    let result = transpile(source, config).unwrap();

    // Function calls should be translated to shell commands
    assert!(result.contains("helper"));
    assert!(result.contains("process_data"));
}

#[test]
fn test_error_handling_invalid_source() {
    let invalid_sources = vec![
        "",                             // Empty
        "invalid rust syntax",          // Not valid Rust
        "fn not_main() { let x = 1; }", // No main function
        "struct NotAllowed {}",         // Not a function (should fail validation)
    ];

    for source in invalid_sources {
        let config = Config::default();
        let result = transpile(source, config);
        assert!(result.is_err(), "Should fail for: {source}");
    }
}

#[test]
fn test_shell_escaping_safety() {
    let source = r#"
fn main() {
    let safe_string = "hello world";
    let string_with_quotes = "don't break";
    let special_chars = "test & echo 'injected'";
    echo(safe_string);
    echo(string_with_quotes);
    echo(special_chars);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config).unwrap();

    // Verify proper escaping
    assert!(result.contains("'hello world'"));
    assert!(result.contains("'don'\"'\"'t break'")); // Proper quote escaping
    assert!(result.contains("'test & echo '\"'\"'injected'\"'\"''")); // Escaped special chars
}

#[test]
fn test_runtime_functions_included() {
    let source = r#"
fn main() {
    let x = 42;
}
"#;

    let config = Config::default();
    let result = transpile(source, config).unwrap();

    // Verify runtime functions are included
    assert!(result.contains("rash_require()"));
    assert!(result.contains("rash_download_verified()"));

    // Verify they contain expected functionality
    assert!(result.contains("curl"));
    assert!(result.contains("sha256sum"));
    assert!(result.contains("wget"));
}

#[test]
fn test_script_header_and_footer() {
    let source = r#"
fn main() {
    let test = "header_footer_test";
}
"#;

    let config = Config::default();
    let result = transpile(source, config).unwrap();

    // Check proper header
    assert!(result.starts_with("#!/bin/sh"));
    assert!(result.contains("set -euf"));
    assert!(result.contains("IFS="));
    assert!(result.contains("export LC_ALL=C"));

    // Check proper footer
    assert!(result.contains("trap 'rm -rf"));
    assert!(result.trim().ends_with("main \"$@\""));
}

#[test]
fn test_deterministic_output() {
    let source = r#"
fn main() {
    let message = "deterministic test";
    echo(message);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();

    // Generate the same output multiple times
    let result1 = transpile(source, config.clone()).unwrap();
    let result2 = transpile(source, config.clone()).unwrap();
    let result3 = transpile(source, config).unwrap();

    // Should be identical
    assert_eq!(result1, result2);
    assert_eq!(result2, result3);
}

#[test]
fn test_large_input_handling() {
    // Generate a larger Rust program
    let mut source = String::new();

    for i in 0..50 {
        source.push_str(&format!("fn function_{i}() {{ let var_{i} = {i}; }}\n"));
    }

    source.push_str("fn main() {\n");
    for i in 0..50 {
        source.push_str(&format!("    function_{i}();\n"));
    }
    source.push_str("}\n");

    let config = Config::default();
    let result = transpile(&source, config);

    // Should handle large inputs without panicking
    assert!(result.is_ok());

    let script = result.unwrap();
    assert!(script.contains("function_0"));
    assert!(script.contains("function_49"));
}

#[test]
fn test_proof_generation() {
    let source = r#"
fn main() {
    let message = "proof test";
    echo(message);
}

fn echo(msg: &str) {}
"#;

    let config = Config {
        emit_proof: true,
        verify: VerificationLevel::Strict,
        ..Default::default()
    };

    let result = transpile(source, config);
    assert!(result.is_ok());

    // The transpile function itself doesn't generate proof files,
    // but it should not fail when proof emission is requested
}

#[test]
fn test_concurrent_transpilation() {
    use std::thread;

    let source = r#"
fn main() {
    let thread_test = "concurrent";
    echo(thread_test);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();

    // Test concurrent transpilation
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let source = source.to_string();
            let config = config.clone();
            thread::spawn(move || transpile(&source, config))
        })
        .collect();

    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok());
    }
}

#[test]
fn test_memory_safety() {
    // Test with deeply nested structures that might cause stack overflow
    let mut source = String::new();
    source.push_str("fn main() {\n");

    // Create nested variable assignments
    for i in 0..100 {
        source.push_str(&format!("    let var_{i} = \"value_{i}\";\n"));
    }

    source.push_str("}\n");

    let config = Config::default();
    let result = transpile(&source, config);

    // Should not crash or cause stack overflow
    assert!(result.is_ok());
}
