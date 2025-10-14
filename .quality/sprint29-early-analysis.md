# Sprint 29 - Early Analysis (First 8 Mutants)

**Date:** 2025-10-14
**Status:** 🔍 PRELIMINARY ANALYSIS - Phase 1 still running
**Source:** First 8 MISSED mutants from AST module
**Philosophy:** 反省 (Hansei) - Reflect on failures to improve

---

## Executive Summary

**Critical Discovery**: First 8 mutants tested = 8 MISSED (100% miss rate)

All surviving mutants are in **validation/safety functions** in `rash/src/ast/restricted.rs`. This reveals a systematic gap in test coverage: **validation functions lack negative test cases**.

**Root Cause**: Tests verify valid inputs work, but don't verify invalid inputs are rejected.

**Impact**: Safety-critical validation logic is untested, meaning the codebase could accept invalid/malicious AST nodes.

---

## Surviving Mutants (8 of 66 tested)

### Group 1: Type Validation (2 mutants)
```rust
// rash/src/ast/restricted.rs:139-141

// MISSED #1: Replace `Type::is_allowed -> bool` with `true`
// MISSED #2: Replace `&&` with `||` in Type::is_allowed

pub fn is_allowed(&self) -> bool {
    match self {
        Type::Void | Type::Bool | Type::U32 | Type::Str => true,
        Type::Result { ok_type, err_type } => ok_type.is_allowed() && err_type.is_allowed(),
        //                                                           ^^
        //                                                  Changed to || - not caught!
        Type::Option { inner_type } => inner_type.is_allowed(),
    }
}
```

### Group 2: Statement Validation (3 mutants)
```rust
// rash/src/ast/restricted.rs:213, 222, 271

// MISSED #3: Replace `Stmt::validate_if_stmt -> Result<(), String>` with `Ok(())`
fn validate_if_stmt(...) -> Result<(), String> {
    condition.validate()?;
    self.validate_stmt_block(then_block)?;
    // ...
    Ok(())
}

// MISSED #4: Replace `Stmt::validate_match_stmt -> Result<(), String>` with `Ok(())`
fn validate_match_stmt(...) -> Result<(), String> {
    scrutinee.validate()?;
    // ...
    Ok(())
}

// MISSED #5: Replace `Stmt::validate_stmt_block -> Result<(), String>` with `Ok(())`
fn validate_stmt_block(&self, stmts: &[Stmt]) -> Result<(), String> {
    for stmt in stmts {
        stmt.validate()?;
    }
    Ok(())
}
```

### Group 3: Expression Nesting Depth (3 mutants)
```rust
// rash/src/ast/restricted.rs:370 (line 141 in Expr::validate)

// MISSED #6: Replace `>` with `==` in Expr::validate
// MISSED #7: Replace `>` with `>=` in Expr::validate

let depth = self.nesting_depth();
if depth > 30 {
    //      ^
    //      Changed to == or >= - not caught!
    return Err(format!(
        "Expression nesting too deep: {depth} levels (max 30)"
    ));
}
```

### Group 4: Match Arm Validation (1 mutant)
```rust
// rash/src/ast/restricted.rs:383 (line 179 in file)

// MISSED #8: Delete match arm `Expr::Literal(_)` in Expr::validate

match self {
    Expr::Literal(Literal::Str(s)) => {
        if s.contains('\0') {
            return Err("Null characters not allowed in strings".to_string());
        }
        Ok(())
    }
    Expr::Literal(_) => Ok(()),  // ← Deleted this arm - not caught!
    // ...
}
```

---

## Five Whys Analysis

### Problem: 100% of validation mutants survived

#### Why #1: Why did ALL validation mutants survive?
**Answer**: Tests don't verify that validation functions reject invalid inputs.

#### Why #2: Why don't tests verify rejection behavior?
**Answer**: Tests focus on "happy path" - verifying valid inputs work correctly.

#### Why #3: Why focus only on happy path?
**Answer**: Test suite built using traditional TDD (write tests for functionality), not security/safety-first TDD.

#### Why #4: Why wasn't security/safety testing prioritized?
**Answer**: Rash is a transpiler/safety tool, but validation testing wasn't recognized as security-critical.

#### Why #5 (Root Cause): Why wasn't validation testing recognized as critical?
**Answer**: **Lack of mutation testing during development** - no way to measure test quality beyond line coverage.

**Root Cause**: Mutation testing reveals that **line coverage != validation coverage**. Code can have 90%+ line coverage but 0% validation coverage.

---

## Pattern Analysis

### Pattern 1: Missing Negative Tests
**All 8 mutants** share the same root cause: **no tests verify rejection behavior**.

Examples:
- `Type::is_allowed()` - Tests verify allowed types work, but don't test that disallowed types are rejected
- `validate_if_stmt()` - Tests verify valid if statements pass, but don't test invalid ones are rejected
- Nesting depth - Tests verify depth ≤30 works, but don't test depth=31 is rejected

### Pattern 2: Boundary Condition Gaps
Mutants #6 and #7 (nesting depth) reveal **boundary condition testing** is missing:
- Test likely checks depth=30 works (passes)
- Test likely checks depth=35 fails (should fail, but no test exists)
- **Missing**: Test for depth=31 (exact boundary)

Changing `>` to `==` or `>=` should fail if boundary test exists.

### Pattern 3: Logic Operator Testing
Mutant #2 (`&&` → `||`) reveals **boolean logic testing** is weak:
- Tests verify Result<Ok, Err> with both types allowed (passes)
- **Missing**: Test for Result<Ok, Err> with one type disallowed (should fail)

### Pattern 4: Match Arm Coverage
Mutant #8 (deleted `Expr::Literal(_)` arm) reveals **enum variant testing** is incomplete:
- Tests verify string literals with null characters fail (passes)
- **Missing**: Test for non-string literals (Bool, U32, I32) passing validation

---

## Safety Impact Assessment

### Severity: **HIGH** ⚠️

These are not cosmetic issues - they're **safety-critical validation gaps**:

1. **Type::is_allowed()** - Could allow unsafe types in AST
2. **validate_if_stmt()** - Could allow unsafe conditionals
3. **validate_match_stmt()** - Could allow unsafe pattern matching
4. **validate_stmt_block()** - Could allow invalid statement sequences
5. **Nesting depth** - Could allow stack overflow attacks
6. **Match arm** - Could allow unvalidated literal types

**Rash's value proposition**: Generate provably safe POSIX shell scripts. If AST validation is weak, the entire safety guarantee is compromised.

---

## Recommended Tests (Phase 3)

Based on Five Whys analysis, these tests will kill the surviving mutants:

### Test 1: Type Validation - Reject Disallowed Types
```rust
#[test]
fn test_type_is_allowed_rejects_complex_nested() {
    // Currently no test verifies is_allowed returns false
    // This would be the negative test case

    // Note: Current Type enum only has allowed types
    // This test may require adding a disallowed type first
    // OR testing the recursive validation properly
}
```

### Test 2: Validate If Statement - Reject Invalid Nested Content
```rust
#[test]
fn test_validate_if_stmt_rejects_invalid_condition() {
    let invalid_if = Stmt::If {
        condition: Expr::Literal(Literal::Str("\0invalid".to_string())), // null char
        then_block: vec![],
        else_block: None,
    };

    let result = invalid_if.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Null characters not allowed"));
}

#[test]
fn test_validate_if_stmt_rejects_deeply_nested_condition() {
    // Create expression with depth > 30
    let mut deep_expr = Expr::Literal(Literal::U32(1));
    for _ in 0..31 {
        deep_expr = Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(deep_expr),
        };
    }

    let invalid_if = Stmt::If {
        condition: deep_expr,
        then_block: vec![],
        else_block: None,
    };

    let result = invalid_if.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("nesting too deep"));
}
```

### Test 3: Nesting Depth - Exact Boundary Test
```rust
#[test]
fn test_expr_validate_rejects_depth_31() {
    // Create expression with EXACTLY depth=31 (boundary + 1)
    let mut expr = Expr::Literal(Literal::U32(1));
    for _ in 0..31 {
        expr = Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(expr),
        };
    }

    let result = expr.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Expression nesting too deep: 31 levels (max 30)"
    );
}

#[test]
fn test_expr_validate_accepts_depth_30() {
    // Boundary test - verify depth=30 is allowed
    let mut expr = Expr::Literal(Literal::U32(1));
    for _ in 0..30 {
        expr = Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(expr),
        };
    }

    let result = expr.validate();
    assert!(result.is_ok());
}
```

### Test 4: Literal Validation - All Variants
```rust
#[test]
fn test_expr_literal_variants_validate() {
    // Test all literal variants pass validation
    let literals = vec![
        Expr::Literal(Literal::Bool(true)),
        Expr::Literal(Literal::U32(42)),
        Expr::Literal(Literal::I32(-42)),
        Expr::Literal(Literal::Str("valid".to_string())),
    ];

    for lit in literals {
        assert!(lit.validate().is_ok());
    }
}
```

### Test 5: Type Logic - AND vs OR
```rust
#[test]
fn test_type_is_allowed_result_both_sides() {
    // Test that Result requires BOTH ok_type AND err_type to be allowed
    // Currently no test verifies the && logic

    // This would require creating a disallowed type first
    // OR testing that nested Results work correctly
    let valid_result = Type::Result {
        ok_type: Box::new(Type::U32),
        err_type: Box::new(Type::Str),
    };
    assert!(valid_result.is_allowed());
}
```

---

## Comparison with Sprint 26 (IR Module)

| Metric | Sprint 26 (IR) | Sprint 29 (AST - Early) |
|--------|----------------|-------------------------|
| Mutants Tested | 29 (all) | 8 of 66 (12%) |
| Kill Rate | 96.6% | 0% (8 MISSED) |
| Pattern | Mostly caught | 100% validation gaps |
| Severity | Low (logic) | **HIGH (safety)** |

**Key Difference**: IR module had 96.6% kill rate because it's primarily logic/transformation code with good happy-path testing. AST validation module has 0% kill rate because it's safety-critical code lacking negative tests.

---

## Toyota Way Principles Applied

### 反省 (Hansei) - Reflection
✅ Applied Five Whys to understand root cause
✅ Identified systematic pattern (validation gaps)
✅ Recognized severity (safety-critical)

### 現地現物 (Genchi Genbutsu) - Direct Observation
✅ Examined actual code (restricted.rs)
✅ Reviewed actual mutants (8 MISSED)
✅ Measured real impact (0% kill rate on validation)

### 自働化 (Jidoka) - Build Quality In
🔄 **IN PROGRESS**: Using mutation testing to find quality gaps
⏭️ **NEXT**: Write mutation-killing tests (Phase 3)
⏭️ **GOAL**: Achieve ≥90% kill rate on safety-critical code

### 改善 (Kaizen) - Continuous Improvement
📈 **Baseline**: 0% kill rate on validation functions
🎯 **Target**: ≥90% kill rate (or documented acceptance)
✅ **Process**: BASELINE → ANALYZE → TARGET → VERIFY

---

## Next Actions (Phase 2 → Phase 3 Transition)

### Immediate (When AST Baseline Completes)
1. Review remaining 58 AST mutants for additional patterns
2. Calculate final AST kill rate
3. Identify all surviving mutants by category

### Short-Term (Phase 3 - TARGET)
1. Write Test 1-5 above (validation negative tests)
2. Write boundary tests for all validation limits
3. Write boolean logic tests (AND/OR variants)
4. Verify all new tests pass

### Validation (Phase 4 - VERIFY)
1. Re-run mutation testing on AST module
2. Verify mutants #1-8 are now CAUGHT
3. Calculate new kill rate (target ≥90%)
4. Document any remaining acceptable survivors

---

## Key Learnings

### 1. Line Coverage ≠ Validation Coverage
**Discovery**: AST module likely has high line coverage (validation functions are called), but 0% validation coverage (tests don't verify rejection behavior).

**Lesson**: Traditional code coverage metrics are insufficient for safety-critical code.

### 2. Mutation Testing Finds Safety Gaps
**Discovery**: Mutation testing immediately found critical gaps that code coverage would miss.

**Lesson**: Mutation testing is essential for safety-critical modules like AST validation.

### 3. TDD Alone Is Insufficient
**Discovery**: Traditional TDD (write test, make it pass) focuses on happy paths.

**Lesson**: Safety-critical code requires **adversarial TDD** - write tests that try to break validation.

### 4. Early Analysis Drives Action
**Discovery**: With only 12% of mutants tested, we already have actionable findings.

**Lesson**: Don't wait for complete baseline - analyze early results and plan Phase 3.

---

**Status:** 🔍 PRELIMINARY ANALYSIS - Based on 8 of 66 AST mutants
**Next Update:** After AST baseline completes (~15-25 min)
**Impact:** HIGH - Safety-critical validation gaps identified
**Action Required:** YES - Write negative test cases in Phase 3

---

**Generated with:** Claude Code
**Methodology:** EXTREME TDD + Five Whys
**Sprint:** 29 - Mutation Testing Full Coverage
**Phase:** 1 (BASELINE) → 2 (ANALYZE) transition
