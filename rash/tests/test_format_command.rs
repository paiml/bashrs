// test_format_command.rs - EXTREME TDD tests for bashrs format command
// Following ruchy design patterns for formatter
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper function to create rash command
fn rash_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
}

/// Helper to create test bash script
fn create_test_script(dir: &TempDir, name: &str, content: &str) -> String {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write test script");
    path.to_string_lossy().to_string()
}

// RED TEST 1: Format command exists
#[test]
fn test_format_001_command_exists() {
    rash_cmd()
        .arg("format")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Format bash scripts"));
}

// RED TEST 2: Format basic script with inconsistent indentation
#[test]
fn test_format_002_basic_formatting() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script = create_test_script(&temp_dir, "unformatted.sh", r#"#!/bin/bash
# Inconsistent indentation
if [ -n "$VAR" ]; then
echo "test"
    echo "inconsistent"
fi

function greet() {
echo "hello"
}
"#);

    rash_cmd()
        .arg("format")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains("formatted"));

    // Verify file was formatted
    let formatted = fs::read_to_string(&script).expect("Failed to read formatted file");
    assert!(formatted.contains("  echo \"test\"")); // Consistent indentation
    assert!(formatted.contains("  echo \"inconsistent\""));
}

// RED TEST 3: Check mode - should detect unformatted code
#[test]
fn test_format_003_check_mode_unformatted() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script = create_test_script(&temp_dir, "unformatted.sh", r#"#!/bin/bash
if [ -n "$VAR" ]; then
echo "bad indent"
fi
"#);

    rash_cmd()
        .arg("format")
        .arg("--check")
        .arg(&script)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not properly formatted"));
}

// RED TEST 4: Check mode - properly formatted should pass
#[test]
fn test_format_004_check_mode_formatted() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script = create_test_script(&temp_dir, "formatted.sh", r#"#!/bin/bash

if [ -n "$VAR" ]; then
  echo "good indent"
fi
"#);

    rash_cmd()
        .arg("format")
        .arg("--check")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains("properly formatted"));
}

// RED TEST 5: Quote unquoted variables
#[test]
fn test_format_005_quote_variables() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script = create_test_script(&temp_dir, "unquoted.sh", r#"#!/bin/bash
VAR=value
echo $VAR
"#);

    rash_cmd()
        .arg("format")
        .arg(&script)
        .assert()
        .success();

    let formatted = fs::read_to_string(&script).expect("Failed to read formatted file");
    assert!(formatted.contains("\"$VAR\"") || formatted.contains("${VAR}"));
}

// RED TEST 6: Normalize function syntax
#[test]
fn test_format_006_normalize_functions() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script = create_test_script(&temp_dir, "functions.sh", r#"#!/bin/bash
function foo {
  echo "no parens"
}

bar() {
echo "inconsistent indent"
}
"#);

    rash_cmd()
        .arg("format")
        .arg(&script)
        .assert()
        .success();

    let formatted = fs::read_to_string(&script).expect("Failed to read formatted file");
    // Should normalize to: function_name() {
    assert!(formatted.contains("foo()") || formatted.contains("function foo()"));
    assert!(formatted.contains("  echo")); // Consistent indentation
}

// RED TEST 7: Format if statements consistently
#[test]
fn test_format_007_if_statements() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script = create_test_script(&temp_dir, "if.sh", r#"#!/bin/bash
if [ "$VAR" = "test" ]
then
echo "split then"
fi

if [ "$VAR" = "test" ]; then
  echo "inline then"
fi
"#);

    rash_cmd()
        .arg("format")
        .arg(&script)
        .assert()
        .success();

    let formatted = fs::read_to_string(&script).expect("Failed to read formatted file");
    // Should normalize to inline "then"
    assert!(formatted.contains("]; then"));
}

// RED TEST 8: Preserve comments and shebangs
#[test]
fn test_format_008_preserve_comments() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script = create_test_script(&temp_dir, "comments.sh", r#"#!/bin/bash
# Header comment
VAR="value" # Inline comment

# Function comment
foo() {
  # Inside function
  echo "test"
}
"#);

    rash_cmd()
        .arg("format")
        .arg(&script)
        .assert()
        .success();

    let formatted = fs::read_to_string(&script).expect("Failed to read formatted file");
    assert!(formatted.contains("#!/bin/bash"));
    assert!(formatted.contains("# Header comment"));
    assert!(formatted.contains("# Inline comment"));
    assert!(formatted.contains("# Function comment"));
    assert!(formatted.contains("# Inside function"));
}

// RED TEST 9: Format with tabs instead of spaces
#[test]
fn test_format_009_tabs_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create config file
    let config_path = temp_dir.path().join(".bashrs-fmt.toml");
    fs::write(&config_path, r#"
indent_width = 2
use_tabs = true
"#).expect("Failed to write config");

    let script = create_test_script(&temp_dir, "script.sh", r#"#!/bin/bash
if [ -n "$VAR" ]; then
echo "test"
fi
"#);

    rash_cmd()
        .arg("format")
        .arg(&script)
        .assert()
        .success();

    let formatted = fs::read_to_string(&script).expect("Failed to read formatted file");
    // Should use tabs for indentation
    assert!(formatted.contains("\techo"));
}

// RED TEST 10: Ignore directive support
#[test]
fn test_format_010_ignore_directive() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script = create_test_script(&temp_dir, "ignore.sh", r#"#!/bin/bash

# bashrs-fmt-ignore
if [ -n "$VAR" ]; then
echo "intentionally bad formatting"
fi

# Normal formatting
if [ -n "$VAR2" ]; then
echo "should be formatted"
fi
"#);

    rash_cmd()
        .arg("format")
        .arg(&script)
        .assert()
        .success();

    let formatted = fs::read_to_string(&script).expect("Failed to read formatted file");
    // First if should remain unformatted
    let lines: Vec<&str> = formatted.lines().collect();
    let ignore_section = lines.iter()
        .skip_while(|line| !line.contains("bashrs-fmt-ignore"))
        .take(4)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(ignore_section.contains("echo \"intentionally bad formatting\""));

    // Second if should be formatted with proper indentation
    assert!(formatted.contains("  echo \"should be formatted\""));
}

// RED TEST 11: Format case statements
#[test]
fn test_format_011_case_statements() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script = create_test_script(&temp_dir, "case.sh", r#"#!/bin/bash
case $VAR in
start)
echo "starting"
;;
stop)
echo "stopping"
;;
*)
echo "unknown"
;;
esac
"#);

    rash_cmd()
        .arg("format")
        .arg(&script)
        .assert()
        .success();

    let formatted = fs::read_to_string(&script).expect("Failed to read formatted file");
    // Should have consistent indentation
    assert!(formatted.contains("  start)") || formatted.contains("start)"));
    assert!(formatted.contains("    echo") || formatted.contains("  echo"));
}

// RED TEST 12: Multiple files formatting
#[test]
fn test_format_012_multiple_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script1 = create_test_script(&temp_dir, "script1.sh", r#"#!/bin/bash
echo "test1"
"#);
    let script2 = create_test_script(&temp_dir, "script2.sh", r#"#!/bin/bash
echo "test2"
"#);

    rash_cmd()
        .arg("format")
        .arg(&script1)
        .arg(&script2)
        .assert()
        .success()
        .stdout(predicate::str::contains("script1.sh"))
        .stdout(predicate::str::contains("script2.sh"));
}

// RED TEST 13: Output option (format to different file)
#[test]
fn test_format_013_output_option() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_script = create_test_script(&temp_dir, "input.sh", r#"#!/bin/bash
echo "test"
"#);
    let output_path = temp_dir.path().join("output.sh");

    rash_cmd()
        .arg("format")
        .arg(&input_script)
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    // Verify output file was created
    assert!(output_path.exists());

    // Original should remain unchanged
    let original = fs::read_to_string(&input_script).expect("Failed to read original");
    assert!(original.contains("echo \"test\""));
}

// RED TEST 14: Dry-run mode (show diff without applying)
#[test]
fn test_format_014_dry_run() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script = create_test_script(&temp_dir, "script.sh", r#"#!/bin/bash
if [ -n "$VAR" ]; then
echo "test"
fi
"#);

    let original = fs::read_to_string(&script).expect("Failed to read original");

    rash_cmd()
        .arg("format")
        .arg("--dry-run")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains("Would format:"));

    // File should remain unchanged
    let after = fs::read_to_string(&script).expect("Failed to read after dry-run");
    assert_eq!(original, after);
}

// RED TEST 15: Indent width configuration
#[test]
fn test_format_015_indent_width() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create config with 4-space indent
    let config_path = temp_dir.path().join(".bashrs-fmt.toml");
    fs::write(&config_path, r#"
indent_width = 4
use_tabs = false
"#).expect("Failed to write config");

    let script = create_test_script(&temp_dir, "script.sh", r#"#!/bin/bash
if [ -n "$VAR" ]; then
echo "test"
fi
"#);

    rash_cmd()
        .arg("format")
        .arg(&script)
        .assert()
        .success();

    let formatted = fs::read_to_string(&script).expect("Failed to read formatted file");
    // Should use 4 spaces for indentation
    assert!(formatted.contains("    echo \"test\""));
}
