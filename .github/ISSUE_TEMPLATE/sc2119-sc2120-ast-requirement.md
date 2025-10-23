---
name: SC2119/SC2120 - AST-Based Function Analysis Required
about: Enable SC2119/SC2120 linter rules with proper AST-based function call analysis
title: '[ENHANCEMENT] Implement AST-based function analysis for SC2119/SC2120'
labels: enhancement, linter, ast-required, deferred-v7.0
assignees: ''
---

## Background

**Current Status**: SC2119 and SC2120 are implemented but disabled due to false positives.

**Coverage Impact**: These are the ONLY 2 rules preventing 100% ShellCheck SC2xxx coverage (currently at 99.4%, 323/325 rules).

## Problem Description

### SC2119: Use foo "$@" if function's $1 should mean script's $1

**What it detects**: Functions called with arguments that don't reference positional parameters.

**Implementation**: `rash/src/linter/rules/sc2119.rs`
- ✅ Fully implemented with regex-based function analysis
- ✅ 10 comprehensive tests
- ❌ **5 tests fail** when enabled (false positives)

**Example False Positive**:
```bash
my_func() { echo "$@"; }
my_func "arg1" "arg2"
```
**Expected**: No warning (function uses `$@`)
**Actual**: Warning triggered incorrectly

### SC2120: foo references arguments, but none are ever passed

**What it detects**: Functions that use positional parameters but are never called with arguments.

**Implementation**: `rash/src/linter/rules/sc2120.rs`
- ✅ Fully implemented with regex-based function analysis
- ✅ 10 comprehensive tests
- ❌ **7 tests fail** when enabled (false positives)

**Example False Positive**:
```bash
my_func() { echo "hello $1"; }
my_func "world"
```
**Expected**: No warning (arguments passed)
**Actual**: Warning triggered incorrectly

## Root Cause

**Regex-based analysis limitations**:
1. Cannot accurately track function call sites across the script
2. Cannot handle complex argument passing patterns
3. Cannot distinguish between function calls in different scopes
4. Misses function calls through variables or indirection

**What's needed**: Proper AST (Abstract Syntax Tree) based analysis that:
- Tracks function definitions with parameter usage
- Identifies all function call sites
- Matches calls to definitions
- Handles scoping correctly

## Current Workaround

Rules are commented out in `rash/src/linter/rules/mod.rs`:

```rust
// pub mod sc2119;  // TODO: Requires AST parsing for proper function analysis (has false positives)
// pub mod sc2120;  // TODO: Requires AST parsing for proper function analysis (has false positives)
```

And in `lint_shell()`:
```rust
// result.merge(sc2119::check(source));  // Deferred: False positives without AST
// result.merge(sc2120::check(source));  // Deferred: False positives without AST
```

## Acceptance Criteria

- [ ] Implement proper bash AST parser (or integrate existing parser)
- [ ] Refactor SC2119 to use AST-based function call tracking
- [ ] Refactor SC2120 to use AST-based function call tracking
- [ ] All 20 tests pass (10 per rule)
- [ ] No false positives on real-world shell scripts
- [ ] Enable rules in `mod.rs` (uncomment)
- [ ] Achieve 100% ShellCheck SC2xxx coverage (325/325)

## Technical Approach

### Option 1: Extend Existing bash_parser Module
- Leverage `rash/src/bash_parser/` infrastructure
- Add function call tracking to semantic analysis
- Build symbol table for function definitions

### Option 2: Integrate External Parser
- Consider `bash_parser` crate or similar
- Build adapter layer for bashrs
- May have different AST representation

### Option 3: Hybrid Approach
- Keep regex for simple cases (fast path)
- Use AST for complex cases (accurate path)
- Fall back to AST when regex confidence is low

## Testing Strategy

### Current Tests (Must Pass)

**SC2119 Tests** (`rash/src/linter/rules/sc2119.rs`):
1. `test_sc2119_call_with_args_but_no_use` ✅
2. `test_sc2119_function_uses_args_ok` ❌ (false positive)
3. `test_sc2119_no_args_passed_ok` ✅
4. `test_sc2119_function_uses_at_ok` ❌ (false positive)
5. `test_sc2119_function_uses_star_ok` ❌ (false positive)
6. `test_sc2119_function_uses_numbered_param_ok` ❌ (false positive)
7. `test_sc2119_braces_in_param_ref` ❌ (false positive)
8. `test_sc2119_multiple_calls` ✅
9. `test_sc2119_nested_functions` ✅
10. `test_sc2119_function_undefined_ok` ✅

**SC2120 Tests** (`rash/src/linter/rules/sc2120.rs`):
1. `test_sc2120_uses_args_but_never_passed` ❌ (false positive)
2. `test_sc2120_args_passed_ok` ✅
3. `test_sc2120_no_args_used_ok` ✅
4. `test_sc2120_uses_at` ❌ (false positive)
5. `test_sc2120_uses_star` ❌ (false positive)
6. `test_sc2120_uses_numbered_param` ❌ (false positive)
7. `test_sc2120_braces_in_param_ref` ❌ (false positive)
8. `test_sc2120_multiple_functions` ❌ (false positive)
9. `test_sc2120_called_without_and_with_args` ✅
10. `test_sc2120_function_call_with_pipe` ❌ (false positive)

### Additional Tests Needed

- [ ] Complex function scoping (local, global)
- [ ] Indirect function calls (`$func_name arg`)
- [ ] Functions in sourced files
- [ ] Functions with same names in different scopes
- [ ] Recursive functions
- [ ] Property-based tests (100+ cases)

## Impact

**Coverage**: 99.4% → **100%** ShellCheck SC2xxx coverage
**Quality**: Zero false positives on function analysis
**Value**: Complete ShellCheck-equivalent linting

## Timeline

**Estimated Effort**: 2-3 weeks (10-15 dev days)
- Week 1: AST infrastructure (5 days)
- Week 2: SC2119/SC2120 refactor (3-5 days)
- Week 3: Testing & validation (2-3 days)

**Target Release**: v7.0.0 (AST-based linter milestone)

## Related Issues

- Sprint 117 findings: `docs/sprints/SPRINT-117-FINDINGS.md`
- ROADMAP: `ROADMAP.yaml` (v7.0+ AST infrastructure)

## References

- SC2119 implementation: `rash/src/linter/rules/sc2119.rs`
- SC2120 implementation: `rash/src/linter/rules/sc2120.rs`
- ShellCheck wiki: https://www.shellcheck.net/wiki/SC2119
- ShellCheck wiki: https://www.shellcheck.net/wiki/SC2120

## Sprint 117 Context

This issue was identified during Sprint 117 (ROADMAP audit). When attempting to enable these rules:
- Attempted activation on 2025-10-23
- 12 tests failed (5 SC2119 + 7 SC2120)
- **STOP THE LINE**: Immediately reverted per zero regressions policy
- Confirmed AST requirement documented

**Zero Defects Policy**: Rules will remain disabled until AST-based analysis is implemented and all tests pass.
