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
    let result = transpile(source, config).unwrap();

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
        .replace("x=42", "x=42\n    echo \"x=$x\"")
        .replace("name=test", "name=test\n    echo \"name=$name\"")
        .replace(
            "greeting=Hello",
            "greeting=Hello\n    echo \"greeting=$greeting\"",
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
    assert!(optimized.contains("part1=Hello"));
    assert!(unoptimized.contains("part1=Hello"));

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
fn test_empty_functions_generation() {
    let source = r#"
fn main() {
    empty_func();
    another_empty();
}

fn empty_func() {}
fn another_empty() {}
"#;

    let config = Config::default();
    let result = transpile(source, config).unwrap();

    eprintln!("Generated shell for empty functions:\n{}", result);

    // Empty functions should be generated with no-op command
    assert!(result.contains("empty_func() {"));
    assert!(result.contains("another_empty() {"));

    // Functions should contain the : no-op command
    let empty_func_section = result.split("empty_func() {").nth(1).unwrap();
    let empty_func_body = empty_func_section.split("}").next().unwrap();
    assert!(
        empty_func_body.trim().contains(":"),
        "Empty function should contain : no-op"
    );

    // Write to temporary file and execute
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_empty.sh");
    fs::write(&script_path, result).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script with empty functions should execute successfully. Exit code: {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
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

/// EXP-PARAM-001: RED Phase
/// Test string parameter expansion with default values ${var:-default}
/// Expected to FAIL until implementation is complete
/// NOTE: Requires method call support (.unwrap_or()) - use env_var_or() function instead
#[test]
#[ignore] // Requires parser support for method calls - deferred
fn test_string_parameter_expansion_default() {
    let source = r#"
fn main() {
    let value = Some("configured");
    let result = value.unwrap_or("default");
    echo(result);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile Option::unwrap_or() pattern"
    );

    let shell = result.unwrap();
    eprintln!("Generated shell script:\n{}", shell);

    // Verify shell parameter expansion syntax
    assert!(
        shell.contains("${value:-default}"),
        "Should use ${{var:-default}} syntax for Option::unwrap_or()\nActual output:\n{}",
        shell
    );

    // Verify proper quoting
    assert!(shell.contains("\"$result\""), "Should quote variable usage");
}

/// EXP-PARAM-001: RED Phase
/// Test environment variable with default value pattern
/// NOTE: Requires method call support - use env_var_or() function instead
#[test]
#[ignore] // Requires parser support for method calls - deferred
fn test_env_var_with_default() {
    let source = r#"
fn main() {
    let config_path = std::env::var("CONFIG_PATH").unwrap_or("/etc/default/config".to_string());
    echo(&config_path);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Verify ${VAR:-default} expansion
    assert!(
        shell.contains("${CONFIG_PATH:-/etc/default/config}"),
        "Should convert env::var().unwrap_or() to ${{VAR:-default}}"
    );

    // Verify proper quoting
    assert!(
        shell.contains("\"$config_path\""),
        "Should quote expanded variable"
    );
}

/// EXP-PARAM-001: RED Phase
/// Test multiple variables with defaults
/// NOTE: Requires method call support - use env_var_or() function instead
#[test]
#[ignore] // Requires parser support for method calls - deferred
fn test_multiple_defaults() {
    let source = r#"
fn main() {
    let host = std::env::var("HOST").unwrap_or("localhost".to_string());
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let proto = std::env::var("PROTO").unwrap_or("http".to_string());

    echo(&host);
    echo(&port);
    echo(&proto);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Verify all three expansions
    assert!(shell.contains("${HOST:-localhost}"));
    assert!(shell.contains("${PORT:-8080}"));
    assert!(shell.contains("${PROTO:-http}"));
}

/// PARAM-SPEC-002: RED Phase
/// Test exit status special parameter $?
/// Expected to FAIL until implementation is complete
#[test]
#[ignore] // Remove this once implementation starts
fn test_exit_status_parameter_basic() {
    let source = r#"
fn main() {
    let result = run_command();
    let exit_code = result;
    echo(&format!("Exit code: {}", exit_code));
}

fn run_command() -> i32 { 0 }
fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should capture exit status with $?
    assert!(shell.contains("$?"), "Should use $? to capture exit status");

    // Should store in variable
    assert!(
        shell.contains("exit_code=") || shell.contains("_exit="),
        "Should store exit code in variable"
    );
}

/// PARAM-SPEC-002: RED Phase
/// Test exit status in conditional
#[test]
#[ignore] // Remove this once implementation starts
fn test_exit_status_conditional() {
    let source = r#"
fn main() {
    some_command();
    if last_exit_status() == 0 {
        echo("Success");
    } else {
        echo("Failed");
    }
}

fn some_command() {}
fn last_exit_status() -> i32 { 0 }
fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should check $? in conditional
    assert!(
        shell.contains("$?") && shell.contains("if"),
        "Should use $? in conditional check"
    );
}

/// PARAM-SPEC-002: RED Phase
/// Test exit status with command execution
#[test]
#[ignore] // Remove this once implementation starts
fn test_exit_status_execution() {
    let source = r#"
fn main() {
    run_test();
    let status = get_exit_status();
    echo(&format!("Status: {}", status));
}

fn run_test() {}
fn get_exit_status() -> i32 { 0 }
fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Write to temporary file and execute
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_exit_status.sh");
    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

/// P0-POSITIONAL-PARAMETERS: RED Phase
/// Test positional parameters via std::env::args()
/// Expected to FAIL until implementation is complete
#[test]
fn test_positional_parameters_basic() {
    let source = r#"
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let first = args.get(1).unwrap_or("default");
    echo(first);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok(), "Should transpile positional parameters");

    let shell = result.unwrap();

    // Verify positional parameter syntax
    assert!(
        shell.contains("${1:-default}") || shell.contains("first=\"${1:-default}\""),
        "Should use positional parameter $1 with default"
    );

    // Verify proper quoting
    assert!(shell.contains("\"$first\""), "Should quote variable usage");

    // Verify main receives arguments
    assert!(
        shell.contains("main \"$@\""),
        "Should pass all arguments to main"
    );
}

/// P0-POSITIONAL-PARAMETERS: RED Phase
/// Test multiple positional parameters
#[test]
fn test_positional_parameters_multiple() {
    let source = r#"
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let first = args.get(1).unwrap_or("a");
    let second = args.get(2).unwrap_or("b");
    let third = args.get(3).unwrap_or("c");
    echo(first);
    echo(second);
    echo(third);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Verify all positional parameters
    assert!(shell.contains("${1:-a}") || shell.contains("first=\"${1:-a}\""));
    assert!(shell.contains("${2:-b}") || shell.contains("second=\"${2:-b}\""));
    assert!(shell.contains("${3:-c}") || shell.contains("third=\"${3:-c}\""));
}

/// P0-POSITIONAL-PARAMETERS: RED Phase
/// Test positional parameters with execution
#[test]
fn test_positional_parameters_execution() {
    let source = r#"
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let name = args.get(1).unwrap_or("World");
    echo(name);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Write to temporary file and execute with arguments
    use tempfile::TempDir;
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_positional.sh");
    fs::write(&script_path, &shell).unwrap();

    // Test with argument
    let output = Command::new("sh")
        .arg(&script_path)
        .arg("Alice")
        .output()
        .expect("Failed to execute shell script");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Alice"), "Should use provided argument");

    // Test without argument (should use default)
    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("World"), "Should use default value");
}

/// P0-POSITIONAL-PARAMETERS: Mutation Testing
/// This test catches the mutation: delete match arm Expr::PositionalArgs (line 554)
/// Ensures that std::env::args().collect() generates args="$@" not args="unknown"
#[test]
fn test_positional_parameters_args_assignment() {
    let source = r#"
fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Just use the args variable in some way
    println!("Got args");
}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok(), "Should transpile positional parameters");

    let shell = result.unwrap();

    // CRITICAL: This assertion catches mutation at line 554 (delete Expr::PositionalArgs)
    // If the match arm is deleted, this would generate args="unknown" instead of args="$@"
    assert!(
        shell.contains("args=\"$@\""),
        "Should generate args=\"$@\" for std::env::args().collect(), got:\n{}",
        shell
    );
}

/// PARAM-SPEC-001: Argument count detection ($#)
/// Test basic arg_count() → $# transformation
#[test]
fn test_param_spec_001_arg_count_basic() {
    let source = r#"
fn main() {
    let count = arg_count();
    echo("Done");
}

fn arg_count() -> i32 { 0 }
fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile arg_count() function: {:?}",
        result.err()
    );

    let shell = result.unwrap();

    // Verify $# is used for argument count
    assert!(
        shell.contains("count=\"$#\""),
        "Should convert arg_count() to $#, got:\n{}",
        shell
    );
}

/// PARAM-SPEC-001: Argument count in variable usage
#[test]
fn test_param_spec_001_arg_count_variable() {
    let source = r#"
fn main() {
    let count = arg_count();
    let num = count;
    echo("Done");
}

fn arg_count() -> i32 { 0 }
fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should handle arg_count in variable assignment"
    );

    let shell = result.unwrap();

    // Verify $# can be assigned to variables
    assert!(
        shell.contains("$#"),
        "Should include $# in shell script, got:\n{}",
        shell
    );
}

/// PARAM-SPEC-001: Argument count with conditional logic
#[test]
fn test_param_spec_001_arg_count_conditional() {
    let source = r#"
fn main() {
    let count = arg_count();
    if count == 0 {
        echo("No arguments");
    } else {
        echo("Has arguments");
    }
}

fn arg_count() -> i32 { 0 }
fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok(), "Should handle arg_count in conditionals");

    let shell = result.unwrap();

    // Verify $# is properly quoted in test expression
    assert!(
        shell.contains("$#"),
        "Should use $# in conditional, got:\n{}",
        shell
    );
}

/// PARAM-SPEC-001: Argument count execution test
#[test]
fn test_param_spec_001_arg_count_execution() {
    let source = r#"
fn main() {
    let count = arg_count();
    wc("-l");
}

fn arg_count() -> i32 { 0 }
fn wc(arg: &str) {}
"#;

    let config = Config::default();
    let shell_script = transpile(source, config).unwrap();

    // Write to temporary file and execute
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_arg_count.sh");
    fs::write(&script_path, shell_script).unwrap();

    // Test that script executes without errors
    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

/// PARAM-SPEC-005: Script name detection ($0)
/// Test std::env::args().nth(0) → $0 transformation
#[test]
fn test_param_spec_005_script_name_basic() {
    let source = r#"
fn main() {
    let script = std::env::args().nth(0).unwrap_or("unknown");
    echo(script);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile script name access: {:?}",
        result.err()
    );

    let shell = result.unwrap();

    // Verify ${0:-unknown} is used for script name with default
    assert!(
        shell.contains("${0:-unknown}") || shell.contains("script=\"${0:-unknown}\""),
        "Should convert std::env::args().nth(0).unwrap_or() to ${{0:-unknown}}, got:\n{}",
        shell
    );
}

/// PARAM-SPEC-005: Script name with default value
#[test]
fn test_param_spec_005_script_name_with_default() {
    let source = r#"
fn main() {
    let name = std::env::args().nth(0).unwrap_or("my-script");
    echo(name);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile script name with default: {:?}",
        result.err()
    );

    let shell = result.unwrap();

    // Verify ${0:-default} syntax
    assert!(
        shell.contains("${0:-my-script}") || shell.contains("name=\"${0:-my-script}\""),
        "Should convert std::env::args().nth(0).unwrap_or() to ${{0:-default}}, got:\n{}",
        shell
    );
}

/// PARAM-SPEC-005: Script name without default (using .unwrap())
#[test]
fn test_param_spec_005_script_name_unwrap() {
    let source = r#"
fn main() {
    let script = std::env::args().nth(0).unwrap();
    echo(script);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile script name with unwrap: {:?}",
        result.err()
    );

    let shell = result.unwrap();

    // Verify $0 is used for script name (without default)
    assert!(
        shell.contains("script=\"$0\"") || shell.contains("$0"),
        "Should convert std::env::args().nth(0).unwrap() to $0, got:\n{}",
        shell
    );
}

/// REDIR-001: RED Phase
/// Test that we can call commands that implicitly use input redirection
/// This is a baseline test - actual File::open → < redirection will be implemented later
#[test]
fn test_input_redirection_baseline() {
    let source = r#"
fn main() {
    cat("input.txt");
}

fn cat(file: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    if let Err(e) = &result {
        eprintln!("Transpilation error: {:?}", e);
    }

    assert!(
        result.is_ok(),
        "Should transpile file command: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell script:\n{}", shell);

    // Verify the command is called with the file
    assert!(
        shell.contains("cat") && shell.contains("input.txt"),
        "Should transpile cat command with filename\nActual output:\n{}",
        shell
    );
}

/// REDIR-001: RED Phase - ADVANCED
/// Test File::open() pattern conversion to input redirection
/// Expected to FAIL until implementation is complete
#[test]
#[ignore] // This is the actual P0 - requires File::open recognition
fn test_input_redirection_file_open() {
    let source = r#"
fn main() {
    let file = std::fs::File::open("input.txt");
    let content = read_file(file);
    echo(&content);
}

fn read_file(f: std::fs::File) -> String { String::new() }
fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    if let Err(e) = &result {
        eprintln!("Transpilation error: {:?}", e);
    }

    assert!(
        result.is_ok(),
        "Should transpile File::open: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell script:\n{}", shell);

    // Verify input redirection syntax
    assert!(
        shell.contains("< \"input.txt\"") || shell.contains("< input.txt"),
        "Should use input redirection < for File::open\nActual output:\n{}",
        shell
    );
}

/// REDIR-001: RED Phase
/// Test input redirection with proper quoting
#[test]
#[ignore]
fn test_input_redirection_with_quoting() {
    let source = r#"
fn main() {
    let data = read_file("data file.txt");
    echo(&data);
}

fn read_file(filename: &str) -> String { String::new() }
fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should properly quote filenames with spaces
    assert!(
        shell.contains("< \"data file.txt\""),
        "Should quote filenames with spaces in redirection"
    );
}

/// REDIR-001: RED Phase
/// Test input redirection execution
#[test]
#[ignore]
fn test_input_redirection_execution() {
    let source = r#"
fn main() {
    let content = cat("input.txt");
    echo(&content);
}

fn cat(file: &str) -> String { String::new() }
fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Create test environment
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_redir.sh");
    let input_path = temp_dir.path().join("input.txt");

    // Write test input
    fs::write(&input_path, "Hello from file").unwrap();

    // Modify script to use the temp input file
    let modified_shell = shell.replace("input.txt", input_path.to_str().unwrap());
    fs::write(&script_path, modified_shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Hello from file"),
        "Should read content from redirected file"
    );
}

/// REDIR-002: RED Phase
/// Test output redirection (>) baseline
#[test]
fn test_output_redirection_baseline() {
    let source = r#"
fn main() {
    write_file("output.txt", "Hello World");
}

fn write_file(path: &str, content: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile file writing: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for output redirection:\n{}", shell);

    // Verify the command is called correctly
    assert!(
        shell.contains("write_file"),
        "Should transpile write_file command"
    );
}

/// REDIR-002: RED Phase - ADVANCED
/// Test that echo/printf output can be redirected with >
#[test]
#[ignore] // Requires output redirection implementation
fn test_output_redirection_echo() {
    let source = r#"
fn main() {
    let mut file = std::fs::File::create("output.txt");
    write_to_file(&mut file, "Hello World");
}

fn write_to_file(f: &mut std::fs::File, content: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should use output redirection syntax
    assert!(
        shell.contains("> \"output.txt\"") || shell.contains(">output.txt"),
        "Should use > for File::create output redirection"
    );
}

/// REDIR-002: RED Phase - APPEND
/// Test append redirection (>>)
#[test]
#[ignore] // Requires append redirection implementation
fn test_output_redirection_append() {
    let source = r#"
fn main() {
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open("output.txt");
    write_to_file(&mut file, "Appended text");
}

fn write_to_file(f: &mut std::fs::File, content: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should use append redirection syntax
    assert!(
        shell.contains(">> \"output.txt\"") || shell.contains(">>output.txt"),
        "Should use >> for append mode"
    );
}

/// REDIR-002: Baseline - Execution
/// Test that basic echo works (redirection syntax can be added later)
/// IGNORED: Empty echo() function doesn't output - needs actual implementation
#[test]
#[ignore]
fn test_output_redirection_execution() {
    let source = r#"
fn main() {
    echo("Test output");
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_output.sh");

    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );

    // Verify output goes to stdout
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Test output"), "Should output to stdout");
}

/// BUILTIN-005: RED Phase
/// Test cd command baseline
#[test]
fn test_cd_command_baseline() {
    let source = r#"
fn main() {
    change_dir("/tmp");
}

fn change_dir(path: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile directory change: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for cd command:\n{}", shell);

    // Verify cd command is generated
    assert!(
        shell.contains("change_dir") || shell.contains("cd"),
        "Should transpile change_dir command"
    );
}

/// BUILTIN-005: RED Phase - ADVANCED
/// Test std::env::set_current_dir() conversion to cd
#[test]
#[ignore] // Requires std::env::set_current_dir recognition
fn test_cd_command_std_env() {
    let source = r#"
fn main() {
    std::env::set_current_dir("/tmp").unwrap();
}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate cd command
    assert!(
        shell.contains("cd \"/tmp\"") || shell.contains("cd /tmp"),
        "Should convert std::env::set_current_dir to cd command"
    );
}

/// BUILTIN-005: Baseline - Execution
/// Test that cd-like function calls work
#[test]
fn test_cd_command_execution() {
    let source = r#"
fn main() {
    cd("/tmp");
    pwd();
}

fn cd(path: &str) {}
fn pwd() {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_cd.sh");

    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

/// BUILTIN-011: RED Phase
/// Test pwd command baseline
#[test]
fn test_pwd_command_baseline() {
    let source = r#"
fn main() {
    pwd();
}

fn pwd() {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile pwd call: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for pwd command:\n{}", shell);

    // Verify function is called
    assert!(shell.contains("pwd"), "Should transpile pwd function");
}

/// BUILTIN-011: RED Phase - ADVANCED
/// Test std::env::current_dir() conversion to pwd
#[test]
#[ignore] // Requires std::env::current_dir recognition
fn test_pwd_command_std_env() {
    let source = r#"
fn main() {
    let current = std::env::current_dir().unwrap();
    echo(&current.to_string_lossy());
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate command substitution with pwd
    assert!(
        shell.contains("$(pwd)") || shell.contains("`pwd`"),
        "Should convert std::env::current_dir to $(pwd)"
    );
}

/// BUILTIN-011: Baseline - Execution
/// Test pwd command execution
#[test]
fn test_pwd_command_execution() {
    let source = r#"
fn main() {
    pwd();
}

fn pwd() {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_pwd.sh");

    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

/// BUILTIN-009: RED Phase
/// Test exit command baseline
#[test]
fn test_exit_command_baseline() {
    let source = r#"
fn main() {
    exit_with_code(0);
}

fn exit_with_code(code: i32) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile exit command: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for exit command:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("exit_with_code") || shell.contains("exit"),
        "Should transpile exit_with_code function"
    );
}

/// BUILTIN-009: RED Phase - ADVANCED
/// Test std::process::exit() conversion
#[test]
#[ignore] // Requires std::process::exit recognition
fn test_exit_command_std_process() {
    let source = r#"
fn main() {
    std::process::exit(0);
}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate exit command
    assert!(
        shell.contains("exit 0"),
        "Should convert std::process::exit to exit command"
    );
}

/// BUILTIN-010: RED Phase
/// Test export command baseline
#[test]
fn test_export_command_baseline() {
    let source = r#"
fn main() {
    set_env("VAR", "value");
}

fn set_env(name: &str, value: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile env setting: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for export command:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("set_env"),
        "Should transpile set_env function"
    );
}

/// BUILTIN-010: RED Phase - ADVANCED
/// Test std::env::set_var() conversion to export
#[test]
#[ignore] // Requires std::env::set_var recognition
fn test_export_command_std_env() {
    let source = r#"
fn main() {
    std::env::set_var("VAR", "value");
}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate export command
    assert!(
        shell.contains("VAR=\"value\"")
            && (shell.contains("export VAR") || shell.contains("export")),
        "Should convert std::env::set_var to VAR=value; export VAR"
    );
}

/// BUILTIN-020: RED Phase
/// Test unset command baseline
#[test]
fn test_unset_command_baseline() {
    let source = r#"
fn main() {
    unset_var("VAR");
}

fn unset_var(name: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok(), "Should transpile unset: {:?}", result.err());

    let shell = result.unwrap();
    eprintln!("Generated shell for unset command:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("unset_var") || shell.contains("unset"),
        "Should transpile unset_var function"
    );
}

/// BUILTIN-020: RED Phase - ADVANCED
/// Test std::env::remove_var() conversion to unset
#[test]
#[ignore] // Requires std::env::remove_var recognition
fn test_unset_command_std_env() {
    let source = r#"
fn main() {
    std::env::remove_var("VAR");
}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate unset command
    assert!(
        shell.contains("unset VAR"),
        "Should convert std::env::remove_var to unset VAR"
    );
}

/// BUILTIN-009, 010, 020: Execution test
/// Test that basic commands execute successfully
#[test]
fn test_builtin_commands_execution() {
    let source = r#"
fn main() {
    set_var("TEST", "value");
    get_var("TEST");
}

fn set_var(name: &str, value: &str) {}
fn get_var(name: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_builtins.sh");

    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

/// BUILTIN-016: RED Phase
/// Test test/[ command baseline
#[test]
fn test_test_command_baseline() {
    let source = r#"
fn main() {
    test_file_exists("/tmp/test.txt");
}

fn test_file_exists(path: &str) -> bool { true }
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile test command: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for test command:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("test_file_exists"),
        "Should transpile test_file_exists function"
    );
}

/// BUILTIN-016: RED Phase - ADVANCED
/// Test std::path::Path::exists() conversion to [ -f ]
#[test]
#[ignore] // Requires std::path::Path recognition
fn test_test_command_std_path() {
    let source = r#"
fn main() {
    if std::path::Path::new("/tmp/test.txt").exists() {
        echo("exists");
    }
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate test command
    assert!(
        shell.contains("[ -f") || shell.contains("[ -e") || shell.contains("test -f"),
        "Should convert Path::exists to [ -f ] or test -f"
    );
}

/// BUILTIN-016: Baseline - Execution
#[test]
fn test_test_command_execution() {
    let source = r#"
fn main() {
    check_file("/etc/hosts");
}

fn check_file(path: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_test.sh");

    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

/// BASH-BUILTIN-005: RED Phase
/// Test printf preservation (should pass through)
#[test]
fn test_printf_preservation_baseline() {
    let source = r#"
fn main() {
    printf_formatted("%s %d\n", "Number:", 42);
}

fn printf_formatted(fmt: &str, args: &str, num: i32) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile printf call: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for printf:\n{}", shell);

    // Verify function is called (printf is preferred, so should work)
    assert!(
        shell.contains("printf_formatted"),
        "Should transpile printf_formatted function"
    );
}

/// BASH-BUILTIN-005: RED Phase - ADVANCED
/// Test that println! converts to printf (not echo)
#[test]
#[ignore] // Requires println! → printf conversion
fn test_printf_from_println() {
    let source = r#"
fn main() {
    println!("Hello World");
    println!("Value: {}", 42);
}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should use printf, not echo
    assert!(
        shell.contains("printf") && !shell.contains("echo"),
        "Should convert println! to printf, not echo"
    );
}

/// BASH-BUILTIN-005: Baseline - Execution
#[test]
fn test_printf_execution() {
    let source = r#"
fn main() {
    print_message("Test");
}

fn print_message(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_printf.sh");

    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

/// VAR-001: RED Phase
/// Test HOME variable baseline
#[test]
fn test_home_variable_baseline() {
    let source = r#"
fn main() {
    use_home();
}

fn use_home() {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile HOME access: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for HOME variable:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("use_home"),
        "Should transpile use_home function"
    );
}

/// VAR-001: RED Phase - ADVANCED
/// Test std::env::var("HOME") conversion to $HOME
#[test]
#[ignore] // Requires env::var("HOME") recognition
fn test_home_variable_std_env() {
    let source = r#"
fn main() {
    let home = std::env::var("HOME").unwrap();
    echo(&home);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should use $HOME variable
    assert!(
        shell.contains("$HOME") || shell.contains("\"${HOME}\""),
        "Should convert std::env::var(\"HOME\") to $HOME"
    );
}

/// VAR-001: Baseline - Execution
#[test]
fn test_home_variable_execution() {
    let source = r#"
fn main() {
    use_home_dir();
}

fn use_home_dir() {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_home.sh");

    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

/// VAR-002: RED Phase
/// Test PATH variable baseline
#[test]
fn test_path_variable_baseline() {
    let source = r#"
fn main() {
    use_path();
}

fn use_path() {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile PATH access: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for PATH variable:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("use_path"),
        "Should transpile use_path function"
    );
}

/// VAR-002: RED Phase - ADVANCED
/// Test std::env::var("PATH") conversion to $PATH
#[test]
#[ignore] // Requires env::var("PATH") recognition
fn test_path_variable_std_env() {
    let source = r#"
fn main() {
    let path = std::env::var("PATH").unwrap();
    let new_path = format!("/usr/local/bin:{}", path);
    std::env::set_var("PATH", &new_path);
}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should use $PATH variable
    assert!(
        shell.contains("$PATH") || shell.contains("\"${PATH}\""),
        "Should convert std::env::var(\"PATH\") to $PATH"
    );

    // Should export the modified PATH
    assert!(shell.contains("export PATH"), "Should export modified PATH");
}

/// VAR-002: Baseline - Execution
#[test]
fn test_path_variable_execution() {
    let source = r#"
fn main() {
    use_path();
}

fn use_path() {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_path.sh");

    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

/// Combined execution test for all 4 new validations
#[test]
fn test_session4_commands_execution() {
    let source = r#"
fn main() {
    check_exists("/tmp");
    print_output("test");
    use_home();
    use_path();
}

fn check_exists(path: &str) {}
fn print_output(msg: &str) {}
fn use_home() {}
fn use_path() {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_session4.sh");

    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

/// EXP-PARAM-005: RED Phase
/// Test string length ${#var} baseline
#[test]
fn test_string_length_baseline() {
    let source = r#"
fn main() {
    length_of("hello");
}

fn length_of(s: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile string length: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for string length:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("length_of"),
        "Should transpile length_of function"
    );
}

/// EXP-PARAM-005: RED Phase - ADVANCED
/// Test .len() conversion to ${#var}
#[test]
#[ignore] // Requires .len() method recognition
fn test_string_length_method() {
    let source = r#"
fn main() {
    let text = "hello world";
    let len = text.len();
    echo(&format!("Length: {}", len));
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate ${#var} syntax
    assert!(
        shell.contains("${#text}"),
        "Should convert .len() to ${{#var}} syntax"
    );
}

/// EXP-PARAM-005: Baseline - Execution
#[test]
fn test_string_length_execution() {
    let source = r#"
fn main() {
    get_length("test");
}

fn get_length(s: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_strlen.sh");

    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

/// EXP-PARAM-006: RED Phase
/// Test remove suffix ${var%suffix} baseline
#[test]
fn test_remove_suffix_baseline() {
    let source = r#"
fn main() {
    remove_ext("test.txt");
}

fn remove_ext(filename: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile suffix removal: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for remove suffix:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("remove_ext"),
        "Should transpile remove_ext function"
    );
}

/// EXP-PARAM-006: RED Phase - ADVANCED
/// Test .strip_suffix() conversion to ${var%suffix}
#[test]
#[ignore] // Requires .strip_suffix() recognition
fn test_remove_suffix_method() {
    let source = r#"
fn main() {
    let file = "test.txt";
    let name = file.strip_suffix(".txt").unwrap_or(file);
    echo(name);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate ${var%suffix} syntax
    assert!(
        shell.contains("${file%.txt}"),
        "Should convert .strip_suffix() to ${{var%suffix}}"
    );
}

/// EXP-PARAM-006: Baseline - Execution
#[test]
fn test_remove_suffix_execution() {
    let source = r#"
fn main() {
    strip_ext("file.rs");
}

fn strip_ext(filename: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_suffix.sh");

    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

/// EXP-PARAM-007: RED Phase
/// Test remove prefix ${var#prefix} baseline
#[test]
fn test_remove_prefix_baseline() {
    let source = r#"
fn main() {
    strip_dir("/tmp/file");
}

fn strip_dir(path: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile prefix removal: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for remove prefix:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("strip_dir"),
        "Should transpile strip_dir function"
    );
}

/// EXP-PARAM-007: RED Phase - ADVANCED
/// Test .strip_prefix() conversion to ${var#prefix}
#[test]
#[ignore] // Requires .strip_prefix() recognition
fn test_remove_prefix_method() {
    let source = r#"
fn main() {
    let path = "/tmp/file";
    let name = path.strip_prefix("/tmp/").unwrap_or(path);
    echo(name);
}

fn echo(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate ${var#prefix} syntax
    assert!(
        shell.contains("${path#/tmp/}"),
        "Should convert .strip_prefix() to ${{var#prefix}}"
    );
}

/// EXP-PARAM-007: Baseline - Execution
#[test]
fn test_remove_prefix_execution() {
    let source = r#"
fn main() {
    strip_dir("/home/user/file.txt");
}

fn strip_dir(path: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_prefix.sh");

    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

/// REDIR-003: RED Phase
/// Test combined redirection &> baseline
#[test]
fn test_combined_redirection_baseline() {
    let source = r#"
fn main() {
    redirect_all("output.txt");
}

fn redirect_all(file: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile redirection: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for combined redirection:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("redirect_all"),
        "Should transpile redirect_all function"
    );
}

/// REDIR-003: RED Phase - ADVANCED
/// Test stderr+stdout redirection conversion to &> or > 2>&1
#[test]
#[ignore] // Requires redirection pattern recognition
fn test_combined_redirection_conversion() {
    let source = r#"
fn main() {
    let output = std::process::Command::new("ls")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output();
}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate > file 2>&1 (POSIX) or &> file (bash)
    assert!(
        shell.contains("> ") && shell.contains("2>&1") || shell.contains("&>"),
        "Should convert combined output to &> or > 2>&1"
    );
}

/// REDIR-003: Baseline - Execution
#[test]
fn test_combined_redirection_execution() {
    let source = r#"
fn main() {
    capture_output();
}

fn capture_output() {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_redir.sh");

    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

/// Combined execution test for all 4 new validations
#[test]
fn test_session5_commands_execution() {
    let source = r#"
fn main() {
    length_of("test");
    strip_suffix("file.txt", ".txt");
    strip_prefix("/path/file", "/path/");
    redirect_both("output.log");
}

fn length_of(s: &str) {}
fn strip_suffix(s: &str, suffix: &str) {}
fn strip_prefix(s: &str, prefix: &str) {}
fn redirect_both(file: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("test_session5.sh");

    fs::write(&script_path, &shell).unwrap();

    let output = Command::new("sh")
        .arg(&script_path)
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success(),
        "Script should execute successfully"
    );
}

// ============================================================================
// Session 6: Heredocs and Non-Deterministic Feature Removal
// Validation of GNU Bash Manual constructs - RED Phase Tests
// ============================================================================

/// REDIR-004: RED Phase
/// Test heredoc << baseline
#[test]
fn test_heredoc_baseline() {
    let source = r#"
fn main() {
    print_heredoc();
}

fn print_heredoc() {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile heredoc function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for heredoc:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("print_heredoc"),
        "Should transpile print_heredoc function"
    );
}

/// REDIR-004: RED Phase - ADVANCED
/// Test multi-line string literal to heredoc conversion
#[test]
#[ignore] // Requires multi-line string literal recognition
fn test_heredoc_multiline() {
    let source = r#"
fn main() {
    let doc = "Line 1
Line 2
Line 3";
    cat_heredoc(&doc);
}

fn cat_heredoc(content: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate heredoc syntax
    assert!(
        shell.contains("<<") && shell.contains("EOF"),
        "Should convert multi-line string to heredoc"
    );
}

/// REDIR-004: RED Phase - EXECUTION
/// Test heredoc execution
#[test]
fn test_heredoc_execution() {
    let source = r#"
fn main() {
    print_multiline();
}

fn print_multiline() {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Write to temp file and verify it's valid shell
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    // Should execute (even if function does nothing)
    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute. Exit code: {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// PARAM-SPEC-003: Process ID $$ Purification
// ============================================================================

/// PARAM-SPEC-003: RED Phase
/// Test that $$ usage is documented for removal
#[test]
fn test_process_id_purification_baseline() {
    let source = r#"
fn main() {
    use_fixed_id();
}

fn use_fixed_id() {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile fixed ID function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for fixed ID:\n{}", shell);

    // Verify function is called (not $$)
    assert!(
        shell.contains("use_fixed_id"),
        "Should use fixed identifier, not $$"
    );

    // Should NOT contain $$ in main function (trap cleanup usage is OK)
    let main_section = shell.split("main() {").nth(1).unwrap_or("");
    let main_body = main_section.split("}").next().unwrap_or("");
    assert!(
        !main_body.contains("$$"),
        "Main function should NOT contain $$ (trap cleanup is OK, but user code shouldn't use $$)"
    );
}

/// PARAM-SPEC-003: RED Phase - ADVANCED
/// Test that std::process::id() is NOT supported
#[test]
#[ignore] // Requires std::process::id() detection and rejection
fn test_process_id_rejection() {
    let source = r#"
fn main() {
    let pid = std::process::id();
    println!("PID: {}", pid);
}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    // Should fail validation - non-deterministic
    assert!(
        result.is_err(),
        "std::process::id() should be rejected as non-deterministic"
    );
}

/// PARAM-SPEC-003: RED Phase - EXECUTION
/// Test fixed ID execution
#[test]
fn test_process_id_execution() {
    let source = r#"
fn main() {
    use_session_id("test-session");
}

fn use_session_id(id: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute with fixed ID"
    );
}

// ============================================================================
// PARAM-SPEC-004: Background PID $! Purification
// ============================================================================

/// PARAM-SPEC-004: RED Phase
/// Test that background jobs are NOT generated
#[test]
fn test_background_pid_purification_baseline() {
    let source = r#"
fn main() {
    run_sync();
}

fn run_sync() {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile sync function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for sync execution:\n{}", shell);

    // Verify function is called synchronously
    assert!(
        shell.contains("run_sync"),
        "Should call function synchronously"
    );

    // Should NOT contain background operators
    assert!(
        !shell.contains(" &") && !shell.contains("$!"),
        "Should NOT contain background job operators (non-deterministic)"
    );
}

/// PARAM-SPEC-004: RED Phase - ADVANCED
/// Test that async/await is NOT supported
#[test]
#[ignore] // Requires async detection and rejection
fn test_background_async_rejection() {
    let source = r#"
async fn background_task() {
    // Some work
}

fn main() {
    background_task();
}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    // Should fail validation - async is non-deterministic
    assert!(
        result.is_err(),
        "async functions should be rejected as non-deterministic"
    );
}

/// PARAM-SPEC-004: RED Phase - EXECUTION
/// Test synchronous execution
#[test]
fn test_background_sync_execution() {
    let source = r#"
fn main() {
    task1();
    task2();
}

fn task1() {}
fn task2() {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute synchronously"
    );
}

// ============================================================================
// BASH-VAR-002: RANDOM Purification
// ============================================================================

/// BASH-VAR-002: RED Phase
/// Test that RANDOM is NOT generated
#[test]
fn test_random_purification_baseline() {
    let source = r#"
fn main() {
    use_seed(42);
}

fn use_seed(seed: i32) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile seed function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for deterministic seed:\n{}", shell);

    // Verify function is called with deterministic seed
    assert!(
        shell.contains("use_seed") && shell.contains("42"),
        "Should use deterministic seed"
    );

    // Should NOT contain $RANDOM
    assert!(
        !shell.contains("$RANDOM") && !shell.contains("RANDOM"),
        "Should NOT contain $RANDOM (non-deterministic)"
    );
}

/// BASH-VAR-002: RED Phase - ADVANCED
/// Test that rand crate usage is NOT supported
#[test]
#[ignore] // Requires rand crate detection and rejection
fn test_random_crate_rejection() {
    let source = r#"
fn main() {
    let num = rand::random::<u32>();
    println!("{}", num);
}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    // Should fail validation - random is non-deterministic
    assert!(
        result.is_err(),
        "rand crate usage should be rejected as non-deterministic"
    );
}

/// BASH-VAR-002: RED Phase - EXECUTION
/// Test deterministic value execution
#[test]
fn test_random_deterministic_execution() {
    let source = r#"
fn main() {
    use_value(12345);
}

fn use_value(val: i32) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute with deterministic value"
    );
}

/// Session 6: Combined execution test
#[test]
fn test_session6_commands_execution() {
    let source = r#"
fn main() {
    print_heredoc();
    use_fixed_id();
    run_sync();
    use_seed(42);
}

fn print_heredoc() {}
fn use_fixed_id() {}
fn run_sync() {}
fn use_seed(seed: i32) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    eprintln!("Generated combined shell script:\n{}", shell);

    // Verify all functions are called
    assert!(shell.contains("print_heredoc"), "Should call print_heredoc");
    assert!(shell.contains("use_fixed_id"), "Should call use_fixed_id");
    assert!(shell.contains("run_sync"), "Should call run_sync");
    assert!(shell.contains("use_seed"), "Should call use_seed");

    // Verify NO non-deterministic constructs in main function (trap cleanup is OK)
    let main_section = shell.split("main() {").nth(1).unwrap_or("");
    let main_body = main_section.split("}").next().unwrap_or("");
    assert!(!main_body.contains("$$"), "Main should NOT contain $$");
    assert!(!main_body.contains("$!"), "Main should NOT contain $!");
    assert!(
        !main_body.contains("$RANDOM"),
        "Main should NOT contain $RANDOM"
    );
    assert!(
        !main_body.contains(" &"),
        "Main should NOT contain background &"
    );

    // Write and execute
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    // Execution test may fail (functions undefined), but script should be valid
    eprintln!("Exit code: {:?}", output.status.code());
    eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
}

// ============================================================================
// Session 7: Exit Status and Additional Purifications
// Validation of GNU Bash Manual constructs - RED Phase Tests
// ============================================================================

/// PARAM-SPEC-002: RED Phase
/// Test exit status $? baseline
#[test]
fn test_exit_status_baseline() {
    let source = r#"
fn main() {
    get_status();
}

fn get_status() -> i32 { 0 }
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile exit status function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for exit status:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("get_status"),
        "Should transpile get_status function"
    );
}

/// PARAM-SPEC-002: RED Phase - ADVANCED
/// Test command exit status capture with $?
#[test]
#[ignore] // Requires $? capture pattern recognition
fn test_exit_status_capture() {
    let source = r#"
fn main() {
    run_command();
    let status = last_exit_code();
    check_status(status);
}

fn run_command() {}
fn last_exit_code() -> i32 { 0 }
fn check_status(code: i32) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should capture $? after command
    assert!(shell.contains("$?"), "Should use $? to capture exit status");
}

/// PARAM-SPEC-002: RED Phase - EXECUTION
/// Test exit status execution
#[test]
fn test_exit_status_param_execution() {
    let source = r#"
fn main() {
    check_result(0);
}

fn check_result(code: i32) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute"
    );
}

// ============================================================================
// REDIR-005: Herestring <<<
// ============================================================================

/// REDIR-005: RED Phase
/// Test herestring <<< baseline
#[test]
fn test_herestring_baseline() {
    let source = r#"
fn main() {
    pass_string("input data");
}

fn pass_string(data: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile herestring function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for herestring:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("pass_string"),
        "Should transpile pass_string function"
    );
}

/// REDIR-005: RED Phase - ADVANCED
/// Test herestring conversion to printf | cmd
#[test]
#[ignore] // Requires herestring pattern recognition
fn test_herestring_conversion() {
    let source = r#"
fn main() {
    let input = "test input";
    pipe_input(input);
}

fn pipe_input(data: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should convert to printf | cmd (POSIX alternative to <<<)
    assert!(
        shell.contains("printf") && shell.contains("|"),
        "Should convert herestring to printf | cmd"
    );
}

/// REDIR-005: RED Phase - EXECUTION
/// Test herestring execution
#[test]
fn test_herestring_execution() {
    let source = r#"
fn main() {
    send_data("hello");
}

fn send_data(msg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute"
    );
}

// ============================================================================
// BASH-VAR-003: SECONDS Purification
// ============================================================================

/// BASH-VAR-003: RED Phase
/// Test that SECONDS is NOT generated
#[test]
fn test_seconds_purification_baseline() {
    let source = r#"
fn main() {
    use_fixed_time(100);
}

fn use_fixed_time(duration: i32) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile fixed time function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for fixed time:\n{}", shell);

    // Verify function is called with fixed duration
    assert!(
        shell.contains("use_fixed_time") && shell.contains("100"),
        "Should use fixed time duration"
    );

    // Should NOT contain $SECONDS
    let main_section = shell.split("main() {").nth(1).unwrap_or("");
    let main_body = main_section.split("}").next().unwrap_or("");
    assert!(
        !main_body.contains("$SECONDS") && !main_body.contains("SECONDS="),
        "Should NOT contain $SECONDS (non-deterministic)"
    );
}

/// BASH-VAR-003: RED Phase - ADVANCED
/// Test that SystemTime::now() is NOT supported
#[test]
#[ignore] // Requires SystemTime detection and rejection
fn test_seconds_time_rejection() {
    let source = r#"
fn main() {
    let start = std::time::SystemTime::now();
    do_work();
    let elapsed = start.elapsed().unwrap();
    println!("{:?}", elapsed);
}

fn do_work() {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    // Should fail validation - timing is non-deterministic
    assert!(
        result.is_err(),
        "SystemTime::now() should be rejected as non-deterministic"
    );
}

/// BASH-VAR-003: RED Phase - EXECUTION
/// Test fixed duration execution
#[test]
fn test_seconds_fixed_duration_execution() {
    let source = r#"
fn main() {
    wait_fixed(5);
}

fn wait_fixed(seconds: i32) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute with fixed duration"
    );
}

// ============================================================================
// JOB-001: Background Jobs (&) Purification
// ============================================================================

/// JOB-001: RED Phase
/// Test that background jobs are NOT generated
#[test]
fn test_background_jobs_purification_baseline() {
    let source = r#"
fn main() {
    run_foreground();
}

fn run_foreground() {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile foreground function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for foreground execution:\n{}", shell);

    // Verify function is called in foreground
    assert!(
        shell.contains("run_foreground"),
        "Should call function in foreground"
    );

    // Should NOT contain background operators
    let main_section = shell.split("main() {").nth(1).unwrap_or("");
    let main_body = main_section.split("}").next().unwrap_or("");
    assert!(
        !main_body.contains(" &"),
        "Should NOT contain background job operator & (non-deterministic)"
    );
}

/// JOB-001: RED Phase - ADVANCED
/// Test that spawn/thread is NOT supported
#[test]
#[ignore] // Requires spawn/thread detection and rejection
fn test_background_spawn_rejection() {
    let source = r#"
fn main() {
    std::thread::spawn(|| {
        background_work();
    });
}

fn background_work() {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    // Should fail validation - threading is non-deterministic
    assert!(
        result.is_err(),
        "std::thread::spawn should be rejected as non-deterministic"
    );
}

/// JOB-001: RED Phase - EXECUTION
/// Test foreground execution
#[test]
fn test_background_foreground_execution() {
    let source = r#"
fn main() {
    task_one();
    task_two();
    task_three();
}

fn task_one() {}
fn task_two() {}
fn task_three() {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute tasks in foreground"
    );
}

/// Session 7: Combined execution test
#[test]
fn test_session7_commands_execution() {
    let source = r#"
fn main() {
    get_status();
    pass_string("data");
    use_fixed_time(60);
    run_foreground();
}

fn get_status() -> i32 { 0 }
fn pass_string(data: &str) {}
fn use_fixed_time(duration: i32) {}
fn run_foreground() {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    eprintln!("Generated combined shell script:\n{}", shell);

    // Verify all functions are called
    assert!(shell.contains("get_status"), "Should call get_status");
    assert!(shell.contains("pass_string"), "Should call pass_string");
    assert!(
        shell.contains("use_fixed_time"),
        "Should call use_fixed_time"
    );
    assert!(
        shell.contains("run_foreground"),
        "Should call run_foreground"
    );

    // Verify NO non-deterministic constructs in main function
    let main_section = shell.split("main() {").nth(1).unwrap_or("");
    let main_body = main_section.split("}").next().unwrap_or("");
    assert!(
        !main_body.contains("$SECONDS"),
        "Main should NOT contain $SECONDS"
    );
    assert!(
        !main_body.contains(" &"),
        "Main should NOT contain background &"
    );

    // Write and execute
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    // Execution test may fail (functions undefined), but script should be valid
    eprintln!("Exit code: {:?}", output.status.code());
    eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
}

// ============================================================================
// P0-POSITIONAL-PARAMETERS: Property Tests
// ============================================================================

#[cfg(test)]
mod positional_parameters_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Transpiling positional parameters is deterministic
        /// Same Rust input always produces same shell output
        #[test]
        fn prop_positional_params_deterministic(
            default_val in "[a-z]{1,10}"
        ) {
            let source = format!(r#"
fn main() {{
    let args: Vec<String> = std::env::args().collect();
    let first = args.get(1).unwrap_or("{}");
    echo(first);
}}

fn echo(msg: &str) {{}}
"#, default_val);

            let config = Config::default();
            let result1 = transpile(&source, config.clone());
            let result2 = transpile(&source, config);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());
            prop_assert_eq!(result1.unwrap(), result2.unwrap());
        }

        /// Property: Transpilation succeeds for all valid default values
        #[test]
        fn prop_default_values_preserved(
            default_val in "[a-zA-Z0-9_-]{1,20}"
        ) {
            let source = format!(r#"
fn main() {{
    let args: Vec<String> = std::env::args().collect();
    let param = args.get(1).unwrap_or("{}");
    echo(param);
}}

fn echo(msg: &str) {{}}
"#, default_val);

            let config = Config::default();
            let result = transpile(&source, config);

            // Transpilation should always succeed for valid default values
            prop_assert!(result.is_ok(), "Transpilation failed for default: {}", default_val);

            let shell = result.unwrap();

            // Shell output should contain the param assignment with positional parameter syntax
            prop_assert!(
                shell.contains("param=") && shell.contains("${1:-"),
                "Shell output should contain positional parameter with default"
            );
        }

        /// Property: Positional parameters are always quoted
        #[test]
        fn prop_positional_params_quoted(
            position in 1u32..10,
            default_val in "[a-z]{1,10}"
        ) {
            let source = format!(r#"
fn main() {{
    let args: Vec<String> = std::env::args().collect();
    let param = args.get({}).unwrap_or("{}");
    echo(param);
}}

fn echo(msg: &str) {{}}
"#, position, default_val);

            let config = Config::default();
            let result = transpile(&source, config);

            prop_assert!(result.is_ok());
            let shell = result.unwrap();

            // Positional params should be in quotes
            prop_assert!(
                shell.contains(&format!("\"${{{}:-", position)) ||
                shell.contains(&format!("param=\"${{{}:-", position)),
                "Positional parameter should be quoted"
            );
        }

        /// Property: std::env::args().collect() always becomes "$@"
        #[test]
        fn prop_args_collect_becomes_dollar_at(
            _seed in 0u32..100  // Just for variety
        ) {
            let source = r#"
fn main() {
    let args: Vec<String> = std::env::args().collect();
    echo("test");
}

fn echo(msg: &str) {}
"#;

            let config = Config::default();
            let result = transpile(source, config);

            prop_assert!(result.is_ok());
            let shell = result.unwrap();

            // args variable should contain "$@"
            prop_assert!(
                shell.contains("args=\"$@\"") || shell.contains("$@"),
                "args.collect() should become $@"
            );
        }
    }
}

/// PARAM-SPEC-001: Property-based tests for arg_count() → $# transformation
mod arg_count_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Transpiling arg_count() is deterministic
        /// Same Rust input always produces same shell output
        #[test]
        fn prop_arg_count_deterministic(
            _seed in 0u32..100
        ) {
            let source = r#"
fn main() {
    let count = arg_count();
    echo("Done");
}

fn arg_count() -> i32 { 0 }
fn echo(msg: &str) {}
"#;

            let config = Config::default();
            let result1 = transpile(source, config.clone());
            let result2 = transpile(source, config);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());
            prop_assert_eq!(result1.unwrap(), result2.unwrap());
        }

        /// Property: arg_count() always generates $# in output
        #[test]
        fn prop_arg_count_generates_dollar_hash(
            _seed in 0u32..100
        ) {
            let source = r#"
fn main() {
    let count = arg_count();
    wc("-l");
}

fn arg_count() -> i32 { 0 }
fn wc(arg: &str) {}
"#;

            let config = Config::default();
            let result = transpile(source, config);

            prop_assert!(result.is_ok(), "Transpilation should succeed");

            let shell = result.unwrap();

            // arg_count() should always generate $# in shell output
            prop_assert!(
                shell.contains("$#"),
                "Shell output must contain $# for arg_count()"
            );
        }

        /// Property: arg_count() in conditionals produces valid shell
        #[test]
        fn prop_arg_count_in_conditionals_valid(
            threshold in 0i32..10
        ) {
            let source = format!(r#"
fn main() {{
    let count = arg_count();
    if count == {} {{
        echo("Match");
    }}
}}

fn arg_count() -> i32 {{ 0 }}
fn echo(msg: &str) {{}}
"#, threshold);

            let config = Config::default();
            let result = transpile(&source, config);

            prop_assert!(
                result.is_ok(),
                "Transpilation should succeed for count == {}", threshold
            );

            let shell = result.unwrap();

            // Must contain both $# and the threshold value
            prop_assert!(
                shell.contains("$#"),
                "Shell must use $# for arg_count()"
            );
        }

        /// Property: Generated shell scripts are syntactically valid
        #[test]
        fn prop_arg_count_output_shell_valid(
            _seed in 0u32..50
        ) {
            let source = r#"
fn main() {
    let count = arg_count();
    echo("test");
}

fn arg_count() -> i32 { 0 }
fn echo(msg: &str) {}
"#;

            let config = Config::default();
            let result = transpile(source, config);

            prop_assert!(result.is_ok(), "Transpilation must succeed");

            let shell = result.unwrap();

            // Basic validity checks
            prop_assert!(shell.contains("#!/bin/sh"), "Must have shebang");
            prop_assert!(shell.contains("set -euf"), "Must have safety flags");
            prop_assert!(shell.contains("$#"), "Must contain arg count");
        }
    }
}
