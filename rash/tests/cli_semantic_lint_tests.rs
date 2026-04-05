#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

//! CLI integration tests for semantic lint analysis (PMAT-212)
//!
//! Tests SEM001 (unused variable) and SEM002 (undefined variable) rules
//! powered by the AST-based SemanticAnalyzer.

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
// SEM001: Unused variable detection
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT212_sem001_unused_variable() {
    let f = shell_file("#!/bin/bash\nunused=\"hello\"\necho world");
    bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("SEM001"))
        .stdout(predicate::str::contains("unused"));
}

#[test]
fn test_PMAT212_sem001_exported_variable_not_flagged() {
    let f = shell_file("#!/bin/bash\nexport API_KEY=\"secret\"\necho done");
    bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("SEM001").not());
}

#[test]
fn test_PMAT212_sem001_underscore_prefix_not_flagged() {
    let f = shell_file("#!/bin/bash\n_unused=\"ignored\"\necho done");
    bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("SEM001").not());
}

#[test]
fn test_PMAT212_sem001_multiple_unused() {
    let f = shell_file("#!/bin/bash\na=\"one\"\nb=\"two\"\necho done");
    bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("SEM001"));
}

// ---------------------------------------------------------------------------
// Clean scripts should NOT trigger SEM rules
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT212_clean_script_no_sem() {
    let f = shell_file("#!/bin/sh\necho \"hello world\"");
    let output = bashrs_cmd().arg("lint").arg(f.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("SEM001") && !stdout.contains("SEM002"),
        "Clean script should not trigger SEM rules, got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// Semantic analysis is tolerant of parse failures
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT212_parse_failure_graceful() {
    // Invalid bash syntax — semantic analysis should silently skip,
    // regex rules should still fire for what they can catch
    let f = shell_file("#!/bin/bash\nif then fi\necho $RANDOM");
    bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .assert()
        // Should still get regex-based DET001 for $RANDOM
        .stdout(predicate::str::contains("DET001"));
}

// ---------------------------------------------------------------------------
// SEM rules appear in JSON output
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT212_sem001_json_format() {
    let f = shell_file("#!/bin/bash\nunused=\"hello\"\necho world");
    bashrs_cmd()
        .arg("lint")
        .arg("--format")
        .arg("json")
        .arg(f.path())
        .assert()
        .stdout(predicate::str::contains("SEM001"))
        .stdout(predicate::str::contains("semantic analysis"));
}

// ---------------------------------------------------------------------------
// Suppression works with SEM rules
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT212_sem001_suppression() {
    let f = shell_file("#!/bin/bash\n# shellcheck disable=SEM001\nunused=\"hello\"\necho world");
    let output = bashrs_cmd().arg("lint").arg(f.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("SEM001"),
        "SEM001 should be suppressed, got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// Integration: SEM rules coexist with regex rules
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT212_sem_and_regex_coexist() {
    let f = shell_file("#!/bin/bash\nunused=\"hello\"\necho $RANDOM");
    bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .assert()
        // SEM001 from semantic analyzer
        .stdout(predicate::str::contains("SEM001"))
        // DET001 from regex rule
        .stdout(predicate::str::contains("DET001"));
}
