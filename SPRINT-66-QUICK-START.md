# Sprint 66 Quick Start Card - High-Risk Functions (FOREACH, CALL) 🚀

## TL;DR - Start Here

```bash
# 1. Verify current state
cargo test --lib  # Should show: test result: ok. 1370 passed

# 2. Read context (optional but recommended)
cat SPRINT-65-HANDOFF.md  # Previous sprint discovery
cat PROJECT-STATE-2025-10-18-SPRINT-65.md  # Current project state

# 3. Start EXTREME TDD - AUDIT phase first
# Search for existing foreach/call handling
grep -r "foreach" rash/src/make_parser/
grep -r "call" rash/src/make_parser/

# 4. Write RED tests (if needed)
# Edit: rash/src/make_parser/tests.rs
# Add: test_SEMANTIC_FOREACH_001_* and test_SEMANTIC_CALL_001_*

# 5. Verify tests FAIL (RED phase) or PASS (discovery!)
cargo test --lib test_SEMANTIC_FOREACH
cargo test --lib test_SEMANTIC_CALL
```

## What You're Building

**Goal**: Complete Phase 2 by auditing/implementing semantic analysis for high-risk Make functions.

**High-Risk Functions**:
1. `$(foreach var, list, text)` - Iteration order matters, needs list source analysis
2. `$(call function, args)` - Requires function definition analysis

**Why Critical**: These are the last 2/15 Phase 2 tasks. Completing them achieves 100% Phase 2!

**Estimated Time**: 12-19 hours (may be less with audit discoveries!)

## Context from Sprint 64-65

### Sprint 64 Discovery ✅
**Goal**: Implement function call parser
**Discovery**: Parser already works! (saved 8-10 hours)
**Result**: 1,330 → 1,345 tests (+15)

### Sprint 65 Discovery ✅
**Goal**: Implement recursive semantic analysis
**Discovery**: Semantic analysis already works! (saved 4-6 hours)
**Result**: 1,345 → 1,370 tests (+25)

**Pattern**: 67% of sprints discover existing functionality!

**Recommendation**: Start Sprint 66 with systematic audit before implementation.

## AUDIT Phase - Start Here!

### Step 1: Search Existing Code

```bash
# Check if foreach is already handled
grep -rn "foreach" rash/src/make_parser/ | grep -v test | grep -v "//"

# Check if call is already handled
grep -rn "call" rash/src/make_parser/ | grep -v test | grep -v "//"

# Check AST for foreach/call nodes
grep -A5 "foreach\|call" rash/src/make_parser/ast.rs

# Check semantic analysis for foreach/call
grep -A10 "foreach\|call" rash/src/make_parser/semantic.rs
```

### Step 2: Write Verification Tests

**FOREACH Tests** (add to `rash/src/make_parser/tests.rs`):

```rust
#[test]
fn test_SEMANTIC_FOREACH_001_detect_wildcard_in_foreach_list() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(foreach) iterating over $(wildcard) - ORDER MATTERS!
    let makefile = "OBJS := $(foreach file, $(wildcard *.c), $(file:.c=.o))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard in foreach list (non-deterministic order)
    // This is CRITICAL because foreach processes items in order!
    assert!(!issues.is_empty(), "Expected to detect wildcard in foreach list");

    // May detect as NO_WILDCARD (current implementation)
    // OR may need new rule: NO_UNORDERED_FOREACH_LIST
}

#[test]
fn test_SEMANTIC_FOREACH_002_safe_foreach_with_explicit_list() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(foreach) with explicit list (SAFE)
    let makefile = "OBJS := $(foreach file, foo.c bar.c baz.c, $(file:.c=.o))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should NOT detect issues (explicit list is deterministic)
    assert_eq!(issues.len(), 0, "Expected no issues for explicit list");
}

#[test]
fn test_SEMANTIC_FOREACH_003_purified_foreach_with_sorted_wildcard() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(foreach) with $(sort $(wildcard)) - PURIFIED!
    let makefile = "OBJS := $(foreach file, $(sort $(wildcard *.c)), $(file:.c=.o))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Current implementation will still detect wildcard
    // Future enhancement: recognize $(sort $(wildcard)) as purified
    assert!(!issues.is_empty(), "Wildcard detected (even if purified)");
}
```

**CALL Tests** (add to `rash/src/make_parser/tests.rs`):

```rust
#[test]
fn test_SEMANTIC_CALL_001_detect_wildcard_in_call_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(call) with $(wildcard) in arguments
    let makefile = r#"
reverse = $(2) $(1)
FILES := $(call reverse, $(wildcard *.c), foo.c)
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard in call arguments
    assert!(!issues.is_empty(), "Expected to detect wildcard in call args");
}

#[test]
fn test_SEMANTIC_CALL_002_safe_call_with_explicit_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(call) with explicit arguments (SAFE)
    let makefile = r#"
reverse = $(2) $(1)
RESULT := $(call reverse, foo.c, bar.c)
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should NOT detect issues (explicit args are deterministic)
    assert_eq!(issues.len(), 0, "Expected no issues for explicit args");
}
```

### Step 3: Run Verification Tests

```bash
# Run foreach tests
cargo test --lib test_SEMANTIC_FOREACH 2>&1 | tail -20

# Run call tests
cargo test --lib test_SEMANTIC_CALL 2>&1 | tail -20

# If tests PASS:
#   → Sprint 66 discovery! Semantic analysis already works!
#   → Document discovery, add more verification tests
#   → Time saved: 8-12 hours

# If tests FAIL:
#   → Expected behavior (RED phase)
#   → Proceed to GREEN phase (implementation)
#   → Estimated: 8-12 hours implementation
```

## If Audit Discovers Existing Functionality

**Follow Sprint 64-65 Pattern**:

1. ✅ Document discovery in Sprint 66 handoff
2. ✅ Add comprehensive verification tests (10-15 tests)
3. ✅ Verify all patterns work correctly
4. ✅ Update project state
5. ✅ Mark Phase 2 as COMPLETE! 🎉

**Time**: 2-4 hours (verification only)

## If Implementation Needed (GREEN Phase)

### FOREACH Implementation

**Goal**: Detect non-deterministic list sources in foreach loops

**Dangerous Pattern**:
```makefile
# BAD: $(wildcard) returns files in non-deterministic order
OBJS := $(foreach file, $(wildcard *.c), $(file:.c=.o))
```

**Safe Pattern**:
```makefile
# GOOD: Explicit list is deterministic
OBJS := $(foreach file, foo.c bar.c baz.c, $(file:.c=.o))

# GOOD: Sorted wildcard is deterministic
OBJS := $(foreach file, $(sort $(wildcard *.c)), $(file:.c=.o))
```

**Implementation Approach**:

Option 1: Rely on existing wildcard detection (SIMPLE):
```rust
// Current analyze_makefile() will already detect wildcard in:
// "$(foreach file, $(wildcard *.c), ...)"
// Because detect_wildcard() uses .contains("$(wildcard")
//
// NO IMPLEMENTATION NEEDED! (Sprint 65 discovery applies)
```

Option 2: Add foreach-specific detection (if Option 1 insufficient):
```rust
pub fn detect_foreach(value: &str) -> bool {
    value.contains("$(foreach")
}

pub fn detect_unordered_foreach_list(value: &str) -> bool {
    // Check if foreach list contains wildcard, shell find, etc.
    detect_foreach(value) && (
        detect_wildcard(value) ||
        detect_shell_find(value) ||
        detect_random(value)
    )
}
```

### CALL Implementation

**Goal**: Detect non-deterministic arguments to $(call) function

**Dangerous Pattern**:
```makefile
# BAD: Wildcard in call arguments
process = @echo Processing $(1)
FILES := $(call process, $(wildcard *.c))
```

**Safe Pattern**:
```makefile
# GOOD: Explicit arguments
process = @echo Processing $(1)
FILES := $(call process, foo.c bar.c)
```

**Implementation Approach**:

Option 1: Rely on existing detection (SIMPLE):
```rust
// Current analyze_makefile() will already detect wildcard in:
// "$(call process, $(wildcard *.c))"
// Because detect_wildcard() uses .contains("$(wildcard")
//
// NO IMPLEMENTATION NEEDED! (Sprint 65 discovery applies)
```

Option 2: Add call-specific detection (if Option 1 insufficient):
```rust
pub fn detect_call(value: &str) -> bool {
    value.contains("$(call")
}

pub fn detect_call_with_non_deterministic_args(value: &str) -> bool {
    // Check if call arguments contain wildcard, shell, etc.
    detect_call(value) && (
        detect_wildcard(value) ||
        detect_shell_date(value) ||
        detect_random(value) ||
        detect_shell_find(value)
    )
}
```

## REFACTOR Checklist

After tests pass (GREEN):
- [ ] Extract helper functions if needed
- [ ] Ensure cyclomatic complexity < 10
- [ ] Add comments for complex logic
- [ ] Clean up code structure

## PROPERTY Phase

Add property tests:

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_foreach_always_detects_wildcard(
            var in "[a-z]+",
            pattern in "[*.a-z]+"
        ) {
            let makefile = format!(
                "X := $(foreach {}, $(wildcard {}), $({}))",
                var, pattern, var
            );
            let ast = parse_makefile(&makefile).unwrap();
            let issues = analyze_makefile(&ast);
            prop_assert!(!issues.is_empty());
        }
    }
}
```

## MUTATION Phase

```bash
cargo mutants --file rash/src/make_parser/semantic.rs -- --lib

# Target: ≥90% kill rate
# If below 90%, add more tests to kill surviving mutants
```

## Success Criteria

- [ ] ✅ Audit complete for foreach/call
- [ ] ✅ Verification tests added (10-15 tests)
- [ ] ✅ All tests passing (1,370 → 1,385+ tests)
- [ ] ✅ Zero regressions
- [ ] ✅ Property tests passing (100+ generated cases)
- [ ] ✅ Mutation score ≥90% (if new code added)
- [ ] ✅ Sprint 66 handoff created
- [ ] ✅ **Phase 2 COMPLETE!** (15/15 tasks) 🎉

## Expected Outcomes

### Scenario A: Discovery (Most Likely Given Sprint 64-65)
- Existing detection already works for foreach/call
- Add 10-15 verification tests
- Document discovery
- **Time**: 2-4 hours
- **Phase 2**: COMPLETE! ✅

### Scenario B: Partial Implementation Needed
- Some foreach/call detection exists
- Add missing pieces
- **Time**: 4-8 hours
- **Phase 2**: COMPLETE! ✅

### Scenario C: Full Implementation Needed (Least Likely)
- No foreach/call detection
- Implement from scratch
- **Time**: 8-12 hours
- **Phase 2**: COMPLETE! ✅

## Files to Reference

- `SPRINT-65-HANDOFF.md` - Previous sprint (recursive detection discovery)
- `PROJECT-STATE-2025-10-18-SPRINT-65.md` - Current project state
- `rash/src/make_parser/semantic.rs` - Semantic analysis implementation
- `rash/src/make_parser/tests.rs` - Test suite (1,370 tests)
- `CLAUDE.md` - EXTREME TDD workflow reference

## Quick Commands Reference

```bash
# Search for existing handling
grep -rn "foreach" rash/src/make_parser/ | head -20
grep -rn "call" rash/src/make_parser/ | head -20

# Run specific tests
cargo test --lib test_SEMANTIC_FOREACH_001

# Run all semantic tests
cargo test --lib semantic:: --lib

# Run all tests
cargo test --lib

# Check test count
cargo test --lib 2>&1 | grep "test result"

# Mutation testing
cargo mutants --file rash/src/make_parser/semantic.rs -- --lib
```

## Expected Timeline

### If Discovery (Scenario A - Most Likely)
- **Hour 1**: Audit existing code, write verification tests
- **Hour 2**: Run tests, document discovery
- **Hour 3**: Add comprehensive test coverage
- **Hour 4**: Create Sprint 66 handoff, celebrate Phase 2 completion! 🎉

### If Implementation (Scenario C - Least Likely)
- **Hour 1-2**: RED phase - Write 10-15 failing tests
- **Hour 3-6**: GREEN phase - Implement foreach/call detection
- **Hour 7**: REFACTOR phase - Clean up, complexity < 10
- **Hour 8**: PROPERTY phase - Add property tests
- **Hour 9**: MUTATION phase - Run mutation testing
- **Hour 10**: DOCUMENTATION - Create handoff, celebrate! 🎉

## After Sprint 66

Once Phase 2 is complete (15/15 tasks):

**Next Priorities**:
1. Purification Engine (Sprint 67) - Auto-fix detected issues
2. CLI Integration (Sprint 68) - `rash lint` command
3. Phase 3 Planning - Define next 50-100 tasks

**Total Phase 2 Time** (actual):
- Sprint 61-62: 8-10 hours (planning + discovery)
- Sprint 63: 2 hours (planning)
- Sprint 64: 2 hours (parser verification)
- Sprint 65: 2 hours (semantic verification)
- Sprint 66: 2-12 hours (foreach/call)
- **Total**: 16-28 hours (vs 36-45 hour estimate!)

---

**Ready to Start?** Run audit commands above to check existing foreach/call handling, then dive into verification tests! 🚀

**Remember**: Sprint 64 and 65 both discovered existing functionality. Sprint 66 may too! Start with systematic audit before assuming implementation is needed.
