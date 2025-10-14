# Sprint 28 Specification - Complete Missing Stdlib Functions

**Priority:** P2_MEDIUM  
**Duration:** 2-3 hours  
**Philosophy:** 自働化 (Jidoka) - Build quality in through EXTREME TDD  
**Status:** PENDING

---

## Overview

Sprint 28 completes the implementation of stdlib functions that are **already declared** but not yet implemented in the emitter. This sprint follows the proven EXTREME TDD pattern from Sprint 27.

### Scope: Complete Missing Functions

**Missing Functions (Already Declared):**
1. `string_split()` - Split string by delimiter
2. `array_len()` - Get array length  
3. `array_join()` - Join array elements into string

These functions are already listed in `is_stdlib_function()` but lack implementation in the POSIX emitter.

---

## Current State Analysis

### Already Implemented (13 functions)
- ✅ `string_trim` - Remove whitespace
- ✅ `string_contains` - Substring check
- ✅ `string_len` - String length
- ✅ `string_replace` - Replace substring
- ✅ `string_to_upper` - Uppercase conversion
- ✅ `string_to_lower` - Lowercase conversion
- ✅ `fs_exists` - File/directory exists
- ✅ `fs_read_file` - Read file contents
- ✅ `fs_write_file` - Write to file
- ✅ `fs_copy` - Copy file
- ✅ `fs_remove` - Remove file
- ✅ `fs_is_file` - Check if regular file
- ✅ `fs_is_dir` - Check if directory

### Sprint 27 Functions (7 functions)
- ✅ `env()` - Environment variable access
- ✅ `env_var_or()` - Environment variable with default
- ✅ `arg()` - Positional argument access
- ✅ `args()` - All arguments access
- ✅ `arg_count()` - Argument count
- ✅ `exit_code()` - Exit code access

### Missing Implementations (3 functions) - SPRINT 28 SCOPE
- ❌ `string_split` - NOT YET IMPLEMENTED
- ❌ `array_len` - NOT YET IMPLEMENTED
- ❌ `array_join` - NOT YET IMPLEMENTED

---

## Sprint 28 Goals

### Primary Objective
Implement the 3 missing stdlib functions using EXTREME TDD methodology.

### Success Criteria
- ✅ All 3 functions implemented and tested
- ✅ POSIX-compliant shell code generated
- ✅ 100% test pass rate maintained
- ✅ EXTREME TDD methodology followed (RED-GREEN-REFACTOR)
- ✅ Completed within 2-3 hours

---

## Technical Design

### Function 1: string_split(str, delimiter)

**Purpose:** Split a string by delimiter, return as newline-separated output

**Rust Signature:**
```rust
fn string_split(text: &str, delimiter: &str) -> String
```

**POSIX Shell Implementation:**
```bash
rash_string_split() {
    text="$1"
    delimiter="$2"
    # Use parameter expansion and printf for POSIX compliance
    # Replace delimiter with newline
    printf '%s\n' "$text" | tr "$delimiter" '\n'
}
```

**Example Usage:**
```rust
let parts = string_split("a,b,c", ",");
// Shell: parts=$(rash_string_split "a,b,c" ",")
// Output: "a\nb\nc"
```

**Edge Cases:**
- Empty string → empty output
- Empty delimiter → error or character split
- Delimiter not found → original string
- Multiple consecutive delimiters → empty lines

---

### Function 2: array_len(array_string)

**Purpose:** Count elements in newline-separated array string

**Rust Signature:**
```rust
fn array_len(array: &str) -> i32
```

**POSIX Shell Implementation:**
```bash
rash_array_len() {
    array="$1"
    # Count non-empty lines
    if [ -z "$array" ]; then
        printf '0'
    else
        printf '%s\n' "$array" | wc -l | tr -d ' '
    fi
}
```

**Example Usage:**
```rust
let count = array_len("a\nb\nc");
// Shell: count=$(rash_array_len "a\nb\nc")
// Output: 3
```

**Edge Cases:**
- Empty string → 0
- Single element → 1
- Trailing newline handling

---

### Function 3: array_join(array_string, separator)

**Purpose:** Join newline-separated array elements with separator

**Rust Signature:**
```rust
fn array_join(array: &str, separator: &str) -> String
```

**POSIX Shell Implementation:**
```bash
rash_array_join() {
    array="$1"
    separator="$2"
    
    # Read lines and join with separator
    first=1
    result=""
    while IFS= read -r line; do
        if [ "$first" = 1 ]; then
            result="$line"
            first=0
        else
            result="${result}${separator}${line}"
        fi
    done <<EOF
$array
EOF
    printf '%s' "$result"
}
```

**Example Usage:**
```rust
let joined = array_join("a\nb\nc", ",");
// Shell: joined=$(rash_array_join "a\nb\nc" ",")
// Output: "a,b,c"
```

**Edge Cases:**
- Empty array → empty string
- Single element → element (no separator)
- Empty separator → concatenation

---

## EXTREME TDD Plan

### RED Phase (1 hour)

**Tasks:**
1. Write failing tests for all 3 functions (stdlib recognition + implementation)
2. Add panic stubs in emitter to make tests compile but fail
3. Verify all tests fail with expected messages

**Expected Tests:**
- 6 stdlib tests (2 per function: recognition + metadata)
- 6 implementation tests (2 per function: basic + edge case)
- **Total: ~12 new tests**

**Files to Modify:**
- `rash/src/stdlib.rs` - Add tests for function recognition
- `rash/src/emitter/tests.rs` - Add tests for shell generation
- `rash/src/emitter/posix.rs` - Add panic stubs

---

### GREEN Phase (1 hour)

**Tasks:**
1. Implement `write_string_split_function()` in emitter
2. Implement `write_array_len_function()` in emitter
3. Implement `write_array_join_function()` in emitter
4. Update `write_runtime()` to include new functions
5. Verify all tests pass

**Files to Modify:**
- `rash/src/emitter/posix.rs` - Implement 3 new shell function generators
- Update `write_runtime()` method to call new generators

---

### REFACTOR Phase (Optional, 0.5 hours)

**Potential Improvements:**
- Optimize shell implementations if needed
- Add property-based tests
- Improve error messages
- Documentation updates

---

## Expected Test Growth

**Before Sprint 28:** 845 tests
**After Sprint 28:** ~857 tests (+12)

**Test Breakdown:**
- Stdlib recognition: +6 tests
- Shell generation: +6 tests

---

## Risk Assessment

### Low Risk Items
- ✅ Functions already declared in stdlib.rs
- ✅ Pattern established by previous 13 functions
- ✅ POSIX shell techniques well-understood
- ✅ EXTREME TDD reduces implementation risk

### Medium Risk Items
- ⚠️ Array handling in POSIX shell (newline-separated strings)
- ⚠️ Edge cases with empty strings/arrays
- ⚠️ Delimiter escaping in string_split

### Mitigation Strategies
- Write comprehensive edge case tests
- Test with real POSIX shells (dash, ash)
- Follow existing string function patterns
- Use shellcheck validation

---

## Dependencies

**Required:**
- ✅ Sprint 27 complete (no dependencies on new features)
- ✅ Existing stdlib infrastructure
- ✅ POSIX emitter framework

**No Blocking Issues:** All infrastructure in place

---

## Out of Scope

**Deferred to Future Sprints:**
- Advanced array operations (push, pop, slice)
- Multi-dimensional arrays
- Associative arrays (bash-specific)
- String split with regex
- Performance optimizations

---

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- Write tests before implementation (RED phase)
- Verify quality at each step
- No shortcuts, proper TDD cycle

### 反省 (Hansei) - Reflection
- Learn from Sprint 27's patterns
- Apply lessons from previous stdlib implementations
- Continuous improvement mindset

### 改善 (Kaizen) - Continuous Improvement
- Each function builds on previous learnings
- Refine shell implementation techniques
- Improve test coverage strategies

### 現地現物 (Genchi Genbutsu) - Direct Observation
- Test against real POSIX shells
- Validate with shellcheck
- Verify actual behavior, not assumptions

---

## Timeline

**Total Duration:** 2-3 hours

| Phase | Duration | Tasks |
|-------|----------|-------|
| RED | 1.0 hour | Write 12 failing tests, add panic stubs |
| GREEN | 1.0 hour | Implement 3 shell functions |
| REFACTOR | 0.5 hour | Optional improvements |
| Buffer | 0.5 hour | Contingency for issues |

---

## Success Metrics

### Quality Gates
- ✅ 100% test pass rate (857/857)
- ✅ Zero compilation errors
- ✅ Zero clippy warnings
- ✅ All functions shellcheck-compliant
- ✅ EXTREME TDD cycle followed

### Deliverables
- ✅ 3 new stdlib function implementations
- ✅ 12 new tests
- ✅ Updated emitter with new functions
- ✅ Specification document (this file)
- ✅ Completion report

---

## Files to Modify

### Phase: RED
1. `rash/src/stdlib.rs` - Add 6 recognition tests
2. `rash/src/emitter/tests.rs` - Add 6 implementation tests
3. `rash/src/emitter/posix.rs` - Add panic stubs

### Phase: GREEN
1. `rash/src/emitter/posix.rs` - Implement 3 new functions
   - `write_string_split_function()`
   - `write_array_len_function()`
   - `write_array_join_function()`
   - Update `write_runtime()` to include new functions

---

## Acceptance Criteria

**Sprint 28 is complete when:**
1. ✅ All 3 functions implemented
2. ✅ All 12 new tests passing
3. ✅ Total test count: 857/857 (100%)
4. ✅ POSIX-compliant shell code generated
5. ✅ Shellcheck validation passes
6. ✅ Zero compiler warnings
7. ✅ Specification and completion report created
8. ✅ ROADMAP updated

---

**Prepared by:** Claude Code  
**Methodology:** EXTREME TDD + Toyota Way Principles  
**Sprint:** 28 - Complete Missing Stdlib Functions  
**Date:** 2025-10-14
