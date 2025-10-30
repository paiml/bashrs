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
/// use bashrs::repl::determinism::format_replay_diff;
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

// ===== REPL-011-003: DIFF EXPLANATION TESTS =====

#[cfg(test)]
mod diff_tests {
    use super::*;

    #[test]
    fn test_REPL_011_003_format_single_difference() {
        // ARRANGE: One difference
        let differences = vec![OutputDifference {
            line: 1,
            run1: "Random: 12345".to_string(),
            run2: "Random: 67890".to_string(),
        }];

        // ACT: Format diff
        let formatted = format_replay_diff(&differences);

        // ASSERT: Should show line number and both outputs
        assert!(formatted.contains("Line 1:"));
        assert!(formatted.contains("Run 1: Random: 12345"));
        assert!(formatted.contains("Run 2: Random: 67890"));
    }

    #[test]
    fn test_REPL_011_003_format_multiple_differences() {
        // ARRANGE: Multiple differences
        let differences = vec![
            OutputDifference {
                line: 1,
                run1: "First: abc".to_string(),
                run2: "First: xyz".to_string(),
            },
            OutputDifference {
                line: 3,
                run1: "Third: 123".to_string(),
                run2: "Third: 456".to_string(),
            },
        ];

        // ACT: Format diff
        let formatted = format_replay_diff(&differences);

        // ASSERT: Should show all differences
        assert!(formatted.contains("Line 1:"));
        assert!(formatted.contains("Line 3:"));
        assert!(formatted.contains("2 difference"));
    }

    #[test]
    fn test_REPL_011_003_format_no_differences() {
        // ARRANGE: Empty differences
        let differences = vec![];

        // ACT: Format diff
        let formatted = format_replay_diff(&differences);

        // ASSERT: Should show success message
        assert!(formatted.contains("deterministic"));
    }

    #[test]
    fn test_REPL_011_003_format_empty_lines() {
        // ARRANGE: Differences with empty output
        let differences = vec![OutputDifference {
            line: 5,
            run1: "".to_string(),
            run2: "Something appeared".to_string(),
        }];

        // ACT: Format diff
        let formatted = format_replay_diff(&differences);

        // ASSERT: Should handle empty strings
        assert!(formatted.contains("Line 5:"));
        assert!(formatted.contains("Something appeared"));
    }

    #[test]
    fn test_REPL_011_003_replay_result_format() {
        // ARRANGE: Non-deterministic replay result
        let result = ReplayResult {
            is_deterministic: false,
            runs: vec![
                RunOutput {
                    run_number: 1,
                    stdout: "Random: 12345\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 2,
                    stdout: "Random: 67890\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            ],
            differences: vec![OutputDifference {
                line: 1,
                run1: "Random: 12345".to_string(),
                run2: "Random: 67890".to_string(),
            }],
        };

        // ACT: Format result
        let formatted = result.format_result();

        // ASSERT: Should show status, runs, and differences
        assert!(formatted.contains("Runs: 2"));
        assert!(formatted.contains("Exit codes:"));
        assert!(formatted.contains("Line 1:"));
    }

    #[test]
    fn test_REPL_011_003_deterministic_result_format() {
        // ARRANGE: Deterministic replay result
        let result = ReplayResult {
            is_deterministic: true,
            runs: vec![
                RunOutput {
                    run_number: 1,
                    stdout: "Constant output\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 2,
                    stdout: "Constant output\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            ],
            differences: vec![],
        };

        // ACT: Format result
        let formatted = result.format_result();

        // ASSERT: Should show success status
        assert!(formatted.contains("deterministic"));
    }
}

#[cfg(test)]
mod diff_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_011_003_format_never_panics(
            num_diffs in 0usize..10,
            line in 1usize..100,
        ) {
            // Generate arbitrary differences
            let differences: Vec<OutputDifference> = (0..num_diffs)
                .map(|i| OutputDifference {
                    line: line + i,
                    run1: format!("run1_{}", i),
                    run2: format!("run2_{}", i),
                })
                .collect();

            // Format should never panic
            let _ = format_replay_diff(&differences);
        }

        #[test]
        fn prop_REPL_011_003_format_preserves_line_numbers(
            line_num in 1usize..1000,
        ) {
            let differences = vec![OutputDifference {
                line: line_num,
                run1: "a".to_string(),
                run2: "b".to_string(),
            }];

            let formatted = format_replay_diff(&differences);

            // Line number should appear in output
            prop_assert!(
                formatted.contains(&format!("Line {}:", line_num)),
                "Should contain line number {}: {}",
                line_num,
                formatted
            );
        }

        #[test]
        fn prop_REPL_011_003_empty_always_deterministic(
            _any in 0..100,
        ) {
            let formatted = format_replay_diff(&[]);

            // Empty differences should always show deterministic
            prop_assert!(
                formatted.contains("deterministic"),
                "Empty diff should show deterministic: {}",
                formatted
            );
        }
    }
}

// ===== IDEMPOTENCY CHECKER (REPL-012-001) =====

// Idempotency checking for bash scripts
//
// Detects non-idempotent operations that may fail on re-run:
// - mkdir without -p: Fails if directory exists
// - rm without -f: Fails if file doesn't exist
// - ln -s without -f: Fails if symlink exists

/// Detects non-idempotent operations in bash scripts
#[derive(Debug, Clone, PartialEq)]
pub struct IdempotencyChecker {
    /// Issues detected in the script
    detections: Vec<IdempotencyIssue>,
}

/// A single non-idempotent operation detection
#[derive(Debug, Clone, PartialEq)]
pub struct IdempotencyIssue {
    /// Line number where operation was found (1-indexed)
    pub line: usize,
    /// Type of non-idempotent operation
    pub operation_type: NonIdempotentOperation,
    /// The actual code that triggered the detection
    pub code: String,
    /// Human-readable explanation
    pub explanation: String,
    /// Suggested fix
    pub suggestion: String,
}

/// Types of non-idempotent operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonIdempotentOperation {
    /// mkdir without -p flag
    MkdirWithoutP,
    /// rm without -f flag
    RmWithoutF,
    /// ln -s without -f flag
    LnWithoutF,
}

impl IdempotencyChecker {
    /// Create a new idempotency checker
    pub fn new() -> Self {
        Self {
            detections: Vec::new(),
        }
    }

    /// Scan bash script for non-idempotent operations
    ///
    /// Returns: List of detected issues
    pub fn scan(&mut self, script: &str) -> Vec<IdempotencyIssue> {
        // Clear previous results
        self.detections.clear();

        for (line_num, line) in script.lines().enumerate() {
            let line_num = line_num + 1; // 1-indexed

            // Detect mkdir without -p
            if line.contains("mkdir ") && !line.contains("mkdir -p") {
                self.detections.push(IdempotencyIssue {
                    line: line_num,
                    operation_type: NonIdempotentOperation::MkdirWithoutP,
                    code: line.to_string(),
                    explanation: "mkdir without -p fails if directory already exists".to_string(),
                    suggestion: "Add -p flag: mkdir -p".to_string(),
                });
            }

            // Detect rm without -f (but not rm -rf)
            if line.contains("rm ") && !line.contains(" -f") && !line.contains(" -rf") {
                self.detections.push(IdempotencyIssue {
                    line: line_num,
                    operation_type: NonIdempotentOperation::RmWithoutF,
                    code: line.to_string(),
                    explanation: "rm without -f fails if file doesn't exist".to_string(),
                    suggestion: "Add -f flag: rm -f".to_string(),
                });
            }

            // Detect ln -s without -f
            if line.contains("ln -s") && !line.contains("ln -sf") {
                self.detections.push(IdempotencyIssue {
                    line: line_num,
                    operation_type: NonIdempotentOperation::LnWithoutF,
                    code: line.to_string(),
                    explanation: "ln -s without -f fails if symlink already exists".to_string(),
                    suggestion: "Add -f flag: ln -sf".to_string(),
                });
            }
        }

        self.detections.clone()
    }

    /// Check if script is idempotent (no issues found)
    pub fn is_idempotent(&self) -> bool {
        self.detections.is_empty()
    }

    /// Get count of issues by operation type
    pub fn count_by_operation(&self, operation: NonIdempotentOperation) -> usize {
        self.detections
            .iter()
            .filter(|issue| issue.operation_type == operation)
            .count()
    }
}

impl Default for IdempotencyChecker {
    fn default() -> Self {
        Self::new()
    }
}

// ===== IDEMPOTENCY TESTS =====

#[cfg(test)]
mod idempotency_tests {
    use super::*;

    // ===== REPL-012-001: IDEMPOTENCY SCANNER TESTS =====

    #[test]
    fn test_REPL_012_001_detect_mkdir_without_p() {
        // ARRANGE: Script with mkdir (no -p)
        let script = "mkdir /tmp/testdir";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect mkdir without -p
        assert_eq!(issues.len(), 1, "Should detect 1 issue");
        assert_eq!(issues[0].operation_type, NonIdempotentOperation::MkdirWithoutP);
        assert_eq!(issues[0].line, 1);
        assert!(issues[0].explanation.contains("mkdir"));
        assert!(issues[0].suggestion.contains("-p"));
    }

    #[test]
    fn test_REPL_012_001_mkdir_with_p_is_idempotent() {
        // ARRANGE: Script with mkdir -p
        let script = "mkdir -p /tmp/testdir";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should not detect any issues
        assert_eq!(issues.len(), 0, "mkdir -p should be idempotent");
        assert!(checker.is_idempotent());
    }

    #[test]
    fn test_REPL_012_001_detect_rm_without_f() {
        // ARRANGE: Script with rm (no -f)
        let script = "rm /tmp/testfile";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect rm without -f
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].operation_type, NonIdempotentOperation::RmWithoutF);
        assert!(issues[0].explanation.contains("rm"));
        assert!(issues[0].suggestion.contains("-f"));
    }

    #[test]
    fn test_REPL_012_001_rm_with_f_is_idempotent() {
        // ARRANGE: Script with rm -f
        let script = "rm -f /tmp/testfile";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should not detect any issues
        assert_eq!(issues.len(), 0, "rm -f should be idempotent");
    }

    #[test]
    fn test_REPL_012_001_detect_ln_without_f() {
        // ARRANGE: Script with ln -s (no -f)
        let script = "ln -s /source /target";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect ln -s without -f
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].operation_type, NonIdempotentOperation::LnWithoutF);
        assert!(issues[0].explanation.contains("ln"));
        assert!(issues[0].suggestion.contains("-f"));
    }

    #[test]
    fn test_REPL_012_001_ln_with_sf_is_idempotent() {
        // ARRANGE: Script with ln -sf
        let script = "ln -sf /source /target";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should not detect any issues
        assert_eq!(issues.len(), 0, "ln -sf should be idempotent");
    }

    #[test]
    fn test_REPL_012_001_multiple_issues() {
        // ARRANGE: Script with multiple non-idempotent operations
        let script = r#"
mkdir /tmp/dir1
rm /tmp/file1
ln -s /source /target
mkdir -p /tmp/dir2
        "#;
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect 3 issues (line 5 is idempotent)
        assert_eq!(issues.len(), 3);
        assert_eq!(checker.count_by_operation(NonIdempotentOperation::MkdirWithoutP), 1);
        assert_eq!(checker.count_by_operation(NonIdempotentOperation::RmWithoutF), 1);
        assert_eq!(checker.count_by_operation(NonIdempotentOperation::LnWithoutF), 1);
    }

    #[test]
    fn test_REPL_012_001_fully_idempotent_script() {
        // ARRANGE: Fully idempotent script
        let script = r#"
#!/bin/sh
mkdir -p /app/releases
rm -f /app/current
ln -sf /app/releases/v1.0.0 /app/current
        "#;
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect no issues
        assert_eq!(issues.len(), 0);
        assert!(checker.is_idempotent());
    }

    #[test]
    fn test_REPL_012_001_rescan_clears_previous() {
        // ARRANGE: Checker with previous detections
        let mut checker = IdempotencyChecker::new();
        checker.scan("mkdir /tmp/test1");
        assert_eq!(checker.scan("mkdir /tmp/test1").len(), 1);

        // ACT: Scan new script
        let issues = checker.scan("mkdir -p /tmp/test2");

        // ASSERT: Should clear previous detections
        assert_eq!(issues.len(), 0, "Rescan should clear previous detections");
        assert!(checker.is_idempotent());
    }

    #[test]
    fn test_REPL_012_001_line_numbers_correct() {
        // ARRANGE: Multi-line script
        let script = r#"
# Line 1: comment
mkdir /tmp/dir  # Line 2: issue
# Line 3: comment
rm /tmp/file    # Line 4: issue
        "#;
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Line numbers should be correct
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].line, 3); // Line 3 in 1-indexed
        assert_eq!(issues[1].line, 5); // Line 5 in 1-indexed
    }
}

#[cfg(test)]
mod idempotency_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_012_001_mkdir_without_p_always_detected(
            path in "[a-z0-9/]{1,50}"
        ) {
            let script = format!("mkdir {}", path);
            let mut checker = IdempotencyChecker::new();
            let issues = checker.scan(&script);

            // mkdir without -p should always be detected
            prop_assert_eq!(issues.len(), 1);
            prop_assert_eq!(issues[0].operation_type, NonIdempotentOperation::MkdirWithoutP);
        }

        #[test]
        fn prop_REPL_012_001_mkdir_with_p_never_detected(
            path in "[a-z0-9/]{1,50}"
        ) {
            let script = format!("mkdir -p {}", path);
            let mut checker = IdempotencyChecker::new();
            let issues = checker.scan(&script);

            // mkdir -p should never be detected as non-idempotent
            prop_assert_eq!(issues.len(), 0);
            prop_assert!(checker.is_idempotent());
        }

        #[test]
        fn prop_REPL_012_001_scan_never_panics(
            script in ".*{0,1000}"
        ) {
            let mut checker = IdempotencyChecker::new();
            // Should never panic on any input
            let _ = checker.scan(&script);
        }

        #[test]
        fn prop_REPL_012_001_rescan_always_clears(
            script1 in "mkdir [a-z]{1,20}",
            script2 in "mkdir -p [a-z]{1,20}"
        ) {
            let mut checker = IdempotencyChecker::new();

            // First scan should find issue
            let issues1 = checker.scan(&script1);
            prop_assert_eq!(issues1.len(), 1);

            // Second scan should clear and find no issues
            let issues2 = checker.scan(&script2);
            prop_assert_eq!(issues2.len(), 0);
            prop_assert!(checker.is_idempotent());
        }
    }
}

// ===== REPL-012-002: IDEMPOTENCY REPORT FORMATTING =====

/// Format idempotency scan results for display
///
/// Shows a user-friendly report of non-idempotent operations with
/// line numbers, problematic code, explanations, and fix suggestions.
///
/// # Examples
///
/// ```
/// use bashrs::repl::determinism::{IdempotencyIssue, NonIdempotentOperation};
/// use bashrs::repl::format_idempotency_report;
///
/// let issues = vec![
///     IdempotencyIssue {
///         line: 5,
///         operation_type: NonIdempotentOperation::MkdirWithoutP,
///         code: "mkdir /tmp/foo".to_string(),
///         explanation: "mkdir without -p fails if directory already exists".to_string(),
///         suggestion: "Add -p flag: mkdir -p".to_string(),
///     },
/// ];
///
/// let formatted = format_idempotency_report(&issues);
/// assert!(formatted.contains("Line 5:"));
/// assert!(formatted.contains("mkdir /tmp/foo"));
/// assert!(formatted.contains("Add -p flag"));
/// ```
pub fn format_idempotency_report(issues: &[IdempotencyIssue]) -> String {
    if issues.is_empty() {
        return String::from("✓ Script is idempotent - safe to re-run");
    }

    let mut output = String::new();
    output.push_str("⚠️  Non-idempotent operations detected!\n\n");
    output.push_str(&format!("Found {} issue(s):\n\n", issues.len()));

    for issue in issues {
        output.push_str(&format!("Line {}:\n", issue.line));
        output.push_str(&format!("  Code:        {}\n", issue.code));
        output.push_str(&format!("  Problem:     {}\n", issue.explanation));
        output.push_str(&format!("  💡 Fix:      {}\n", issue.suggestion));
        output.push('\n');
    }

    output
}

impl IdempotencyChecker {
    /// Format scan results for display
    ///
    /// Shows idempotency status, issue counts, operation breakdown,
    /// and detailed issue reports.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::IdempotencyChecker;
    ///
    /// let mut checker = IdempotencyChecker::new();
    /// let script = "mkdir /tmp/foo\nrm /tmp/bar";
    /// checker.scan(script);
    ///
    /// let report = checker.format_report();
    /// assert!(report.contains("non-idempotent") || report.contains("idempotent"));
    /// ```
    pub fn format_report(&self) -> String {
        let mut output = String::new();

        // Show idempotency status
        if self.is_idempotent() {
            output.push_str("✓ Script is idempotent\n");
            output.push_str("  Safe to run multiple times\n");
        } else {
            output.push_str("⚠️  Script is non-idempotent\n");
            output.push_str(&format!("  {} issue(s) found\n", self.detections.len()));
        }

        // Show issue breakdown by type
        let mkdir_count = self.count_by_operation(NonIdempotentOperation::MkdirWithoutP);
        let rm_count = self.count_by_operation(NonIdempotentOperation::RmWithoutF);
        let ln_count = self.count_by_operation(NonIdempotentOperation::LnWithoutF);

        if mkdir_count > 0 || rm_count > 0 || ln_count > 0 {
            output.push_str("\nOperation breakdown:\n");
            if mkdir_count > 0 {
                output.push_str(&format!("  mkdir (missing -p): {}\n", mkdir_count));
            }
            if rm_count > 0 {
                output.push_str(&format!("  rm (missing -f):    {}\n", rm_count));
            }
            if ln_count > 0 {
                output.push_str(&format!("  ln (missing -f):    {}\n", ln_count));
            }
        }

        // Show detailed issues
        if !self.detections.is_empty() {
            output.push('\n');
            output.push_str(&format_idempotency_report(&self.detections));
        }

        output
    }
}

#[cfg(test)]
mod idempotency_report_tests {
    use super::*;

    // ===== REPL-012-002: REPORT FORMATTING TESTS =====

    #[test]
    fn test_REPL_012_002_format_single_mkdir_issue() {
        // ARRANGE: One mkdir issue
        let issues = vec![IdempotencyIssue {
            line: 10,
            operation_type: NonIdempotentOperation::MkdirWithoutP,
            code: "mkdir /tmp/test".to_string(),
            explanation: "mkdir without -p fails if directory already exists".to_string(),
            suggestion: "Add -p flag: mkdir -p".to_string(),
        }];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show line, code, problem, and fix
        assert!(formatted.contains("Line 10:"));
        assert!(formatted.contains("mkdir /tmp/test"));
        assert!(formatted.contains("mkdir without -p"));
        assert!(formatted.contains("Add -p flag"));
        assert!(formatted.contains("⚠️") || formatted.contains("warning"));
    }

    #[test]
    fn test_REPL_012_002_format_single_rm_issue() {
        // ARRANGE: One rm issue
        let issues = vec![IdempotencyIssue {
            line: 15,
            operation_type: NonIdempotentOperation::RmWithoutF,
            code: "rm /tmp/file.txt".to_string(),
            explanation: "rm without -f fails if file doesn't exist".to_string(),
            suggestion: "Add -f flag: rm -f".to_string(),
        }];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show rm-specific details
        assert!(formatted.contains("Line 15:"));
        assert!(formatted.contains("rm /tmp/file.txt"));
        assert!(formatted.contains("rm without -f"));
        assert!(formatted.contains("Add -f flag"));
    }

    #[test]
    fn test_REPL_012_002_format_multiple_issues() {
        // ARRANGE: Multiple issues
        let issues = vec![
            IdempotencyIssue {
                line: 5,
                operation_type: NonIdempotentOperation::MkdirWithoutP,
                code: "mkdir foo".to_string(),
                explanation: "mkdir without -p fails if directory already exists".to_string(),
                suggestion: "Add -p flag: mkdir -p".to_string(),
            },
            IdempotencyIssue {
                line: 10,
                operation_type: NonIdempotentOperation::RmWithoutF,
                code: "rm bar".to_string(),
                explanation: "rm without -f fails if file doesn't exist".to_string(),
                suggestion: "Add -f flag: rm -f".to_string(),
            },
        ];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show all issues
        assert!(formatted.contains("Line 5:"));
        assert!(formatted.contains("Line 10:"));
        assert!(formatted.contains("Found 2 issue(s)") || formatted.contains("2 issue"));
    }

    #[test]
    fn test_REPL_012_002_format_no_issues() {
        // ARRANGE: Empty issues
        let issues = vec![];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show success message
        assert!(formatted.contains("✓") || formatted.contains("success"));
        assert!(formatted.contains("idempotent"));
        assert!(formatted.contains("safe") || formatted.contains("re-run"));
    }

    #[test]
    fn test_REPL_012_002_checker_format_report() {
        // ARRANGE: Checker with detected issues
        let mut checker = IdempotencyChecker::new();
        let script = r#"
mkdir /tmp/foo
rm /tmp/bar
ln -s /tmp/baz /tmp/link
"#;
        checker.scan(script);

        // ACT: Format report
        let formatted = checker.format_report();

        // ASSERT: Should show status and breakdown
        assert!(formatted.contains("non-idempotent") || formatted.contains("issue"));
        assert!(formatted.contains("mkdir") || formatted.contains("rm") || formatted.contains("ln"));
    }

    #[test]
    fn test_REPL_012_002_checker_format_idempotent() {
        // ARRANGE: Checker with idempotent script
        let mut checker = IdempotencyChecker::new();
        let script = r#"
mkdir -p /tmp/foo
rm -f /tmp/bar
ln -sf /tmp/baz /tmp/link
"#;
        checker.scan(script);

        // ACT: Format report
        let formatted = checker.format_report();

        // ASSERT: Should show success status
        assert!(formatted.contains("✓") || formatted.contains("success"));
        assert!(formatted.contains("idempotent"));
    }

    #[test]
    fn test_REPL_012_002_format_preserves_line_numbers() {
        // ARRANGE: Issues with specific line numbers
        let issues = vec![
            IdempotencyIssue {
                line: 42,
                operation_type: NonIdempotentOperation::MkdirWithoutP,
                code: "mkdir test".to_string(),
                explanation: "explanation".to_string(),
                suggestion: "suggestion".to_string(),
            },
        ];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Line number should be preserved
        assert!(formatted.contains("Line 42:") || formatted.contains("42"));
    }
}

#[cfg(test)]
mod idempotency_report_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_012_002_format_never_panics(
            num_issues in 0usize..10,
            line in 1usize..100,
        ) {
            // Generate arbitrary issues
            let issues: Vec<IdempotencyIssue> = (0..num_issues)
                .map(|i| IdempotencyIssue {
                    line: line + i,
                    operation_type: NonIdempotentOperation::MkdirWithoutP,
                    code: format!("mkdir /tmp/test{}", i),
                    explanation: "explanation".to_string(),
                    suggestion: "suggestion".to_string(),
                })
                .collect();

            // Format should never panic
            let _ = format_idempotency_report(&issues);
        }

        #[test]
        fn prop_REPL_012_002_empty_always_idempotent(
            _any in 0..100,
        ) {
            let formatted = format_idempotency_report(&[]);

            // Empty issues should always show idempotent
            prop_assert!(
                (formatted.contains("✓") || formatted.contains("success")) && formatted.contains("idempotent"),
                "Empty report should show idempotent: {}",
                formatted
            );
        }

        #[test]
        fn prop_REPL_012_002_count_matches_issues(
            num_issues in 1usize..20,
        ) {
            let issues: Vec<IdempotencyIssue> = (0..num_issues)
                .map(|i| IdempotencyIssue {
                    line: i + 1,
                    operation_type: NonIdempotentOperation::RmWithoutF,
                    code: format!("rm file{}", i),
                    explanation: "explanation".to_string(),
                    suggestion: "suggestion".to_string(),
                })
                .collect();

            let formatted = format_idempotency_report(&issues);

            // Count should match number of issues (flexible matching for different formats)
            let count_str = format!("{}", num_issues);
            prop_assert!(
                formatted.contains(&count_str),
                "Report should show count {}: {}",
                num_issues,
                formatted
            );
        }
    }
}
