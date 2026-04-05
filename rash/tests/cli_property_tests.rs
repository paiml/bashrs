#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

//! CLI integration tests for bashrs property (PMAT-218)

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
fn test_PMAT218_property_in_help() {
    bashrs_cmd()
        .arg("property")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--properties"))
        .stdout(predicate::str::contains("--iterations"))
        .stdout(predicate::str::contains("--format"));
}

// ---------------------------------------------------------------------------
// Basic property tests
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT218_clean_script_passes() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("All properties satisfied"));
}

#[test]
fn test_PMAT218_idempotency_violation() {
    let f = shell_file("#!/bin/sh\nmkdir /tmp/foo");
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("FAIL"))
        .stdout(predicate::str::contains("idempotency"))
        .stdout(predicate::str::contains("mkdir"));
}

#[test]
fn test_PMAT218_determinism_violation() {
    let f = shell_file("#!/bin/sh\necho $RANDOM");
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("determinism"))
        .stdout(predicate::str::contains("$RANDOM"));
}

#[test]
fn test_PMAT218_posix_violation() {
    let f = shell_file("#!/bin/bash\n[[ -f /tmp/x ]]");
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("posix"))
        .stdout(predicate::str::contains("bashism"));
}

#[test]
fn test_PMAT218_safety_violation() {
    let f = shell_file("#!/bin/sh\neval $USER_INPUT");
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("safety"))
        .stdout(predicate::str::contains("eval"));
}

// ---------------------------------------------------------------------------
// Filter tests
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT218_filter_single_property() {
    let f = shell_file("#!/bin/sh\nmkdir /tmp/foo\necho $RANDOM");
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .arg("--properties")
        .arg("idempotency")
        .assert()
        .failure()
        .stdout(predicate::str::contains("idempotency"))
        // Should NOT test determinism when filtered
        .stdout(predicate::str::contains("determinism").not());
}

#[test]
fn test_PMAT218_filter_multiple_properties() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .arg("--properties")
        .arg("posix,safety")
        .assert()
        .success()
        .stdout(predicate::str::contains("posix"))
        .stdout(predicate::str::contains("safety"));
}

#[test]
fn test_PMAT218_filter_invalid_property() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .arg("--properties")
        .arg("invalid")
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// Output format tests
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT218_json_output() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"all_passed\": true"))
        .stdout(predicate::str::contains("\"properties\""));
}

#[test]
fn test_PMAT218_json_with_violations() {
    let f = shell_file("#!/bin/sh\nmkdir /tmp/x");
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"all_passed\": false"))
        .stdout(predicate::str::contains("\"violations\""));
}

// ---------------------------------------------------------------------------
// Iterations flag
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT218_iterations_flag() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .arg("--iterations")
        .arg("50")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"iterations\": 50"));
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT218_missing_file() {
    bashrs_cmd()
        .arg("property")
        .arg("/nonexistent/script.sh")
        .assert()
        .failure();
}

#[test]
fn test_PMAT218_empty_script() {
    let f = shell_file("");
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT218_suggestions_shown() {
    let f = shell_file("#!/bin/sh\nmkdir /tmp/foo");
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("suggestion"))
        .stdout(predicate::str::contains("mkdir -p"));
}

// ===========================================================================
// Custom Property DSL tests (PMAT-225)
// ===========================================================================

fn toml_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn test_PMAT225_custom_flag_in_help() {
    bashrs_cmd()
        .arg("property")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--custom"));
}

#[test]
fn test_PMAT225_custom_forbid_pass() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\necho hello");
    let toml = toml_file(
        &dir,
        "props.toml",
        r#"
[[property]]
name = "no-sudo"
description = "No sudo usage"

[[property.rule]]
forbid = "sudo "
message = "sudo detected"
"#,
    );
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .arg("--custom")
        .arg(&toml)
        .assert()
        .success()
        .stdout(predicate::str::contains("Custom Properties"))
        .stdout(predicate::str::contains("PASS"))
        .stdout(predicate::str::contains("no-sudo"));
}

#[test]
fn test_PMAT225_custom_forbid_fail() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\nsudo rm -rf /tmp/x");
    let toml = toml_file(
        &dir,
        "props.toml",
        r#"
[[property]]
name = "no-sudo"
description = "No sudo usage"

[[property.rule]]
forbid = "sudo "
message = "sudo detected"
suggestion = "Run as root instead"
"#,
    );
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .arg("--custom")
        .arg(&toml)
        .assert()
        .failure()
        .stdout(predicate::str::contains("FAIL"))
        .stdout(predicate::str::contains("sudo detected"))
        .stdout(predicate::str::contains("Run as root"));
}

#[test]
fn test_PMAT225_custom_require_pass() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\nset -e\necho hello");
    let toml = toml_file(
        &dir,
        "props.toml",
        r#"
[[property]]
name = "needs-set-e"
description = "Must use set -e"

[[property.rule]]
require = "^set -e"
message = "set -e is required"
"#,
    );
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .arg("--custom")
        .arg(&toml)
        .assert()
        .success()
        .stdout(predicate::str::contains("PASS"))
        .stdout(predicate::str::contains("needs-set-e"));
}

#[test]
fn test_PMAT225_custom_require_fail() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\necho hello");
    let toml = toml_file(
        &dir,
        "props.toml",
        r#"
[[property]]
name = "needs-set-e"
description = "Must use set -e"

[[property.rule]]
require = "^set -e"
message = "set -e is required"
suggestion = "Add set -e after the shebang"
"#,
    );
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .arg("--custom")
        .arg(&toml)
        .assert()
        .failure()
        .stdout(predicate::str::contains("FAIL"))
        .stdout(predicate::str::contains("set -e is required"))
        .stdout(predicate::str::contains("Add set -e"));
}

#[test]
fn test_PMAT225_custom_multiple_properties() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\nset -e\necho hello");
    let toml = toml_file(
        &dir,
        "props.toml",
        r#"
[[property]]
name = "no-sudo"

[[property.rule]]
forbid = "sudo "

[[property]]
name = "needs-set-e"

[[property.rule]]
require = "^set -e"
"#,
    );
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .arg("--custom")
        .arg(&toml)
        .assert()
        .success()
        .stdout(predicate::str::contains("no-sudo"))
        .stdout(predicate::str::contains("needs-set-e"));
}

#[test]
fn test_PMAT225_custom_regex_pattern() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\nPASSWORD=secret123");
    let toml = toml_file(
        &dir,
        "props.toml",
        r#"
[[property]]
name = "no-hardcoded-secrets"
description = "No hardcoded passwords"

[[property.rule]]
forbid = "(?i)password\\s*="
message = "Hardcoded password detected"
"#,
    );
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .arg("--custom")
        .arg(&toml)
        .assert()
        .failure()
        .stdout(predicate::str::contains("Hardcoded password"));
}

#[test]
fn test_PMAT225_custom_json_output() {
    let dir = TempDir::new().unwrap();
    let f = shell_file("#!/bin/sh\nsudo rm -rf /tmp/x");
    let toml = toml_file(
        &dir,
        "props.toml",
        r#"
[[property]]
name = "no-sudo"

[[property.rule]]
forbid = "sudo "
message = "sudo detected"
"#,
    );
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .arg("--custom")
        .arg(&toml)
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"custom_properties\""))
        .stdout(predicate::str::contains("\"no-sudo\""))
        .stdout(predicate::str::contains("\"sudo detected\""));
}

#[test]
fn test_PMAT225_custom_json_no_custom() {
    // Without --custom, JSON should not have custom_properties key
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("property")
        .arg(f.path())
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"custom_properties\"").not());
}

include!("cli_property_tests_incl2.rs");
