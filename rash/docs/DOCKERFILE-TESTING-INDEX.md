# Dockerfile Testing Parity - Documentation Index

Complete implementation plan for achieving testing parity between Dockerfile and Makefile/script.sh transformations. Created November 11, 2025.

## Quick Links

### Main Documents
1. **[DOCKERFILE-TESTING-PARITY-PLAN.md](./DOCKERFILE-TESTING-PARITY-PLAN.md)** (1764 lines, 50KB)
   - Full detailed implementation plan
   - All 6 phases with breakdown
   - EXTREME TDD approach
   - Timeline, risks, resources
   - **Read this first for comprehensive overview**

2. **[DOCKERFILE-TESTING-QUICK-START.md](./DOCKERFILE-TESTING-QUICK-START.md)** (380 lines, 11KB)
   - Quick reference guide
   - Phase summaries with status
   - Commands and quality gates
   - Testing patterns
   - **Use for daily reference during implementation**

3. **DOCKERFILE-TESTING-INDEX.md** (this file)
   - Navigation guide
   - Document overview
   - Section breakdown
   - **Use to find what you need**

## Current State Analysis

### Test Coverage Summary
```
Current (November 11, 2025):
├── CLI Tests: 16
├── Unit Tests: 0
├── Property Tests: 14 (with 10 proptest blocks)
├── Integration Tests: 0
├── Code Coverage: ~75%
├── Mutation Testing: None
└── Total Tests: 30

Target (December 20, 2025):
├── CLI Tests: 35+
├── Unit Tests: 52+
├── Property Tests: 40+ (with 100+ cases per block)
├── Integration Tests: 20+
├── Code Coverage: >85%
├── Mutation Testing: >90% kill rate
└── Total Tests: 147+

Improvements: 30 → 147+ (4.9x increase)
```

### File Locations

**Current Tests**:
- `/home/noah/src/bashrs/rash/tests/cli_dockerfile_purify.rs` (16 tests)
- `/home/noah/src/bashrs/rash/tests/property_dockerfile_purify.rs` (14 property blocks)

**Implementation Files**:
- `/home/noah/src/bashrs/rash/src/linter/rules/docker001.rs` (User directive)
- `/home/noah/src/bashrs/rash/src/linter/rules/docker002.rs` (Image pinning)
- `/home/noah/src/bashrs/rash/src/linter/rules/docker003.rs` (Cleanup)
- `/home/noah/src/bashrs/rash/src/linter/rules/docker004.rs` (Health checks)
- `/home/noah/src/bashrs/rash/src/linter/rules/docker005.rs` (Package flags)
- `/home/noah/src/bashrs/rash/src/linter/rules/docker006.rs` (ADD to COPY)

## Implementation Phases

### Phase 1: Test Infrastructure Enhancement (40 hours)
**Duration**: Weeks 1-2
**Status**: Ready to start

**Deliverables**:
- Expand CLI tests from 16 to 35+ (add 19 tests)
- Generate 20+ property test blocks (100+ cases each)
- Document 40+ edge cases

**Key File**: See "Phase 1: Test Infrastructure Enhancement" section in DOCKERFILE-TESTING-PARITY-PLAN.md

---

### Phase 2: Unit Test Expansion (60 hours)
**Duration**: Weeks 3-4
**Status**: Depends on Phase 1

**Deliverables**:
- Add 52+ new unit tests
- Add 20+ integration tests
- Target >85% coverage per module

**Key File**: See "Phase 2: Unit Test Expansion" section in DOCKERFILE-TESTING-PARITY-PLAN.md

**Per-Rule Tests**:
- docker001.rs: +10 tests
- docker002.rs: +12 tests
- docker003.rs: +8 tests
- docker004.rs: +8 tests
- docker005.rs: +8 tests
- docker006.rs: +6 tests

---

### Phase 3: Mutation Testing Implementation (80 hours)
**Duration**: Weeks 5-6
**Status**: Depends on Phase 2

**Deliverables**:
- cargo-mutants infrastructure
- >90% kill rate per rule
- CI/CD integration

**Key File**: See "Phase 3: Mutation Testing Implementation" section in DOCKERFILE-TESTING-PARITY-PLAN.md

**Setup**:
```bash
cargo install cargo-mutants
# Create mutants.toml
cargo mutants --file src/linter/rules/docker*.rs
```

---

### Phase 4: Coverage Analysis & Gap Filling (60 hours)
**Duration**: Weeks 7-8
**Status**: Depends on Phase 3

**Deliverables**:
- >85% coverage per module
- 40+ gap-filling tests
- Gap analysis documented

**Key File**: See "Phase 4: Coverage Analysis & Gap Filling" section in DOCKERFILE-TESTING-PARITY-PLAN.md

---

### Phase 5: Documentation & Specification (40 hours)
**Duration**: Weeks 8-9
**Status**: Depends on Phase 4

**Deliverables**:
- Update unified-testing-quality-spec.md
- Create DOCKERFILE-TESTING-ROADMAP.yaml
- Create DOCKERFILE-TESTING-GUIDE.md

**Key File**: See "Phase 5: Documentation & Specification" section in DOCKERFILE-TESTING-PARITY-PLAN.md

---

### Phase 6: Release & Verification (40 hours)
**Duration**: Week 10
**Status**: Depends on Phase 5

**Deliverables**:
- Final verification
- CHANGELOG update
- v7.0.0 release
- Post-release retrospective

**Key File**: See "Phase 6: Release & Post-Release" section in DOCKERFILE-TESTING-PARITY-PLAN.md

---

## Testing Approach

### EXTREME TDD Methodology

Each feature follows: **RED → GREEN → REFACTOR → VERIFY**

1. **RED Phase**: Write failing tests first
   - Tests should fail initially (correct!)
   - Property tests generate 100+ cases minimum
   - All code paths defined through tests

2. **GREEN Phase**: Make tests pass
   - 100% pass rate required
   - Code complexity <10
   - All code paths covered

3. **REFACTOR Phase**: Polish code
   - Extract helper functions
   - Improve readability
   - Ensure quality standards

4. **VERIFY Phase**: Mutation testing
   - Run mutations
   - Target >90% kill rate
   - Document survivors

**See**: "EXTREME TDD Workflow" section in DOCKERFILE-TESTING-QUICK-START.md

---

## Quality Gates

### Before Committing
```bash
cargo test --lib src/linter/rules/docker*.rs
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
```

### Before Pull Request
```bash
cargo test --test cli_dockerfile_*
cargo test --test property_dockerfile_*
cargo llvm-cov --test cli_dockerfile_*
```

### Before Release
```bash
cargo test --all-targets
cargo llvm-cov --fail-under-lines 85
cargo mutants --fail-if-below 90
cargo clippy -- -D warnings
```

**See**: "Quality Gates" section in DOCKERFILE-TESTING-QUICK-START.md

---

## Testing Commands Reference

### Run All Dockerfile Tests
```bash
cargo test --lib src/linter/rules/docker*.rs
cargo test --test cli_dockerfile_*
cargo test --test property_dockerfile_*
```

### Coverage Analysis
```bash
cargo llvm-cov --test cli_dockerfile_* --html
open target/coverage/dockerfile/index.html
cargo llvm-cov --test cli_dockerfile_* --fail-under-lines 85
```

### Mutation Testing
```bash
cargo install cargo-mutants
cargo mutants --file src/linter/rules/docker*.rs
cargo mutants --fail-if-below 90
```

**See**: "Testing Commands" section in DOCKERFILE-TESTING-QUICK-START.md

---

## Key Metrics

### Timeline
- **Start Date**: November 11, 2025
- **Target Release**: December 20, 2025 (v7.0.0)
- **Duration**: 8-10 weeks
- **Effort**: 280-320 hours

### Coverage Goals
- **Code Coverage**: 75% → >85% (10% improvement)
- **Mutation Kill Rate**: 0% → >90% (new capability)
- **Total Tests**: 30 → 147+ (4.9x increase)

### Per-Category Improvements
- **CLI Tests**: 16 → 35+ (2.2x)
- **Unit Tests**: 0 → 52+ (new)
- **Property Tests**: 14 → 40+ (3x)
- **Integration Tests**: 0 → 20+ (new)

**See**: "Key Statistics" section in DOCKERFILE-TESTING-QUICK-START.md

---

## Risk Management

### High-Risk Areas
1. **Mutation Testing Difficulty**
   - Mitigation: Start with 80%, gradually increase
   - Fallback: Accept >85% if 90% unachievable

2. **Multi-Stage Build Complexity**
   - Mitigation: Dedicated generators + integration tests
   - Fallback: Extend Phase 2 by 1-2 weeks

3. **Performance Impact**
   - Mitigation: Parallel execution, benchmarking
   - Fallback: <10% slowdown acceptable

**See**: "Risk Assessment" section in DOCKERFILE-TESTING-PARITY-PLAN.md

---

## Success Criteria

### Must Have (Blocking)
- 147+ total tests
- >85% code coverage per module
- >90% mutation kill rate per rule
- 100% test pass rate
- STOP THE LINE protocol enforced

### Should Have (Important)
- Documentation complete
- <10% CI/CD time increase
- Patterns repeatable for future work
- Developer confidence high

**See**: "Success Metrics & Quality Gates" section in DOCKERFILE-TESTING-PARITY-PLAN.md

---

## Next Steps

### Immediate (Before Starting)
1. Review DOCKERFILE-TESTING-PARITY-PLAN.md (full plan)
2. Review DOCKERFILE-TESTING-QUICK-START.md (quick ref)
3. Approve Phase 1 implementation
4. Set up sprint board with phases

### Week 1 (Phase 1 Start)
1. Extend CLI tests from 16 to 35+ (RED phase)
2. Add property test blocks (RED phase)
3. Document edge cases
4. Verify all new tests fail (this is correct!)

### Weekly During Implementation
1. Status check-ins
2. Address blockers
3. Follow EXTREME TDD strictly
4. Document progress

### After Completion
1. Release v7.0.0 with testing parity
2. Document lessons learned
3. Plan Phase 7 (Rust → Shell v3.0)

**See**: "Next Steps" section in DOCKERFILE-TESTING-QUICK-START.md

---

## References

### EXTREME TDD Guidelines
- **File**: `/home/noah/src/bashrs/CLAUDE.md`
- **Section**: "EXTREME TDD Definition"
- **Topics**: RED/GREEN/REFACTOR/VERIFY, property testing, mutation testing

### Test Examples
- **Makefile Tests**: `/home/noah/src/bashrs/rash/tests/makefile_parsing.rs`
- **Script Tests**: 6004+ tests in `/home/noah/src/bashrs/rash/tests/`
- **Current Dockerfile Tests**: `/home/noah/src/bashrs/rash/tests/cli_dockerfile_purify.rs`

### Existing Infrastructure
- **Coverage Tool**: cargo-llvm-cov (already installed)
- **Property Testing**: proptest 1.0+ (already in Cargo.toml)
- **CLI Testing**: assert_cmd 2.0 (already in Cargo.toml)
- **Mutation Testing**: cargo-mutants (to install)

---

## Document Maintenance

### How to Use These Documents

1. **First Time**: Read DOCKERFILE-TESTING-PARITY-PLAN.md (full overview)
2. **During Implementation**: Reference DOCKERFILE-TESTING-QUICK-START.md (daily guide)
3. **Finding Specific Info**: Use DOCKERFILE-TESTING-INDEX.md (this file)

### Document Updates

- Update DOCKERFILE-TESTING-QUICK-START.md with Phase status as you progress
- Add Phase-specific lessons learned to this index
- Create new sections as needed (e.g., "Lessons Learned", "Blocked Items")

### Keeping Plans Current

- Weeks 1-2: Update Phase 1 status
- Weeks 3-4: Update Phase 2 status
- Weeks 5-6: Update Phase 3 status
- Weeks 7-8: Update Phase 4 status
- Weeks 8-9: Update Phase 5 status
- Week 10: Update Phase 6 status + retrospective

---

## Getting Help

### Questions About Implementation Plan
- See DOCKERFILE-TESTING-PARITY-PLAN.md
- Section: "Phase X: [Phase Name]"

### Questions About Daily Workflow
- See DOCKERFILE-TESTING-QUICK-START.md
- Section: "Testing Commands" or "EXTREME TDD Workflow"

### Questions About EXTREME TDD
- See `/home/noah/src/bashrs/CLAUDE.md`
- Section: "EXTREME TDD Definition"

### Questions About Test Patterns
- See `/home/noah/src/bashrs/rash/tests/makefile_parsing.rs` (Makefile reference)
- See `/home/noah/src/bashrs/rash/tests/` (6004+ script tests reference)

---

**Index Document**: DOCKERFILE-TESTING-INDEX.md
**Created**: November 11, 2025
**Status**: ✅ Complete
**Next**: Review DOCKERFILE-TESTING-PARITY-PLAN.md and approve Phase 1
