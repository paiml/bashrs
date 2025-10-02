# Sprint 10 Plan: Fix P0 Critical Edge Cases

**Sprint**: 10
**Date**: 2025-10-02
**Status**: 📋 PLANNED
**Duration**: 3-4 hours
**Prerequisites**: Sprints 1-9 complete, edge cases documented

---

## Executive Summary

Sprint 10 focuses on fixing **3 critical P0 edge cases** discovered during book documentation. These are blockers for basic functionality and must be fixed before the book can have working examples.

---

## Discovered Issues

### 🔴 TICKET-5001: Empty Function Bodies Generate No-ops

**Priority**: P0 - Critical
**Impact**: Makes most example code non-functional
**Root Cause**: Empty `Vec<Stmt>` → `ShellIR::Noop` → `:` emitted

**Example**:
```rust
fn echo(msg: &str) {
    // Empty - should call shell echo
}
```

**Current Output**:
```sh
echo() {
    msg="$1"
    :  # ← BUG!
}
```

**Root Cause Analysis (Five Whys)**:
1. **Why does it emit `:`?** → Because `emit_noop()` generates `:` for `ShellIR::Noop`
2. **Why is it `ShellIR::Noop`?** → Because `convert_statements()` creates Noop for empty bodies
3. **Why empty bodies?** → Parser converts empty Rust fn bodies to empty `Vec<Stmt>`
4. **Why is that wrong?** → Rash assumes empty fn = "call builtin", not "do nothing"
5. **Root cause**: Semantic mismatch - empty fn should mean "delegate to shell builtin"

**Fix Strategy**:
1. Add flag to `Function` AST: `is_builtin_wrapper: bool`
2. Parser detects empty bodies → sets flag
3. IR generator: if `is_builtin_wrapper`, generate `ShellCommand` not `Noop`
4. Emitter: emit actual shell command call

**Tests to Add**:
- `test_empty_function_calls_builtin.rs`
- `test_echo_wrapper.rs`
- `test_empty_with_params.rs`

**Files to Modify**:
- `rash/src/ast/restricted.rs` - Add `is_builtin: bool` to `Function`
- `rash/src/services/parser.rs` - Detect empty bodies (line 107)
- `rash/src/ir/mod.rs` - Handle builtin wrappers
- `rash/src/emitter/posix.rs` - Emit shell commands

---

### 🔴 TICKET-5002: `println!` Macro Not Supported

**Priority**: P0 - Critical
**Impact**: Book Chapter 1 examples don't work
**Root Cause**: Macro expansion not handled, treated as unsupported statement

**Example**:
```rust
fn main() {
    println!("Hello, World!");
}
```

**Current Error**:
```
Error: AST validation error: Unsupported statement type
```

**Root Cause Analysis (Five Whys)**:
1. **Why unsupported?** → `convert_stmt()` doesn't handle `SynStmt::Macro`
2. **Why not handled?** → No macro case in match statement (line 189)
3. **Why no macro support?** → Original design avoided macros
4. **Why avoid macros?** → Complexity, but `println!` is essential
5. **Root cause**: Design decision to skip macros, but `println!` is core Rust idiom

**Fix Strategy**:
1. Add `SynStmt::Macro` case to `convert_stmt()`
2. Pattern match on `println!` specifically
3. Convert to `Stmt::FunctionCall` with special "rash_println" function
4. Emitter generates `printf '%s\n' "$1"` wrapper

**Alternative**: Support `print!` and `println!` as "known macros" list

**Tests to Add**:
- `test_println_basic.rs`
- `test_println_with_variable.rs`
- `test_println_multiple.rs`
- `test_print_without_newline.rs` (if supported)

**Files to Modify**:
- `rash/src/services/parser.rs` - Add macro handling (line 189)
- `rash/src/ast/restricted.rs` - Possibly add `Stmt::Print` variant
- `rash/src/emitter/posix.rs` - Generate printf wrappers

---

### 🔴 TICKET-5003: Negative Integers Transpile to `unknown`

**Priority**: P0 - Critical
**Impact**: Negative numbers completely broken
**Root Cause**: Unary minus not handled in expression conversion

**Example**:
```rust
fn main() {
    let x = -1;
    let y = -42;
}
```

**Current Output**:
```sh
main() {
    x=unknown
    y=unknown
}
```

**Root Cause Analysis (Five Whys)**:
1. **Why `unknown`?** → `convert_expr()` returns `unknown` for unhandled cases
2. **Why unhandled?** → `-1` is `UnaryOp(Neg, Lit(1))`, not just `Lit(-1)`
3. **Why not Lit(-1)?** → Rust parser treats `-1` as unary negation applied to `1`
4. **Why not handled?** → `convert_unary_op()` doesn't handle Neg on literals
5. **Root cause**: Expression conversion doesn't simplify `-literal` to negative literal

**Fix Strategy**:
1. In `convert_expr()`, detect `UnaryOp(Neg, Lit(Int(n)))`
2. Simplify to `Lit(Int(-n))` directly
3. Or: Handle in IR generation, emit `-n` in shell

**Alternative**: Store sign in `Literal::Integer` AST node

**Tests to Add**:
- `test_negative_integer_literal.rs`
- `test_negative_in_expression.rs`
- `test_negative_zero.rs`
- `test_double_negation.rs` (`--x`)

**Files to Modify**:
- `rash/src/services/parser.rs` - Handle negative literals (line 350+)
- `rash/src/ast/restricted.rs` - Maybe change `Literal::Integer(u32)` to `i32`
- `rash/src/emitter/posix.rs` - Emit negative numbers correctly

---

## EXTREME TDD Process

### Phase 1: RED (Write Tests First)
**Duration**: 30 minutes

```bash
# Create test files
tests/edge-cases/test_01_empty_function.rs
tests/edge-cases/test_02_println_macro.rs
tests/edge-cases/test_03_negative_integers.rs

# Run tests - should all FAIL
cargo test edge_cases -- --nocapture
```

**Expected**: All 3 tests fail with documented error messages

### Phase 2: GREEN (Make Tests Pass)
**Duration**: 2 hours

**TICKET-5001** (1 hour):
1. Add `is_builtin: bool` to `Function` struct
2. Modify parser to detect empty bodies
3. Update IR generation for builtins
4. Update emitter to call shell commands
5. Run tests until passing

**TICKET-5002** (45 minutes):
1. Add `SynStmt::Macro` case to `convert_stmt()`
2. Parse `println!` macro arguments
3. Convert to function call AST
4. Generate printf wrapper in emitter
5. Run tests until passing

**TICKET-5003** (15 minutes):
1. Detect unary negation on integer literals
2. Simplify to negative literal in AST
3. Update emitter to handle negative numbers
4. Run tests until passing

### Phase 3: REFACTOR (Verify & Optimize)
**Duration**: 30 minutes

1. Run full test suite (539 + 3 new = 542 tests)
2. Verify 100% pass rate
3. Run coverage - should maintain >85%
4. Run ShellCheck on generated scripts
5. Check complexity with pmat
6. Update documentation

---

## Success Criteria

- ✅ All 542 tests passing (100% pass rate)
- ✅ 3 new edge case tests added and passing
- ✅ Book Chapter 1 examples work
- ✅ Coverage maintained >85%
- ✅ Complexity maintained <10
- ✅ ShellCheck 100% pass rate
- ✅ Documentation updated

---

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- ✅ Write tests before implementation (RED first)
- ✅ Fix root cause, not symptoms
- ✅ Verify with automated tests

### 反省 (Hansei) - Reflection
- ✅ Five Whys analysis for each issue
- ✅ Document root causes
- ✅ Learn from mistakes (no macros → need macros)

### 改善 (Kaizen) - Continuous Improvement
- ✅ Fix critical issues immediately (P0)
- ✅ Improve book examples iteratively
- ✅ Build on previous sprint learnings

### 現地現物 (Genchi Genbutsu) - Direct Observation
- ✅ Tested actual transpiler output
- ✅ Found real edge cases, not theoretical
- ✅ Verified with actual shell execution

---

## Risk Assessment

### Low Risk
- **TICKET-5003** (negative integers): Simple fix, well-understood
- **TICKET-5002** (println!): Common pattern, many examples

### Medium Risk
- **TICKET-5001** (empty functions): Requires AST changes, could affect many tests

### Mitigation
- Run full test suite after each ticket
- Commit after each green phase
- Roll back if >5% tests fail

---

## Files to Modify (Summary)

```
rash/src/ast/restricted.rs           # Add is_builtin flag
rash/src/services/parser.rs          # Handle macros, empty bodies, negation
rash/src/ir/mod.rs                   # Convert builtins to shell commands
rash/src/emitter/posix.rs            # Emit builtins, printf, negative numbers
tests/edge-cases/test_01_*.rs        # New test files
rash-book/src/ch01-hello-shell-tdd.md  # Update with working examples
```

**Estimated LOC**: +150 (code), +200 (tests), -50 (simplified)

---

## Next Steps After Sprint 10

1. **Update book**: Chapter 1 examples should work
2. **Rebuild book**: `mdbook build rash-book`
3. **Deploy to GitHub Pages**: Workflow should auto-deploy
4. **Sprint 11**: Fix P1 high-priority issues (comparisons, function nesting)
5. **Sprint 12**: Property tests for fixed edge cases

---

## References

- Edge cases documentation: `rash-book/src/ch18-limitations.md`
- Test status dashboard: `rash-book/src/test-status.md`
- Sprint 7-9 reports: `.quality/sprint*-complete.md`
- ROADMAP: `ROADMAP.md`

---

**Status**: Ready to begin Sprint 10
**Estimated Time**: 3-4 hours
**Blocking**: Book examples, MCP server work
**Priority**: P0 - Must fix before continuing
