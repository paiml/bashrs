#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

//! CLI integration tests for `bashrs lint --changed` (PMAT-210)
//!
//! Uses assert_cmd (MANDATORY per CLAUDE.md).
//! Tests incremental linting: only lint git-changed shell files.

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::TempDir;

#[allow(deprecated)]
fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

// ---------------------------------------------------------------------------
// Help and CLI arg validation
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT210_lint_changed_in_help() {
    bashrs_cmd()
        .arg("lint")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--changed"))
        .stdout(predicate::str::contains("--since"));
}

#[test]
fn test_PMAT210_lint_changed_help_shows_description() {
    bashrs_cmd()
        .arg("lint")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Only lint files changed in git"))
        .stdout(predicate::str::contains("CI optimization"));
}

#[test]
fn test_PMAT210_lint_since_requires_changed() {
    // --since without --changed should fail
    bashrs_cmd()
        .arg("lint")
        .arg("--since")
        .arg("main")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_PMAT210_lint_changed_no_files_required() {
    // --changed should not require positional FILE args
    // (it discovers files from git)
    // This will succeed or fail based on git state, but should NOT
    // fail with "required arguments not provided"
    let result = bashrs_cmd().arg("lint").arg("--changed").assert();
    // Should not fail with missing FILE argument
    let output = String::from_utf8_lossy(&result.get_output().stderr);
    assert!(
        !output.contains("required arguments were not provided"),
        "Should not require FILE when --changed is used"
    );
}

// ---------------------------------------------------------------------------
// Git repo tests (uses a temporary git repo)
// ---------------------------------------------------------------------------

/// Create a temporary git repo with an initial commit.
fn setup_git_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let path = dir.path();

    // Init repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .unwrap();

    // Configure git user (needed for commit)
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(path)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(path)
        .output()
        .unwrap();

    // Create initial file and commit
    std::fs::write(path.join("clean.sh"), "#!/bin/sh\necho \"hello\"\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "clean.sh"])
        .current_dir(path)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(path)
        .output()
        .unwrap();

    dir
}

#[test]
fn test_PMAT210_lint_changed_no_changes() {
    let dir = setup_git_repo();

    bashrs_cmd()
        .arg("lint")
        .arg("--changed")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No changed shell files found"));
}

#[test]
fn test_PMAT210_lint_changed_with_dirty_file() {
    let dir = setup_git_repo();

    // Modify the committed file to have lint issues
    std::fs::write(dir.path().join("clean.sh"), "#!/bin/bash\necho $RANDOM\n").unwrap();

    bashrs_cmd()
        .arg("lint")
        .arg("--changed")
        .current_dir(dir.path())
        .assert()
        .failure() // lint finds errors
        .stdout(predicate::str::contains("DET001"));
}

#[test]
fn test_PMAT210_lint_changed_with_untracked_file() {
    let dir = setup_git_repo();

    // Add a new untracked shell file
    std::fs::write(
        dir.path().join("new_script.sh"),
        "#!/bin/bash\necho $RANDOM\n",
    )
    .unwrap();

    bashrs_cmd()
        .arg("lint")
        .arg("--changed")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("new_script.sh"));
}

#[test]
fn test_PMAT210_lint_changed_ignores_non_shell() {
    let dir = setup_git_repo();

    // Add a non-shell file — should not be linted
    std::fs::write(dir.path().join("readme.md"), "# Hello\n").unwrap();

    bashrs_cmd()
        .arg("lint")
        .arg("--changed")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No changed shell files found"));
}

#[test]
fn test_PMAT210_lint_changed_with_staged_file() {
    let dir = setup_git_repo();

    // Create and stage a new file with issues
    std::fs::write(dir.path().join("staged.sh"), "#!/bin/bash\necho $RANDOM\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "staged.sh"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    bashrs_cmd()
        .arg("lint")
        .arg("--changed")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("staged.sh"));
}

#[test]
fn test_PMAT210_lint_changed_with_makefile() {
    let dir = setup_git_repo();

    // Add a Makefile (should be picked up by --changed)
    let mut f = std::fs::File::create(dir.path().join("Makefile")).unwrap();
    writeln!(f, "all:\n\techo hello").unwrap();

    // Makefile lint may find warnings (MAKE003, MAKE013, etc.), so don't assert success.
    // Just verify the file was discovered and linted.
    bashrs_cmd()
        .arg("lint")
        .arg("--changed")
        .current_dir(dir.path())
        .assert()
        .stdout(
            predicate::str::contains("Found 1 changed file")
                .or(predicate::str::contains("Makefile")),
        );
}

#[test]
fn test_PMAT210_lint_changed_since_default_is_head() {
    bashrs_cmd()
        .arg("lint")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("[default: HEAD]"));
}

#[test]
fn test_PMAT210_lint_changed_not_a_git_repo() {
    let dir = TempDir::new().unwrap();
    // No git init — should fail with clear error
    std::fs::write(dir.path().join("script.sh"), "#!/bin/sh\necho hi\n").unwrap();

    bashrs_cmd()
        .arg("lint")
        .arg("--changed")
        .current_dir(dir.path())
        .assert()
        .failure();
}

#[test]
fn test_PMAT210_lint_changed_with_fix() {
    let dir = setup_git_repo();

    // Add a file with fixable issues
    std::fs::write(
        dir.path().join("fixable.sh"),
        "#!/bin/sh\nmkdir /tmp/test\n",
    )
    .unwrap();

    // --changed --fix should work together
    bashrs_cmd()
        .arg("lint")
        .arg("--changed")
        .arg("--fix")
        .current_dir(dir.path())
        .assert()
        .stdout(predicate::str::contains("IDEM001").or(predicate::str::contains("fixable.sh")));
}

#[test]
fn test_PMAT210_lint_changed_plus_explicit_files() {
    let dir = setup_git_repo();

    // Create an explicit file to lint
    std::fs::write(dir.path().join("explicit.sh"), "#!/bin/sh\necho hello\n").unwrap();

    // --changed plus explicit FILE args should lint both
    bashrs_cmd()
        .arg("lint")
        .arg("--changed")
        .arg(dir.path().join("explicit.sh"))
        .current_dir(dir.path())
        .assert()
        .success();
}
