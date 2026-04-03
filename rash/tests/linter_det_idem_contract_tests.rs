#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! Provable Contract Tests: linter-det-idem-v1.yaml
//!
//! Each test pair attempts to FALSIFY a DET/IDEM linter rule contract.
//! SOUND tests: known-bad input MUST be flagged.
//! PRECISE tests: known-safe input must NOT be flagged.
//!
//! Reference: GH-183 (KZ-11: Missing provable contracts)

fn has_diagnostic(source: &str, code: &str) -> bool {
    let result = bashrs::linter::lint_shell(source);
    result.diagnostics.iter().any(|d| d.code == code)
}

// ============================================================================
// DET001: $RANDOM
// ============================================================================

#[test]
fn falsify_DET001_SOUND_random_usage() {
    assert!(
        has_diagnostic("x=$RANDOM", "DET001"),
        "F-DET001-SOUND: $RANDOM MUST be flagged"
    );
}

#[test]
fn falsify_DET001_SOUND_random_in_echo() {
    assert!(
        has_diagnostic("echo $RANDOM", "DET001"),
        "F-DET001-SOUND: $RANDOM in echo MUST be flagged"
    );
}

#[test]
fn falsify_DET001_PRECISE_literal_integer() {
    assert!(
        !has_diagnostic("x=42", "DET001"),
        "F-DET001-PRECISE: literal integer must NOT be flagged"
    );
}

// ============================================================================
// DET002: timestamps
// ============================================================================

#[test]
fn falsify_DET002_SOUND_date_subst() {
    assert!(
        has_diagnostic("ts=$(date +%s)", "DET002"),
        "F-DET002-SOUND: date command substitution MUST be flagged"
    );
}

#[test]
fn falsify_DET002_SOUND_date_assignment() {
    assert!(
        has_diagnostic("now=$(date)", "DET002"),
        "F-DET002-SOUND: date in command substitution MUST be flagged"
    );
}

#[test]
fn falsify_DET002_PRECISE_static_string() {
    assert!(
        !has_diagnostic("ts=\"2024-01-01\"", "DET002"),
        "F-DET002-PRECISE: static date string must NOT be flagged"
    );
}

// ============================================================================
// DET003: unordered wildcards
// ============================================================================

#[test]
fn falsify_DET003_SOUND_unordered_glob() {
    assert!(
        has_diagnostic("for f in *.txt; do echo \"$f\"; done", "DET003"),
        "F-DET003-SOUND: unordered glob in for-loop MUST be flagged"
    );
}

#[test]
fn falsify_DET003_PRECISE_sorted_glob() {
    assert!(
        !has_diagnostic(
            "for f in $(ls *.txt | sort); do echo \"$f\"; done",
            "DET003"
        ),
        "F-DET003-PRECISE: sorted glob must NOT be flagged"
    );
}

// ============================================================================
// DET004: system state commands
// ============================================================================

#[test]
fn falsify_DET004_SOUND_free() {
    assert!(
        has_diagnostic("mem=$(free -m)", "DET004"),
        "F-DET004-SOUND: free command MUST be flagged as system-state"
    );
}

#[test]
fn falsify_DET004_SOUND_uptime() {
    assert!(
        has_diagnostic("u=$(uptime)", "DET004"),
        "F-DET004-SOUND: uptime command MUST be flagged as system-state"
    );
}

#[test]
fn falsify_DET004_PRECISE_echo() {
    assert!(
        !has_diagnostic("msg=$(echo hello)", "DET004"),
        "F-DET004-PRECISE: echo must NOT be flagged as system-state"
    );
}

// ============================================================================
// IDEM001: mkdir without -p
// ============================================================================

#[test]
fn falsify_IDEM001_SOUND_mkdir_bare() {
    assert!(
        has_diagnostic("mkdir /tmp/testdir", "IDEM001"),
        "F-IDEM001-SOUND: mkdir without -p MUST be flagged"
    );
}

#[test]
fn falsify_IDEM001_PRECISE_mkdir_p() {
    assert!(
        !has_diagnostic("mkdir -p /tmp/testdir", "IDEM001"),
        "F-IDEM001-PRECISE: mkdir -p must NOT be flagged"
    );
}

// ============================================================================
// IDEM002: rm without -f
// ============================================================================

#[test]
fn falsify_IDEM002_SOUND_rm_bare() {
    assert!(
        has_diagnostic("rm /tmp/old", "IDEM002"),
        "F-IDEM002-SOUND: rm without -f MUST be flagged"
    );
}

#[test]
fn falsify_IDEM002_PRECISE_rm_f() {
    assert!(
        !has_diagnostic("rm -f /tmp/old", "IDEM002"),
        "F-IDEM002-PRECISE: rm -f must NOT be flagged"
    );
}

// ============================================================================
// IDEM003: ln without -sf
// ============================================================================

#[test]
fn falsify_IDEM003_SOUND_ln_s_bare() {
    assert!(
        has_diagnostic("ln -s /tmp/a /tmp/b", "IDEM003"),
        "F-IDEM003-SOUND: ln -s without -f MUST be flagged"
    );
}

#[test]
fn falsify_IDEM003_PRECISE_ln_sf() {
    assert!(
        !has_diagnostic("ln -sf /tmp/a /tmp/b", "IDEM003"),
        "F-IDEM003-PRECISE: ln -sf must NOT be flagged"
    );
}
