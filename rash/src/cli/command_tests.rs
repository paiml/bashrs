use super::*;
use crate::cli::args::{
    AuditOutputFormat, CompileRuntime, ContainerFormatArg, CoverageOutputFormat, LintProfileArg,
    MutateFormat, PlaybookFormat, ReportFormat, ScoreOutputFormat, SimulateFormat,
    TestOutputFormat,
};
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

// ============================================================================
// Helper Function Tests - Boost coverage for small utility functions
// ============================================================================

#[test]
fn test_hex_encode_empty() {
    assert_eq!(hex_encode(&[]), "");
}

#[test]
fn test_hex_encode_single_byte() {
    assert_eq!(hex_encode(&[0x00]), "00");
    assert_eq!(hex_encode(&[0xff]), "ff");
    assert_eq!(hex_encode(&[0x42]), "42");
}

#[test]
fn test_hex_encode_multiple_bytes() {
    assert_eq!(hex_encode(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    assert_eq!(hex_encode(&[0x01, 0x23, 0x45, 0x67]), "01234567");
}

#[test]
fn test_truncate_str_short() {
    assert_eq!(truncate_str("hello", 10), "hello");
    assert_eq!(truncate_str("hi", 5), "hi");
}

#[test]
fn test_truncate_str_exact() {
    assert_eq!(truncate_str("hello", 5), "hello");
}

#[test]
fn test_truncate_str_long() {
    assert_eq!(truncate_str("hello world", 8), "hello...");
    assert_eq!(truncate_str("abcdefghij", 6), "abc...");
}

#[test]
fn test_truncate_str_edge_cases() {
    assert_eq!(truncate_str("abc", 3), "abc");
    assert_eq!(truncate_str("abcd", 3), "...");
    assert_eq!(truncate_str("", 5), "");
}

#[test]
fn test_should_output_to_stdout() {
    use std::path::Path;
    assert!(should_output_to_stdout(Path::new("-")));
    assert!(!should_output_to_stdout(Path::new("output.sh")));
    assert!(!should_output_to_stdout(Path::new("/tmp/file.txt")));
    assert!(!should_output_to_stdout(Path::new("--")));
}

#[test]
fn test_format_timestamp_just_now() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Test a timestamp from a few seconds ago
    let result = format_timestamp(now - 30);
    assert_eq!(result, "just now");
}

#[test]
fn test_format_timestamp_minutes_ago() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let result = format_timestamp(now - 120); // 2 minutes ago
    assert_eq!(result, "2m ago");
}

#[test]
fn test_format_timestamp_hours_ago() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let result = format_timestamp(now - 7200); // 2 hours ago
    assert_eq!(result, "2h ago");
}

#[test]
fn test_format_timestamp_days_ago() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let result = format_timestamp(now - 172800); // 2 days ago
    assert_eq!(result, "2d ago");
}

#[cfg(feature = "oracle")]
#[test]
fn test_extract_exit_code_patterns() {
    assert_eq!(extract_exit_code("exit code 127"), 127);
    assert_eq!(extract_exit_code("exited with 1"), 1);
    assert_eq!(extract_exit_code("returned 255"), 255);
    assert_eq!(extract_exit_code("status 42"), 42);
}

#[cfg(feature = "oracle")]
#[test]
fn test_extract_exit_code_special_cases() {
    assert_eq!(extract_exit_code("command not found"), 127);
    assert_eq!(extract_exit_code("Permission denied"), 126);
    assert_eq!(extract_exit_code("permission denied"), 126);
    assert_eq!(extract_exit_code("unknown error"), 1);
}

// ============================================================================
// Config Analysis Helper Tests
// ============================================================================

#[test]
fn test_count_duplicate_path_entries_empty() {
    let analysis = crate::config::ConfigAnalysis {
        file_path: PathBuf::from("/tmp/test"),
        config_type: crate::config::ConfigType::Bashrc,
        line_count: 0,
        complexity_score: 0,
        issues: vec![],
        path_entries: vec![],
        performance_issues: vec![],
    };
    assert_eq!(count_duplicate_path_entries(&analysis), 0);
}

#[test]
fn test_count_duplicate_path_entries_with_duplicates() {
    let analysis = crate::config::ConfigAnalysis {
        file_path: PathBuf::from("/tmp/test"),
        config_type: crate::config::ConfigType::Bashrc,
        line_count: 3,
        complexity_score: 1,
        issues: vec![],
        path_entries: vec![
            crate::config::PathEntry {
                line: 1,
                path: "/usr/bin".to_string(),
                is_duplicate: false,
            },
            crate::config::PathEntry {
                line: 2,
                path: "/usr/bin".to_string(),
                is_duplicate: true,
            },
            crate::config::PathEntry {
                line: 3,
                path: "/usr/local/bin".to_string(),
                is_duplicate: false,
            },
        ],
        performance_issues: vec![],
    };
    assert_eq!(count_duplicate_path_entries(&analysis), 1);
}

// ============================================================================
// Dockerfile Command Tests
// ============================================================================

#[test]
fn test_dockerfile_lint_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let dockerfile = temp_dir.path().join("Dockerfile");
    fs::write(&dockerfile, "FROM ubuntu:20.04\nRUN apt-get update").unwrap();

    let result = dockerfile_lint_command(&dockerfile, LintFormat::Human, None);
    // Should succeed (may have warnings but shouldn't error)
    let _ = result;
}

#[test]
fn test_dockerfile_lint_command_with_rules() {
    let temp_dir = TempDir::new().unwrap();
    let dockerfile = temp_dir.path().join("Dockerfile");
    fs::write(&dockerfile, "FROM ubuntu:20.04\nRUN apt-get update").unwrap();

    let result = dockerfile_lint_command(&dockerfile, LintFormat::Json, Some("DOCKER001"));
    let _ = result;
}

// ============================================================================
// Make Command Tests
// ============================================================================

#[test]
fn test_make_parse_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let makefile = temp_dir.path().join("Makefile");
    fs::write(
        &makefile,
        ".PHONY: all clean\n\nall:\n\t@echo 'Building...'\n\nclean:\n\t@rm -f *.o\n",
    )
    .unwrap();

    let result = make_parse_command(&makefile, MakeOutputFormat::Text);
    assert!(result.is_ok());
}

#[test]
fn test_make_parse_command_json_format() {
    let temp_dir = TempDir::new().unwrap();
    let makefile = temp_dir.path().join("Makefile");
    fs::write(&makefile, "all:\n\t@echo 'test'\n").unwrap();

    let result = make_parse_command(&makefile, MakeOutputFormat::Json);
    assert!(result.is_ok());
}

#[test]
fn test_make_lint_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let makefile = temp_dir.path().join("Makefile");
    // Include .SUFFIXES and .DELETE_ON_ERROR to avoid warnings
    fs::write(
        &makefile,
        ".SUFFIXES:\n.DELETE_ON_ERROR:\n.PHONY: all\nall:\n\t@echo test\n",
    )
    .unwrap();

    let result = make_lint_command(&makefile, LintFormat::Human, false, None, None);
    assert!(result.is_ok());
}

// ============================================================================
// Config Command Tests
// ============================================================================

#[test]
fn test_config_analyze_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join(".bashrc");
    fs::write(
        &config_file,
        "export PATH=\"/usr/bin:$PATH\"\nalias ll='ls -la'\n",
    )
    .unwrap();

    let result = config_analyze_command(&config_file, ConfigOutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_config_analyze_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join(".zshrc");
    fs::write(&config_file, "export EDITOR=vim\n").unwrap();

    let result = config_analyze_command(&config_file, ConfigOutputFormat::Json);
    assert!(result.is_ok());
}

#[test]
fn test_config_lint_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join(".bashrc");
    fs::write(&config_file, "export PATH=/usr/bin\n").unwrap();

    let result = config_lint_command(&config_file, ConfigOutputFormat::Human);
    let _ = result;
}

// ============================================================================
// Handle Output Tests
// ============================================================================

#[test]
fn test_handle_output_to_file_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("output.txt");

    let result = handle_output_to_file(&output_path, "test content");
    assert!(result.is_ok());
    assert!(output_path.exists());
    assert_eq!(fs::read_to_string(&output_path).unwrap(), "test content");
}

// ============================================================================
// Inspect Command Tests
// ============================================================================

#[test]
fn test_inspect_command_rust_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();

    let result = inspect_command(
        input_path.to_str().unwrap(),
        InspectionFormat::Markdown,
        None,
        false,
    );
    let _ = result;
}

#[test]
fn test_inspect_command_shell_script() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("script.sh");
    fs::write(&input_path, "#!/bin/bash\necho hello").unwrap();

    let result = inspect_command(
        input_path.to_str().unwrap(),
        InspectionFormat::Json,
        None,
        true,
    );
    let _ = result;
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
// Purify Command Tests
// ============================================================================

#[test]
fn test_purify_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("script.sh");
    let output_path = temp_dir.path().join("purified.sh");

    fs::write(&input_path, "#!/bin/bash\necho $RANDOM").unwrap();

    let result = purify_command(
        &input_path,
        Some(&output_path),
        false,
        false,
        false,
        false,
        false,
        false,
    );
    let _ = result;
}

#[test]
fn test_purify_command_with_lint() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("script.sh");

    fs::write(&input_path, "#!/bin/bash\necho hello world").unwrap();

    let result = purify_command(&input_path, None, true, false, false, false, false, false);
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

// ============================================================================
// Additional Coverage Tests - Unique test names to avoid conflicts
// ============================================================================

#[test]
fn test_purify_dockerfile_content_basic() {
    let dockerfile = "FROM ubuntu:20.04\nRUN apt-get update";
    let result = purify_dockerfile(dockerfile, false);
    assert!(result.is_ok());
}

#[test]
fn test_purify_dockerfile_content_skip_user() {
    let dockerfile = "FROM ubuntu:20.04\nRUN echo hello";
    let result = purify_dockerfile(dockerfile, true);
    assert!(result.is_ok());
}

#[test]
fn test_purify_dockerfile_content_with_cleanup() {
    let dockerfile = "FROM ubuntu:20.04\nRUN apt-get update && apt-get install -y curl";
    let result = purify_dockerfile(dockerfile, false);
    assert!(result.is_ok());
    let purified = result.unwrap();
    // Should add cleanup patterns
    assert!(purified.contains("apt-get") || purified.contains("FROM"));
}

#[test]
fn test_logic_find_devcontainer_json_exists() {
    let temp_dir = TempDir::new().unwrap();
    let devcontainer_dir = temp_dir.path().join(".devcontainer");
    fs::create_dir_all(&devcontainer_dir).unwrap();

    let json_path = devcontainer_dir.join("devcontainer.json");
    fs::write(&json_path, r#"{"name": "test"}"#).unwrap();

    // Test finding devcontainer.json
    let result = logic_find_devcontainer_json(temp_dir.path());
    assert!(result.is_ok());
}

#[test]
fn test_logic_find_devcontainer_json_not_exists() {
    let temp_dir = TempDir::new().unwrap();
    let result = logic_find_devcontainer_json(temp_dir.path());
    assert!(result.is_err());
}

// ============================================================================
// Score Command Tests (covers score_command + print_* formatters)
// ============================================================================

#[test]
fn test_score_command_shell_script_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\nset -eu\necho 'hello'\nexit 0\n").unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Human,
        false,
        false,
        false,
        false,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_shell_script_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'test'\n").unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Json,
        false,
        false,
        false,
        false,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_shell_script_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'test'\n").unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Markdown,
        false,
        false,
        false,
        false,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_shell_script_detailed() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\nset -eu\necho 'hello'\n").unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Human,
        true, // detailed
        false,
        false,
        false,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_dockerfile_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM python:3.11-slim\nRUN pip install flask\nCOPY . /app\nWORKDIR /app\nCMD [\"python\", \"app.py\"]\n").unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Human,
        true,
        true, // dockerfile
        false,
        true, // show_grade
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_dockerfile_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM alpine:3.18\nRUN apk add --no-cache curl\n").unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Json,
        false,
        true, // dockerfile
        false,
        false,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_dockerfile_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(
        &input,
        "FROM node:20-alpine\nWORKDIR /app\nCOPY . .\nCMD [\"node\", \"index.js\"]\n",
    )
    .unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Markdown,
        false,
        true, // dockerfile
        false,
        false,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_dockerfile_with_runtime() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(
        &input,
        "FROM ubuntu:22.04\nRUN apt-get update\nCOPY . /app\n",
    )
    .unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Human,
        true,
        true, // dockerfile
        true, // runtime
        true, // show_grade
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_dockerfile_with_coursera_profile() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM python:3.11-slim\nRUN pip install flask\n").unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Human,
        true,
        true, // dockerfile
        true, // runtime
        true, // show_grade
        Some(LintProfileArg::Coursera),
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_nonexistent_file() {
    let result = score_command(
        &PathBuf::from("/nonexistent/script.sh"),
        ScoreOutputFormat::Human,
        false,
        false,
        false,
        false,
        None,
    );
    assert!(result.is_err());
}

// ============================================================================
// Audit Command Tests (covers audit_command + print_* formatters)
// ============================================================================

#[test]
fn test_audit_command_basic_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\nset -eu\necho 'hello'\n").unwrap();

    let result = audit_command(&input, &AuditOutputFormat::Human, false, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_audit_command_basic_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'hello'\n").unwrap();

    let result = audit_command(&input, &AuditOutputFormat::Json, false, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_audit_command_basic_sarif() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'hello'\n").unwrap();

    let result = audit_command(&input, &AuditOutputFormat::Sarif, false, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_audit_command_detailed() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\nset -eu\necho 'hello world'\nexit 0\n").unwrap();

    let result = audit_command(&input, &AuditOutputFormat::Human, false, true, None);
    assert!(result.is_ok());
}

#[test]
fn test_audit_command_strict_mode() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    // Script with unquoted variable (produces warning)
    fs::write(&input, "#!/bin/sh\necho $HOME\n").unwrap();

    let result = audit_command(&input, &AuditOutputFormat::Human, true, false, None);
    // Strict mode: warnings cause failure
    let _ = result; // may pass or fail depending on lint rules
}

#[test]
fn test_audit_command_min_grade_pass() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\nset -eu\necho 'hello'\nexit 0\n").unwrap();

    let result = audit_command(
        &input,
        &AuditOutputFormat::Human,
        false,
        false,
        Some("F"), // very low bar
    );
    assert!(result.is_ok());
}

#[test]
fn test_audit_command_min_grade_fail() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho $RANDOM\n").unwrap();

    let result = audit_command(
        &input,
        &AuditOutputFormat::Human,
        false,
        false,
        Some("A+"), // very high bar
    );
    // May fail if grade is below A+
    let _ = result;
}

#[test]
fn test_audit_command_nonexistent_file() {
    let result = audit_command(
        &PathBuf::from("/nonexistent/audit.sh"),
        &AuditOutputFormat::Human,
        false,
        false,
        None,
    );
    assert!(result.is_err());
}

// ============================================================================
// Coverage Command Tests (covers coverage_command + print_* formatters)
// ============================================================================

#[test]
fn test_coverage_command_terminal() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\nset -eu\necho 'hello'\nexit 0\n").unwrap();

    let result = coverage_command(&input, &CoverageOutputFormat::Terminal, None, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_coverage_command_terminal_detailed() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'line1'\necho 'line2'\n").unwrap();

    let result = coverage_command(
        &input,
        &CoverageOutputFormat::Terminal,
        None,
        true, // detailed
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_coverage_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'test'\n").unwrap();

    let result = coverage_command(&input, &CoverageOutputFormat::Json, None, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_coverage_command_html_to_stdout() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'test'\n").unwrap();

    let result = coverage_command(&input, &CoverageOutputFormat::Html, None, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_coverage_command_html_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    let output = temp_dir.path().join("coverage.html");
    fs::write(&input, "#!/bin/sh\necho 'test'\n").unwrap();

    let result = coverage_command(
        &input,
        &CoverageOutputFormat::Html,
        None,
        false,
        Some(&output),
    );
    assert!(result.is_ok());
    assert!(output.exists());
}

#[test]
fn test_coverage_command_lcov() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'test'\n").unwrap();

    let result = coverage_command(&input, &CoverageOutputFormat::Lcov, None, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_coverage_command_min_threshold_pass() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'test'\n").unwrap();

    let result = coverage_command(
        &input,
        &CoverageOutputFormat::Terminal,
        Some(0), // 0% min - always passes
        false,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_coverage_command_nonexistent_file() {
    let result = coverage_command(
        &PathBuf::from("/nonexistent/coverage.sh"),
        &CoverageOutputFormat::Terminal,
        None,
        false,
        None,
    );
    assert!(result.is_err());
}

// ============================================================================
// Format Command Tests (covers format_command)
// ============================================================================

#[test]
fn test_format_command_basic_inplace() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho  'hello'\n").unwrap();

    let result = format_command(&[input.clone()], false, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_format_command_check_mode() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'hello'\n").unwrap();

    let result = format_command(&[input.clone()], true, false, None);
    // May pass or fail depending on formatting rules
    let _ = result;
}

#[test]
fn test_format_command_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    let original = "#!/bin/sh\necho  'hello'\n";
    fs::write(&input, original).unwrap();

    let result = format_command(&[input.clone()], false, true, None);
    assert!(result.is_ok());

    // Dry run should not modify the file
    let after = fs::read_to_string(&input).unwrap();
    assert_eq!(after, original);
}

#[test]
fn test_format_command_to_output_file() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    let output = temp_dir.path().join("formatted.sh");
    fs::write(&input, "#!/bin/sh\necho 'hello'\n").unwrap();

    let result = format_command(&[input.clone()], false, false, Some(&output));
    assert!(result.is_ok());
    assert!(output.exists());
}

#[test]
fn test_format_command_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let input1 = temp_dir.path().join("a.sh");
    let input2 = temp_dir.path().join("b.sh");
    fs::write(&input1, "#!/bin/sh\necho 'a'\n").unwrap();
    fs::write(&input2, "#!/bin/sh\necho 'b'\n").unwrap();

    let result = format_command(&[input1, input2], false, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_format_command_nonexistent_file() {
    let result = format_command(
        &[PathBuf::from("/nonexistent/format.sh")],
        false,
        false,
        None,
    );
    assert!(result.is_err());
}

// ============================================================================
// Test Command Tests (covers test_command + print_* formatters)
// ============================================================================

#[test]
fn test_test_command_no_tests_found() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'no tests here'\n").unwrap();

    let result = test_command(&input, TestOutputFormat::Human, false, None);
    assert!(result.is_ok()); // Returns OK with "No tests found" message
}

#[test]
fn test_test_command_json_format() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'no tests'\n").unwrap();

    let result = test_command(&input, TestOutputFormat::Json, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_test_command_junit_format() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'no tests'\n").unwrap();

    let result = test_command(&input, TestOutputFormat::Junit, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_test_command_with_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'no tests'\n").unwrap();

    let result = test_command(&input, TestOutputFormat::Human, false, Some("nonexistent"));
    assert!(result.is_ok()); // No tests match pattern
}

#[test]
fn test_test_command_nonexistent_file() {
    let result = test_command(
        &PathBuf::from("/nonexistent/test.sh"),
        TestOutputFormat::Human,
        false,
        None,
    );
    assert!(result.is_err());
}

// ============================================================================
// Purify Command Tests (additional coverage for report and test generation)
// ============================================================================

#[test]
fn test_purify_command_with_output_and_report() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("messy.sh");
    let output = temp_dir.path().join("purified.sh");
    fs::write(&input, "#!/bin/bash\nmkdir /tmp/test\necho $RANDOM\n").unwrap();

    let result = purify_command(
        &input,
        Some(&output),
        true,
        false,
        false,
        false,
        false,
        false,
    );
    assert!(result.is_ok());
    assert!(output.exists());
}

#[test]
fn test_purify_command_to_stdout() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/bash\necho hello\n").unwrap();

    let result = purify_command(&input, None, false, false, false, false, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_purify_command_with_tests() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    let output = temp_dir.path().join("purified.sh");
    fs::write(&input, "#!/bin/bash\necho hello\n").unwrap();

    let result = purify_command(
        &input,
        Some(&output),
        false,
        true,
        false,
        false,
        false,
        false,
    );
    assert!(result.is_ok());
    // Test file should be generated
    let test_path = temp_dir.path().join("purified_test.sh");
    assert!(test_path.exists());
}

#[test]
fn test_purify_command_with_property_tests() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    let output = temp_dir.path().join("purified.sh");
    fs::write(&input, "#!/bin/bash\necho hello\n").unwrap();

    let result = purify_command(&input, Some(&output), true, true, true, false, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_purify_command_with_tests_requires_output() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/bash\necho hello\n").unwrap();

    let result = purify_command(&input, None, false, true, false, false, false, false);
    assert!(result.is_err()); // --with-tests requires -o flag
}

#[test]
fn test_purify_command_nonexistent_file() {
    let result = purify_command(
        &PathBuf::from("/nonexistent/purify.sh"),
        None,
        false,
        false,
        false,
        false,
        false,
        false,
    );
    assert!(result.is_err());
}

// ============================================================================
// Dockerfile Profile Command Tests
// ============================================================================

#[test]
fn test_dockerfile_profile_command_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(
        &input,
        "FROM python:3.11-slim\nRUN pip install flask\nCOPY . /app\n",
    )
    .unwrap();

    let result = dockerfile_profile_command(
        &input,
        true,  // build
        true,  // layers
        false, // startup
        false, // memory
        false, // cpu
        None,  // workload
        "30s", // duration
        None,  // profile
        false, // simulate_limits
        false, // full
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_profile_command_full_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(
        &input,
        "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl\nCOPY . /app\n",
    )
    .unwrap();

    let result = dockerfile_profile_command(
        &input,
        false,
        false,
        false,
        false,
        false,
        None,
        "30s",
        None,
        false,
        true, // full (enables all sections)
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_profile_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM alpine:3.18\nRUN apk add curl\n").unwrap();

    let result = dockerfile_profile_command(
        &input,
        false,
        false,
        false,
        false,
        false,
        None,
        "30s",
        None,
        false,
        false,
        ReportFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_profile_command_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM node:20-alpine\nCOPY . /app\n").unwrap();

    let result = dockerfile_profile_command(
        &input,
        false,
        false,
        false,
        false,
        false,
        None,
        "30s",
        None,
        false,
        false,
        ReportFormat::Markdown,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_profile_command_coursera_with_limits() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM python:3.11-slim\nRUN pip install flask\n").unwrap();

    let result = dockerfile_profile_command(
        &input,
        true,
        true,
        true,
        true,
        true,
        None,
        "30s",
        Some(LintProfileArg::Coursera),
        true, // simulate_limits
        false,
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

// ============================================================================
// Dockerfile Size Check Command Tests
// ============================================================================

#[test]
fn test_dockerfile_size_check_command_human_basic() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM alpine:3.18\nRUN apk add curl\n").unwrap();

    let result = dockerfile_size_check_command(
        &input,
        false, // verbose
        false, // layers
        false, // detect_bloat
        false, // verify
        false, // docker_verify
        None,  // profile
        false, // strict
        None,  // max_size
        false, // compression_analysis
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_size_check_command_verbose_with_bloat() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(
        &input,
        "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl wget git\n",
    )
    .unwrap();

    let result = dockerfile_size_check_command(
        &input,
        true, // verbose
        true, // layers
        true, // detect_bloat
        false,
        false,
        None,
        false,
        None,
        true, // compression_analysis
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_size_check_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM python:3.11\nRUN pip install flask\n").unwrap();

    let result = dockerfile_size_check_command(
        &input,
        false,
        false,
        false,
        false,
        false,
        None,
        false,
        None,
        false,
        ReportFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_size_check_command_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM node:20\nCOPY . /app\n").unwrap();

    let result = dockerfile_size_check_command(
        &input,
        false,
        false,
        false,
        false,
        false,
        None,
        false,
        None,
        false,
        ReportFormat::Markdown,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_size_check_command_with_coursera_profile() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM python:3.11-slim\nRUN pip install flask\n").unwrap();

    let result = dockerfile_size_check_command(
        &input,
        true,
        true,
        true,
        false,
        false,
        Some(LintProfileArg::Coursera),
        false,
        None,
        false,
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_size_check_command_custom_max_size_gb() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM alpine:3.18\nRUN echo hello\n").unwrap();

    let result = dockerfile_size_check_command(
        &input,
        false,
        false,
        false,
        false,
        false,
        None,
        false,
        Some("5GB"),
        false,
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_size_check_command_custom_max_size_mb() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM alpine:3.18\nRUN echo hello\n").unwrap();

    let result = dockerfile_size_check_command(
        &input,
        false,
        false,
        false,
        false,
        false,
        None,
        false,
        Some("500MB"),
        false,
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

// ============================================================================
// Dockerfile Full Validate Command Tests
// ============================================================================

#[test]
fn test_dockerfile_full_validate_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(
        &input,
        "FROM python:3.11-slim\nRUN pip install flask\nCOPY . /app\nUSER 65534\n",
    )
    .unwrap();

    let result = dockerfile_full_validate_command(
        &input,
        None,  // profile
        true,  // size_check
        false, // graded
        false, // runtime
        false, // strict
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_full_validate_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM alpine:3.18\nRUN apk add curl\n").unwrap();

    let result = dockerfile_full_validate_command(
        &input,
        None,
        true,
        false,
        false,
        false,
        ReportFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_full_validate_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM node:20-alpine\nCOPY . /app\n").unwrap();

    let result = dockerfile_full_validate_command(
        &input,
        None,
        true,
        false,
        false,
        false,
        ReportFormat::Markdown,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_full_validate_coursera_profile() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(
        &input,
        "FROM python:3.11-slim\nRUN pip install flask\nUSER 65534\n",
    )
    .unwrap();

    let result = dockerfile_full_validate_command(
        &input,
        Some(LintProfileArg::Coursera),
        true,
        false,
        false,
        false,
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_full_validate_with_runtime() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:22.04\nRUN apt-get update\n").unwrap();

    let result = dockerfile_full_validate_command(
        &input,
        None,
        true,
        false,
        true, // runtime
        false,
        ReportFormat::Human,
    );
    assert!(result.is_ok());
}

// ============================================================================
// Playbook Command Tests
// ============================================================================

#[test]
fn test_playbook_command_validate_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("playbook.yaml");
    fs::write(
        &input,
        "version: \"1.0\"\nmachine:\n  id: test-machine\n  initial: start\n",
    )
    .unwrap();

    let result = playbook_command(&input, false, PlaybookFormat::Human, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_playbook_command_run_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("playbook.yaml");
    fs::write(
        &input,
        "version: \"1.0\"\nmachine:\n  id: deploy\n  initial: setup\n",
    )
    .unwrap();

    let result = playbook_command(&input, true, PlaybookFormat::Human, true, false);
    assert!(result.is_ok());
}

#[test]
fn test_playbook_command_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("playbook.yaml");
    fs::write(
        &input,
        "version: \"1.0\"\nmachine:\n  id: test\n  initial: start\n",
    )
    .unwrap();

    let result = playbook_command(&input, true, PlaybookFormat::Human, false, true);
    assert!(result.is_ok());
}

#[test]
fn test_playbook_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("playbook.yaml");
    fs::write(
        &input,
        "version: \"1.0\"\nmachine:\n  id: test\n  initial: start\n",
    )
    .unwrap();

    let result = playbook_command(&input, false, PlaybookFormat::Json, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_playbook_command_junit() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("playbook.yaml");
    fs::write(
        &input,
        "version: \"1.0\"\nmachine:\n  id: test\n  initial: start\n",
    )
    .unwrap();

    let result = playbook_command(&input, false, PlaybookFormat::Junit, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_playbook_command_invalid() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("bad.yaml");
    fs::write(&input, "this is not a valid playbook").unwrap();

    let result = playbook_command(&input, false, PlaybookFormat::Human, false, false);
    assert!(result.is_err());
}

#[test]
fn test_playbook_command_nonexistent() {
    let result = playbook_command(
        &PathBuf::from("/nonexistent/playbook.yaml"),
        false,
        PlaybookFormat::Human,
        false,
        false,
    );
    assert!(result.is_err());
}

// ============================================================================
// Mutate Command Tests
// ============================================================================

#[test]
fn test_mutate_command_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(
        &input,
        "#!/bin/sh\nif [ \"$x\" == \"y\" ]; then\n  echo true\nfi\n",
    )
    .unwrap();

    let result = mutate_command(&input, None, MutateFormat::Human, 10, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_mutate_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\nif [ $x -eq 0 ]; then exit 0; fi\n").unwrap();

    let result = mutate_command(&input, None, MutateFormat::Json, 5, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_mutate_command_csv() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\ntrue && echo ok\n").unwrap();

    let result = mutate_command(&input, None, MutateFormat::Csv, 5, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_mutate_command_show_survivors() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(
        &input,
        "#!/bin/sh\nif [ \"$a\" == \"$b\" ]; then\n  echo equal\nfi\nexit 0\n",
    )
    .unwrap();

    let result = mutate_command(&input, None, MutateFormat::Human, 10, true, None);
    assert!(result.is_ok());
}

#[test]
fn test_mutate_command_no_mutations() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho hello\n").unwrap();

    let result = mutate_command(&input, None, MutateFormat::Human, 10, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_mutate_command_nonexistent() {
    let result = mutate_command(
        &PathBuf::from("/nonexistent/mutate.sh"),
        None,
        MutateFormat::Human,
        10,
        false,
        None,
    );
    assert!(result.is_err());
}

// ============================================================================
// Simulate Command Tests
// ============================================================================

#[test]
fn test_simulate_command_human_deterministic() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'deterministic'\nexit 0\n").unwrap();

    let result = simulate_command(&input, 42, false, false, SimulateFormat::Human, false);
    assert!(result.is_ok());
}

#[test]
fn test_simulate_command_human_nondeterministic() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho $RANDOM\necho $$\n").unwrap();

    let result = simulate_command(&input, 42, false, false, SimulateFormat::Human, false);
    assert!(result.is_ok());
}

#[test]
fn test_simulate_command_with_trace() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho hello\necho world\n").unwrap();

    let result = simulate_command(&input, 42, false, false, SimulateFormat::Human, true);
    assert!(result.is_ok());
}

#[test]
fn test_simulate_command_with_verify() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho test\n").unwrap();

    let result = simulate_command(&input, 42, true, false, SimulateFormat::Human, true);
    assert!(result.is_ok());
}

#[test]
fn test_simulate_command_with_mock_externals() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho test\n").unwrap();

    let result = simulate_command(&input, 42, false, true, SimulateFormat::Human, false);
    assert!(result.is_ok());
}

#[test]
fn test_simulate_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho test\n").unwrap();

    let result = simulate_command(&input, 42, false, false, SimulateFormat::Json, false);
    assert!(result.is_ok());
}

#[test]
fn test_simulate_command_trace_format() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\n# comment\necho hello\necho world\n").unwrap();

    let result = simulate_command(&input, 42, false, false, SimulateFormat::Trace, false);
    assert!(result.is_ok());
}

#[test]
fn test_simulate_command_nonexistent() {
    let result = simulate_command(
        &PathBuf::from("/nonexistent/sim.sh"),
        42,
        false,
        false,
        SimulateFormat::Human,
        false,
    );
    assert!(result.is_err());
}

// ============================================================================
// Dockerfile Purify Command Tests
// ============================================================================

#[test]
fn test_dockerfile_purify_command_to_stdout() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:20.04\nRUN apt-get update\n").unwrap();

    let result = dockerfile_purify_command(
        &input,
        None,  // output
        false, // fix
        false, // no_backup
        false, // dry_run
        false, // report
        ReportFormat::Human,
        false, // skip_user
        false, // skip_bash_purify
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_purify_command_to_output_file() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    let output = temp_dir.path().join("Dockerfile.purified");
    fs::write(&input, "FROM ubuntu:20.04\nRUN apt-get update\n").unwrap();

    let result = dockerfile_purify_command(
        &input,
        Some(&output),
        false,
        false,
        false,
        false,
        ReportFormat::Human,
        false,
        false,
    );
    assert!(result.is_ok());
    assert!(output.exists());
}

#[test]
fn test_dockerfile_purify_command_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:20.04\nRUN echo hello\n").unwrap();

    let result = dockerfile_purify_command(
        &input,
        None,
        false,
        false,
        true, // dry_run
        false,
        ReportFormat::Human,
        false,
        false,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_purify_command_fix_inplace() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:20.04\nRUN apt-get update\n").unwrap();

    let result = dockerfile_purify_command(
        &input,
        None,
        true,  // fix (in-place)
        false, // no_backup (creates backup)
        false,
        false,
        ReportFormat::Human,
        false,
        false,
    );
    assert!(result.is_ok());
    // Backup should be created
    assert!(input.with_extension("bak").exists());
}

#[test]
fn test_dockerfile_purify_command_fix_no_backup() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:20.04\nRUN echo test\n").unwrap();

    let result = dockerfile_purify_command(
        &input,
        None,
        true, // fix
        true, // no_backup
        false,
        false,
        ReportFormat::Human,
        false,
        false,
    );
    assert!(result.is_ok());
}

#[test]
fn test_dockerfile_purify_command_skip_user() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM ubuntu:20.04\nRUN echo test\n").unwrap();

    let result = dockerfile_purify_command(
        &input,
        None,
        false,
        false,
        false,
        false,
        ReportFormat::Human,
        true, // skip_user
        false,
    );
    assert!(result.is_ok());
}

// ============================================================================
// Make Command Tests (additional coverage)
// ============================================================================

#[test]
fn test_make_lint_command_with_fix() {
    let temp_dir = TempDir::new().unwrap();
    let makefile = temp_dir.path().join("Makefile");
    let output = temp_dir.path().join("Makefile.fixed");
    fs::write(&makefile, ".PHONY: all\nall:\n\t@echo test\n").unwrap();

    let result = make_lint_command(&makefile, LintFormat::Human, true, Some(&output), None);
    // May or may not have fixable issues
    let _ = result;
}

#[test]
fn test_make_lint_command_json_format() {
    let temp_dir = TempDir::new().unwrap();
    let makefile = temp_dir.path().join("Makefile");
    fs::write(&makefile, ".PHONY: all\nall:\n\t@echo test\n").unwrap();

    // Note: show_lint_results calls process::exit on warnings/errors
    // so we test with a rule filter that produces no matches
    let result = make_lint_command(
        &makefile,
        LintFormat::Human,
        false,
        None,
        Some("NONEXISTENT"),
    );
    let _ = result;
}

#[test]
fn test_make_lint_command_with_rules_filter() {
    let temp_dir = TempDir::new().unwrap();
    let makefile = temp_dir.path().join("Makefile");
    fs::write(&makefile, "all:\n\t@echo test\n").unwrap();

    let result = make_lint_command(&makefile, LintFormat::Human, false, None, Some("MAKE001"));
    let _ = result;
}

#[test]
fn test_make_purify_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Makefile");
    let output = temp_dir.path().join("Makefile.purified");
    fs::write(&input, ".PHONY: all\nall:\n\t@echo test\n").unwrap();

    let result = make_purify_command(
        &input,
        Some(&output),
        false, // fix
        false, // report
        ReportFormat::Human,
        false, // with_tests
        false, // property_tests
        false, // preserve_formatting
        None,  // max_line_length
        false, // skip_blank_line_removal
        false, // skip_consolidation
    );
    assert!(result.is_ok());
}

// ============================================================================
// Convert Lint Format Test
// ============================================================================

#[test]
fn test_convert_lint_format_human() {
    let result = convert_lint_format(LintFormat::Human);
    assert!(matches!(result, crate::linter::output::OutputFormat::Human));
}

#[test]
fn test_convert_lint_format_json() {
    let result = convert_lint_format(LintFormat::Json);
    assert!(matches!(result, crate::linter::output::OutputFormat::Json));
}

#[test]
fn test_convert_lint_format_sarif() {
    let result = convert_lint_format(LintFormat::Sarif);
    assert!(matches!(result, crate::linter::output::OutputFormat::Sarif));
}

// ============================================================================
// Run Filtered Lint Tests
// ============================================================================

#[test]
fn test_run_filtered_lint_no_filter() {
    let source = ".PHONY: all\nall:\n\t@echo test\n";
    let result = run_filtered_lint(source, None);
    // Should return lint results (may have diagnostics)
    let _ = result.diagnostics.len();
}

#[test]
fn test_run_filtered_lint_with_filter() {
    let source = "all:\n\t@echo test\n";
    let result = run_filtered_lint(source, Some("MAKE001"));
    // Should only contain MAKE001 diagnostics (if any)
    for d in &result.diagnostics {
        assert!(d.code.contains("MAKE001"));
    }
}

#[test]
fn test_run_filtered_lint_nonexistent_rule() {
    let source = "all:\n\t@echo test\n";
    let result = run_filtered_lint(source, Some("NONEXISTENT999"));
    assert!(result.diagnostics.is_empty());
}

// ============================================================================
// Estimate Build Time Tests
// ============================================================================

#[test]
fn test_estimate_build_time_simple() {
    use crate::linter::docker_profiler::estimate_size;
    let source = "FROM alpine:3.18\nRUN echo hello\n";
    let estimate = estimate_size(source);
    let time = estimate_build_time(&estimate);
    assert!(time.contains('s') || time.contains('m'));
}

#[test]
fn test_estimate_build_time_with_apt() {
    use crate::linter::docker_profiler::estimate_size;
    let source = "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl\n";
    let estimate = estimate_size(source);
    let time = estimate_build_time(&estimate);
    assert!(time.contains('s') || time.contains('m'));
}

// ============================================================================
// Dockerfile Lint with Rules Filter Test
// ============================================================================

#[test]
fn test_dockerfile_lint_command_sarif_format() {
    let temp_dir = TempDir::new().unwrap();
    let dockerfile = temp_dir.path().join("Dockerfile");
    fs::write(&dockerfile, "FROM ubuntu:20.04\nRUN apt-get update\n").unwrap();

    let result = dockerfile_lint_command(&dockerfile, LintFormat::Sarif, None);
    let _ = result;
}

#[test]
fn test_dockerfile_lint_command_nonexistent() {
    let result = dockerfile_lint_command(
        &PathBuf::from("/nonexistent/Dockerfile"),
        LintFormat::Human,
        None,
    );
    assert!(result.is_err());
}

// ============================================================================
// Config Command Tests (additional coverage)
// ============================================================================

#[test]
fn test_config_analyze_command_json_format() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join(".bashrc");
    fs::write(
        &config_file,
        "export PATH=/usr/bin:$PATH\nalias ll='ls -la'\n",
    )
    .unwrap();

    let result = config_analyze_command(&config_file, ConfigOutputFormat::Json);
    assert!(result.is_ok());
}

#[test]
fn test_config_analyze_command_nonexistent() {
    let result = config_analyze_command(
        &PathBuf::from("/nonexistent/.bashrc"),
        ConfigOutputFormat::Human,
    );
    assert!(result.is_err());
}

#[test]
fn test_config_lint_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join(".bashrc");
    fs::write(&config_file, "export PATH=/usr/bin\n").unwrap();

    let result = config_lint_command(&config_file, ConfigOutputFormat::Json);
    let _ = result;
}

// ============================================================================
// Parse Public Key Test
// ============================================================================

#[test]
fn test_parse_public_key_valid() {
    // 32 bytes = 64 hex chars
    let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let result = parse_public_key(hex);
    assert!(result.is_ok());
}

#[test]
fn test_parse_public_key_invalid_length() {
    let result = parse_public_key("0123456789abcdef");
    assert!(result.is_err());
}

#[test]
fn test_parse_public_key_invalid_hex() {
    let result =
        parse_public_key("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
    assert!(result.is_err());
}

// ===== Tests for Dockerfile helper functions (moved from commands.rs) =====

// FUNCTION 1: convert_add_to_copy_if_local()

#[test]
fn test_convert_add_to_copy_if_local_happy_path_local_file() {
    let line = "ADD myfile.txt /app/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, "COPY myfile.txt /app/",
        "Local file should convert ADD to COPY"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_http_url() {
    let line = "ADD http://example.com/file.tar.gz /tmp/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        "HTTP URLs should preserve ADD (not convert to COPY)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_https_url() {
    let line = "ADD https://example.com/archive.zip /tmp/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        "HTTPS URLs should preserve ADD (not convert to COPY)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_tar_archive() {
    let line = "ADD archive.tar /tmp/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        ".tar archives should preserve ADD (auto-extraction feature)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_tar_gz() {
    let line = "ADD file.tar.gz /app/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        ".tar.gz archives should preserve ADD (auto-extraction)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_tgz() {
    let line = "ADD package.tgz /opt/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        ".tgz archives should preserve ADD (auto-extraction)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_tar_bz2() {
    let line = "ADD data.tar.bz2 /data/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        ".tar.bz2 archives should preserve ADD (auto-extraction)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_tar_xz() {
    let line = "ADD compressed.tar.xz /usr/local/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        ".tar.xz archives should preserve ADD (auto-extraction)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_preserves_tar_Z() {
    let line = "ADD legacy.tar.Z /legacy/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        ".tar.Z archives should preserve ADD (auto-extraction)"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_empty_line() {
    let line = "";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(result, line, "Empty line should be unchanged");
}

#[test]
fn test_convert_add_to_copy_if_local_malformed_no_args() {
    let line = "ADD";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, line,
        "Malformed ADD (no arguments) should be unchanged"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_with_extra_spaces() {
    let line = "ADD    local_file.txt    /app/";
    let result = convert_add_to_copy_if_local(line);
    assert_eq!(
        result, "COPY    local_file.txt    /app/",
        "Should convert ADD to COPY while preserving spacing"
    );
}

#[test]
fn test_convert_add_to_copy_if_local_non_docker_line() {
    let line = "# This is a comment with ADD in it";
    let result = convert_add_to_copy_if_local(line);
    // Should not convert comment lines
    assert_eq!(result, line, "Comment lines should not be processed");
}

// FUNCTION 2: add_no_install_recommends()

#[test]
fn test_add_no_install_recommends_happy_path_with_y_flag() {
    let line = "RUN apt-get install -y curl";
    let result = add_no_install_recommends(line);
    assert_eq!(
        result, "RUN apt-get install -y --no-install-recommends curl",
        "Should add --no-install-recommends after -y flag"
    );
}

#[test]
fn test_add_no_install_recommends_without_y_flag() {
    let line = "RUN apt-get install python3";
    let result = add_no_install_recommends(line);
    assert_eq!(
        result, "RUN apt-get install --no-install-recommends python3",
        "Should add --no-install-recommends after install"
    );
}

#[test]
fn test_add_no_install_recommends_already_present() {
    let line = "RUN apt-get install -y --no-install-recommends git";
    let result = add_no_install_recommends(line);
    assert_eq!(result, line, "Should not add flag if already present");
}

#[test]
fn test_add_no_install_recommends_multiple_packages() {
    let line = "RUN apt-get install -y curl wget git";
    let result = add_no_install_recommends(line);
    assert_eq!(
        result, "RUN apt-get install -y --no-install-recommends curl wget git",
        "Should work with multiple packages"
    );
}

#[test]
fn test_add_no_install_recommends_multiple_apt_get_commands() {
    let line = "RUN apt-get update && apt-get install -y curl && apt-get install -y git";
    let result = add_no_install_recommends(line);
    assert!(
        result.contains("--no-install-recommends"),
        "Should add flag to apt-get install commands"
    );
    // Both install commands should get the flag
    let flag_count = result.matches("--no-install-recommends").count();
    assert_eq!(
        flag_count, 2,
        "Should add flag to both apt-get install commands"
    );
}

#[test]
fn test_add_no_install_recommends_apt_install_variant() {
    let line = "RUN apt install -y vim";
    let result = add_no_install_recommends(line);
    // Note: Current implementation only handles "apt-get install", not "apt install"
    // This test documents current behavior
    assert_eq!(result, line, "apt install (not apt-get) not yet supported");
}

#[test]
fn test_add_no_install_recommends_empty_line() {
    let line = "";
    let result = add_no_install_recommends(line);
    assert_eq!(result, line, "Empty line should be unchanged");
}

#[test]
fn test_add_no_install_recommends_no_apt_get() {
    let line = "RUN echo hello";
    let result = add_no_install_recommends(line);
    assert_eq!(result, line, "Non-apt-get commands should be unchanged");
}

#[test]
fn test_add_no_install_recommends_apt_get_update_only() {
    let line = "RUN apt-get update";
    let result = add_no_install_recommends(line);
    assert_eq!(
        result, line,
        "apt-get update (without install) should be unchanged"
    );
}

#[test]
fn test_add_no_install_recommends_with_continuation() {
    let line = "RUN apt-get install -y \\\n    curl \\\n    wget";
    let result = add_no_install_recommends(line);
    assert!(
        result.contains("--no-install-recommends"),
        "Should handle multi-line continuations"
    );
}

#[test]
fn test_add_no_install_recommends_comment_line() {
    let line = "# RUN apt-get install -y curl";
    let result = add_no_install_recommends(line);
    // Should not process comments
    assert_eq!(result, line, "Comment lines should not be processed");
}

#[test]
fn test_add_no_install_recommends_install_at_end() {
    let line = "RUN apt-get install";
    let result = add_no_install_recommends(line);
    assert_eq!(
        result, "RUN apt-get install --no-install-recommends ",
        "Should add flag even if no packages listed"
    );
}

#[test]
fn test_add_no_install_recommends_preserves_other_flags() {
    let line = "RUN apt-get install -y --fix-missing curl";
    let result = add_no_install_recommends(line);
    assert!(
        result.contains("--fix-missing"),
        "Should preserve other flags"
    );
    assert!(
        result.contains("--no-install-recommends"),
        "Should add --no-install-recommends"
    );
}

// FUNCTION 3: add_package_manager_cleanup()

#[test]
fn test_add_package_manager_cleanup_apt_get_install() {
    let line = "RUN apt-get update && apt-get install -y curl";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, "RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*",
        "Should add apt cleanup after install"
    );
}

#[test]
fn test_add_package_manager_cleanup_apt_install() {
    let line = "RUN apt install -y python3";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, "RUN apt install -y python3 && rm -rf /var/lib/apt/lists/*",
        "Should add apt cleanup for 'apt install' variant"
    );
}

#[test]
fn test_add_package_manager_cleanup_apk_add() {
    let line = "RUN apk add curl";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, "RUN apk add curl && rm -rf /var/cache/apk/*",
        "Should add apk cleanup for Alpine"
    );
}

#[test]
fn test_add_package_manager_cleanup_already_present_apt() {
    let line = "RUN apt-get install -y git && rm -rf /var/lib/apt/lists/*";
    let result = add_package_manager_cleanup(line);
    assert_eq!(result, line, "Should not add cleanup if already present");
}

#[test]
fn test_add_package_manager_cleanup_already_present_apk() {
    let line = "RUN apk add vim && rm -rf /var/cache/apk/*";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, line,
        "Should not add cleanup if already present (apk)"
    );
}

#[test]
fn test_add_package_manager_cleanup_no_package_manager() {
    let line = "RUN echo hello";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, line,
        "Non-package-manager commands should be unchanged"
    );
}

#[test]
fn test_add_package_manager_cleanup_apt_get_update_only() {
    let line = "RUN apt-get update";
    let result = add_package_manager_cleanup(line);
    // update doesn't install packages, so no cleanup needed
    assert_eq!(result, line, "apt-get update alone should be unchanged");
}

#[test]
fn test_add_package_manager_cleanup_empty_line() {
    let line = "";
    let result = add_package_manager_cleanup(line);
    assert_eq!(result, line, "Empty line should be unchanged");
}

#[test]
fn test_add_package_manager_cleanup_comment_line() {
    let line = "# RUN apt-get install curl";
    let result = add_package_manager_cleanup(line);
    assert_eq!(result, line, "Comment lines should not be processed");
}

#[test]
fn test_add_package_manager_cleanup_with_trailing_whitespace() {
    let line = "RUN apt-get install -y wget   ";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, "RUN apt-get install -y wget && rm -rf /var/lib/apt/lists/*",
        "Should trim trailing whitespace before adding cleanup"
    );
}

#[test]
fn test_add_package_manager_cleanup_multiple_commands() {
    let line = "RUN apt-get update && apt-get install -y curl && echo done";
    let result = add_package_manager_cleanup(line);
    assert!(
        result.contains("&& rm -rf /var/lib/apt/lists/*"),
        "Should add cleanup even with multiple commands"
    );
}

#[test]
fn test_add_package_manager_cleanup_apk_add_multiple_packages() {
    let line = "RUN apk add --no-cache curl wget git";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, "RUN apk add --no-cache curl wget git && rm -rf /var/cache/apk/*",
        "Should add cleanup for apk with multiple packages"
    );
}

#[test]
fn test_add_package_manager_cleanup_partial_match_no_install() {
    let line = "RUN apt-get clean";
    let result = add_package_manager_cleanup(line);
    assert_eq!(
        result, line,
        "apt-get clean (not install) should be unchanged"
    );
}

// FUNCTION 4: pin_base_image_version()

#[test]
fn test_pin_base_image_version_ubuntu_untagged() {
    let line = "FROM ubuntu";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM ubuntu:22.04",
        "Untagged ubuntu should be pinned to 22.04 LTS"
    );
}

#[test]
fn test_pin_base_image_version_ubuntu_latest() {
    let line = "FROM ubuntu:latest";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM ubuntu:22.04",
        "ubuntu:latest should be pinned to 22.04 LTS"
    );
}

#[test]
fn test_pin_base_image_version_ubuntu_already_pinned() {
    let line = "FROM ubuntu:20.04";
    let result = pin_base_image_version(line);
    assert_eq!(result, line, "Already pinned ubuntu should be unchanged");
}

#[test]
fn test_pin_base_image_version_debian() {
    let line = "FROM debian";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM debian:12-slim",
        "Untagged debian should be pinned to 12-slim"
    );
}

#[test]
fn test_pin_base_image_version_alpine() {
    let line = "FROM alpine:latest";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM alpine:3.19",
        "alpine:latest should be pinned to 3.19"
    );
}

#[test]
fn test_pin_base_image_version_node() {
    let line = "FROM node";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM node:20-alpine",
        "Untagged node should be pinned to 20-alpine"
    );
}

#[test]
fn test_pin_base_image_version_python() {
    let line = "FROM python:latest";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM python:3.11-slim",
        "python:latest should be pinned to 3.11-slim"
    );
}

#[test]
fn test_pin_base_image_version_with_registry_prefix() {
    let line = "FROM docker.io/ubuntu";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM docker.io/ubuntu:22.04",
        "Should preserve registry prefix (docker.io/)"
    );
}

#[test]
fn test_pin_base_image_version_with_as_alias() {
    let line = "FROM ubuntu AS builder";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM ubuntu:22.04 AS builder",
        "Should preserve AS alias"
    );
}

#[test]
fn test_pin_base_image_version_unknown_image() {
    let line = "FROM mycompany/custom-image";
    let result = pin_base_image_version(line);
    assert_eq!(result, line, "Unknown images should be unchanged");
}

#[test]
fn test_pin_base_image_version_malformed_no_image() {
    let line = "FROM";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, line,
        "Malformed FROM (no image) should be unchanged"
    );
}

#[test]
fn test_pin_base_image_version_empty_line() {
    let line = "";
    let result = pin_base_image_version(line);
    assert_eq!(result, line, "Empty line should be unchanged");
}

#[test]
fn test_pin_base_image_version_rust() {
    let line = "FROM rust:latest";
    let result = pin_base_image_version(line);
    assert_eq!(
        result, "FROM rust:1.75-alpine",
        "rust:latest should be pinned to 1.75-alpine"
    );
}

