#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

//! CLI integration tests for score trend tracking (PMAT-215)

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

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
// Help and CLI arg tests
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT215_score_trend_in_help() {
    bashrs_cmd()
        .arg("score")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--trend"))
        .stdout(predicate::str::contains("--no-save"));
}

#[test]
fn test_PMAT215_trend_help_description() {
    bashrs_cmd()
        .arg("score")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("score trend"))
        .stdout(predicate::str::contains("scores.jsonl"));
}

// ---------------------------------------------------------------------------
// Score saving tests (use tempdir to avoid polluting working dir)
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT215_score_saves_to_history() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\necho hello");

    // Run score in the temp dir so .bashrs/scores.jsonl is created there
    bashrs_cmd()
        .arg("score")
        .arg(f.path())
        .current_dir(dir.path())
        .assert()
        .success();

    // Check that scores.jsonl was created
    let history_path = dir.path().join(".bashrs/scores.jsonl");
    assert!(
        history_path.exists(),
        "Score history file should be created"
    );

    let content = std::fs::read_to_string(&history_path).unwrap();
    assert!(!content.is_empty(), "History should have content");
    assert!(
        content.contains("score"),
        "History should contain score data"
    );
    assert!(
        content.contains("grade"),
        "History should contain grade data"
    );
}

#[test]
fn test_PMAT215_no_save_flag() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\necho hello");

    bashrs_cmd()
        .arg("score")
        .arg(f.path())
        .arg("--no-save")
        .current_dir(dir.path())
        .assert()
        .success();

    let history_path = dir.path().join(".bashrs/scores.jsonl");
    assert!(
        !history_path.exists(),
        "Score history should NOT be created with --no-save"
    );
}

// ---------------------------------------------------------------------------
// Trend display tests
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT215_trend_no_history() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\necho hello");

    bashrs_cmd()
        .arg("score")
        .arg(f.path())
        .arg("--trend")
        .arg("5")
        .arg("--no-save")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No score history found"));
}

#[test]
fn test_PMAT215_trend_shows_entries() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\necho hello");

    // Run score 3 times to build history
    for _ in 0..3 {
        bashrs_cmd()
            .arg("score")
            .arg(f.path())
            .current_dir(dir.path())
            .assert()
            .success();
    }

    // Now request trend
    bashrs_cmd()
        .arg("score")
        .arg(f.path())
        .arg("--trend")
        .arg("5")
        .arg("--no-save")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Score Trend"))
        .stdout(predicate::str::contains("entries"));
}

#[test]
fn test_PMAT215_trend_limits_entries() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\necho hello");

    // Run score 5 times
    for _ in 0..5 {
        bashrs_cmd()
            .arg("score")
            .arg(f.path())
            .current_dir(dir.path())
            .assert()
            .success();
    }

    // Request only last 2
    bashrs_cmd()
        .arg("score")
        .arg(f.path())
        .arg("--trend")
        .arg("2")
        .arg("--no-save")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("2 entries"));
}

// ---------------------------------------------------------------------------
// JSON format with trend
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT215_score_json_still_works() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\necho hello");

    bashrs_cmd()
        .arg("score")
        .arg(f.path())
        .arg("--format")
        .arg("json")
        .arg("--no-save")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("score"));
}

// ---------------------------------------------------------------------------
// Score preserves existing behavior
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT215_score_basic_still_works() {
    let f = shell_file("#!/bin/sh\necho hello");

    bashrs_cmd()
        .arg("score")
        .arg(f.path())
        .arg("--no-save")
        .assert()
        .success()
        .stdout(predicate::str::contains("Quality Score"));
}

#[test]
fn test_PMAT215_score_grade_flag() {
    let f = shell_file("#!/bin/sh\necho hello");

    bashrs_cmd()
        .arg("score")
        .arg(f.path())
        .arg("--grade")
        .arg("--no-save")
        .assert()
        .success()
        .stdout(predicate::str::contains("Grade:"));
}
