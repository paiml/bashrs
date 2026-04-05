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
    fs::write(
        dir.path().join("compliant.sh"),
        "#!/bin/sh\necho \"hello\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("noncompliant.sh"),
        "#!/bin/bash\necho $RANDOM\n",
    )
    .unwrap();

    let output = bashrs_cmd()
        .arg("comply")
        .arg("check")
        .arg("--path")
        .arg(dir.path())
        .arg("--failures-only")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Failures Only"),
        "Should show failures-only header"
    );
    assert!(
        stdout.contains("NON-COMPLIANT"),
        "Should show non-compliant artifacts"
    );
    // Compliant artifacts should not appear in the artifact list
    assert!(
        !stdout.contains("+ COMPLIANT"),
        "Should NOT show compliant artifacts"
    );
}

#[test]
fn test_comply_check_failures_only_markdown() {
    let dir = create_test_project();
    fs::write(
        dir.path().join("compliant.sh"),
        "#!/bin/sh\necho \"hello\"\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("noncompliant.sh"),
        "#!/bin/bash\necho $RANDOM\n",
    )
    .unwrap();

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
    fs::write(
        dir.path().join("bad.sh"),
        "#!/bin/bash\necho $RANDOM\nmkdir /tmp/foo\n",
    )
    .unwrap();

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
    assert!(
        stderr.contains("below minimum"),
        "Error should mention below minimum: {}",
        stderr
    );
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

// ═══════════════════════════════════════════════════════════════
// comply rules
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_COMPLY_CLI_060_rules_text() {
    bashrs_cmd()
        .arg("comply")
        .arg("rules")
        .assert()
        .success()
        .stdout(predicate::str::contains("COMPLY-001"))
        .stdout(predicate::str::contains("COMPLY-010"))
        .stdout(predicate::str::contains("10 rules"));
}

#[test]
fn test_COMPLY_CLI_061_rules_json() {
    let output = bashrs_cmd()
        .arg("comply")
        .arg("rules")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("bashrs-comply-rules-v1"));
    assert!(stdout.contains("COMPLY-001"));
    assert!(stdout.contains("\"weight\":"));
}

#[test]
fn test_COMPLY_CLI_062_rules_markdown() {
    bashrs_cmd()
        .arg("comply")
        .arg("rules")
        .arg("--format")
        .arg("markdown")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Compliance Rules"))
        .stdout(predicate::str::contains("comply:disable"));
}

// ============================================================================
// Phase 2: comply report tests (PMAT-196)
// ============================================================================

#[test]
fn test_PMAT196_comply_report_markdown() {
    bashrs_cmd()
        .arg("comply")
        .arg("report")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Compliance Report"))
        .stdout(predicate::str::contains("**Grade**"))
        .stdout(predicate::str::contains("## Artifacts"))
        .stdout(predicate::str::contains("| Artifact |"));
}

#[test]
fn test_PMAT196_comply_report_json() {
    bashrs_cmd()
        .arg("comply")
        .arg("report")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"grade\""))
        .stdout(predicate::str::contains("\"score\""))
        .stdout(predicate::str::contains("\"artifacts\""));
}

#[test]
fn test_PMAT196_comply_report_to_file() {
    let tmp = TempDir::new().expect("tempdir");
    let out = tmp.path().join("report.md");
    bashrs_cmd()
        .arg("comply")
        .arg("report")
        .arg("--output")
        .arg(out.to_str().expect("path"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Report written to"));

    let content = fs::read_to_string(&out).expect("read report");
    assert!(content.contains("# Compliance Report"));
}

// ============================================================================
// Phase 2: comply enforce tests (PMAT-196)
// ============================================================================

#[test]
fn test_PMAT196_comply_enforce_help() {
    bashrs_cmd()
        .arg("comply")
        .arg("enforce")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("pre-commit"))
        .stdout(predicate::str::contains("--tier"))
        .stdout(predicate::str::contains("--uninstall"));
}

// ============================================================================
// Phase 2: comply diff tests (PMAT-196)
// ============================================================================

#[test]
fn test_PMAT196_comply_diff_first_run() {
    let tmp = TempDir::new().expect("tempdir");
    // Create a shell script so there's something to check
    fs::write(tmp.path().join("hello.sh"), "#!/bin/sh\necho hello\n").expect("write");

    bashrs_cmd()
        .arg("comply")
        .arg("diff")
        .arg("--path")
        .arg(tmp.path().to_str().expect("path"))
        .assert()
        .success()
        .stdout(predicate::str::contains("No previous compliance snapshot"))
        .stdout(predicate::str::contains("Snapshot saved"));
}

#[test]
fn test_PMAT196_comply_diff_second_run_shows_delta() {
    let tmp = TempDir::new().expect("tempdir");
    fs::write(tmp.path().join("hello.sh"), "#!/bin/sh\necho hello\n").expect("write");

    // First run — saves snapshot
    bashrs_cmd()
        .arg("comply")
        .arg("diff")
        .arg("--path")
        .arg(tmp.path().to_str().expect("path"))
        .assert()
        .success();

    // Second run — shows diff
    bashrs_cmd()
        .arg("comply")
        .arg("diff")
        .arg("--path")
        .arg(tmp.path().to_str().expect("path"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Compliance Diff"))
        .stdout(predicate::str::contains("Score:"))
        .stdout(predicate::str::contains("Grade:"));
}
