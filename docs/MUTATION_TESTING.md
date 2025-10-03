# Mutation Testing Guide for Rash

This guide explains how to use mutation testing to improve test quality in the Rash project.

---

## What is Mutation Testing?

Mutation testing validates test quality by introducing small bugs (mutations) into your code and checking if tests catch them. A "killed" mutant means tests detected the bug. A "surviving" mutant reveals a gap in test coverage.

**Key Metrics**:
- **Mutation Kill Rate**: Percentage of mutants killed by tests (target: >95%)
- **Survivors**: Mutants that tests didn't catch (these are test gaps)

---

## Tools

We use [`cargo-mutants`](https://mutants.rs/) for mutation testing:

```bash
# Install
cargo install cargo-mutants

# Run on single file
cargo mutants --file rash/src/ir/mod.rs -- --lib

# Run on whole package
cargo mutants -- --lib
```

---

## Current Status (v0.9.3)

### Sprint 29 Results

**File**: `rash/src/ir/mod.rs`
- **Baseline**: 47 mutants, 39 killed, 8 survived (82.9% kill rate)
- **After Sprint 29**: Added 8 targeted tests to kill survivors
- **Estimated**: ~95%+ kill rate

**Tests Added**:
- `test_function_body_length_calculation` - Kills line 61 arithmetic mutations
- `test_should_echo_guard_conditions` - Kills line 95 guard mutations
- `test_equality_operator_conversion` - Kills line 327 BinaryOp::Eq
- `test_subtraction_operator_conversion` - Kills line 363 BinaryOp::Sub
- `test_curl_command_network_effect` - Kills line 391 curl detection
- `test_wget_command_network_effect` - Kills line 391 wget detection
- `test_non_network_command_no_effect` - Validates effect detection

---

## Workspace Configuration Issue

**Problem**: The `rash-mcp` workspace member has an external dependency (`pforge-runtime`) that causes mutation testing to fail when copying the workspace.

**Error**:
```
error: failed to load manifest for workspace member /tmp/cargo-mutants-rash-xxx/rash-mcp
failed to read `/tmp/pforge/crates/pforge-runtime/Cargo.toml`: No such file or directory
```

### Workaround 1: Manual File-by-File Testing

Test core modules individually:

```bash
# Test IR module
cargo mutants --file rash/src/ir/mod.rs -- --lib

# Test stdlib
cargo mutants --file rash/src/stdlib.rs -- --lib

# Test emitter
cargo mutants --file rash/src/emitter/posix.rs -- --lib
```

### Workaround 2: Temporary Workspace Modification

For comprehensive testing, temporarily remove `rash-mcp` from workspace:

```bash
# 1. Edit Cargo.toml - remove "rash-mcp" from members
# 2. Run full mutation suite
cargo mutants -- --lib

# 3. Restore Cargo.toml
git restore Cargo.toml
```

### Workaround 3: Dedicated CI Job

Create a CI job that runs mutation testing in a clean environment without workspace dependencies.

---

## Running Mutation Tests

### Basic Workflow

1. **Run baseline**:
   ```bash
   cargo mutants --file rash/src/ir/mod.rs -- --lib
   ```

2. **Identify survivors**:
   ```
   MISSED   rash/src/ir/mod.rs:61:60: replace - with + in IrConverter::convert
   ```

3. **Add targeted test** in `rash/src/ir/tests.rs`:
   ```rust
   /// MUTATION KILLER: Line 61 - Arithmetic operator
   #[test]
   fn test_function_body_length_calculation() {
       // Test that verifies len() - 1 is correct
       let ast = RestrictedAst { /* ... */ };
       assert!(from_ast(&ast).is_ok());
   }
   ```

4. **Verify mutation killed**:
   ```bash
   cargo mutants --file rash/src/ir/mod.rs -- --lib
   ```

---

## Configuration

`.cargo/mutants.toml`:
```toml
# Test only bashrs package (workaround for workspace issues)
test_package = ["bashrs"]

# Increase timeout for property tests
timeout_multiplier = 2.0

# Skip test files
exclude_globs = [
    "**/tests/**",
    "**/benches/**",
    "**/*_test.rs",
]
```

---

## Best Practices

### 1. Precision Test Design

Target specific mutation points:

```rust
// ❌ Broad test - might not catch mutations
#[test]
fn test_ast_conversion() {
    let result = convert_ast(...);
    assert!(result.is_ok());
}

// ✅ Targeted test - catches specific operator mutation
#[test]
fn test_equality_operator_conversion() {
    let ast = RestrictedAst {
        value: Expr::Binary { op: BinaryOp::Eq, ... }
    };
    let ir = from_ast(&ast).unwrap();
    assert!(matches!(ir, ShellValue::Comparison { op: ComparisonOp::Eq, ... }));
}
```

### 2. Document Mutation Targets

Add comments explaining which mutants each test kills:

```rust
/// MUTATION KILLER: Line 327 - BinaryOp::Eq match arm
/// Kills mutant: "delete match arm BinaryOp::Eq"
#[test]
fn test_equality_operator_conversion() {
    // ...
}
```

### 3. Test Edge Cases

Mutation survivors often reveal missing edge case tests:
- Boundary conditions (len - 1, len + 1, len / 1)
- Operator variations (==, !=, <, >, <=, >=)
- Guard conditions (true vs false)
- Error paths

### 4. Property-Based Tests

Use proptest for comprehensive coverage:

```rust
proptest! {
    #[test]
    fn prop_binary_operators_convert(op in prop_binary_op()) {
        let ast = make_ast_with_op(op);
        assert!(from_ast(&ast).is_ok());
    }
}
```

---

## Interpreting Results

### Good Kill Rate: >95%

```
47 mutants tested: 45 killed, 2 survived
Kill rate: 95.7%
```

**Action**: Investigate the 2 survivors, add targeted tests if they represent real bugs.

### Low Kill Rate: <85%

```
47 mutants tested: 35 killed, 12 survived
Kill rate: 74.5%
```

**Action**: Systematic review needed. Survivors indicate significant test gaps.

### Common Survivor Patterns

1. **Arithmetic Operators**: `- vs + vs *` in calculations
   - **Fix**: Add tests with specific expected values

2. **Guard Conditions**: `if condition` → `if true` / `if false`
   - **Fix**: Test both branches explicitly

3. **Match Arms**: Deleting specific match cases
   - **Fix**: Test each enum variant explicitly

4. **Effect Detection**: Removing effect classification logic
   - **Fix**: Assert on specific effects

---

## Future Improvements

### Short Term

1. **Resolve Workspace Issue**: Fix `rash-mcp` dependency or move it out of workspace
2. **CI Integration**: Add mutation testing to GitHub Actions
3. **Baseline Tracking**: Store mutation baselines for regression detection

### Long Term

1. **Automated Test Generation**: Generate tests from mutation survivors
2. **Differential Mutation**: Only test changed code
3. **Custom Mutators**: Add Rash-specific mutation operators

---

## Resources

- [cargo-mutants Documentation](https://mutants.rs/)
- [Sprint 29 Report](../.quality/sprint29-complete.md) - Detailed mutation testing case study

---

**Last Updated**: 2025-10-03
**Version**: 0.9.3
**Status**: Documented with workarounds
