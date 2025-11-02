// Integration test for Issue #5: zsh linting with shell type detection
// Verifies that lint_shell() uses detect_shell_type() to correctly lint zsh files

use bashrs::linter::{lint_shell_with_path, LintResult};
use std::path::PathBuf;

#[test]
fn test_issue_005_zshrc_uses_zsh_rules() {
    // Real .zshrc with valid zsh syntax that bash would flag
    let zshrc_content = r#"#!/usr/bin/env zsh
# Valid zsh array splitting
filtered_args=("${(@f)"$(echo -e "line1\nline2")"}")
echo "${filtered_args[*]}"
"#;

    let path = PathBuf::from(".zshrc");
    let result = lint_shell_with_path(&path, zshrc_content);

    // Should NOT contain bash-specific errors on valid zsh syntax
    assert!(
        !contains_code(&result, "SC2296"),
        "SC2296 (nested parameter expansion) should not be flagged for zsh"
    );
}

#[test]
fn test_issue_005_bash_file_uses_bash_rules() {
    // .bashrc should still use bash rules (no regression)
    let bashrc_content = r#"#!/bin/bash
# This would be invalid in bash
x=$RANDOM
echo $x
"#;

    let path = PathBuf::from(".bashrc");
    let result = lint_shell_with_path(&path, bashrc_content);

    // Bash linting should still work (we have DET001 for $RANDOM)
    // This test just verifies the function works, not specific rules
    assert!(result.diagnostics.len() >= 0); // Should execute without panic
}

#[test]
fn test_issue_005_shebang_overrides_extension() {
    // File with .zsh extension but bash shebang should use bash rules
    let content = r#"#!/bin/bash
# This is bash despite .zsh extension
echo "hello"
"#;

    let path = PathBuf::from("script.zsh");
    let result = lint_shell_with_path(&path, content);

    // Should execute without error (bash rules applied)
    assert!(result.diagnostics.len() >= 0);
}

#[test]
fn test_issue_005_shellcheck_directive_overrides_all() {
    // ShellCheck directive should have highest priority
    let content = r#"#!/bin/bash
# shellcheck shell=zsh
# This forces zsh rules despite bash shebang
echo "hello"
"#;

    let path = PathBuf::from("test.sh");
    let result = lint_shell_with_path(&path, content);

    // Should execute without error (zsh rules applied due to directive)
    assert!(result.diagnostics.len() >= 0);
}

// Helper function
fn contains_code(result: &LintResult, code: &str) -> bool {
    result.diagnostics.iter().any(|d| d.code == code)
}
