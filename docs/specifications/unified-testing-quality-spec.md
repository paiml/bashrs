# Unified Testing and Quality Specification

**Status**: Draft
**Version**: 1.0.0
**Last Updated**: 2025-11-11

## Purpose

This specification defines a unified testing and quality framework for bashrs to ensure consistent, comprehensive test coverage across all supported file types (bash scripts, Makefiles, Dockerfiles).

## Scope

**In Scope**:
- Unit testing standards for all file types
- Property-based testing requirements
- Mutation testing thresholds
- Test naming conventions
- Coverage targets
- Quality gates

**Out of Scope**:
- End-to-end integration testing (separate spec)
- Performance benchmarking (separate spec)
- Production deployment testing

## Testing Capabilities by File Type

### script.sh (Bash Scripts)

**Current State**: Most comprehensive (6517+ tests)

**Capabilities**:
1. **Unit Tests**: All bash parser functions tested
2. **Property-Based Tests**: 100+ generated cases per feature (proptest)
3. **Mutation Tests**: >90% kill rate target
4. **CLI Tests**: `assert_cmd` for all CLI operations
5. **Purification Tests**: Determinism and idempotency validation
6. **Linter Tests**: 14 security and quality rules
7. **REPL Tests**: Interactive bash shell testing

**Example Test**:
```rust
#[test]
fn test_bash_parse_variable_expansion() {
    let script = r#"echo ${VAR:-default}"#;
    let result = parse_bash(script);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().statements.len(), 1);
}
```

### Makefile

**Current State**: Unique auto-test generation capability

**Capabilities**:
1. **Unit Tests**: Parser and generator function tests
2. **Auto-Generated Tests**: `--with-tests` flag creates executable test suites
3. **Determinism Tests**: Multiple runs produce identical output
4. **Idempotency Tests**: Safe to re-run
5. **POSIX Compliance Tests**: Generated tests verify shellcheck compliance
6. **Property-Based Tests**: Generative testing for parser invariants
7. **Mutation Tests**: Generator code quality validation

**Example Generated Test**:
```sh
#!/bin/sh
# Auto-generated test suite for Makefile

test_determinism() {
    make clean && make > output1.txt
    make clean && make > output2.txt
    diff output1.txt output2.txt || {
        echo "FAIL: Non-deterministic output"
        exit 1
    }
}
```

**Example Unit Test**:
```rust
#[test]
fn test_make_purify_with_tests_flag() {
    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("Makefile")
        .arg("--with-tests")
        .arg("-o")
        .arg("Makefile.purified")
        .assert()
        .success();

    assert!(Path::new("Makefile.purified.test.sh").exists());
}
```

### Dockerfile

**Current State**: 16 CLI tests for 6 transformations

**Capabilities**:
1. **CLI Tests**: All transformations tested via `assert_cmd`
2. **Transformation Tests**: Each DOCKER rule validated
3. **Idempotency Tests**: Transformations safe to re-run
4. **Property-Based Tests**: Planned (not yet implemented)
5. **Mutation Tests**: Planned (not yet implemented)

**Example Test**:
```rust
#[test]
fn test_dockerfile_docker001_replaces_from_latest() {
    let dockerfile = "FROM ubuntu:latest\nRUN apt-get update";
    let input_file = create_temp_dockerfile(dockerfile);

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(input_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("FROM ubuntu:stable-slim"));
}
```

## Quality Targets

### Coverage Targets (MANDATORY)

| File Type | Target | Current | Status |
|-----------|--------|---------|--------|
| script.sh parser | >85% | ~90% | ✅ Met |
| script.sh linter | >85% | ~85% | ✅ Met |
| Makefile parser | >85% | ~88% | ✅ Met |
| Makefile generator | >85% | ~90% | ✅ Met |
| Dockerfile CLI | >85% | ~75% | ⚠️ Below target |

### Mutation Testing Targets (MANDATORY)

| Module | Target Kill Rate | Status |
|--------|------------------|--------|
| bash_parser | >90% | ✅ Met |
| make_parser | >90% | ⚠️ In progress (Issue #3) |
| linter | >90% | ✅ Met |
| CLI commands | >90% | ⏳ Not yet measured |

### Property Testing Targets (MANDATORY)

| Feature | Minimum Cases | Status |
|---------|--------------|--------|
| Bash parsing | 100+ | ✅ Met |
| Make parsing | 100+ | ✅ Met |
| Purification | 100+ | ✅ Met |
| Linting | 100+ | ✅ Met |

## Test Naming Convention

**MANDATORY Format**: `test_<TASK_ID>_<feature>_<scenario>`

**Examples**:
- `test_BASH_001_parse_variable_expansion()`
- `test_MAKE_WITH_TESTS_002_determinism_test_passes()`
- `test_DOCKER001_replaces_from_latest()`

**Rationale**:
- Enables traceability to requirements
- Clear feature identification
- Scenario-based organization

## EXTREME TDD Requirements

All new features MUST follow EXTREME TDD workflow:

### Phase 1: RED
- Write failing test first
- Verify test fails with expected error
- Document expected behavior

### Phase 2: GREEN
- Implement minimal code to pass test
- Verify test passes
- No refactoring yet

### Phase 3: REFACTOR
- Clean up implementation
- Extract helper functions
- Ensure complexity <10
- Run clippy, cargo fmt

### Phase 4: VERIFY
- Run full test suite (all 6517+ tests)
- Verify zero regressions
- Check code coverage

### Phase 5: PROPERTY TESTS
- Add property-based tests (100+ cases)
- Verify invariants hold
- Document edge cases

### Phase 6: MUTATION TESTS
- Run mutation testing on new code
- Target >90% kill rate
- Fix under-tested code

### Phase 7: INTEGRATION
- End-to-end workflow tests
- Verify shellcheck compliance
- Test with real-world examples

## CLI Testing Standards

**MANDATORY**: All CLI tests MUST use `assert_cmd` crate.

**Pattern**:
```rust
use assert_cmd::Command;
use predicates::prelude::*;

fn bashrs_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
}

#[test]
fn test_cli_feature() {
    bashrs_cmd()
        .arg("subcommand")
        .arg("input.sh")
        .assert()
        .success()
        .stdout(predicate::str::contains("Expected output"));
}
```

**Prohibited**: Direct use of `std::process::Command` for CLI testing (quality defect).

## Quality Gates (MUST PASS)

Before any release, ALL of the following MUST pass:

1. **Test Suite**: 100% pass rate (6517+ tests)
2. **Coverage**: >85% for all modules
3. **Mutation Score**: >90% kill rate
4. **Clippy**: Zero warnings
5. **Complexity**: Median <10 (max 15 for exceptional cases)
6. **Shellcheck**: All generated scripts pass POSIX compliance
7. **Property Tests**: 100+ cases per feature
8. **Documentation**: All features documented in book/

## Known Gaps and Remediation

### Gap 1: Dockerfile Coverage (75% vs 85% target)

**Remediation Plan**:
1. Add property-based tests for transformations
2. Add mutation testing for CLI commands
3. Increase unit test coverage for edge cases
4. **Target Completion**: v6.35.0

### Gap 2: Makefile Generator Mutation Coverage (Issue #3)

**Remediation Plan**:
1. Run mutation testing on generators.rs
2. Add tests for under-tested paths
3. Target >90% kill rate
4. **Target Completion**: P2 (deferred to v6.36.0+)

### Gap 3: CLI Command Mutation Testing

**Remediation Plan**:
1. Establish baseline mutation score
2. Add property tests for CLI edge cases
3. Increase test coverage for error paths
4. **Target Completion**: v6.36.0

## Continuous Improvement

**Kaizen Principles**:
1. Monitor test coverage weekly
2. Add property tests for new features immediately
3. Run mutation tests monthly
4. Update this spec as new patterns emerge
5. Zero tolerance for test regressions

## Verification Commands

```bash
# Run all tests
cargo test --lib

# Measure coverage
cargo llvm-cov --lib --text

# Run property tests
cargo test --lib --release -- --include-ignored

# Run mutation tests
cargo mutants --file rash/src/bash_parser/codegen.rs -- --lib

# Verify CLI tests use assert_cmd
grep -r "std::process::Command" rash/tests/*.rs

# Check shellcheck compliance
shellcheck -s sh examples/*.sh
```

## References

- CLAUDE.md: Project development guidelines
- Issue #3: Mutation coverage tracking
- BASH-INGESTION-ROADMAP.yaml: Feature roadmap
- EXTREME TDD: Red-Green-Refactor-Verify workflow

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-11 | Initial specification |
