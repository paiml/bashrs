#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! Provable Contract Tests: linter-coverage-v1.yaml (GAP-4)
//!
//! Verifies that each linter rule category fires on at least one
//! known-bad input per format. A rule that never fires is dead code.
//!
//! Reference: SSC v13 Section 3 GAP-4

// ============================================================================
// Helpers
// ============================================================================

fn shell_has(source: &str, code: &str) -> bool {
    bashrs::linter::lint_shell(source)
        .diagnostics
        .iter()
        .any(|d| d.code == code)
}

fn makefile_has(source: &str, code: &str) -> bool {
    bashrs::linter::lint_makefile(source)
        .diagnostics
        .iter()
        .any(|d| d.code == code)
}

fn dockerfile_has(source: &str, code: &str) -> bool {
    bashrs::linter::lint_dockerfile_with_profile(source, bashrs::linter::LintProfile::Standard)
        .diagnostics
        .iter()
        .any(|d| d.code == code)
}

// ============================================================================
// F-LCOV-SEC-SHELL: All core SEC rules fire on shell inputs
// ============================================================================

#[test]
fn falsify_LCOV_SEC001_fires() {
    assert!(
        shell_has(r#"eval "$var""#, "SEC001"),
        "SEC001 must fire on eval with variable"
    );
}

#[test]
fn falsify_LCOV_SEC002_fires() {
    assert!(
        shell_has("curl $URL", "SEC002"),
        "SEC002 must fire on unquoted var in network cmd"
    );
}

#[test]
fn falsify_LCOV_SEC003_fires() {
    assert!(
        shell_has("find /tmp -exec sh -c 'echo {}' \\;", "SEC003"),
        "SEC003 must fire on find -exec sh -c with {{}}"
    );
}

#[test]
fn falsify_LCOV_SEC004_fires() {
    assert!(
        shell_has("curl --insecure https://example.com", "SEC004"),
        "SEC004 must fire on --insecure"
    );
}

#[test]
fn falsify_LCOV_SEC005_fires() {
    assert!(
        shell_has(r#"API_KEY="sk-1234567890abcdef1234567890abcdef""#, "SEC005"),
        "SEC005 must fire on hardcoded key"
    );
}

#[test]
fn falsify_LCOV_SEC006_fires() {
    assert!(
        shell_has(r#"TMPFILE="/tmp/myapp.$$""#, "SEC006"),
        "SEC006 must fire on unsafe temp file in /tmp"
    );
}

#[test]
fn falsify_LCOV_SEC007_fires() {
    assert!(
        shell_has("sudo rm -rf $DIR", "SEC007"),
        "SEC007 must fire on sudo with unquoted var"
    );
}

#[test]
fn falsify_LCOV_SEC008_fires() {
    assert!(
        shell_has("curl https://example.com | sh", "SEC008"),
        "SEC008 must fire on curl piped to shell"
    );
}

// ============================================================================
// F-LCOV-DET-SHELL: All DET rules fire on shell inputs
// ============================================================================

#[test]
fn falsify_LCOV_DET001_fires() {
    assert!(
        shell_has("x=$RANDOM", "DET001"),
        "DET001 must fire on $RANDOM"
    );
}

#[test]
fn falsify_LCOV_DET002_fires() {
    assert!(
        shell_has("ts=$(date +%s)", "DET002"),
        "DET002 must fire on date command substitution"
    );
}

#[test]
fn falsify_LCOV_DET003_fires() {
    assert!(
        shell_has("for f in *.txt; do echo \"$f\"; done", "DET003"),
        "DET003 must fire on unordered glob"
    );
}

#[test]
fn falsify_LCOV_DET004_fires() {
    assert!(
        shell_has("mem=$(free -m)", "DET004"),
        "DET004 must fire on system state command"
    );
}

// ============================================================================
// F-LCOV-IDEM-SHELL: All IDEM rules fire on shell inputs
// ============================================================================

#[test]
fn falsify_LCOV_IDEM001_fires() {
    assert!(
        shell_has("mkdir /tmp/testdir", "IDEM001"),
        "IDEM001 must fire on mkdir without -p"
    );
}

#[test]
fn falsify_LCOV_IDEM002_fires() {
    assert!(
        shell_has("rm /tmp/oldfile", "IDEM002"),
        "IDEM002 must fire on rm without -f"
    );
}

#[test]
fn falsify_LCOV_IDEM003_fires() {
    assert!(
        shell_has("ln -s /src /dst", "IDEM003"),
        "IDEM003 must fire on ln -s without -f"
    );
}

// ============================================================================
// F-LCOV-DOCKER: All core DOCKER rules fire on Dockerfile inputs
// ============================================================================

#[test]
fn falsify_LCOV_DOCKER001_fires() {
    assert!(
        dockerfile_has("FROM ubuntu:latest\nRUN apt-get update", "DOCKER001"),
        "DOCKER001 must fire on Dockerfile without USER"
    );
}

#[test]
fn falsify_LCOV_DOCKER002_fires() {
    assert!(
        dockerfile_has("FROM ubuntu:latest\nRUN echo hi", "DOCKER002"),
        "DOCKER002 must fire on :latest tag"
    );
}

#[test]
fn falsify_LCOV_DOCKER003_fires() {
    assert!(
        dockerfile_has(
            "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl",
            "DOCKER003"
        ),
        "DOCKER003 must fire on apt without cleanup"
    );
}

#[test]
fn falsify_LCOV_DOCKER004_fires() {
    assert!(
        dockerfile_has(
            "FROM ubuntu:22.04 AS builder\nFROM alpine:3.18\nCOPY --from=nonexistent /app /app",
            "DOCKER004"
        ),
        "DOCKER004 must fire on invalid COPY --from"
    );
}

#[test]
fn falsify_LCOV_DOCKER005_fires() {
    assert!(
        dockerfile_has(
            "FROM ubuntu:22.04\nRUN apt-get install -y curl",
            "DOCKER005"
        ),
        "DOCKER005 must fire on apt install without --no-install-recommends"
    );
}

#[test]
fn falsify_LCOV_DOCKER006_fires() {
    assert!(
        dockerfile_has("FROM ubuntu:22.04\nADD . /app", "DOCKER006"),
        "DOCKER006 must fire on ADD for local files"
    );
}

// ============================================================================
// F-LCOV-MAKE: Core MAKE rules fire on Makefile inputs
// ============================================================================

#[test]
fn falsify_LCOV_MAKE001_fires() {
    assert!(
        makefile_has("SRCS = $(wildcard *.c)", "MAKE001"),
        "MAKE001 must fire on unsorted wildcard"
    );
}

#[test]
fn falsify_LCOV_MAKE002_fires() {
    assert!(
        makefile_has("all:\n\tmkdir /tmp/foo", "MAKE002"),
        "MAKE002 must fire on mkdir without -p in recipe"
    );
}

#[test]
fn falsify_LCOV_MAKE003_fires() {
    assert!(
        makefile_has("clean:\n\trm -rf $BUILD_DIR", "MAKE003"),
        "MAKE003 must fire on unquoted var in recipe with dangerous cmd"
    );
}

// ============================================================================
// F-LCOV-NO-DEAD: Aggregate coverage check
// ============================================================================

#[test]
fn falsify_LCOV_aggregate_shell_rules_all_fire() {
    // Ensure we're testing ALL core shell safety rules
    let rules = [
        ("SEC001", r#"eval "$var""#),
        ("SEC002", "curl $URL"),
        ("SEC003", "find /tmp -exec sh -c 'echo {}' \\;"),
        ("SEC004", "curl --insecure https://example.com"),
        ("SEC005", r#"API_KEY="sk-1234567890abcdef1234567890abcdef""#),
        ("SEC006", r#"TMPFILE="/tmp/myapp.$$""#),
        ("SEC007", "sudo rm -rf $DIR"),
        ("SEC008", "curl https://example.com | sh"),
        ("DET001", "x=$RANDOM"),
        ("DET002", "ts=$(date +%s)"),
        ("DET003", "for f in *.txt; do echo \"$f\"; done"),
        ("DET004", "mem=$(free -m)"),
        ("IDEM001", "mkdir /tmp/testdir"),
        ("IDEM002", "rm /tmp/oldfile"),
        ("IDEM003", "ln -s /src /dst"),
    ];

    let mut missing = Vec::new();
    for (code, input) in &rules {
        if !shell_has(input, code) {
            missing.push(*code);
        }
    }
    assert!(
        missing.is_empty(),
        "F-LCOV-NO-DEAD: {} shell rule(s) failed to fire: {:?}",
        missing.len(),
        missing
    );
}
