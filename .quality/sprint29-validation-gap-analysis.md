# Sprint 29 - Validation Gap Analysis

**Date:** 2025-10-15
**Objective:** Identify what validation functions currently check vs. what they should check
**Approach:** Deep dive analysis to enhance production code quality

---

## Executive Summary

**Current State:** 45.5% kill rate (30/66 mutants caught)

**Root Cause:** Validation functions have **0% kill rate** (0/6 validation mutants caught)

**Analysis:** Validation functions exist but perform minimal checks, allowing many invalid cases to pass

**Recommendation:** Enhance validation to check for:
1. Null characters in all string literals (Pattern, variable names)
2. Invalid variable/function names (reserved words, special chars)
3. Array/Index/Try/Block expression validation (currently wildcarded)
4. Empty pattern validation
5. Duplicate function names
6. Undefined function calls

---

## Validation Functions Inventory

### 1. RestrictedAst::validate() (Lines 11-29)

**Current Checks:**
- ✅ Entry point exists in function list
- ✅ All functions validate successfully
- ✅ No recursion in call graph

**Missing Checks:**
- ❌ Duplicate function names
- ❌ Function name validity (null chars, reserved words)
- ❌ Undefined function calls (called but not defined)

**Mutation Results:**
- No direct mutants (delegates to Function::validate)

**Enhancement Needed:** YES - Add duplicate name check, undefined call check

---

### 2. Function::validate() (Lines 98-107)

**Current Checks:**
- ✅ All statements in body validate

**Missing Checks:**
- ❌ Function name validity (null chars, special chars, reserved words)
- ❌ Parameter name validity
- ❌ Parameter name uniqueness (duplicate params)
- ❌ Return type validity (Type::is_allowed)
- ❌ Empty body validation (currently allowed)

**Mutation Results:**
- No direct mutants (delegates to Stmt::validate)

**Enhancement Needed:** YES - Add name/param validation

---

### 3. Type::is_allowed() (Lines 138-144)

**Current Checks:**
- ✅ All primitive types allowed (Void, Bool, U32, Str)
- ✅ Result type recursively checks ok_type and err_type
- ✅ Option type recursively checks inner_type

**Missing Checks:**
- None (this is well-implemented)

**Mutation Results:**
- **MISSED** (line 139): `replace Type::is_allowed -> bool with true`
  - Bypasses all type checking
  - **0 tests catch this!**
- **MISSED** (line 141): `replace && with ||`
  - Changes Result validation logic
  - **0 tests catch this!**

**Enhancement Needed:** NO - Add TESTS, not code changes

**Test Needed:**
```rust
#[test]
fn test_type_is_allowed_cannot_be_bypassed() {
    // This test will catch the "return true" mutant
    // Need a type that should NOT be allowed
    // BUT: All types ARE allowed currently!
    // INSIGHT: Type::is_allowed always returns true for defined types
}
```

**CRITICAL INSIGHT:** Type::is_allowed has no invalid types! It always returns true for all defined Type variants. This is why mutants survive - there's no way to test failure cases.

**Solution:** Either:
1. Accept that all types are allowed (current design)
2. Add invalid types (e.g., Type::Forbidden) to test rejection

---

### 4. Stmt::validate() (Lines 180-205)

**Current Checks:**
- ✅ Delegates to specialized validators
- ✅ validate_if_stmt checks condition and blocks
- ✅ validate_match_stmt checks scrutinee, patterns, guards, body
- ✅ validate_for_stmt checks bounded iteration
- ✅ validate_while_stmt checks bounded iteration
- ✅ Break/Continue allowed

**Missing Checks:**
- ❌ Variable name validity in Let { name, .. }
- ❌ Duplicate variable names in same scope

**Mutation Results:**
- **MISSED** (line 213): `replace validate_if_stmt -> Result<(), String> with Ok(())`
  - Bypasses all if statement validation
  - **0 tests catch this!**
- **MISSED** (line 222): `replace validate_match_stmt -> Result<(), String> with Ok(())`
  - Bypasses all match statement validation
  - **0 tests catch this!**
- **MISSED** (line 271): `replace validate_stmt_block -> Result<(), String> with Ok(())`
  - Bypasses all block validation
  - **0 tests catch this!**

**Enhancement Needed:** YES - Add variable name validation in Let

---

### 5. Expr::validate() (Lines 367-410)

**Current Checks:**
- ✅ Nesting depth > 30 rejected
- ✅ String literals with null chars rejected
- ✅ FunctionCall args validated
- ✅ Binary left/right validated
- ✅ Unary operand validated
- ✅ MethodCall receiver and args validated
- ✅ Range start/end validated

**Missing Checks:**
- ❌ Variable name validity (null chars, reserved words)
- ❌ Function name validity in FunctionCall
- ❌ Method name validity in MethodCall
- ❌ **Array, Index, Try, Block validation** (line 408: wildcard returns Ok(())!)

**Mutation Results:**
- **MISSED** (line 370): `replace > with >=`
  - Changes depth limit from 30 to 31
  - **0 tests catch this!**
- **MISSED** (line 370): `replace > with ==`
  - Only depth exactly > 30 triggers error
  - **0 tests catch this!**
- **MISSED** (line 383): `delete match arm Expr::Literal(_)`
  - Wildcard catches it
  - **UNTESTABLE** (Category D)
- **MISSED** (line 384): `delete match arm Expr::Variable(_)`
  - Wildcard catches it
  - **UNTESTABLE** (Category D)
- **MISSED** (line 385-408): All match arm deletions
  - Wildcard catches them
  - **UNTESTABLE** (Category D)

**Enhancement Needed:** YES
1. Add Array validation (validate all elements)
2. Add Index validation (validate object and index)
3. Add Try validation (validate inner expr)
4. Add Block validation (validate all stmts)
5. Add variable name checks
6. Add function name checks

---

### 6. Expr::nesting_depth() (Lines 412-427)

**Current Checks:**
- ✅ Binary: 1 + max(left, right)
- ✅ Unary: 1 + operand
- ✅ FunctionCall: 1 + max(args)
- ✅ MethodCall: 1 + max(receiver, args)
- ✅ Range: 1 + max(start, end)
- ❌ Array, Index, Try, Block: returns 0 (WRONG!)

**Missing Checks:**
- ❌ Array should return 1 + max(elements)
- ❌ Index should return 1 + max(object, index)
- ❌ Try should return 1 + expr
- ❌ Block should return max(stmts)

**Mutation Results:**
- **MISSED** (line 413): `replace nesting_depth -> usize with 0`
  - Returns 0 for all expressions
  - **0 tests catch this!**
- **MISSED** (line 413): `replace nesting_depth -> usize with 1`
  - Returns 1 for all expressions
  - **0 tests catch this!**
- **MISSED** (lines 414-424): All arithmetic mutations
  - Changes depth calculations
  - **0 tests catch this!**

**Enhancement Needed:** YES
1. Add Array depth: 1 + max(elements)
2. Add Index depth: 1 + max(object, index)
3. Add Try depth: 1 + expr
4. Add Block depth: need to define semantics

---

### 7. Pattern::validate() (Lines 526-542)

**Current Checks:**
- ✅ Tuple patterns validated recursively
- ✅ Struct patterns validated recursively
- ❌ Literal, Variable, Wildcard: returns Ok(()) **WITHOUT CHECKING**

**Missing Checks:**
- ❌ Pattern::Literal(Literal::Str(s)) should check for null chars
- ❌ Pattern::Variable(name) should check name validity
- ❌ Empty tuple validation (Tuple(vec![]) might be invalid)
- ❌ Empty struct fields validation

**Mutation Results:**
- **MISSED** (line 527): `replace Pattern::validate -> Result<(), String> with Ok(())`
  - Bypasses all pattern validation
  - **0 tests catch this!**

**Enhancement Needed:** YES - Add literal/variable validation

---

## Summary of Gaps

### Category 1: Critical Gaps (Enable Injection Attacks)

1. **Null characters in variable names** (not checked)
   ```rust
   let var_with_null = "\0bad";  // Should be rejected!
   ```

2. **Null characters in function names** (not checked)
   ```rust
   fn evil\0func() { }  // Should be rejected!
   ```

3. **Null characters in Pattern literals** (not checked)
   ```rust
   match s {
       "\0bad" => { }  // Should be rejected!
   }
   ```

---

### Category 2: Important Gaps (Prevent Invalid AST)

4. **Array/Index/Try/Block not validated** (line 408 wildcard)
   ```rust
   Expr::Array(vec![Expr::Literal(Literal::Str("\0bad"))]);  // Not validated!
   ```

5. **Nesting depth wrong for Array/Index/Try/Block**
   ```rust
   // Deep array nesting not counted
   Expr::Array(vec![Expr::Array(vec![...])])  // depth = 0, should be >0
   ```

6. **Duplicate function names** (not checked)
   ```rust
   functions: vec![
       Function { name: "main", .. },
       Function { name: "main", .. },  // Should be rejected!
   ]
   ```

7. **Undefined function calls** (not caught)
   ```rust
   FunctionCall { name: "undefined", .. }  // Should be rejected!
   ```

---

### Category 3: Design Gaps (Cannot Test Failure)

8. **Type::is_allowed always true** (no invalid types exist)
   - Cannot test rejection behavior
   - Mutant "return true" is equivalent to original

9. **Wildcard match arms** (Category D - Acceptable)
   - ~16-18 mutants untestable
   - Design trade-off for gradual feature addition

---

## Enhancement Strategy

### Phase 1: Critical Security Fixes (HIGH PRIORITY)

**Objective:** Prevent injection attacks via null characters

**Changes:**

1. **Pattern::validate - Check string literals**
   ```rust
   impl Pattern {
       pub fn validate(&self) -> Result<(), String> {
           match self {
               Pattern::Literal(Literal::Str(s)) => {
                   if s.contains('\0') {
                       return Err("Null characters not allowed in pattern literals".to_string());
                   }
                   Ok(())
               }
               Pattern::Literal(_) => Ok(()),
               Pattern::Variable(name) => Self::validate_identifier(name),
               Pattern::Wildcard => Ok(()),
               Pattern::Tuple(patterns) => {
                   for pattern in patterns {
                       pattern.validate()?;
                   }
                   Ok(())
               }
               Pattern::Struct { fields, .. } => {
                   for (field_name, pattern) in fields {
                       Self::validate_identifier(field_name)?;
                       pattern.validate()?;
                   }
                   Ok(())
               }
           }
       }

       fn validate_identifier(name: &str) -> Result<(), String> {
           if name.contains('\0') {
               return Err("Null characters not allowed in identifiers".to_string());
           }
           if name.is_empty() {
               return Err("Identifiers cannot be empty".to_string());
           }
           // TODO: Check for reserved words?
           Ok(())
       }
   }
   ```

2. **Expr::validate - Check variable and function names**
   ```rust
   // In Expr::validate match:
   Expr::Variable(name) => Self::validate_identifier(name),
   Expr::FunctionCall { name, args } => {
       Self::validate_identifier(name)?;
       for arg in args {
           arg.validate()?;
       }
       Ok(())
   }
   Expr::MethodCall { receiver, method, args } => {
       receiver.validate()?;
       Self::validate_identifier(method)?;
       for arg in args {
           arg.validate()?;
       }
       Ok(())
   }
   ```

3. **Stmt::validate - Check Let variable names**
   ```rust
   // In Stmt::validate:
   Stmt::Let { name, value } => {
       Self::validate_identifier(name)?;
       value.validate()
   }
   ```

4. **Function::validate - Check function and parameter names**
   ```rust
   impl Function {
       pub fn validate(&self) -> Result<(), String> {
           Self::validate_identifier(&self.name)?;

           // Check parameter names
           let mut param_names = std::collections::HashSet::new();
           for param in &self.params {
               Self::validate_identifier(&param.name)?;
               if !param_names.insert(&param.name) {
                   return Err(format!("Duplicate parameter name: {}", param.name));
               }
           }

           // Validate all statements
           for stmt in &self.body {
               stmt.validate()?;
           }

           Ok(())
       }

       fn validate_identifier(name: &str) -> Result<(), String> {
           if name.contains('\0') {
               return Err("Null characters not allowed in identifiers".to_string());
           }
           if name.is_empty() {
               return Err("Identifiers cannot be empty".to_string());
           }
           Ok(())
       }
   }
   ```

**Expected Impact:** Catch 6+ validation bypass mutants

---

### Phase 2: Complete Expression Validation (MEDIUM PRIORITY)

**Objective:** Validate Array, Index, Try, Block expressions

**Changes:**

```rust
// In Expr::validate, replace wildcard with:
Expr::Array(elements) => {
    for element in elements {
        element.validate()?;
    }
    Ok(())
}
Expr::Index { object, index } => {
    object.validate()?;
    index.validate()
}
Expr::Try { expr } => expr.validate(),
Expr::Block(stmts) => {
    for stmt in stmts {
        stmt.validate()?;
    }
    Ok(())
}
```

**Expected Impact:** Catch 4 match arm deletion mutants (if wildcard removed)

**Note:** Wildcard removal will make tests **required** for these cases

---

### Phase 3: Fix Nesting Depth (MEDIUM PRIORITY)

**Objective:** Correctly calculate depth for all expression types

**Changes:**

```rust
// In Expr::nesting_depth, replace wildcard with:
Expr::Array(elements) => {
    1 + elements.iter().map(|e| e.nesting_depth()).max().unwrap_or(0)
}
Expr::Index { object, index } => {
    1 + object.nesting_depth().max(index.nesting_depth())
}
Expr::Try { expr } => {
    1 + expr.nesting_depth()
}
Expr::Block(stmts) => {
    // Blocks don't add nesting, but their contents do
    stmts.iter()
        .filter_map(|s| match s {
            Stmt::Expr(e) | Stmt::Return(Some(e)) => Some(e.nesting_depth()),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}
Expr::Literal(_) | Expr::Variable(_) => 0,
```

**Expected Impact:** Catch nesting depth mutants (need tests at depth=30/31)

---

### Phase 4: AST-Level Validation (LOW PRIORITY)

**Objective:** Check function name uniqueness and undefined calls

**Changes:**

```rust
impl RestrictedAst {
    pub fn validate(&self) -> Result<(), String> {
        // Check for duplicate function names
        let mut function_names = std::collections::HashSet::new();
        for function in &self.functions {
            if !function_names.insert(&function.name) {
                return Err(format!("Duplicate function name: {}", function.name));
            }
        }

        // Check for entry point
        if !self.functions.iter().any(|f| f.name == self.entry_point) {
            return Err(format!(
                "Entry point function '{}' not found",
                self.entry_point
            ));
        }

        // Validate each function
        for function in &self.functions {
            function.validate()?;
        }

        // Check for undefined function calls
        let all_calls = self.collect_all_function_calls();
        for call in all_calls {
            if !function_names.contains(&call) {
                return Err(format!("Call to undefined function: {}", call));
            }
        }

        // Check for recursion
        self.check_no_recursion()?;

        Ok(())
    }

    fn collect_all_function_calls(&self) -> Vec<String> {
        let mut calls = Vec::new();
        for function in &self.functions {
            function.collect_function_calls(&mut calls);
        }
        calls
    }
}
```

**Expected Impact:** Better AST validation, but fewer mutation testing benefits

---

## Expected Kill Rate Improvements

### Current State
- **Baseline:** 45.5% (30/66 caught)
- **Validation mutants:** 0% (0/6 caught)
- **Other mutants:** ~50% (30/60 caught)

### After Phase 1 (Critical Security)
- **Validation mutants:** ~83% (5/6 caught)
  - Pattern::validate bypass: CAUGHT (has rejection tests)
  - validate_if_stmt bypass: CAUGHT (has rejection tests)
  - validate_match_stmt bypass: CAUGHT (has rejection tests)
  - validate_stmt_block bypass: CAUGHT (has rejection tests)
  - Type::is_allowed bypass: MISSED (no invalid types)
  - Type::is_allowed && -> ||: CAUGHT (has Result tests)
- **Total:** ~53% (35/66 caught)

### After Phase 2 (Expression Validation)
- **Match arm deletions:** 4-8 more caught (Array, Index, Try, Block)
- **Total:** ~59% (39/66 caught)

### After Phase 3 (Nesting Depth)
- **Nesting depth mutants:** 6-8 more caught
- **Total:** ~68% (45/66 caught)

### Final Expected
- **Testable mutants:** ~90% catch rate
- **Total (with Category D):** ~68% (45/66 caught)
- **Improvement:** 45.5% → 68% (+22.5 percentage points)

---

## Recommended Implementation Order

### Session 1 (This Session): Phase 1 - Critical Security
1. Add validate_identifier helper function
2. Enhance Pattern::validate (string literals, variable names)
3. Enhance Expr::validate (variable, function, method names)
4. Enhance Stmt::validate (Let variable names)
5. Enhance Function::validate (function, parameter names)
6. **Estimated time:** 2-3 hours

### Session 2: Phase 2 - Expression Validation
1. Add Array validation
2. Add Index validation
3. Add Try validation
4. Add Block validation
5. Remove wildcard (force explicit handling)
6. **Estimated time:** 1-2 hours

### Session 3: Phase 3 - Nesting Depth
1. Fix Array depth calculation
2. Fix Index depth calculation
3. Fix Try depth calculation
4. Fix Block depth calculation
5. Add tests at boundary (depth=30/31)
6. **Estimated time:** 1-2 hours

---

## Test Strategy

### Phase 1 Tests (Write After Implementation)

```rust
#[test]
fn test_pattern_literal_string_rejects_null_char() {
    let pattern = Pattern::Literal(Literal::Str("\0bad".to_string()));
    assert!(pattern.validate().is_err());
}

#[test]
fn test_pattern_variable_rejects_null_char() {
    let pattern = Pattern::Variable("\0bad".to_string());
    assert!(pattern.validate().is_err());
}

#[test]
fn test_expr_variable_rejects_null_char() {
    let expr = Expr::Variable("\0bad".to_string());
    assert!(expr.validate().is_err());
}

#[test]
fn test_function_call_rejects_invalid_name() {
    let expr = Expr::FunctionCall {
        name: "\0bad".to_string(),
        args: vec![],
    };
    assert!(expr.validate().is_err());
}

#[test]
fn test_let_stmt_rejects_invalid_variable_name() {
    let stmt = Stmt::Let {
        name: "\0bad".to_string(),
        value: Expr::Literal(Literal::U32(42)),
    };
    assert!(stmt.validate().is_err());
}

#[test]
fn test_function_rejects_invalid_name() {
    let func = Function {
        name: "\0bad".to_string(),
        params: vec![],
        return_type: Type::Void,
        body: vec![],
    };
    assert!(func.validate().is_err());
}

#[test]
fn test_function_rejects_duplicate_parameters() {
    let func = Function {
        name: "test".to_string(),
        params: vec![
            Parameter { name: "x".to_string(), param_type: Type::U32 },
            Parameter { name: "x".to_string(), param_type: Type::U32 },
        ],
        return_type: Type::Void,
        body: vec![],
    };
    assert!(func.validate().is_err());
}
```

---

## Success Criteria

### Minimum Success (After Phase 1)
- [ ] All identifier validation checks null chars
- [ ] Pattern::validate checks string literals
- [ ] Function::validate checks parameter uniqueness
- [ ] Tests added for all new validation
- [ ] Kill rate ≥53% (35/66 mutants)

### Target Success (After Phase 2)
- [ ] Array/Index/Try/Block validated
- [ ] Wildcard removed from Expr::validate
- [ ] Kill rate ≥59% (39/66 mutants)

### Stretch Success (After Phase 3)
- [ ] Nesting depth correct for all types
- [ ] Kill rate ≥68% (45/66 mutants)
- [ ] 90% of testable mutants caught

---

**Generated by:** Claude Code
**Sprint:** 29 - Mutation Testing Full Coverage
**Analysis Type:** Validation Gap Analysis
**Approach:** Deep dive enhancement
**Estimated Total Time:** 5-8 hours (3 sessions)
**Expected Improvement:** 45.5% → 68% kill rate
