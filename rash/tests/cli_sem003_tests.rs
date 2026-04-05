#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

//! CLI integration tests for SEM003: dead code detection (PMAT-213)

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[allow(deprecated)]
fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

fn shell_file(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".sh").unwrap();
    writeln!(f, "{content}").unwrap();
    f
}

// ---------------------------------------------------------------------------
// Positive cases: SEM003 should fire
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT213_exit_then_code() {
    let f = shell_file("#!/bin/bash\necho start\nexit 0\necho unreachable");
    bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("SEM003"))
        .stdout(predicate::str::contains("Unreachable"));
}

#[test]
fn test_PMAT213_return_then_code() {
    let f = shell_file("#!/bin/bash\nreturn 1\necho unreachable");
    bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("SEM003"));
}

#[test]
fn test_PMAT213_exec_then_code() {
    let f = shell_file("#!/bin/bash\nexec /bin/sh\necho unreachable");
    bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("SEM003"));
}

#[test]
fn test_PMAT213_exit_no_code_then_statement() {
    let f = shell_file("#!/bin/bash\nexit\necho dead");
    bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("SEM003"));
}

// ---------------------------------------------------------------------------
// Negative cases: SEM003 should NOT fire
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT213_exit_last_line() {
    let f = shell_file("#!/bin/bash\necho hello\nexit 0");
    let output = bashrs_cmd().arg("lint").arg(f.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("SEM003"),
        "Exit as last statement should not trigger SEM003"
    );
}

#[test]
fn test_PMAT213_exit_in_if_block() {
    let f = shell_file("#!/bin/bash\nif [ -f /tmp/x ]; then\n  exit 1\nfi\necho reachable");
    let output = bashrs_cmd().arg("lint").arg(f.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("SEM003"),
        "Code after if-block with exit should NOT be flagged"
    );
}

#[test]
fn test_PMAT213_exit_in_while_block() {
    let f = shell_file("#!/bin/bash\nwhile true; do\n  exit 1\ndone\necho reachable");
    let output = bashrs_cmd().arg("lint").arg(f.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("SEM003"),
        "Code after while-block with exit should NOT be flagged"
    );
}

#[test]
fn test_PMAT213_no_exit_no_warning() {
    let f = shell_file("#!/bin/sh\necho hello\necho world");
    let output = bashrs_cmd().arg("lint").arg(f.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("SEM003"));
}

#[test]
fn test_PMAT213_exit_variable_not_flagged() {
    let f = shell_file("#!/bin/bash\nexit_code=1\necho $exit_code");
    let output = bashrs_cmd().arg("lint").arg(f.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("SEM003"),
        "exit_code= should not trigger SEM003"
    );
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT213_comments_after_exit() {
    let f = shell_file("#!/bin/bash\nexit 0\n# just a comment");
    let output = bashrs_cmd().arg("lint").arg(f.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("SEM003"),
        "Comments after exit should not trigger SEM003"
    );
}

#[test]
fn test_PMAT213_json_output() {
    let f = shell_file("#!/bin/bash\nexit 0\necho dead");
    bashrs_cmd()
        .arg("lint")
        .arg("--format")
        .arg("json")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("SEM003"));
}

#[test]
fn test_PMAT213_suppression() {
    let f = shell_file("#!/bin/bash\n# shellcheck disable=SEM003\nexit 0\necho suppressed");
    let output = bashrs_cmd().arg("lint").arg(f.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("SEM003"), "SEM003 should be suppressed");
}
