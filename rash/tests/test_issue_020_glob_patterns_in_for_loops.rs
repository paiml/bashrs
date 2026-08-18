#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Test Issue #20: SC2154 false positives for loop variables
//!
//! EXTREME TDD: GREEN Phase - PASSING ✅
//!
//! Issue: bashrs lint incorrectly reports SC2154 for loop variables
//! Example: for dockerfile in docker/*/Dockerfile; do echo "$dockerfile"; done
//!
//! Discovery: Original report included invalid bash syntax (pipe in for loop)
//! This is USER ERROR - bash itself rejects: for x in glob | sort; do
//! Valid syntax requires command substitution: for x in $(find ... | sort); do
//!
//! REAL BUG: SC2154 was flagging loop variables as undefined (NOW FIXED)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to create bashrs command
fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

#[test]
fn test_issue_020_sc2154_simple_loop_variable() {
    // GREEN: Loop variable should NOT trigger SC2154
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("test.sh");

    let bash_code = r#"#!/usr/bin/env bash
for dockerfile in docker/*/fibonacci.Dockerfile; do
    echo "$dockerfile"
done
"#;

    fs::write(&script, bash_code).unwrap();

    // Should lint without SC2154 warnings for 'dockerfile'
    let output = bashrs_cmd().arg("lint").arg(&script).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("SC2154"),
        "Should not flag loop variable 'dockerfile' as undefined"
    );
}

#[test]
fn test_issue_020_sc2154_multiple_loop_variables() {
    // GREEN: Multiple loop variables should NOT trigger SC2154
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("test.sh");

    let bash_code = r#"#!/usr/bin/env bash
for file in src/**/*.rs; do
    echo "$file"
done

for test in tests/**/*.rs; do
    echo "$test"
done
"#;

    fs::write(&script, bash_code).unwrap();

    let output = bashrs_cmd().arg("lint").arg(&script).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("SC2154"),
        "Should not flag loop variables as undefined"
    );
}

#[test]
fn test_issue_020_sc2154_loop_with_command_substitution() {
    // GREEN: Loop variable with command substitution (VALID syntax)
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("test.sh");

    let bash_code = r#"#!/usr/bin/env bash
for dockerfile in $(find docker/*/Dockerfile | sort -r | head -10); do
    lang="$(basename "$(dirname "$dockerfile")")"
    echo "Processing: ${lang}"
done
"#;

    fs::write(&script, bash_code).unwrap();

    let output = bashrs_cmd().arg("lint").arg(&script).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("SC2154"),
        "Should not flag loop or assigned variables as undefined"
    );
}

#[test]
fn test_issue_020_sc2154_glob_pattern() {
    // GREEN: Glob patterns with loop variables (no SC2154 warnings)
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("test.sh");

    let bash_code = r#"#!/usr/bin/env bash
for file in /path/*/*.txt; do
    echo "$file"
done
"#;

    fs::write(&script, bash_code).unwrap();

    let output = bashrs_cmd().arg("lint").arg(&script).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("SC2154"),
        "Should not flag loop variable 'file' as undefined"
    );
}

#[test]
fn test_issue_020_sc2154_original_case_fixed_syntax() {
    // GREEN: Original case from issue with CORRECTED syntax
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("test.sh");

    // Fixed: Use command substitution instead of invalid pipe syntax
    let bash_code = r#"#!/usr/bin/env bash
for dockerfile in $(find docker/*/fibonacci.Dockerfile | sort); do
    lang="$(basename "$(dirname "$dockerfile")")"
    echo "Scoring: ${lang}"
done
"#;

    fs::write(&script, bash_code).unwrap();

    let output = bashrs_cmd().arg("lint").arg(&script).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should NOT contain SC2154 warnings for 'dockerfile' or 'lang'
    assert!(
        !stdout.contains("SC2154"),
        "Should not flag loop variables with SC2154. Output:\n{}",
        stdout
    );
}

#[test]
fn test_issue_020_sc2154_undefined_still_caught() {
    // GREEN: Undefined variables should STILL be caught (not loop vars)
    let temp = TempDir::new().unwrap();
    let script = temp.path().join("test.sh");

    let bash_code = r#"#!/usr/bin/env bash
for file in src/*.rs; do
    echo "$file $undefined_var"
done
"#;

    fs::write(&script, bash_code).unwrap();

    let output = bashrs_cmd().arg("lint").arg(&script).output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain SC2154 for 'undefined_var' but NOT for 'file'
    assert!(
        stdout.contains("SC2154"),
        "Should flag truly undefined variable 'undefined_var'"
    );
    assert!(
        stdout.contains("undefined_var"),
        "SC2154 warning should mention 'undefined_var'"
    );
    // The warning should NOT be about the loop variable 'file'
    assert!(
        !stdout.contains("'file'"),
        "Should NOT flag loop variable 'file'"
    );
}
