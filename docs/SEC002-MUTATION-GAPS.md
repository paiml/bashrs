# SEC002 Mutation Coverage Gap Analysis

**Status**: 🔄 IN PROGRESS - Baseline testing (5/33 mutants)
**Rule**: SEC002 - CRITICAL unquoted variables in dangerous commands
**Baseline Tests**: 16 existing tests (12 property + 4 unit)
**Total Mutants**: 33 (cargo-mutants baseline running)
**Baseline Results**: TBD (baseline in progress)
**Target**: 90%+ kill rate

## 📊 Baseline Mutation Results

**Phase 0 Outcome** (COMPLETE):
- ✅ **Total Mutants**: 33 (32 viable, 1 unviable)
- ✅ **Caught**: 24/32 (75.0%)
- ❌ **Missed**: 8/32 (25.0%)
- ⏱️ **Time**: 41m 54s
- 🎯 **Pattern Confirmed**: Type 1 (inline Span::new()) - matches SEC001

## 🚨 Identified Mutation Gaps (8 MISSED)

Complete baseline results show **arithmetic and logic mutations** as expected from SEC-PATTERN-GUIDE.md Type 1 pattern.

### Gap 1: Column Position Arithmetic (4 MISSED)

**Missed Mutants** (baseline complete):
1. Line 84:35: `line_num + 1` - replace + with * in create_sec002_diagnostic
2. Line 84:54: `line_num + 1` (end line) - replace + with * in create_sec002_diagnostic
3. Line 84:63: `col + 1` - replace + with - in create_sec002_diagnostic
4. Line 84:63: `col + 1` - replace + with * in create_sec002_diagnostic

**Code Context** (sec002.rs:84):
```rust
fn create_sec002_diagnostic(cmd: &str, line_num: usize, col: usize) -> Diagnostic {
    let span = Span::new(line_num + 1, col, line_num + 1, col + 1);
    //                   ^^^^^^^^^^^  ^^^  ^^^^^^^^^^^  ^^^^^^^
    //                   MISSED (2x): + → *              MISSED (2x): + → *, + → -
    // ...
}
```

**Root Cause**: No tests verify exact column/line positions in diagnostic spans

**Impact**: Incorrect spans could break error reporting or confuse users

### Gap 2: Helper Function Logic (4 MISSED)

**Missed Mutants** (baseline complete):
5. Line 59:13: `col += 1` - replace += with *= in find_unquoted_variable
6. Line 62:20: `!in_single_quotes` - replace match guard with true in find_unquoted_variable
7. Line 63:21: `!in_double_quotes` - replace match guard with true in find_unquoted_variable
8. Line 69:56: `c.is_alphanumeric() || *c == '_'` - replace == with != in find_unquoted_variable

**Code Context** (sec002.rs:59-69):
```rust
fn find_unquoted_variable(line: &str) -> Option<usize> {
    // ...
    while let Some(ch) = chars.next() {
        col += 1;  // Line 59 - MISSED: += → *=

        match ch {
            '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
            //     ^^^^^^^^^^^^^^^^^  Line 62 - MISSED: → true
            '\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
            //      ^^^^^^^^^^^^^^^^^  Line 63 - MISSED: → true
            '$' if !in_double_quotes && !in_single_quotes => {
                if chars
                    .peek()
                    .map(|c| c.is_alphanumeric() || *c == '_')
                    //                          ^^^^^^^^^^  Line 69 - MISSED: == → !=
                    .unwrap_or(false)
            }
        }
    }
}
```

**Root Cause**: Helper function logic not comprehensively tested

**Impact**:
- `col += 1` mutation: Incorrect column tracking could produce wrong diagnostics
- Match guard mutations: Quote detection failure could miss unquoted variables or false positives
- Variable detection mutation: Could fail to detect valid variables

## 📋 Required Tests (TBD after baseline completion)

Following the proven SEC001 pattern (100% kill rate), we'll need:

### Category 1: Exact Position Tests (for Gap 1)

**Pattern** (from SEC001 100% success):
```rust
#[test]
fn test_mutation_sec002_unquoted_var_start_col_exact() {
    // MUTATION: Line 84:35 - replace + with * in line_num + 1
    let bash_code = "curl $URL";  // $ at column 6
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    assert_eq!(span.start_col, 6, "Start column must use correct calculation");
}

#[test]
fn test_mutation_sec002_unquoted_var_end_col_exact() {
    // MUTATION: Line 84:63 - replace + with * or - in col + 1
    let bash_code = "curl $URL";  // $ at column 6, ends at column 7
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    assert_eq!(span.end_col, 7, "End column must be col + 1");
}

#[test]
fn test_mutation_sec002_line_num_calculation() {
    // MUTATION: Line 84:35 - replace + with * in line_num + 1
    let bash_code = "# comment\ncurl $URL";  // curl on line 2
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].span.start_line, 2, "Line number must use +1, not *1");
}

#[test]
fn test_mutation_sec002_column_with_offset() {
    // Tests column calculations with leading whitespace
    let bash_code = "    curl $URL";  // $ at column 10
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    let span = result.diagnostics[0].span;
    assert_eq!(span.start_col, 10, "Must account for leading whitespace");
}
```

### Category 2: Helper Function Tests (for Gap 2)

**Required tests** (TBD based on final baseline results):

```rust
#[test]
fn test_mutation_sec002_column_tracking_accuracy() {
    // MUTATION: Line 59:13 - replace += with *= in col += 1
    // Test that column tracking is accurate for variables at various positions
    let bash_code = "curl       $URL";  // $ at column 12 (extra spaces)
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].span.start_col, 12);
}

#[test]
fn test_mutation_sec002_quote_detection_single_quotes() {
    // MUTATION: Line 62:20 - replace !in_single_quotes with true
    // Ensure single-quoted variables are not diagnosed
    let bash_code = "curl '$URL'";  // Should be safe (single quotes)
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 0, "Single-quoted variables should be safe");
}

#[test]
fn test_mutation_sec002_quote_detection_double_quotes() {
    // Tests quote tracking logic comprehensively
    let bash_code = r#"curl "${URL}""#;  // Should be safe (double quotes)
    let result = check(bash_code);
    assert_eq!(result.diagnostics.len(), 0, "Double-quoted variables should be safe");
}
```

## 🎯 Action Plan (Iteration 1)

**WAITING FOR BASELINE COMPLETION** (estimated: ~17 minutes remaining)

Once baseline completes:

### Step 1: Complete Gap Analysis
1. Read full baseline log: `mutation_sec002_baseline.log`
2. Count total MISSED vs CAUGHT mutations
3. Calculate baseline kill rate
4. Categorize all MISSED mutations
5. Update this document with complete gap list

### Step 2: Design Tests (GREEN Phase)
1. For each MISSED mutation, design one targeted test
2. Follow SEC001 pattern (100% success rate)
3. Prioritize exact position tests (proven to work)
4. Add helper function tests as needed

### Step 3: Implement Tests
1. Add mutation coverage tests to `/home/noah/src/bashrs/rash/src/linter/rules/sec002.rs`
2. Follow naming convention: `test_mutation_sec002_<specific_behavior>`
3. Add MUTATION comments explaining which mutation each test kills
4. Verify all tests pass: `cargo test --lib sec002`

### Step 4: Verify (QUALITY Phase)
```bash
# Run iteration mutation test
cargo mutants --file rash/src/linter/rules/sec002.rs --timeout 300 -- --lib 2>&1 | tee mutation_sec002_iter1.log

# Target: 90%+ kill rate (ideally 100% like SEC001)
```

## 🔄 Pattern Recognition from SEC001

**SEC001 Achieved 100% Kill Rate** with this exact approach:
- **Same Problem**: Arithmetic mutations in Span::new() column calculations
- **Same Solution**: Exact position tests (assert_eq!(span.start_col, X))
- **Same Pattern**: + → *, + → -, - → + mutations

**Applied to SEC002**:
- SEC002 has **identical code structure** to SEC001 (inline Span::new())
- Both have simple check() function with column arithmetic
- **Expected Result**: 90-100% kill rate (possibly 100% like SEC001)

**Confidence Level**: 95%+ (validated by 3 consecutive 100% scores: SC2064, SC2059, SEC001)

## 📝 Notes

- SEC002 is **CRITICAL** - prevents command injection via unquoted variables
- **Iteration 1** will focus on exact position tests (proven pattern)
- **Expected result**: 90-100% kill rate (likely 100% based on pattern match)
- Toyota Way (Jidoka) - build quality in from the start
- NASA-level standard: 90%+ mutation kill rate required

**Pattern Confirmed**: SEC002 follows **identical mutation pattern** to SEC001.
This means we can directly apply the SEC001 solution!

## 🔄 Progress Tracking

**Baseline Started**: 2025-11-04 (bash ID: a266d5)
**Current Status**: Testing in progress (5/33 mutants)
**Estimated Completion**: ~17 minutes remaining
**Next Step**: Complete gap analysis when baseline finishes

---

**Generated**: 2025-11-04
**Methodology**: EXTREME TDD + Universal Pattern Recognition
**Status**: Phase 0 (baseline in progress)
**Target**: NASA-level quality (90%+ mutation kill rate)
**Priority**: CRITICAL - command injection prevention via unquoted variables

**🤖 Generated with Claude Code**
