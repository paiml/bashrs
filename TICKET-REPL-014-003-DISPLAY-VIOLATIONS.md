# TICKET-REPL-014-003: Display lint violations in REPL context

**Sprint**: REPL-014 (Purified Output Validation)
**Task ID**: REPL-014-003
**Status**: IN PROGRESS
**Priority**: MEDIUM
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY → MUTATION)
**Dependencies**: REPL-014-001 (Auto-run bashrs linter on purified output), REPL-014-002 (Zero-tolerance quality gate)

---

## Problem Statement

REPL-014-001 added automatic linting with `purify_and_lint()`, and REPL-014-002 added zero-tolerance enforcement with `purify_and_validate()`. However, when violations are reported, the user only sees line numbers without source context or fix suggestions. We need to enhance the display to show:

1. **Line context**: Show the problematic line of code
2. **Fix suggestions**: Display the proposed fix if available
3. **Multi-line context**: Show surrounding lines for better understanding
4. **Visual highlighting**: Make violations easy to spot

**Current Behavior**:
```rust
let result = format_lint_results(&lint_result);
// Shows:
// ERROR [DET001] Line 5: Non-deterministic pattern detected
// No source context shown!
```

**Desired Behavior**:
```rust
let formatted = format_violations_with_context(&lint_result, source_code);
// Shows:
//   4 | mkdir /app/releases
//   5 | echo $RANDOM
//       ^^^^^^^^^^^^^ ERROR [DET001]: Non-deterministic $RANDOM
//   6 | rm /tmp/file
//
// Suggested fix:
//   5 | # Deterministic alternative required
```

---

## Requirements

1. **New Function**: `format_violations_with_context(result: &LintResult, source: &str) -> String`
   - Takes lint results AND source code
   - Returns formatted output with line context
   - Shows ±2 lines of context around each violation
   - Displays fix suggestions when available

2. **Line Context Display**: For each violation, show:
   - Line number with proper padding
   - Source code line
   - Visual indicator (caret `^` or tilde `~`) pointing to problem
   - Diagnostic message with code (e.g., DET001)

3. **Fix Suggestion Display**: When fix available:
   - Show "Suggested fix:" header
   - Display the proposed fix with line numbers
   - Show diff-style formatting (- old, + new)

4. **Multi-line Violations**: Handle violations spanning multiple lines:
   - Show all affected lines
   - Indicate range with visual markers

5. **Integration**: Update `format_purified_lint_result()` to use new formatter:
   - Pass source code along with lint results
   - Maintain backward compatibility with existing code

---

## Data Structures

### No new structs needed

We'll use existing structures:
- `LintResult` (from linter)
- `Diagnostic` (from linter) - already has `span` and `fix` fields
- `Span` (from linter) - has line/column information
- `Fix` (from linter) - has replacement text

---

## Functions

### format_violations_with_context

```rust
/// Format lint violations with source code context
///
/// Displays each violation with:
/// - Line numbers (±2 lines of context)
/// - Source code at that location
/// - Visual indicator (caret) pointing to the issue
/// - Diagnostic message with rule code
/// - Fix suggestion if available
///
/// # Examples
///
/// ```
/// use bashrs::repl::linter::{format_violations_with_context, lint_bash};
///
/// let source = "echo $RANDOM\nmkdir /app\n";
/// let result = lint_bash(source).unwrap();
/// let formatted = format_violations_with_context(&result, source);
///
/// // Output:
/// //   1 | echo $RANDOM
/// //       ^^^^^^^^^^^^^ ERROR [DET001]: Non-deterministic $RANDOM
/// //   2 | mkdir /app
/// //       ^^^^^^^^^^ ERROR [IDEM001]: mkdir without -p
/// ```
pub fn format_violations_with_context(result: &LintResult, source: &str) -> String {
    let mut output = String::new();

    if result.diagnostics.is_empty() {
        return "✓ No violations\n".to_string();
    }

    let lines: Vec<&str> = source.lines().collect();
    let max_line_num = lines.len();
    let line_num_width = max_line_num.to_string().len().max(3);

    for diagnostic in &result.diagnostics {
        let line_idx = diagnostic.span.start.line.saturating_sub(1);

        // Show context: ±2 lines
        let start_line = line_idx.saturating_sub(2);
        let end_line = (line_idx + 3).min(lines.len());

        output.push('\n');

        // Show context lines
        for i in start_line..end_line {
            if i < lines.len() {
                let line_num = i + 1;
                let prefix = if i == line_idx { ">" } else { " " };
                output.push_str(&format!(
                    "{} {:>width$} | {}\n",
                    prefix,
                    line_num,
                    lines[i],
                    width = line_num_width
                ));

                // Show indicator on the problematic line
                if i == line_idx {
                    let col = diagnostic.span.start.column.saturating_sub(1);
                    let indicator_width = if diagnostic.span.end.line == diagnostic.span.start.line {
                        diagnostic.span.end.column.saturating_sub(diagnostic.span.start.column).max(1)
                    } else {
                        lines[i].len().saturating_sub(col).max(1)
                    };

                    output.push_str(&format!(
                        "  {:>width$} | {}{} {} [{}]: {}\n",
                        "",
                        " ".repeat(col),
                        "^".repeat(indicator_width),
                        diagnostic.severity,
                        diagnostic.code,
                        diagnostic.message,
                        width = line_num_width
                    ));
                }
            }
        }

        // Show fix suggestion if available
        if let Some(fix) = &diagnostic.fix {
            output.push_str(&format!("\n  Suggested fix:\n"));
            output.push_str(&format!("  {:>width$} | {}\n",
                line_idx + 1,
                fix.replacement,
                width = line_num_width
            ));
        }
    }

    output
}
```

### format_purified_lint_result_with_context

```rust
/// Format purified lint result with source code context (enhanced version)
///
/// This is an enhanced version of `format_purified_lint_result()` that shows
/// source code context for each violation.
///
/// # Examples
///
/// ```
/// use bashrs::repl::purifier::format_purified_lint_result_with_context;
///
/// let input = "echo $RANDOM";
/// let result = purify_and_lint(input).unwrap();
/// let formatted = format_purified_lint_result_with_context(&result, input);
/// ```
pub fn format_purified_lint_result_with_context(
    result: &PurifiedLintResult,
    original_source: &str,
) -> String {
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

        // Show violations with context
        output.push_str("\n");
        output.push_str(&format_violations_with_context(
            &result.lint_result,
            &result.purified_code,  // Show context from purified code
        ));
    }

    output
}
```

---

## Test Cases

### Unit Tests (6 tests)

#### test_REPL_014_003_format_single_violation

**Input**: Single DET violation
**Expected**: Shows line context with caret indicator

```rust
#[test]
fn test_REPL_014_003_format_single_violation() {
    let source = "echo hello\necho $RANDOM\necho world\n";

    // Create a diagnostic manually for testing
    let diagnostic = Diagnostic {
        code: "DET001".to_string(),
        severity: Severity::Error,
        message: "Non-deterministic $RANDOM".to_string(),
        span: Span {
            start: Position { line: 2, column: 6 },
            end: Position { line: 2, column: 13 },
        },
        fix: None,
    };

    let lint_result = LintResult {
        diagnostics: vec![diagnostic],
    };

    let formatted = format_violations_with_context(&lint_result, source);

    // Should show line context
    assert!(formatted.contains("  1 | echo hello"));
    assert!(formatted.contains("> 2 | echo $RANDOM"));
    assert!(formatted.contains("  3 | echo world"));

    // Should show indicator
    assert!(formatted.contains("^^^^^^^"));  // 7 chars for "$RANDOM"
    assert!(formatted.contains("ERROR [DET001]"));
    assert!(formatted.contains("Non-deterministic $RANDOM"));
}
```

#### test_REPL_014_003_format_with_fix

**Input**: Violation with fix suggestion
**Expected**: Shows both violation and suggested fix

```rust
#[test]
fn test_REPL_014_003_format_with_fix() {
    let source = "mkdir /app\n";

    let diagnostic = Diagnostic {
        code: "IDEM001".to_string(),
        severity: Severity::Error,
        message: "mkdir without -p".to_string(),
        span: Span {
            start: Position { line: 1, column: 1 },
            end: Position { line: 1, column: 11 },
        },
        fix: Some(Fix {
            replacement: "mkdir -p /app".to_string(),
        }),
    };

    let lint_result = LintResult {
        diagnostics: vec![diagnostic],
    };

    let formatted = format_violations_with_context(&lint_result, source);

    // Should show violation
    assert!(formatted.contains("> 1 | mkdir /app"));
    assert!(formatted.contains("IDEM001"));

    // Should show fix
    assert!(formatted.contains("Suggested fix:"));
    assert!(formatted.contains("mkdir -p /app"));
}
```

#### test_REPL_014_003_multiple_violations

**Input**: Multiple violations on different lines
**Expected**: Shows each violation with context

```rust
#[test]
fn test_REPL_014_003_multiple_violations() {
    let source = "echo $RANDOM\nmkdir /app\nrm /tmp/file\n";

    let diagnostics = vec![
        Diagnostic {
            code: "DET001".to_string(),
            severity: Severity::Error,
            message: "Non-deterministic $RANDOM".to_string(),
            span: Span {
                start: Position { line: 1, column: 6 },
                end: Position { line: 1, column: 13 },
            },
            fix: None,
        },
        Diagnostic {
            code: "IDEM001".to_string(),
            severity: Severity::Error,
            message: "mkdir without -p".to_string(),
            span: Span {
                start: Position { line: 2, column: 1 },
                end: Position { line: 2, column: 11 },
            },
            fix: Some(Fix {
                replacement: "mkdir -p /app".to_string(),
            }),
        },
    ];

    let lint_result = LintResult { diagnostics };

    let formatted = format_violations_with_context(&lint_result, source);

    // Should show both violations
    assert!(formatted.contains("DET001"));
    assert!(formatted.contains("IDEM001"));
    assert!(formatted.contains("echo $RANDOM"));
    assert!(formatted.contains("mkdir /app"));
}
```

#### test_REPL_014_003_no_violations

**Input**: Clean code with no violations
**Expected**: Shows "No violations" message

```rust
#[test]
fn test_REPL_014_003_no_violations() {
    let source = "echo hello\n";
    let lint_result = LintResult {
        diagnostics: vec![],
    };

    let formatted = format_violations_with_context(&lint_result, source);

    assert!(formatted.contains("✓ No violations"));
}
```

#### test_REPL_014_003_purified_result_with_context

**Input**: PurifiedLintResult with violations
**Expected**: Shows purified code and violations with context

```rust
#[test]
fn test_REPL_014_003_purified_result_with_context() {
    let original = "mkdir /app\n";

    // Manually create a purified result with violation for testing
    let diagnostic = Diagnostic {
        code: "IDEM001".to_string(),
        severity: Severity::Error,
        message: "mkdir without -p".to_string(),
        span: Span {
            start: Position { line: 1, column: 1 },
            end: Position { line: 1, column: 11 },
        },
        fix: Some(Fix {
            replacement: "mkdir -p /app".to_string(),
        }),
    };

    let lint_result = LintResult {
        diagnostics: vec![diagnostic],
    };

    let purified_result = PurifiedLintResult {
        purified_code: "mkdir /app\n".to_string(),
        lint_result,
        is_clean: false,
    };

    let formatted = format_purified_lint_result_with_context(&purified_result, original);

    // Should show purified code
    assert!(formatted.contains("Purified:"));
    assert!(formatted.contains("mkdir /app"));

    // Should show violation count
    assert!(formatted.contains("critical violation"));

    // Should show context
    assert!(formatted.contains("> 1 | mkdir /app"));
    assert!(formatted.contains("IDEM001"));

    // Should show fix
    assert!(formatted.contains("Suggested fix:"));
    assert!(formatted.contains("mkdir -p /app"));
}
```

#### test_REPL_014_003_edge_of_file

**Input**: Violation on first/last line
**Expected**: Handles edge cases gracefully (no out-of-bounds)

```rust
#[test]
fn test_REPL_014_003_edge_of_file() {
    // Test violation on first line
    let source1 = "echo $RANDOM\n";
    let diagnostic1 = Diagnostic {
        code: "DET001".to_string(),
        severity: Severity::Error,
        message: "Non-deterministic $RANDOM".to_string(),
        span: Span {
            start: Position { line: 1, column: 6 },
            end: Position { line: 1, column: 13 },
        },
        fix: None,
    };

    let formatted1 = format_violations_with_context(
        &LintResult { diagnostics: vec![diagnostic1] },
        source1,
    );

    // Should not crash, should show line 1
    assert!(formatted1.contains("> 1 | echo $RANDOM"));

    // Test violation on last line
    let source2 = "echo hello\necho world\necho $RANDOM\n";
    let diagnostic2 = Diagnostic {
        code: "DET001".to_string(),
        severity: Severity::Error,
        message: "Non-deterministic $RANDOM".to_string(),
        span: Span {
            start: Position { line: 3, column: 6 },
            end: Position { line: 3, column: 13 },
        },
        fix: None,
    };

    let formatted2 = format_violations_with_context(
        &LintResult { diagnostics: vec![diagnostic2] },
        source2,
    );

    // Should not crash, should show lines 1-3
    assert!(formatted2.contains("  1 | echo hello"));
    assert!(formatted2.contains("  2 | echo world"));
    assert!(formatted2.contains("> 3 | echo $RANDOM"));
}
```

### Integration Test (1 test)

#### test_REPL_014_003_integration_purify_and_format

**Scenario**: Full workflow - purify messy bash, show violations with context

```rust
#[test]
fn test_REPL_014_003_integration_purify_and_format() {
    let messy_bash = r#"
mkdir /app/releases
echo $RANDOM
rm /tmp/old
"#;

    // Purify and lint
    let result = purify_and_lint(messy_bash);

    if let Ok(purified_result) = result {
        // Format with context
        let formatted = format_purified_lint_result_with_context(&purified_result, messy_bash);

        // Should show purified code
        assert!(formatted.contains("Purified:"));

        // If there are violations, should show context
        if !purified_result.is_clean {
            // Should show line numbers and context
            assert!(formatted.contains("|"));

            // Should show violation codes
            let has_det = !purified_result.det_violations().is_empty();
            let has_idem = !purified_result.idem_violations().is_empty();

            if has_det {
                assert!(formatted.contains("DET"));
            }
            if has_idem {
                assert!(formatted.contains("IDEM"));
            }
        }
    }
}
```

### Property Tests (1 test)

#### prop_format_never_panics

**Property**: Formatting should never panic on any input

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_format_never_panics(
            source in ".*{0,500}",
            line in 1usize..100,
            col in 1usize..100,
        ) {
            // Create a diagnostic at potentially out-of-bounds position
            let diagnostic = Diagnostic {
                code: "TEST001".to_string(),
                severity: Severity::Error,
                message: "Test message".to_string(),
                span: Span {
                    start: Position { line, column: col },
                    end: Position { line, column: col + 5 },
                },
                fix: None,
            };

            let lint_result = LintResult {
                diagnostics: vec![diagnostic],
            };

            // Should not panic on any input
            let _ = format_violations_with_context(&lint_result, &source);
        }
    }
}
```

---

## EXTREME TDD Phases

### Phase 1: RED (Write Failing Tests)

1. Add imports to `rash/src/repl/linter.rs`:
   ```rust
   use crate::linter::{Diagnostic, LintResult, Severity, Span, Position, Fix};
   ```

2. Add stub `format_violations_with_context()` with `unimplemented!()`:
   ```rust
   pub fn format_violations_with_context(result: &LintResult, source: &str) -> String {
       unimplemented!("REPL-014-003: Not yet implemented")
   }
   ```

3. Add to `rash/src/repl/purifier.rs`:
   ```rust
   pub fn format_purified_lint_result_with_context(
       result: &PurifiedLintResult,
       original_source: &str,
   ) -> String {
       unimplemented!("REPL-014-003: Not yet implemented")
   }
   ```

4. Write all 6 unit tests (should panic with "not yet implemented")

5. Write 1 integration test (should panic)

6. Write 1 property test (should panic)

7. **Run tests** - expect 8 failures/panics

**Expected**: All tests fail (RED phase confirmed)

### Phase 2: GREEN (Implement Minimum)

1. Implement `format_violations_with_context()`:
   - Parse source into lines
   - For each diagnostic, show ±2 lines context
   - Show line numbers with proper padding
   - Show caret indicators
   - Show fix suggestions if available

2. Implement `format_purified_lint_result_with_context()`:
   - Reuse existing `format_purified_lint_result()` structure
   - Call `format_violations_with_context()` for violations

3. **Run tests** - expect all 8 tests to pass

**Expected**: All tests pass (GREEN phase confirmed)

### Phase 3: REFACTOR (Clean Up)

1. Run `cargo clippy` - fix any warnings
2. Extract helper functions if needed:
   - `format_line_context()` for line display
   - `format_indicator()` for caret display
   - `format_fix_suggestion()` for fix display
3. Ensure complexity <10 per function
4. Add comprehensive documentation
5. **Run all library tests** - expect 100% pass

**Expected**: Clean code, all tests passing

### Phase 4: PROPERTY (Generative Testing)

1. Run property test with 100+ cases
2. Verify never panics on any input
3. Verify handles edge cases (empty source, out-of-bounds positions)
4. **Analyze failures** - should be zero

**Expected**: 100+ property test cases pass

### Phase 5: MUTATION (Resilience Testing)

1. Run `cargo mutants --file rash/src/repl/linter.rs`
2. Target: ≥90% kill rate
3. Fix any surviving mutants
4. **Verify** mutation score

**Expected**: Mutation score ≥90%

### Phase 6: COMMIT

1. Update `docs/REPL-DEBUGGER-ROADMAP.yaml`:
   - Mark REPL-014-003 as "completed"
   - Add all test names
2. Create commit message following Toyota Way principles
3. **Commit** with message

---

## Implementation Location

**File**: `rash/src/repl/linter.rs`

**Changes**:
1. Add imports (if not already present):
   ```rust
   use crate::linter::{Diagnostic, LintResult, Severity, Span, Position, Fix};
   ```
2. Add `format_violations_with_context()` function (after existing `format_lint_results()`)
3. Add tests in `#[cfg(test)] mod tests` section
4. Add property test in `#[cfg(test)] mod property_tests` section

**File**: `rash/src/repl/purifier.rs`

**Changes**:
1. Add `format_purified_lint_result_with_context()` function (after `format_purified_lint_result()`)
2. Add tests in `#[cfg(test)] mod tests` section

**Module Exports** (`rash/src/repl/mod.rs`):
- Add to exports:
  ```rust
  pub use linter::{..., format_violations_with_context};
  pub use purifier::{..., format_purified_lint_result_with_context};
  ```

---

## Success Criteria

- [ ] ✅ All 6 unit tests pass
- [ ] ✅ 1 integration test passes
- [ ] ✅ 1 property test passes (100+ cases)
- [ ] ✅ Clippy clean (no warnings)
- [ ] ✅ All library tests pass
- [ ] ✅ Mutation score ≥90% (on new code)
- [ ] ✅ Complexity <10 per function
- [ ] ✅ Roadmap updated
- [ ] ✅ Committed with proper message

---

## Dependencies

- REPL-014-001: `PurifiedLintResult` struct
- REPL-014-001: `format_purified_lint_result()` function
- Existing: `LintResult`, `Diagnostic`, `Span`, `Position`, `Fix` structs from linter

---

## Notes

- This task enhances user experience by showing WHERE violations occur
- Line context helps users understand the problem quickly
- Fix suggestions guide users toward correct solutions
- Maintains backward compatibility (existing formatter still works)
- Optional enhancement: Could be extended with color/syntax highlighting in future

---

## Related Tasks

- **REPL-014-001**: Auto-run bashrs linter on purified output (COMPLETE)
- **REPL-014-002**: Zero-tolerance quality gate (COMPLETE)
- **REPL-005-001**: Call purifier from REPL (COMPLETE)
- **REPL-006-001**: Run linter from REPL (COMPLETE)
