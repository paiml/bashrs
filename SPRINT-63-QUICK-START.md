# Sprint 63 Quick Start Card - Function Call Parser Implementation 🚀

## TL;DR - Start Here

```bash
# 1. Verify current state
cargo test --lib  # Should show: test result: ok. 1330 passed

# 2. Read the full plan (optional but recommended)
cat SPRINT-63-HANDOFF.md

# 3. Start EXTREME TDD - RED phase
# Edit: rash/src/make_parser/tests.rs
# Add the test below, then run:
cargo test test_PARSER_FUNC_001_basic_filter
# Should FAIL (RED) ✅

# 4. Implement parser (GREEN phase)
# Edit: rash/src/make_parser/parser.rs
# Make the test pass

# 5. Continue TDD cycle for all 15-20 tests
```

## What You're Building

**Goal**: Parse GNU Make function calls like `$(filter %.o, foo.o bar.c)` into AST nodes.

**Why Critical**: This unlocks recursive purification for all 13 deterministic functions audited in Sprints 61-62.

**Estimated Time**: 8-10 hours

## RED Phase - First Test (Copy & Paste)

Add to `rash/src/make_parser/tests.rs`:

```rust
#[test]
fn test_PARSER_FUNC_001_basic_filter() {
    let makefile = "OBJS := $(filter %.o, foo.o bar.c baz.o)";
    let ast = parse_makefile(makefile).unwrap();

    // Should parse variable with function call
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            // For now, just verify it contains the function call
            assert!(value.contains("$(filter"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}
```

Run: `cargo test test_PARSER_FUNC_001_basic_filter`
**Expected**: Test FAILS (RED) ✅

## GREEN Phase - Implementation Hints

**Files to Edit**:
1. `rash/src/make_parser/parser.rs` - Main parser logic
2. `rash/src/make_parser/ast.rs` - Already has `FunctionCall` enum! (lines 151-165)

**Key Insight**: `FunctionCall` AST structure already exists:
```rust
FunctionCall {
    name: String,        // e.g., "filter"
    args: Vec<String>,   // e.g., ["%.o", "foo.o bar.c baz.o"]
    span: Span,
}
```

**Implementation Strategy**:
1. Recognize `$(` followed by function name
2. Parse comma-separated arguments
3. Handle nested calls: `$(filter %.o, $(wildcard *.c))`
4. Create `FunctionCall` AST nodes

## Full Test Suite (Add incrementally)

```rust
// Test 1: Basic filter (already shown above)
#[test]
fn test_PARSER_FUNC_001_basic_filter() { /* ... */ }

// Test 2: Sort function
#[test]
fn test_PARSER_FUNC_002_basic_sort() {
    let makefile = "SORTED := $(sort foo bar baz foo)";
    let ast = parse_makefile(makefile).unwrap();
    // Verify function call parsed
}

// Test 3: Multiple arguments
#[test]
fn test_PARSER_FUNC_003_multiple_args() {
    let makefile = "OBJS := $(filter %.o %.a, foo.o bar.c baz.a)";
    // Verify both patterns parsed
}

// Test 4: CRITICAL - Nested function calls
#[test]
fn test_PARSER_FUNC_004_nested_wildcard() {
    let makefile = "OBJS := $(filter %.o, $(wildcard *.c))";
    // Verify inner $(wildcard) parsed inside outer $(filter)
}

// Test 5: Word function
#[test]
fn test_PARSER_FUNC_005_word() {
    let makefile = "SECOND := $(word 2, foo bar baz)";
}

// Test 6: Notdir function
#[test]
fn test_PARSER_FUNC_006_notdir() {
    let makefile = "FILES := $(notdir src/main.c include/util.h)";
}

// Test 7: Addsuffix function
#[test]
fn test_PARSER_FUNC_007_addsuffix() {
    let makefile = "OBJS := $(addsuffix .o, foo bar baz)";
}

// Continue with remaining 13 functions...
```

## REFACTOR Checklist

After tests pass (GREEN):
- [ ] Extract helper functions for parsing function calls
- [ ] Ensure cyclomatic complexity < 10
- [ ] Add comments for complex logic
- [ ] Clean up code structure

## PROPERTY Phase

Add to `rash/src/make_parser/tests.rs`:

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_function_calls_parse(
            func_name in "[a-z]+",
            arg in "[a-z0-9%._-]+"
        ) {
            let makefile = format!("VAR := $({} {})", func_name, arg);
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());
        }
    }
}
```

Run: `cargo test prop_function_calls_parse`

## MUTATION Phase

```bash
cargo mutants --file rash/src/make_parser/parser.rs -- --lib

# Target: ≥90% kill rate
# If below 90%, add more tests to kill surviving mutants
```

## Success Criteria

- [ ] ✅ 15-20 new tests added (test_PARSER_FUNC_001 through test_PARSER_FUNC_015+)
- [ ] ✅ All tests passing: 1,330 → 1,345+ tests
- [ ] ✅ Property tests passing (100+ generated cases)
- [ ] ✅ Mutation score ≥90%
- [ ] ✅ Zero regressions (old tests still pass)
- [ ] ✅ FunctionCall AST nodes created for all test cases

## Context from Previous Sprints

### Sprint 61 Discovery (Read for context)
**Key Insight**: Deterministic functions (filter, sort, etc.) don't need purification themselves, but their **arguments** might contain non-deterministic code.

**Example**:
```makefile
$(filter %.o, $(wildcard *.c))
              ^^^^^^^^^^^^^^^
              Non-deterministic! Needs $(sort)

# Purified:
$(filter %.o, $(sort $(wildcard *.c)))
```

### Sprint 62 Validation
**Pattern confirmed** across all 13 deterministic functions:
- filter, filter-out, sort, word, wordlist, words
- firstword, lastword, notdir, suffix, basename
- addsuffix, addprefix

All need parser support for recursive argument analysis.

## Files to Reference

- `SPRINT-63-HANDOFF.md` - Full implementation plan
- `rash/src/make_parser/ast.rs` - FunctionCall already defined!
- `rash/src/make_parser/parser.rs` - Where you'll implement
- `rash/src/make_parser/tests.rs` - Where you'll add tests
- `CLAUDE.md` - EXTREME TDD workflow reference

## Troubleshooting

**Test fails with parse error?**
- Check if lexer recognizes `$(function_name`
- Verify comma parsing for arguments

**Complex nested calls fail?**
- Implement recursive descent in parser
- Parse inner function calls first

**Mutation score < 90%?**
- Add edge case tests
- Test error conditions
- Test boundary cases

## Quick Commands Reference

```bash
# Run specific test
cargo test test_PARSER_FUNC_001_basic_filter

# Run all parser tests
cargo test parser:: --lib

# Run all tests
cargo test --lib

# Check test count
cargo test --lib 2>&1 | grep "test result"

# Run property tests
cargo test prop_function_calls

# Mutation testing (takes 10-20 minutes)
cargo mutants --file rash/src/make_parser/parser.rs -- --lib

# Format code
cargo fmt

# Lint
cargo clippy
```

## Expected Timeline

- **Hour 1-2**: RED phase - Write 15-20 failing tests
- **Hour 3-6**: GREEN phase - Implement parser for function calls
- **Hour 7**: REFACTOR phase - Clean up, complexity < 10
- **Hour 8**: PROPERTY phase - Add property tests
- **Hour 9**: MUTATION phase - Run mutation testing, add tests
- **Hour 10**: DOCUMENTATION - Update roadmap, create handoff

## After Sprint 63

Once parser is complete, you'll be ready for:
- **Sprint 64**: Recursive semantic analysis (6-8 hours)
- **Sprint 65**: Recursive purification engine (10-12 hours)
- **Sprint 66**: High-risk functions FOREACH + CALL (12-15 hours)

**Total to Phase 2 completion**: ~36-45 hours from now

---

**Ready to Start?** Run: `cargo test --lib` to verify current state, then dive into the RED phase! 🚀

**Questions?** See SPRINT-63-HANDOFF.md for detailed implementation plan.
