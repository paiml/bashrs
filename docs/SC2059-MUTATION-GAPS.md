# SC2059 Mutation Coverage Gap Analysis

**Status**: Phase 1 Complete - Baseline Established
**Rule**: SC2059 - CRITICAL printf format string injection
**Baseline Tests**: 10 existing unit tests
**Total Mutants**: 12
**Baseline Results**: 7 missed, 5 caught (41.7% kill rate)

## 📊 Baseline Mutation Results

**Phase 1 Outcome**:
- ✅ Caught: 5/12 (41.7%)
- ❌ Missed: 7/12 (58.3%)
- 🎯 **Kill Rate: 41.7%** (Target: 90%+)

**Impact**: Need to add 5-7 targeted tests to catch the remaining 7 missed mutants.

## 🚨 Identified Mutation Gaps

All 7 missed mutants are **arithmetic column calculation mutations** in the `check()` function.

### Gap: Column Position Arithmetic (7 MISSED)

**Missed Mutants**:
- Line 45:33: `line_num + 1` - replace + with *
- Line 53:41: `mat.start() + 1` - replace + with *
- Line 54:37: `mat.end() + 1` - replace + with * (2x: also + with -)
- Line 71:45: `mat.start() + 1` - replace + with *
- Line 72:41: `mat.end() + 1` - replace + with * (2x: also + with -)

**Root Cause**: No tests verify exact column positions in diagnostic spans

**Impact**: Incorrect spans could break auto-fix or confuse users

**Code Context**:
```rust
// Lines 52-54 (PRINTF_WITH_VAR pattern)
if let Some(mat) = PRINTF_WITH_VAR.find(line) {
    let start_col = mat.start() + 1;  // Line 53 - MISSED
    let end_col = mat.end() + 1;      // Line 54 - MISSED (2x)

// Lines 68-72 (PRINTF_WITH_EXPANSION pattern)
if let Some(mat) = PRINTF_WITH_EXPANSION.find(line) {
    let start_col = mat.start() + 1;  // Line 71 - MISSED
    let end_col = mat.end() + 1;      // Line 72 - MISSED (2x)
```

### Required Tests (7 total)

All tests need to verify **exact column positions** to catch arithmetic mutations:

```rust
#[test]
fn test_mutation_sc2059_printf_var_start_col_exact() {
    // MUTATION: Line 53:41 - replace + with *
    let bash_code = "printf $fmt arg";  // $fmt starts at column 8
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    assert_eq!(span.start_col, 8, "Start column must use +1, not *1");
}

#[test]
fn test_mutation_sc2059_printf_var_end_col_exact() {
    // MUTATION: Line 54:37 - replace + with * or -
    let bash_code = "printf $fmt";  // $fmt ends at column 12
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    assert_eq!(span.end_col, 12, "End column must use +1, not *1 or -1");
}

#[test]
fn test_mutation_sc2059_printf_expansion_start_col_exact() {
    // MUTATION: Line 71:45 - replace + with *
    let bash_code = r#"printf "hello $name""#;  // String starts at column 8
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    assert_eq!(span.start_col, 8, "Start column calculation must use +1");
}

#[test]
fn test_mutation_sc2059_printf_expansion_end_col_exact() {
    // MUTATION: Line 72:41 - replace + with * or -
    let bash_code = r#"printf "$var""#;  // String ends at column 14
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    assert_eq!(span.end_col, 14, "End column must use +1, not *1 or -1");
}

#[test]
fn test_mutation_sc2059_line_num_calculation() {
    // MUTATION: Line 45:33 - replace + with * in line_num calculation
    let bash_code = "# comment\nprintf $var";  // printf on line 2
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].span.start_line, 2, "Line number must use +1");
}

#[test]
fn test_mutation_sc2059_column_positions_with_offset() {
    // Tests column calculations with leading whitespace
    let bash_code = "    printf $fmt";  // $fmt starts at column 12
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    assert_eq!(span.start_col, 12);
    assert_eq!(span.end_col, 16);
}

#[test]
fn test_mutation_sc2059_expansion_column_accuracy() {
    // Tests PRINTF_WITH_EXPANSION pattern column accuracy
    let bash_code = r#"printf "test $var""#;
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    // Verify span covers the entire format string
    assert!(span.end_col > span.start_col);
    assert_eq!(span.start_col, 8);  // After "printf "
}
```

## 📊 Summary of Mutations

**Total Missed**: 7/12 (58.3%)

**Breakdown by Type**:
1. Column calculation arithmetic: 7 mutants (all + → * or + → -)
   - mat.start() + 1: 2 mutations (lines 53, 71)
   - mat.end() + 1: 4 mutations (lines 54 x2, 72 x2)
   - line_num + 1: 1 mutation (line 45)

**Caught**: 5/12 (41.7%)
- Likely: Basic detection tests, severity checks, message content

## 🎯 Action Plan (Iteration 1)

**Priority**: HIGH (CRITICAL security rule - format string injection)

**Estimated Work**: 7 new targeted tests

**Test Strategy**:
1. Verify exact column positions for all diagnostic spans
2. Test both PRINTF_WITH_VAR and PRINTF_WITH_EXPANSION patterns
3. Include edge cases: whitespace, multi-line, line number calculation

**EXTREME TDD Workflow**:
```bash
# Phase 1: RED - Add 7 targeted tests for column positions
# Phase 2: GREEN - Verify all tests pass
# Phase 3: REFACTOR - Clean up test suite
# Phase 4: QUALITY - Re-run mutation testing
cargo mutants --file rash/src/linter/rules/sc2059.rs --timeout 300 -- --lib

# Target: 90%+ kill rate (11/12 mutants caught)
```

## 🔄 Lessons Learned from SC2086

**Similar Issue**:
- SC2086 had same problem: column calculations not tested
- SC2086 Iteration 2: Added exact column position tests
- Result: Improved from 25.7% → 57.1% kill rate

**Applied to SC2059**:
- Need exact column position verification
- Test all arithmetic operations (+ mutations)
- Include line number calculation tests

## 📝 Notes

- SC2059 is **CRITICAL** - prevents format string injection vulnerabilities
- **Iteration 1** should focus exclusively on column position tests
- **Expected result**: 90%+ kill rate (11/12 mutants caught)
- Simple, focused tests are more effective than complex ones
- Toyota Way (Jidoka) - build quality in, stop the line for quality gaps
- NASA-level standard: 90%+ mutation kill rate required

---

**Generated**: 2025-11-03
**Methodology**: EXTREME TDD + Mutation Testing
**Status**: Phase 1 complete, Iteration 1 planned
**Target**: NASA-level quality (90%+ mutation kill rate)
**Priority**: HIGH - CRITICAL security rule

**🤖 Generated with Claude Code**
