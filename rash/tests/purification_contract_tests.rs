#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! Provable Contract Tests: purification-pipeline-v1.yaml
//!
//! Each test attempts to FALSIFY a purification invariant.
//! The bash purifier transforms non-deterministic bash into safe,
//! idempotent POSIX sh. A single counterexample breaks the contract.
//!
//! Reference: GH-183 (KZ-11: Missing provable contracts)

use bashrs::repl::purifier::{purify_and_lint, purify_bash};

// ============================================================================
// F-PURIFY-001 / F-PURIFY-002: Determinism
// ============================================================================

/// F-PURIFY-001: $RANDOM must be replaced with deterministic value
#[test]
fn falsify_PURIFY_001_random_removed() {
    let output = purify_bash("x=$RANDOM").unwrap();
    assert!(
        !output.contains("$RANDOM"),
        "F-PURIFY-001: purified output must NOT contain $RANDOM, got:\n{}",
        output
    );
}

/// F-PURIFY-001 variant: multiple $RANDOM occurrences
#[test]
fn falsify_PURIFY_001_random_multiple() {
    let output = purify_bash("a=$RANDOM\nb=$RANDOM").unwrap();
    assert!(
        !output.contains("$RANDOM"),
        "F-PURIFY-001: all $RANDOM occurrences must be purified"
    );
}

/// F-PURIFY-002: Purification determinism — same input, same output
#[test]
fn falsify_PURIFY_002_determinism() {
    let input = "mkdir /tmp/foo\nrm /tmp/bar\necho $x";
    let r1 = purify_bash(input).unwrap();
    let r2 = purify_bash(input).unwrap();
    assert_eq!(
        r1, r2,
        "F-PURIFY-002: purify(input) must equal purify(input)"
    );
}

/// F-PURIFY-002 variant: complex input
#[test]
fn falsify_PURIFY_002_determinism_complex() {
    let input = "#!/bin/bash\nx=$RANDOM\nmkdir /tmp/a\nrm /tmp/b\necho $var";
    let r1 = purify_bash(input).unwrap();
    let r2 = purify_bash(input).unwrap();
    assert_eq!(r1, r2, "F-PURIFY-002: complex purification must be deterministic");
}

// ============================================================================
// F-PURIFY-003 / F-PURIFY-004: Idempotency enforcement
// ============================================================================

/// F-PURIFY-003: mkdir → mkdir -p
#[test]
fn falsify_PURIFY_003_mkdir_p() {
    let output = purify_bash("mkdir /tmp/testdir").unwrap();
    assert!(
        output.contains("mkdir -p"),
        "F-PURIFY-003: mkdir must become 'mkdir -p', got:\n{}",
        output
    );
}

/// F-PURIFY-003 variant: mkdir with existing flags preserved
#[test]
fn falsify_PURIFY_003_mkdir_p_preserves_path() {
    let output = purify_bash("mkdir /opt/myapp/data").unwrap();
    assert!(
        output.contains("mkdir -p") && output.contains("/opt/myapp/data"),
        "F-PURIFY-003: mkdir -p must preserve the original path"
    );
}

/// F-PURIFY-004: rm → rm -f
#[test]
fn falsify_PURIFY_004_rm_f() {
    let output = purify_bash("rm /tmp/old").unwrap();
    assert!(
        output.contains("rm -f"),
        "F-PURIFY-004: rm must become 'rm -f', got:\n{}",
        output
    );
}

// ============================================================================
// F-PURIFY-005: POSIX compliance
// ============================================================================

/// F-PURIFY-005: CLI purify replaces #!/bin/bash → #!/bin/sh
/// NOTE: The library API (purify_bash) preserves shebangs; the CLI layer
/// performs the replacement. This test validates via CLI.
#[test]
fn falsify_PURIFY_005_posix_shebang_cli() {
    use std::io::Write;
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(b"#!/bin/bash\necho hello\n").unwrap();
    f.flush().unwrap();

    let output = assert_cmd::Command::cargo_bin("bashrs")
        .unwrap()
        .arg("purify")
        .arg(f.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("#!/bin/sh"),
        "F-PURIFY-005: CLI purify must emit #!/bin/sh, got:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("#!/bin/bash"),
        "F-PURIFY-005: CLI purify must replace #!/bin/bash"
    );
}

// ============================================================================
// F-PURIFY-006: Variable safety
// ============================================================================

/// F-PURIFY-006: unquoted $var → "$var"
#[test]
fn falsify_PURIFY_006_variable_quoting() {
    let output = purify_bash("echo $unquoted_var").unwrap();
    assert!(
        output.contains("\"$unquoted_var\""),
        "F-PURIFY-006: unquoted variable must be double-quoted, got:\n{}",
        output
    );
}

// ============================================================================
// F-PURIFY-007 / F-PURIFY-008: Purify-then-lint cleanliness
// ============================================================================

/// F-PURIFY-007: purified non-deterministic input has zero DET violations
#[test]
fn falsify_PURIFY_007_no_det_violations_after_purify() {
    let result = purify_and_lint("#!/bin/bash\nx=$RANDOM").unwrap();
    let det_violations: Vec<_> = result
        .lint_result
        .diagnostics
        .iter()
        .filter(|d| d.code.starts_with("DET"))
        .collect();
    assert!(
        det_violations.is_empty(),
        "F-PURIFY-007: purified output must have zero DET violations, found: {:?}",
        det_violations.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

/// F-PURIFY-008: purified non-idempotent input has zero IDEM violations
#[test]
fn falsify_PURIFY_008_no_idem_violations_after_purify() {
    let result = purify_and_lint("mkdir /tmp/dir").unwrap();
    let idem_violations: Vec<_> = result
        .lint_result
        .diagnostics
        .iter()
        .filter(|d| d.code.starts_with("IDEM"))
        .collect();
    assert!(
        idem_violations.is_empty(),
        "F-PURIFY-008: purified output must have zero IDEM violations, found: {:?}",
        idem_violations
            .iter()
            .map(|d| &d.code)
            .collect::<Vec<_>>()
    );
}

// ============================================================================
// F-PURIFY-009 / F-PURIFY-010: Behavioral equivalence
// ============================================================================

/// F-PURIFY-009: purify(purify(x)) == purify(x) — idempotent purification
#[test]
fn falsify_PURIFY_009_purification_idempotent() {
    let input = "#!/bin/sh\necho \"hello\"";
    let once = purify_bash(input).unwrap();
    let twice = purify_bash(&once).unwrap();
    assert_eq!(
        once, twice,
        "F-PURIFY-009: purify(purify(input)) must equal purify(input)"
    );
}

/// F-PURIFY-009 variant: complex input idempotent
#[test]
fn falsify_PURIFY_009_purification_idempotent_complex() {
    let input = "#!/bin/bash\nx=$RANDOM\nmkdir /tmp/a\necho $var";
    let once = purify_bash(input).unwrap();
    let twice = purify_bash(&once).unwrap();
    assert_eq!(
        once, twice,
        "F-PURIFY-009: complex purification must be idempotent"
    );
}

/// F-PURIFY-010: echo with literal args preserved
#[test]
fn falsify_PURIFY_010_echo_literal_preserved() {
    let output = purify_bash("echo hello world").unwrap();
    assert!(
        output.contains("echo") && output.contains("hello") && output.contains("world"),
        "F-PURIFY-010: echo literal args must be preserved, got:\n{}",
        output
    );
}
