# Mutation Testing Baseline

**Last Updated**: 2025-11-25
**Tool**: cargo-mutants v25.3.1
**Target**: ≥80% mutation score (EXTREME TDD standard)

## Overview

Mutation testing verifies test quality by introducing small code changes (mutants) and checking if tests catch them. A high mutation score indicates effective tests.

## Running Mutation Tests

```bash
# Full mutation testing (slow, ~30-60 minutes)
make mutants

# Test specific module
make mutation-file FILE=rash/src/linter/rules/det001.rs

# Quick test on recent changes
make mutants-quick

# View results
make mutants-report
```

## Baseline Scores

| Module | Mutants | Killed | Timeout | Missed | Score | Status |
|--------|---------|--------|---------|--------|-------|--------|
| `linter/rules/det001.rs` | 9 | TBD | TBD | TBD | TBD | Pending |
| `linter/rules/sc2086.rs` | TBD | TBD | TBD | TBD | TBD | Pending |
| `linter/autofix.rs` | TBD | TBD | TBD | TBD | TBD | Pending |
| `bash_transpiler/purification.rs` | TBD | TBD | TBD | TBD | TBD | Pending |

### Status Legend
- **Pass**: ≥80% mutation score
- **Warning**: 60-79% mutation score
- **Fail**: <60% mutation score
- **Pending**: Not yet measured

## Interpreting Results

### Killed Mutants (Good)
Tests detected the code change and failed appropriately.

### Missed Mutants (Needs Improvement)
Code changed but no test failed - indicates weak assertions or missing test coverage.

### Timeout Mutants (Neutral)
Mutant caused infinite loop - typically acceptable.

### Unviable Mutants (Ignore)
Mutant created code that doesn't compile - not counted.

## Improving Low Scores

1. **Review missed mutants**: `cat mutants.out/missed.txt`
2. **Add property tests**: Cover edge cases with proptest
3. **Strengthen assertions**: Ensure tests verify return values
4. **Test error paths**: Ensure error conditions are tested

## Configuration

See `.cargo/mutants.toml` for mutation testing configuration.

```toml
# Current configuration
test_package = "bashrs"
timeout_multiplier = 2.0
exclude_re = ["tests/", "benches/", "*_test.rs"]
```

## CI/CD Integration

Mutation testing is NOT run in CI due to long execution time.
Run manually before major releases:

```bash
make mutants  # Full analysis
make mutants-report  # View summary
```

## References

- [cargo-mutants documentation](https://github.com/sourcefrog/cargo-mutants)
- [EXTREME TDD methodology](./CLAUDE.md)
- [Issue #54](https://github.com/paiml/bashrs/issues/54)
