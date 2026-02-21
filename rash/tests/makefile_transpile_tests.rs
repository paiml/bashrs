#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! Makefile Transpilation Integration Tests
//!
//! Tests the Rust DSL -> Makefile transpilation pipeline.
//! Uses assert_cmd for CLI testing (MANDATORY per CLAUDE.md).

use assert_cmd::Command;
use std::io::Write;
use tempfile::NamedTempFile;

/// Create a bashrs command (MANDATORY pattern per CLAUDE.md)
#[allow(deprecated)]
fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

/// Create a temporary Rust file with given content
fn create_temp_rust_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");
    file
}

#[test]
fn test_MAKE_BUILD_001_basic_generation() {
    // Test that transpile_makefile produces valid output with variables
    let rust_code = r#"
        fn main() {
            let cc = "gcc";
            let cflags = "-O2 -Wall";
        }
    "#;

    let result = bashrs::transpile_makefile(rust_code, bashrs::Config::default()).unwrap();
    assert!(
        result.contains("CC := gcc"),
        "Expected 'CC := gcc' in generated Makefile: {}",
        result
    );
    assert!(
        result.contains("CFLAGS := -O2 -Wall"),
        "Expected 'CFLAGS := -O2 -Wall' in generated Makefile"
    );
}

#[test]
fn test_MAKE_BUILD_002_variables_uppercase() {
    // Variable names should be uppercased
    let rust_code = r#"
        fn main() {
            let my_var = "value";
        }
    "#;

    let result = bashrs::transpile_makefile(rust_code, bashrs::Config::default()).unwrap();
    assert!(
        result.contains("MY_VAR :="),
        "Variable names should be uppercased: {}",
        result
    );
}

#[test]
fn test_MAKE_BUILD_003_cli_make_build() {
    // Test CLI: bashrs make build <file.rs> -o Makefile
    let rust_code = r#"
        fn main() {
            let cc = "gcc";
        }
    "#;

    let input_file = create_temp_rust_file(rust_code);
    let output_file = NamedTempFile::new().unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("build")
        .arg(input_file.path())
        .arg("-o")
        .arg(output_file.path())
        .assert()
        .success();

    // Verify output file contains a Makefile
    let output = std::fs::read_to_string(output_file.path()).unwrap();
    assert!(
        output.contains("CC := gcc"),
        "CLI should produce valid Makefile: {}",
        output
    );
}

#[test]
fn test_MAKE_BUILD_004_determinism() {
    let rust_code = r#"
        fn main() {
            let cc = "gcc";
            let cflags = "-O2";
        }
    "#;

    let config = bashrs::Config::default();
    let result1 = bashrs::transpile_makefile(rust_code, config.clone()).unwrap();
    let result2 = bashrs::transpile_makefile(rust_code, config).unwrap();
    assert_eq!(
        result1, result2,
        "Makefile transpilation must be deterministic"
    );
}
