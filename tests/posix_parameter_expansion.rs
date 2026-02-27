//! POSIX Parameter Expansion Compliance Tests
//!
//! Tests bashrs compliance with POSIX Shell Command Language specification
//! for parameter expansion patterns (Section 2.6.2).
//!
//! References:
//! - POSIX spec: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_06_02
//! - Google Shell Style Guide: https://google.github.io/styleguide/shellguide.html
//! - ShellCheck: https://www.shellcheck.net/

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to create temporary Rust source file
fn create_rust_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let file_path = dir.path().join(name);
    fs::write(&file_path, content).unwrap();
    file_path
}

/// Helper to run bashrs build and return generated shell script
fn transpile(rust_code: &str) -> String {
    let temp_dir = TempDir::new().unwrap();
    let input = create_rust_file(&temp_dir, "test.rs", rust_code);
    let output = temp_dir.path().join("test.sh");

    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("build")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .assert()
        .success();

    fs::read_to_string(output).unwrap()
}

#[test]
fn test_posix_variable_quoting() {
    // POSIX 2.6.2: Variables should be quoted to prevent word splitting
    let rust_code = r#"
fn main() {
    let name = "Alice Smith";
    echo("{name}");
}
"#;

    let shell = transpile(rust_code);

    // Should use double quotes for variable expansion
    assert!(
        shell.contains(r#"echo "$name""#) || shell.contains(r#"echo "${name}""#),
        "Variables should be quoted: {}",
        shell
    );
}

#[test]
fn test_posix_string_literals() {
    // POSIX 2.6.2: String literals should use single quotes (no expansion)
    let rust_code = r#"
fn main() {
    echo("Hello $USER");
}
"#;

    let shell = transpile(rust_code);

    // Should use single quotes to prevent expansion
    assert!(
        shell.contains("'Hello $USER'"),
        "String literals should use single quotes: {}",
        shell
    );
}

#[test]
fn test_posix_command_substitution() {
    // POSIX 2.6.3: Command substitution should use $(...) not backticks
    let rust_code = r#"
fn main() {
    let current_dir = capture("pwd");
    echo("{current_dir}");
}
"#;

    let shell = transpile(rust_code);

    // Should use $() syntax, not backticks
    assert!(
        shell.contains("$(pwd)") || shell.contains("$( pwd )"),
        "Should use $() for command substitution: {}",
        shell
    );

    // Should NOT use backticks
    assert!(
        !shell.contains("`pwd`"),
        "Should not use backticks: {}",
        shell
    );
}

#[test]
fn test_posix_arithmetic_expansion() {
    // POSIX 2.6.4: Arithmetic should use $((...))
    let rust_code = r#"
fn main() {
    let x = 1 + 2;
    echo("{x}");
}
"#;

    let shell = transpile(rust_code);

    // Should use $((...)) for arithmetic
    assert!(
        shell.contains("$((") || shell.contains("$(( "),
        "Should use $((...)) for arithmetic: {}",
        shell
    );
}

#[test]
fn test_posix_test_expressions() {
    // POSIX 2.6: Test expressions should use [ ... ] syntax
    let rust_code = r#"
fn main() {
    let x = 5;
    if x > 0 {
        echo("positive");
    }
}
"#;

    let shell = transpile(rust_code);

    // Should use POSIX test syntax
    assert!(
        shell.contains("[ ") || shell.contains("test "),
        "Should use POSIX test syntax: {}",
        shell
    );

    // Should use -gt for integer comparison
    assert!(
        shell.contains("-gt") || shell.contains("> "),
        "Should use -gt for integer comparison: {}",
        shell
    );
}

#[test]
fn test_google_style_variable_naming() {
    // Google style: Use clear variable names with underscores
    let rust_code = r#"
fn main() {
    let user_name = "alice";
    let home_dir = "/home/alice";
    echo("{user_name} {home_dir}");
}
"#;

    let shell = transpile(rust_code);

    // Variable names should be preserved
    assert!(
        shell.contains("user_name") || shell.contains("USER_NAME"),
        "Should preserve variable names: {}",
        shell
    );
}

#[test]
fn test_google_style_error_to_stderr() {
    // Google style: Error messages should go to STDERR
    let rust_code = r#"
fn main() {
    eprint("Error occurred");
}
"#;

    let shell = transpile(rust_code);

    // Should redirect to stderr (>&2)
    assert!(
        shell.contains(">&2") || shell.contains("1>&2"),
        "Error messages should go to stderr: {}",
        shell
    );
}

#[test]
fn test_shellcheck_sc2086_no_unquoted_variables() {
    // ShellCheck SC2086: Double quote to prevent globbing and word splitting
    let rust_code = r#"
fn main() {
    let files = "*.txt";
    exec("ls {files}");
}
"#;

    let shell = transpile(rust_code);

    // Variables in commands should be quoted
    assert!(
        shell.contains(r#""$files""#) || shell.contains(r#""${files}""#),
        "Variables should be quoted (SC2086): {}",
        shell
    );
}

#[test]
fn test_shellcheck_sc2046_no_unquoted_command_substitution() {
    // ShellCheck SC2046: Quote command substitutions
    let rust_code = r#"
fn main() {
    let result = capture("find . -name '*.rs'");
    echo("{result}");
}
"#;

    let shell = transpile(rust_code);

    // Command substitution results should be quoted when used
    assert!(
        shell.contains(r#""$result""#) || shell.contains(r#""${result}""#),
        "Command substitution results should be quoted (SC2046): {}",
        shell
    );
}

#[test]
fn test_shellcheck_sc2116_no_useless_echo() {
    // ShellCheck SC2116: Don't wrap strings in echo unnecessarily
    let rust_code = r#"
fn main() {
    echo("hello");
}
"#;

    let shell = transpile(rust_code);

    // Should use direct echo, not echo $(echo ...)
    let echo_count = shell.matches("echo").count();
    assert!(
        echo_count <= 2, // May have echo in boilerplate + user echo
        "Should not have nested echo calls (SC2116): {}",
        shell
    );
}

#[test]
fn test_safety_injection_prevention() {
    // Safety: Prevent command injection via special characters
    let rust_code = r#"
fn main() {
    let malicious = "'; rm -rf /; echo '";
    echo("{malicious}");
}
"#;

    let shell = transpile(rust_code);

    // Special characters should be escaped/quoted safely
    assert!(
        shell.contains("'") && shell.contains("rm -rf"),
        "Should safely handle special characters: {}",
        shell
    );

    // Should NOT allow command execution
    assert!(
        !shell.contains("'; rm -rf /"),
        "Should prevent command injection: {}",
        shell
    );
}

#[test]
fn test_safety_word_splitting() {
    // Safety: Prevent word splitting on spaces
    let rust_code = r#"
fn main() {
    let path = "/path/with spaces/file.txt";
    exec("cat {path}");
}
"#;

    let shell = transpile(rust_code);

    // Path should be quoted to prevent word splitting
    assert!(
        shell.contains(r#""$path""#)
            || shell.contains("'$path'")
            || shell.contains("'/path/with spaces/file.txt'"),
        "Paths with spaces should be quoted: {}",
        shell
    );
}

#[test]
fn test_safety_glob_prevention() {
    // Safety: Prevent unintended glob expansion
    let rust_code = r#"
fn main() {
    let pattern = "*.txt";
    echo("{pattern}");
}
"#;

    let shell = transpile(rust_code);

    // Glob pattern should be quoted to prevent expansion
    assert!(
        shell.contains("'*.txt'") || shell.contains(r#""$pattern""#),
        "Glob patterns should be quoted: {}",
        shell
    );
}

#[test]
fn test_posix_set_options() {
    // POSIX: Generated scripts should use safe set options
    let rust_code = r#"
fn main() {
    echo("test");
}
"#;

    let shell = transpile(rust_code);

    // Should include safety options
    assert!(
        shell.contains("set -e") || shell.contains("set -u") || shell.contains("set -euf"),
        "Should include safety set options: {}",
        shell
    );
}

#[test]
fn test_posix_ifs_safety() {
    // POSIX: IFS should be set to safe value
    let rust_code = r#"
fn main() {
    echo("test");
}
"#;

    let shell = transpile(rust_code);

    // Should set IFS to newline (or safe value)
    assert!(
        shell.contains("IFS=") || shell.contains("IFS='"),
        "Should set IFS for safety: {}",
        shell
    );
}

#[test]
fn test_posix_shebang() {
    // POSIX: Should use proper shebang
    let rust_code = r#"
fn main() {
    echo("test");
}
"#;

    let shell = transpile(rust_code);

    // Should have shebang
    assert!(
        shell.starts_with("#!/bin/sh") || shell.starts_with("#!/usr/bin/env sh"),
        "Should have POSIX shebang: {}",
        shell
    );
}

#[test]
fn test_comparison_operators_posix() {
    // POSIX: Integer comparison operators should use -eq, -ne, -lt, -gt, -le, -ge
    let rust_code = r#"
fn main() {
    let x = 5;
    if x == 5 { echo("equal"); }
    if x != 3 { echo("not equal"); }
    if x < 10 { echo("less"); }
    if x > 0 { echo("greater"); }
}
"#;

    let shell = transpile(rust_code);

    // Should use POSIX integer comparison operators
    assert!(
        shell.contains("-eq")
            || shell.contains("-ne")
            || shell.contains("-lt")
            || shell.contains("-gt"),
        "Should use POSIX comparison operators: {}",
        shell
    );
}
