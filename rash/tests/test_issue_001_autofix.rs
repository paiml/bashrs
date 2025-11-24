#![allow(clippy::unwrap_used)]
// Tests can use unwrap() for simplicity
// Issue #1: Bash Auto-fix Creates Invalid Syntax
// Following EXTREME TDD methodology - RED phase
#![allow(non_snake_case)] // Test naming convention: test_<TASK_ID>_<feature>_<scenario>

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn rash_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
}

/// Test that auto-fix preserves bash syntax validity
#[test]
fn test_ISSUE_001_autofix_preserves_syntax() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.sh");

    // ARRANGE: Create bash script with shellcheck warnings
    fs::write(
        &test_file,
        r#"#!/bin/bash
echo -e "${RED}Error${NC}"
local val=$(echo "$x" | cut -d. -f1)
rm file.txt
"#,
    )
    .unwrap();

    // ACT: Apply auto-fix
    rash_cmd()
        .arg("lint")
        .arg(&test_file)
        .arg("--fix")
        .assert()
        .success();

    // ASSERT: Fixed file passes bash syntax check
    let output = std::process::Command::new("bash")
        .arg("-n")
        .arg(&test_file)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Fixed script should pass bash syntax check.\nstderr: {}\nFixed content:\n{}",
        String::from_utf8_lossy(&output.stderr),
        fs::read_to_string(&test_file).unwrap()
    );
}

/// Test that auto-fix doesn't add extra closing braces
#[test]
fn test_ISSUE_001_autofix_no_extra_braces() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.sh");

    // ARRANGE: Script with color variables (common pattern)
    fs::write(
        &test_file,
        r#"#!/bin/bash
NC='\033[0m'
BLUE='\033[0;34m'
echo -e "${BLUE}text${NC}"
"#,
    )
    .unwrap();

    // ACT: Apply auto-fix
    rash_cmd()
        .arg("lint")
        .arg(&test_file)
        .arg("--fix")
        .assert()
        .success();

    // ASSERT: No malformed syntax
    let fixed = fs::read_to_string(&test_file).unwrap();

    assert!(
        !fixed.contains(r#""}"}"#),
        "Should not have extra closing braces. Content:\n{}",
        fixed
    );
    assert!(
        !fixed.contains(r#""${NC}"}"#),
        "Should not have malformed variable refs. Content:\n{}",
        fixed
    );

    // NEW: Check for extra empty quotes (the actual bug!)
    assert!(
        !fixed.contains(r#""${NC}"""#),
        "Should not have extra empty quotes after variable. Content:\n{}",
        fixed
    );
    assert!(
        !fixed.contains(r#""""#),
        "Should not have double empty quotes. Content:\n{}",
        fixed
    );

    // Verify syntax
    let output = std::process::Command::new("bash")
        .arg("-n")
        .arg(&test_file)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Should pass bash syntax check. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Test that SC2116 (useless echo) fix is syntactically valid
#[test]
fn test_ISSUE_001_autofix_sc2116_correctly() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("test.sh");

    // ARRANGE: Script with useless echo
    fs::write(
        &test_file,
        r#"#!/bin/bash
x="3.14"
local val=$(echo "$x" | cut -d. -f1)
echo $val
"#,
    )
    .unwrap();

    // ACT: Apply auto-fix
    rash_cmd()
        .arg("lint")
        .arg(&test_file)
        .arg("--fix")
        .assert()
        .success();

    // ASSERT: Fixed syntax is valid
    let fixed = fs::read_to_string(&test_file).unwrap();

    // Should not have broken pipe syntax
    assert!(
        !fixed.contains(r#"local val="$x" | cut"#),
        "Should not have broken pipe syntax. Content:\n{}",
        fixed
    );

    // Verify bash syntax
    let output = std::process::Command::new("bash")
        .arg("-n")
        .arg(&test_file)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "SC2116 fix should be syntactically valid. stderr: {}\nContent:\n{}",
        String::from_utf8_lossy(&output.stderr),
        fixed
    );
}

/// Test auto-fix with multiple issues in one file
#[test]
fn test_ISSUE_001_autofix_multiple_issues() {
    let temp = TempDir::new().unwrap();
    let test_file = temp.path().join("complex.sh");

    // ARRANGE: Complex script with multiple issues
    fs::write(
        &test_file,
        r#"#!/bin/bash
RED='\033[0;31m'
NC='\033[0m'

cleanup() {
    rm /tmp/test
    echo -e "${RED}Cleaned${NC}"
}

process() {
    local count=$(echo "$1" | wc -l)
    echo $count
}
"#,
    )
    .unwrap();

    // ACT: Apply auto-fix
    rash_cmd()
        .arg("lint")
        .arg(&test_file)
        .arg("--fix")
        .assert()
        .success();

    // ASSERT: All fixes preserve syntax
    let output = std::process::Command::new("bash")
        .arg("-n")
        .arg(&test_file)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Multi-issue fix should preserve syntax. stderr: {}\nContent:\n{}",
        String::from_utf8_lossy(&output.stderr),
        fs::read_to_string(&test_file).unwrap()
    );
}
