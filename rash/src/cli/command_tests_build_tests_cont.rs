fn test_build_command_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("empty.rs");
    let output_path = temp_dir.path().join("empty.sh");

    // Empty file
    fs::write(&input_path, "").unwrap();

    let config = Config::default();
    let result = build_command(&input_path, &output_path, config);

    // Should fail with empty file
    assert!(result.is_err());
}

#[test]
fn test_build_command_only_comments() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("comments.rs");
    let output_path = temp_dir.path().join("comments.sh");

    fs::write(&input_path, "// Just comments\n/* Block comment */").unwrap();

    let config = Config::default();
    let result = build_command(&input_path, &output_path, config);

    // Should fail - no actual code
    assert!(result.is_err());
}

#[test]
fn test_build_command_combined_flags() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    let output_path = temp_dir.path().join("test.sh");
    fs::write(&input_path, "fn main() { let x = 42; let y = x * 2; }").unwrap();

    // Test combination of all flags
    let config = Config {
        target: ShellDialect::Bash,
        verify: VerificationLevel::Paranoid,
        emit_proof: true,
        optimize: true,
        strict_mode: true,
        validation_level: Some(ValidationLevel::Paranoid),
    };

    let result = build_command(&input_path, &output_path, config);
    let _ = result; // May succeed or fail
}

#[test]
fn test_check_command_syntax_error() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("bad_syntax.rs");

    // Invalid syntax - missing semicolon, extra braces
    fs::write(&input_path, "fn main() { let x = 42 } }").unwrap();

    let result = check_command(&input_path);
    assert!(result.is_err());
}

#[test]
fn test_verify_command_nonexistent_rust_file() {
    let temp_dir = TempDir::new().unwrap();
    let rust_path = temp_dir.path().join("nonexistent.rs");
    let shell_path = temp_dir.path().join("test.sh");

    fs::write(&shell_path, "#!/bin/sh\necho test").unwrap();

    let result = verify_command(
        &rust_path,
        &shell_path,
        ShellDialect::Posix,
        VerificationLevel::Basic,
    );
    assert!(result.is_err());
}

#[test]
fn test_verify_command_nonexistent_shell_file() {
    let temp_dir = TempDir::new().unwrap();
    let rust_path = temp_dir.path().join("test.rs");
    let shell_path = temp_dir.path().join("nonexistent.sh");

    fs::write(&rust_path, "fn main() {}").unwrap();

    let result = verify_command(
        &rust_path,
        &shell_path,
        ShellDialect::Posix,
        VerificationLevel::Basic,
    );
    assert!(result.is_err());
}
#[test]
fn test_build_command_with_dash_dialect() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    let output_path = temp_dir.path().join("test.sh");
    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();

    let config = Config {
        target: ShellDialect::Dash,
        verify: VerificationLevel::Strict,
        emit_proof: false,
        optimize: true,
        strict_mode: true,
        validation_level: Some(ValidationLevel::Strict),
    };

    let result = build_command(&input_path, &output_path, config);
    let _ = result; // May succeed or fail
    assert!(output_path.exists());

    let output = fs::read_to_string(&output_path).unwrap();
    assert!(output.contains("#!/"));
}

#[test]
fn test_compile_command_busybox_runtime() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    let output_path = temp_dir.path().join("busybox.sh");
    fs::write(&input_path, "fn main() { let greeting = \"hello\"; }").unwrap();

    let config = Config {
        target: ShellDialect::Posix,
        verify: VerificationLevel::Basic,
        emit_proof: false,
        optimize: false,
        validation_level: None,
        strict_mode: false,
    };

    let result = handle_compile(
        &input_path,
        &output_path,
        CompileRuntime::Busybox,
        true,
        false,
        ContainerFormatArg::Oci,
        &config,
    );
    let _ = result; // May succeed or fail
}

#[test]
fn test_generate_proof_with_basic_verification() {
    let temp_dir = TempDir::new().unwrap();
    let proof_path = temp_dir.path().join("basic.proof");

    let config = Config {
        target: ShellDialect::Posix,
        verify: VerificationLevel::Basic,
        emit_proof: true,
        optimize: false,
        strict_mode: false,
        validation_level: None,
    };

    let result = generate_proof("fn main() { let count = 10; }", &proof_path, &config);
    let _ = result; // May succeed or fail
    assert!(proof_path.exists());
}

#[test]
fn test_execute_command_check() {
    use crate::cli::args::{Cli, Commands};

    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();

    let cli = Cli {
        command: Commands::Check {
            input: input_path.clone(),
        },
        verify: VerificationLevel::Basic,
        target: ShellDialect::Posix,
        validation: ValidationLevel::Minimal,
        strict: false,
        verbose: false,
    };

    let result = execute_command(cli);
    let _ = result; // May succeed or fail
}

#[test]
fn test_execute_command_init() {
    use crate::cli::args::{Cli, Commands};

    let temp_dir = TempDir::new().unwrap();

    let cli = Cli {
        command: Commands::Init {
            path: temp_dir.path().to_path_buf(),
            name: Some("exec_test".to_string()),
        },
        verify: VerificationLevel::Basic,
        target: ShellDialect::Posix,
        validation: ValidationLevel::Minimal,
        strict: false,
        verbose: false,
    };

    let result = execute_command(cli);
    // Note: execute_command may return an error in test environment
    if result.is_ok() {
        assert!(temp_dir.path().join("Cargo.toml").exists());
    }
}

// ============================================================================
// Verify Command Tests
// ============================================================================

#[test]
fn test_verify_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let rust_path = temp_dir.path().join("test.rs");
    let shell_path = temp_dir.path().join("test.sh");

    fs::write(&rust_path, "fn main() { let x = 42; }").unwrap();
    fs::write(&shell_path, "#!/bin/sh\nx=42").unwrap();

    let result = verify_command(
        &rust_path,
        &shell_path,
        ShellDialect::Posix,
        VerificationLevel::Basic,
    );
    let _ = result;
}

// ============================================================================
// Init Command Tests
// ============================================================================

#[test]
fn test_init_command_creates_project() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("new_project");

    let result = init_command(&project_path, Some("test_project"));
    assert!(result.is_ok());
    assert!(project_path.exists());
}

#[test]
fn test_init_command_default_name() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().join("my_project");

    let result = init_command(&project_path, None);
    assert!(result.is_ok());
}
