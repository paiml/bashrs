# SEC Iteration Mutation Fixes

**Date**: 2025-11-04
**Context**: SEC batch iteration tests revealed 16 missed mutations across SEC002, SEC004-SEC007
**Pattern**: 62.5% of missed mutations are arithmetic (`+` → `*`) in column position calculations

## 📊 Summary

**5 of 6 rules complete** (SEC008 still running):
- SEC002: 84.8% kill rate (4 missed)
- SEC004: 76.9% kill rate (6 missed) ← **WORST**
- SEC005: 82.1% kill rate (3 missed)
- SEC006: 85.7% kill rate (2 missed)
- SEC007: 88.9% kill rate (1 missed)

**Average**: 83.7% kill rate (16 missed out of 110 tested)

## 🎯 Universal Pattern: Column Position Arithmetic

**10 out of 16 missed mutations** (62.5%) are arithmetic mutations in `Span` calculations:

```rust
// MISSED mutation: col + 23 → col * 23
let span = Span::new(
    line_num + 1,
    col + 1,        // ← Not validated
    line_num + 1,
    col + 23,       // ← MISSED: Tests don't check exact position
);
```

### Why Current Tests Miss These

Current tests validate:
- ✅ Diagnostic was created
- ✅ Diagnostic has correct rule ID (e.g., "SEC004")
- ✅ Diagnostic has correct severity

Current tests DON'T validate:
- ❌ **Span points to exact source location**
- ❌ **Column numbers are mathematically correct**
- ❌ **End column = start column + string length**

## 🔧 Fix Strategy

### 1. Add Property-Based Span Validation Tests

Create tests that verify:
```rust
#[test]
fn test_sec004_span_positions_exact() {
    let source = "wget --no-check-certificate https://example.com";
    let result = check(source);

    // Verify diagnostic was created
    assert_eq!(result.diagnostics.len(), 1);
    let diag = &result.diagnostics[0];

    // NEW: Verify span points to exact string location
    assert_eq!(diag.span.start_line, 1);
    assert_eq!(diag.span.start_col, 6);  // "--no-check-certificate" starts at column 6
    assert_eq!(diag.span.end_line, 1);
    assert_eq!(diag.span.end_col, 28);   // Ends at column 28 (5 + 23)

    // NEW: Verify substring extraction matches
    let lines: Vec<&str> = source.lines().collect();
    let line = lines[diag.span.start_line - 1];
    let start_idx = diag.span.start_col - 1;
    let end_idx = diag.span.end_col - 1;
    let extracted = &line[start_idx..end_idx];
    assert_eq!(extracted, "--no-check-certificate");
}
```

### 2. Property-Based Tests for All Variants

For each SEC rule, add tests that:
1. Extract the exact substring using the span
2. Verify it matches the expected pattern
3. Test multiple positions (start, middle, end of line)

### 3. Universal Span Validation Pattern

```rust
/// Property: Span must point to exact source location
fn verify_span_extracts_correctly(source: &str, span: &Span, expected: &str) {
    let lines: Vec<&str> = source.lines().collect();
    assert!(span.start_line > 0 && span.start_line <= lines.len());

    let line = lines[span.start_line - 1];
    let start_idx = span.start_col - 1;
    let end_idx = span.end_col - 1;

    assert!(start_idx < line.len(), "Start column out of bounds");
    assert!(end_idx <= line.len(), "End column out of bounds");
    assert!(start_idx < end_idx, "Start must be before end");

    let extracted = &line[start_idx..end_idx];
    assert_eq!(extracted, expected, "Span doesn't point to correct substring");
}
```

## 📋 Missed Mutations by Rule

### SEC002 (4 missed)
- `sec002.rs:62:20`: `!in_single_quotes` → `true` (guard mutation)
- `sec002.rs:63:21`: `!in_double_quotes` → `true` (guard mutation)
- `sec002.rs:69:56`: `==` → `!=` (equality mutation)
- `sec002.rs:84:54`: `col + position.len()` → `col * position.len()` ← **ARITHMETIC**

**Fix**: Add test verifying span extraction for unquoted variables

### SEC004 (6 missed - WORST)
- `sec004.rs:40:30`: `col + 23` → `col * 23` ← **ARITHMETIC**
- `sec004.rs:60:30`: `col + 2` → `col * 2` ← **ARITHMETIC**
- `sec004.rs:62:30`: `col + 4` → `col * 4` ← **ARITHMETIC**
- `sec004.rs:77:30`: `col + 1` → `col * 1` ← **ARITHMETIC**
- `sec004.rs:79:30`: `col + 11` → `col * 11` ← **ARITHMETIC**
- `sec004.rs:35:34`: `&&` → `||` (logic mutation)

**Fix**: Add 3 span extraction tests (one per pattern: --no-check-certificate, -k, --insecure)

### SEC005 (3 missed - ALL ARITHMETIC)
- `sec005.rs:72:18`: `position + 1` → `position * 1` ← **ARITHMETIC**
- `sec005.rs:73:26`: `position + format_str.len()` → `position * format_str.len()` ← **ARITHMETIC**
- `sec005.rs:73:40`: `position + format_str.len() + 1` → `position + format_str.len() * 1` ← **ARITHMETIC**

**Fix**: Add test verifying span extraction for `printf` format strings

### SEC006 (2 missed)
- `sec006.rs:34:35`: `&&` → `||` (logic mutation)
- `sec006.rs:46:38`: `col + option.len() + 1` → `col + option.len() * 1` ← **ARITHMETIC**

**Fix**: Add test verifying span extraction for `find -exec` patterns

### SEC007 (1 missed)
- `sec007.rs:49:42`: `col + 1` → `col * 1` ← **ARITHMETIC**

**Fix**: Add test verifying span extraction for command injection patterns

## 🚀 Implementation Plan

### Phase 1: Create Universal Span Validator (Iteration 2)
Create helper function `verify_span_extracts_correctly()` in test utils.

### Phase 2: Add SEC004 Span Tests (Iteration 2)
Add 3 tests covering all patterns:
- `test_sec004_wget_span_exact()`
- `test_sec004_curl_k_span_exact()`
- `test_sec004_curl_insecure_span_exact()`
- `test_sec004_logic_and_not_or()`

### Phase 3: Add SEC002, SEC005-SEC007 Span Tests (Iteration 3)
Apply same pattern to other rules.

### Phase 4: Property-Based Fuzzing (Iteration 4)
Add proptest for random placements:
```rust
proptest! {
    #[test]
    fn prop_sec004_span_always_correct(
        prefix in "[a-z ]{0,50}",
        suffix in "[a-z ]{0,50}"
    ) {
        let source = format!("{}wget --no-check-certificate{}", prefix, suffix);
        let result = check(&source);
        if !result.diagnostics.is_empty() {
            let diag = &result.diagnostics[0];
            verify_span_extracts_correctly(&source, &diag.span, "--no-check-certificate");
        }
    }
}
```

## ✅ Success Criteria

After implementing fixes:
- **SEC004**: 76.9% → **95%+** kill rate
- **SEC002**: 84.8% → **95%+** kill rate
- **SEC005**: 82.1% → **100%** kill rate (only 3 arithmetic mutations)
- **SEC006**: 85.7% → **95%+** kill rate
- **SEC007**: 88.9% → **100%** kill rate (only 1 arithmetic mutation)

**Target Average**: **83.7% → 95%+** (raising baseline by 11+ points)

## 🔍 Pattern Discovery

This confirms the **universal mutation pattern** we discovered:
1. Arithmetic mutations (`+` → `*`) in position calculations
2. Logic mutations (`&&` → `||`) in guard conditions

**Key Insight**: Most mutation testing tools focus on logic mutations, but **arithmetic mutations in offset calculations** are equally important for security linters where exact source positions matter.

## 📚 References

- SEC batch iteration tests: `./run_sec_iteration_tests.sh`
- Mutation logs: `sec_iteration_tests_full.log`
- Universal pattern first discovered: SC2086 Iteration 3 (2025-11-03)
- Applied successfully to: SC2059, SEC001

---

**Next Steps**:
1. Wait for SEC008 to complete
2. Calculate final average
3. Implement Phase 1 (universal span validator)
4. Implement Phase 2 (SEC004 fixes)
5. Run SEC004 iteration 2 mutation test
6. Verify 95%+ kill rate achieved
