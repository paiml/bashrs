#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

//! CLI integration tests for test generation (PMAT-216)

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
fn test_PMAT216_generate_in_help() {
    bashrs_cmd()
        .arg("test")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--generate"))
        .stdout(predicate::str::contains("--output"));
}

#[test]
fn test_PMAT216_generate_help_description() {
    bashrs_cmd()
        .arg("test")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("BATS"));
}

// ---------------------------------------------------------------------------
// Basic generation tests
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT216_generate_simple_script() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("#!/usr/bin/env bats"))
        .stdout(predicate::str::contains("script exists"));
}

#[test]
fn test_PMAT216_generate_detects_functions() {
    let f = shell_file("#!/bin/bash\nmy_func() {\n  echo hi\n}");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("my_func is defined"))
        .stdout(predicate::str::contains("my_func runs without error"));
}

#[test]
fn test_PMAT216_generate_detects_function_keyword() {
    let f = shell_file("#!/bin/bash\nfunction helper {\n  echo hi\n}");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("helper is defined"));
}

#[test]
fn test_PMAT216_generate_detects_args() {
    let f = shell_file("#!/bin/bash\necho \"Hello $1, you are $2\"");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("runs with 2 argument(s)"))
        .stdout(predicate::str::contains("handles missing arguments"));
}

#[test]
fn test_PMAT216_generate_detects_set_e() {
    let f = shell_file("#!/bin/bash\nset -e\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("set -e"));
}

#[test]
fn test_PMAT216_generate_syntax_check_bash() {
    let f = shell_file("#!/bin/bash\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("bash -n"));
}

#[test]
fn test_PMAT216_generate_syntax_check_sh() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("sh -n"));
}

// ---------------------------------------------------------------------------
// Output to file
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT216_generate_output_to_file() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\nmy_func() { echo hi; }");
    let out = dir.path().join("test.bats");

    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .arg("--output")
        .arg(&out)
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated"));

    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("#!/usr/bin/env bats"));
    assert!(content.contains("my_func"));
}

#[test]
fn test_PMAT216_output_requires_generate() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--output")
        .arg("/tmp/out.bats")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT216_generate_empty_script() {
    let f = shell_file("");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("script exists"));
}

#[test]
fn test_PMAT216_generate_multiple_functions() {
    let f = shell_file("#!/bin/bash\nfoo() { :; }\nbar() { :; }\nbaz() { :; }");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("foo is defined"))
        .stdout(predicate::str::contains("bar is defined"))
        .stdout(predicate::str::contains("baz is defined"));
}

// ---------------------------------------------------------------------------
// Fix verification tests (set -euo, file deps, env var filtering)
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT216_generate_detects_set_euo_pipefail() {
    let f = shell_file("#!/bin/bash\nset -euo pipefail\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("set -e"));
}

#[test]
fn test_PMAT216_generate_detects_set_eu() {
    let f = shell_file("#!/bin/bash\nset -eu\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("exits on error"));
}

#[test]
fn test_PMAT216_generate_file_dep_mid_line() {
    let f = shell_file("#!/bin/bash\nif [[ -f /etc/os-release ]]; then\n  echo found\nfi");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("/etc/os-release"));
}

#[test]
fn test_PMAT216_generate_file_dep_test_cmd() {
    let f = shell_file("#!/bin/sh\ntest -f /usr/local/bin/app && echo ok");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("/usr/local/bin/app"));
}

#[test]
fn test_PMAT216_generate_no_local_vars_as_env() {
    // local vars (lowercase) should NOT appear in env var tests
    let f = shell_file("#!/bin/bash\nmy_func() {\n  local name=\"test\"\n  echo $name\n}");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("my_func is defined"))
        // lowercase `name` must NOT appear in env var tests
        .stdout(predicate::str::contains("handles unset name").not());
}

#[test]
fn test_PMAT216_generate_uppercase_env_vars() {
    let f = shell_file("#!/bin/bash\necho $MY_CONFIG\necho $DB_HOST");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains("handles unset MY_CONFIG"))
        .stdout(predicate::str::contains("handles unset DB_HOST"));
}

#[test]
fn test_PMAT216_generate_missing_input_file() {
    bashrs_cmd()
        .arg("test")
        .arg("/nonexistent/script.sh")
        .arg("--generate")
        .assert()
        .failure();
}

#[test]
fn test_PMAT216_generate_bats_header() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Generated by: bashrs test --generate",
        ))
        .stdout(predicate::str::contains("Run with: bats"));
}

// ---------------------------------------------------------------------------
// Property test generation (PMAT-223)
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT223_generate_type_property_header() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .arg("--generate-type")
        .arg("property")
        .assert()
        .success()
        .stdout(predicate::str::contains("Property Tests"))
        .stdout(predicate::str::contains("--generate-type property"));
}

#[test]
fn test_PMAT223_property_idempotency_checks() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .arg("--generate-type")
        .arg("property")
        .assert()
        .success()
        .stdout(predicate::str::contains("idempotency: running twice"))
        .stdout(predicate::str::contains("no mkdir without -p"))
        .stdout(predicate::str::contains("no rm without -f"));
}

#[test]
fn test_PMAT223_property_determinism_checks() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .arg("--generate-type")
        .arg("property")
        .assert()
        .success()
        .stdout(predicate::str::contains("no RANDOM usage"))
        .stdout(predicate::str::contains("no process ID usage"))
        .stdout(predicate::str::contains("no date-dependent output"));
}

#[test]
fn test_PMAT223_property_posix_checks() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .arg("--generate-type")
        .arg("property")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "posix: script passes syntax check",
        ))
        .stdout(predicate::str::contains("no bashisms"))
        .stdout(predicate::str::contains("no source keyword"));
}

#[test]
fn test_PMAT223_property_safety_checks() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .arg("--generate-type")
        .arg("property")
        .assert()
        .success()
        .stdout(predicate::str::contains("no eval with variables"))
        .stdout(predicate::str::contains("no curl pipe to shell"))
        .stdout(predicate::str::contains("no chmod 777"));
}

#[test]
fn test_PMAT223_property_bash_shebang_uses_bash() {
    let f = shell_file("#!/bin/bash\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .arg("--generate-type")
        .arg("property")
        .assert()
        .success()
        .stdout(predicate::str::contains("bash -n"));
}

#[test]
fn test_PMAT223_property_output_to_file() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\necho hello");
    let out = dir.path().join("prop.bats");

    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .arg("--generate-type")
        .arg("property")
        .arg("--output")
        .arg(&out)
        .assert()
        .success();

    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("Property Tests"));
    assert!(content.contains("idempotency"));
}

// ---------------------------------------------------------------------------
// Integration test generation (PMAT-223)
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT223_generate_type_integration_header() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .arg("--generate-type")
        .arg("integration")
        .assert()
        .success()
        .stdout(predicate::str::contains("Integration Tests"))
        .stdout(predicate::str::contains("--generate-type integration"));
}

#[test]
fn test_PMAT223_integration_exit_code_tests() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("test")
        .arg(f.path())
        .arg("--generate")
        .arg("--generate-type")
        .arg("integration")
        .assert()
        .success()
        .stdout(predicate::str::contains("exits with 0 on success"))
        .stdout(predicate::str::contains("produces output on stdout"))
        .stdout(predicate::str::contains("no errors on stderr"));
}

include!("cli_test_generate_tests_incl2.rs");
