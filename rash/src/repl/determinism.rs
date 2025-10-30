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
