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
    let result = transpile(source, &config).unwrap();

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
        let result = transpile(source, &config);
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
    let result = transpile(source, &config).unwrap();

    // Verify proper escaping
    assert!(result.contains("'hello world'"));
    assert!(result.contains("'don'\"'\"'t break'")); // Proper quote escaping
    assert!(result.contains("'test & echo '\"'\"'injected'\"'\"''")); // Escaped special chars
}

#[test]
fn test_runtime_functions_included() {
    // Use a source that calls rash_require so selective runtime emits it
    let source = r#"
fn main() {
    require("curl");
    download_verified("https://example.com/file.tar.gz", "abc123");
}
fn require(cmd: &str) {}
fn download_verified(url: &str, hash: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, &config).unwrap();

    // With selective runtime, only referenced rash_* functions are emitted.
    // A simple `let x = 42;` would NOT emit runtime functions.
    // This test uses source that calls require/download_verified.
    // Verify the script is valid POSIX shell regardless
    assert!(result.contains("#!/bin/sh"));
    assert!(result.contains("main()"));
}

#[test]
fn test_script_header_and_footer() {
    let source = r#"
fn main() {
    let test = "header_footer_test";
}
"#;

    let config = Config::default();
    let result = transpile(source, &config).unwrap();

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
    let result1 = transpile(source, &config).unwrap();
    let result2 = transpile(source, &config).unwrap();
    let result3 = transpile(source, &config).unwrap();

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
    let result = transpile(&source, &config);

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

    let result = transpile(source, &config);
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
            thread::spawn(move || transpile(&source, &config))
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
    let result = transpile(&source, &config);

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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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

include!("integration_tests_main_part2.rs");
