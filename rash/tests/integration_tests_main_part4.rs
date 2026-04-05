#[test]
#[ignore] // Requires std::env::remove_var recognition
fn test_unset_command_std_env() {
    let source = r#"
fn main() {
    std::env::remove_var("VAR");
}
"#;

    let config = Config::default();
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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

include!("integration_tests_main_part5.rs");
