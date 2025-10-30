# TICKET: REPL-011-002

## Title
Replay Verification (Run Script Twice, Compare Outputs)

## Priority
**P1 - High** (Second task in REPL-011 Determinism Checker sprint)

## Status
🟢 **READY TO START** - Dependencies met (REPL-011-001 completed)

## Context
Building on REPL-011-001 (pattern detection), this task adds **replay verification** - the gold standard for proving determinism.

**Concept**: A truly deterministic script produces **identical output** on every execution (given same inputs/environment).

**Why Replay Verification?**
- Pattern detection (REPL-011-001) catches known anti-patterns
- Replay verification catches **all** non-determinism, even subtle cases
- Provides definitive proof: "This script IS/IS NOT deterministic"

**Purpose**: Verify determinism through actual execution rather than static analysis alone.

## Dependencies
- ✅ REPL-011-001 (Pattern detection) completed
- ✅ DeterminismChecker exists
- ✅ REPL can execute bash scripts

## Acceptance Criteria

### 1. Add `ReplayVerifier` struct

```rust
/// Verifies determinism by running scripts multiple times and comparing outputs
#[derive(Debug, Clone)]
pub struct ReplayVerifier {
    /// Number of replay runs to perform (default: 2)
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
    /// Line number where difference occurs
    pub line: usize,
    /// Output from first run
    pub run1: String,
    /// Output from second run
    pub run2: String,
}

impl ReplayVerifier {
    /// Create a new replay verifier
    pub fn new() -> Self {
        Self { replay_count: 2 }
    }

    /// Set number of replay runs (default: 2, min: 2)
    pub fn with_replay_count(mut self, count: usize) -> Self {
        self.replay_count = count.max(2);
        self
    }

    /// Verify determinism by running script multiple times
    ///
    /// Returns: ReplayResult with comparison of all runs
    pub fn verify(&self, script: &str) -> ReplayResult {
        let mut runs = Vec::new();

        // Run script multiple times
        for run_number in 1..=self.replay_count {
            let output = Self::execute_script(script);
            runs.push(RunOutput {
                run_number,
                stdout: output.stdout,
                stderr: output.stderr,
                exit_code: output.exit_code,
            });
        }

        // Compare outputs
        let differences = Self::find_differences(&runs[0], &runs[1]);
        let is_deterministic = differences.is_empty();

        ReplayResult {
            is_deterministic,
            runs,
            differences,
        }
    }

    /// Execute bash script and capture output
    fn execute_script(script: &str) -> RunOutput {
        // Use std::process::Command to execute script
        use std::process::Command;

        let output = Command::new("bash")
            .arg("-c")
            .arg(script)
            .output()
            .expect("Failed to execute script");

        RunOutput {
            run_number: 0, // Will be set by caller
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
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
                    line: i + 1,
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
```

### 2. Unit Tests (RED → GREEN → REFACTOR)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ===== REPL-011-002: REPLAY VERIFICATION TESTS =====

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
        assert!(result.is_deterministic, "Simple script should be deterministic");
        assert_eq!(result.runs.len(), 2);
        assert_eq!(result.differences.len(), 0);

        // Both runs should have identical output
        assert_eq!(result.runs[0].stdout, result.runs[1].stdout);
    }

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
        assert!(!result.is_deterministic, "Script with $RANDOM should be non-deterministic");
        assert_eq!(result.runs.len(), 2);
        assert!(!result.differences.is_empty(), "Should have differences");

        // Runs should have different output
        assert_ne!(result.runs[0].stdout, result.runs[1].stdout);
    }

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
```

### 3. Property Tests

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
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
            prop_assert_eq!(result.runs[0].stdout, result.runs[1].stdout);
        }

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
```

### 4. Quality Gates

- [ ] ✅ All unit tests pass (≥8 tests)
- [ ] ✅ All property tests pass (≥3 tests)
- [ ] ✅ Coverage >85%
- [ ] ✅ Clippy warnings: 0
- [ ] ✅ Complexity <10 per function
- [ ] ✅ Integration with DeterminismChecker
- [ ] ✅ Mutation score ≥90% (deferred to separate pass)

## EXTREME TDD Methodology

### RED Phase
1. Write failing test: `test_REPL_011_002_deterministic_script`
2. Write failing test: `test_REPL_011_002_nondeterministic_script`
3. Write failing test: `test_REPL_011_002_multiple_replays`
4. Write failing test: `test_REPL_011_002_difference_detection`
5. Write failing test: `test_REPL_011_002_empty_script`
6. Write failing test: `test_REPL_011_002_multiline_output`
7. Write failing test: `test_REPL_011_002_exit_code_tracking`
8. Write failing test: `test_REPL_011_002_min_replay_count`
9. Run tests → **FAIL** ✅ (expected)

### GREEN Phase
1. Implement `ReplayVerifier` struct
2. Implement `ReplayResult`, `RunOutput`, `OutputDifference` structs
3. Implement `verify()` method
4. Implement `execute_script()` helper (using std::process::Command)
5. Implement `find_differences()` helper
6. Run tests → **PASS** ✅

### REFACTOR Phase
1. Extract script execution into separate module if needed
2. Consider async execution for parallel runs (future optimization)
3. Keep complexity <10
4. Run tests → **PASS** ✅

### PROPERTY Phase
1. Add property test: `prop_REPL_011_002_deterministic_scripts_always_identical`
2. Add property test: `prop_REPL_011_002_multiple_runs_consistent`
3. Add property test: `prop_REPL_011_002_verify_never_panics`
4. Run property tests (100+ cases) → **PASS** ✅

### MUTATION Phase (Deferred)
1. Run `cargo mutants --file rash/src/repl/determinism.rs`
2. Target: ≥90% kill rate
3. Address any MISSED mutants

## Implementation Plan

### Files to Modify
- `rash/src/repl/determinism.rs` - Add ReplayVerifier (after DeterminismChecker)

### Files to Create
- None (extends existing determinism module)

### Test Files
- `rash/src/repl/determinism.rs` - Unit tests in module
- `rash/src/repl/determinism.rs` - Property tests in module

## Task Breakdown

- [ ] **Task 1**: Write RED tests for ReplayVerifier
- [ ] **Task 2**: Implement ReplayVerifier struct and helpers (GREEN phase)
- [ ] **Task 3**: Implement verify() method with script execution
- [ ] **Task 4**: Refactor if needed (REFACTOR phase)
- [ ] **Task 5**: Add property tests (PROPERTY phase)
- [ ] **Task 6**: Verify all quality gates
- [ ] **Task 7**: Update roadmap (mark REPL-011-002 complete)
- [ ] **Task 8**: Commit with proper message

## Example Usage

```bash
$ bashrs repl
bashrs> :debug examples/deploy.sh
bashrs> :verify-replay

Running determinism replay verification (2 runs)...

Run 1: exit 0 (3 lines output)
Run 2: exit 0 (3 lines output)

❌ Non-deterministic output detected!

Differences:
Line 1:
  Run 1: Random: 12345
  Run 2: Random: 67890

Suggestion: Remove $RANDOM or use fixed seed
```

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- EXTREME TDD ensures replay verification is correct from first line
- Property tests catch edge cases in output comparison

### 反省 (Hansei) - Reflect and Improve
- Learn from REPL-011-001 (pattern detection)
- Combine static and dynamic analysis for comprehensive verification

### 改善 (Kaizen) - Continuous Improvement
- Move beyond pattern detection to actual execution verification
- Gold standard for determinism proof

### 現地現物 (Genchi Genbutsu) - Go and See
- Actually run the scripts, don't just analyze them
- Observe real behavior in real environments

## Related Files
- `rash/src/repl/determinism.rs` - DeterminismChecker (REPL-011-001)
- `rash/src/repl/executor.rs` - Script execution (reference)
- `docs/REPL-DEBUGGER-ROADMAP.yaml` - Roadmap reference

## Comparison: Static vs Dynamic Analysis

| Feature | REPL-011-001 (Pattern Detection) | REPL-011-002 (Replay Verification) |
|---------|----------------------------------|-------------------------------------|
| Method | Static analysis | Dynamic execution |
| Speed | Fast (no execution) | Slower (runs script) |
| Coverage | Known patterns only | All non-determinism |
| False positives | Possible | None (actual behavior) |
| False negatives | Possible (unknown patterns) | None (if runs differ, non-deterministic) |
| Use case | Quick scan | Definitive proof |

**Combined Power**: Use pattern detection for quick feedback, replay verification for proof!

## Success Criteria Summary
```
BEFORE: Pattern detection only (static analysis)
AFTER:  ✅ Replay verification runs scripts multiple times
        ✅ Compares outputs to verify determinism
        ✅ Detects ALL non-determinism (not just known patterns)
        ✅ Provides line-by-line diff of differences
        ✅ All quality gates passed
        ✅ Property tests validate verification
```

---

**Created**: 2025-10-30
**Sprint**: REPL-011 (Determinism Checker)
**Estimated Time**: 2-3 hours
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY → MUTATION)

---
