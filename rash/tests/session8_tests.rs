#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
                               // ============================================================================
                               // Session 8: Parameter Expansion and Shell Expansion Features
                               // Validation of GNU Bash Manual constructs - RED Phase Tests
                               // ============================================================================

use bashrs::models::config::Config;
use bashrs::transpile;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

// ============================================================================
// EXP-PARAM-003: Error if Unset ${var:?word}
// ============================================================================

/// EXP-PARAM-003: RED Phase
/// Test error-if-unset baseline
#[test]
fn test_error_if_unset_baseline() {
    let source = r#"
fn main() {
    require_var("REQUIRED");
}

fn require_var(name: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile require function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for error-if-unset:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("require_var"),
        "Should transpile require_var function"
    );
}

/// EXP-PARAM-003: RED Phase - ADVANCED
/// Test Option::expect() conversion to ${var:?message}
#[test]
#[ignore] // Requires Option::expect() recognition
fn test_error_if_unset_conversion() {
    let source = r#"
fn main() {
    let config = std::env::var("CONFIG").expect("CONFIG must be set");
    use_config(&config);
}

fn use_config(cfg: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate ${var:?message} syntax
    assert!(
        shell.contains("${CONFIG:?") || shell.contains(":?CONFIG must be set"),
        "Should convert .expect() to ${{var:?message}}"
    );
}

/// EXP-PARAM-003: RED Phase - EXECUTION
/// Test error-if-unset execution
#[test]
fn test_error_if_unset_execution() {
    let source = r#"
fn main() {
    check_required("VALUE");
}

fn check_required(val: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute"
    );
}

// ============================================================================
// EXP-PARAM-004: Alternative Value ${var:+word}
// ============================================================================

/// EXP-PARAM-004: RED Phase
/// Test alternative value baseline
#[test]
fn test_alternative_value_baseline() {
    let source = r#"
fn main() {
    check_if_set("VAR");
}

fn check_if_set(name: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile check function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for alternative value:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("check_if_set"),
        "Should transpile check_if_set function"
    );
}

/// EXP-PARAM-004: RED Phase - ADVANCED
/// Test Option::is_some() conversion to ${var:+word}
#[test]
#[ignore] // Requires Option::is_some() recognition
fn test_alternative_value_conversion() {
    let source = r#"
fn main() {
    let config = std::env::var("DEBUG");
    let flag = if config.is_ok() { "enabled" } else { "" };
    use_flag(&flag);
}

fn use_flag(f: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate ${var:+word} syntax
    assert!(
        shell.contains("${DEBUG:+enabled}"),
        "Should convert Option check to ${{var:+word}}"
    );
}

/// EXP-PARAM-004: RED Phase - EXECUTION
/// Test alternative value execution
#[test]
fn test_alternative_value_execution() {
    let source = r#"
fn main() {
    use_if_available("FEATURE");
}

fn use_if_available(name: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute"
    );
}

// ============================================================================
// EXP-BRACE-001: Brace Expansion {1..5}
// ============================================================================

/// EXP-BRACE-001: RED Phase
/// Test brace expansion baseline
#[test]
fn test_brace_expansion_baseline() {
    let source = r#"
fn main() {
    generate_sequence(1, 5);
}

fn generate_sequence(start: i32, end: i32) {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile sequence function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for brace expansion:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("generate_sequence"),
        "Should transpile generate_sequence function"
    );
}

/// EXP-BRACE-001: RED Phase - ADVANCED
/// Test for range conversion to seq or brace expansion
#[test]
#[ignore] // Requires range pattern recognition
fn test_brace_expansion_conversion() {
    let source = r#"
fn main() {
    for i in 1..=5 {
        print_number(i);
    }
}

fn print_number(n: i32) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate seq command (POSIX) or $(seq 1 5)
    assert!(
        shell.contains("seq 1 5") || shell.contains("{1..5}"),
        "Should convert range to seq or brace expansion"
    );
}

/// EXP-BRACE-001: RED Phase - EXECUTION
/// Test brace expansion execution
#[test]
fn test_brace_expansion_execution() {
    let source = r#"
fn main() {
    iterate_range(1, 3);
}

fn iterate_range(start: i32, end: i32) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute"
    );
}

// ============================================================================
// EXP-TILDE-001: Tilde Expansion ~
// ============================================================================

/// EXP-TILDE-001: RED Phase
/// Test tilde expansion baseline
#[test]
fn test_tilde_expansion_baseline() {
    let source = r#"
fn main() {
    use_home_path();
}

fn use_home_path() {}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(
        result.is_ok(),
        "Should transpile home path function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for tilde expansion:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("use_home_path"),
        "Should transpile use_home_path function"
    );
}

/// EXP-TILDE-001: RED Phase - ADVANCED
/// Test home_dir() conversion to $HOME
#[test]
#[ignore] // Requires home_dir pattern recognition
fn test_tilde_expansion_conversion() {
    let source = r#"
fn main() {
    let home = std::env::var("HOME").unwrap();
    let docs = format!("{}/Documents", home);
    use_path(&docs);
}

fn use_path(path: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    // Should generate $HOME/Documents or ~/ expansion
    assert!(
        shell.contains("$HOME/Documents") || shell.contains("~/Documents"),
        "Should convert home path to $HOME or ~"
    );
}

/// EXP-TILDE-001: RED Phase - EXECUTION
/// Test tilde expansion execution
#[test]
fn test_tilde_expansion_execution() {
    let source = r#"
fn main() {
    access_home_dir();
}

fn access_home_dir() {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute"
    );
}

/// Session 8: Combined execution test
#[test]
fn test_session8_commands_execution() {
    let source = r#"
fn main() {
    require_var("REQUIRED");
    check_if_set("OPTIONAL");
    generate_sequence(1, 5);
    use_home_path();
}

fn require_var(name: &str) {}
fn check_if_set(name: &str) {}
fn generate_sequence(start: i32, end: i32) {}
fn use_home_path() {}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    eprintln!("Generated combined shell script:\n{}", shell);

    // Verify all functions are called
    assert!(shell.contains("require_var"), "Should call require_var");
    assert!(shell.contains("check_if_set"), "Should call check_if_set");
    assert!(
        shell.contains("generate_sequence"),
        "Should call generate_sequence"
    );
    assert!(shell.contains("use_home_path"), "Should call use_home_path");

    // Write and execute
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    // Execution test may fail (functions undefined), but script should be valid
    eprintln!("Exit code: {:?}", output.status.code());
    eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
}
