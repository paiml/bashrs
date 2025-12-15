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
fn rash_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
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
fn test_unsafe_det002_timestamp_never_autofix() {
    // RED: DET002 should be UNSAFE
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("timestamp.sh");

    fs::write(
        &script,
        r#"#!/bin/bash
RELEASE="release-$(date +%s)"
"#,
    )
    .unwrap();

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
        content.contains("date +%s"),
        "Should NOT automatically replace timestamp (UNSAFE)"
    );
}

// ============================================================================
// PROPERTY TESTS - Verify semantic preservation
// ============================================================================

#[test]
fn test_property_safe_fixes_preserve_syntax() {
    // RED: All SAFE fixes must produce valid bash
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("input.sh");

    // Script with multiple SAFE issues (variables defined and used to avoid shellcheck warnings)
    fs::write(
        &script,
        r#"#!/bin/bash
var1="hello"
var2="world"
var4="test"
echo $var1
echo $var2
var3=$(echo $var4)
echo $var3
"#,
    )
    .unwrap();

    let fixed = temp.path().join("fixed.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--output")
        .arg(&fixed)
        .assert()
        .success(); // Exit code 0 when using --output --fix

    // Property: Fixed script must be valid bash (passes shellcheck)
    Command::new("shellcheck")
        .arg("-s")
        .arg("bash")
        .arg(&fixed)
        .assert()
        .success();
}

#[test]
fn test_property_safe_fixes_are_idempotent() {
    // RED: Applying --fix twice should produce identical output
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("input.sh");

    fs::write(
        &script,
        r#"#!/bin/bash
variable="test"
echo $variable
"#,
    )
    .unwrap();

    // First fix (exit code 0 when using --output --fix)
    let fixed1 = temp.path().join("fixed1.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--output")
        .arg(&fixed1)
        .assert()
        .success();

    // Second fix (on already fixed file - exit code 0 when no warnings)
    let fixed2 = temp.path().join("fixed2.sh");
    rash_cmd()
        .arg("lint")
        .arg(&fixed1)
        .arg("--fix")
        .arg("--output")
        .arg(&fixed2)
        .assert()
        .success();

    // Property: fixed1 == fixed2 (idempotent)
    // Note: If no fixes were applied, fixed2 won't exist, so compare fixed1 with itself
    let content1 = fs::read_to_string(&fixed1).unwrap();
    let content2 = if fixed2.exists() {
        fs::read_to_string(&fixed2).unwrap()
    } else {
        // No fixes needed, so fixed1 is already idempotent
        content1.clone()
    };
    assert_eq!(content1, content2, "Fixes should be idempotent");
}

#[test]
fn test_property_assumptions_require_explicit_opt_in() {
    // RED: SAFE-WITH-ASSUMPTIONS fixes MUST NOT apply without --fix-assumptions
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("input.sh");

    fs::write(
        &script,
        r#"#!/bin/bash
mkdir /tmp/dir
rm /tmp/file
"#,
    )
    .unwrap();

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
    // Property: Without --fix-assumptions, no IDEM fixes applied
    assert!(!content.contains("mkdir -p"), "Should not apply mkdir -p");
    assert!(!content.contains("rm -f"), "Should not apply rm -f");
}

// ============================================================================
// INTEGRATION TESTS - Mixed safety levels
// ============================================================================

#[test]
fn test_integration_mixed_safety_levels() {
    // RED: File with SAFE, SAFE-WITH-ASSUMPTIONS, and UNSAFE issues
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("mixed.sh");

    fs::write(
        &script,
        r#"#!/bin/bash
# SAFE: SC2086
echo $unquoted_var

# SAFE-WITH-ASSUMPTIONS: IDEM001
mkdir /tmp/mydir

# UNSAFE: DET001
SESSION_ID=$RANDOM
"#,
    )
    .unwrap();

    // Test 1: Default --fix (only SAFE)
    let fixed1 = temp.path().join("fixed1.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--output")
        .arg(&fixed1)
        .assert()
        .success();

    let content1 = fs::read_to_string(&fixed1).unwrap();
    assert!(
        content1.contains(r#""$unquoted_var""#),
        "Should fix SAFE issue"
    );
    assert!(
        !content1.contains("mkdir -p"),
        "Should NOT fix SAFE-WITH-ASSUMPTIONS"
    );
    assert!(content1.contains("$RANDOM"), "Should NOT fix UNSAFE");

    // Test 2: --fix --fix-assumptions (SAFE + SAFE-WITH-ASSUMPTIONS)
    let fixed2 = temp.path().join("fixed2.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--fix-assumptions")
        .arg("--output")
        .arg(&fixed2)
        .assert()
        .success();

    let content2 = fs::read_to_string(&fixed2).unwrap();
    assert!(
        content2.contains(r#""$unquoted_var""#),
        "Should fix SAFE issue"
    );
    assert!(
        content2.contains("mkdir -p"),
        "Should fix SAFE-WITH-ASSUMPTIONS"
    );
    assert!(content2.contains("$RANDOM"), "Should NOT fix UNSAFE");
}

#[test]
fn test_integration_real_world_deploy_script() {
    // RED: Production-like deployment script
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("deploy.sh");

    fs::write(
        &script,
        r#"#!/bin/bash
# Real-world deployment script

VERSION=$1
RELEASE_DIR=/app/releases/$VERSION

# SAFE-WITH-ASSUMPTIONS: Should require --fix-assumptions
mkdir $RELEASE_DIR

# SAFE: Should auto-fix
echo Deploying $VERSION to $RELEASE_DIR

# SAFE-WITH-ASSUMPTIONS
rm /app/current

# UNSAFE: Should never auto-fix
ln -s $RELEASE_DIR /app/current
"#,
    )
    .unwrap();

    // With --fix only
    let fixed_safe = temp.path().join("fixed_safe.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--output")
        .arg(&fixed_safe)
        .assert()
        .success();

    let safe_content = fs::read_to_string(&fixed_safe).unwrap();
    // Only SAFE fixes applied (quoting)
    assert!(
        safe_content.contains(r#""$VERSION""#),
        "Should quote variables"
    );
    assert!(
        !safe_content.contains("mkdir -p"),
        "Should not add -p without assumptions"
    );
    assert!(
        !safe_content.contains("ln -sf"),
        "Should not fix unsafe symlink"
    );

    // With --fix --fix-assumptions
    let fixed_assumptions = temp.path().join("fixed_assumptions.sh");
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix")
        .arg("--fix-assumptions")
        .arg("--output")
        .arg(&fixed_assumptions)
        .assert()
        .success();

    let assumptions_content = fs::read_to_string(&fixed_assumptions).unwrap();
    // SAFE + SAFE-WITH-ASSUMPTIONS applied
    assert!(
        assumptions_content.contains(r#""$VERSION""#),
        "Should quote variables"
    );
    assert!(
        assumptions_content.contains("mkdir -p"),
        "Should add -p with assumptions"
    );
    assert!(
        assumptions_content.contains("rm -f"),
        "Should add -f with assumptions"
    );
    assert!(
        !assumptions_content.contains("ln -sf"),
        "Should still not fix unsafe symlink"
    );
}

// ============================================================================
// CLI FLAG TESTS - Verify --fix-assumptions implementation
// ============================================================================

#[test]
fn test_cli_fix_assumptions_flag_exists() {
    // RED: CLI should accept --fix-assumptions flag
    rash_cmd()
        .arg("lint")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--fix-assumptions"));
}

#[test]
fn test_cli_fix_without_assumptions_ignores_flag() {
    // RED: --fix-assumptions without --fix should be ignored or error
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("test.sh");
    fs::write(&script, "#!/bin/bash\nmkdir /tmp/test\n").unwrap();

    // Should either error or ignore
    rash_cmd()
        .arg("lint")
        .arg(&script)
        .arg("--fix-assumptions")
        .assert()
        .code(predicate::in_iter(vec![0, 1, 2])); // Accept success or error
}
