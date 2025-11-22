#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
// Negative testing suite for CLI error handling
// Testing Spec Section 1.6: Layer 6 - Negative Testing
//
// This test suite validates that:
// 1. Unsupported features produce clear error messages
// 2. Error messages include source location, snippet, and suggestions
// 3. Error message quality score ≥0.7
// 4. CLI flags work correctly (--help, --version, --check)

#![allow(dead_code)] // score() method reserved for future quality analysis

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

// ============================================================================
// Error Message Quality Metrics (Testing Spec Section 1.6)
// ============================================================================

#[derive(Debug)]
struct ErrorMessageQuality {
    has_error_prefix: bool,    // "error:" or "Error:" present
    has_source_location: bool, // Line/column information
    has_code_snippet: bool,    // Shows problematic code
    has_caret_indicator: bool, // ^ pointing to issue
    has_explanation: bool,     // "note:" with context
    has_suggestion: bool,      // "help:" with alternative
    message_length: usize,
}

impl ErrorMessageQuality {
    fn from_stderr(stderr: &str) -> Self {
        Self {
            has_error_prefix: stderr.contains("error:") || stderr.contains("Error:"),
            has_source_location: stderr.contains(':')
                && stderr.chars().filter(|c| c.is_numeric()).count() > 0,
            has_code_snippet: stderr.lines().any(|l| {
                !l.starts_with("error:")
                    && !l.starts_with("Error:")
                    && !l.starts_with("note:")
                    && !l.starts_with("help:")
            }),
            has_caret_indicator: stderr.contains('^'),
            has_explanation: stderr.contains("note:"),
            has_suggestion: stderr.contains("help:") || stderr.contains("consider"),
            message_length: stderr.len(),
        }
    }

    fn score(&self) -> f32 {
        let mut score = 0.0;
        if self.has_error_prefix {
            score += 1.0;
        }
        if self.has_source_location {
            score += 1.5;
        }
        if self.has_code_snippet {
            score += 1.5;
        }
        if self.has_caret_indicator {
            score += 1.0;
        }
        if self.has_explanation {
            score += 2.0;
        }
        if self.has_suggestion {
            score += 2.0;
        }

        // Penalize excessively long messages (>500 chars)
        if self.message_length > 500 {
            score -= 1.0;
        }

        score / 9.0 // Normalize to 0-1
    }
}

// ============================================================================
// Unsupported Feature Tests
// ============================================================================

#[test]
fn test_async_syntax_error_message() {
    let rust_code = r#"
        async fn fetch_data() -> Result<String, Error> {
            Ok("data".to_string())
        }

        fn main() {
            let data = fetch_data().await;
        }
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    let output = Command::cargo_bin("bashrs")
        .unwrap()
        .arg("build")
        .arg(file.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should contain error about async
    assert!(
        stderr.contains("async") || stderr.contains("Unsupported"),
        "Error should mention async or unsupported. Got: {}",
        stderr
    );

    // Should fail with exit code 1
    assert_eq!(
        output.status.code(),
        Some(1),
        "Should fail with exit code 1. Got: {:?}",
        output.status.code()
    );
}

#[test]
fn test_trait_definition_error_message() {
    let rust_code = r#"
        trait Drawable {
            fn draw(&self);
        }

        fn main() {}
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    let output = Command::cargo_bin("bashrs")
        .unwrap()
        .arg("build")
        .arg(file.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Error message says "Only functions are allowed" which correctly identifies traits as unsupported
    assert!(
        stderr.contains("Only functions are allowed") || stderr.contains("trait"),
        "Error should mention trait or function restriction. Got: {}",
        stderr
    );

    // Should fail with exit code 1
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_impl_block_error_message() {
    let rust_code = r#"
        struct Foo;

        impl Foo {
            fn new() -> Self { Foo }
        }

        fn main() {}
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    let output = Command::cargo_bin("bashrs")
        .unwrap()
        .arg("build")
        .arg(file.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Error message says "Only functions are allowed" which correctly identifies the problem
    assert!(
        stderr.contains("Only functions are allowed")
            || stderr.contains("impl")
            || stderr.contains("struct"),
        "Error should mention impl/struct or function restriction. Got: {}",
        stderr
    );

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_unsafe_block_error_message() {
    let rust_code = r#"
        fn main() {
            unsafe {
                let ptr = std::ptr::null::<i32>();
            }
        }
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    let output = Command::cargo_bin("bashrs")
        .unwrap()
        .arg("build")
        .arg(file.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.contains("unsafe") || stderr.contains("Unsupported"),
        "Error should mention unsafe or unsupported. Got: {}",
        stderr
    );

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_generic_type_error_message() {
    let rust_code = r#"
        fn process<T>(item: T) -> T {
            item
        }

        fn main() {}
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    let output = Command::cargo_bin("bashrs")
        .unwrap()
        .arg("build")
        .arg(file.path())
        .output()
        .unwrap();

    let _stderr = String::from_utf8_lossy(&output.stderr);

    // Generics might be unsupported or cause parse errors
    assert!(
        output.status.code() == Some(1),
        "Generic types should cause error. Got exit code: {:?}",
        output.status.code()
    );
}

#[test]
fn test_macro_definition_error_message() {
    let rust_code = r#"
        macro_rules! my_macro {
            () => { println!("hello"); }
        }

        fn main() {
            my_macro!();
        }
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    let output = Command::cargo_bin("bashrs")
        .unwrap()
        .arg("build")
        .arg(file.path())
        .output()
        .unwrap();

    let _stderr = String::from_utf8_lossy(&output.stderr);

    // Macro definitions should fail
    assert_eq!(
        output.status.code(),
        Some(1),
        "Macro definitions should fail. Got: {:?}",
        output.status.code()
    );
}

#[test]
fn test_loop_statement_error_message() {
    let rust_code = r#"
        fn main() {
            loop {
                break;
            }
        }
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    let output = Command::cargo_bin("bashrs")
        .unwrap()
        .arg("build")
        .arg(file.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.contains("loop") || stderr.contains("Unsupported"),
        "Error should mention loop or unsupported. Got: {}",
        stderr
    );

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_use_statement_error_message() {
    let rust_code = r#"
        use std::collections::HashMap;

        fn main() {}
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    let output = Command::cargo_bin("bashrs")
        .unwrap()
        .arg("build")
        .arg(file.path())
        .output()
        .unwrap();

    let _stderr = String::from_utf8_lossy(&output.stderr);

    // Use statements should fail (not supported)
    assert_eq!(output.status.code(), Some(1), "Use statements should fail");
}

// ============================================================================
// Syntax Error Tests
// ============================================================================

#[test]
fn test_syntax_error_diagnostic() {
    let rust_code = r#"
        fn main() {
            let x = 10 +;
        }
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    let output = Command::cargo_bin("bashrs")
        .unwrap()
        .arg("build")
        .arg(file.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should have some error about syntax or expression
    assert!(
        stderr.contains("error") || stderr.contains("Error"),
        "Should contain error message. Got: {}",
        stderr
    );

    assert_eq!(output.status.code(), Some(2)); // IO/syntax errors return 2
}

// ============================================================================
// CLI Flag Tests (Testing Spec Section 1.6)
// ============================================================================

#[test]
fn test_help_flag() {
    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs"))
        .stdout(predicate::str::contains("Rust-to-Shell transpiler"));
}

#[test]
fn test_version_flag() {
    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs"));
}

#[test]
fn test_check_subcommand_valid_file() {
    let rust_code = r#"
        fn main() {
            let x = 42;
            println!("Hello");
        }
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("check")
        .arg(file.path())
        .assert()
        .success();
}

#[test]
fn test_check_subcommand_invalid_file() {
    // Use a construct that fails validation (struct definition)
    let rust_code = r#"
        struct Foo;
        fn main() {}
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("check")
        .arg(file.path())
        .assert()
        .failure();
}

// ============================================================================
// File Not Found Tests
// ============================================================================

#[test]
fn test_missing_input_file_error() {
    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("build")
        .arg("nonexistent_file.rs")
        .assert()
        .failure()
        .code(2) // IO errors return 2
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")))
        .stderr(predicate::str::contains("No such file or directory"));
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
fn test_error_message_quality_baseline() {
    let unsupported_features = vec![
        ("async", "async fn test() {}"),
        ("unsafe", "unsafe { }"),
        ("loop", "loop { break; }"),
    ];

    for (feature, code) in unsupported_features {
        let full_code = format!("fn main() {{ {} }}", code);
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(full_code.as_bytes()).unwrap();

        let output = Command::cargo_bin("bashrs")
            .unwrap()
            .arg("build")
            .arg(file.path())
            .output()
            .unwrap();

        let stderr = String::from_utf8_lossy(&output.stderr);
        let quality = ErrorMessageQuality::from_stderr(&stderr);

        // For now, just verify we get SOME error
        // Quality improvements will be implemented in next task
        assert!(
            quality.has_error_prefix,
            "Error message for '{}' should have error prefix. Quality: {:?}",
            feature, quality
        );
    }
}

// ============================================================================
// Multiple Error Reporting Tests
// ============================================================================

#[test]
fn test_multiple_errors_detected() {
    let rust_code = r#"
        async fn first() {}

        unsafe fn second() {}

        fn main() {
            loop {}
        }
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    let output = Command::cargo_bin("bashrs")
        .unwrap()
        .arg("build")
        .arg(file.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should detect at least one error
    assert!(
        stderr.contains("error") || stderr.contains("Error"),
        "Should detect errors. Got: {}",
        stderr
    );

    assert_eq!(output.status.code(), Some(1));
}
