# TICKET: REPL-008-002-NEXT

## Title
Implement Next Command (Skip Over Functions)

## Priority
**P1 - High** (Next in roadmap sequence after REPL-007)

## Status
🟢 **READY TO START** - Dependencies met (REPL-008-001 completed)

## Context
The REPL debugger needs a "next" command that steps to the next line at the same call depth, skipping over function calls without entering them. This is essential for efficient debugging when you don't want to step into every function.

**Difference from Step**:
- **Step** (`step`): Executes next line, entering functions
- **Next** (`next`): Executes next line, skipping over functions (staying at same depth)

## Dependencies
- ✅ REPL-008-001: Step command (completed - commit 1552582e)
- ✅ REPL-007-001: Breakpoint system (completed - commit 21ab465f)

## Acceptance Criteria

### 1. Implement `next()` method in DebugSession

```rust
/// Execute until next line at same call depth
/// Returns: (line_number, line_content, finished)
pub fn next(&mut self) -> (usize, String, bool) {
    // Save current call depth
    // Step forward until:
    //   - Same or lower depth reached
    //   - OR end of execution
}
```

### 2. Unit Tests (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_REPL_008_002_next_same_level() {
    // ARRANGE: Script with simple statements
    let script = "echo line1\necho line2\necho line3\n";
    let mut session = DebugSession::from_script(script);

    // ACT: next should go to line 2
    let (line, _, finished) = session.next();

    // ASSERT
    assert_eq!(line, 2);
    assert!(!finished);
}

#[test]
fn test_REPL_008_002_next_skips_function() {
    // ARRANGE: Script with function call
    let script = r#"
function greet() {
    echo "Hello"
    echo "World"
}

echo "Before"
greet
echo "After"
"#;
    let mut session = DebugSession::from_script(script);

    // ACT: Start at "echo Before" (line 7)
    // Move to line 7
    while session.current_line() < 7 {
        session.step();
    }

    // ACT: next should skip entire greet() function
    let (line, content, _) = session.next();

    // ASSERT: Should be at "echo After" (line 9)
    assert_eq!(line, 9);
    assert!(content.contains("After"));
}
```

### 3. Property Tests

```rust
proptest! {
    #[test]
    fn prop_next_never_goes_deeper(script in bash_script_strategy()) {
        let mut session = DebugSession::from_script(&script);
        let depth_before = session.call_depth();

        session.next();

        let depth_after = session.call_depth();
        prop_assert!(depth_after <= depth_before);
    }

    #[test]
    fn prop_next_eventually_finishes(script in simple_script_strategy()) {
        let mut session = DebugSession::from_script(&script);

        let mut iterations = 0;
        while !session.is_finished() && iterations < 1000 {
            session.next();
            iterations += 1;
        }

        prop_assert!(session.is_finished());
    }
}
```

### 4. Quality Gates

- [ ] ✅ All unit tests pass (≥6 tests)
- [ ] ✅ All property tests pass (≥2 tests)
- [ ] ✅ Coverage >85%
- [ ] ✅ Clippy warnings: 0
- [ ] ✅ Complexity <10 per function
- [ ] ✅ Integration test with CLI
- [ ] ✅ Mutation score ≥90% (deferred to separate pass)

## EXTREME TDD Methodology

### RED Phase
1. Write failing test: `test_REPL_008_002_next_same_level`
2. Write failing test: `test_REPL_008_002_next_skips_function`
3. Run tests → **FAIL** ✅ (expected)

### GREEN Phase
1. Implement `next()` method in `DebugSession`
2. Add call depth tracking if not present
3. Run tests → **PASS** ✅

### REFACTOR Phase
1. Extract helper methods if needed
2. Simplify logic
3. Ensure complexity <10
4. Run tests → **PASS** ✅

### PROPERTY Phase
1. Add property tests for depth invariant
2. Add property tests for termination
3. Run property tests (100+ cases) → **PASS** ✅

### MUTATION Phase (Deferred)
1. Run `cargo mutants --file rash/src/repl/debugger.rs`
2. Target: ≥90% kill rate
3. Address any MISSED mutants

## Implementation Plan

### Files to Modify
- `rash/src/repl/debugger.rs` - Add `next()` method
- `rash/src/repl/debugger.rs` - Add call depth tracking (if missing)

### Files to Create
- None (all in existing modules)

### Test Files
- `rash/src/repl/debugger.rs` - Unit tests in module
- `rash/tests/cli_repl_tests.rs` - Integration tests (optional)

## Task Breakdown

- [ ] **Task 1**: Write RED tests for `next()` command
- [ ] **Task 2**: Implement call depth tracking (if needed)
- [ ] **Task 3**: Implement `next()` method (GREEN phase)
- [ ] **Task 4**: Refactor and simplify (REFACTOR phase)
- [ ] **Task 5**: Add property tests (PROPERTY phase)
- [ ] **Task 6**: Verify all quality gates
- [ ] **Task 7**: Update roadmap (mark REPL-008-002 complete)
- [ ] **Task 8**: Commit with proper message

## Example Usage

```bash
$ bashrs repl
bashrs> :debug examples/script.sh
bashrs> :break 10
bashrs> :continue
Stopped at line 10: greet "Alice"
bashrs> :next
Stopped at line 11: echo "Done"
bashrs> :print result
result = "Success"
```

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- EXTREME TDD ensures quality from first line
- Property tests catch edge cases automatically

### 反省 (Hansei) - Reflect and Improve
- Each test teaches us about function call depth semantics
- Learn from step command implementation

### 改善 (Kaizen) - Continuous Improvement
- Improve debugger usability with each feature
- Make debugging experience smoother

### 現地現物 (Genchi Genbutsu) - Go and See
- Test with real bash scripts
- Validate against real debugging scenarios

## Related Files
- `rash/src/repl/debugger.rs` - DebugSession implementation
- `rash/src/repl/breakpoint.rs` - Breakpoint system
- `docs/REPL-DEBUGGER-ROADMAP.yaml` - Roadmap reference

## References
- REPL-008-001: Step command (foundation)
- REPL-007-001: Breakpoint system
- GDB documentation: `next` vs `step` semantics
- bashdb: Next command implementation

## Success Criteria Summary
```
BEFORE: Can only step through scripts line-by-line (including functions)
AFTER:  ✅ Can skip over function calls efficiently
        ✅ Next command implemented with EXTREME TDD
        ✅ All quality gates passed
        ✅ Property tests validate invariants
```

---

**Created**: 2025-10-30
**Sprint**: REPL-008 (Execution Control)
**Estimated Time**: 2-4 hours
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY → MUTATION)

---
