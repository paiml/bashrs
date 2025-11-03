# SC2086 Mutation Coverage Gap Analysis

**Status**: Phase 4 Complete - Second Iteration Needed
**Rule**: SC2086 - CRITICAL word splitting/globbing protection
**Baseline Tests**: 12 existing unit tests
**Iteration 1 Tests**: +9 mutation coverage tests (commit 329b5c11)
**Total Mutants**: 35
**Final Results**: 24 missed, 9 caught, 1 unviable (25.7% kill rate)

## 📊 Final Mutation Results

**Iteration 1 Outcome**:
- ✅ Caught: 9/35 (25.7%)
- ❌ Missed: 24/35 (68.6%)
- ⚠️ Unviable: 1/35 (2.9%)
- 🎯 **Kill Rate: 25.7%** (Target: 90%+)

**Impact**: First iteration added targeted tests but **24 additional mutants require tests**. Need second iteration with focus on helper functions.

## 🚨 Identified Mutation Gaps (Second Iteration)

### Gap 1: Helper Function - should_skip_line() (6 MISSED)

**Missed Mutants**:
- Line 22:27: `&&` replaced with `||` (condition logic)
- Line 22:30: delete `!` (negation removal)
- Line 22:53: `&&` replaced with `||` (condition logic)
- Line 22:56: delete `!` (negation removal)
- Line 25:27: `<` replaced with `<=` (comparison boundary)
- Line 25:27: `<` replaced with `>` (comparison direction)
- Line 25:27: `<` replaced with `==` (comparison type)

**Root Cause**: No tests for should_skip_line() helper function

**Impact**: Could cause false positives (flagging comments/assignments) or false negatives (missing actual violations)

**Required Tests** (7 total for should_skip_line):
```rust
#[test]
fn test_should_skip_comment_lines() {
    let result = check("# This is a comment with $VAR\necho $ACTUAL");
    assert_eq!(result.diagnostics.len(), 1); // Only $ACTUAL
}

#[test]
fn test_should_not_skip_assignments_in_tests() {
    let result = check("if [ $VAR = value ]; then echo ok; fi");
    assert_eq!(result.diagnostics.len(), 1); // Should detect $VAR
}
```

### Gap 2: Helper Function - find_dollar_position() (1 MISSED)

**Missed Mutant**:
- Line 37:5: replace `find_dollar_position -> usize` with `0`

**Root Cause**: No tests verifying correct $ position calculation

**Impact**: Incorrect span start positions in diagnostics

**Required Test**:
```rust
#[test]
fn test_dollar_position_calculation() {
    let result = check("ls ${FILE}");
    let span = result.diagnostics[0].span;
    assert_eq!(span.start_col, 4); // Should find $ at position 4, not 0
}
```

### Gap 3: Helper Function - is_already_quoted() (2 MISSED)

**Missed Mutants**:
- Line 63:5: replace `is_already_quoted -> bool` with `false`
- Line 65:35: replace `&&` with `||` in `is_already_quoted`

**Root Cause**: No explicit tests for is_already_quoted() behavior

**Impact**: Could incorrectly flag already-quoted variables

**Required Tests**:
```rust
#[test]
fn test_is_already_quoted_detection() {
    // Should NOT flag (already quoted)
    let result1 = check("echo \"$VAR\"");
    assert_eq!(result1.diagnostics.len(), 0);

    // Should flag (not quoted)
    let result2 = check("echo $VAR");
    assert_eq!(result2.diagnostics.len(), 1);
}
```

### Gap 4: Column Calculation - Additional Operators (10 MISSED)

**Missed Mutants** (calculate_end_column):
- Line 45:21: `+` replaced with `*` (MISSED - brace_pos calculation)
- Line 45:21: `+` replaced with `-` (MISSED)
- Line 45:33: `+` replaced with `*` (MISSED - +1 for })
- Line 45:33: `+` replaced with `-` (MISSED)
- Line 47:21: `+` replaced with `*` (MISSED - fallback)
- Line 47:21: `+` replaced with `-` (MISSED)
- Line 50:17: `+` replaced with `*` (MISSED - simple case)
- Line 50:17: `+` replaced with `-` (MISSED)

**Missed Mutants** (check function):
- Line 121:34: `+` replaced with `*` (MISSED - line indexing)
- Line 121:34: `+` replaced with `-` (MISSED)

**Root Cause**: Tests verify some positions but not all arithmetic edge cases

**Impact**: Incorrect diagnostic spans could break auto-fix

**Required Tests** (already partially covered, need more precise checks):
```rust
#[test]
fn test_column_calculation_edge_cases() {
    // Test with offset variables
    let result = check("    echo ${VAR}"); // 4 spaces prefix
    let span = result.diagnostics[0].span;
    assert_eq!(span.start_col, 10); // 4 spaces + "echo " + "$"
}
```

### Gap 5: Arithmetic Context - return value mutations (2 CAUGHT ✅)

**Caught Mutants**:
- Line 56: `is_in_arithmetic_context -> bool` returns `true` ✅
- Line 56: `is_in_arithmetic_context -> bool` returns `false` ✅

**Status**: These 2 mutants ARE caught by iteration 1 tests!

### Gap 6: Check Function Logic (2 MISSED)

**Missed Mutants**:
- Line 111:50: `||` replaced with `&&` in check (MISSED)
- Line 127:30: `&&` replaced with `||` in check (MISSED)

**Root Cause**: Tests don't verify all conditional branches

**Required Tests**
```rust
#[test]
fn test_mutation_arithmetic_false_positive() {
    // MUTATION: If is_in_arithmetic_context always returns true,
    // this test should fail (we'd skip detection incorrectly)
    let bash_code = "echo $VAR";  // Not in arithmetic
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1, "Should detect unquoted var outside arithmetic");
}

#[test]
fn test_mutation_arithmetic_false_negative() {
    // MUTATION: If is_in_arithmetic_context always returns false,
    // this test should fail (we'd incorrectly flag safe arithmetic)
    let bash_code = "result=$(( $x + $y ))";
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 0, "Should NOT flag variables in arithmetic");
}

#[test]
fn test_mutation_arithmetic_both_conditions() {
    // MUTATION: If && becomes ||, this should fail
    // Verifies BOTH $(( and )) are required
    let bash_code1 = "echo $(( $VAR";  // Missing closing ))
    let result1 = check(bash_code1);
    assert!(result1.diagnostics.len() > 0, "Should flag incomplete arithmetic");

    let bash_code2 = "echo $VAR ))";  // Missing opening $((
    let result2 = check(bash_code2);
    assert!(result2.diagnostics.len() > 0, "Should flag incomplete arithmetic");
}
```

### Gap 2: Column Calculation Arithmetic

**Missed Mutants**:
- Line 45: `+` replaced with `*` in `calculate_end_column` (MISSED)
- Line 50: `+` replaced with `*` in `calculate_end_column` (MISSED)
- Line 50: `+` replaced with `-` in `calculate_end_column` (MISSED)

**Root Cause**: Tests don't verify exact column positions in spans

**Impact**: Incorrect diagnostic spans could confuse users or break auto-fix

**Required Tests**:
```rust
#[test]
fn test_mutation_column_calculation_braced() {
    // MUTATION: If + becomes * or -, column positions will be wrong
    let bash_code = "echo ${VAR}";
    let result = check(bash_code);

    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;

    // Verify exact column positions
    assert_eq!(span.start_col, 6, "Start should be at $");
    assert_eq!(span.end_col, 12, "End should include closing }");
}

#[test]
fn test_mutation_column_calculation_simple() {
    // MUTATION: Verifies column calculation for simple $VAR
    let bash_code = "echo $VAR";
    let result = check(bash_code);

    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;

    assert_eq!(span.start_col, 6, "Start should be at $");
    assert_eq!(span.end_col, 10, "End should be after VAR");
}
```

### Gap 3: Check Function Logic

**Missed Mutants**:
- Line 121: `+` replaced with `-` in `check` (MISSED)
- Line 127: `&&` replaced with `||` in `check` (MISSED)

**Root Cause**: Tests don't verify line number calculations and condition combinations

**Impact**: Diagnostics could report wrong line numbers, conditional logic could be broken

**Required Tests**:
```rust
#[test]
fn test_mutation_line_numbers() {
    // MUTATION: If + becomes -, line numbers will be off
    let bash_code = r#"
#!/bin/bash
echo "first"
echo $VAR
echo "last"
"#;

    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].span.start_line, 4, "Should be line 4");
}

#[test]
fn test_mutation_arithmetic_check_logic() {
    // MUTATION: If && becomes || in check function (line 127),
    // verify arithmetic detection still works correctly
    let bash_code = "result=$(( $x + $y ))";
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 0, "Arithmetic context check must work");
}
```

## 📊 Summary of Mutations

**Total Missed**: 24/35 (68.6%)

**Breakdown by Category**:
1. should_skip_line() helper: 6 mutants (logical operators, comparisons)
2. find_dollar_position() helper: 1 mutant (return value)
3. is_already_quoted() helper: 2 mutants (return value, logic)
4. calculate_end_column() arithmetic: 8 mutants (various operators)
5. check() function arithmetic: 2 mutants (line indexing)
6. check() function logic: 2 mutants (conditional operators)
7. Arithmetic context (CAUGHT): 2 mutants ✅
8. Column calculations (PARTIAL): Some caught, some missed

**Caught by Iteration 1**: 9 mutants (25.7%)
**Improvement needed**: 15+ additional tests to reach 90%+ kill rate

## 🎯 Action Plan (Iteration 2)

**Priority**: HIGH (CRITICAL security rule)

**Estimated Work**: 15-20 new tests

**Test Categories Needed**:
1. **Helper function tests** (9 tests):
   - should_skip_line(): 7 tests (comments, assignments, edge cases)
   - find_dollar_position(): 1 test (position calculation)
   - is_already_quoted(): 1 test (quoting detection)

2. **Column calculation tests** (4-6 tests):
   - Arithmetic operator variations (*, -, +)
   - Edge cases with spacing/offsets
   - Boundary conditions

3. **Check function logic tests** (2-3 tests):
   - Conditional branch coverage
   - Line number calculation edge cases
   - Multiple condition combinations

**EXTREME TDD Workflow (Iteration 2)**:
```bash
# Phase 1: RED - Add targeted tests for 24 missed mutants
# Phase 2: GREEN - Verify all tests pass
# Phase 3: REFACTOR - Clean up test suite
# Phase 4: QUALITY - Re-run mutation testing
cargo mutants --file rash/src/linter/rules/sc2086.rs --timeout 300 -- --lib

# Target: 90%+ kill rate (32/35 mutants caught)
```

## 📝 Notes

- SC2086 is **CRITICAL** - prevents injection vulnerabilities
- **Iteration 1** added 9 tests, caught 9 mutants (25.7% kill rate)
- **Iteration 2** needs 15+ tests to reach 90%+ target
- Focus on helper functions (currently untested)
- Helper function mutations represent potential bugs that would go undetected
- Toyota Way (Jidoka) - build quality in, stop the line for quality gaps
- NASA-level standard: 90%+ mutation kill rate required

## 🔄 Lessons Learned

**From Iteration 1**:
1. ✅ Targeted tests for specific mutants DO work
2. ✅ Column calculation tests caught related mutants
3. ✅ Arithmetic context tests caught return value mutations
4. ❌ Forgot to test helper functions (should_skip_line, find_dollar_position, is_already_quoted)
5. ❌ Need more comprehensive arithmetic operator testing

**For Iteration 2**:
1. Test ALL helper functions explicitly
2. Test ALL arithmetic operators (not just +)
3. Test ALL conditional operators (&&, ||, comparisons)
4. One test per mutation gap (systematic approach)

---

**Generated**: 2025-11-03 (Updated after Iteration 1 completion)
**Methodology**: EXTREME TDD + Mutation Testing
**Status**: Iteration 1 complete (25.7% kill rate), Iteration 2 pending
**Target**: NASA-level quality (90%+ mutation kill rate)
**Priority**: HIGH - CRITICAL security rule
