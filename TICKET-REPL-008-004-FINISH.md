# TICKET: REPL-008-004

## Title
Finish (Exit Current Function)

## Priority
**P1 - High** (Completes REPL-008 Execution Control sprint)

## Status
🟢 **READY TO START** - Dependencies met (REPL-008-001 Step completed)

## Context
The REPL debugger currently supports:
- **step**: Execute next line (any context)
- **next**: Execute next line (skip into functions)
- **continue**: Run until breakpoint

This task adds **finish** command to exit the current function and return to the caller.

**Purpose**: Complete the standard debugger execution control set, allowing developers to quickly exit function bodies.

## Dependencies
- ✅ REPL-008-001: Step execution (completed)
- ✅ Call stack tracking exists in DebugSession
- ✅ StackFrame struct defined

## Acceptance Criteria

### 1. Add `finish()` method to DebugSession

```rust
/// Execute until the current function returns
///
/// Continues execution until we exit the current stack frame.
/// If already at the top level (main), this behaves like continue to end.
///
/// # Returns
///
/// - `ContinueResult::BreakpointHit(line)` if stopped at breakpoint
/// - `ContinueResult::Finished` if execution completed
pub fn finish(&mut self) -> ContinueResult {
    // Get current call stack depth
    let current_depth = self.call_stack.len();

    // If at top level (<main>), just continue to end
    if current_depth <= 1 {
        return self.continue_execution();
    }

    // Execute until we return to shallower depth
    loop {
        // Check if we hit a breakpoint
        if self.breakpoints.should_break(self.current_line, &self.variables) {
            return ContinueResult::BreakpointHit(self.current_line);
        }

        // Check if we returned from function
        if self.call_stack.len() < current_depth {
            return ContinueResult::Finished;
        }

        // Check if execution finished
        if self.is_finished() {
            return ContinueResult::Finished;
        }

        // Step to next line
        self.step();
    }
}
```

### 2. Unit Tests (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_REPL_008_004_finish_returns_from_function() {
    // ARRANGE: Script with function call
    let script = r#"
function foo() {
    echo "in foo"
    echo "still in foo"
}
echo "main"
foo
echo "after foo"
"#;
    let mut session = DebugSession::new(script);

    // Step into function
    session.step(); // main
    session.step(); // foo call
    session.step(); // echo "in foo"

    // Current depth should be 2 (<main> + foo)
    assert_eq!(session.call_depth(), 2);

    // ACT: Call finish() to exit function
    let result = session.finish();

    // ASSERT: Should have returned from function
    assert!(matches!(result, ContinueResult::Finished));
    assert_eq!(session.call_depth(), 1); // Back to <main>
}

#[test]
fn test_REPL_008_004_finish_at_top_level() {
    // ARRANGE: Script with no functions
    let script = "echo line1\necho line2\necho line3";
    let mut session = DebugSession::new(script);

    // At line 1, depth 1 (<main> only)
    assert_eq!(session.call_depth(), 1);

    // ACT: Call finish() at top level
    let result = session.finish();

    // ASSERT: Should continue to end (behaves like continue)
    assert!(matches!(result, ContinueResult::Finished));
    assert!(session.is_finished());
}

#[test]
fn test_REPL_008_004_finish_stops_at_breakpoint() {
    // ARRANGE: Script with function and breakpoint after return
    let script = r#"
function foo() {
    echo "in foo"
}
foo
echo "after foo"
"#;
    let mut session = DebugSession::new(script);

    // Set breakpoint on line 5 (echo "after foo")
    session.set_breakpoint(5);

    // Step into function
    session.step(); // foo call
    session.step(); // echo "in foo"

    // ACT: Call finish() - should stop at breakpoint
    let result = session.finish();

    // ASSERT: Should hit breakpoint
    assert!(matches!(result, ContinueResult::BreakpointHit(5)));
    assert!(!session.is_finished());
}

#[test]
fn test_REPL_008_004_finish_nested_functions() {
    // ARRANGE: Script with nested function calls
    let script = r#"
function inner() {
    echo "inner"
}
function outer() {
    echo "outer start"
    inner
    echo "outer end"
}
outer
echo "main"
"#;
    let mut session = DebugSession::new(script);

    // Step into outer, then into inner
    session.step(); // outer call
    session.step(); // echo "outer start"
    session.step(); // inner call
    session.step(); // echo "inner"

    // Should be at depth 3 (<main> + outer + inner)
    assert_eq!(session.call_depth(), 3);

    // ACT: finish() should return from inner to outer
    let result = session.finish();

    // ASSERT: Should be back in outer function
    assert!(matches!(result, ContinueResult::Finished));
    assert_eq!(session.call_depth(), 2); // <main> + outer
}

#[test]
fn test_REPL_008_004_finish_when_already_finished() {
    // ARRANGE: Script at end
    let script = "echo done";
    let mut session = DebugSession::new(script);

    // Step to end
    session.step();
    session.step();
    assert!(session.is_finished());

    // ACT: Call finish() when already finished
    let result = session.finish();

    // ASSERT: Should remain finished
    assert!(matches!(result, ContinueResult::Finished));
    assert!(session.is_finished());
}
```

### 3. Property Tests

```rust
proptest! {
    #[test]
    fn prop_REPL_008_004_finish_never_increases_depth(num_lines in 1usize..10) {
        let script = (0..num_lines)
            .map(|i| format!("echo line{}", i))
            .collect::<Vec<_>>()
            .join("\n");

        let mut session = DebugSession::new(&script);
        let initial_depth = session.call_depth();

        // finish() should never increase call depth
        let _ = session.finish();
        let final_depth = session.call_depth();

        prop_assert!(final_depth <= initial_depth);
    }

    #[test]
    fn prop_REPL_008_004_finish_always_finishes_or_hits_breakpoint(
        num_lines in 1usize..10
    ) {
        let script = (0..num_lines)
            .map(|i| format!("echo line{}", i))
            .collect::<Vec<_>>()
            .join("\n");

        let mut session = DebugSession::new(&script);
        let result = session.finish();

        // Result must be one of these two variants
        prop_assert!(
            matches!(result, ContinueResult::Finished) ||
            matches!(result, ContinueResult::BreakpointHit(_))
        );
    }

    #[test]
    fn prop_REPL_008_004_finish_deterministic(num_lines in 1usize..10) {
        let script = (0..num_lines)
            .map(|i| format!("echo line{}", i))
            .collect::<Vec<_>>()
            .join("\n");

        // Run finish twice on identical sessions
        let mut session1 = DebugSession::new(&script);
        let mut session2 = DebugSession::new(&script);

        let result1 = session1.finish();
        let result2 = session2.finish();

        // Should produce identical results
        prop_assert_eq!(result1, result2);
        prop_assert_eq!(session1.is_finished(), session2.is_finished());
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
1. Write failing test: `test_REPL_008_004_finish_returns_from_function`
2. Write failing test: `test_REPL_008_004_finish_at_top_level`
3. Write failing test: `test_REPL_008_004_finish_stops_at_breakpoint`
4. Write failing test: `test_REPL_008_004_finish_nested_functions`
5. Write failing test: `test_REPL_008_004_finish_when_already_finished`
6. Run tests → **FAIL** ✅ (expected)

### GREEN Phase
1. Implement `finish()` method in DebugSession
2. Track initial call depth
3. Loop until depth decreases or execution finishes
4. Check for breakpoints during execution
5. Run tests → **PASS** ✅

### REFACTOR Phase
1. Extract call depth checking logic if needed
2. Ensure code reuses existing `step()` and `continue_execution()`
3. Keep complexity <10
4. Run tests → **PASS** ✅

### PROPERTY Phase
1. Add property tests for depth invariants
2. Add property tests for result validity
3. Add property tests for determinism
4. Run property tests (100+ cases) → **PASS** ✅

### MUTATION Phase (Deferred)
1. Run `cargo mutants --file rash/src/repl/debugger.rs`
2. Target: ≥90% kill rate
3. Address any MISSED mutants

## Implementation Plan

### Files to Modify
- `rash/src/repl/debugger.rs` - Add `finish()` method (after `continue_execution`)
- `rash/src/repl/debugger.rs` - Add unit tests in test module
- `rash/src/repl/debugger.rs` - Add property tests in proptest module

### Files to Create
- None (all in existing modules)

### Test Files
- `rash/src/repl/debugger.rs` - Unit tests in module
- `rash/tests/cli_repl_tests.rs` - Integration tests (optional)

## Task Breakdown

- [ ] **Task 1**: Write RED tests for finish() method
- [ ] **Task 2**: Implement finish() method (GREEN phase)
- [ ] **Task 3**: Refactor if needed (REFACTOR phase)
- [ ] **Task 4**: Add property tests (PROPERTY phase)
- [ ] **Task 5**: Verify all quality gates
- [ ] **Task 6**: Update roadmap (mark REPL-008-004 complete)
- [ ] **Task 7**: Commit with proper message

## Example Usage

```bash
$ bashrs repl
bashrs> :debug examples/deploy.sh
bashrs> :break 15
bashrs> :run

Stopped at breakpoint line 15 (inside deploy_release function)

bashrs> :finish
Finished deploy_release function, returned to main at line 42

bashrs> :step
Executing line 43: echo "Deployment complete"
```

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- EXTREME TDD ensures finish() works correctly from first line
- Property tests catch edge cases in call depth tracking

### 反省 (Hansei) - Reflect and Improve
- Learn from step(), next(), continue() implementations
- Reuse existing execution control patterns

### 改善 (Kaizen) - Continuous Improvement
- Complete the standard debugger command set
- Improve developer debugging experience

### 現地現物 (Genchi Genbutsu) - Go and See
- Test with real bash scripts with function calls
- Validate finish() behaves like standard debuggers

## Related Files
- `rash/src/repl/debugger.rs` - DebugSession and execution control
- `rash/src/repl/breakpoint.rs` - Breakpoint management (reference)
- `docs/REPL-DEBUGGER-ROADMAP.yaml` - Roadmap reference
- `TICKET-REPL-008-001-STEP.md` - Step implementation (reference)

## Differences from Other Commands

| Feature | step() | next() | continue() | finish() |
|---------|--------|--------|------------|----------|
| Execution | Single line | Single line | Until breakpoint | Until function returns |
| Respects breakpoints | No | No | Yes | Yes |
| Call depth change | Any | Stay same level | Any | Decrease only |
| Use case | Fine-grained | Skip functions | Fast execution | Exit function |

## Success Criteria Summary
```
BEFORE: No way to quickly exit from deep function call
AFTER:  ✅ finish() command exits current function
        ✅ Respects breakpoints during execution
        ✅ Handles nested functions correctly
        ✅ All quality gates passed
        ✅ Property tests validate invariants
```

---

**Created**: 2025-10-30
**Sprint**: REPL-008 (Execution Control)
**Estimated Time**: 2-3 hours
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY → MUTATION)

---
