# TICKET-REPL-014-001: Auto-run bashrs linter on purified output

**Sprint**: REPL-014 (Purified Output Validation)
**Task ID**: REPL-014-001
**Status**: IN PROGRESS
**Priority**: HIGH
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY → MUTATION)

---

## Problem Statement

bashrs purifies bash scripts to make them deterministic and idempotent, but currently does not validate that the purified output is actually safe and correct. We need to automatically run bashrs's own linter (355+ rules across DET/IDEM/SEC/SC/MAKE categories) on the purified output to ensure quality.

**Current Behavior**:
```rust
let purified = purify_bash("mkdir /tmp/test")?;
// Returns: "mkdir -p /tmp/test"
// No validation that purified output is clean!
```

**Desired Behavior**:
```rust
let result = purify_and_lint("mkdir /tmp/test")?;
// Returns: PurifiedLintResult {
//     purified_code: "mkdir -p /tmp/test",
//     lint_result: LintResult { diagnostics: [] },  // Clean!
//     is_clean: true
// }
```

---

## Requirements

1. **New Function**: `purify_and_lint(input: &str) -> Result<PurifiedLintResult>`
   - Purifies bash code using existing `purify_bash()`
   - Lints purified output using existing `lint_bash()`
   - Returns both purified code and lint results

2. **New Struct**: `PurifiedLintResult`
   - `purified_code: String` - The purified bash code
   - `lint_result: LintResult` - Lint results for purified code
   - `is_clean: bool` - True if no DET/IDEM/SEC violations

3. **Quality Gate**: Zero-tolerance for critical violations
   - No DET (determinism) violations allowed
   - No IDEM (idempotency) violations allowed
   - No SEC (security) violations allowed
   - SC/MAKE violations are warnings only

4. **Integration**: Seamless with existing REPL workflow
   - Can be called from REPL commands
   - Works with existing purifier and linter

---

## Data Structures

### PurifiedLintResult

```rust
/// Result of purifying and linting bash code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurifiedLintResult {
    /// The purified bash code
    pub purified_code: String,

    /// Lint results for the purified code
    pub lint_result: LintResult,

    /// True if purified code has no critical violations (DET/IDEM/SEC)
    pub is_clean: bool,
}

impl PurifiedLintResult {
    pub fn new(purified_code: String, lint_result: LintResult) -> Self {
        let is_clean = Self::check_is_clean(&lint_result);
        Self {
            purified_code,
            lint_result,
            is_clean,
        }
    }

    fn check_is_clean(lint_result: &LintResult) -> bool {
        // Check for critical violations: DET*, IDEM*, SEC*
        !lint_result.diagnostics.iter().any(|d| {
            d.code.starts_with("DET")
            || d.code.starts_with("IDEM")
            || d.code.starts_with("SEC")
        })
    }

    /// Get count of critical violations (DET/IDEM/SEC)
    pub fn critical_violations(&self) -> usize {
        self.lint_result.diagnostics.iter()
            .filter(|d| {
                d.code.starts_with("DET")
                || d.code.starts_with("IDEM")
                || d.code.starts_with("SEC")
            })
            .count()
    }

    /// Get DET violations only
    pub fn det_violations(&self) -> Vec<&Diagnostic> {
        self.lint_result.diagnostics.iter()
            .filter(|d| d.code.starts_with("DET"))
            .collect()
    }

    /// Get IDEM violations only
    pub fn idem_violations(&self) -> Vec<&Diagnostic> {
        self.lint_result.diagnostics.iter()
            .filter(|d| d.code.starts_with("IDEM"))
            .collect()
    }

    /// Get SEC violations only
    pub fn sec_violations(&self) -> Vec<&Diagnostic> {
        self.lint_result.diagnostics.iter()
            .filter(|d| d.code.starts_with("SEC"))
            .collect()
    }
}
```

---

## Functions

### purify_and_lint

```rust
/// Purify bash input and lint the purified output
///
/// This combines purification with linting to ensure the purified
/// output meets bashrs quality standards (no DET/IDEM/SEC violations).
///
/// # Examples
///
/// ```
/// use bashrs::repl::purifier::purify_and_lint;
///
/// let result = purify_and_lint("mkdir /tmp/test").unwrap();
/// assert_eq!(result.purified_code, "mkdir -p /tmp/test");
/// assert!(result.is_clean);
/// ```
pub fn purify_and_lint(input: &str) -> anyhow::Result<PurifiedLintResult> {
    // Step 1: Purify the input
    let purified_code = purify_bash(input)?;

    // Step 2: Lint the purified output
    let lint_result = lint_bash(&purified_code)?;

    // Step 3: Create result
    Ok(PurifiedLintResult::new(purified_code, lint_result))
}
```

### format_purified_lint_result

```rust
/// Format purified lint result for display in REPL
pub fn format_purified_lint_result(result: &PurifiedLintResult) -> String {
    let mut output = String::new();

    // Show purified code
    output.push_str("Purified:\n");
    output.push_str(&result.purified_code);
    output.push_str("\n\n");

    // Show lint results
    if result.is_clean {
        output.push_str("✓ Purified output is CLEAN (no DET/IDEM/SEC violations)\n");
    } else {
        output.push_str(&format!(
            "✗ Purified output has {} critical violation(s)\n",
            result.critical_violations()
        ));

        if !result.det_violations().is_empty() {
            output.push_str(&format!("  DET: {}\n", result.det_violations().len()));
        }
        if !result.idem_violations().is_empty() {
            output.push_str(&format!("  IDEM: {}\n", result.idem_violations().len()));
        }
        if !result.sec_violations().is_empty() {
            output.push_str(&format!("  SEC: {}\n", result.sec_violations().len()));
        }
    }

    // Show full lint report
    if !result.lint_result.diagnostics.is_empty() {
        output.push_str("\nLint Report:\n");
        output.push_str(&format_lint_results(&result.lint_result));
    }

    output
}
```

---

## Test Cases

### Unit Tests (6 tests)

#### test_REPL_014_001_purify_and_lint_mkdir

**Input**: `"mkdir /tmp/test"`
**Expected**:
- `purified_code` contains `"mkdir -p"`
- `is_clean` is `true`
- No DET/IDEM/SEC violations

```rust
#[test]
fn test_REPL_014_001_purify_and_lint_mkdir() {
    let input = "mkdir /tmp/test";
    let result = purify_and_lint(input);

    assert!(result.is_ok());
    let result = result.unwrap();

    // Purified should add -p flag
    assert!(result.purified_code.contains("mkdir -p"));

    // Should be clean (no DET/IDEM/SEC violations)
    assert!(result.is_clean, "Purified mkdir should be clean");
    assert_eq!(result.critical_violations(), 0);
}
```

#### test_REPL_014_001_purify_and_lint_random

**Input**: `"echo $RANDOM"`
**Expected**:
- `purified_code` does NOT contain `$RANDOM`
- `is_clean` is `true`
- No DET violations

```rust
#[test]
fn test_REPL_014_001_purify_and_lint_random() {
    let input = "echo $RANDOM";
    let result = purify_and_lint(input);

    assert!(result.is_ok());
    let result = result.unwrap();

    // Purified should remove $RANDOM
    assert!(!result.purified_code.contains("$RANDOM"));

    // Should be clean (no DET violations)
    assert!(result.is_clean, "Purified random should be clean");
    assert_eq!(result.det_violations().len(), 0);
}
```

#### test_REPL_014_001_no_det_violations

**Input**: `"echo $(date +%s)"`
**Expected**:
- Purified code has no timestamps
- No DET violations

```rust
#[test]
fn test_REPL_014_001_no_det_violations() {
    let input = "echo $(date +%s)";
    let result = purify_and_lint(input);

    assert!(result.is_ok());
    let result = result.unwrap();

    // No DET violations allowed
    assert_eq!(result.det_violations().len(), 0,
        "Purified output should have no DET violations");
}
```

#### test_REPL_014_001_no_idem_violations

**Input**: `"rm /tmp/file.txt"`
**Expected**:
- Purified code uses `rm -f`
- No IDEM violations

```rust
#[test]
fn test_REPL_014_001_no_idem_violations() {
    let input = "rm /tmp/file.txt";
    let result = purify_and_lint(input);

    assert!(result.is_ok());
    let result = result.unwrap();

    // Should add -f flag
    assert!(result.purified_code.contains("rm -f"));

    // No IDEM violations allowed
    assert_eq!(result.idem_violations().len(), 0,
        "Purified output should have no IDEM violations");
}
```

#### test_REPL_014_001_no_sec_violations

**Input**: `"echo $var"` (unquoted variable)
**Expected**:
- Purified code quotes variable
- No SEC violations

```rust
#[test]
fn test_REPL_014_001_no_sec_violations() {
    let input = "echo $var";
    let result = purify_and_lint(input);

    assert!(result.is_ok());
    let result = result.unwrap();

    // Should quote variable
    assert!(result.purified_code.contains("\"$var\"")
            || result.purified_code.contains("${var}"));

    // No SEC violations allowed
    assert_eq!(result.sec_violations().len(), 0,
        "Purified output should have no SEC violations");
}
```

#### test_REPL_014_001_format_result

**Input**: Purified lint result
**Expected**:
- Formatted output contains purified code
- Shows "CLEAN" status
- Shows violation counts

```rust
#[test]
fn test_REPL_014_001_format_result() {
    let input = "mkdir /tmp/test";
    let result = purify_and_lint(input).unwrap();

    let formatted = format_purified_lint_result(&result);

    // Should contain purified code
    assert!(formatted.contains("mkdir -p"));

    // Should show clean status
    assert!(formatted.contains("CLEAN") || formatted.contains("✓"));
}
```

### Integration Test (1 test)

#### test_REPL_014_001_purify_and_lint_integration

**Scenario**: Complete workflow from messy bash to clean purified output

```rust
#[test]
fn test_REPL_014_001_purify_and_lint_integration() {
    // Messy bash with multiple issues
    let input = r#"
mkdir /app/releases
echo $RANDOM
rm /tmp/old
echo $unquoted
"#;

    let result = purify_and_lint(input);
    assert!(result.is_ok());
    let result = result.unwrap();

    // All transformations should be applied
    assert!(result.purified_code.contains("mkdir -p"));
    assert!(!result.purified_code.contains("$RANDOM"));
    assert!(result.purified_code.contains("rm -f"));

    // Should be clean
    assert!(result.is_clean, "Complete purification should be clean");
    assert_eq!(result.critical_violations(), 0);
}
```

### Property Tests (2 tests)

#### prop_purified_always_clean

**Property**: Purified output should ALWAYS be clean (no DET/IDEM/SEC violations)

```rust
proptest! {
    #[test]
    fn prop_purified_always_clean(input in "(mkdir|rm|ln|echo).*{1,100}") {
        if let Ok(result) = purify_and_lint(&input) {
            // Critical property: purified output must be clean
            prop_assert!(
                result.is_clean,
                "Purified output must always be clean, but found {} violations",
                result.critical_violations()
            );

            prop_assert_eq!(result.det_violations().len(), 0);
            prop_assert_eq!(result.idem_violations().len(), 0);
            prop_assert_eq!(result.sec_violations().len(), 0);
        }
    }
}
```

#### prop_purify_and_lint_never_panics

**Property**: Function should never panic on any input

```rust
proptest! {
    #[test]
    fn prop_purify_and_lint_never_panics(input in ".*{0,1000}") {
        // Should gracefully handle any input
        let _ = purify_and_lint(&input);
        // If we get here without panic, test passes
    }
}
```

---

## EXTREME TDD Phases

### Phase 1: RED (Write Failing Tests)

1. Create `PurifiedLintResult` struct (stub)
2. Add stub `purify_and_lint()` function with `unimplemented!()`
3. Add stub `format_purified_lint_result()` with `unimplemented!()`
4. Write all 6 unit tests
5. Write 1 integration test
6. Write 2 property tests
7. **Run tests** - expect 9 failures/panics

**Expected**: All tests fail (RED phase confirmed)

### Phase 2: GREEN (Implement Minimum)

1. Implement `PurifiedLintResult::new()`
2. Implement `check_is_clean()`
3. Implement helper methods: `critical_violations()`, `det_violations()`, etc.
4. Implement `purify_and_lint()` - call existing functions
5. Implement `format_purified_lint_result()`
6. **Run tests** - expect all 9 tests to pass

**Expected**: All tests pass (GREEN phase confirmed)

### Phase 3: REFACTOR (Clean Up)

1. Run `cargo clippy` - fix any warnings
2. Extract helper functions if needed
3. Ensure complexity <10 per function
4. Add comprehensive documentation
5. **Run all 5,593 library tests** - expect 100% pass

**Expected**: Clean code, all tests passing

### Phase 4: PROPERTY (Generative Testing)

1. Run property tests with 100+ cases each
2. Verify critical property: purified output ALWAYS clean
3. Verify never panics on any input
4. **Analyze failures** - should be zero

**Expected**: 200+ property test cases pass

### Phase 5: MUTATION (Resilience Testing)

1. Run `cargo mutants --file rash/src/repl/purifier.rs`
2. Target: ≥90% kill rate
3. Fix any surviving mutants
4. **Verify** mutation score

**Expected**: Mutation score ≥90%

### Phase 6: COMMIT

1. Update `docs/REPL-DEBUGGER-ROADMAP.yaml`:
   - Mark REPL-014-001 as "completed"
   - Add all test names
2. Create commit message following Toyota Way principles
3. **Commit** with message

---

## Implementation Location

**File**: `rash/src/repl/purifier.rs`

**Changes**:
1. Add `use crate::linter::{lint_shell, LintResult, Severity, Diagnostic};`
2. Add `PurifiedLintResult` struct (after line 13)
3. Add `purify_and_lint()` function (after line 40)
4. Add `format_purified_lint_result()` function (after line 68)
5. Add tests in `#[cfg(test)] mod tests` section
6. Add property tests in `#[cfg(test)] mod property_tests` section

**Module Exports** (`rash/src/repl/mod.rs`):
- Add to line 44: `pub use purifier::{..., PurifiedLintResult, purify_and_lint, format_purified_lint_result};`

---

## Success Criteria

- [ ] ✅ All 6 unit tests pass
- [ ] ✅ 1 integration test passes
- [ ] ✅ 2 property tests pass (100+ cases each)
- [ ] ✅ Clippy clean (no warnings)
- [ ] ✅ All 5,593 library tests pass
- [ ] ✅ Mutation score ≥90%
- [ ] ✅ Complexity <10 per function
- [ ] ✅ Roadmap updated
- [ ] ✅ Committed with proper message

---

## Dependencies

- Existing `purify_bash()` function
- Existing `lint_bash()` function
- Existing `LintResult` struct
- Existing `Diagnostic` struct
- Existing linter rules (DET/IDEM/SEC/SC/MAKE)

---

## Notes

- This task focuses on AUTO-RUNNING the linter, not on enforcing zero-tolerance
- Zero-tolerance enforcement is REPL-014-002 (next task)
- This lays groundwork for quality gates in subsequent tasks
- Uses bashrs's own 355+ linter rules (NOT external shellcheck)
- Avoids technical debt by leveraging existing infrastructure

---

## Related Tasks

- **REPL-005-001**: Call purifier from REPL (COMPLETE)
- **REPL-006-001**: Run linter from REPL (COMPLETE)
- **REPL-013-003**: Alternative suggestions (COMPLETE)
- **REPL-014-002**: Zero-tolerance quality gate (NEXT)
- **REPL-014-003**: Display lint violations in REPL context (FUTURE)
