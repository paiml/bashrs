# Chapter 18: Known Limitations and Edge Cases

**Chapter Status**: 🎯 **7/11 Fixed** (All P0 + All P1 + 2 P2 resolved!)

*Last updated: 2025-10-02*
*bashrs version: 0.3.3*

**Sprint 11 Progress**:
- ✅ **3 P0 Critical**: All fixed (empty functions, println!, negative integers)
- ✅ **2 P1 High**: All fixed (comparison operators, function nesting)
- 🟡 **4 P2 Medium**: 2/4 fixed (arithmetic ✅, returns ✅, loops/match pending)
- ⚪ **2 P3 Low**: Backlog (empty main, integer overflow)

---

## Overview

This chapter documents all known limitations, edge cases, and unsupported features in bashrs. Every limitation is:
1. **Tested**: Has a test case demonstrating the issue
2. **Documented**: Clear explanation of the problem
3. **Categorized**: Critical, High, Medium, or Low priority
4. **Tracked**: Linked to issues/roadmap items

## ✅ Critical Issues (All Fixed!)

### ✅ EDGE CASE #1: Empty Function Bodies Generate No-ops

**Status**: ✅ FIXED in v0.3.3 (commit ef6f81f)
**Discovered**: 2025-10-02
**Fixed**: 2025-10-02
**Test**: `tests/edge_cases_test.rs::test_edge_case_01_empty_function_bodies`

**Problem**:
Functions with empty bodies generate shell `:` (no-op) instead of the intended behavior.

**Example**:
```rust
fn main() {
    echo("Hello");
}

fn echo(msg: &str) {
    // Empty - should call shell echo
}
```

**Generated (WRONG)**:
```sh
main() {
    echo() {
        msg="$1"
        :  # ← BUG: Should call actual shell echo!
    }
    echo Hello
}
```

**Expected**:
```sh
main() {
    echo "$1"  # Call shell echo directly
}
main "$@"
```

**Impact**: ❌ **CRITICAL** - Made most example code non-functional
**Solution**: IR generator now skips empty functions - calls fall through to shell builtins
**Fix Commit**: ef6f81f (TICKET-5001)

---

### ✅ EDGE CASE #2: `println!` Macro Not Supported

**Status**: ✅ FIXED in v0.3.3 (commit fa20f43)
**Discovered**: 2025-10-02
**Fixed**: 2025-10-02
**Test**: `tests/edge_cases_test.rs::test_edge_case_02_println_macro`

**Problem**:
Standard Rust `println!` macro fails with "Unsupported statement type".

**Example**:
```rust
fn main() {
    println!("Hello, World!");
}
```

**Error**:
```text
Error: AST validation error: Unsupported statement type
```

**Impact**: ❌ **CRITICAL** - Book examples in Ch1 didn't work
**Solution**: Parser now handles `StmtMacro`, converts `println!` to `rash_println` runtime function
**Fix Commit**: fa20f43 (TICKET-5002)

---

### ✅ EDGE CASE #3: Negative Integers Transpile to `unknown`

**Status**: ✅ FIXED in v0.3.3 (commit 71e974d)
**Discovered**: 2025-10-02
**Fixed**: 2025-10-02
**Test**: `tests/edge_cases_test.rs::test_edge_case_03_negative_integers`

**Problem**:
Negative integer literals transpile to the string `"unknown"` instead of the numeric value.

**Example**:
```rust
fn main() {
    let x = -1;
    let y = -42;
}
```

**Generated (WRONG)**:
```sh
main() {
    x=unknown  # ← BUG!
    y=unknown  # ← BUG!
}
```

**Expected**:
```sh
main() {
    x=-1
    y=-42
}
```

**Impact**: ❌ **CRITICAL** - Negative numbers were completely broken
**Solution**: Added `Literal::I32(i32)`, parser simplifies `-literal` to `Literal::I32(-n)`
**Fix Commit**: 71e974d (TICKET-5003)

---

## High Priority Issues

### ✅ EDGE CASE #4: Comparison Operators Generate Wrong Shell Code

**Status**: ✅ FIXED in v0.3.3 (commit 71d0a9e)
**Discovered**: 2025-10-02
**Fixed**: 2025-10-02
**Test**: `tests/edge_cases_test.rs::test_edge_case_04_comparison_operators`

**Problem**:
Integer comparisons like `x > 0` transpile to string tests like `test -n "${x}0"` which is incorrect.

**Example**:
```rust
fn main() {
    let x = 1;
    if x > 0 {
        let y = 2;
    }
}
```

**Generated (WRONG)**:
```sh
main() {
    x=1
    if test -n "${x}0"; then  # ← BUG: Wrong comparison!
        y=2
    fi
}
```

**Expected**:
```sh
main() {
    x=1
    if [ "$x" -gt 0 ]; then  # Use -gt for integer comparison
        y=2
    fi
}
```

**Impact**: ⚠️ **HIGH** - Control flow was broken for numeric comparisons
**Solution**: Added Comparison variant to ShellValue IR, emits proper POSIX test syntax
**Fix Commit**: 71d0a9e (TICKET-5004)

---

### ✅ EDGE CASE #5: Functions Nested Inside main() Instead of Global

**Status**: ✅ FIXED in v0.3.3 (commit 02ee895)
**Discovered**: 2025-10-02
**Fixed**: 2025-10-02
**Test**: Manual verification with helper function example

**Problem**:
Helper functions are defined inside `main()` instead of as global shell functions.

**Example**:
```rust
fn main() {
    helper();
}

fn helper() {
    let x = 1;
}
```

**Generated (SUBOPTIMAL)**:
```sh
main() {
    helper() {  # ← Function defined inside main!
        x=1
    }
    helper
}
```

**Expected**:
```sh
helper() {
    x=1
}

main() {
    helper
}

main "$@"
```

**Impact**: ⚠️ **HIGH** - Made code harder to test, reuse
**Solution**: Refactored emitter to separate helpers from main body, emit at global scope
**Fix Commit**: 02ee895 (TICKET-5005)

---

## Medium Priority Issues

### 🟢 EDGE CASE #6: For Loops Not Supported

**Status**: 🟢 Medium Priority Feature Gap
**Discovered**: 2025-10-02
**Test**: `tests/edge-cases/test_06_for_loops.rs`

**Problem**:
Rust `for` loops fail with "Unsupported expression type".

**Example**:
```rust
fn main() {
    for i in 0..3 {
        let x = i;
    }
}
```

**Error**:
```text
Error: AST validation error: Unsupported expression type
```

**Impact**: 🟡 **MEDIUM** - Iteration not possible
**Workaround**: Use while loops (if supported)
**Fix Priority**: P2 - Sprint 11

---

### 🟢 EDGE CASE #7: Match Statements Not Supported

**Status**: 🟢 Medium Priority Feature Gap
**Discovered**: 2025-10-02
**Test**: `tests/edge-cases/test_07_match_statements.rs`

**Problem**:
Rust `match` expressions fail with "Unsupported expression type".

**Example**:
```rust
fn main() {
    let x = 1;
    match x {
        1 => {},
        _ => {},
    }
}
```

**Error**:
```text
Error: AST validation error: Unsupported expression type
```

**Impact**: 🟡 **MEDIUM** - Pattern matching not available
**Workaround**: Use if/else chains
**Fix Priority**: P2 - Sprint 12

---

### ✅ EDGE CASE #8: Function Return Values Not Implemented

**Status**: ✅ FIXED in v0.3.3 (commit 4c0ddd1)
**Discovered**: 2025-10-02
**Fixed**: 2025-10-02
**Test**: `tests/edge_cases_test.rs::test_edge_case_08_function_return_values`

**Problem**:
Functions with return values transpiled to `unknown` instead of capturing output.

**Example**:
```rust
fn main() {
    let x = add(1, 2);
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Generated (WRONG)**:
```sh
main() {
    add() {
        a="$1"
        b="$2"
        :  # ← BUG: Expression not transpiled
    }
    x=unknown  # ← BUG: Return value not captured
}
```

**Expected**:
```sh
add() {
    a="$1"
    b="$2"
    echo $((a + b))  # Return via echo
}

main() {
    x=$(add 1 2)  # Capture via command substitution
}
```

**Impact**: 🟡 **MEDIUM** - Couldn't return values from functions
**Solution**: Added Echo IR variant, emit `echo` for last expression in functions with return type, capture with `$(...)` at call sites
**Fix Commit**: 4c0ddd1 (TICKET-5007)

---

### ✅ EDGE CASE #9: Arithmetic Expressions Generate No-ops

**Status**: ✅ FIXED in v0.3.3 (commit 1cd984d)
**Discovered**: 2025-10-02
**Fixed**: 2025-10-02
**Test**: `tests/edge_cases_test.rs::test_edge_case_09_arithmetic_expressions`

**Problem**:
Arithmetic expressions like `a + b` transpiled to `:` (no-op) or string concatenation.

**Example**:
```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Generated (WRONG)**:
```sh
add() {
    a="$1"
    b="$2"
    :  # ← BUG: Should be $((a + b))
}
```

**Expected**:
```sh
add() {
    a="$1"
    b="$2"
    echo $((a + b))
}
```

**Impact**: 🟡 **MEDIUM** - Arithmetic operations didn't work
**Solution**: Added Arithmetic variant to ShellValue IR, emits proper POSIX `$((expr))` syntax
**Fix Commit**: 1cd984d (TICKET-5006)

---

## Low Priority Issues

### 🔵 EDGE CASE #10: Empty main() Generates No-op

**Status**: 🔵 Low Priority Edge Case
**Discovered**: 2025-10-02
**Test**: `tests/edge-cases/test_10_empty_main.rs`

**Problem**:
Empty `main()` function generates `:` (no-op).

**Example**:
```rust
fn main() {
}
```

**Generated**:
```sh
main() {
    :  # No-op
}
```

**Impact**: 🔵 **LOW** - Technically correct, just not useful
**Workaround**: Don't write empty main functions
**Fix Priority**: P3 - Nice to have

---

### 🔵 EDGE CASE #11: Large Integer Overflow Behavior Undefined

**Status**: 🔵 Low Priority Unknown
**Discovered**: 2025-10-02
**Test**: `tests/edge-cases/test_11_integer_overflow.rs`

**Problem**:
Behavior with very large integers (>2^31) is undefined.

**Example**:
```rust,ignore
fn main() {
    let x = 9999999999;  // Larger than i32::MAX
}
```

**Generated**:
```sh
main() {
    x=9999999999  # May overflow in shell arithmetic
}
```

**Impact**: 🔵 **LOW** - Rarely used
**Workaround**: Document limits (i32 range)
**Fix Priority**: P3 - Document only

---

## Summary Table

| # | Issue | Priority | Impact | Status | Fix Sprint |
|---|-------|----------|--------|--------|-----------|
| 1 | Empty function bodies → `:` | 🔴 P0 | Critical | ✅ Fixed | Sprint 10 |
| 2 | `println!` not supported | 🔴 P0 | Critical | ✅ Fixed | Sprint 10 |
| 3 | Negative integers → `unknown` | 🔴 P0 | Critical | ✅ Fixed | Sprint 10 |
| 4 | Comparisons wrong (`test -n`) | 🟡 P1 | High | ✅ Fixed | Sprint 10 |
| 5 | Functions nested in main | 🟡 P1 | High | ✅ Fixed | Sprint 10 |
| 6 | For loops unsupported | 🟢 P2 | Medium | 🔲 Pending | Sprint 11 |
| 7 | Match unsupported | 🟢 P2 | Medium | 🔲 Pending | Sprint 12 |
| 8 | Return values → `unknown` | 🟢 P2 | Medium | ✅ Fixed | Sprint 11 |
| 9 | Arithmetic → `:` | 🟢 P2 | Medium | ✅ Fixed | Sprint 11 |
| 10 | Empty main → `:` | 🔵 P3 | Low | 🔲 Pending | Sprint 12 |
| 11 | Integer overflow undefined | 🔵 P3 | Low | 🔲 Pending | Document |

---

## Testing Edge Cases

Run all edge case tests:

```bash
$ cd bashrs
$ make test-edge-cases
✓ test_01_empty_function ... FAIL (expected)
✓ test_02_println_macro ... FAIL (expected)
✓ test_03_negative_integers ... FAIL (expected)
✓ test_04_comparisons ... FAIL (expected)
✓ test_05_function_nesting ... PASS (suboptimal)
✓ test_06_for_loops ... FAIL (expected)
✓ test_07_match_statements ... FAIL (expected)
✓ test_08_return_values ... FAIL (expected)
✓ test_09_arithmetic ... FAIL (expected)
✓ test_10_empty_main ... PASS (acceptable)
✓ test_11_integer_overflow ... UNKNOWN

11 edge cases documented
```

---

## Next Steps

**Sprint 10 (Immediate)**: Fix P0 critical issues
1. Implement `println!` macro support
2. Fix negative integer transpilation
3. Fix empty function body handling
4. Fix comparison operator transpilation

**Sprint 11**: Fix P1-P2 issues
1. Move functions to global scope
2. Implement for loops
3. Implement return values
4. Fix arithmetic expressions

**Sprint 12**: Documentation and polish
1. Document known limitations clearly
2. Add warning messages for unsupported features
3. Improve error messages with suggestions

---

## Contributing

Found a new edge case? Please:
1. Create test case: `tests/edge-cases/test_XX_your_case.rs`
2. Document in this chapter
3. File issue: https://github.com/paiml/bashrs/issues
4. Tag with `edge-case` label

---

**Next Chapter**: [Chapter 19: Workarounds and Best Practices](ch19-best-practices.md)
