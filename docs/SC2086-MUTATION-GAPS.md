# SC2086 Mutation Coverage Gap Analysis

**Status**: In Progress (Mutation test running)
**Rule**: SC2086 - CRITICAL word splitting/globbing protection
**Baseline Tests**: 12 existing unit tests
**Total Mutants**: 35
**Preliminary Results**: 8 missed mutants identified (test ~25% complete)

## 🚨 Identified Mutation Gaps

### Gap 1: Arithmetic Context Detection

**Missed Mutants**:
- Line 56: `is_in_arithmetic_context -> bool` returns `true` (MISSED)
- Line 56: `is_in_arithmetic_context -> bool` returns `false` (MISSED)
- Line 58: `&&` replaced with `||` in `is_in_arithmetic_context` (FAILED build)

**Root Cause**: Missing tests that verify arithmetic context is correctly identified/rejected

**Impact**: Could cause false negatives (missing unquoted vars) or false positives (flagging safe arithmetic)

**Required Tests**:
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

## 📊 Expected Improvements

**Before**: ~77% kill rate (8 missed out of ~35 mutants so far)
**Target**: 90%+ kill rate
**Tests to Add**: ~9 mutation coverage tests

## 🎯 EXTREME TDD Plan

### Phase 1: RED (Write Failing Tests)
1. Add all 9 tests above
2. Verify each test PASSES with current code
3. Verify tests would FAIL if mutations were applied

### Phase 2: GREEN (Already Passing)
- Current implementation should pass all new tests
- If not, fix implementation

### Phase 3: REFACTOR (If Needed)
- Clean up any test duplication
- Ensure complexity stays <10

### Phase 4: QUALITY (Re-run Mutation Testing)
- Run: `cargo mutants --file rash/src/linter/rules/sc2086.rs --timeout 300 -- --lib`
- Target: 90%+ kill rate
- Verify all gaps closed

## 📝 Notes

- SC2086 is **CRITICAL** - prevents injection vulnerabilities
- High test quality essential for security
- Each mutation gap represents a potential bug that tests wouldn't catch
- This analysis demonstrates Toyota Way (Jidoka) - building quality in through rigorous testing

---

**Generated**: 2025-11-03
**Methodology**: EXTREME TDD + Mutation Testing
**Quality Target**: NASA-level (90%+ mutation kill rate)
