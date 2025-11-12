# Dockerfile Testing Parity - Quick Start Guide

## Overview

This document provides a quick reference for implementing Dockerfile testing parity. For detailed information, see `/home/noah/src/bashrs/rash/docs/DOCKERFILE-TESTING-PARITY-PLAN.md`.

## Key Statistics

- **Current Tests**: 16 CLI + 14 property = 30 total
- **Target Tests**: 35+ CLI + 52+ unit + 40+ property + 20+ integration = 147+ total
- **Coverage**: 75% → >85% (gap: 10%)
- **Mutation Testing**: None → >90% kill rate
- **Timeline**: 8-10 weeks (280-320 hours)
- **Release**: December 20, 2025 (v7.0.0)

## Implementation Phases

### Phase 1: Test Infrastructure (Weeks 1-2, 40 hours)
**Status**: Ready to start
**Deliverables**:
- 35+ CLI tests (RED phase)
- 20+ property test blocks (RED phase)
- 40+ edge case documentation

**Key Files**:
- `/home/noah/src/bashrs/rash/tests/cli_dockerfile_purify.rs` (add 19 tests)
- `/home/noah/src/bashrs/rash/tests/property_dockerfile_purify.rs` (add 10 blocks)
- New: `/home/noah/src/bashrs/rash/docs/DOCKERFILE-EDGE-CASES.md`

**Expected Outcome**: All new tests fail (RED phase - this is correct!)

---

### Phase 2: Unit Test Expansion (Weeks 3-4, 60 hours)
**Prerequisites**: Phase 1 complete
**Deliverables**:
- 52+ new unit tests (GREEN phase)
- 20+ integration tests (GREEN phase)
- All tests passing

**Per-Rule Breakdown**:
```
docker001.rs: +10 unit tests
docker002.rs: +12 unit tests
docker003.rs: +8 unit tests
docker004.rs: +8 unit tests
docker005.rs: +8 unit tests
docker006.rs: +6 unit tests
integration: +20 tests
```

**Key Files**:
- `/home/noah/src/bashrs/rash/src/linter/rules/docker*.rs` (inline tests)
- New: `/home/noah/src/bashrs/rash/tests/cli_dockerfile_integration.rs`

**Expected Outcome**: All tests GREEN (passing)

---

### Phase 3: Mutation Testing (Weeks 5-6, 80 hours)
**Prerequisites**: Phase 2 complete
**Setup**:
```bash
cargo install cargo-mutants
# Create mutants.toml configuration
cargo mutants --file src/linter/rules/docker*.rs
```

**Deliverables**:
- Mutation test infrastructure
- >90% kill rate per rule
- Mutation tests in CI/CD

**Key Files**:
- New: `/home/noah/src/bashrs/rash/mutants.toml`
- Updated: `/home/noah/src/bashrs/rash/Makefile` (add mutation targets)

**Process**:
1. Run mutations
2. Analyze survivors
3. Write tests for missed branches
4. Re-run until >90% kill rate
5. Repeat per rule

**Expected Outcome**: >90% kill rate per rule, documented survivors

---

### Phase 4: Coverage Analysis (Weeks 7-8, 60 hours)
**Prerequisites**: Phase 3 complete
**Tools**:
```bash
cargo llvm-cov --test cli_dockerfile_*
cargo llvm-cov report --html
```

**Deliverables**:
- >85% coverage per module (verified)
- 40+ gap-filling tests
- Gap analysis documented

**Process**:
1. Generate coverage reports
2. Identify gaps per rule
3. Write tests to fill gaps
4. Verify >85% coverage

**Expected Outcome**: Coverage >85% per module, documented gaps

---

### Phase 5: Documentation (Weeks 8-9, 40 hours)
**Prerequisites**: Phases 1-4 complete
**Deliverables**:
- Updated `unified-testing-quality-spec.md`
- New `DOCKERFILE-TESTING-ROADMAP.yaml`
- New `DOCKERFILE-TESTING-GUIDE.md`

**Key Files to Update**:
- `/home/noah/src/bashrs/rash/docs/specifications/unified-testing-quality-spec.md`
- New: `/home/noah/src/bashrs/rash/docs/DOCKERFILE-TESTING-ROADMAP.yaml`
- New: `/home/noah/src/bashrs/rash/docs/guides/DOCKERFILE-TESTING-GUIDE.md`

**Content**:
- Test naming conventions
- Quality gates (RED/GREEN/REFACTOR)
- CI/CD integration specs
- Testing patterns and examples

**Expected Outcome**: Documentation complete, examples compilable

---

### Phase 6: Release & Verification (Week 10, 40 hours)
**Prerequisites**: All phases complete
**Verification Checklist**:
```
[ ] All unit tests passing (100%)
[ ] All CLI tests passing (100%)
[ ] All property tests passing (100+ cases each)
[ ] All integration tests passing (100%)
[ ] Coverage >85% per module
[ ] Mutations >90% kill rate per rule
[ ] CI/CD all green
[ ] No performance regression
[ ] Documentation complete
```

**Release Activities**:
- Update CHANGELOG.md
- Prepare release notes
- Create Git tag
- Verify GitHub release
- Post-release retrospective

**Expected Outcome**: v7.0.0 released with testing parity achieved

---

## Test Organization

```
tests/
├── cli_dockerfile_purify.rs (35+ tests)
│   ├── DOCKER001-006 coverage
│   ├── CLI flag combinations
│   └── Error handling
├── cli_dockerfile_integration.rs (20+ tests)
│   ├── Complete pipeline tests
│   ├── Cross-rule interactions
│   ├── Error recovery
│   ├── Edge cases
│   └── Performance
└── property_dockerfile_purify.rs (40+ property blocks)
    ├── Determinism (1 block)
    ├── Idempotency (1 block)
    ├── Ordering properties (3 blocks)
    ├── Semantic preservation (4 blocks)
    ├── Transformation properties (3 blocks)
    ├── Stress tests (4 tests)
    └── Edge cases (4 tests)

src/linter/rules/
├── docker001.rs (10+ unit tests)
├── docker002.rs (12+ unit tests)
├── docker003.rs (8+ unit tests)
├── docker004.rs (8+ unit tests)
├── docker005.rs (8+ unit tests)
└── docker006.rs (6+ unit tests)
```

---

## Testing Commands

### Run All Dockerfile Tests
```bash
# CLI tests
cargo test --test cli_dockerfile_purify

# Property tests (100+ cases each)
cargo test --test property_dockerfile_purify

# Integration tests
cargo test --test cli_dockerfile_integration

# All Dockerfile tests
cargo test --lib src/linter/rules/docker*.rs
cargo test --test cli_dockerfile_*
cargo test --test property_dockerfile_*
```

### Coverage Analysis
```bash
# Generate coverage report
cargo llvm-cov --test cli_dockerfile_* --html

# View detailed coverage
open target/coverage/dockerfile/index.html

# CI/CD style coverage check
cargo llvm-cov --test cli_dockerfile_* --fail-under-lines 85
```

### Mutation Testing
```bash
# Install mutation testing tool
cargo install cargo-mutants

# Run mutation tests on docker rules
cargo mutants --file src/linter/rules/docker*.rs

# CI/CD style mutation testing (strict)
cargo mutants --file src/linter/rules/docker*.rs --fail-if-below 90
```

### Makefile Targets (To Be Added)
```bash
# Run Dockerfile-specific tests
make test-dockerfile

# Generate Dockerfile coverage
make coverage-dockerfile

# Run Dockerfile mutation tests
make mutate-dockerfile

# Full quality gate check
make dockerfile-quality-gates
```

---

## Quality Gates

### Before Committing
```bash
cargo test --lib src/linter/rules/docker*.rs  # Must pass
cargo clippy --all-targets -- -D warnings     # Must pass
cargo fmt -- --check                           # Must pass
```

### Before Pull Request
```bash
cargo test --test cli_dockerfile_*             # Must pass (100%)
cargo test --test property_dockerfile_*        # Must pass (100+ cases)
cargo llvm-cov --test cli_dockerfile_*         # Must be >85%
```

### Before Release
```bash
cargo test --all-targets                       # All tests pass
cargo llvm-cov --fail-under-lines 85          # Coverage >85%
cargo mutants --fail-if-below 90               # Kill rate >90%
cargo fmt -- --check                           # Format clean
cargo clippy -- -D warnings                    # No warnings
```

---

## EXTREME TDD Workflow

### For Each Feature

1. **RED Phase**: Write failing test
   ```rust
   #[test]
   fn test_DOCKER_001_feature_scenario() {
       // This test should FAIL initially
       assert!(false, "Not implemented yet");
   }
   ```

2. **GREEN Phase**: Implement feature
   ```rust
   // Implementation in docker001.rs
   fn add_user_directive(dockerfile: &str) -> String {
       // Implementation here
   }
   
   // Test now PASSES
   #[test]
   fn test_DOCKER_001_feature_scenario() {
       let result = add_user_directive(INPUT);
       assert!(result.contains("USER"));
   }
   ```

3. **REFACTOR Phase**: Polish code
   - Complexity <10
   - All tests passing
   - Documentation complete
   - Property tests added

4. **VERIFY Phase**: Mutation testing
   - Run: `cargo mutants --file docker001.rs`
   - Verify: >90% kill rate
   - Document: Surviving mutations

---

## Success Criteria

### Quantitative
- ✅ 147+ total tests (was 30)
- ✅ >85% code coverage per module
- ✅ >90% mutation kill rate per rule
- ✅ 0 test failures
- ✅ <10% CI/CD time increase

### Qualitative
- ✅ All quality gates pass
- ✅ Documentation complete
- ✅ Patterns documented for future work
- ✅ Developer confidence high
- ✅ Zero defects (STOP THE LINE protocol)

---

## File Locations

| File | Purpose | Status |
|------|---------|--------|
| `/home/noah/src/bashrs/rash/docs/DOCKERFILE-TESTING-PARITY-PLAN.md` | Full implementation plan (1764 lines) | ✅ Complete |
| `/home/noah/src/bashrs/rash/tests/cli_dockerfile_purify.rs` | CLI tests | ⏳ Extend from 16 to 35+ |
| `/home/noah/src/bashrs/rash/tests/property_dockerfile_purify.rs` | Property tests | ⏳ Extend from 14 to 40+ |
| `/home/noah/src/bashrs/rash/src/linter/rules/docker*.rs` | Unit tests inline | ⏳ Add per-rule unit tests |
| `/home/noah/src/bashrs/rash/Makefile` | Build automation | ⏳ Add mutation targets |
| `/home/noah/src/bashrs/rash/mutants.toml` | Mutation configuration | 📋 To create |
| `/home/noah/src/bashrs/rash/docs/DOCKERFILE-TESTING-ROADMAP.yaml` | Detailed roadmap | 📋 To create |
| `/home/noah/src/bashrs/rash/docs/guides/DOCKERFILE-TESTING-GUIDE.md` | Developer guide | 📋 To create |

---

## Next Steps

1. ✅ Review implementation plan
2. ⏳ Approve Phase 1 (test infrastructure)
3. ⏳ Begin Phase 1 Week 1
4. ⏳ Weekly status check-ins
5. ⏳ Complete all 6 phases
6. ⏳ Release v7.0.0 with testing parity
7. ⏳ Plan Phase 7 (Rust → Shell v3.0)

---

## Support & Questions

For detailed information, see:
- **Full Plan**: `/home/noah/src/bashrs/rash/docs/DOCKERFILE-TESTING-PARITY-PLAN.md`
- **EXTREME TDD Guidelines**: `/home/noah/src/bashrs/CLAUDE.md`
- **Makefile Test Reference**: `/home/noah/src/bashrs/rash/tests/makefile_parsing.rs`
- **Script Test Reference**: 6004+ tests in `/home/noah/src/bashrs/rash/tests/`

---

**Document**: DOCKERFILE-TESTING-QUICK-START.md
**Created**: November 11, 2025
**Status**: ✅ Ready to implement
**Next**: Begin Phase 1
