use super::*;
use crate::models::{ShellDialect, VerificationLevel};
use crate::validation::ValidationLevel;
use crate::cli::args::{CompileRuntime, ContainerFormatArg};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_build_command() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    let output_path = temp_dir.path().join("test.sh");

    // Write test Rust code
    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();

    // Test build command
    let config = Config {
        target: ShellDialect::Posix,
        verify: VerificationLevel::Basic,
        emit_proof: false,
        optimize: true,
        strict_mode: false,
        validation_level: None,
    };

    let result = build_command(&input_path, &output_path, config);

    assert!(result.is_ok());
    assert!(output_path.exists());

    // Check output contains expected shell code
    let output = fs::read_to_string(&output_path).unwrap();
    assert!(output.contains("#!/bin/sh"));
    assert!(output.contains("x=42"));
}

#[test]
fn test_check_command() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");

    // Valid Rust code
    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();
    let result = check_command(&input_path);
    assert!(result.is_ok());

    // Invalid Rust code
    fs::write(&input_path, "fn main() { unsafe { } }").unwrap();
    let result = check_command(&input_path);
    assert!(result.is_err());
}

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    let result = init_command(project_path, Some("test_project"));
    assert!(result.is_ok());

    // Check that files were created
    assert!(project_path.join("Cargo.toml").exists());
    assert!(project_path.join("src").exists());
    assert!(project_path.join("src/main.rs").exists());
    assert!(project_path.join(".rash.toml").exists());

    // Check Cargo.toml contains project name
    let cargo_toml = fs::read_to_string(project_path.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("name = \"test_project\""));
}

#[test]
fn test_compile_command_self_extracting() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    let output_path = temp_dir.path().join("test_self_extract.sh");
    
    // Create test input
    fs::write(&input_path, "fn main() { let msg = \"test\"; }").unwrap();
    
    let config = Config {
        target: ShellDialect::Posix,
        verify: VerificationLevel::Basic,
        emit_proof: false,
        optimize: true,
        validation_level: Some(ValidationLevel::Minimal),
        strict_mode: false,
    };
    
    // Test self-extracting script
    let result = handle_compile(
        &input_path,
        &output_path,
        CompileRuntime::Dash,
        true, // self_extracting
        false, // container
        ContainerFormatArg::Oci,
        &config
    );
    
    assert!(result.is_ok());
    assert!(output_path.exists());
    
    // Verify it's executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&output_path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o111, 0o111);
    }
}

#[test]
fn test_verify_command() {
    let temp_dir = TempDir::new().unwrap();
    let rust_path = temp_dir.path().join("test.rs");
    let shell_path = temp_dir.path().join("test.sh");

    // Write Rust code
    fs::write(&rust_path, "fn main() { let x = 42; }").unwrap();

    // First transpile to get the expected shell code
    let config = Config {
        target: ShellDialect::Posix,
        verify: VerificationLevel::Basic,
        emit_proof: false,
        optimize: true,
        strict_mode: false,
        validation_level: None,
    };

    let source = fs::read_to_string(&rust_path).unwrap();
    let shell_code = crate::transpile(&source, config).unwrap();
    fs::write(&shell_path, &shell_code).unwrap();

    // Now verify they match
    let result = verify_command(
        &rust_path,
        &shell_path,
        ShellDialect::Posix,
        VerificationLevel::Basic,
    );
    assert!(result.is_ok());
}

#[test]
fn test_generate_proof() {
    let temp_dir = TempDir::new().unwrap();
    let proof_path = temp_dir.path().join("test.proof");

    let config = Config {
        target: ShellDialect::Bash,
        verify: VerificationLevel::Strict,
        emit_proof: true,
        optimize: false,
        strict_mode: false,
        validation_level: None,
    };

    let result = generate_proof("fn main() {}", &proof_path, &config);
    assert!(result.is_ok());
    assert!(proof_path.exists());

    // Check proof content
    let proof = fs::read_to_string(&proof_path).unwrap();
    assert!(proof.contains("\"version\": \"1.0\""));
    assert!(proof.contains("\"verification_level\": \"Strict\""));
    assert!(proof.contains("\"target\": \"Bash\""));
}

#[test]
fn test_normalize_shell_script() {
    let script = r#"#!/bin/sh
# This is a comment
x=42    
    # Another comment
y=43

"#;

    let normalized = normalize_shell_script(script);
    assert_eq!(normalized, "x=42\ny=43");
}

#[test]
fn test_execute_command_integration() {
    use crate::cli::args::{Cli, Commands};

    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    let output_path = temp_dir.path().join("test.sh");

    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();

    let cli = Cli {
        command: Commands::Build {
            input: input_path.clone(),
            output: output_path.clone(),
            emit_proof: false,
            no_optimize: false,
        },
        verify: VerificationLevel::Basic,
        target: ShellDialect::Posix,
        validation: crate::validation::ValidationLevel::Minimal,
        strict: false,
        verbose: false,
    };

    let result = execute_command(cli);
    assert!(result.is_ok());
    assert!(output_path.exists());
}

#[test]
fn test_error_handling() {
    // Test with non-existent file
    let result = check_command(&PathBuf::from("/nonexistent/file.rs"));
    assert!(result.is_err());

    // Test build with invalid output path
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    fs::write(&input_path, "fn main() {}").unwrap();

    let config = Config::default();
    let result = build_command(
        &input_path,
        &PathBuf::from("/nonexistent/dir/output.sh"),
        config,
    );
    assert!(result.is_err());
}

#[test]
fn test_inspect_command_echo_example() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test basic echo example
    let result = inspect_command("echo-example", InspectionFormat::Markdown, None, false);
    assert!(result.is_ok());
}

#[test]
fn test_inspect_command_bootstrap_example() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test bootstrap example
    let result = inspect_command("bootstrap-example", InspectionFormat::Json, None, false);
    assert!(result.is_ok());
}

#[test]
fn test_inspect_command_json_ast() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test with JSON AST input
    let json_ast = r#"{"ExecuteCommand": {"command_name": "echo", "args": ["test"]}}"#;
    let result = inspect_command(json_ast, InspectionFormat::Markdown, None, false);
    assert!(result.is_ok());
}

#[test]
fn test_inspect_command_invalid_input() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test with invalid input
    let result = inspect_command("invalid-example", InspectionFormat::Markdown, None, false);
    assert!(result.is_err());
}

#[test]
fn test_inspect_command_html_format() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test HTML format
    let result = inspect_command("echo-example", InspectionFormat::Html, None, false);
    assert!(result.is_ok());
}

#[test]
fn test_inspect_command_with_output_file() {
    use super::inspect_command;
    use super::InspectionFormat;
    use tempfile::NamedTempFile;

    // Test with output file
    let temp_file = NamedTempFile::new().unwrap();
    let result = inspect_command(
        "echo-example",
        InspectionFormat::Markdown,
        Some(temp_file.path()),
        false,
    );
    assert!(result.is_ok());

    // Verify file was written
    let content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(!content.is_empty());
    assert!(content.contains("Formal Verification Report"));
}

#[test]
fn test_inspect_command_invalid_json() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test with malformed JSON
    let invalid_json = r#"{"invalid": json}"#;
    let result = inspect_command(invalid_json, InspectionFormat::Json, None, false);
    assert!(result.is_err());
}

#[test]
fn test_inspect_command_all_formats() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test all supported formats
    for format in [
        InspectionFormat::Markdown,
        InspectionFormat::Json,
        InspectionFormat::Html,
    ] {
        let result = inspect_command("echo-example", format.clone(), None, false);
        assert!(result.is_ok(), "Failed with format: {format:?}");
    }
}
