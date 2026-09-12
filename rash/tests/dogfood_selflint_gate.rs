//! Falsification: gate S (`scripts/dogfood/selflint.sh`), the per-file
//! selflint ratchet (PMAT-263, PMAT-253 phase 4, decided 3-0 by blind
//! quorum 2026-09-11, decision D4).
//!
//! These tests run the SCRIPT, not the `bashrs` binary directly — the
//! subject under test is the ratchet comparison logic, and the installed
//! `bashrs` on PATH is a fixed oracle for "how many ERROR-severity
//! diagnostics does this file have", not something these tests re-implement.
//!
//! Every scenario runs in its own `tempfile::TempDir`, pointed at through
//! `SELFLINT_ROOT`, holding exactly the files that scenario needs — the
//! script scans every `*.sh` file under its root, so a shared fixture tree
//! would let one scenario's file affect another's verdict.

#![allow(clippy::unwrap_used)]
#![allow(non_snake_case)] // ticket-mandated test names: test_PMAT263_selflint_<scenario>

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// The repository root, found from this test binary's own manifest dir
/// (`rash/`) rather than the process's current directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rash/ has a parent directory")
        .to_path_buf()
}

fn selflint_script() -> PathBuf {
    repo_root()
        .join("scripts")
        .join("dogfood")
        .join("selflint.sh")
}

/// A shell script with exactly `n` ERROR-severity diagnostics, verified
/// against the installed `bashrs` (SC2188 "Redirection without command",
/// one per bare `>` line — not a warning, not a style nit, one error per
/// line, deterministically).
fn script_with_n_errors(n: usize) -> String {
    let mut body = String::from("#!/bin/sh\n");
    for _ in 0..n {
        body.push_str(">\n");
    }
    body
}

/// Sets up `<tmp>/scripts/dogfood/selflint-ratchet.tsv` (the path the
/// script reads, relative to `SELFLINT_ROOT`) plus whatever `*.sh` fixture
/// files the scenario names. `ratchet_rows` is `(errors, path)`; an empty
/// slice writes an empty (but present) ratchet file.
fn fixture_root(ratchet_rows: &[(usize, &str)], sh_files: &[(&str, usize)]) -> TempDir {
    let tmp = TempDir::new().expect("create tempdir");
    let ratchet_dir = tmp.path().join("scripts").join("dogfood");
    fs::create_dir_all(&ratchet_dir).expect("create scripts/dogfood/");

    let mut ratchet = String::new();
    for (errors, path) in ratchet_rows {
        ratchet.push_str(&format!("{errors}\t{path}\n"));
    }
    fs::write(ratchet_dir.join("selflint-ratchet.tsv"), ratchet).expect("write ratchet");

    for (name, errors) in sh_files {
        let path = tmp.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture parent dir");
        }
        fs::write(&path, script_with_n_errors(*errors)).expect("write fixture script");
    }

    tmp
}

#[test]
fn test_PMAT263_selflint_at_ratchet_passes() {
    let tmp = fixture_root(&[(2, "ok.sh")], &[("ok.sh", 2)]);

    Command::new("bash")
        .arg(selflint_script())
        .env("SELFLINT_ROOT", tmp.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("GATE S PASS"));
}

#[test]
fn test_PMAT263_selflint_over_ratchet_fails() {
    // 3 errors found, ratchet allows 2 — one over.
    let tmp = fixture_root(&[(2, "bad.sh")], &[("bad.sh", 3)]);

    Command::new("bash")
        .arg(selflint_script())
        .env("SELFLINT_ROOT", tmp.path())
        .assert()
        .failure()
        .stdout(
            predicates::str::contains("GATE S FAIL")
                .and(predicates::str::contains("bad.sh"))
                .and(predicates::str::contains('3'))
                .and(predicates::str::contains('2')),
        );
}

#[test]
fn test_PMAT263_selflint_under_ratchet_passes() {
    // 1 error found, ratchet allows 2 — an improvement, not a failure.
    let tmp = fixture_root(&[(2, "better.sh")], &[("better.sh", 1)]);

    Command::new("bash")
        .arg(selflint_script())
        .env("SELFLINT_ROOT", tmp.path())
        .assert()
        .success()
        .stdout(
            predicates::str::contains("GATE S PASS")
                .and(predicates::str::contains("improved"))
                .and(predicates::str::contains("better.sh")),
        );
}

#[test]
fn test_PMAT263_selflint_new_file_fails() {
    // "new.sh" carries an error but has no row in the ratchet at all.
    let tmp = fixture_root(&[], &[("new.sh", 1)]);

    Command::new("bash")
        .arg(selflint_script())
        .env("SELFLINT_ROOT", tmp.path())
        .assert()
        .failure()
        .stdout(
            predicates::str::contains("GATE S FAIL")
                .and(predicates::str::contains("new.sh"))
                .and(predicates::str::contains("new file")),
        );
}
