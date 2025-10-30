# TICKET: REPL-010-003

## Title
Explain Transformations at Current Line

## Priority
**P1 - High** (Next in roadmap sequence after REPL-010-002)

## Status
🟢 **READY TO START** - Dependencies met (REPL-010-001, REPL-010-002 completed)

## Context
The REPL debugger currently shows enhanced diffs with explanations:
```
- mkdir /tmp/foo
+ mkdir -p /tmp/foo
(added idempotency flag -p to mkdir)
```

This task adds a focused explanation method that returns **only the explanations** without the diff markers, useful for:
- Status messages in the REPL
- Inline hints during debugging
- Quick transformation summaries

**Purpose**: Provide concise, focused explanations of what purification will do at the current line.

## Dependencies
- ✅ REPL-010-001: Compare original vs purified (completed)
- ✅ REPL-010-002: Enhanced diff highlighting (completed)
- ✅ `detect_transformations()` helper exists
- ✅ LineComparison struct exists

## Acceptance Criteria

### 1. Add `explain_current_line()` method

```rust
/// Explain transformations that will be applied at the current line
/// Returns: Human-readable explanation, or None if no transformations
pub fn explain_current_line(&self) -> Option<String> {
    let comparison = self.compare_current_line()?;

    if !comparison.differs {
        return None; // No transformations
    }

    let explanations = Self::detect_transformations(
        &comparison.original,
        &comparison.purified
    );

    if explanations.is_empty() {
        // Lines differ but no specific transformations detected
        Some("Script will be transformed".to_string())
    } else {
        // Join explanations into readable sentence
        Some(explanations.join(", "))
    }
}
```

### 2. Unit Tests (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_REPL_010_003_explain_mkdir_p() {
    // ARRANGE: Script with non-idempotent mkdir
    let script = "mkdir /tmp/foo";
    let session = DebugSession::new(script);

    // ACT: Get explanation
    let explanation = session.explain_current_line();

    // ASSERT: Should explain idempotency
    assert!(explanation.is_some(), "Should have explanation");
    let text = explanation.unwrap();
    assert!(text.contains("idempot") || text.contains("idem"));
    assert!(text.contains("-p") || text.contains("mkdir"));
}

#[test]
fn test_REPL_010_003_explain_quote() {
    // ARRANGE: Script with unquoted variable
    let script = "echo $USER";
    let session = DebugSession::new(script);

    // ACT: Get explanation
    let explanation = session.explain_current_line();

    // ASSERT: Should explain quoting (if purifier transforms this)
    if let Some(text) = explanation {
        assert!(text.contains("quot") || text.contains("safe"));
    }
}

#[test]
fn test_REPL_010_003_explain_ln_sf() {
    // ARRANGE: Script with ln -s
    let script = "ln -s /tmp/src /tmp/link";
    let session = DebugSession::new(script);

    // ACT: Get explanation
    let explanation = session.explain_current_line();

    // ASSERT: Should explain -f addition (if transformed)
    if let Some(text) = explanation {
        assert!(
            text.contains("-f") || text.contains("idempot") || text.contains("idem"),
            "Explanation should mention -f or idempotency: {}", text
        );
    }
}

#[test]
fn test_REPL_010_003_explain_no_change() {
    // ARRANGE: Script that's already purified
    let script = "mkdir -p /tmp/foo";
    let session = DebugSession::new(script);

    // ACT: Get explanation
    let explanation = session.explain_current_line();

    // ASSERT: Should return None (no transformations)
    assert!(explanation.is_none(), "Should have no explanation when unchanged");
}

#[test]
fn test_REPL_010_003_explain_multiple_changes() {
    // ARRANGE: Script with multiple transformations
    let script = "rm $FILE";
    let session = DebugSession::new(script);

    // ACT: Get explanation
    let explanation = session.explain_current_line();

    // ASSERT: Should explain multiple transformations
    if let Some(text) = explanation {
        // Should mention either quoting or -f flag
        assert!(
            text.contains("quot") || text.contains("-f"),
            "Should explain at least one transformation: {}", text
        );
    }
}
```

### 3. Property Tests

```rust
proptest! {
    #[test]
    fn prop_REPL_010_003_explain_none_when_identical(
        cmd in "mkdir -p|rm -f|ln -sf",
        arg in "[a-z/$]{1,20}"
    ) {
        // Already-purified commands should have no explanation
        let script = format!("{} {}", cmd, arg);
        let session = DebugSession::new(&script);

        if let Some(comparison) = session.compare_current_line() {
            if !comparison.differs {
                let explanation = session.explain_current_line();
                prop_assert!(
                    explanation.is_none(),
                    "Should have no explanation when lines identical"
                );
            }
        }
    }

    #[test]
    fn prop_REPL_010_003_explain_some_when_differs(
        cmd in "mkdir|ln -s"
    ) {
        // Non-idempotent commands should have explanation
        let script = format!("{} /tmp/test", cmd);
        let session = DebugSession::new(&script);

        if let Some(comparison) = session.compare_current_line() {
            if comparison.differs {
                let explanation = session.explain_current_line();
                prop_assert!(
                    explanation.is_some(),
                    "Should have explanation when lines differ"
                );
            }
        }
    }

    #[test]
    fn prop_REPL_010_003_explain_never_panics(
        cmd in "[a-z]{1,10}",
        arg in "[a-z/$]{1,20}"
    ) {
        let script = format!("{} {}", cmd, arg);
        let session = DebugSession::new(&script);

        // Should never panic
        let _ = session.explain_current_line();
    }
}
```

### 4. Quality Gates

- [ ] ✅ All unit tests pass (≥5 tests)
- [ ] ✅ All property tests pass (≥3 tests)
- [ ] ✅ Coverage >85%
- [ ] ✅ Clippy warnings: 0
- [ ] ✅ Complexity <10 per function
- [ ] ✅ Integration test with CLI
- [ ] ✅ Mutation score ≥90% (deferred to separate pass)

## EXTREME TDD Methodology

### RED Phase
1. Write failing test: `test_REPL_010_003_explain_mkdir_p`
2. Write failing test: `test_REPL_010_003_explain_quote`
3. Write failing test: `test_REPL_010_003_explain_ln_sf`
4. Write failing test: `test_REPL_010_003_explain_no_change`
5. Write failing test: `test_REPL_010_003_explain_multiple_changes`
6. Run tests → **FAIL** ✅ (expected)

### GREEN Phase
1. Implement `explain_current_line()` method
2. Use existing `detect_transformations()` helper
3. Return `None` when no transformations
4. Return joined explanations when transformations exist
5. Run tests → **PASS** ✅

### REFACTOR Phase
1. Extract explanation formatting if needed
2. Ensure code reuses existing helpers
3. Keep complexity <10
4. Run tests → **PASS** ✅

### PROPERTY Phase
1. Add property test: explanations match transformation presence
2. Add property test: no panic on any input
3. Run property tests (100+ cases) → **PASS** ✅

### MUTATION Phase (Deferred)
1. Run `cargo mutants --file rash/src/repl/debugger.rs`
2. Target: ≥90% kill rate
3. Address any MISSED mutants

## Implementation Plan

### Files to Modify
- `rash/src/repl/debugger.rs` - Add `explain_current_line()` method (after line 480)
- `rash/src/repl/debugger.rs` - Add unit tests in test module
- `rash/src/repl/debugger.rs` - Add property tests in proptest module

### Files to Create
- None (all in existing modules)

### Test Files
- `rash/src/repl/debugger.rs` - Unit tests in module
- `rash/tests/cli_repl_tests.rs` - Integration tests (optional)

## Task Breakdown

- [ ] **Task 1**: Write RED tests for explain_current_line()
- [ ] **Task 2**: Implement explain_current_line() method (GREEN phase)
- [ ] **Task 3**: Refactor if needed (REFACTOR phase)
- [ ] **Task 4**: Add property tests (PROPERTY phase)
- [ ] **Task 5**: Verify all quality gates
- [ ] **Task 6**: Update roadmap (mark REPL-010-003 complete)
- [ ] **Task 7**: Commit with proper message

## Example Usage

```bash
$ bashrs repl
bashrs> :debug examples/deploy.sh
bashrs> :break 5
bashrs> :step

Stopped at line 5: mkdir /app/releases
Transformations: added idempotency flag -p to mkdir

bashrs> :explain
added idempotency flag -p to mkdir

bashrs> :step

Stopped at line 6: rm /app/current
Transformations: added idempotency flag -f to rm

bashrs> :step

Stopped at line 7: ln -s /app/releases/v1 /app/current
Transformations: added idempotency flag -f to ln
```

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- EXTREME TDD ensures explanations are correct from first line
- Property tests catch edge cases in explanation logic

### 反省 (Hansei) - Reflect and Improve
- Learn from REPL-010-002 implementation
- Reuse existing helpers (detect_transformations)

### 改善 (Kaizen) - Continuous Improvement
- Make debugging insights clearer with focused explanations
- Improve developer understanding of purification

### 現地現物 (Genchi Genbutsu) - Go and See
- Test with real bash scripts
- Validate explanations make sense to developers

## Related Files
- `rash/src/repl/debugger.rs` - DebugSession and LineComparison
- `rash/src/repl/purifier.rs` - Purification logic (reference)
- `docs/REPL-DEBUGGER-ROADMAP.yaml` - Roadmap reference
- `TICKET-REPL-010-002-HIGHLIGHT.md` - Previous ticket (diff highlighting)

## Differences from REPL-010-002

| Feature | REPL-010-002 | REPL-010-003 |
|---------|--------------|--------------|
| Method | `format_diff_highlighting()` | `explain_current_line()` |
| Output | Full diff with markers | Explanation only |
| Format | `- original\n+ purified\n(explanation)` | `explanation` |
| Use case | Visual comparison | Quick status message |
| Return type | `String` (always) | `Option<String>` (None if no change) |

## Success Criteria Summary
```
BEFORE: Need full diff to see what changed
AFTER:  ✅ Concise explanation available via explain_current_line()
        ✅ Returns None when no transformations
        ✅ Reuses existing transformation detection
        ✅ All quality gates passed
        ✅ Property tests validate behavior
```

---

**Created**: 2025-10-30
**Sprint**: REPL-010 (Purification-Aware Debugging)
**Estimated Time**: 1-2 hours
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY → MUTATION)

---
