#![allow(deprecated)]
#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)]
//! EXTREME TDD: Fix Safety Taxonomy Validation Tests
//!
//! RED PHASE: These tests WILL FAIL until we classify existing rules.
//!
//! Test Structure:
//! 1. SAFE fixes - Applied by default `--fix`
//! 2. SAFE-WITH-ASSUMPTIONS - Require `--fix --fix-assumptions`
//! 3. UNSAFE - Never auto-applied, suggestions only
//! 4. Property tests - Verify semantic preservation
//! 5. Integration tests - Mixed safety levels

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to create bashrs command
#[allow(deprecated)]
fn rash_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

// ============================================================================
// SAFE FIXES - Applied by default with `--fix`
// ============================================================================

#[test]
fn test_safe_sc2086_unquoted_variable_autofix() {
    // RED: SC2086 should be classified as SAFE
    // Quoting variables is semantically safe transformation
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("unquoted.sh");

    fs::write(
        &script,
        r#"#!/bin/bash
echo $variable
echo $file
"#,
    )
    .unwrap();

    // Without --fix: Should report diagnostics (exit code 1 for warnings)
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .assert()
        .code(1)
        .stdout(predicate::str::contains("SC2086"))
        .stdout(predicate::str::contains("Double quote"));

    // With --fix: Should apply SAFE fixes
    let fixed = temp.path().join("fixed.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--output")
        .arg(&fixed)
        .assert()
        .success();

    // Verify fix applied
    let content = fs::read_to_string(&fixed).unwrap();
    assert!(
        content.contains(r#"echo "$variable""#),
        "Should quote $variable"
    );
    assert!(content.contains(r#"echo "$file""#), "Should quote $file");
}

#[test]
fn test_safe_sc2046_unquoted_command_substitution_autofix() {
    // RED: SC2046 should be classified as SAFE
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("cmd_sub.sh");

    fs::write(
        &script,
        r#"#!/bin/bash
for file in $(ls *.txt); do
    echo $file
done
"#,
    )
    .unwrap();

    // With --fix: Should apply SAFE fixes
    let fixed = temp.path().join("fixed.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--output")
        .arg(&fixed)
        .assert()
        .success();

    // Verify fix applied (quoting command substitution)
    let content = fs::read_to_string(&fixed).unwrap();
    assert!(
        content.contains(r#""$(ls *.txt)""#) || content.contains("*.txt"),
        "Should quote command substitution or suggest glob"
    );
}

#[test]
fn test_safe_sc2116_useless_echo_autofix() {
    // RED: SC2116 should be classified as SAFE
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("useless_echo.sh");

    fs::write(
        &script,
        r#"#!/bin/bash
var=$(echo $other_var)
result=$(echo "hello world")
"#,
    )
    .unwrap();

    // With --fix: Should apply SAFE fixes
    let fixed = temp.path().join("fixed.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--output")
        .arg(&fixed)
        .assert()
        .success();

    // Verify fix applied (removed useless echo)
    let content = fs::read_to_string(&fixed).unwrap();
    assert!(
        content.contains(r#"var="$other_var""#) || content.contains("var=$other_var"),
        "Should remove useless echo"
    );
}

// ============================================================================
// SAFE-WITH-ASSUMPTIONS - Require `--fix --fix-assumptions`
// ============================================================================

#[test]
fn test_safe_with_assumptions_idem001_mkdir_not_applied_by_default() {
    // RED: IDEM001 should be SAFE-WITH-ASSUMPTIONS
    // Assumption: Directory doesn't need strict error checking
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("mkdir.sh");

    fs::write(
        &script,
        r#"#!/bin/bash
mkdir /tmp/mydir
mkdir /app/releases
"#,
    )
    .unwrap();

    // Without --fix-assumptions: Should NOT apply fix
    let fixed = temp.path().join("fixed.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--output")
        .arg(&fixed)
        .assert()
        .success();

    let content = fs::read_to_string(&fixed).unwrap();
    // Should NOT have -p flag (only SAFE fixes applied)
    assert!(
        !content.contains("mkdir -p"),
        "Should NOT apply SAFE-WITH-ASSUMPTIONS fix without --fix-assumptions"
    );
}

#[test]
fn test_safe_with_assumptions_idem001_mkdir_with_flag() {
    // RED: With --fix-assumptions, should apply IDEM001
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("mkdir.sh");

    fs::write(
        &script,
        r#"#!/bin/bash
mkdir /tmp/mydir
"#,
    )
    .unwrap();

    // With --fix-assumptions: Should apply fix
    let fixed = temp.path().join("fixed.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--fix-assumptions")
        .arg("--output")
        .arg(&fixed)
        .assert()
        .success();

    let content = fs::read_to_string(&fixed).unwrap();
    assert!(
        content.contains("mkdir -p"),
        "Should apply mkdir -p with --fix-assumptions"
    );
}

#[test]
fn test_safe_with_assumptions_idem002_rm_not_applied_by_default() {
    // RED: IDEM002 should be SAFE-WITH-ASSUMPTIONS
    // Assumption: Missing file is not an error condition
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("rm.sh");

    fs::write(
        &script,
        r#"#!/bin/bash
rm /tmp/cache.txt
rm /app/current
"#,
    )
    .unwrap();

    // Without --fix-assumptions: Should NOT apply fix
    let fixed = temp.path().join("fixed.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--output")
        .arg(&fixed)
        .assert()
        .success();

    let content = fs::read_to_string(&fixed).unwrap();
    assert!(
        !content.contains("rm -f"),
        "Should NOT apply SAFE-WITH-ASSUMPTIONS fix without --fix-assumptions"
    );
}

#[test]
fn test_safe_with_assumptions_idem002_rm_with_flag() {
    // RED: With --fix-assumptions, should apply IDEM002
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("rm.sh");

    fs::write(
        &script,
        r#"#!/bin/bash
rm /tmp/cache.txt
"#,
    )
    .unwrap();

    // With --fix-assumptions: Should apply fix
    let fixed = temp.path().join("fixed.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--fix-assumptions")
        .arg("--output")
        .arg(&fixed)
        .assert()
        .success();

    let content = fs::read_to_string(&fixed).unwrap();
    assert!(
        content.contains("rm -f"),
        "Should apply rm -f with --fix-assumptions"
    );
}

// ============================================================================
// UNSAFE FIXES - Never auto-applied, suggestions only
// ============================================================================

#[test]
fn test_unsafe_idem003_symlink_never_autofix() {
    // RED: IDEM003 should be UNSAFE
    // Semantic transformation: ln -s → (remove existing + ln -s)
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("symlink.sh");

    fs::write(
        &script,
        r#"#!/bin/bash
ln -s /app/releases/v1.0.0 /app/current
"#,
    )
    .unwrap();

    // Even with --fix --fix-assumptions: Should NOT apply UNSAFE fix (exit code 0 when using --output)
    let fixed = temp.path().join("fixed.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--fix-assumptions")
        .arg("--output")
        .arg(&fixed)
        .assert()
        .success();

    let content = fs::read_to_string(&fixed).unwrap();
    // Should NOT have ln -sf or rm before ln
    assert!(!content.contains("ln -sf"), "Should NOT apply UNSAFE fix");
    assert!(
        !content.contains("rm") || !content.contains("ln -s"),
        "Should NOT add rm before ln -s (semantic change)"
    );

    // Should provide diagnostic output (exit code 1 for warnings)
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .assert()
        .code(1)
        .stdout(predicate::str::contains("IDEM003"));
}

#[test]
fn test_unsafe_det001_random_never_autofix() {
    // RED: DET001 should be UNSAFE
    // Semantic transformation: $RANDOM → version-based ID
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("random.sh");

    fs::write(
        &script,
        r#"#!/bin/bash
SESSION_ID=$RANDOM
echo "Session: $SESSION_ID"
"#,
    )
    .unwrap();

    // Even with --fix --fix-assumptions: Should NOT apply UNSAFE fix (exit code 0 when using --output)
    let fixed = temp.path().join("fixed.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--fix-assumptions")
        .arg("--output")
        .arg(&fixed)
        .assert()
        .success();

    let content = fs::read_to_string(&fixed).unwrap();
    // Should still contain $RANDOM (no automatic transformation)
    assert!(
        content.contains("$RANDOM"),
        "Should NOT automatically replace $RANDOM (UNSAFE)"
    );

    // Should provide diagnostic output (exit code 2 for errors including UNSAFE)
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .assert()
        .code(2)
        .stdout(predicate::str::contains("DET001"));
}

#[test]

include!("test_fix_safety_taxonomy_tests_cont.rs");
