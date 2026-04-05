#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(deprecated)]
#![allow(non_snake_case)]
// SSC v11 CLI Integration Tests
// Tests classify, explain, fix, safety-check, and corpus SSC commands
// Uses assert_cmd (MANDATORY per CLAUDE.md)

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

fn write_temp_script(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("create temp");
    f.write_all(content.as_bytes()).expect("write temp");
    f.flush().expect("flush temp");
    f
}

// ============================================================================
// bashrs classify
// ============================================================================

#[test]
fn test_PMAT142_classify_safe_script() {
    let f = write_temp_script("#!/bin/sh\necho \"hello world\"\n");
    bashrs_cmd()
        .arg("classify")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("safe"));
}

#[test]
fn test_PMAT142_classify_unsafe_script() {
    let f = write_temp_script("#!/bin/bash\neval \"$user_input\"\n");
    bashrs_cmd()
        .arg("classify")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("unsafe"));
}

#[test]
fn test_PMAT142_classify_json_output() {
    let f = write_temp_script("#!/bin/sh\necho hello\n");
    bashrs_cmd()
        .arg("classify")
        .arg("--json")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"label\""));
}

#[test]
fn test_PMAT142_classify_makefile() {
    let f = write_temp_script("all:\n\techo hello\n");
    bashrs_cmd()
        .arg("classify")
        .arg("--format")
        .arg("makefile")
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT142_classify_dockerfile() {
    let f = write_temp_script("FROM alpine:latest\nRUN echo hello\n");
    bashrs_cmd()
        .arg("classify")
        .arg("--format")
        .arg("dockerfile")
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT152_classify_multi_label_safe() {
    let f = write_temp_script("#!/bin/sh\necho \"hello world\"\n");
    bashrs_cmd()
        .arg("classify")
        .arg("--multi-label")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("safe"));
}

#[test]
fn test_PMAT152_classify_multi_label_unsafe() {
    let f = write_temp_script("#!/bin/bash\neval \"$x\"\nmkdir /tmp/build\necho $RANDOM\n");
    bashrs_cmd()
        .arg("classify")
        .arg("--multi-label")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("unsafe"));
}

#[test]
fn test_PMAT152_classify_multi_label_json() {
    let f = write_temp_script("#!/bin/bash\neval \"$x\"\n");
    bashrs_cmd()
        .arg("classify")
        .arg("--multi-label")
        .arg("--json")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"labels\""));
}

#[test]
fn test_PMAT152_classify_nonexistent_file() {
    bashrs_cmd()
        .arg("classify")
        .arg("/nonexistent/script.sh")
        .assert()
        .failure();
}

// ============================================================================
// bashrs explain
// ============================================================================

#[test]
fn test_PMAT142_explain_safe_script() {
    let f = write_temp_script("#!/bin/sh\necho \"hello world\"\n");
    bashrs_cmd()
        .arg("explain")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SAFE"));
}

#[test]
fn test_PMAT142_explain_unsafe_script() {
    let f = write_temp_script("#!/bin/bash\neval \"$user_input\"\n");
    bashrs_cmd()
        .arg("explain")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SEC001"));
}

#[test]
fn test_PMAT142_explain_json_output() {
    let f = write_temp_script("#!/bin/bash\neval $x\n");
    bashrs_cmd()
        .arg("explain")
        .arg("--json")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"risk_level\""));
}

#[test]
fn test_PMAT142_explain_determinism_findings() {
    let f = write_temp_script("#!/bin/bash\necho $RANDOM\n");
    bashrs_cmd()
        .arg("explain")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("DET001"));
}

#[test]
fn test_PMAT142_explain_idempotency_findings() {
    let f = write_temp_script("#!/bin/sh\nmkdir /tmp/build\n");
    bashrs_cmd()
        .arg("explain")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("IDEM001"));
}

#[test]
fn test_PMAT152_explain_makefile() {
    let f = write_temp_script("all:\n\techo hello\n");
    bashrs_cmd()
        .arg("explain")
        .arg("--format")
        .arg("makefile")
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT152_explain_nonexistent_file() {
    bashrs_cmd()
        .arg("explain")
        .arg("/nonexistent/script.sh")
        .assert()
        .failure();
}

// ============================================================================
// bashrs fix
// ============================================================================

#[test]
fn test_PMAT142_fix_dry_run() {
    let f = write_temp_script("#!/bin/bash\necho $HOME\nmkdir /tmp/test\n");
    bashrs_cmd()
        .arg("fix")
        .arg("--dry-run")
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT142_fix_with_output() {
    let f = write_temp_script("#!/bin/bash\necho $HOME\n");
    let out = tempfile::NamedTempFile::new().unwrap();
    bashrs_cmd()
        .arg("fix")
        .arg("--output")
        .arg(out.path())
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT142_fix_assumptions_flag() {
    let f = write_temp_script("#!/bin/sh\nmkdir /tmp/build\n");
    bashrs_cmd()
        .arg("fix")
        .arg("--dry-run")
        .arg("--assumptions")
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT142_fix_safe_script_no_changes() {
    let f = write_temp_script("#!/bin/sh\necho \"hello\"\n");
    bashrs_cmd()
        .arg("fix")
        .arg("--dry-run")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("0 fixes"));
}

#[test]
fn test_PMAT152_fix_nonexistent_file() {
    bashrs_cmd()
        .arg("fix")
        .arg("--dry-run")
        .arg("/nonexistent/script.sh")
        .assert()
        .failure();
}

// ============================================================================
// bashrs safety-check
// ============================================================================

#[test]
fn test_PMAT142_safety_check_safe() {
    let f = write_temp_script("#!/bin/sh\necho \"hello world\"\n");
    bashrs_cmd()
        .arg("safety-check")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("safe"));
}

#[test]
fn test_PMAT142_safety_check_unsafe() {
    let f = write_temp_script("#!/bin/bash\neval \"$x\"\n");
    bashrs_cmd()
        .arg("safety-check")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("unsafe"));
}

#[test]
fn test_PMAT142_safety_check_json() {
    let f = write_temp_script("#!/bin/sh\necho hello\n");
    bashrs_cmd()
        .arg("safety-check")
        .arg("--json")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"label\""));
}

#[test]
fn test_PMAT152_safety_check_makefile() {
    let f = write_temp_script("all:\n\techo hello\n");
    bashrs_cmd()
        .arg("safety-check")
        .arg("--format")
        .arg("makefile")
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT152_safety_check_nonexistent_file() {
    bashrs_cmd()
        .arg("safety-check")
        .arg("/nonexistent/script.sh")
        .assert()
        .failure();
}

// ============================================================================
// bashrs corpus model-card
// ============================================================================

#[test]
fn test_PMAT142_corpus_model_card_stdout() {
    bashrs_cmd()
        .arg("corpus")
        .arg("model-card")
        .assert()
        .success()
        .stdout(predicate::str::contains("---"))
        .stdout(predicate::str::contains("Shell Safety"));
}

#[test]
fn test_PMAT142_corpus_model_card_to_file() {
    let out = tempfile::NamedTempFile::new().unwrap();
    bashrs_cmd()
        .arg("corpus")
        .arg("model-card")
        .arg("--output")
        .arg(out.path())
        .assert()
        .success();
    let content = std::fs::read_to_string(out.path()).unwrap();
    assert!(content.starts_with("---"));
    assert!(content.contains("license: apache-2.0"));
}

// ============================================================================
// bashrs corpus training-config
// ============================================================================

#[test]

include!("cli_ssc_tests_incl2.rs");
