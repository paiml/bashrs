#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
                               // Integration tests for shell-specific rule filtering
                               // Tests that rules are filtered based on detected shell type

use bashrs::linter::{lint_shell_with_path, Severity};
use std::path::PathBuf;

/// Test that bash-only rules don't fire on zsh files
/// RED Phase: This test SHOULD FAIL initially (rules not filtered yet)
#[test]
fn test_bash_array_rule_skipped_for_zsh() {
    // ARRANGE: Zsh array syntax (valid in zsh, would be flagged in bash)
    let zsh_content = r#"#!/usr/bin/env zsh
# Zsh arrays are different from bash
arr=(one two three)
echo ${arr[1]}  # zsh is 1-indexed
"#;
    let path = PathBuf::from(".zshrc");

    // ACT: Lint as zsh file
    let result = lint_shell_with_path(&path, zsh_content);

    // ASSERT: Should not have bash-specific array warnings
    // In future, we'll have bash-specific rules that check array syntax
    // For now, verify no bash-only rules fire

    // This is a placeholder - we'll add specific bash-only rules later
    // For now, just verify the system works
    assert!(result
        .diagnostics
        .iter()
        .all(|d| d.severity != Severity::Error));
}

/// Test that universal rules still fire on all shell types
#[test]
fn test_universal_rules_fire_on_all_shells() {
    // ARRANGE: $RANDOM usage (universal - bad in all shells for determinism)
    let bash_content = r#"#!/bin/bash
SESSION_ID=$RANDOM
"#;
    let zsh_content = r#"#!/usr/bin/env zsh
SESSION_ID=$RANDOM
"#;

    // ACT: Lint both
    let bash_result = lint_shell_with_path(&PathBuf::from("test.bash"), bash_content);
    let zsh_result = lint_shell_with_path(&PathBuf::from(".zshrc"), zsh_content);

    // ASSERT: Both should have DET001 warning about $RANDOM
    assert!(bash_result
        .diagnostics
        .iter()
        .any(|d| d.message.contains("RANDOM")));
    assert!(zsh_result
        .diagnostics
        .iter()
        .any(|d| d.message.contains("RANDOM")));
}

/// Test that POSIX sh files don't get bash-specific warnings
#[test]
fn test_sh_files_skip_bash_only_rules() {
    // ARRANGE: POSIX sh with basic constructs
    let sh_content = r#"#!/bin/sh
# POSIX sh - no arrays, no [[]], no process substitution
if [ -f "$file" ]; then
    echo "File exists"
fi
"#;
    let path = PathBuf::from("install.sh");

    // ACT: Lint as sh file
    let result = lint_shell_with_path(&path, sh_content);

    // ASSERT: No bash-specific rule violations
    // Universal rules (quoting, etc.) should still apply
    assert!(result
        .diagnostics
        .iter()
        .all(|d| !d.message.contains("bash")));
}

/// Test that bash files get full rule set
#[test]
fn test_bash_files_get_all_applicable_rules() {
    // ARRANGE: Bash script with determinism issue
    let bash_content = r#"#!/bin/bash
TIMESTAMP=$(date +%s)
mkdir /tmp/build-$TIMESTAMP
"#;
    let path = PathBuf::from("build.bash");

    // ACT: Lint as bash file
    let result = lint_shell_with_path(&path, bash_content);

    // ASSERT: Should have warnings (DET002 for timestamp, IDEM001 for mkdir)
    assert!(!result.diagnostics.is_empty());
    // At least timestamp warning
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.message.contains("date") || d.message.contains("timestamp")));
}

/// Property test: Filtering is deterministic
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_filtering_is_deterministic(content in ".{0,100}") {
            let path1 = PathBuf::from(".zshrc");
            let path2 = PathBuf::from(".zshrc");

            let result1 = lint_shell_with_path(&path1, &content);
            let result2 = lint_shell_with_path(&path2, &content);

            // Same input should produce same diagnostics
            prop_assert_eq!(result1.diagnostics.len(), result2.diagnostics.len());
        }
    }

    proptest! {
        #[test]
        fn prop_universal_rules_apply_regardless_of_shell(
            content in r"(SESSION_ID=\$RANDOM|TIMESTAMP=\$\(date \+%s\))"
        ) {
            // Universal rules (DET001, DET002) should fire for any shell
            let bash_result = lint_shell_with_path(&PathBuf::from("test.bash"), &content);
            let zsh_result = lint_shell_with_path(&PathBuf::from(".zshrc"), &content);
            let sh_result = lint_shell_with_path(&PathBuf::from("test.sh"), &content);

            // All should have warnings (universal rules apply to all)
            let has_warnings = !bash_result.diagnostics.is_empty()
                && !zsh_result.diagnostics.is_empty()
                && !sh_result.diagnostics.is_empty();

            prop_assert!(has_warnings, "Universal rules should fire for all shell types");
        }
    }
}

// === Batch 2 Integration Tests ===

/// Test that [[ ]] rules (SC2108-SC2110) don't fire on POSIX sh
#[test]
fn test_double_bracket_rules_skipped_for_sh() {
    // ARRANGE: POSIX sh script with single brackets (valid)
    let sh_content = r#"#!/bin/sh
# POSIX sh only supports [ ], not [[ ]]
if [ "$x" = "1" -a "$y" = "2" ]; then
    echo "Both conditions true"
fi
"#;
    let path = PathBuf::from("install.sh");

    // ACT: Lint as sh file
    let result = lint_shell_with_path(&path, sh_content);

    // ASSERT: Should NOT have SC2108 (use && in [[ ]])
    // because POSIX sh doesn't have [[ ]], so this rule shouldn't apply
    assert!(
        !result.diagnostics.iter().any(|d| d.code == "SC2108"),
        "SC2108 should not fire on POSIX sh files"
    );
}

/// Test that [[ ]] rules DO fire on bash files
#[test]
fn test_double_bracket_rules_fire_for_bash() {
    // ARRANGE: Bash script with [[ ]] using -a (should warn)
    let bash_content = r#"#!/bin/bash
if [[ "$x" = "1" -a "$y" = "2" ]]; then
    echo "Both conditions true"
fi
"#;
    let path = PathBuf::from("script.bash");

    // ACT: Lint as bash file
    let result = lint_shell_with_path(&path, bash_content);

    // ASSERT: SHOULD have SC2108 (use && instead of -a in [[ ]])
    assert!(
        result.diagnostics.iter().any(|d| d.code == "SC2108"),
        "SC2108 should fire on bash files with [[ ]] and -a"
    );
}

/// Test that function keyword rules (SC2111-SC2113) don't fire on POSIX sh
#[test]
fn test_function_keyword_rules_skipped_for_sh() {
    // ARRANGE: POSIX sh script with POSIX function syntax
    let sh_content = r#"#!/bin/sh
# POSIX sh function syntax (no 'function' keyword)
my_func() {
    echo "Hello"
}
"#;
    let path = PathBuf::from("lib.sh");

    // ACT: Lint as sh file
    let result = lint_shell_with_path(&path, sh_content);

    // ASSERT: Should NOT have SC2111-SC2113 (function keyword warnings)
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|d| d.code == "SC2111" || d.code == "SC2112" || d.code == "SC2113"),
        "Function keyword rules should not fire on POSIX sh"
    );
}

/// Test that function keyword rules DO fire on bash files with 'function' keyword
#[test]
fn test_function_keyword_rules_fire_for_bash() {
    // ARRANGE: Bash script with 'function' keyword
    let bash_content = r#"#!/bin/bash
function my_func {
    echo "Hello"
}
"#;
    let path = PathBuf::from("script.bash");

    // ACT: Lint as bash file
    let result = lint_shell_with_path(&path, bash_content);

    // ASSERT: SHOULD have SC2112 (function keyword is non-standard)
    assert!(
        result.diagnostics.iter().any(|d| d.code == "SC2112"),
        "SC2112 should fire on bash files with 'function' keyword"
    );
}

/// Test that universal arithmetic rules fire on all shells
#[test]
fn test_arithmetic_rules_fire_on_all_shells() {
    // ARRANGE: expr usage (deprecated in all shells)
    let bash_content = r#"#!/bin/bash
result=$(expr 1 + 2)
"#;
    let sh_content = r#"#!/bin/sh
result=$(expr 1 + 2)
"#;
    let zsh_content = r#"#!/usr/bin/env zsh
result=$(expr 1 + 2)
"#;

    // ACT: Lint all three
    let bash_result = lint_shell_with_path(&PathBuf::from("script.bash"), bash_content);
    let sh_result = lint_shell_with_path(&PathBuf::from("script.sh"), sh_content);
    let zsh_result = lint_shell_with_path(&PathBuf::from(".zshrc"), zsh_content);

    // ASSERT: All should have SC2003 (expr is antiquated)
    assert!(
        bash_result.diagnostics.iter().any(|d| d.code == "SC2003"),
        "SC2003 should fire on bash"
    );
    assert!(
        sh_result.diagnostics.iter().any(|d| d.code == "SC2003"),
        "SC2003 should fire on sh"
    );
    assert!(
        zsh_result.diagnostics.iter().any(|d| d.code == "SC2003"),
        "SC2003 should fire on zsh"
    );
}

/// Test that universal quoting rules fire on all shells
#[test]
fn test_quoting_rules_fire_on_all_shells() {
    // ARRANGE: Subshell variable assignment (universal issue)
    let bash_content = r#"#!/bin/bash
(foo=bar)
echo "$foo"
"#;
    let sh_content = r#"#!/bin/sh
(foo=bar)
echo "$foo"
"#;

    // ACT: Lint both
    let bash_result = lint_shell_with_path(&PathBuf::from("script.bash"), bash_content);
    let sh_result = lint_shell_with_path(&PathBuf::from("script.sh"), sh_content);

    // ASSERT: Both should have SC2030 (variable modified in subshell)
    assert!(
        bash_result.diagnostics.iter().any(|d| d.code == "SC2030"),
        "SC2030 should fire on bash"
    );
    assert!(
        sh_result.diagnostics.iter().any(|d| d.code == "SC2030"),
        "SC2030 should fire on sh"
    );
}
