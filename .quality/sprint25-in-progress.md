# Sprint 25: Mutation Testing Excellence - EXTREME TDD

**Sprint ID**: sprint-25
**Duration**: 2 weeks
**Goal**: Achieve ≥90% mutation kill rate across all core modules
**Status**: Planning
**Quality Grade Target**: Perfect A+ (100/100)
**Start Date**: 2025-10-09

## Sprint Overview

### Objectives
1. Run comprehensive mutation testing on parser, IR, emitter, and verifier modules
2. Identify and close test coverage gaps revealed by surviving mutants
3. Achieve ≥90% mutation kill rate (up from 83% baseline)
4. Improve overall quality score from A+ (98/100) to Perfect A+ (100/100)

### Success Criteria
- [ ] All 5 tickets completed (RED-GREEN-REFACTOR)
- [ ] All tests passing (100% pass rate maintained)
- [ ] Mutation score ≥90% across all core modules
- [ ] Quality gates passed (complexity, coverage, SATD)
- [ ] Documentation updated

## Current Status (Pre-Sprint)

### Quality Metrics Baseline
```yaml
metrics:
  test_count: 667
  test_pass_rate: 100% (667/667 passing, 2 ignored)
  property_tests: 52 (~26,000+ cases)
  coverage_total: 90.53%
  coverage_core: 85.36%
  complexity_median: 1.0
  mutation_score: 83% (IR module baseline)
  satd_comments: 0
  unsafe_blocks: 0
  quality_score: 98/100
  grade: A+
  binary_size: 1.8MB (target: <3MB)
  performance: 19.1µs (target: <10ms)
```

### Tools Installed
- ✅ ShellCheck 0.8.0
- ✅ cargo-llvm-cov 0.6.20
- ⚠️  pmat (not installed, optional)
- ⚠️  cargo-mutants (needs installation)

## Tickets

### RASH-2501: Parser Module Mutation Testing
**Priority**: Critical
**Status**: TODO
**Duration**: 4 hours

#### Requirements
- [ ] Run mutation testing on src/parser/ module
- [ ] Achieve ≥90% mutation kill rate
- [ ] Document all surviving mutants
- [ ] Write targeted tests to kill surviving mutants

#### Tests (RED Phase)
```bash
# Install cargo-mutants if not already installed
cargo install cargo-mutants

# Run mutation tests on parser (RED - expect surviving mutants)
cargo mutants --package bashrs --file src/parser/mod.rs
cargo mutants --package bashrs --file src/parser/parse.rs

# Expected: Some mutants will survive (< 90% kill rate)
```

#### Implementation (GREEN Phase)
Write targeted tests to kill surviving mutants:

```rust
#[test]
fn test_parser_edge_case_from_mutant_X() {
    // Test specifically targets mutant X
    // Should kill the mutant when re-run
}
```

#### Refactoring (REFACTOR Phase)
- [ ] Extract duplicate test setup into helper functions
- [ ] Organize tests by mutant category
- [ ] Add documentation explaining why tests exist
- [ ] Verify no regression in performance

#### Acceptance Criteria
- [ ] Parser module ≥90% mutation kill rate
- [ ] All new tests pass
- [ ] Complexity maintained ≤10
- [ ] Coverage maintained ≥85%
- [ ] No SATD comments
- [ ] Documentation updated

---

### RASH-2502: IR Module Mutation Testing Enhancement
**Priority**: Critical
**Status**: TODO
**Duration**: 4 hours

#### Requirements
- [ ] Run full mutation testing on src/ir/ module
- [ ] Improve from 83% baseline to ≥90% kill rate
- [ ] Target specific surviving mutants from previous analysis
- [ ] Write comprehensive tests for IR conversion edge cases

#### Tests (RED Phase)
```bash
# Run mutation tests on IR module
cargo mutants --package bashrs --file src/ir/mod.rs
cargo mutants --package bashrs --file src/ir/convert.rs

# Current baseline: 83% kill rate
# Target: ≥90% kill rate
```

#### Implementation (GREEN Phase)
Add tests for gaps identified in Sprint 24:

```rust
#[test]
fn test_ir_conversion_edge_case_empty_block() {
    // Targets specific surviving mutant
}

#[test]
fn test_ir_conversion_nested_structures() {
    // Improves branch coverage
}

proptest! {
    #[test]
    fn prop_ir_conversion_never_panics(ast in arbitrary_ast()) {
        // Property: IR conversion should never panic
        let _ = convert_to_ir(&ast);
    }
}
```

#### Acceptance Criteria
- [ ] IR module ≥90% mutation kill rate
- [ ] 8 critical gaps closed (from Sprint 24 analysis)
- [ ] All new tests pass
- [ ] Property test coverage expanded
- [ ] Documentation updated with mutation testing insights

---

### RASH-2503: Emitter Module Mutation Testing
**Priority**: Critical
**Status**: TODO
**Duration**: 5 hours

#### Requirements
- [ ] Run mutation testing on src/emitter/ module
- [ ] Achieve ≥90% mutation kill rate
- [ ] Focus on shell code generation correctness
- [ ] Verify POSIX compliance for all mutants

#### Tests (RED Phase)
```bash
# Run mutation tests on emitter
cargo mutants --package bashrs --file src/emitter/mod.rs
cargo mutants --package bashrs --file src/emitter/emit.rs

# Expected: Surviving mutants in shell code generation
```

#### Implementation (GREEN Phase)
Critical: Emitter generates shell code, mutations are safety-critical:

```rust
#[test]
fn test_emitter_proper_quoting_all_contexts() {
    // Ensures no injection vulnerabilities from mutations
}

#[test]
fn test_emitter_posix_compliance_mutations() {
    // All mutants must still generate POSIX-compliant shell
    // Use ShellCheck to verify
}

proptest! {
    #[test]
    fn prop_emitted_shell_always_shellcheck_clean(ir in arbitrary_ir()) {
        let shell_code = emit_shell(&ir).unwrap();
        // Property: All emitted code must pass ShellCheck
        assert!(shellcheck_passes(&shell_code));
    }
}
```

#### Special Considerations
- **Safety-critical**: Emitter mutations could introduce injection vulnerabilities
- **ShellCheck integration**: Every mutant must still pass ShellCheck
- **Determinism**: Mutations must not break byte-identical output property

#### Acceptance Criteria
- [ ] Emitter module ≥90% mutation kill rate
- [ ] All surviving mutants verified safe (no injection risks)
- [ ] ShellCheck passes for all mutant-generated code
- [ ] Determinism maintained
- [ ] Documentation updated with safety analysis

---

### RASH-2504: Verifier Module Mutation Testing
**Priority**: High
**Status**: TODO
**Duration**: 3 hours

#### Requirements
- [ ] Run mutation testing on src/verifier/ module
- [ ] Achieve ≥90% mutation kill rate
- [ ] Ensure verification logic is sound under mutations
- [ ] Test all verification levels (none, basic, strict, paranoid)

#### Tests (RED Phase)
```bash
# Run mutation tests on verifier
cargo mutants --package bashrs --file src/verifier/mod.rs
cargo mutants --package bashrs --file src/verifier/properties.rs
```

#### Implementation (GREEN Phase)
Verifier correctness is critical - must catch all issues:

```rust
#[test]
fn test_verifier_catches_injection_attempts() {
    // Verifier mutations must not weaken security
}

#[test]
fn test_verifier_all_levels_comprehensive() {
    // Test none, basic, strict, paranoid levels
    // Mutations should not bypass any level
}
```

#### Acceptance Criteria
- [ ] Verifier module ≥90% mutation kill rate
- [ ] No false negatives (all dangerous code still caught)
- [ ] No false positives (safe code still passes)
- [ ] All verification levels tested
- [ ] Security properties maintained

---

### RASH-2505: Mutation Testing Infrastructure & Documentation
**Priority**: Medium
**Status**: TODO
**Duration**: 2 hours

#### Requirements
- [ ] Document mutation testing process
- [ ] Create make target for mutation testing
- [ ] Update ROADMAP.md with Sprint 25 completion
- [ ] Update quality documentation with new metrics
- [ ] Create mutation testing guide for contributors

#### Implementation
```bash
# Add to Makefile
.PHONY: test-mutants
test-mutants:
\tcargo mutants --package bashrs --file src/parser/
\tcargo mutants --package bashrs --file src/ir/
\tcargo mutants --package bashrs --file src/emitter/
\tcargo mutants --package bashrs --file src/verifier/
```

Create docs/mutation-testing.md:
- How mutation testing works
- How to run mutation tests
- How to interpret results
- How to write tests that kill mutants

#### Acceptance Criteria
- [ ] Makefile updated with test-mutants target
- [ ] Documentation complete
- [ ] ROADMAP.md updated
- [ ] Quality metrics updated
- [ ] Sprint 25 marked complete

---

## Quality Metrics Tracking

### Before Sprint
```yaml
metrics:
  test_count: 667
  test_pass_rate: 100%
  property_tests: 52
  coverage_core: 90.53%
  complexity_median: 1.0
  mutation_score: 83%
  quality_score: 98/100
  grade: A+
```

### After Sprint (Target)
```yaml
metrics:
  test_count: 700+  # Add ~30-40 targeted tests
  test_pass_rate: 100%  # Must maintain
  property_tests: 55+  # Add mutation-killing properties
  coverage_core: 90.53%+  # Should maintain or increase
  complexity_median: 1.0  # Should maintain
  mutation_score: 90%+  # PRIMARY GOAL
  quality_score: 100/100  # Perfect A+
  grade: A+
```

### Actual After Sprint
```yaml
metrics:
  test_count: [TBD]
  test_pass_rate: [TBD]
  property_tests: [TBD]
  coverage_core: [TBD]
  complexity_median: [TBD]
  mutation_score: [TBD]
  quality_score: [TBD]
  grade: [TBD]
```

## Performance Benchmarks

### Baseline
```
transpile_simple: 19.1µs ± 0.8µs
# (523x better than 10ms target!)
```

### Target
```
transpile_simple: ≤25µs (allow small regression for added tests)
# Still well under 10ms target
```

### Actual
```
[TBD after sprint]
```

## Toyota Way Application

### 自働化 (Jidoka) - Build Quality In
**Applied**:
- [ ] EXTREME TDD followed (RED-GREEN-REFACTOR)
- [ ] Mutation testing as quality gate
- [ ] Zero defects policy maintained

**Evidence**:
- Mutation tests run FIRST (RED)
- Targeted tests written (GREEN)
- Refactored for clarity (REFACTOR)
- No SATD comments introduced

### 反省 (Hansei) - Reflection
**Applied**:
- [ ] Analyze why mutants survive
- [ ] Root cause analysis for test gaps
- [ ] Document lessons learned

**Issues Found**:
[TBD - document as found]

### 改善 (Kaizen) - Continuous Improvement
**Applied**:
- [ ] 83% → 90%+ mutation score (8.4% improvement)
- [ ] Test suite strengthened significantly
- [ ] Quality infrastructure improved

**Improvements Made**:
[TBD - document as completed]

### 現地現物 (Genchi Genbutsu) - Go and See
**Applied**:
- [ ] Run mutation tests to see actual gaps
- [ ] Test on real shell interpreters
- [ ] Measure actual mutation kill rates

**Observations**:
[TBD - document observations]

## Sprint Retrospective

### What Went Well ✅
[TBD at sprint end]

### What Went Wrong ❌
[TBD at sprint end]

### What We Learned 📚
[TBD at sprint end]

### What We'll Do Differently Next Sprint 🔄
[TBD at sprint end]

## Technical Debt

### Debt Added This Sprint
- **None** (zero tolerance policy) ✅

### Debt Resolved This Sprint
[TBD - document resolved items]

## Documentation Updates

### Files to Update
- [ ] ROADMAP.md - Sprint 25 completion
- [ ] CHANGELOG.md - v1.0.0 release notes (if releasing)
- [ ] docs/mutation-testing.md - New guide (create)
- [ ] Makefile - test-mutants target
- [ ] .quality/sprint25-complete.md - Final metrics

### New Documentation Created
- [ ] docs/mutation-testing.md - Mutation testing guide
- [ ] .quality/sprint25-complete.md - Sprint completion report

## CI/CD Status

### Build Status
- [x] All builds passing
- [x] No errors
- [ ] Clippy warnings (dependency duplicates only, acceptable)

### Test Status
- [x] Unit tests: 667/667 passing
- [x] Property tests: 52 properties
- [x] ShellCheck tests: passing
- [x] Determinism tests: passing
- [ ] Mutation tests: 83% baseline → target 90%+

### Coverage Status
- [x] Total project: 90.53%
- [x] Core modules: ≥85%

### Quality Gates
- [x] Complexity: ≤10 (median 1.0)
- [x] SATD: 0
- [ ] Mutation score: 83% → target 90%+

## Next Steps

### Week 1 (Days 1-7)
**Focus**: Parser + IR modules

**Day 1-2**: RASH-2501 (Parser)
- Install cargo-mutants
- Run parser mutation tests
- Analyze surviving mutants
- Write targeted tests

**Day 3-4**: RASH-2502 (IR)
- Run IR mutation tests
- Close 8 critical gaps from Sprint 24
- Add property tests for IR conversion

**Day 5**: Review & refactor
- Verify all tests passing
- Check complexity maintained
- Update documentation

### Week 2 (Days 8-14)
**Focus**: Emitter + Verifier + Documentation

**Day 8-10**: RASH-2503 (Emitter)
- Run emitter mutation tests
- Safety-critical testing
- ShellCheck integration for mutants

**Day 11-12**: RASH-2504 (Verifier)
- Run verifier mutation tests
- Test all verification levels
- Ensure no security weakening

**Day 13-14**: RASH-2505 (Documentation)
- Create mutation testing guide
- Update Makefile
- Update ROADMAP.md
- Sprint retrospective
- Celebrate 90%+ achievement! 🎉

## Risk Assessment

### High Risk
**Risk**: Mutation testing reveals critical security gaps in emitter
**Impact**: Could require significant emitter refactoring
**Mitigation**:
- Start with emitter analysis early (parallel with parser)
- ShellCheck integration for all mutants
- Security-focused property tests

### Medium Risk
**Risk**: Achieving 90% may require more tests than estimated
**Impact**: Sprint may take longer than 2 weeks
**Mitigation**:
- Time-box each module to planned hours
- Focus on highest-impact mutants first
- Defer nice-to-have tests to Sprint 26 if needed

### Low Risk
**Risk**: Performance regression from added tests
**Impact**: Minimal - tests don't affect runtime performance
**Mitigation**:
- Benchmark after each module
- Verify transpile performance unchanged

---

**Sprint Start**: 2025-10-09
**Sprint End**: 2025-10-23 (estimated)
**Sprint Leader**: Noah Gift
**Status**: Planning → Ready to begin
**Quality Grade Target**: Perfect A+ (100/100)

---

## Sign-off

### Ready to Begin Checklist
- [x] ShellCheck installed
- [x] cargo-llvm-cov installed
- [x] All 667 tests passing
- [x] Version consistency achieved
- [x] Quality baseline documented
- [ ] cargo-mutants installed (Day 1 task)
- [x] Sprint tracking document created
- [x] Team aligned on goals

**Next Action**: Install cargo-mutants and begin RASH-2501 (Parser Mutation Testing)

---

**Status**: 📋 Planning Complete → Ready to Execute
**Confidence**: High (all prerequisites met, clear plan)
**Expected Outcome**: 90%+ mutation kill rate, Perfect A+ quality grade
