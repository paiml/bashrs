# TICKET: REPL-010-002

## Title
Implement Enhanced Difference Highlighting

## Priority
**P1 - High** (Next in roadmap sequence after REPL-010-001)

## Status
🟢 **READY TO START** - Dependencies met (REPL-010-001 completed)

## Context
The REPL debugger currently shows basic line-by-line diffs between original and purified bash:
```
- mkdir /tmp/foo
+ mkdir -p /tmp/foo
```

This task enhances highlighting to specifically mark what changed:
```
- mkdir /tmp/foo
+ mkdir [-p] /tmp/foo  (added idempotency flag)
```

**Purpose**: Help developers understand exactly what purification transformed, making debugging insights clearer.

## Dependencies
- ✅ REPL-010-001: Compare original vs purified (completed - commit fd06dee4)
- ✅ LineComparison struct exists
- ✅ format_diff_highlighting() method exists (basic implementation)

## Acceptance Criteria

### 1. Enhance `format_diff_highlighting()` method

```rust
/// Enhanced diff highlighting with specific change markers
/// Returns: Formatted string with changes highlighted
pub fn format_diff_highlighting(&self, comparison: &LineComparison) -> String {
    if !comparison.differs {
        return format!("  {}\n(no changes)", comparison.original);
    }

    // Detect what kind of transformation occurred:
    // - Idempotency flag added (mkdir → mkdir -p)
    // - Variable quoting ($var → "$var")
    // - Safe flags (rm → rm -f, ln → ln -sf)
    // - Other transformations

    // Return format:
    // - original
    // + purified [highlighted]
    // (explanation of change)
}
```

### 2. Unit Tests (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_REPL_010_002_highlight_mkdir_p() {
    // ARRANGE: Script with non-idempotent mkdir
    let script = "mkdir /tmp/foo\n";
    let mut session = DebugSession::new(script);

    // ACT: Compare lines
    let comparison = session.compare_current_line().unwrap();
    let highlighted = session.format_diff_highlighting(&comparison);

    // ASSERT: Should highlight -p flag addition
    assert!(highlighted.contains("mkdir"));
    assert!(highlighted.contains("-p"));
    assert!(highlighted.contains("idempotent") || highlighted.contains("idem"));
}

#[test]
fn test_REPL_010_002_highlight_quote() {
    // ARRANGE: Script with unquoted variable
    let script = "echo $USER\n";
    let mut session = DebugSession::new(script);

    // ACT: Compare lines
    let comparison = session.compare_current_line().unwrap();
    let highlighted = session.format_diff_highlighting(&comparison);

    // ASSERT: Should highlight quote addition
    assert!(highlighted.contains("\""));
    assert!(highlighted.contains("quot") || highlighted.contains("safe"));
}

#[test]
fn test_REPL_010_002_highlight_rm_f() {
    // ARRANGE: Script with non-idempotent rm
    let script = "rm /tmp/file\n";
    let mut session = DebugSession::new(script);

    // ACT: Compare lines
    let comparison = session.compare_current_line().unwrap();
    let highlighted = session.format_diff_highlighting(&comparison);

    // ASSERT: Should highlight -f flag addition
    assert!(highlighted.contains("rm"));
    assert!(highlighted.contains("-f"));
}

#[test]
fn test_REPL_010_002_highlight_no_change() {
    // ARRANGE: Script that's already purified
    let script = "mkdir -p /tmp/foo\n";
    let mut session = DebugSession::new(script);

    // ACT: Compare lines
    let comparison = session.compare_current_line().unwrap();
    let highlighted = session.format_diff_highlighting(&comparison);

    // ASSERT: Should show "no changes"
    assert!(highlighted.contains("no change"));
}

#[test]
fn test_REPL_010_002_highlight_multiple_changes() {
    // ARRANGE: Script with multiple transformations
    let script = "rm $FILE\n";
    let mut session = DebugSession::new(script);

    // ACT: Compare lines
    let comparison = session.compare_current_line().unwrap();
    let highlighted = session.format_diff_highlighting(&comparison);

    // ASSERT: Should highlight both -f and quoting
    assert!(highlighted.contains("-f") || highlighted.contains("quot"));
}
```

### 3. Property Tests

```rust
proptest! {
    #[test]
    fn prop_highlight_always_valid_format(script in bash_line_strategy()) {
        let mut session = DebugSession::new(&script);
        if let Some(comparison) = session.compare_current_line() {
            let highlighted = session.format_diff_highlighting(&comparison);

            // Should always have original line
            prop_assert!(highlighted.contains(&comparison.original) ||
                         highlighted.contains("no change"));

            // Should not panic or produce empty output
            prop_assert!(!highlighted.is_empty());
        }
    }

    #[test]
    fn prop_highlight_differs_implies_explanation(script in bash_line_strategy()) {
        let mut session = DebugSession::new(&script);
        if let Some(comparison) = session.compare_current_line() {
            let highlighted = session.format_diff_highlighting(&comparison);

            if comparison.differs {
                // Should show both lines when differs
                prop_assert!(highlighted.contains('-'));
                prop_assert!(highlighted.contains('+'));
            } else {
                // Should not show diff markers when identical
                prop_assert!(highlighted.contains("no change") ||
                             !highlighted.starts_with('-'));
            }
        }
    }
}
```

### 4. Quality Gates

- [ ] ✅ All unit tests pass (≥5 tests)
- [ ] ✅ All property tests pass (≥2 tests)
- [ ] ✅ Coverage >85%
- [ ] ✅ Clippy warnings: 0
- [ ] ✅ Complexity <10 per function
- [ ] ✅ Integration test with CLI
- [ ] ✅ Mutation score ≥90% (deferred to separate pass)

## EXTREME TDD Methodology

### RED Phase
1. Write failing test: `test_REPL_010_002_highlight_mkdir_p`
2. Write failing test: `test_REPL_010_002_highlight_quote`
3. Write failing test: `test_REPL_010_002_highlight_rm_f`
4. Write failing test: `test_REPL_010_002_highlight_no_change`
5. Run tests → **FAIL** ✅ (expected)

### GREEN Phase
1. Implement detection logic in `format_diff_highlighting()`
2. Add transformation type categorization:
   - Idempotency flags (-p, -f, -sf)
   - Variable quoting
   - Other safety transformations
3. Add explanation text generation
4. Run tests → **PASS** ✅

### REFACTOR Phase
1. Extract helper methods for diff detection
2. Create `TransformationType` enum if needed
3. Simplify highlighting logic
4. Ensure complexity <10
5. Run tests → **PASS** ✅

### PROPERTY Phase
1. Add property tests for format validity
2. Add property tests for explanation presence
3. Run property tests (100+ cases) → **PASS** ✅

### MUTATION Phase (Deferred)
1. Run `cargo mutants --file rash/src/repl/debugger.rs`
2. Target: ≥90% kill rate
3. Address any MISSED mutants

## Implementation Plan

### Files to Modify
- `rash/src/repl/debugger.rs` - Enhance `format_diff_highlighting()` method
- `rash/src/repl/debugger.rs` - Add helper methods for diff analysis

### Files to Create
- None (all in existing modules)

### Test Files
- `rash/src/repl/debugger.rs` - Unit tests in module
- `rash/tests/cli_repl_tests.rs` - Integration tests (optional)

## Task Breakdown

- [ ] **Task 1**: Write RED tests for enhanced highlighting
- [ ] **Task 2**: Implement diff detection logic (GREEN phase)
- [ ] **Task 3**: Add transformation categorization
- [ ] **Task 4**: Refactor and simplify (REFACTOR phase)
- [ ] **Task 5**: Add property tests (PROPERTY phase)
- [ ] **Task 6**: Verify all quality gates
- [ ] **Task 7**: Update roadmap (mark REPL-010-002 complete)
- [ ] **Task 8**: Commit with proper message

## Example Usage

```bash
$ bashrs repl
bashrs> :debug examples/deploy.sh
bashrs> :break 5
bashrs> :step

Stopped at line 5:
- mkdir /app/releases
+ mkdir [-p] /app/releases
(added idempotency flag)

bashrs> :compare
Original: rm /app/current
Purified: rm [-f] /app/current
(added safe deletion flag)
```

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- EXTREME TDD ensures highlighting is correct from first line
- Property tests catch edge cases in diff detection

### 反省 (Hansei) - Reflect and Improve
- Learn from REPL-010-001 implementation
- Improve diff clarity based on user needs

### 改善 (Kaizen) - Continuous Improvement
- Make debugging insights clearer with each enhancement
- Improve developer understanding of purification

### 現地現物 (Genchi Genbutsu) - Go and See
- Test with real bash scripts
- Validate highlighting makes sense to developers

## Related Files
- `rash/src/repl/debugger.rs` - DebugSession and LineComparison
- `rash/src/repl/purifier.rs` - Purification logic (reference)
- `docs/REPL-DEBUGGER-ROADMAP.yaml` - Roadmap reference

## References
- REPL-010-001: Compare original vs purified (foundation)
- GNU diff: Unified diff format
- Git diff: Highlighting best practices

## Success Criteria Summary
```
BEFORE: Basic line-by-line diff (- original / + purified)
AFTER:  ✅ Specific changes highlighted with context
        ✅ Transformation explanations provided
        ✅ All quality gates passed
        ✅ Property tests validate highlighting
```

---

**Created**: 2025-10-30
**Sprint**: REPL-010 (Purification-Aware Debugging)
**Estimated Time**: 2-3 hours
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY → MUTATION)

---
