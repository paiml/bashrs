#![allow(deprecated)]
#![allow(non_snake_case)] // test_<TASK_ID>_<feature>_<scenario> naming convention
#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)]
//! GH-227 end-to-end: SEC010/SEC014 path traversal must follow the data, not
//! the spelling of a variable name.
//!
//! * `i227a` — every path is built from string literals            -> 0 findings
//! * `i227b` — the path is built from `$1`                         -> findings kept
//! * `i227c` — a real dominating `case` guard that exits           -> 0 findings
//! * `i227d` — a function *named* `validate_path` whose body is `:` -> findings kept
//!
//! Only SEC010/SEC014 counts are asserted here. `i227c` additionally trips
//! SC2317 ("command appears unreachable") because that rule does not understand
//! that only the guard arm of a one-line `case` exits — a separate defect.

use assert_cmd::Command;
use std::io::Write;
use tempfile::NamedTempFile;

fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

const LITERAL: &str = r#"#!/bin/bash
set -euo pipefail
OUT_DIR="build/results"
mkdir -p "$OUT_DIR"
cat > "$OUT_DIR/report.md" <<'INNER'
hello
INNER
"#;

const TAINTED: &str = r#"#!/bin/bash
OUT_DIR="build/$1"
mkdir -p "$OUT_DIR"
cat > "$OUT_DIR/report.md" <<'INNER'
hello
INNER
"#;

const HARDENED: &str = r#"#!/bin/bash
case "$1" in ""|*..*|/*) echo "bad name" >&2; exit 2 ;; esac
OUT_DIR="build/$1"
mkdir -p "$OUT_DIR"
cat > "$OUT_DIR/report.md" <<'INNER'
hello
INNER
"#;

const NOOP_VALIDATOR: &str = r#"#!/bin/bash
validate_path() {
    :
}
OUT_DIR="build/$1"
validate_path "$OUT_DIR"
mkdir -p "$OUT_DIR"
cat > "$OUT_DIR/report.md" <<'INNER'
hello
INNER
"#;

/// Lint `script` and return how many times each of SEC010 / SEC014 appears.
fn path_findings(script: &str) -> (usize, usize) {
    let mut file = NamedTempFile::with_suffix(".sh").unwrap();
    file.write_all(script.as_bytes()).unwrap();
    file.flush().unwrap();

    let output = bashrs_cmd()
        .arg("lint")
        .arg("--no-ignore")
        .arg("--format")
        .arg("json")
        .arg(file.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    (
        stdout.matches("\"SEC010\"").count(),
        stdout.matches("\"SEC014\"").count(),
    )
}

#[test]
fn test_GH227_cli_literal_paths_produce_no_traversal_findings() {
    assert_eq!(path_findings(LITERAL), (0, 0));
}

#[test]
fn test_GH227_cli_positional_taint_is_still_reported() {
    assert_eq!(path_findings(TAINTED), (2, 1));
}

#[test]
fn test_GH227_cli_dominating_guard_clears_findings() {
    assert_eq!(path_findings(HARDENED), (0, 0));
}

#[test]
fn test_GH227_cli_noop_validator_does_not_clear_findings() {
    assert_eq!(path_findings(NOOP_VALIDATOR), (2, 1));
}

#[test]
fn test_GH227_cli_literal_script_exits_zero() {
    // The whole point of the ticket: a script whose paths are literals must not
    // fail a build. Before the fix this exited 2 (SEC010 was an Error).
    let mut file = NamedTempFile::with_suffix(".sh").unwrap();
    file.write_all(LITERAL.as_bytes()).unwrap();
    file.flush().unwrap();

    bashrs_cmd()
        .arg("lint")
        .arg("--no-ignore")
        .arg(file.path())
        .assert()
        .success();
}

#[test]
fn test_GH227_cli_positional_taint_exits_two() {
    let mut file = NamedTempFile::with_suffix(".sh").unwrap();
    file.write_all(TAINTED.as_bytes()).unwrap();
    file.flush().unwrap();

    bashrs_cmd()
        .arg("lint")
        .arg("--no-ignore")
        .arg(file.path())
        .assert()
        .code(2);
}
