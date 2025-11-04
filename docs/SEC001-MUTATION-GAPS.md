# SEC001 Mutation Coverage Gap Analysis

**Status**: Phase 0 Complete - Baseline Verified
**Rule**: SEC001 - CRITICAL command injection via eval
**Baseline Tests**: 6 existing unit tests
**Total Mutants**: 16
**Baseline Results**: 62.5% kill rate (10/16 caught, 6/16 missed)
**Target**: 90%+ kill rate

## 📊 Baseline Mutation Results

**Phase 0 Outcome** (COMPLETE):
- ✅ **Total Mutants**: 16
- ✅ **Caught**: 10 (62.5%)
- ❌ **Missed**: 6 (37.5%)
- 🎯 **Pattern Confirmed**: Identical to SC2064 - column calculation arithmetic

## 🚨 Identified Mutation Gaps (COMPLETE)

All 6 missed mutants are **arithmetic column/line calculation mutations** in the `check()` function.

### Gap: Column Position Arithmetic (6 MISSED)

**Missed Mutants** (baseline verified):
1. Line 39:56: `col - 1` - replace - with / (char_before calculation)
2. Line 39:56: `col - 1` - replace - with +
3. Line 59:30: `line_num + 1` - replace + with * (start line)
4. Line 60:25: `col + 1` - replace + with * (start column)
5. Line 61:30: `col + 5` - replace + with * (end column, "eval" = 4 chars + 1)
6. Line 62:25: `line_num + 1` - replace + with * (end line)

**Root Cause**: No tests verify exact column positions in diagnostic spans

**Impact**: Incorrect spans could break error reporting or confuse users

**Code Context**:
```rust
// Line 39: Check character before "eval"
let char_before = line.chars().nth(col - 1);  // MISSED: - → / and - → +

// Lines 59-61: Calculate diagnostic span
let span = Span::new(
    line_num + 1,  // Line 59 - MISSED: + → *
    col + 1,       // Line 60 - MISSED: + → *
    line_num + 1,
    col + 5,       // Line 61 - MISSED: + → *
);
```

### Required Tests (6 total)

All tests need to verify **exact column/line positions** to catch arithmetic mutations.

**Pattern Recognition**: This is **identical** to SC2064's mutation gaps!

```rust
#[test]
fn test_mutation_sec001_eval_start_col_exact() {
    // MUTATION: Line 60:25 - replace + with *
    let bash_code = "eval \"$cmd\"";  // eval starts at column 1
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    assert_eq!(span.start_col, 1, "Start column must use +1, not *1");
}

#[test]
fn test_mutation_sec001_eval_end_col_exact() {
    // MUTATION: Line 61:30 - replace + with *
    let bash_code = "eval \"$cmd\"";  // "eval" ends at column 4
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    assert_eq!(span.end_col, 5, "End column must be col + 5, not col * 5");
}

#[test]
fn test_mutation_sec001_line_num_calculation() {
    // MUTATION: Line 59:30 - replace + with * in line_num + 1
    let bash_code = "# comment\neval \"$var\"";  // eval on line 2
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].span.start_line, 2, "Line number must use +1, not *1");
}

#[test]
fn test_mutation_sec001_column_with_offset() {
    // Tests column calculations with leading whitespace
    let bash_code = "    eval \"$cmd\"";  // eval starts at column 5
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    assert_eq!(span.start_col, 5, "Must account for leading whitespace");
    assert_eq!(span.end_col, 9, "End must be start + 4 (\"eval\")");
}

#[test]
fn test_mutation_sec001_char_before_calculation() {
    // MUTATION: Line 39:56 - replace - with / or + in col - 1
    // Tests the char_before boundary check
    let bash_code = " eval \"$cmd\"";  // Space before eval at col 0
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    // Verifies col - 1 correctly checks the space character
}
```

## 🎯 Action Plan (Iteration 1)

**Priority**: CRITICAL (Error severity - command injection vulnerability)

**Estimated Work**: 5 new targeted tests (matching SC2064 pattern)

**Test Strategy**:
1. Verify exact column positions for all diagnostic spans
2. Test line number calculation (multiline input)
3. Test char_before boundary calculation (col - 1)
4. Include edge cases: whitespace, exact end column positions

**EXTREME TDD Workflow**:
```bash
# Phase 1: RED - Add 5 targeted tests for column positions
# Phase 2: GREEN - Verify all tests pass
# Phase 3: REFACTOR - Clean up test suite
# Phase 4: QUALITY - Re-run mutation testing
cargo mutants --file rash/src/linter/rules/sec001.rs --timeout 300 -- --lib

# Target: 90%+ kill rate (15/16 mutants caught)
```

## 🔄 Lessons from SC2064

**SC2064 Achieved 100% Kill Rate** with this exact approach:
- **Same Problem**: Arithmetic mutations in column calculations
- **Same Solution**: Exact position tests (assert_eq!(span.start_col, X))
- **Same Pattern**: + → *, + → -, - → + mutations

**Applied to SEC001**:
- SEC001 has **identical code structure** to SC2064
- Both have simple check() function with column arithmetic
- **Expected Result**: 90%+ kill rate (possibly 100% like SC2064)

## 📝 Notes

- SEC001 is **CRITICAL** - prevents command injection via eval
- **Iteration 1** should focus exclusively on column position tests
- **Expected result**: 90%+ kill rate (14-15/16 mutants caught)
- Simple, focused tests are more effective than complex ones
- Toyota Way (Jidoka) - build quality in from the start
- NASA-level standard: 90%+ mutation kill rate required

**Pattern Confirmed**: SEC001 follows **identical mutation pattern** to SC2064.
This means we can directly apply the SC2064 solution!

---

**Generated**: 2025-11-04
**Methodology**: EXTREME TDD + Mutation Testing
**Status**: Phase 0 (baseline), Iteration 1 planned
**Target**: NASA-level quality (90%+ mutation kill rate)
**Priority**: CRITICAL - command injection prevention

**🤖 Generated with Claude Code**
