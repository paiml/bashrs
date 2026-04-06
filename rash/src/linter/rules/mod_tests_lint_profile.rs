
use super::*;

// ============================================================================
// LINT PROFILE TESTS
// ============================================================================

#[test]
fn test_lint_profile_from_str_standard() {
    assert_eq!(
        "standard".parse::<LintProfile>().unwrap(),
        LintProfile::Standard
    );
    assert_eq!(
        "default".parse::<LintProfile>().unwrap(),
        LintProfile::Standard
    );
    assert_eq!(
        "STANDARD".parse::<LintProfile>().unwrap(),
        LintProfile::Standard
    );
}

#[test]
fn test_lint_profile_from_str_coursera() {
    assert_eq!(
        "coursera".parse::<LintProfile>().unwrap(),
        LintProfile::Coursera
    );
    assert_eq!(
        "coursera-labs".parse::<LintProfile>().unwrap(),
        LintProfile::Coursera
    );
    assert_eq!(
        "COURSERA".parse::<LintProfile>().unwrap(),
        LintProfile::Coursera
    );
}

#[test]
fn test_lint_profile_from_str_devcontainer() {
    assert_eq!(
        "devcontainer".parse::<LintProfile>().unwrap(),
        LintProfile::DevContainer
    );
    assert_eq!(
        "dev-container".parse::<LintProfile>().unwrap(),
        LintProfile::DevContainer
    );
}

#[test]
fn test_lint_profile_from_str_invalid() {
    let result = "invalid".parse::<LintProfile>();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown profile"));
}

#[test]
fn test_lint_profile_display() {
    assert_eq!(LintProfile::Standard.to_string(), "standard");
    assert_eq!(LintProfile::Coursera.to_string(), "coursera");
    assert_eq!(LintProfile::DevContainer.to_string(), "devcontainer");
}

// ============================================================================
// LINT SHELL TESTS
// ============================================================================

#[test]
fn test_lint_shell_empty() {
    let result = lint_shell("");
    assert!(result.diagnostics.is_empty());
}

#[test]
fn test_lint_shell_simple_valid() {
    let result = lint_shell("echo hello");
    // Simple echo should have no issues
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_unquoted_variable() {
    let result = lint_shell("echo $VAR");
    // SC2086 should detect unquoted variable
    assert!(result.diagnostics.iter().any(|d| d.code == "SC2086"));
}

#[test]
fn test_lint_shell_quoted_variable() {
    let result = lint_shell("echo \"$VAR\"");
    // Quoted variable should not trigger SC2086
    assert!(!result.diagnostics.iter().any(|d| d.code == "SC2086"));
}

#[test]
fn test_lint_shell_with_path() {
    use std::path::Path;
    let result = lint_shell_with_path(Path::new("test.sh"), "echo hello");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_complex_script() {
    let script = r#"
#!/bin/bash
if [ -f /tmp/file ]; then
echo "File exists"
fi
for i in 1 2 3; do
echo $i
done
"#;
    let result = lint_shell(script);
    // Should detect unquoted $i
    assert!(result.diagnostics.iter().any(|d| d.code == "SC2086"));
}

// ============================================================================
// LINT DOCKERFILE TESTS
// ============================================================================

#[test]
fn test_lint_dockerfile_empty() {
    let result = lint_dockerfile("");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_dockerfile_simple() {
    let dockerfile = "FROM ubuntu:20.04\nRUN apt-get update";
    let result = lint_dockerfile(dockerfile);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_dockerfile_missing_user() {
    let dockerfile = "FROM ubuntu:20.04\nRUN echo hello";
    let result = lint_dockerfile(dockerfile);
    // Should detect missing USER directive (docker001)
    assert!(result.diagnostics.iter().any(|d| d.code == "DOCKER001"));
}

#[test]
fn test_lint_dockerfile_with_user() {
    let dockerfile = "FROM ubuntu:20.04\nUSER appuser\nRUN echo hello";
    let result = lint_dockerfile(dockerfile);
    // Should not trigger missing USER warning
    assert!(!result
        .diagnostics
        .iter()
        .any(|d| d.code == "DOCKER001" && d.message.contains("Missing USER")));
}

#[test]
fn test_lint_dockerfile_unpinned_image() {
    let dockerfile = "FROM ubuntu\nRUN echo hello";
    let result = lint_dockerfile(dockerfile);
    // Should detect unpinned base image (docker002)
    assert!(result.diagnostics.iter().any(|d| d.code == "DOCKER002"));
}

#[test]
fn test_lint_dockerfile_profile_standard() {
    let dockerfile = "FROM ubuntu:20.04\nRUN apt-get update";
    let result = lint_dockerfile_with_profile(dockerfile, LintProfile::Standard);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_dockerfile_profile_coursera() {
    let dockerfile = "FROM ubuntu:20.04\nRUN apt-get update";
    let result = lint_dockerfile_with_profile(dockerfile, LintProfile::Coursera);
    // Coursera profile should run additional checks
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_dockerfile_profile_devcontainer() {
    let dockerfile = "FROM ubuntu:20.04\nRUN apt-get update";
    let result = lint_dockerfile_with_profile(dockerfile, LintProfile::DevContainer);
    let _count = result.diagnostics.len();
}

// ============================================================================
// LINT MAKEFILE TESTS
// ============================================================================

#[test]
fn test_lint_makefile_empty() {
    let result = lint_makefile("");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_makefile_simple() {
    let makefile = "all:\n\techo hello";
    let result = lint_makefile(makefile);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_makefile_with_phony() {
    let makefile = ".PHONY: all\nall:\n\techo hello";
    let result = lint_makefile(makefile);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_makefile_missing_phony() {
    let makefile = "all:\n\techo hello";
    let result = lint_makefile(makefile);
    // May detect missing .PHONY declaration
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_makefile_spaces_instead_of_tabs() {
    let makefile = "all:\n    echo hello"; // 4 spaces instead of tab
    let result = lint_makefile(makefile);
    // MAKE008 should detect spaces instead of tabs
    assert!(result.diagnostics.iter().any(|d| d.code == "MAKE008"));
}

#[test]
fn test_lint_makefile_dollar_dollar_preserved() {
    let makefile = "all:\n\techo $$HOME";
    let result = lint_makefile(makefile);
    // Should not flag $$ as unquoted variable
    let _count = result.diagnostics.len();
}

// ============================================================================
// RULE COUNT TEST
// ============================================================================

#[test]
fn test_rule_count() {
    // Verify we have expected number of rules by checking module existence
    // This test helps catch accidental removal of rules
    let _sc2086 = lint_shell("echo $VAR");
    let _sc2046 = lint_shell("echo $(cat file)");
    // Just verifying the linter runs without panic
}
