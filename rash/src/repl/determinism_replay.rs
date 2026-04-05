//! Replay verification for determinism checking
//!
//! Verifies determinism by running scripts multiple times and comparing outputs.

use super::determinism::{OutputDifference, ReplayResult, RunOutput};

/// Verifies determinism by running scripts multiple times and comparing outputs
#[derive(Debug, Clone)]
pub struct ReplayVerifier {
    /// Number of replay runs to perform (default: 2, min: 2)
    replay_count: usize,
}

impl ReplayVerifier {
    /// Create a new replay verifier with default settings
    pub fn new() -> Self {
        Self { replay_count: 2 }
    }

    /// Set number of replay runs (min: 2)
    pub fn with_replay_count(mut self, count: usize) -> Self {
        self.replay_count = count.max(2);
        self
    }

    /// Verify determinism by running script multiple times
    pub fn verify(&self, script: &str) -> ReplayResult {
        let mut runs = Vec::new();

        // Run script multiple times
        for run_number in 1..=self.replay_count {
            let mut output = Self::execute_script(script);
            output.run_number = run_number;
            runs.push(output);
        }

        // Compare outputs between first two runs
        let differences = if runs.len() >= 2 {
            if let (Some(run0), Some(run1)) = (runs.first(), runs.get(1)) {
                Self::find_differences(run0, run1)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let is_deterministic = differences.is_empty();

        ReplayResult {
            is_deterministic,
            runs,
            differences,
        }
    }

    /// Execute bash script and capture output
    pub(crate) fn execute_script(script: &str) -> RunOutput {
        use std::process::Command;

        match Command::new("bash").arg("-c").arg(script).output() {
            Ok(output) => RunOutput {
                run_number: 0, // Will be set by caller
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().unwrap_or(-1),
            },
            Err(e) => RunOutput {
                run_number: 0,
                stdout: String::new(),
                stderr: format!("Failed to execute bash script: {}", e),
                exit_code: -1,
            },
        }
    }

    /// Find differences between two runs
    pub(crate) fn find_differences(run1: &RunOutput, run2: &RunOutput) -> Vec<OutputDifference> {
        let mut differences = Vec::new();

        // Compare stdout line by line
        let lines1: Vec<&str> = run1.stdout.lines().collect();
        let lines2: Vec<&str> = run2.stdout.lines().collect();

        let max_lines = lines1.len().max(lines2.len());
        for i in 0..max_lines {
            let line1 = lines1.get(i).unwrap_or(&"");
            let line2 = lines2.get(i).unwrap_or(&"");

            if line1 != line2 {
                differences.push(OutputDifference {
                    line: i + 1, // 1-indexed
                    run1: line1.to_string(),
                    run2: line2.to_string(),
                });
            }
        }

        differences
    }
}

impl Default for ReplayVerifier {
    fn default() -> Self {
        Self::new()
    }
}

// ===== REPL-011-003: DIFF EXPLANATION =====

/// Format replay verification differences for display
///
/// Takes a vector of OutputDifference and formats them in a human-friendly way.
/// Shows line numbers and the actual output from each run.
///
/// # Examples
///
/// ```
/// use bashrs::repl::determinism::OutputDifference;
/// use bashrs::repl::determinism_replay::format_replay_diff;
///
/// let differences = vec![
///     OutputDifference {
///         line: 1,
///         run1: "Random: 12345".to_string(),
///         run2: "Random: 67890".to_string(),
///     },
/// ];
///
/// let formatted = format_replay_diff(&differences);
/// assert!(formatted.contains("Line 1:"));
/// ```
pub fn format_replay_diff(differences: &[OutputDifference]) -> String {
    if differences.is_empty() {
        return String::from("✓ No differences detected - script is deterministic");
    }

    let mut output = String::new();
    output.push_str("❌ Non-deterministic output detected!\n\n");
    output.push_str(&format!("Found {} difference(s):\n\n", differences.len()));

    for diff in differences {
        output.push_str(&format!("Line {}:\n", diff.line));
        output.push_str(&format!("  Run 1: {}\n", diff.run1));
        output.push_str(&format!("  Run 2: {}\n", diff.run2));
        output.push('\n');
    }

    output
}

impl ReplayResult {
    /// Format replay result for display
    ///
    /// Shows determinism status, run count, exit codes, and any differences.
    pub fn format_result(&self) -> String {
        let mut output = String::new();

        // Show determinism status
        if self.is_deterministic {
            output.push_str("✓ Script is deterministic\n");
        } else {
            output.push_str("❌ Script is non-deterministic\n");
        }

        // Show run count
        output.push_str(&format!("\nRuns: {}\n", self.runs.len()));

        // Show exit codes
        output.push_str("Exit codes: ");
        for run in &self.runs {
            output.push_str(&format!("{} ", run.exit_code));
        }
        output.push('\n');

        // Show differences if any
        if !self.differences.is_empty() {
            output.push('\n');
            output.push_str(&format_replay_diff(&self.differences));
        }

        output
    }
}

// ===== TESTS =====

#[cfg(test)]
mod replay_tests {
    use super::*;

    // ===== REPL-011-002: REPLAY VERIFICATION TESTS =====

    /// Test: REPL-011-002-001 - Deterministic script verification
    #[test]
    fn test_REPL_011_002_deterministic_script() {
        // ARRANGE: Simple deterministic script
        let script = r#"
echo "line1"
echo "line2"
echo "line3"
        "#;
        let verifier = ReplayVerifier::new();

        // ACT: Verify determinism
        let result = verifier.verify(script);

        // ASSERT: Should be deterministic
        assert!(
            result.is_deterministic,
            "Simple script should be deterministic"
        );
        assert_eq!(result.runs.len(), 2);
        assert_eq!(result.differences.len(), 0);

        // Both runs should have identical output
        assert_eq!(result.runs[0].stdout, result.runs[1].stdout);
    }

    /// Test: REPL-011-002-002 - Non-deterministic script detection
    #[test]
    fn test_REPL_011_002_nondeterministic_script() {
        // ARRANGE: Script with $RANDOM (non-deterministic)
        let script = r#"
echo "Random: $RANDOM"
        "#;
        let verifier = ReplayVerifier::new();

        // ACT: Verify determinism
        let result = verifier.verify(script);

        // ASSERT: Should be non-deterministic
        assert!(
            !result.is_deterministic,
            "Script with $RANDOM should be non-deterministic"
        );
        assert_eq!(result.runs.len(), 2);
        assert!(!result.differences.is_empty(), "Should have differences");

        // Runs should have different output
        assert_ne!(result.runs[0].stdout, result.runs[1].stdout);
    }

    /// Test: REPL-011-002-003 - Multiple replay runs
    #[test]
    fn test_REPL_011_002_multiple_replays() {
        // ARRANGE: Deterministic script with 5 replays
        let script = "echo 'hello world'";
        let verifier = ReplayVerifier::new().with_replay_count(5);

        // ACT: Verify with multiple runs
        let result = verifier.verify(script);

        // ASSERT: All 5 runs should be identical
        assert!(result.is_deterministic);
        assert_eq!(result.runs.len(), 5);

        // All runs should have same output
        let first_output = &result.runs[0].stdout;
        for run in &result.runs[1..] {
            assert_eq!(&run.stdout, first_output);
        }
    }

    /// Test: REPL-011-002-004 - Difference detection
    #[test]
    fn test_REPL_011_002_difference_detection() {
        // ARRANGE: Script that outputs different values
        let script = "echo $RANDOM";
        let verifier = ReplayVerifier::new();

        // ACT: Verify determinism
        let result = verifier.verify(script);

        // ASSERT: Should detect differences
        assert!(!result.is_deterministic);
        assert_eq!(result.differences.len(), 1);
        assert_eq!(result.differences[0].line, 1);
        assert_ne!(result.differences[0].run1, result.differences[0].run2);
    }

    /// Test: REPL-011-002-005 - Empty script handling
    #[test]
    fn test_REPL_011_002_empty_script() {
        // ARRANGE: Empty script
        let script = "";
        let verifier = ReplayVerifier::new();

        // ACT: Verify determinism
        let result = verifier.verify(script);

        // ASSERT: Empty script is deterministic
        assert!(result.is_deterministic);
        assert_eq!(result.runs[0].stdout, "");
        assert_eq!(result.runs[1].stdout, "");
    }

    /// Test: REPL-011-002-006 - Multiline output
    #[test]
    fn test_REPL_011_002_multiline_output() {
        // ARRANGE: Script with multiple lines of output
        let script = r#"
for i in 1 2 3; do
    echo "Line $i"
done
        "#;
        let verifier = ReplayVerifier::new();

        // ACT: Verify determinism
        let result = verifier.verify(script);

        // ASSERT: Should be deterministic
        assert!(result.is_deterministic);
        assert!(result.runs[0].stdout.contains("Line 1"));
        assert!(result.runs[0].stdout.contains("Line 2"));
        assert!(result.runs[0].stdout.contains("Line 3"));
    }

    /// Test: REPL-011-002-007 - Exit code tracking
    #[test]
    fn test_REPL_011_002_exit_code_tracking() {
        // ARRANGE: Script that exits with error
        let script = "echo 'error'; exit 42";
        let verifier = ReplayVerifier::new();

        // ACT: Verify determinism
        let result = verifier.verify(script);

        // ASSERT: Exit codes should match
        assert!(result.is_deterministic);
        assert_eq!(result.runs[0].exit_code, 42);
        assert_eq!(result.runs[1].exit_code, 42);
    }

    /// Test: REPL-011-002-008 - Minimum replay count
    #[test]
    fn test_REPL_011_002_min_replay_count() {
        // ARRANGE: Verifier with replay_count < 2 (should be clamped to 2)
        let script = "echo 'test'";
        let verifier = ReplayVerifier::new().with_replay_count(1);

        // ACT: Verify determinism
        let result = verifier.verify(script);

        // ASSERT: Should still run at least 2 times
        assert_eq!(result.runs.len(), 2);
    }
}

// ===== PROPERTY TESTS =====

#[cfg(test)]
mod replay_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Simple echo statements should always be deterministic
        #[test]
        fn prop_REPL_011_002_deterministic_scripts_always_identical(
            line in "[a-z ]{1,30}"
        ) {
            // Simple echo statements should always be deterministic
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
            // Deterministic scripts should be consistent across N runs
            let script = "echo 'consistent'";
            let verifier = ReplayVerifier::new().with_replay_count(replay_count);
            let result = verifier.verify(script);

            prop_assert!(result.is_deterministic);
            prop_assert_eq!(result.runs.len(), replay_count);

            // All runs should have identical output
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
            // Verifier should never panic on any input
            let verifier = ReplayVerifier::new();
            let _ = verifier.verify(&script);
        }
    }
}


include!("determinism_replay_tests_REPL_2.rs");
