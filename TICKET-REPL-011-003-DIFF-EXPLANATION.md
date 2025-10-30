# TICKET: REPL-011-003

## Title
Replay Diff Explanation (Format Replay Verification Differences)

## Priority
**P1 - High** (Third task in REPL-011 Determinism Checker sprint)

## Status
🟢 **READY TO START** - Dependencies met (REPL-011-002 completed)

## Context
Building on REPL-011-002 (replay verification), this task adds **human-friendly diff explanation** for non-deterministic output.

**Concept**: When scripts produce different output across runs, format the differences in a clear, readable way.

**Why Diff Explanation?**
- REPL-011-002 detects differences but returns raw `OutputDifference` vectors
- Users need friendly formatting to understand what changed
- Good error messages help developers fix non-determinism quickly

**Purpose**: Format replay verification differences for human consumption.

## Dependencies
- ✅ REPL-011-002 (Replay verification) completed
- ✅ `OutputDifference` struct exists with line, run1, run2 fields
- ✅ `ReplayResult` available with differences vector

## Acceptance Criteria

### 1. Add `format_replay_diff()` function

```rust
/// Format replay verification differences for display
///
/// # Examples
///
/// ```
/// use bashrs::repl::determinism::OutputDifference;
/// use bashrs::repl::format_replay_diff;
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
/// assert!(formatted.contains("Run 1: Random: 12345"));
/// assert!(formatted.contains("Run 2: Random: 67890"));
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
```

### 2. Add display helper for `ReplayResult`

```rust
impl ReplayResult {
    /// Format replay result for display
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
```

### 3. Unit Tests (RED → GREEN → REFACTOR)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ===== REPL-011-003: DIFF EXPLANATION TESTS =====

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
        assert!(formatted.contains("❌"));
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
        assert!(formatted.contains("Found 2 difference(s)"));
    }

    #[test]
    fn test_REPL_011_003_format_no_differences() {
        // ARRANGE: Empty differences
        let differences = vec![];

        // ACT: Format diff
        let formatted = format_replay_diff(&differences);

        // ASSERT: Should show success message
        assert!(formatted.contains("✓"));
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
        assert!(formatted.contains("❌"));
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
        assert!(formatted.contains("✓"));
        assert!(formatted.contains("deterministic"));
        assert!(!formatted.contains("❌"));
    }
}
```

### 4. Property Tests

```rust
#[cfg(test)]
mod property_tests {
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
                formatted.contains("✓") && formatted.contains("deterministic"),
                "Empty diff should show deterministic: {}",
                formatted
            );
        }
    }
}
```

### 5. Quality Gates

- [ ] ✅ All unit tests pass (≥6 tests)
- [ ] ✅ All property tests pass (≥3 tests)
- [ ] ✅ Coverage >85%
- [ ] ✅ Clippy warnings: 0
- [ ] ✅ Complexity <10 per function
- [ ] ✅ Mutation score ≥90% (deferred to separate pass)

## EXTREME TDD Methodology

### RED Phase
1. Write failing test: `test_REPL_011_003_format_single_difference`
2. Write failing test: `test_REPL_011_003_format_multiple_differences`
3. Write failing test: `test_REPL_011_003_format_no_differences`
4. Write failing test: `test_REPL_011_003_format_empty_lines`
5. Write failing test: `test_REPL_011_003_replay_result_format`
6. Write failing test: `test_REPL_011_003_deterministic_result_format`
7. Run tests → **FAIL** ✅ (expected)

### GREEN Phase
1. Implement `format_replay_diff()` function
2. Implement `ReplayResult::format_result()` method
3. Run tests → **PASS** ✅

### REFACTOR Phase
1. Extract formatting helpers if needed
2. Ensure string building is efficient
3. Keep complexity <10
4. Run tests → **PASS** ✅

### PROPERTY Phase
1. Add property test: `prop_REPL_011_003_format_never_panics`
2. Add property test: `prop_REPL_011_003_format_preserves_line_numbers`
3. Add property test: `prop_REPL_011_003_empty_always_deterministic`
4. Run property tests (100+ cases) → **PASS** ✅

### MUTATION Phase (Deferred)
1. Run `cargo mutants --file rash/src/repl/determinism.rs`
2. Target: ≥90% kill rate
3. Address any MISSED mutants

## Implementation Plan

### Files to Modify
- `rash/src/repl/determinism.rs` - Add `format_replay_diff()` and `ReplayResult::format_result()`

### Files to Create
- None (extends existing determinism module)

### Test Files
- `rash/src/repl/determinism.rs` - Unit tests in module
- `rash/src/repl/determinism.rs` - Property tests in module

## Task Breakdown

- [ ] **Task 1**: Write RED tests for diff formatting
- [ ] **Task 2**: Implement `format_replay_diff()` (GREEN phase)
- [ ] **Task 3**: Implement `ReplayResult::format_result()` (GREEN phase)
- [ ] **Task 4**: Refactor if needed (REFACTOR phase)
- [ ] **Task 5**: Add property tests (PROPERTY phase)
- [ ] **Task 6**: Verify all quality gates
- [ ] **Task 7**: Update roadmap (mark REPL-011-003 complete)
- [ ] **Task 8**: Commit with proper message

## Example Usage

```bash
$ bashrs repl
bashrs> :debug examples/deploy.sh
bashrs> :verify-replay

Running determinism replay verification (2 runs)...

❌ Script is non-deterministic

Runs: 2
Exit codes: 0 0

❌ Non-deterministic output detected!

Found 1 difference(s):

Line 1:
  Run 1: Random: 12345
  Run 2: Random: 67890

Suggestion: Remove $RANDOM or use fixed seed
```

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- EXTREME TDD ensures diff formatting is correct from first line
- Property tests catch edge cases in formatting logic

### 反省 (Hansei) - Reflect and Improve
- Learn from existing `diff.rs` (original vs purified)
- Apply same clarity principles to replay diffs

### 改善 (Kaizen) - Continuous Improvement
- Clear error messages help developers fix issues faster
- User-friendly output improves developer experience

### 現地現物 (Genchi Genbutsu) - Go and See
- Show actual differences, not just "different"
- Help developers understand what changed

## Related Files
- `rash/src/repl/determinism.rs` - ReplayVerifier (REPL-011-002)
- `rash/src/repl/diff.rs` - Original vs purified diff (reference)
- `docs/REPL-DEBUGGER-ROADMAP.yaml` - Roadmap reference

## Success Criteria Summary
```
BEFORE: Raw OutputDifference vectors (hard to read)
AFTER:  ✅ Formatted replay diffs with line numbers
        ✅ Clear Run 1 vs Run 2 comparison
        ✅ Human-friendly status messages (✓/❌)
        ✅ ReplayResult::format_result() for complete output
        ✅ All quality gates passed
        ✅ Property tests validate formatting
```

---

**Created**: 2025-10-30
**Sprint**: REPL-011 (Determinism Checker)
**Estimated Time**: 1-2 hours
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY → MUTATION)

---
