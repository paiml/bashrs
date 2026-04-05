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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
    let shell = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

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
    let result = transpile(source, &config);

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
    let result = transpile(source, &config);

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
    let result = transpile(source, &config);

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
    let shell_script = transpile(source, &config).unwrap();

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
    let result = transpile(source, &config);

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
    let result = transpile(source, &config);

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
    let result = transpile(source, &config);

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
    let result = transpile(source, &config);

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
    let result = transpile(source, &config);

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
    let shell = transpile(source, &config).unwrap();

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
