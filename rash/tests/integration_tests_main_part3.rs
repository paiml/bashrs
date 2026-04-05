
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
    let shell = transpile(source, &config).unwrap();

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

include!("integration_tests_main_part3.rs");
#[test]
fn test_output_redirection_baseline() {
    let source = r#"
fn main() {
    write_file("output.txt", "Hello World");
}

fn write_file(path: &str, content: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

    assert!(result.is_ok(), "Should transpile unset: {:?}", result.err());

    let shell = result.unwrap();
    eprintln!("Generated shell for unset command:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("unset_var") || shell.contains("unset"),
        "Should transpile unset_var function"
    );
}

include!("integration_tests_main_part4.rs");
