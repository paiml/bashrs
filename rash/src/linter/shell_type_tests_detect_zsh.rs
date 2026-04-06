
use super::*;
use std::path::PathBuf;

// ===== Shebang Detection Tests =====

#[test]
fn test_detect_zsh_from_shebang_env() {
    let content = "#!/usr/bin/env zsh\necho hello";
    let path = PathBuf::from("test.sh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Zsh);
}

#[test]
fn test_detect_zsh_from_shebang_direct() {
    let content = "#!/bin/zsh\necho hello";
    let path = PathBuf::from("test.sh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Zsh);
}

#[test]
fn test_detect_bash_from_shebang() {
    let content = "#!/bin/bash\necho hello";
    let path = PathBuf::from("test.sh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Bash);
}

#[test]
fn test_detect_sh_from_shebang() {
    let content = "#!/bin/sh\necho hello";
    let path = PathBuf::from("test.sh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Sh);
}

#[test]
fn test_detect_ksh_from_shebang() {
    let content = "#!/bin/ksh\necho hello";
    let path = PathBuf::from("test.sh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Ksh);
}

// ===== ShellCheck Directive Tests =====

#[test]
fn test_detect_zsh_from_shellcheck_directive() {
    let content = "# shellcheck shell=zsh\necho hello";
    let path = PathBuf::from("test.sh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Zsh);
}

#[test]
fn test_detect_bash_from_shellcheck_directive() {
    let content = "# shellcheck shell=bash\necho hello";
    let path = PathBuf::from("test.sh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Bash);
}

#[test]
fn test_shellcheck_directive_with_whitespace() {
    let content = "#   shellcheck   shell=zsh  \necho hello";
    let path = PathBuf::from("test.sh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Zsh);
}

// ===== File Extension Tests =====

#[test]
fn test_detect_zsh_from_zshrc() {
    let content = "echo hello";
    let path = PathBuf::from(".zshrc");
    assert_eq!(detect_shell_type(&path, content), ShellType::Zsh);
}

#[test]
fn test_detect_zsh_from_zsh_extension() {
    let content = "echo hello";
    let path = PathBuf::from("script.zsh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Zsh);
}

#[test]
fn test_detect_zsh_from_zshenv() {
    let content = "echo hello";
    let path = PathBuf::from(".zshenv");
    assert_eq!(detect_shell_type(&path, content), ShellType::Zsh);
}

#[test]
fn test_detect_zsh_from_zprofile() {
    let content = "echo hello";
    let path = PathBuf::from(".zprofile");
    assert_eq!(detect_shell_type(&path, content), ShellType::Zsh);
}

#[test]
fn test_detect_bash_from_bashrc() {
    let content = "echo hello";
    let path = PathBuf::from(".bashrc");
    assert_eq!(detect_shell_type(&path, content), ShellType::Bash);
}

#[test]
fn test_detect_bash_from_bash_profile() {
    let content = "echo hello";
    let path = PathBuf::from(".bash_profile");
    assert_eq!(detect_shell_type(&path, content), ShellType::Bash);
}

// ===== Priority Tests =====

#[test]
fn test_shebang_overrides_extension() {
    // .zsh extension but bash shebang → bash wins
    let content = "#!/bin/bash\necho hello";
    let path = PathBuf::from("script.zsh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Bash);
}

#[test]
fn test_shellcheck_directive_overrides_shebang() {
    // bash shebang but zsh directive → directive wins
    let content = "#!/bin/bash\n# shellcheck shell=zsh\necho hello";
    let path = PathBuf::from("test.sh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Zsh);
}

// ===== Default Tests =====

#[test]
fn test_default_to_bash() {
    let content = "echo hello";
    let path = PathBuf::from("script.sh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Bash);
}

#[test]
fn test_no_extension_defaults_to_bash() {
    let content = "echo hello";
    let path = PathBuf::from("script");
    assert_eq!(detect_shell_type(&path, content), ShellType::Bash);
}

// ===== Edge Cases =====

#[test]
fn test_empty_file_defaults_to_bash() {
    let content = "";
    let path = PathBuf::from("test.sh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Bash);
}

#[test]
fn test_shebang_with_spaces() {
    let content = "#! /usr/bin/env zsh\necho hello";
    let path = PathBuf::from("test.sh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Zsh);
}

#[test]
fn test_to_shellcheck_name() {
    assert_eq!(ShellType::Bash.to_shellcheck_name(), "bash");
    assert_eq!(ShellType::Zsh.to_shellcheck_name(), "zsh");
    assert_eq!(ShellType::Sh.to_shellcheck_name(), "sh");
    assert_eq!(ShellType::Ksh.to_shellcheck_name(), "ksh");
    assert_eq!(ShellType::Auto.to_shellcheck_name(), "auto");
}

// ===== Mutation Test Coverage (RED phase - add failing tests) =====

#[test]
fn test_detect_bash_from_bash_login() {
    // MUTATION: Deleting .bash_login from match arm should cause this to fail
    let content = "echo hello";
    let path = PathBuf::from(".bash_login");
    assert_eq!(detect_shell_type(&path, content), ShellType::Bash);
}

#[test]
fn test_detect_bash_from_bash_logout() {
    // MUTATION: Deleting .bash_logout from match arm should cause this to fail
    let content = "echo hello";
    let path = PathBuf::from(".bash_logout");
    assert_eq!(detect_shell_type(&path, content), ShellType::Bash);
}

#[test]
fn test_detect_bash_from_bash_extension() {
    // MUTATION: Deleting "bash" from path extension detection should fail
    let content = "echo hello";
    let path = PathBuf::from("script.bash");
    assert_eq!(detect_shell_type(&path, content), ShellType::Bash);
}

#[test]
fn test_detect_ksh_from_ksh_extension() {
    // MUTATION: Deleting "ksh" from path extension detection should fail
    let content = "echo hello";
    let path = PathBuf::from("script.ksh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Ksh);
}

#[test]
fn test_detect_auto_from_shellcheck_directive() {
    // MUTATION: Deleting "auto" from shellcheck directive should fail
    let content = "# shellcheck shell=auto\necho hello";
    let path = PathBuf::from("test.sh");
    assert_eq!(detect_shell_type(&path, content), ShellType::Auto);
}

#[test]
fn test_shellcheck_directive_requires_all_conditions() {
    // MUTATION: Changing && to || in line 93 should cause this to fail
    // This verifies that ALL conditions must be met (starts with #, contains shellcheck, contains shell=)

    // Missing "shellcheck" keyword - should NOT match
    let content_no_shellcheck = "# shell=zsh\necho hello";
    let path = PathBuf::from("test.sh");
    assert_eq!(
        detect_shell_type(&path, content_no_shellcheck),
        ShellType::Bash
    ); // Defaults to Bash

    // Missing "shell=" keyword - should NOT match
    let content_no_shell_equals = "# shellcheck disable=SC2086\necho hello";
    assert_eq!(
        detect_shell_type(&path, content_no_shell_equals),
        ShellType::Bash
    ); // Defaults to Bash

    // Not a comment (missing #) - should NOT match
    let content_no_hash = "shellcheck shell=zsh\necho hello";
    assert_eq!(detect_shell_type(&path, content_no_hash), ShellType::Bash); // Defaults to Bash
}

#[test]
fn test_shellcheck_directive_bash_detection() {
    // MUTATION: Deleting "bash" match arm in shellcheck directive should fail
    // Verify bash is explicitly detected (not just defaulted to)
    let content = "# shellcheck shell=bash\necho hello";
    let path = PathBuf::from("test.zsh"); // Conflicting extension
                                          // ShellCheck directive should override extension
    assert_eq!(detect_shell_type(&path, content), ShellType::Bash);
}

// ===== Property Tests =====

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_detect_always_returns_valid_shell_type(
            content in ".*",
            filename in "[a-zA-Z0-9_.-]{1,20}"
        ) {
            let path = PathBuf::from(&filename);
            let shell_type = detect_shell_type(&path, &content);

            // Should always return a valid shell type (never panic)
            prop_assert!(matches!(
                shell_type,
                ShellType::Bash | ShellType::Zsh | ShellType::Sh | ShellType::Ksh | ShellType::Auto
            ));
        }

        #[test]
        fn prop_shebang_detection_consistent(
            shell in "(bash|zsh|sh|ksh)"
        ) {
            let content = format!("#!/usr/bin/env {}\necho test", shell);
            let path = PathBuf::from("test.sh");
            let detected = detect_shell_type(&path, &content);

            // Verify detection matches shebang
            match shell.as_str() {
                "bash" => prop_assert_eq!(detected, ShellType::Bash),
                "zsh" => prop_assert_eq!(detected, ShellType::Zsh),
                "sh" => prop_assert_eq!(detected, ShellType::Sh),
                "ksh" => prop_assert_eq!(detected, ShellType::Ksh),
                _ => unreachable!(),
            }
        }

        #[test]
        fn prop_zshrc_always_detected_as_zsh(
            content in ".*"
        ) {
            let path = PathBuf::from(".zshrc");
            let detected = detect_shell_type(&path, &content);

            // Unless there's a conflicting shebang, .zshrc should be zsh
            if !content.starts_with("#!/") {
                prop_assert_eq!(detected, ShellType::Zsh);
            }
        }

        #[test]
        fn prop_bashrc_always_detected_as_bash(
            content in ".*"
        ) {
            let path = PathBuf::from(".bashrc");
            let detected = detect_shell_type(&path, &content);

            // Unless there's a conflicting shebang, .bashrc should be bash
            if !content.starts_with("#!/") {
                prop_assert_eq!(detected, ShellType::Bash);
            }
        }

        #[test]
        fn prop_shellcheck_directive_overrides_everything(
            shell in "(bash|zsh|sh|ksh)",
            filename in "[a-zA-Z0-9_.-]{1,20}"
        ) {
            let content = format!("# shellcheck shell={}\necho test", shell);
            let path = PathBuf::from(&filename);
            let detected = detect_shell_type(&path, &content);

            // ShellCheck directive should always win
            match shell.as_str() {
                "bash" => prop_assert_eq!(detected, ShellType::Bash),
                "zsh" => prop_assert_eq!(detected, ShellType::Zsh),
                "sh" => prop_assert_eq!(detected, ShellType::Sh),
                "ksh" => prop_assert_eq!(detected, ShellType::Ksh),
                _ => unreachable!(),
            }
        }

        #[test]
        fn prop_unknown_files_default_to_bash(
            content in "[^#]*", // Content not starting with #
            filename in "[a-zA-Z0-9_]{1,20}" // No extension
        ) {
            let path = PathBuf::from(&filename);
            let detected = detect_shell_type(&path, &content);

            // Unknown files should default to bash
            prop_assert_eq!(detected, ShellType::Bash);
        }
    }
}
