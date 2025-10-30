//! Determinism checking for bash scripts
//!
//! Detects non-deterministic patterns that violate deterministic execution:
//! - $RANDOM: Random number generation
//! - $$: Process ID
//! - Timestamps: date commands
//! - $BASHPID: Bash-specific process ID
//! - $SRANDOM: Cryptographic random (Bash 5.1+)

/// Detects non-deterministic patterns in bash scripts
#[derive(Debug, Clone, PartialEq)]
pub struct DeterminismChecker {
    /// Patterns detected in the script
    detections: Vec<DeterminismIssue>,
}

/// A single non-deterministic pattern detection
#[derive(Debug, Clone, PartialEq)]
pub struct DeterminismIssue {
    /// Line number where pattern was found (1-indexed)
    pub line: usize,
    /// Type of non-deterministic pattern
    pub pattern_type: NonDeterministicPattern,
    /// The actual code that triggered the detection
    pub code: String,
    /// Human-readable explanation
    pub explanation: String,
}

/// Types of non-deterministic patterns
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NonDeterministicPattern {
    /// $RANDOM variable
    Random,
    /// $$ (process ID)
    ProcessId,
    /// date command or timestamp generation
    Timestamp,
    /// $BASHPID variable
    BashPid,
    /// $SRANDOM (cryptographic random, Bash 5.1+)
    SecureRandom,
}

impl DeterminismChecker {
    /// Create a new determinism checker
    pub fn new() -> Self {
        Self {
            detections: Vec::new(),
        }
    }

    /// Scan bash script for non-deterministic patterns
    ///
    /// Returns: List of detected issues
    pub fn scan(&mut self, script: &str) -> Vec<DeterminismIssue> {
        // Clear previous results
        self.detections.clear();

        for (line_num, line) in script.lines().enumerate() {
            let line_num = line_num + 1; // 1-indexed

            // Detect $RANDOM
            if line.contains("$RANDOM") || line.contains("${RANDOM}") {
                self.detections.push(DeterminismIssue {
                    line: line_num,
                    pattern_type: NonDeterministicPattern::Random,
                    code: line.to_string(),
                    explanation:
                        "Uses $RANDOM which produces different values on each run".to_string(),
                });
            }

            // Detect $$ (process ID)
            if line.contains("$$") {
                self.detections.push(DeterminismIssue {
                    line: line_num,
                    pattern_type: NonDeterministicPattern::ProcessId,
                    code: line.to_string(),
                    explanation: "Uses $$ (process ID) which changes on each execution"
                        .to_string(),
                });
            }

            // Detect timestamps: date, $(date), `date`, etc.
            if line.contains("date")
                && (line.contains("$(date")
                    || line.contains("`date")
                    || line.contains("date +")
                    || line.contains("date -"))
            {
                self.detections.push(DeterminismIssue {
                    line: line_num,
                    pattern_type: NonDeterministicPattern::Timestamp,
                    code: line.to_string(),
                    explanation: "Uses date command which produces different values over time"
                        .to_string(),
                });
            }

            // Detect $BASHPID
            if line.contains("$BASHPID") || line.contains("${BASHPID}") {
                self.detections.push(DeterminismIssue {
                    line: line_num,
                    pattern_type: NonDeterministicPattern::BashPid,
                    code: line.to_string(),
                    explanation: "Uses $BASHPID which changes on each execution".to_string(),
                });
            }

            // Detect $SRANDOM (Bash 5.1+)
            if line.contains("$SRANDOM") || line.contains("${SRANDOM}") {
                self.detections.push(DeterminismIssue {
                    line: line_num,
                    pattern_type: NonDeterministicPattern::SecureRandom,
                    code: line.to_string(),
                    explanation:
                        "Uses $SRANDOM (cryptographic random) which produces different values on each run"
                            .to_string(),
                });
            }
        }

        self.detections.clone()
    }

    /// Check if script is deterministic (no issues found)
    pub fn is_deterministic(&self) -> bool {
        self.detections.is_empty()
    }

    /// Get count of issues by pattern type
    pub fn count_by_pattern(&self, pattern: NonDeterministicPattern) -> usize {
        self.detections
            .iter()
            .filter(|issue| issue.pattern_type == pattern)
            .count()
    }
}

impl Default for DeterminismChecker {
    fn default() -> Self {
        Self::new()
    }
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;

    // ===== REPL-011-001: DETERMINISM CHECKER TESTS =====

    /// Test: REPL-011-001-001 - Detect $RANDOM
    #[test]
    fn test_REPL_011_001_detect_random() {
        // ARRANGE: Script with $RANDOM
        let script = "SESSION_ID=$RANDOM\necho $SESSION_ID";
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Should detect $RANDOM
        assert_eq!(issues.len(), 1, "Should find 1 issue");
        assert_eq!(issues[0].line, 1);
        assert_eq!(issues[0].pattern_type, NonDeterministicPattern::Random);
        assert!(issues[0].explanation.contains("RANDOM"));
        assert!(!checker.is_deterministic());
    }

    /// Test: REPL-011-001-002 - Detect $$
    #[test]
    fn test_REPL_011_001_detect_pid() {
        // ARRANGE: Script with $$
        let script = "TMPFILE=/tmp/script_$$\necho $TMPFILE";
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Should detect $$
        assert_eq!(issues.len(), 1, "Should find 1 issue");
        assert_eq!(issues[0].line, 1);
        assert_eq!(issues[0].pattern_type, NonDeterministicPattern::ProcessId);
        assert!(issues[0].explanation.contains("process ID"));
        assert!(!checker.is_deterministic());
    }

    /// Test: REPL-011-001-003 - Detect date command
    #[test]
    fn test_REPL_011_001_detect_timestamp() {
        // ARRANGE: Script with date command
        let script = "RELEASE=$(date +%s)\necho $RELEASE";
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Should detect date command
        assert_eq!(issues.len(), 1, "Should find 1 issue");
        assert_eq!(issues[0].line, 1);
        assert_eq!(issues[0].pattern_type, NonDeterministicPattern::Timestamp);
        assert!(issues[0].explanation.contains("date"));
        assert!(!checker.is_deterministic());
    }

    /// Test: REPL-011-001-004 - Detect $BASHPID
    #[test]
    fn test_REPL_011_001_detect_bashpid() {
        // ARRANGE: Script with $BASHPID
        let script = "echo \"Running as $BASHPID\"";
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Should detect $BASHPID
        assert_eq!(issues.len(), 1, "Should find 1 issue");
        assert_eq!(issues[0].line, 1);
        assert_eq!(issues[0].pattern_type, NonDeterministicPattern::BashPid);
        assert!(issues[0].explanation.contains("BASHPID"));
    }

    /// Test: REPL-011-001-005 - Detect $SRANDOM
    #[test]
    fn test_REPL_011_001_detect_srandom() {
        // ARRANGE: Script with $SRANDOM (Bash 5.1+)
        let script = "TOKEN=$SRANDOM\necho $TOKEN";
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Should detect $SRANDOM
        assert_eq!(issues.len(), 1, "Should find 1 issue");
        assert_eq!(issues[0].line, 1);
        assert_eq!(issues[0].pattern_type, NonDeterministicPattern::SecureRandom);
        assert!(issues[0].explanation.contains("SRANDOM"));
    }

    /// Test: REPL-011-001-006 - Deterministic script passes
    #[test]
    fn test_REPL_011_001_deterministic_script() {
        // ARRANGE: Script without non-deterministic patterns
        let script = "VERSION=1.0.0\necho \"Release $VERSION\"";
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Should find no issues
        assert_eq!(issues.len(), 0, "Should find 0 issues");
        assert!(checker.is_deterministic());
    }

    /// Test: REPL-011-001-007 - Multiple patterns detected
    #[test]
    fn test_REPL_011_001_multiple_patterns() {
        // ARRANGE: Script with multiple non-deterministic patterns
        let script = r#"
SESSION_ID=$RANDOM
TMPFILE=/tmp/script_$$
TIMESTAMP=$(date +%s)
        "#
        .trim();
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Should detect all 3 patterns
        assert_eq!(issues.len(), 3, "Should find 3 issues");
        assert_eq!(
            checker.count_by_pattern(NonDeterministicPattern::Random),
            1
        );
        assert_eq!(
            checker.count_by_pattern(NonDeterministicPattern::ProcessId),
            1
        );
        assert_eq!(
            checker.count_by_pattern(NonDeterministicPattern::Timestamp),
            1
        );
    }

    /// Test: REPL-011-001-008 - Rescan clears previous results
    #[test]
    fn test_REPL_011_001_rescan_clears_previous() {
        // ARRANGE: Scanner with previous results
        let mut checker = DeterminismChecker::new();
        checker.scan("SESSION_ID=$RANDOM");
        assert_eq!(checker.detections.len(), 1);

        // ACT: Scan new script
        let issues = checker.scan("echo 'hello'");

        // ASSERT: Previous results should be cleared
        assert_eq!(issues.len(), 0);
        assert!(checker.is_deterministic());
    }

    /// Test: REPL-011-001-009 - Line numbers are correct
    #[test]
    fn test_REPL_011_001_line_numbers_correct() {
        // ARRANGE: Multi-line script with issues on different lines
        let script = "echo 'start'\nID=$RANDOM\necho 'middle'\nTMP=$$\necho 'end'";
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Line numbers should be correct
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].line, 2); // $RANDOM on line 2
        assert_eq!(issues[1].line, 4); // $$ on line 4
    }
}

// ===== PROPERTY TESTS =====

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Scripts with $RANDOM should always be detected
        #[test]
        fn prop_REPL_011_001_pattern_detection_never_false_negative(
            prefix in "[a-z_]{0,10}",
            suffix in "[a-z_]{0,10}"
        ) {
            // Scripts with $RANDOM should always be detected
            let script = format!("{}$RANDOM{}", prefix, suffix);
            let mut checker = DeterminismChecker::new();
            let issues = checker.scan(&script);

            prop_assert!(
                issues.iter().any(|i| i.pattern_type == NonDeterministicPattern::Random),
                "Should always detect $RANDOM: '{}'", script
            );
        }

        /// Property: Simple variable assignments should be deterministic
        #[test]
        fn prop_REPL_011_001_deterministic_scripts_pass(
            var_name in "[A-Z_]{1,10}",
            value in "[a-z0-9]{1,20}"
        ) {
            // Simple variable assignments should be deterministic
            let script = format!("{}={}", var_name, value);
            let mut checker = DeterminismChecker::new();
            let issues = checker.scan(&script);

            prop_assert_eq!(issues.len(), 0, "Simple assignments should be deterministic");
            prop_assert!(checker.is_deterministic());
        }

        /// Property: Scanner should never panic on any input
        #[test]
        fn prop_REPL_011_001_scan_never_panics(script in ".*{0,1000}") {
            // Scanner should never panic on any input
            let mut checker = DeterminismChecker::new();
            let _ = checker.scan(&script);
        }

        /// Property: Rescanning should always clear previous results
        #[test]
        fn prop_REPL_011_001_rescan_always_clears(
            script1 in ".*{0,100}",
            script2 in ".*{0,100}"
        ) {
            // Rescanning should always clear previous results
            let mut checker = DeterminismChecker::new();
            checker.scan(&script1);
            let issues2 = checker.scan(&script2);

            // Second scan results should match fresh scan
            let mut fresh_checker = DeterminismChecker::new();
            let fresh_issues = fresh_checker.scan(&script2);

            prop_assert_eq!(issues2.len(), fresh_issues.len());
        }
    }
}

// ===== REPL-011-002: REPLAY VERIFICATION =====

/// Verifies determinism by running scripts multiple times and comparing outputs
#[derive(Debug, Clone)]
pub struct ReplayVerifier {
    /// Number of replay runs to perform (default: 2, min: 2)
    replay_count: usize,
}

/// Result of replay verification
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayResult {
    /// Whether all runs produced identical output
    pub is_deterministic: bool,
    /// Outputs from each run
    pub runs: Vec<RunOutput>,
    /// Lines that differ between runs (if non-deterministic)
    pub differences: Vec<OutputDifference>,
}

/// Output from a single script execution
#[derive(Debug, Clone, PartialEq)]
pub struct RunOutput {
    /// Run number (1-indexed)
    pub run_number: usize,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
}

/// Difference between two runs
#[derive(Debug, Clone, PartialEq)]
pub struct OutputDifference {
    /// Line number where difference occurs (1-indexed)
    pub line: usize,
    /// Output from first run
    pub run1: String,
    /// Output from second run
    pub run2: String,
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
            Self::find_differences(&runs[0], &runs[1])
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
    fn execute_script(script: &str) -> RunOutput {
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
    fn find_differences(run1: &RunOutput, run2: &RunOutput) -> Vec<OutputDifference> {
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
        assert!(
            !result.differences.is_empty(),
            "Should have differences"
        );

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
