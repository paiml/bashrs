#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
// Integration test for Issue #5: zsh shell type detection
// Verifies that bashrs correctly detects zsh files

use bashrs::linter::{detect_shell_type, ShellType};
use std::path::PathBuf;

#[test]
fn test_issue_005_zshrc_detected_as_zsh() {
    // Issue #5: .zshrc should be detected as zsh, not bash
    let content = r#"#!/usr/bin/env zsh
# Test .zshrc file

# zsh array splitting (valid zsh syntax that bash might flag)
filtered_args=("${(@f)"$(echo -e "line1\nline2")"}")

echo "Array: ${filtered_args[*]}"
"#;

    let path = PathBuf::from(".zshrc");
    let shell_type = detect_shell_type(&path, content);

    assert_eq!(
        shell_type,
        ShellType::Zsh,
        "Issue #5: .zshrc should be detected as zsh"
    );
}

#[test]
fn test_issue_005_bash_shebang_overrides_zsh_extension() {
    // Issue #5: Shebang should have priority over extension
    let content = r#"#!/bin/bash
# This is actually a bash script with .zsh extension

echo "Hello from bash"
"#;

    let path = PathBuf::from("script.zsh");
    let shell_type = detect_shell_type(&path, content);

    assert_eq!(
        shell_type,
        ShellType::Bash,
        "Issue #5: bash shebang should override .zsh extension"
    );
}

#[test]
fn test_issue_005_shellcheck_directive_highest_priority() {
    // Issue #5: ShellCheck directive should override everything
    let content = r#"#!/bin/bash
# shellcheck shell=zsh
# This tells linters to treat it as zsh

echo "Hello"
"#;

    let path = PathBuf::from("script.sh");
    let shell_type = detect_shell_type(&path, content);

    assert_eq!(
        shell_type,
        ShellType::Zsh,
        "Issue #5: shellcheck directive should have highest priority"
    );
}

#[test]
fn test_issue_005_zsh_files_detected() {
    // Test all common zsh config files
    let test_cases = vec![
        (".zshrc", "echo hello"),
        (".zshenv", "export PATH=/bin"),
        (".zprofile", "echo profile"),
        ("script.zsh", "echo script"),
    ];

    for (filename, content) in test_cases {
        let path = PathBuf::from(filename);
        let shell_type = detect_shell_type(&path, content);

        assert_eq!(
            shell_type,
            ShellType::Zsh,
            "Issue #5: {} should be detected as zsh",
            filename
        );
    }
}

#[test]
fn test_issue_005_bash_files_still_work() {
    // Ensure bash detection still works (no regression)
    let test_cases = vec![
        (".bashrc", "echo hello"),
        (".bash_profile", "export PATH=/bin"),
        ("script.sh", "#!/bin/bash\necho script"),
    ];

    for (filename, content) in test_cases {
        let path = PathBuf::from(filename);
        let shell_type = detect_shell_type(&path, content);

        assert_eq!(
            shell_type,
            ShellType::Bash,
            "Issue #5: {} should be detected as bash (no regression)",
            filename
        );
    }
}

#[test]
fn test_issue_005_default_to_bash() {
    // Files with no indicators default to bash
    let content = "echo hello";
    let path = PathBuf::from("script");

    let shell_type = detect_shell_type(&path, content);

    assert_eq!(
        shell_type,
        ShellType::Bash,
        "Issue #5: unknown files should default to bash"
    );
}

#[test]
fn test_issue_005_real_zsh_syntax() {
    // Real zsh syntax from the GitHub issue
    let zsh_content = r#"#!/usr/bin/env zsh

filter_region_args() {
    for arg in "$@"; do
        if [[ "$arg" != --region=* ]]; then
            echo "$arg"
        fi
    done
}

# This is valid zsh syntax that bash might flag with SC2296
filtered_args=("${(@f)"$(filter_region_args "${@}")"}")

echo "Filtered args: ${filtered_args[*]}"
"#;

    let path = PathBuf::from(".zshrc");
    let shell_type = detect_shell_type(&path, zsh_content);

    assert_eq!(
        shell_type,
        ShellType::Zsh,
        "Issue #5: Real zsh syntax should be detected as zsh"
    );
}
