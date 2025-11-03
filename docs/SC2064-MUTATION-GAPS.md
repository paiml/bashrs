# SC2064 Mutation Coverage Gap Analysis

**Status**: Iteration 1 Complete - 100% Kill Rate Achieved ✅
**Rule**: SC2064 - CRITICAL trap expansion timing issue
**Baseline Tests**: 10 unit tests + 6 property tests
**Iteration 1 Tests**: +4 column position tests
**Total Mutants**: 7
**Baseline Results**: 4 missed, 3 caught (42.9% kill rate)
**Iteration 1 Results**: 0 missed, 7 caught (100% kill rate) ✅
**Target**: 90%+ kill rate ✅ **EXCEEDED - Perfect score achieved!**

## 📊 Baseline Mutation Results

**Phase 0 Outcome**:
- ✅ Caught: 3/7 (42.9%)
- ❌ Missed: 4/7 (57.1%)
- 🎯 **Kill Rate: 42.9%** (Target: 90%+)

**Impact**: Need to add 4 targeted tests to catch the remaining 4 missed mutants.

## 🚨 Identified Mutation Gaps

All 4 missed mutants are **arithmetic column calculation mutations** in the `check()` function.

### Gap: Column Position Arithmetic (4 MISSED)

**Missed Mutants**:
- Line 39:33: `line_num + 1` - replace + with *
- Line 47:41: `mat.start() + 1` - replace + with *
- Line 48:37: `mat.end() + 1` - replace + with - (also + with *)

**Root Cause**: No tests verify exact column positions in diagnostic spans

**Impact**: Incorrect spans could break auto-fix or confuse users

**Code Context**:
```rust
// Lines 38-39 (line number calculation)
for (line_num, line) in source.lines().enumerate() {
    let line_num = line_num + 1;  // Line 39 - MISSED

// Lines 46-48 (column calculations)
if let Some(mat) = TRAP_DOUBLE_QUOTED.find(line) {
    let start_col = mat.start() + 1;  // Line 47 - MISSED
    let end_col = mat.end() + 1;      // Line 48 - MISSED (2x: + → * and + → -)
```

### Required Tests (4 total)

All tests need to verify **exact column positions** to catch arithmetic mutations:

```rust
#[test]
fn test_mutation_sc2064_trap_start_col_exact() {
    // MUTATION: Line 47:41 - replace + with *
    let bash_code = r#"trap "rm $tmpfile" EXIT"#;  // trap starts at column 1
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    assert_eq!(span.start_col, 1, "Start column must use +1, not *1");
}

#[test]
fn test_mutation_sc2064_trap_end_col_exact() {
    // MUTATION: Line 48:37 - replace + with * or -
    let bash_code = r#"trap "rm $tmpfile" EXIT"#;  // Pattern ends at column 18
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    assert_eq!(span.end_col, 18, "End column must use +1, not *1 or -1");
}

#[test]
fn test_mutation_sc2064_line_num_calculation() {
    // MUTATION: Line 39:33 - replace + with * in line_num calculation
    let bash_code = "# comment\ntrap \"rm $var\" EXIT";  // trap on line 2
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].span.start_line, 2, "Line number must use +1");
}

#[test]
fn test_mutation_sc2064_column_positions_with_offset() {
    // Tests column calculations with leading whitespace
    let bash_code = r#"    trap "rm $file" EXIT"#;  // trap starts at column 5
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    assert_eq!(span.start_col, 5, "Must account for leading whitespace");
    assert!(span.end_col > span.start_col, "End must be after start");
}
```

## 📊 Iteration 1 Results

**Date**: 2025-11-03 (Commit: pending)
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → QUALITY)

**Final Outcome**:
- ✅ **Kill Rate**: 100% (7/7 caught) - **PERFECT SCORE!**
- ❌ **Missed**: 0/7 (0%)
- 📈 **Improvement**: +57.1 percentage points from baseline (42.9% → 100%)
- 🎯 **Exceeded Target**: 90% target exceeded by 10 points!

**Tests Added**:
1. ✅ `test_mutation_sc2064_trap_start_col_exact` - Catches line 47 mutation
2. ✅ `test_mutation_sc2064_trap_end_col_exact` - Catches line 48 mutations (both + → * and + → -)
3. ✅ `test_mutation_sc2064_line_num_calculation` - Catches line 39 mutation
4. ✅ `test_mutation_sc2064_column_positions_with_offset` - Additional coverage

**Mutation Test Output**:
```
Found 7 mutants to test
ok       Unmutated baseline in 44.8s build + 38.5s test
7 mutants tested in 9m 55s: 7 caught
```

**All Mutations Caught**:
- Line 39:33: `line_num + 1` - replace + with * ✅ CAUGHT
- Line 47:41: `mat.start() + 1` - replace + with * ✅ CAUGHT
- Line 48:37: `mat.end() + 1` - replace + with - ✅ CAUGHT
- Line 48:37: `mat.end() + 1` - replace + with * ✅ CAUGHT
- (Plus 3 previously caught mutations) ✅ CAUGHT

**Quality Metrics**:
- ✅ All 20 tests passing (100% pass rate)
- ✅ Zero clippy warnings
- ✅ Code formatted with cargo fmt
- ✅ Complexity <10
- ✅ 100% mutation kill rate (target: 90%)

**Toyota Way Principles Applied**:
- 🚨 **Jidoka (自働化)**: Built quality into tests through exact column verification
- 🔍 **Genchi Genbutsu (現地現物)**: Empirically validated with cargo-mutants
- 📈 **Kaizen (改善)**: Improved from 42.9% → 100% (+57.1 points)
- 🎯 **Hansei (反省)**: Identified gap, fixed systematically, achieved perfection

## 📊 Summary of Mutations

**Total Missed**: 4/7 (57.1%)

**Breakdown by Type**:
1. Column calculation arithmetic: 4 mutants (all + → * or + → -)
   - mat.start() + 1: 1 mutation (line 47)
   - mat.end() + 1: 2 mutations (line 48: + → * and + → -)
   - line_num + 1: 1 mutation (line 39)

**Caught**: 3/7 (42.9%)
- Likely: Basic detection tests, severity checks, message content

## 🎯 Action Plan (Iteration 1)

**Priority**: HIGH (CRITICAL security rule - trap expansion timing)

**Estimated Work**: 4 new targeted tests

**Test Strategy**:
1. Verify exact column positions for all diagnostic spans
2. Test line number calculation (multiline input)
3. Include edge cases: whitespace, exact end column positions

**EXTREME TDD Workflow**:
```bash
# Phase 1: RED - Add 4 targeted tests for column positions
# Phase 2: GREEN - Verify all tests pass
# Phase 3: REFACTOR - Clean up test suite
# Phase 4: QUALITY - Re-run mutation testing
cargo mutants --file rash/src/linter/rules/sc2064.rs --timeout 300 -- --lib

# Target: 90%+ kill rate (6/7 mutants caught)
```

## 🔄 Lessons Learned from SC2059

**Similar Issue**:
- SC2059 had same problem: column calculations not tested
- SC2059 Iteration 1: Added exact column position tests
- Result: Improved from 41.7% → 91.7% kill rate (+50 points)

**Applied to SC2064**:
- Need exact column position verification (same pattern)
- Test all arithmetic operations (+ mutations)
- Include line number calculation tests
- Property tests alone are NOT sufficient - need exact value assertions

## 📝 Notes

- SC2064 is **CRITICAL** - prevents trap timing bugs (variables expand too early)
- **Iteration 1** should focus exclusively on column position tests
- **Expected result**: 90%+ kill rate (6/7 mutants caught)
- Simple, focused tests are more effective than complex ones
- Toyota Way (Jidoka) - build quality in, stop the line for quality gaps
- NASA-level standard: 90%+ mutation kill rate required

**Why Property Tests Failed**:
- Property tests check invariants (>= 1, end > start)
- They do NOT check EXACT values (e.g., start_col == 5)
- Mutations like `mat.start() * 1` still satisfy >= 1 invariant
- NEED: Exact position assertions to catch arithmetic mutations

---

**Generated**: 2025-11-03
**Methodology**: EXTREME TDD + Mutation Testing
**Status**: Phase 0 complete, Iteration 1 planned
**Target**: NASA-level quality (90%+ mutation kill rate)
**Priority**: HIGH - CRITICAL security rule

**🤖 Generated with Claude Code**
