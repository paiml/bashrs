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
