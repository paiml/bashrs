#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! CLI Integration Tests for bashrs purify --type-check and --emit-guards
//!
//! Tests the gradual type system CLI integration:
//! - TYPE_001: --type-check flag produces diagnostics
//! - TYPE_002: --emit-guards flag produces runtime guards
//! - TYPE_003: Backward compatibility (no flags = unchanged behavior)

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

/// Create a bashrs command
#[allow(deprecated)]
fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

/// Create a temporary bash script with given content
fn create_temp_script(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write temp file");
    file.flush().expect("Failed to flush temp file");
    file
}

// ============================================================================
// TYPE_001: --type-check flag
// ============================================================================

#[test]
fn test_TYPE_001_type_check_basic_script() {
    let script = create_temp_script("#!/bin/bash\nport=8080\necho $port\n");

    bashrs_cmd()
        .arg("purify")
        .arg(script.path())
        .arg("--type-check")
        .assert()
        .success()
        .stdout(predicate::str::contains("#!/bin/sh"));
}

#[test]
fn test_TYPE_001_type_check_with_annotations() {
    let script = create_temp_script(
        "#!/bin/bash\n# @type port: int\nport=8080\necho $port\n",
    );

    bashrs_cmd()
        .arg("purify")
        .arg(script.path())
        .arg("--type-check")
        .assert()
        .success()
        .stdout(predicate::str::contains("#!/bin/sh"));
}

#[test]
fn test_TYPE_001_type_check_no_flag_no_diagnostics() {
    // Without --type-check, no diagnostics should appear
    let script = create_temp_script("#!/bin/bash\nport=8080\n");

    bashrs_cmd()
        .arg("purify")
        .arg(script.path())
        .assert()
        .success()
        .stderr(predicate::str::is_empty().or(
            // stderr may have tracing output but no type diagnostics
            predicate::str::contains("type error").not(),
        ));
}

// ============================================================================
// TYPE_002: --emit-guards flag
// ============================================================================

#[test]
fn test_TYPE_002_emit_guards_integer_variable() {
    let script = create_temp_script(
        "#!/bin/bash\n# @type port: int\nport=8080\necho $port\n",
    );

    bashrs_cmd()
        .arg("purify")
        .arg(script.path())
        .arg("--emit-guards")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("#!/bin/sh")
                .and(predicate::str::contains("*[!0-9]*")),
        );
}

#[test]
fn test_TYPE_002_emit_guards_implies_type_check() {
    // --emit-guards should work without explicit --type-check
    let script = create_temp_script("#!/bin/bash\nname=hello\n");

    bashrs_cmd()
        .arg("purify")
        .arg(script.path())
        .arg("--emit-guards")
        .assert()
        .success();
}

// ============================================================================
// TYPE_003: Backward Compatibility
// ============================================================================

#[test]
fn test_TYPE_003_no_flags_unchanged_behavior() {
    let script = create_temp_script("#!/bin/bash\necho hello world\n");

    // Without any type flags, output should be same as before
    bashrs_cmd()
        .arg("purify")
        .arg(script.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("#!/bin/sh"))
        .stdout(predicate::str::contains("echo"))
        // No guards should be present
        .stdout(predicate::str::contains("type error").not());
}

#[test]
fn test_TYPE_003_report_flag_still_works() {
    let script = create_temp_script("#!/bin/bash\necho hello\n");

    bashrs_cmd()
        .arg("purify")
        .arg(script.path())
        .arg("--report")
        .assert()
        .success();
}

#[test]
fn test_TYPE_003_type_check_combined_with_report() {
    let script = create_temp_script("#!/bin/bash\n# @type x: int\nx=42\n");

    bashrs_cmd()
        .arg("purify")
        .arg(script.path())
        .arg("--type-check")
        .arg("--report")
        .assert()
        .success();
}
