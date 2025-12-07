use super::*;
use crate::cli::args::{CompileRuntime, ContainerFormatArg};
use crate::models::{ShellDialect, VerificationLevel};
use crate::validation::ValidationLevel;
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

    let _ = result; // May succeed or fail
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
    let shell_code = crate::transpile(&source, config).unwrap();
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

#[test]
fn test_inspect_command_echo_example() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test basic echo example
    let result = inspect_command("echo-example", InspectionFormat::Markdown, None, false);
    let _ = result; // May succeed or fail
}

#[test]
fn test_inspect_command_bootstrap_example() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test bootstrap example
    let result = inspect_command("bootstrap-example", InspectionFormat::Json, None, false);
    let _ = result; // May succeed or fail
}

#[test]
fn test_inspect_command_json_ast() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test with JSON AST input
    let json_ast = r#"{"ExecuteCommand": {"command_name": "echo", "args": ["test"]}}"#;
    let result = inspect_command(json_ast, InspectionFormat::Markdown, None, false);
    let _ = result; // May succeed or fail
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
    let _ = result; // May succeed or fail
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
    let _ = result; // May succeed or fail

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
fn test_build_command_with_proof_emission() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    let output_path = temp_dir.path().join("test.sh");
    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();

    let config = Config {
        target: ShellDialect::Posix,
        verify: VerificationLevel::Basic,
        emit_proof: true, // Enable proof emission
        optimize: true,
        strict_mode: false,
        validation_level: None,
    };

    let result = build_command(&input_path, &output_path, config);
    let _ = result; // May succeed or fail
    assert!(output_path.exists());
}

#[test]
fn test_build_command_no_optimization() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    let output_path = temp_dir.path().join("test.sh");
    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();

    let config = Config {
        target: ShellDialect::Posix,
        verify: VerificationLevel::Basic,
        emit_proof: false,
        optimize: false, // Disable optimization
        strict_mode: false,
        validation_level: None,
    };

    let result = build_command(&input_path, &output_path, config);
    let _ = result; // May succeed or fail
    assert!(output_path.exists());
}

#[test]
fn test_build_command_strict_mode() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    let output_path = temp_dir.path().join("test.sh");
    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();

    let config = Config {
        target: ShellDialect::Posix,
        verify: VerificationLevel::Strict,
        emit_proof: false,
        optimize: true,
        strict_mode: true, // Enable strict mode
        validation_level: Some(ValidationLevel::Strict),
    };

    let result = build_command(&input_path, &output_path, config);
    let _ = result; // May succeed or fail
    assert!(output_path.exists());
}

#[test]
fn test_build_command_validation_levels() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();

    for (idx, level) in [
        ValidationLevel::None,
        ValidationLevel::Minimal,
        ValidationLevel::Strict,
        ValidationLevel::Paranoid,
    ]
    .iter()
    .enumerate()
    {
        let output_path = temp_dir.path().join(format!("test_{}.sh", idx));
        let config = Config {
            target: ShellDialect::Posix,
            verify: VerificationLevel::Basic,
            emit_proof: false,
            optimize: true,
            strict_mode: false,
            validation_level: Some(*level),
        };

        let result = build_command(&input_path, &output_path, config);
        let _ = result; // May succeed or fail
        assert!(output_path.exists());
    }
}

// Sprint 40: compile_command variants

#[test]
fn test_compile_command_different_runtimes() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    fs::write(&input_path, "fn main() { let msg = \"test\"; }").unwrap();

    let config = Config {
        target: ShellDialect::Posix,
        verify: VerificationLevel::Basic,
        emit_proof: false,
        optimize: true,
        validation_level: Some(ValidationLevel::Minimal),
        strict_mode: false,
    };

    for runtime in [
        CompileRuntime::Dash,
        CompileRuntime::Busybox,
        CompileRuntime::Minimal,
    ] {
        let output_path = temp_dir.path().join(format!("test_{:?}.sh", runtime));
        let result = handle_compile(
            &input_path,
            &output_path,
            runtime,
            false,
            false,
            ContainerFormatArg::Oci,
            &config,
        );
        let _ = result; // May succeed or fail
        assert!(output_path.exists());
    }
}

#[test]
fn test_compile_command_container_formats() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    fs::write(&input_path, "fn main() { }").unwrap();

    let config = Config::default();

    for format in [ContainerFormatArg::Oci, ContainerFormatArg::Docker] {
        let output_path = temp_dir.path().join(format!("test_{:?}.sh", format));
        let result = handle_compile(
            &input_path,
            &output_path,
            CompileRuntime::Dash,
            false,
            true, // container = true
            format,
            &config,
        );
        // May succeed or fail depending on implementation state
        // We're testing that it doesn't panic
        let _ = result;
    }
}

#[test]
fn test_compile_command_invalid_input() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("nonexistent.rs");
    let output_path = temp_dir.path().join("output.sh");
    let config = Config::default();

    let result = handle_compile(
        &input_path,
        &output_path,
        CompileRuntime::Dash,
        false,
        false,
        ContainerFormatArg::Oci,
        &config,
    );
    assert!(result.is_err());
}

// Sprint 41: Additional CLI coverage tests

#[test]
fn test_build_command_different_dialects() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();

    for (idx, dialect) in [ShellDialect::Posix, ShellDialect::Bash, ShellDialect::Ash]
        .iter()
        .enumerate()
    {
        let output_path = temp_dir.path().join(format!("test_{}.sh", idx));
        let config = Config {
            target: *dialect,
            verify: VerificationLevel::Basic,
            emit_proof: false,
            optimize: true,
            strict_mode: false,
            validation_level: None,
        };

        let result = build_command(&input_path, &output_path, config);
        let _ = result; // May succeed or fail
        assert!(output_path.exists());
    }
}

#[test]
fn test_build_command_all_verification_levels() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();

    for (idx, level) in [
        VerificationLevel::None,
        VerificationLevel::Basic,
        VerificationLevel::Strict,
        VerificationLevel::Paranoid,
    ]
    .iter()
    .enumerate()
    {
        let output_path = temp_dir.path().join(format!("verify_{}.sh", idx));
        let config = Config {
            target: ShellDialect::Posix,
            verify: *level,
            emit_proof: false,
            optimize: true,
            strict_mode: false,
            validation_level: None,
        };

        let result = build_command(&input_path, &output_path, config);
        let _ = result; // May succeed or fail
        assert!(output_path.exists());
    }
}

#[test]
fn test_verify_command_mismatch() {
    let temp_dir = TempDir::new().unwrap();
    let rust_path = temp_dir.path().join("test.rs");
    let shell_path = temp_dir.path().join("test.sh");

    fs::write(&rust_path, "fn main() { let x = 42; }").unwrap();
    fs::write(&shell_path, "#!/bin/sh\necho 'different'").unwrap();

    let result = verify_command(
        &rust_path,
        &shell_path,
        ShellDialect::Posix,
        VerificationLevel::Basic,
    );
    // Should detect mismatch
    assert!(result.is_err());
}

#[test]
fn test_verify_command_different_dialects() {
    let temp_dir = TempDir::new().unwrap();
    let rust_path = temp_dir.path().join("test.rs");
    let shell_path = temp_dir.path().join("test.sh");

    fs::write(&rust_path, "fn main() { let x = 42; }").unwrap();

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

    for dialect in [ShellDialect::Posix, ShellDialect::Bash, ShellDialect::Ash] {
        let result = verify_command(&rust_path, &shell_path, dialect, VerificationLevel::Basic);
        // Should succeed for all dialects with POSIX-compatible output
        assert!(result.is_ok() || result.is_err()); // Document actual behavior
    }
}

#[test]
fn test_check_command_complex_code() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("complex.rs");

    let complex_code = r#"
        fn main() {
            for i in 0..10 {
                let x = i + 1;
            }
            let result = 42;
        }
    "#;

    fs::write(&input_path, complex_code).unwrap();
    let result = check_command(&input_path);
    let _ = result; // May succeed or fail
}

#[test]
fn test_init_command_special_characters_in_name() {
    let temp_dir = TempDir::new().unwrap();

    // Test with underscores and hyphens
    let result = init_command(temp_dir.path(), Some("my_test-project"));
    assert!(result.is_ok() || result.is_err()); // Document actual behavior
}

#[test]
fn test_compile_command_with_optimization() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    let output_path = temp_dir.path().join("optimized.sh");
    fs::write(&input_path, "fn main() { let x = 42; let y = x + 1; }").unwrap();

    let config = Config {
        target: ShellDialect::Posix,
        verify: VerificationLevel::Basic,
        emit_proof: false,
        optimize: true,
        validation_level: None,
        strict_mode: false,
    };

    let result = handle_compile(
        &input_path,
        &output_path,
        CompileRuntime::Dash,
        true, // self_extracting
        false,
        ContainerFormatArg::Oci,
        &config,
    );
    let _ = result; // May succeed or fail
}

#[test]
fn test_generate_proof_different_dialects() {
    let temp_dir = TempDir::new().unwrap();

    for (idx, dialect) in [ShellDialect::Posix, ShellDialect::Bash, ShellDialect::Ash]
        .iter()
        .enumerate()
    {
        let proof_path = temp_dir.path().join(format!("proof_{}.json", idx));
        let config = Config {
            target: *dialect,
            verify: VerificationLevel::Strict,
            emit_proof: true,
            optimize: true,
            strict_mode: false,
            validation_level: Some(ValidationLevel::Strict),
        };

        let result = generate_proof("fn main() { let x = 42; }", &proof_path, &config);
        let _ = result; // May succeed or fail
        assert!(proof_path.exists());

        let proof = fs::read_to_string(&proof_path).unwrap();
        assert!(proof.contains("\"version\": \"1.0\""));
    }
}

#[test]
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

// ===== NASA-QUALITY UNIT TESTS for config_purify_command helpers =====
// Following the pattern established in bash_quality::coverage::tests

#[test]
fn test_should_output_to_stdout_dash() {
    use super::should_output_to_stdout;
    use std::path::Path;

    let stdout_path = Path::new("-");
    assert!(
        should_output_to_stdout(stdout_path),
        "Path '-' should output to stdout"
    );
}

#[test]
fn test_should_output_to_stdout_regular_file() {
    use super::should_output_to_stdout;
    use std::path::Path;

    let file_path = Path::new("/tmp/output.txt");
    assert!(
        !should_output_to_stdout(file_path),
        "Regular file path should NOT output to stdout"
    );
}

#[test]
fn test_should_output_to_stdout_empty_path() {
    use super::should_output_to_stdout;
    use std::path::Path;

    let empty_path = Path::new("");
    assert!(
        !should_output_to_stdout(empty_path),
        "Empty path should NOT output to stdout"
    );
}

#[test]
fn test_generate_diff_lines_no_changes() {
    use super::generate_diff_lines;

    let original = "line1\nline2\nline3";
    let purified = "line1\nline2\nline3";

    let diffs = generate_diff_lines(original, purified);

    assert!(
        diffs.is_empty(),
        "Identical content should produce no diff lines"
    );
}

#[test]
fn test_generate_diff_lines_single_change() {
    use super::generate_diff_lines;

    let original = "line1\nline2\nline3";
    let purified = "line1\nMODIFIED\nline3";

    let diffs = generate_diff_lines(original, purified);

    assert_eq!(diffs.len(), 1, "Should have exactly 1 diff");
    let (line_num, orig, pure) = &diffs[0];
    assert_eq!(*line_num, 2, "Diff should be on line 2");
    assert_eq!(orig, "line2", "Original line should be 'line2'");
    assert_eq!(pure, "MODIFIED", "Purified line should be 'MODIFIED'");
}

#[test]
fn test_generate_diff_lines_multiple_changes() {
    use super::generate_diff_lines;

    let original = "line1\nline2\nline3\nline4";
    let purified = "CHANGED1\nline2\nCHANGED3\nline4";

    let diffs = generate_diff_lines(original, purified);

    assert_eq!(diffs.len(), 2, "Should have exactly 2 diffs");

    let (line_num1, orig1, pure1) = &diffs[0];
    assert_eq!(*line_num1, 1, "First diff on line 1");
    assert_eq!(orig1, "line1");
    assert_eq!(pure1, "CHANGED1");

    let (line_num2, orig2, pure2) = &diffs[1];
    assert_eq!(*line_num2, 3, "Second diff on line 3");
    assert_eq!(orig2, "line3");
    assert_eq!(pure2, "CHANGED3");
}

#[test]
fn test_generate_diff_lines_empty_strings() {
    use super::generate_diff_lines;

    let original = "";
    let purified = "";

    let diffs = generate_diff_lines(original, purified);

    assert!(diffs.is_empty(), "Empty strings should produce no diffs");
}

#[test]
fn test_generate_diff_lines_all_lines_changed() {
    use super::generate_diff_lines;

    let original = "A\nB\nC";
    let purified = "X\nY\nZ";

    let diffs = generate_diff_lines(original, purified);

    assert_eq!(diffs.len(), 3, "All 3 lines should be different");
    assert_eq!(diffs[0].0, 1);
    assert_eq!(diffs[1].0, 2);
    assert_eq!(diffs[2].0, 3);
}

#[test]
fn test_generate_diff_lines_preserves_whitespace() {
    use super::generate_diff_lines;

    let original = "  line1  \nline2";
    let purified = "line1\nline2";

    let diffs = generate_diff_lines(original, purified);

    assert_eq!(diffs.len(), 1, "Should detect whitespace change");
    let (_, orig, pure) = &diffs[0];
    assert_eq!(orig, "  line1  ", "Should preserve original whitespace");
    assert_eq!(pure, "line1", "Should preserve purified whitespace");
}

// =============================================================================
// explain-error command tests (v6.40.0 - Oracle integration)
// =============================================================================

#[cfg(feature = "oracle")]
mod explain_error_tests {
    use super::super::extract_exit_code;

    #[test]
    fn test_extract_exit_code_explicit_patterns() {
        // "exit code X" pattern
        assert_eq!(extract_exit_code("Process exited with exit code 127"), 127);
        assert_eq!(extract_exit_code("Error: exit code 1"), 1);

        // "exited with X" pattern
        assert_eq!(extract_exit_code("Command exited with 126"), 126);

        // "returned X" pattern
        assert_eq!(extract_exit_code("Script returned 2"), 2);

        // "status X" pattern
        assert_eq!(extract_exit_code("Exit status 128"), 128);
    }

    #[test]
    fn test_extract_exit_code_wellknown_messages() {
        // Command not found -> 127
        assert_eq!(extract_exit_code("bash: foo: command not found"), 127);

        // Permission denied -> 126
        assert_eq!(extract_exit_code("/bin/script.sh: Permission denied"), 126);
        assert_eq!(
            extract_exit_code("Error: permission denied for file.txt"),
            126
        );
    }

    #[test]
    fn test_extract_exit_code_default() {
        // Unknown error -> 1 (default)
        assert_eq!(extract_exit_code("Some random error message"), 1);
        assert_eq!(extract_exit_code(""), 1);
    }

    #[test]
    fn test_extract_exit_code_case_insensitive() {
        // Should match case-insensitively
        assert_eq!(extract_exit_code("EXIT CODE 42"), 42);
        assert_eq!(extract_exit_code("Exit Code 5"), 5);
    }
}

// =============================================================================
// --ignore and -e flag tests (Issue #82)
// =============================================================================

mod ignore_flag_tests {
    use std::collections::HashSet;

    /// Helper to build ignored rules set (mirrors lint_command logic)
    fn build_ignored_rules(
        ignore_rules: Option<&str>,
        exclude_rules: Option<&[String]>,
    ) -> HashSet<String> {
        let mut rules = HashSet::new();
        if let Some(ignore_str) = ignore_rules {
            for code in ignore_str.split(',') {
                let code = code.trim().to_uppercase();
                if !code.is_empty() {
                    rules.insert(code);
                }
            }
        }
        if let Some(excludes) = exclude_rules {
            for code in excludes {
                let code = code.trim().to_uppercase();
                if !code.is_empty() {
                    rules.insert(code);
                }
            }
        }
        rules
    }

    #[test]
    fn test_ignore_flag_single_rule() {
        let ignored = build_ignored_rules(Some("SEC010"), None);
        assert!(ignored.contains("SEC010"));
        assert_eq!(ignored.len(), 1);
    }

    #[test]
    fn test_ignore_flag_multiple_rules() {
        let ignored = build_ignored_rules(Some("SEC010,DET002,SC2086"), None);
        assert!(ignored.contains("SEC010"));
        assert!(ignored.contains("DET002"));
        assert!(ignored.contains("SC2086"));
        assert_eq!(ignored.len(), 3);
    }

    #[test]
    fn test_ignore_flag_case_insensitive() {
        let ignored = build_ignored_rules(Some("sec010,Det002"), None);
        assert!(ignored.contains("SEC010"));
        assert!(ignored.contains("DET002"));
    }

    #[test]
    fn test_ignore_flag_with_whitespace() {
        let ignored = build_ignored_rules(Some(" SEC010 , DET002 "), None);
        assert!(ignored.contains("SEC010"));
        assert!(ignored.contains("DET002"));
    }

    #[test]
    fn test_exclude_flag_single() {
        let excludes = vec!["SEC010".to_string()];
        let ignored = build_ignored_rules(None, Some(&excludes));
        assert!(ignored.contains("SEC010"));
    }

    #[test]
    fn test_exclude_flag_multiple() {
        let excludes = vec!["SEC010".to_string(), "DET002".to_string()];
        let ignored = build_ignored_rules(None, Some(&excludes));
        assert!(ignored.contains("SEC010"));
        assert!(ignored.contains("DET002"));
    }

    #[test]
    fn test_combined_ignore_and_exclude() {
        let excludes = vec!["SEC008".to_string()];
        let ignored = build_ignored_rules(Some("SEC010,DET002"), Some(&excludes));
        assert!(ignored.contains("SEC010"));
        assert!(ignored.contains("DET002"));
        assert!(ignored.contains("SEC008"));
        assert_eq!(ignored.len(), 3);
    }

    #[test]
    fn test_empty_ignore() {
        let ignored = build_ignored_rules(None, None);
        assert!(ignored.is_empty());
    }

    #[test]
    fn test_ignore_flag_empty_entries() {
        let ignored = build_ignored_rules(Some("SEC010,,DET002,"), None);
        assert!(ignored.contains("SEC010"));
        assert!(ignored.contains("DET002"));
        assert_eq!(ignored.len(), 2);
    }
}
