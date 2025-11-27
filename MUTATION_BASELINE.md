# Mutation Testing Baseline

**Status**: Infrastructure Complete (Issue #54)
**Target**: ≥80% mutation score
**Tool**: cargo-mutants v25.3.1

## Quick Start

```bash
# Full mutation testing (30-60 minutes)
make mutants

# Single file mutation testing (5-10 minutes)
make mutation-file FILE=rash/src/linter/rules/det001.rs

# Changed files only (quick, <10 minutes)
make mutants-quick

# View results
make mutants-report
```

## Baseline Measurement Process

### Step 1: Core Module Testing

Test these modules first (highest impact):

```bash
# Determinism rules (DET001-DET003)
make mutation-file FILE=rash/src/linter/rules/det001.rs
make mutation-file FILE=rash/src/linter/rules/det002.rs
make mutation-file FILE=rash/src/linter/rules/det003.rs

# Word splitting (SC2086)
make mutation-file FILE=rash/src/linter/rules/sc2086.rs

# Autofix engine
make mutation-file FILE=rash/src/linter/autofix.rs

# Purification
make mutation-file FILE=rash/src/bash_transpiler/purification.rs
```

### Step 2: Recording Results

After each run, record in this table:

| Module | Total Mutants | Killed | Timeout | Missed | Score | Date |
|--------|--------------|--------|---------|--------|-------|------|
| det001 | TBD | TBD | TBD | TBD | TBD | - |
| det002 | TBD | TBD | TBD | TBD | TBD | - |
| det003 | TBD | TBD | TBD | TBD | TBD | - |
| sc2086 | TBD | TBD | TBD | TBD | TBD | - |
| autofix | TBD | TBD | TBD | TBD | TBD | - |
| purification | TBD | TBD | TBD | TBD | TBD | - |

### Step 3: Analyzing Missed Mutants

```bash
# View missed mutants
cat mutants.out/missed.txt

# Analyze specific mutations
cat mutants.out/mutants.json | jq '.mutants[] | select(.outcome == "Missed")'
```

## Configuration

Configuration file: `.cargo/mutants.toml`

```toml
# Test only the main bashrs package
test_package = ["bashrs"]

# Timeout multiplier for property tests
timeout_multiplier = 2.0

# Files to skip
exclude_globs = [
    "**/tests/**",
    "**/benches/**",
    "**/*_test.rs",
]
```

## Known Issues

### Path Dependency (verificar)

The `verificar` crate uses a relative path dependency that breaks `cargo mutants` default mode.

**Workaround**: Use `--in-place` flag or the Makefile targets which handle this automatically.

### Long Running Tests

Full mutation testing takes 30-60 minutes. Use `make mutants-quick` for faster feedback.

## Improving Mutation Score

If mutation score is below 80%:

1. **Add property tests** to cover edge cases
2. **Strengthen assertions** (check error types, not just success)
3. **Add boundary tests** for numeric operations
4. **Test error paths** not just happy paths

Example improvement:

```rust
// Before: Weak assertion
#[test]
fn test_det001() {
    let result = check("echo $RANDOM");
    assert!(!result.diagnostics.is_empty());
}

// After: Strong assertion (kills more mutants)
#[test]
fn test_det001_detects_random() {
    let result = check("echo $RANDOM");
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "DET001");
    assert!(result.diagnostics[0].message.contains("non-deterministic"));
}
```

## CI/CD Integration

Future enhancement: Add mutation testing to GitHub Actions

```yaml
# .github/workflows/mutation.yml
name: Mutation Testing
on:
  schedule:
    - cron: '0 4 * * 0'  # Weekly on Sunday 4am
jobs:
  mutants:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install cargo-mutants
      - run: make mutants-quick
      - uses: actions/upload-artifact@v4
        with:
          name: mutation-report
          path: mutants.out/
```

## References

- [CLAUDE.md](./CLAUDE.md): EXTREME TDD methodology
- [cargo-mutants documentation](https://mutants.rs/)
- [EXTREME TDD](https://claude.ai/docs/extreme-tdd): Mutation testing as quality verification
