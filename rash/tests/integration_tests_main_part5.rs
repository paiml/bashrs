#[test]
fn test_string_length_baseline() {
    let source = r#"
fn main() {
    length_of("hello");
}

fn length_of(s: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

    // Should generate heredoc syntax
    assert!(
        shell.contains("<<") && shell.contains("EOF"),
        "Should convert multi-line string to heredoc"
    );
}

include!("integration_tests_main_part6.rs");
