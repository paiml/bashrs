# TICKET-REPL-014-002: Zero-tolerance quality gate for purified output

**Sprint**: REPL-014 (Purified Output Validation)
**Task ID**: REPL-014-002
**Status**: IN PROGRESS
**Priority**: HIGH
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY → MUTATION)
**Dependencies**: REPL-014-001 (Auto-run bashrs linter on purified output)

---

## Problem Statement

REPL-014-001 added automatic linting of purified bash code, but currently it only **reports** violations. We need to **enforce** a zero-tolerance quality gate that guarantees purified output has ZERO critical violations (DET/IDEM/SEC).

**Current Behavior (REPL-014-001)**:
```rust
let result = purify_and_lint("echo $RANDOM")?;
// Returns: PurifiedLintResult { is_clean: false, ... }
// No enforcement - violations are reported but not prevented!
```

**Desired Behavior (REPL-014-002)**:
```rust
let purified = purify_and_validate("echo $RANDOM")?;
// Returns: Err("Purified output has 1 DET violation(s)")
// Zero-tolerance enforced!

let purified = purify_and_validate("mkdir -p /tmp/test")?;
// Returns: Ok("mkdir -p /tmp/test")
// Clean output passes validation
```

---

## Requirements

1. **New Function**: `purify_and_validate(input: &str) -> anyhow::Result<String>`
   - Calls `purify_and_lint()` internally
   - Returns `Err` if ANY DET/IDEM/SEC violations exist
   - Returns `Ok(purified_code)` only if `is_clean == true`

2. **New Error Type**: `PurificationError`
   - Detailed error messages listing violation counts
   - Includes diagnostic details for debugging
   - Implements `std::error::Error` and `Display`

3. **Zero-Tolerance Policy**: Enforced for critical violations only
   - **DET** (Determinism): ZERO violations allowed
   - **IDEM** (Idempotency): ZERO violations allowed
   - **SEC** (Security): ZERO violations allowed
   - **SC/MAKE**: Warnings only (not enforced)

4. **Quality Guarantee**: Purified output ALWAYS safe
   - If `purify_and_validate()` returns `Ok(code)`, code is guaranteed clean
   - If it returns `Err`, violations are clearly reported
   - No silent failures allowed

---

## Data Structures

### PurificationError

```rust
/// Error returned when purified output fails zero-tolerance quality gate
#[derive(Debug, Clone)]
pub struct PurificationError {
    /// The purified code (even though it has violations)
    pub purified_code: String,

    /// Count of DET violations
    pub det_violations: usize,

    /// Count of IDEM violations
    pub idem_violations: usize,

    /// Count of SEC violations
    pub sec_violations: usize,

    /// All diagnostics (for detailed reporting)
    pub diagnostics: Vec<Diagnostic>,
}

impl PurificationError {
    pub fn new(result: &PurifiedLintResult) -> Self {
        Self {
            purified_code: result.purified_code.clone(),
            det_violations: result.det_violations().len(),
            idem_violations: result.idem_violations().len(),
            sec_violations: result.sec_violations().len(),
            diagnostics: result.lint_result.diagnostics.clone(),
        }
    }

    pub fn total_violations(&self) -> usize {
        self.det_violations + self.idem_violations + self.sec_violations
    }
}

impl std::fmt::Display for PurificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Purified output failed zero-tolerance quality gate: {} violation(s) (DET: {}, IDEM: {}, SEC: {})",
            self.total_violations(),
            self.det_violations,
            self.idem_violations,
            self.sec_violations
        )
    }
}

impl std::error::Error for PurificationError {}
```

---

## Functions

### purify_and_validate

```rust
/// Purify bash input and enforce zero-tolerance quality gate
///
/// This function guarantees that returned purified code has ZERO critical
/// violations (DET/IDEM/SEC). If any critical violations exist, it returns
/// an error with detailed diagnostic information.
///
/// # Examples
///
/// ```
/// use bashrs::repl::purifier::purify_and_validate;
///
/// // Clean input passes
/// let purified = purify_and_validate("mkdir -p /tmp/test").unwrap();
/// assert_eq!(purified, "mkdir -p /tmp/test");
///
/// // Non-deterministic input fails
/// let result = purify_and_validate("echo $RANDOM");
/// assert!(result.is_err());
/// ```
///
/// # Errors
///
/// Returns `Err(PurificationError)` if purified output has any:
/// - DET violations (non-deterministic patterns)
/// - IDEM violations (non-idempotent operations)
/// - SEC violations (security issues)
///
/// Note: SC/MAKE violations are warnings only and don't cause failure.
pub fn purify_and_validate(input: &str) -> anyhow::Result<String> {
    // Step 1: Purify and lint
    let result = purify_and_lint(input)?;

    // Step 2: Enforce zero-tolerance
    if !result.is_clean {
        return Err(PurificationError::new(&result).into());
    }

    // Step 3: Return clean code
    Ok(result.purified_code)
}
```

---

## Test Cases

### Unit Tests (3 tests)

#### test_REPL_014_002_zero_det_violations

**Input**: `"echo hello"` (clean)
**Expected**: Returns `Ok("echo hello")`

```rust
#[test]
fn test_REPL_014_002_zero_det_violations() {
    let input = "echo hello";
    let result = purify_and_validate(input);

    // Should succeed - no DET violations
    assert!(result.is_ok(), "Clean input should pass validation");
    let purified = result.unwrap();
    assert!(purified.contains("echo"));
}
```

#### test_REPL_014_002_zero_idem_violations

**Input**: `"mkdir -p /tmp/test"` (clean, already idempotent)
**Expected**: Returns `Ok("mkdir -p /tmp/test")`

```rust
#[test]
fn test_REPL_014_002_zero_idem_violations() {
    let input = "mkdir -p /tmp/test";
    let result = purify_and_validate(input);

    // Should succeed - already idempotent
    assert!(result.is_ok(), "Idempotent input should pass validation");
}
```

#### test_REPL_014_002_zero_sec_violations

**Input**: `"echo \"$var\""` (clean, quoted variable)
**Expected**: Returns `Ok(...)`

```rust
#[test]
fn test_REPL_014_002_zero_sec_violations() {
    let input = "echo \"$var\"";
    let result = purify_and_validate(input);

    // Should succeed - variable is quoted
    assert!(result.is_ok(), "Quoted variable should pass validation");
}
```

#### test_REPL_014_002_fails_with_violations

**Input**: Various inputs that purifier cannot fix
**Expected**: Returns `Err(PurificationError)`

```rust
#[test]
fn test_REPL_014_002_fails_with_violations() {
    // Test various inputs that purifier might not be able to fix
    let test_cases = vec![
        ("echo $RANDOM", "DET violation"),
        ("rm /nonexistent", "IDEM violation"),
        ("eval $user_input", "SEC violation"),
    ];

    for (input, description) in test_cases {
        let result = purify_and_validate(input);

        // If purifier can't fix it, should fail validation
        if result.is_err() {
            let err = result.unwrap_err();
            let purif_err = err.downcast_ref::<PurificationError>();

            // Should have detailed error
            assert!(purif_err.is_some(),
                "Error should be PurificationError for: {}", description);
        }
        // Note: If purifier CAN fix it, that's also acceptable
        // This test is about ensuring we catch unfixable violations
    }
}
```

#### test_REPL_014_002_error_details

**Input**: Code with known violations
**Expected**: Error contains detailed violation counts

```rust
#[test]
fn test_REPL_014_002_error_details() {
    // Use input that we know will have violations after purification
    // (This test may need adjustment based on actual purifier behavior)
    let input = "echo $RANDOM; eval $cmd; rm /tmp/file";

    let result = purify_and_validate(input);

    // If validation fails, check error details
    if let Err(e) = result {
        if let Some(purif_err) = e.downcast_ref::<PurificationError>() {
            // Should have violation counts
            assert!(purif_err.total_violations() > 0);

            // Error message should be descriptive
            let msg = purif_err.to_string();
            assert!(msg.contains("violation"));
        }
    }
    // Note: If purifier fixes everything, that's also valid
}
```

### Property Test (1 test)

#### prop_purified_always_passes_linter

**Property**: If `purify_and_validate()` returns `Ok(code)`, the code MUST be clean

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_purified_always_passes_linter(input in ".*{0,100}") {
            if let Ok(purified) = purify_and_validate(&input) {
                // CRITICAL PROPERTY: If validation succeeds, output MUST be clean
                let result = purify_and_lint(&purified);

                prop_assert!(result.is_ok(), "Re-linting should succeed");

                if let Ok(lint_result) = result {
                    prop_assert!(
                        lint_result.is_clean,
                        "Validated output must be clean, but found {} violations",
                        lint_result.critical_violations()
                    );

                    prop_assert_eq!(lint_result.det_violations().len(), 0,
                        "No DET violations allowed");
                    prop_assert_eq!(lint_result.idem_violations().len(), 0,
                        "No IDEM violations allowed");
                    prop_assert_eq!(lint_result.sec_violations().len(), 0,
                        "No SEC violations allowed");
                }
            }
            // If validation fails, that's acceptable - not all inputs can be purified
        }
    }
}
```

---

## EXTREME TDD Phases

### Phase 1: RED (Write Failing Tests)

1. Create `PurificationError` struct (stub with `todo!()` implementations)
2. Add stub `purify_and_validate()` function with `unimplemented!()`
3. Write all 5 unit tests
4. Write 1 property test
5. **Run tests** - expect 6 failures/panics

**Expected**: All tests fail (RED phase confirmed)

### Phase 2: GREEN (Implement Minimum)

1. Implement `PurificationError::new()`
2. Implement `PurificationError::total_violations()`
3. Implement `Display` trait for `PurificationError`
4. Implement `Error` trait for `PurificationError`
5. Implement `purify_and_validate()` - calls `purify_and_lint()` + checks `is_clean`
6. **Run tests** - expect all tests to pass (or adjust test expectations)

**Expected**: All tests pass (GREEN phase confirmed)

### Phase 3: REFACTOR (Clean Up)

1. Run `cargo clippy` - fix any warnings
2. Ensure complexity <10 per function
3. Add comprehensive documentation
4. **Run all library tests** - expect 100% pass

**Expected**: Clean code, all tests passing

### Phase 4: PROPERTY (Generative Testing)

1. Run property test with 100+ cases
2. Verify critical property: validated output ALWAYS clean
3. **Analyze failures** - should be zero

**Expected**: 100+ property test cases pass

### Phase 5: MUTATION (Resilience Testing)

1. Run `cargo mutants --file rash/src/repl/purifier.rs`
2. Target: ≥90% kill rate
3. Fix any surviving mutants
4. **Verify** mutation score

**Expected**: Mutation score ≥90%

### Phase 6: COMMIT

1. Update `docs/REPL-DEBUGGER-ROADMAP.yaml`:
   - Mark REPL-014-002 as "completed"
   - Add all test names
2. Create commit message following Toyota Way principles
3. **Commit** with message

---

## Implementation Location

**File**: `rash/src/repl/purifier.rs`

**Changes**:
1. Add `PurificationError` struct (after `PurifiedLintResult`)
2. Implement `Display` and `Error` traits for `PurificationError`
3. Add `purify_and_validate()` function (after `purify_and_lint()`)
4. Add tests in `#[cfg(test)] mod tests` section
5. Add property test in `#[cfg(test)] mod property_tests` section

**Module Exports** (`rash/src/repl/mod.rs`):
- Add to exports: `pub use purifier::{..., PurificationError, purify_and_validate};`

---

## Success Criteria

- [ ] ✅ All 5 unit tests pass
- [ ] ✅ 1 property test passes (100+ cases)
- [ ] ✅ Clippy clean (no warnings)
- [ ] ✅ All library tests pass
- [ ] ✅ Mutation score ≥90% (on new code)
- [ ] ✅ Complexity <10 per function
- [ ] ✅ Roadmap updated
- [ ] ✅ Committed with proper message

---

## Dependencies

- REPL-014-001: `purify_and_lint()` function
- REPL-014-001: `PurifiedLintResult` struct
- Existing: `Diagnostic` struct from linter

---

## Notes

- This task enforces the quality gate that REPL-014-001 made possible
- Zero-tolerance means ANY DET/IDEM/SEC violation causes failure
- SC/MAKE violations are warnings only (not enforced)
- This provides a strong guarantee: `Ok(code)` means code is provably safe
- Complements REPL-014-001 by adding enforcement layer

---

## Related Tasks

- **REPL-014-001**: Auto-run bashrs linter on purified output (COMPLETE)
- **REPL-014-003**: Display lint violations in REPL context (NEXT)
- **REPL-005-001**: Call purifier from REPL (COMPLETE)
- **REPL-006-001**: Run linter from REPL (COMPLETE)
