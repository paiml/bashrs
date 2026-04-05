#![allow(non_snake_case, clippy::unwrap_used)]

use super::*;
use proptest::prelude::*;

// ===== REPL-011-002: REPLAY VERIFICATION TESTS =====

/// Test: REPL-011-002-001 - Deterministic script verification
#[test]
fn test_REPL_011_002_deterministic_script() {
    let script = r#"
echo "line1"
echo "line2"
echo "line3"
    "#;
    let verifier = ReplayVerifier::new();
    let result = verifier.verify(script);
    assert!(result.is_deterministic, "Simple script should be deterministic");
    assert_eq!(result.runs.len(), 2);
    assert_eq!(result.differences.len(), 0);
    assert_eq!(result.runs[0].stdout, result.runs[1].stdout);
}

/// Test: REPL-011-002-002 - Non-deterministic script detection
#[test]
fn test_REPL_011_002_nondeterministic_script() {
    let script = r#"
echo "Random: $RANDOM"
    "#;
    let verifier = ReplayVerifier::new();
    let result = verifier.verify(script);
    assert!(!result.is_deterministic, "Script with $RANDOM should be non-deterministic");
    assert_eq!(result.runs.len(), 2);
    assert!(!result.differences.is_empty(), "Should have differences");
    assert_ne!(result.runs[0].stdout, result.runs[1].stdout);
}

/// Test: REPL-011-002-003 - Multiple replay runs
#[test]
fn test_REPL_011_002_multiple_replays() {
    let script = "echo 'hello world'";
    let verifier = ReplayVerifier::new().with_replay_count(5);
    let result = verifier.verify(script);
    assert!(result.is_deterministic);
    assert_eq!(result.runs.len(), 5);
    let first_output = &result.runs[0].stdout;
    for run in &result.runs[1..] {
        assert_eq!(&run.stdout, first_output);
    }
}

/// Test: REPL-011-002-004 - Difference detection
#[test]
fn test_REPL_011_002_difference_detection() {
    let script = "echo $RANDOM";
    let verifier = ReplayVerifier::new();
    let result = verifier.verify(script);
    assert!(!result.is_deterministic);
    assert_eq!(result.differences.len(), 1);
    assert_eq!(result.differences[0].line, 1);
    assert_ne!(result.differences[0].run1, result.differences[0].run2);
}

/// Test: REPL-011-002-005 - Empty script handling
#[test]
fn test_REPL_011_002_empty_script() {
    let script = "";
    let verifier = ReplayVerifier::new();
    let result = verifier.verify(script);
    assert!(result.is_deterministic);
    assert_eq!(result.runs[0].stdout, "");
    assert_eq!(result.runs[1].stdout, "");
}

/// Test: REPL-011-002-006 - Multiline output
#[test]
fn test_REPL_011_002_multiline_output() {
    let script = r#"
for i in 1 2 3; do
    echo "Line $i"
done
    "#;
    let verifier = ReplayVerifier::new();
    let result = verifier.verify(script);
    assert!(result.is_deterministic);
    assert!(result.runs[0].stdout.contains("Line 1"));
    assert!(result.runs[0].stdout.contains("Line 2"));
    assert!(result.runs[0].stdout.contains("Line 3"));
}

/// Test: REPL-011-002-007 - Exit code tracking
#[test]
fn test_REPL_011_002_exit_code_tracking() {
    let script = "echo 'error'; exit 42";
    let verifier = ReplayVerifier::new();
    let result = verifier.verify(script);
    assert!(result.is_deterministic);
    assert_eq!(result.runs[0].exit_code, 42);
    assert_eq!(result.runs[1].exit_code, 42);
}

/// Test: REPL-011-002-008 - Minimum replay count
#[test]
fn test_REPL_011_002_min_replay_count() {
    let script = "echo 'test'";
    let verifier = ReplayVerifier::new().with_replay_count(1);
    let result = verifier.verify(script);
    assert_eq!(result.runs.len(), 2);
}

// ===== PROPERTY TESTS =====

proptest! {
    /// Property: Simple echo statements should always be deterministic
    #[test]
    fn prop_REPL_011_002_deterministic_scripts_always_identical(
        line in "[a-z ]{1,30}"
    ) {
        let script = format!("echo '{}'", line);
        let verifier = ReplayVerifier::new();
        let result = verifier.verify(&script);
        prop_assert!(
            result.is_deterministic,
            "Simple echo should be deterministic: '{}'", script
        );
        prop_assert_eq!(&result.runs[0].stdout, &result.runs[1].stdout);
    }

    /// Property: Deterministic scripts should be consistent across N runs
    #[test]
    fn prop_REPL_011_002_multiple_runs_consistent(
        replay_count in 2usize..10
    ) {
        let script = "echo 'consistent'";
        let verifier = ReplayVerifier::new().with_replay_count(replay_count);
        let result = verifier.verify(script);
        prop_assert!(result.is_deterministic);
        prop_assert_eq!(result.runs.len(), replay_count);
        let first_output = &result.runs[0].stdout;
        for run in &result.runs[1..] {
            prop_assert_eq!(&run.stdout, first_output);
        }
    }

    /// Property: Verifier should never panic on any input
    #[test]
    fn prop_REPL_011_002_verify_never_panics(
        script in ".*{0,100}"
    ) {
        let verifier = ReplayVerifier::new();
        let _ = verifier.verify(&script);
    }
}
