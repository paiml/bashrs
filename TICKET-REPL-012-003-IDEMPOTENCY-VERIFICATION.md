# TICKET: REPL-012-003

## Title
Idempotency Verification (Run 3+ Times, Check Same Result)

## Priority
**P1 - High** (Third task in REPL-012 Idempotency Analyzer sprint)

## Status
🟢 **READY TO START** - Dependencies met (REPL-012-001 completed)

## Context
Building on REPL-012-001 (static idempotency scanning), this task adds **runtime idempotency verification** by executing scripts multiple times.

**Concept**: Run a script N times (default 3) and verify that successive runs produce identical results and leave the system in the same state.

**Why Runtime Verification?**
- REPL-012-001 detects potential issues through static analysis
- Runtime verification proves scripts are actually idempotent through execution
- Catches issues that static analysis might miss (side effects, state changes)
- Provides confidence that scripts are safe to re-run

**Purpose**: Verify idempotency through actual execution and result comparison.

## Dependencies
- ✅ REPL-012-001 (Idempotency scanner) completed
- ✅ REPL-011-002 (ReplayVerifier) available as reference pattern
- ✅ `RunOutput` and `OutputDifference` structs exist

## Acceptance Criteria

### 1. Add `IdempotencyVerifier` struct

```rust
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
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::IdempotencyVerifier;
    ///
    /// let verifier = IdempotencyVerifier::new();
    /// let result = verifier.verify("echo 'hello world'");
    ///
    /// assert!(result.is_idempotent);
    /// assert_eq!(result.runs.len(), 3);
    /// ```
    pub fn verify(&self, _script: &str) -> IdempotencyResult {
        unimplemented!("REPL-012-003: verify not yet implemented")
    }
}

impl Default for IdempotencyVerifier {
    fn default() -> Self {
        Self::new()
    }
}
```

### 2. Add `IdempotencyResult` struct

```rust
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
```

### 3. Unit Tests (RED → GREEN → REFACTOR)

```rust
#[cfg(test)]
mod idempotency_verification_tests {
    use super::*;

    // ===== REPL-012-003: IDEMPOTENCY VERIFICATION TESTS =====

    #[test]
    fn test_REPL_012_003_verifier_new_default() {
        // ARRANGE & ACT: Create default verifier
        let verifier = IdempotencyVerifier::new();

        // ASSERT: Should have 3 runs by default
        assert_eq!(verifier.run_count, 3);
    }

    #[test]
    fn test_REPL_012_003_verifier_custom_count() {
        // ARRANGE & ACT: Create verifier with custom count
        let verifier = IdempotencyVerifier::with_run_count(5);

        // ASSERT: Should have 5 runs
        assert_eq!(verifier.run_count, 5);
    }

    #[test]
    fn test_REPL_012_003_verifier_minimum_two_runs() {
        // ARRANGE & ACT: Try to create verifier with 1 run
        let verifier = IdempotencyVerifier::with_run_count(1);

        // ASSERT: Should enforce minimum of 2 runs
        assert!(verifier.run_count >= 2);
    }

    #[test]
    fn test_REPL_012_003_idempotent_script_passes() {
        // ARRANGE: Idempotent script (echo with constant)
        let verifier = IdempotencyVerifier::new();
        let script = "echo 'hello world'";

        // ACT: Verify idempotency
        let result = verifier.verify(script);

        // ASSERT: Should pass as idempotent
        assert!(result.is_idempotent);
        assert_eq!(result.run_count, 3);
        assert!(result.differences.is_empty());
    }

    #[test]
    fn test_REPL_012_003_nonidempotent_script_fails() {
        // ARRANGE: Non-idempotent script (random)
        let verifier = IdempotencyVerifier::new();
        let script = "echo $RANDOM";

        // ACT: Verify idempotency
        let result = verifier.verify(script);

        // ASSERT: Should fail as non-idempotent
        assert!(!result.is_idempotent);
        assert_eq!(result.run_count, 3);
        assert!(!result.differences.is_empty());
    }

    #[test]
    fn test_REPL_012_003_result_format_idempotent() {
        // ARRANGE: Idempotent result
        let result = IdempotencyResult {
            is_idempotent: true,
            run_count: 3,
            runs: vec![
                RunOutput {
                    run_number: 1,
                    stdout: "hello\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 2,
                    stdout: "hello\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 3,
                    stdout: "hello\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            ],
            differences: vec![],
        };

        // ACT: Format result
        let formatted = result.format_result();

        // ASSERT: Should show success
        assert!(formatted.contains("✓"));
        assert!(formatted.contains("idempotent"));
        assert!(formatted.contains("Verified across 3 runs"));
    }

    #[test]
    fn test_REPL_012_003_result_format_nonidempotent() {
        // ARRANGE: Non-idempotent result
        let result = IdempotencyResult {
            is_idempotent: false,
            run_count: 3,
            runs: vec![
                RunOutput {
                    run_number: 1,
                    stdout: "123\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 2,
                    stdout: "456\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
                RunOutput {
                    run_number: 3,
                    stdout: "789\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            ],
            differences: vec![OutputDifference {
                line: 1,
                run1: "123".to_string(),
                run2: "456".to_string(),
            }],
        };

        // ACT: Format result
        let formatted = result.format_result();

        // ASSERT: Should show failure
        assert!(formatted.contains("❌"));
        assert!(formatted.contains("non-idempotent"));
        assert!(formatted.contains("Tested 3 runs"));
    }
}
```

### 4. Property Tests

```rust
#[cfg(test)]
mod idempotency_verification_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_012_003_verifier_enforces_minimum(
            count in 0usize..10,
        ) {
            let verifier = IdempotencyVerifier::with_run_count(count);

            // Should always have at least 2 runs
            prop_assert!(verifier.run_count >= 2);
        }

        #[test]
        fn prop_REPL_012_003_constant_script_idempotent(
            constant in "[a-z]{1,20}",
        ) {
            let verifier = IdempotencyVerifier::new();
            let script = format!("echo '{}'", constant);

            let result = verifier.verify(&script);

            // Constant output should always be idempotent
            prop_assert!(
                result.is_idempotent,
                "Constant script should be idempotent: {}",
                script
            );
            prop_assert_eq!(result.differences.len(), 0);
        }

        #[test]
        fn prop_REPL_012_003_result_runs_match_count(
            run_count in 2usize..10,
        ) {
            let verifier = IdempotencyVerifier::with_run_count(run_count);
            let result = verifier.verify("echo 'test'");

            // Result should have exactly run_count runs
            prop_assert_eq!(result.runs.len(), run_count);
            prop_assert_eq!(result.run_count, run_count);
        }
    }
}
```

### 5. Quality Gates

- [ ] ✅ All unit tests pass (≥7 tests)
- [ ] ✅ All property tests pass (≥3 tests)
- [ ] ✅ Coverage >85%
- [ ] ✅ Clippy warnings: 0
- [ ] ✅ Complexity <10 per function
- [ ] ✅ Mutation score ≥90% (deferred to separate pass)

## EXTREME TDD Methodology

### RED Phase
1. Write failing test: `test_REPL_012_003_verifier_new_default`
2. Write failing test: `test_REPL_012_003_verifier_custom_count`
3. Write failing test: `test_REPL_012_003_verifier_minimum_two_runs`
4. Write failing test: `test_REPL_012_003_idempotent_script_passes`
5. Write failing test: `test_REPL_012_003_nonidempotent_script_fails`
6. Write failing test: `test_REPL_012_003_result_format_idempotent`
7. Write failing test: `test_REPL_012_003_result_format_nonidempotent`
8. Run tests → **FAIL** ✅ (expected)

### GREEN Phase
1. Implement `IdempotencyVerifier` struct
2. Implement `IdempotencyResult` struct
3. Implement `verify()` method (using shell execution)
4. Implement `IdempotencyResult::format_result()` method
5. Run tests → **PASS** ✅

### REFACTOR Phase
1. Extract script execution helpers if needed
2. Ensure error handling is robust
3. Keep complexity <10
4. Run tests → **PASS** ✅

### PROPERTY Phase
1. Add property test: `prop_REPL_012_003_verifier_enforces_minimum`
2. Add property test: `prop_REPL_012_003_constant_script_idempotent`
3. Add property test: `prop_REPL_012_003_result_runs_match_count`
4. Run property tests (100+ cases) → **PASS** ✅

### MUTATION Phase (Deferred)
1. Run `cargo mutants --file rash/src/repl/determinism.rs`
2. Target: ≥90% kill rate
3. Address any MISSED mutants

## Implementation Plan

### Files to Modify
- `rash/src/repl/determinism.rs` - Add `IdempotencyVerifier` and `IdempotencyResult`
- `rash/src/repl/mod.rs` - Export `IdempotencyVerifier` and `IdempotencyResult`

### Files to Create
- None (extends existing determinism module)

### Test Files
- `rash/src/repl/determinism.rs` - Unit tests in module
- `rash/src/repl/determinism.rs` - Property tests in module

## Task Breakdown

- [ ] **Task 1**: Write RED tests for verification
- [ ] **Task 2**: Implement `IdempotencyVerifier` struct (GREEN phase)
- [ ] **Task 3**: Implement `verify()` method with shell execution (GREEN phase)
- [ ] **Task 4**: Implement `IdempotencyResult::format_result()` (GREEN phase)
- [ ] **Task 5**: Refactor if needed (REFACTOR phase)
- [ ] **Task 6**: Add property tests (PROPERTY phase)
- [ ] **Task 7**: Verify all quality gates
- [ ] **Task 8**: Update roadmap (mark REPL-012-003 complete)
- [ ] **Task 9**: Commit with proper message

## Example Usage

```bash
$ bashrs repl
bashrs> :debug examples/deploy.sh
bashrs> :verify-idempotency

Running idempotency verification (3 runs)...

Run 1: ✓ Completed (exit code: 0)
Run 2: ✓ Completed (exit code: 0)
Run 3: ✓ Completed (exit code: 0)

✓ Script is idempotent
  Verified across 3 runs

Runs: 3
Exit codes: 0 0 0

All runs produced identical output.
```

## Example: Non-Idempotent Script

```bash
$ bashrs repl
bashrs> :debug examples/random.sh
bashrs> :verify-idempotency

Running idempotency verification (3 runs)...

Run 1: ✓ Completed (exit code: 0)
Run 2: ✓ Completed (exit code: 0)
Run 3: ✓ Completed (exit code: 0)

❌ Script is non-idempotent
  Tested 3 runs

Runs: 3
Exit codes: 0 0 0

❌ Non-deterministic output detected!

Found 1 difference(s):

Line 1:
  Run 1: Random: 12345
  Run 2: Random: 67890
```

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- EXTREME TDD ensures verification logic is correct from first line
- Actual script execution validates real-world behavior

### 反省 (Hansei) - Reflect and Improve
- Learn from existing `ReplayVerifier` (REPL-011-002)
- Apply same execution patterns to idempotency verification

### 改善 (Kaizen) - Continuous Improvement
- Runtime verification catches issues static analysis misses
- Multiple runs increase confidence in idempotency

### 現地現物 (Genchi Genbutsu) - Go and See
- Actually run scripts to verify behavior
- Don't just analyze - execute and observe

## Related Files
- `rash/src/repl/determinism.rs` - ReplayVerifier (REPL-011-002, reference pattern)
- `rash/src/repl/determinism.rs` - IdempotencyChecker (REPL-012-001)
- `docs/REPL-DEBUGGER-ROADMAP.yaml` - Roadmap reference

## Success Criteria Summary
```
BEFORE: Static analysis only (might miss runtime issues)
AFTER:  ✅ Runtime verification through actual execution
        ✅ Multiple runs (default 3, configurable)
        ✅ Result comparison and difference detection
        ✅ Clear pass/fail indication
        ✅ Formatted results with IdempotencyResult::format_result()
        ✅ All quality gates passed
        ✅ Property tests validate verification logic
```

---

**Created**: 2025-10-30
**Sprint**: REPL-012 (Idempotency Analyzer)
**Estimated Time**: 2-3 hours (includes shell execution implementation)
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY → MUTATION)

---
