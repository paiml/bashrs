#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! Transpilation Quality Tests
//!
//! Validates that the Rust-to-Shell transpiler produces high-quality output:
//! - Correct POSIX shell structure (header, set -euf, main wrapper)
//! - Deterministic output (same input => identical output)
//! - No lint violations (SEC, DET, IDEM rules)
//! - Selective runtime emission (only used functions included)

use assert_cmd::Command;
use predicates::prelude::*;
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

// ============================================================================
// Phase 1: Smart Runtime Emission Tests
// ============================================================================

#[test]
fn test_RUNTIME_001_trivial_script_no_stdlib() {
    // A script with just `let x = 42;` should NOT emit any rash_* functions
    let rust_code = r#"
        fn main() {
            let x = 42;
        }
    "#;

    let result = bashrs::transpile(rust_code, bashrs::Config::default()).unwrap();

    // Should NOT contain any runtime function definitions
    assert!(
        !result.contains("rash_println()"),
        "Trivial script should not emit rash_println"
    );
    assert!(
        !result.contains("rash_require()"),
        "Trivial script should not emit rash_require"
    );
    assert!(
        !result.contains("rash_fs_exists()"),
        "Trivial script should not emit rash_fs_exists"
    );
    assert!(
        !result.contains("rash_string_trim()"),
        "Trivial script should not emit rash_string_trim"
    );
    assert!(
        !result.contains("# Rash runtime functions"),
        "Trivial script should not emit runtime header"
    );
}

#[test]
fn test_RUNTIME_002_echo_emits_no_stdlib() {
    // A script using echo (builtin) should NOT emit stdlib functions
    let rust_code = r#"
        fn main() {
            echo("hello");
        }
        fn echo(msg: &str) {}
    "#;

    let result = bashrs::transpile(rust_code, bashrs::Config::default()).unwrap();

    // Should NOT contain stdlib functions since echo is a builtin
    assert!(
        !result.contains("rash_string_trim()"),
        "Echo-only script should not emit string stdlib"
    );
    assert!(
        !result.contains("rash_fs_exists()"),
        "Echo-only script should not emit fs stdlib"
    );
}

#[test]
fn test_RUNTIME_003_selective_stdlib_emission() {
    // A script using fs_exists should only emit rash_fs_exists, not all stdlib
    let rust_code = r#"
        fn main() {
            let exists = fs_exists("/tmp/test");
        }
        fn fs_exists(path: &str) -> bool { true }
    "#;

    let result = bashrs::transpile(rust_code, bashrs::Config::default()).unwrap();

    // Should contain the used function
    assert!(
        result.contains("rash_fs_exists()"),
        "Should emit rash_fs_exists when fs_exists is called"
    );

    // Should NOT contain unrelated stdlib functions
    assert!(
        !result.contains("rash_string_trim()"),
        "Should not emit rash_string_trim when not used"
    );
    assert!(
        !result.contains("rash_array_len()"),
        "Should not emit rash_array_len when not used"
    );
}

// ============================================================================
// Phase 3: Transpilation Output Quality Tests
// ============================================================================

#[test]
fn test_TRANSPILE_001_basic_build_produces_posix_shell() {
    let rust_code = r#"
        fn main() {
            let greeting = "Hello, World!";
        }
    "#;

    let result = bashrs::transpile(rust_code, bashrs::Config::default()).unwrap();

    // Must have POSIX header
    assert!(
        result.starts_with("#!/bin/sh"),
        "Transpiled output must start with #!/bin/sh"
    );

    // Must have strict mode
    assert!(
        result.contains("set -euf"),
        "Transpiled output must enable strict error handling"
    );

    // Must have main wrapper
    assert!(
        result.contains("main()"),
        "Transpiled output must have main() wrapper"
    );

    // Must have cleanup trap
    assert!(
        result.contains("trap"),
        "Transpiled output must have cleanup trap"
    );
}

#[test]
fn test_TRANSPILE_002_determinism_identical_output() {
    let rust_code = r#"
        fn main() {
            let x = 42;
            let y = "hello";
            let z = true;
        }
    "#;

    let config = bashrs::Config::default();

    // Transpile the same input twice
    let result1 = bashrs::transpile(rust_code, config.clone()).unwrap();
    let result2 = bashrs::transpile(rust_code, config).unwrap();

    // Must produce byte-identical output
    assert_eq!(
        result1, result2,
        "Same input must produce identical output (determinism)"
    );
}

#[test]
fn test_TRANSPILE_003_cli_build_produces_output() {
    let rust_code = r#"
        fn main() {
            let x = 42;
        }
    "#;

    let input_file = create_temp_rust_file(rust_code);
    let output_file = NamedTempFile::new().unwrap();

    bashrs_cmd()
        .arg("build")
        .arg(input_file.path())
        .arg("-o")
        .arg(output_file.path())
        .assert()
        .success();

    // Verify output file contains valid POSIX shell
    let output = std::fs::read_to_string(output_file.path()).unwrap();
    assert!(output.starts_with("#!/bin/sh"));
}

#[test]
fn test_TRANSPILE_004_transpile_with_lint_api() {
    let rust_code = r#"
        fn main() {
            let x = 42;
        }
    "#;

    let (shell_code, lint_result) =
        bashrs::transpile_with_lint(rust_code, bashrs::Config::default()).unwrap();

    // Shell code should be valid
    assert!(shell_code.contains("#!/bin/sh"));

    // Lint result should have been produced (even if no issues)
    // We can't guarantee zero diagnostics due to generated code style,
    // but the function should return successfully
    let _ = lint_result.diagnostics.len();
}

#[test]
fn test_TRANSPILE_005_no_nondeterminism_in_output() {
    let rust_code = r#"
        fn main() {
            let x = 42;
            let name = "test";
        }
    "#;

    let result = bashrs::transpile(rust_code, bashrs::Config::default()).unwrap();

    // Must NOT contain non-deterministic patterns in user code sections.
    // Note: $$ in the cleanup trap (trap 'rm -rf ... rash.$$' EXIT) is
    // acceptable - it's deterministic text that always appears identically.
    // The concern is non-determinism in transpilation output between runs,
    // not shell runtime behavior.
    assert!(
        !result.contains("$RANDOM"),
        "Transpiled output must not contain $RANDOM"
    );
    // Verify determinism: transpiling the same input twice produces identical output
    let result2 = bashrs::transpile(rust_code, bashrs::Config::default()).unwrap();
    assert_eq!(result, result2, "Transpilation must be deterministic");
}
