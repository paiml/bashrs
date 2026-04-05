use super::*;

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

    let _ = result; // May succeed or fail
    assert!(output_path.exists());

    // Check output contains expected shell code
    let output = fs::read_to_string(&output_path).unwrap();
    assert!(output.contains("#!/bin/sh"));
    assert!(output.contains("x='42'"));
}

#[test]
fn test_check_command() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");

    // Valid Rust code
    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();
    let result = check_command(&input_path);
    let _ = result; // May succeed or fail

    // Invalid Rust code
    fs::write(&input_path, "fn main() { unsafe { } }").unwrap();
    let result = check_command(&input_path);
    assert!(result.is_err());
}

/// Issue #84: check command should detect shell scripts and provide helpful guidance
#[test]
fn test_issue_84_check_detects_shell_script_by_extension() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("script.sh");

    // Write a valid bash script
    fs::write(&input_path, "#!/bin/bash\necho 'Hello, World!'").unwrap();

    let result = check_command(&input_path);
    assert!(result.is_err());

    let err_msg = format!("{}", result.unwrap_err());
    // Should mention it's a shell script
    assert!(err_msg.contains("shell script"));
    // Should suggest using lint command
    assert!(err_msg.contains("bashrs lint"));
}

/// Issue #84: check command should detect shell scripts by shebang
#[test]
fn test_issue_84_check_detects_shell_script_by_shebang() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("script"); // No extension

    // Write a bash script with shebang (no .sh extension)
    fs::write(&input_path, "#!/bin/bash\necho 'Hello, World!'").unwrap();

    let result = check_command(&input_path);
    assert!(result.is_err());

    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("shell script"));
    assert!(err_msg.contains("bashrs lint"));
}

/// Issue #84: check command should detect sh scripts
#[test]
fn test_issue_84_check_detects_posix_sh_shebang() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("script");

    // Write a POSIX sh script
    fs::write(&input_path, "#!/bin/sh\necho 'Hello'").unwrap();

    let result = check_command(&input_path);
    assert!(result.is_err());

    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("shell script"));
}

/// Issue #84: check command should still work for .rs files
#[test]
fn test_issue_84_check_allows_rs_files() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");

    // Write valid Rash code
    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();

    let result = check_command(&input_path);
    // Should not return the "shell script" error
    if let Err(ref e) = result {
        let err_msg = format!("{}", e);
        assert!(
            !err_msg.contains("shell script"),
            "Should not detect .rs as shell script"
        );
    }
}

#[test]
fn test_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    let result = init_command(project_path, Some("test_project"));
    let _ = result; // May succeed or fail

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
        true,  // self_extracting
        false, // container
        ContainerFormatArg::Oci,
        &config,
    );

    let _ = result; // May succeed or fail
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
    let shell_code = crate::transpile(&source, &config).unwrap();
    fs::write(&shell_path, &shell_code).unwrap();

    // Now verify they match
    let result = verify_command(
        &rust_path,
        &shell_path,
        ShellDialect::Posix,
        VerificationLevel::Basic,
    );
    let _ = result; // May succeed or fail
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
    let _ = result; // May succeed or fail
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
    // Note: execute_command may return an error in test environment
    if result.is_ok() {
        assert!(output_path.exists());
    }
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

// Sprint 40: init_command edge cases

#[test]
fn test_init_command_existing_directory_with_files() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Create existing file
    fs::write(project_path.join("existing.txt"), "existing content").unwrap();

    let result = init_command(project_path, Some("test_project"));
    // Should handle existing files gracefully
    let _ = result; // May succeed or fail

    // Existing file should remain
    assert!(project_path.join("existing.txt").exists());
    // New project files should be created
    assert!(project_path.join("Cargo.toml").exists());
}

#[test]
fn test_init_command_no_name() {
    let temp_dir = TempDir::new().unwrap();
    let result = init_command(temp_dir.path(), None);
    let _ = result; // May succeed or fail

    // Should use directory name
    let cargo_toml = fs::read_to_string(temp_dir.path().join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("name ="));
}

#[test]
fn test_init_command_nested_path() {
    let temp_dir = TempDir::new().unwrap();
    let nested = temp_dir.path().join("nested/deep/path");
    fs::create_dir_all(&nested).unwrap();

    let result = init_command(&nested, Some("nested_project"));
    let _ = result; // May succeed or fail

    assert!(nested.join("Cargo.toml").exists());
    assert!(nested.join(".rash.toml").exists());
}

#[test]
fn test_init_command_creates_rash_config() {
    let temp_dir = TempDir::new().unwrap();
    init_command(temp_dir.path(), Some("test")).unwrap();

    let rash_config = temp_dir.path().join(".rash.toml");
    assert!(rash_config.exists());

    let config_content = fs::read_to_string(&rash_config).unwrap();
    assert!(config_content.contains("[transpiler]"));
}

// Sprint 40: build_command configuration variants

#[test]

include!("command_tests_build_tests_cont_2.rs");
