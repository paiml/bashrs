# TICKET-095: exec() String Validation Rejects Valid Shell Commands

**Status**: 🟢 FIXED
**Priority**: P0 - CRITICAL (Blocks whisper.apr QA)
**Assignee**: Claude Code
**Created**: 2026-01-20
**GitHub Issue**: #95
**Blocking**: whisper.apr WAPR-PERF-004 QA script

## Problem Statement

The `validate_string_literal()` function in `rash/src/validation/pipeline.rs` incorrectly rejects valid shell commands passed to `exec()`. This is a **false positive** that blocks legitimate bashrs usage.

### Reproduction

```rust
// scripts/test.rs
#[bashrs::main]
fn main() {
    exec("ldd /usr/bin/foo | grep cuda");  // REJECTED: "Pipe operator detected"
    exec("cmd1 && cmd2");                   // REJECTED: "AND operator detected"
}
```

```bash
$ bashrs build scripts/test.rs -o test.sh
error: Validation error: Pipe operator detected in string literal: 'ldd /usr/bin/foo | grep'
```

### Root Cause

In `pipeline.rs:146-199`, the `validate_string_literal()` function checks ALL string literals for shell operators (`|`, `&&`, `||`, `;`), including strings that are **intentionally** shell commands passed to `exec()`.

The security checks were designed to prevent command injection in interpolated strings, but they incorrectly apply to `exec()` arguments where shell operators are the **expected behavior**.

```rust
// Line 156: This pattern incorrectly flags exec("cmd1 && cmd2")
("&& ", "AND operator detected in string literal"),

// Line 183-199: This logic incorrectly flags exec("cmd1 | cmd2")
if !is_formatting_string && s.contains("| ") { ... }
```

### Impact

- **Blocks**: All bashrs scripts that use pipes or logical operators in `exec()`
- **Severity**: P0 - Cannot build legitimate shell scripts
- **Affected**: whisper.apr, aprender, and any project using bashrs for scripting

## Success Criteria

- [ ] `exec("cmd1 | cmd2")` compiles successfully
- [ ] `exec("cmd1 && cmd2")` compiles successfully
- [ ] `exec("cmd1 || cmd2")` compiles successfully
- [ ] Security checks still apply to non-exec string literals
- [ ] Shellshock protection still active
- [ ] Command substitution `$(...)` in non-exec strings still flagged
- [ ] All existing tests pass
- [ ] New regression tests added
- [ ] Property tests for edge cases

## Proposed Fix

**Option A (Recommended):** Context-aware validation

Modify `validate_expr()` to track context and skip shell operator checks when inside an `exec()` call:

```rust
fn validate_function_call(&self, name: &str, args: &[Expr]) -> RashResult<()> {
    let is_exec_context = name == "exec";
    for arg in args {
        if is_exec_context {
            // Skip shell operator validation for exec() arguments
            self.validate_expr_in_exec_context(arg)?;
        } else {
            self.validate_expr(arg)?;
        }
    }
    Ok(())
}
```

**Option B:** Allowlist approach

Add exec-specific allowlist patterns:

```rust
fn validate_string_literal(&self, s: &str, context: ValidationContext) -> RashResult<()> {
    if context == ValidationContext::ExecArgument {
        // Only check for truly dangerous patterns like shellshock
        return self.validate_exec_command(s);
    }
    // ... existing validation
}
```

## Test Cases

```rust
#[test]
fn test_exec_with_pipe_allowed() {
    let source = r#"
        fn main() {
            exec("cat file | grep pattern");
        }
    "#;
    assert!(compile(source).is_ok());
}

#[test]
fn test_exec_with_and_allowed() {
    let source = r#"
        fn main() {
            exec("cmd1 && cmd2");
        }
    "#;
    assert!(compile(source).is_ok());
}

#[test]
fn test_non_exec_string_with_pipe_still_flagged() {
    let source = r#"
        fn main() {
            let x = "cat file | rm -rf /";  // NOT in exec - should flag
            echo(x);
        }
    "#;
    assert!(compile(source).is_err());
}
```

## Toyota Way Analysis

### Five Whys

1. **Why did bashrs reject the script?** → Validation error on pipe operator
2. **Why was pipe flagged?** → `validate_string_literal()` checks all strings
3. **Why check all strings?** → Security against command injection
4. **Why is this a false positive?** → `exec()` arguments ARE meant to be commands
5. **Root cause?** → **No context-awareness in validation - exec() should be exempt**

### Jidoka

This ticket follows "stop the line" - whisper.apr QA is blocked until fixed.

## References

- `rash/src/validation/pipeline.rs:126-223` - Bug location
- Issue #94 - Related fix for table formatting (partial solution)
- whisper.apr `scripts/perf_qa_2x_whisper_cpp.rs` - Blocked script
