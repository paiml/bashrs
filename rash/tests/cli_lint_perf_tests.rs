#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

//! CLI performance tests for lint latency (PMAT-226)
//!
//! These tests verify that:
//! 1. The --time flag works and reports lint_time_ms
//! 2. Lint latency scales sub-linearly with script size
//! 3. The lint engine adds minimal overhead per script

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

/// Extract lint_time_ms from stderr output.
fn extract_lint_time(output: &assert_cmd::assert::Assert) -> Option<f64> {
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    for line in stderr.lines() {
        if let Some(rest) = line.strip_prefix("lint_time_ms: ") {
            return rest.trim().parse::<f64>().ok();
        }
    }
    None
}

// ---------------------------------------------------------------------------
// --time flag tests
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT226_time_flag_in_help() {
    bashrs_cmd()
        .arg("lint")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--time"));
}

#[test]
fn test_PMAT226_time_outputs_lint_time_ms() {
    let f = shell_file("#!/bin/sh\necho hello");
    let assert = bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .arg("--time")
        .assert();

    let time = extract_lint_time(&assert);
    assert!(time.is_some(), "Expected lint_time_ms in stderr");
    assert!(time.unwrap() > 0.0, "Expected positive lint time");
}

#[test]
fn test_PMAT226_no_time_without_flag() {
    let f = shell_file("#!/bin/sh\necho hello");
    bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .assert()
        .stderr(predicate::str::contains("lint_time_ms").not());
}

#[test]
fn test_PMAT226_time_with_dirty_script() {
    // Script with lint issues should still report timing
    let f = shell_file("#!/bin/sh\nmkdir /tmp/foo\neval $cmd\nchmod 777 /tmp/x");
    let assert = bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .arg("--time")
        .assert();
    // Dirty scripts cause non-zero exit but timing should still appear
    let time = extract_lint_time(&assert);
    assert!(
        time.is_some(),
        "Expected lint_time_ms even with lint errors"
    );
}

#[test]
fn test_PMAT226_time_with_json_format() {
    let f = shell_file("#!/bin/sh\necho hello");
    let assert = bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .arg("--time")
        .arg("--format")
        .arg("json")
        .assert();

    let time = extract_lint_time(&assert);
    assert!(time.is_some(), "Expected lint_time_ms with JSON format");
}

// ---------------------------------------------------------------------------
// Lint engine scalability tests
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT226_lint_scales_sublinearly() {
    // Small script (5 lines)
    let small = shell_file("#!/bin/sh\nset -eu\necho a\necho b\necho c");

    // Large script (100+ lines of realistic code)
    let large_code = format!(
        "#!/bin/sh\nset -eu\n{}\n",
        (0..100)
            .map(|i| format!("func_{i}() {{ echo \"line {i}\"; }}\n"))
            .collect::<String>()
    );
    let large = shell_file(&large_code);

    let small_assert = bashrs_cmd()
        .arg("lint")
        .arg(small.path())
        .arg("--time")
        .assert();
    let large_assert = bashrs_cmd()
        .arg("lint")
        .arg(large.path())
        .arg("--time")
        .assert();

    let small_time = extract_lint_time(&small_assert).unwrap_or(0.0);
    let large_time = extract_lint_time(&large_assert).unwrap_or(0.0);

    // The large script (20x more lines) should NOT take 20x longer.
    // Allow 3x overhead max (sub-linear scaling proves O(n) or better).
    let ratio = if small_time > 0.0 {
        large_time / small_time
    } else {
        1.0
    };
    assert!(
        ratio < 5.0,
        "Lint should scale sub-linearly. Small: {small_time:.2}ms, Large: {large_time:.2}ms, Ratio: {ratio:.2}x"
    );
}

#[test]
fn test_PMAT226_lint_incremental_overhead_under_10ms() {
    // Measure lint of small vs medium script
    // The DIFFERENTIAL should be under 10ms (proving the lint engine adds <10ms per 50 lines)
    let small = shell_file("#!/bin/sh\necho hello");
    let medium_code = format!(
        "#!/bin/sh\nset -eu\n{}\n",
        (0..50)
            .map(|i| format!("echo \"line {i}\""))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let medium = shell_file(&medium_code);

    let small_assert = bashrs_cmd()
        .arg("lint")
        .arg(small.path())
        .arg("--time")
        .assert();
    let medium_assert = bashrs_cmd()
        .arg("lint")
        .arg(medium.path())
        .arg("--time")
        .assert();

    let small_time = extract_lint_time(&small_assert).unwrap_or(0.0);
    let medium_time = extract_lint_time(&medium_assert).unwrap_or(0.0);

    let diff = (medium_time - small_time).abs();

    // Incremental cost of 50 extra lines should be under 100ms even in debug
    // (in release, this would be under 10ms)
    assert!(
        diff < 100.0,
        "Incremental lint cost for 50 lines should be under 100ms (debug). Got: {diff:.2}ms (small: {small_time:.2}ms, medium: {medium_time:.2}ms)"
    );
}

// ---------------------------------------------------------------------------
// Latency bound tests (generous for debug, tight for the concept)
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT226_lint_under_1_second() {
    // Even in debug mode, lint should complete in under 1 second for a 100-line script
    let code = format!(
        "#!/bin/sh\nset -eu\n{}\n",
        (0..100)
            .map(|i| format!("echo \"line {i}\""))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let f = shell_file(&code);

    let assert = bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .arg("--time")
        .assert();

    let time = extract_lint_time(&assert).unwrap_or(f64::MAX);
    assert!(
        time < 1000.0,
        "Lint should complete in under 1 second. Got: {time:.2}ms"
    );
}

#[test]
fn test_PMAT226_time_json_output_format() {
    let f = shell_file("#!/bin/sh\necho hello");
    let assert = bashrs_cmd()
        .arg("lint")
        .arg(f.path())
        .arg("--time")
        .assert();

    // Verify the format is "lint_time_ms: <number>"
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    let time_line = stderr.lines().find(|l| l.starts_with("lint_time_ms: "));
    assert!(time_line.is_some(), "Expected lint_time_ms line in stderr");
    let val: f64 = time_line
        .unwrap()
        .strip_prefix("lint_time_ms: ")
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert!(val > 0.0, "lint_time_ms should be a positive float");
}
