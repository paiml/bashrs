#![allow(deprecated)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
// Tests can use unwrap() for simplicity
// CLI Integration Tests for Rash v2.0.0
// Sprint 73 Phase 3: CLI Integration Tests
//
// Tests all CLI commands following EXTREME TDD methodology:
// - Uses assert_cmd (MANDATORY per CLAUDE.md)
// - Tests success cases, error cases, and edge cases
// - Validates all CLI commands and subcommands
// - End-to-end workflow testing
#![allow(non_snake_case)] // Test naming convention: test_<TASK_ID>_<feature>_<scenario>

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a bashrs command (MANDATORY pattern per CLAUDE.md)
#[allow(deprecated)]
fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

/// Create a temporary file carrying the extension the tool dispatches on.
///
/// The suffix is load-bearing: these helpers used to create extensionless temp
/// files, so `bashrs lint` could not tell Rust from shell and linted a `.rs`
/// fixture with SHELL rules — `let x = 42;` came back as SC1068 "don't put
/// spaces around the = in 'let' assignments".
fn create_temp_file_with_suffix(content: &str, suffix: &str) -> NamedTempFile {
    let mut file = tempfile::Builder::new()
        .suffix(suffix)
        .tempfile()
        .expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");
    file
}

/// Create a temporary Rust file with given content
fn create_temp_rust_file(content: &str) -> NamedTempFile {
    create_temp_file_with_suffix(content, ".rs")
}

/// Create a temporary shell script with given content
fn create_temp_shell_file(content: &str) -> NamedTempFile {
    create_temp_file_with_suffix(content, ".sh")
}

/// Create a temporary Makefile with given content
fn create_temp_makefile(content: &str) -> NamedTempFile {
    create_temp_file_with_suffix(content, ".mk")
}

// ============================================================================
// Test: CLI_001 - Help and Version Commands
// ============================================================================

#[test]
fn test_CLI_001_help_flag() {
    bashrs_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs"))
        .stdout(predicate::str::contains("Rust-to-Shell transpiler"));
}

#[test]
fn test_CLI_001_version_flag() {
    bashrs_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs"));
}

#[test]
fn test_CLI_001_help_subcommand() {
    bashrs_cmd()
        .arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs"))
        .stdout(predicate::str::contains("Commands:"));
}

// ============================================================================
// Test: CLI_002 - Build Command (Rust → Shell Transpilation)
// ============================================================================

#[test]
fn test_CLI_002_build_basic() {
    let rust_code = r#"
fn main() {
    let message = "Hello, World!";
    println!(message);
}
"#;

    let input_file = create_temp_rust_file(rust_code);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("output.sh");

    bashrs_cmd()
        .arg("build")
        .arg(input_file.path())
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();

    // Verify output file was created
    assert!(output_file.exists(), "Output file should exist");

    // Verify output contains sh shebang
    let output_content = fs::read_to_string(&output_file).expect("Failed to read output");
    assert!(
        output_content.contains("#!/bin/sh") || output_content.contains("#!/bin/dash"),
        "Output should have POSIX shebang"
    );
}

#[test]
fn test_CLI_002_build_invalid_rust() {
    let invalid_rust = r#"
fn main() {
    let x = 10 +;  // Syntax error
}
"#;

    let input_file = create_temp_rust_file(invalid_rust);

    bashrs_cmd()
        .arg("build")
        .arg(input_file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")));
}

#[test]
fn test_CLI_002_build_nonexistent_file() {
    bashrs_cmd()
        .arg("build")
        .arg("nonexistent_file.rs")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file"));
}

#[test]
fn test_CLI_002_build_with_emit_proof() {
    let rust_code = r#"
fn main() {
    let x = 42;
}
"#;

    let input_file = create_temp_rust_file(rust_code);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("output.sh");

    bashrs_cmd()
        .arg("build")
        .arg(input_file.path())
        .arg("--output")
        .arg(&output_file)
        .arg("--emit-proof")
        .assert()
        .success();
}

// ============================================================================
// Test: CLI_003 - Check Command
// ============================================================================

#[test]
fn test_CLI_003_check_valid_rust() {
    let rust_code = r#"
fn main() {
    let x = 42;
    println!(x);
}
"#;

    let input_file = create_temp_rust_file(rust_code);

    bashrs_cmd()
        .arg("check")
        .arg(input_file.path())
        .assert()
        .success();
}

#[test]
fn test_CLI_003_check_invalid_rust() {
    let invalid_rust = r#"
fn main() {
    unsafe { let ptr = std::ptr::null::<i32>(); }
}
"#;

    let input_file = create_temp_rust_file(invalid_rust);

    bashrs_cmd()
        .arg("check")
        .arg(input_file.path())
        .assert()
        .failure();
}

#[test]
fn test_CLI_003_check_nonexistent_file() {
    bashrs_cmd()
        .arg("check")
        .arg("nonexistent.rs")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file"));
}

// ============================================================================
// Test: CLI_004 - Init Command
// ============================================================================

#[test]
fn test_CLI_004_init_new_project() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    bashrs_cmd()
        .arg("init")
        .arg(temp_dir.path())
        .arg("--name")
        .arg("test_project")
        .assert()
        .success();

    // Verify project structure was created
    // Note: Exact files depend on init implementation
}

#[test]
fn test_CLI_004_init_current_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // `init` requires an explicit PATH — its own help says so: "(required —
    // prevents accidental CWD mutation)". Asserting that a bare `init`
    // succeeds asserted the absence of that safety property.
    bashrs_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .assert()
        .failure();

    bashrs_cmd()
        .current_dir(temp_dir.path())
        .arg("init")
        .arg("proj")
        .assert()
        .success();
}

// ============================================================================
// Test: CLI_005 - Verify Command
// ============================================================================

#[test]
fn test_CLI_005_verify_matching_files() {
    let rust_code = r#"
fn main() {
    println!("Hello");
}
"#;

    let rust_file = create_temp_rust_file(rust_code);
    let shell_file = create_temp_shell_file("");

    // `verify` compares the source against what the transpiler emits, so the
    // shell side has to BE that emission. Hand-writing it and hoping for a
    // match is why this carried a "may fail depending on the implementation"
    // note; transpile-then-verify is the property actually worth asserting.
    bashrs_cmd()
        .arg("build")
        .arg(rust_file.path())
        .arg("-o")
        .arg(shell_file.path())
        .assert()
        .success();

    bashrs_cmd()
        .arg("verify")
        .arg(rust_file.path())
        .arg(shell_file.path())
        .assert()
        .success();
}

#[test]
fn test_CLI_005_verify_nonexistent_rust() {
    let shell_script = "#!/bin/sh\necho hello\n";
    let shell_file = create_temp_shell_file(shell_script);

    bashrs_cmd()
        .arg("verify")
        .arg("nonexistent.rs")
        .arg(shell_file.path())
        .assert()
        .failure();
}

// ============================================================================
// Test: CLI_006 - Inspect Command
// ============================================================================

#[test]
fn test_CLI_006_inspect_ast_json() {
    // The real TinyAst schema is an externally-tagged enum; `{"type":"Program"}`
    // was never a shape this tool accepted.
    let ast_json =
        r#"{"Sequence":{"commands":[{"ExecuteCommand":{"command_name":"echo","args":["hi"]}}]}}"#;

    bashrs_cmd()
        .arg("inspect")
        .arg(ast_json)
        .arg("--format")
        .arg("json")
        .assert()
        .success();
}

#[test]
fn test_CLI_006_inspect_markdown_output() {
    // The real TinyAst schema is an externally-tagged enum; `{"type":"Program"}`
    // was never a shape this tool accepted.
    let ast_json =
        r#"{"Sequence":{"commands":[{"ExecuteCommand":{"command_name":"echo","args":["hi"]}}]}}"#;

    bashrs_cmd()
        .arg("inspect")
        .arg(ast_json)
        .arg("--format")
        .arg("markdown")
        .assert()
        .success();
}

#[test]
fn test_CLI_006_inspect_with_detailed_traces() {
    // The real TinyAst schema is an externally-tagged enum; `{"type":"Program"}`
    // was never a shape this tool accepted.
    let ast_json =
        r#"{"Sequence":{"commands":[{"ExecuteCommand":{"command_name":"echo","args":["hi"]}}]}}"#;

    bashrs_cmd()
        .arg("inspect")
        .arg(ast_json)
        .arg("--detailed")
        .assert()
        .success();
}

// ============================================================================
// Test: CLI_007 - Compile Command
// ============================================================================

#[test]
fn test_CLI_007_compile_to_binary() {
    let rust_code = r#"
fn main() {
    println!("Hello");
}
"#;

    let input_file = create_temp_rust_file(rust_code);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("binary");

    bashrs_cmd()
        .arg("compile")
        .arg(input_file.path())
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();
}

include!("fragments/cli_integration_tests_CLI_2.rs");
