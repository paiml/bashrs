// ===== REPL-012-003: IDEMPOTENCY VERIFICATION =====

/// Verifies script idempotency by running multiple times and comparing results
///
/// Runs a script N times (default: 3) and checks if all runs produce
/// identical output and exit codes.
#[derive(Debug, Clone)]
pub struct IdempotencyVerifier {
    /// Number of verification runs to perform (default: 3, min: 2)
    run_count: usize,
}

impl IdempotencyVerifier {
    /// Create a new idempotency verifier with default settings (3 runs)
    pub fn new() -> Self {
        Self { run_count: 3 }
    }

    /// Create verifier with custom run count (minimum 2)
    pub fn with_run_count(count: usize) -> Self {
        Self {
            run_count: count.max(2),
        }
    }

    /// Verify script idempotency by running multiple times
    pub fn verify(&self, script: &str) -> IdempotencyResult {
        let mut runs = Vec::new();

        // Run script multiple times
        for run_number in 1..=self.run_count {
            let mut output = ReplayVerifier::execute_script(script);
            output.run_number = run_number;
            runs.push(output);
        }

        // Compare all runs against the first run
        let mut differences = Vec::new();
        if let Some(first_run) = runs.first() {
            // Compare each subsequent run with the first run
            for run in runs.iter().skip(1) {
                let run_diffs = ReplayVerifier::find_differences(first_run, run);
                // Merge differences (avoid duplicates)
                for diff in run_diffs {
                    if !differences.iter().any(|d: &OutputDifference| {
                        d.line == diff.line && d.run1 == diff.run1 && d.run2 == diff.run2
                    }) {
                        differences.push(diff);
                    }
                }
            }
        }

        let is_idempotent = differences.is_empty();

        IdempotencyResult {
            is_idempotent,
            run_count: self.run_count,
            runs,
            differences,
        }
    }
}

impl Default for IdempotencyVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of idempotency verification
#[derive(Debug, Clone, PartialEq)]
pub struct IdempotencyResult {
    /// Whether all runs produced identical output
    pub is_idempotent: bool,
    /// Number of runs performed
    pub run_count: usize,
    /// Outputs from each run
    pub runs: Vec<RunOutput>,
    /// Lines that differ between runs (if non-idempotent)
    pub differences: Vec<OutputDifference>,
}

impl IdempotencyResult {
    /// Format verification result for display
    pub fn format_result(&self) -> String {
        let mut output = String::new();

        // Show idempotency status
        if self.is_idempotent {
            output.push_str("✓ Script is idempotent\n");
            output.push_str(&format!("  Verified across {} runs\n", self.run_count));
        } else {
            output.push_str("❌ Script is non-idempotent\n");
            output.push_str(&format!("  Tested {} runs\n", self.run_count));
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
        assert_eq!(
            issues[0].operation_type,
            NonIdempotentOperation::MkdirWithoutP
        );
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
        assert_eq!(
            checker.count_by_operation(NonIdempotentOperation::MkdirWithoutP),
            1
        );
        assert_eq!(
            checker.count_by_operation(NonIdempotentOperation::RmWithoutF),
            1
        );
        assert_eq!(
            checker.count_by_operation(NonIdempotentOperation::LnWithoutF),
            1
        );
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
    include!("determinism_idempotency_idempotencyverifier_prop_repl_012_001_mkdir_without_p_always_detected.rs");
}
