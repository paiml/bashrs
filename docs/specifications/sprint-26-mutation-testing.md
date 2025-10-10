# Sprint 26: Mutation Testing Excellence

**Status**: IN PROGRESS
**Created**: 2025-10-10
**Goal**: Achieve ≥90% mutation kill rate across core modules
**Duration**: ~2 weeks
**Priority**: HIGH

---

## Background

From Sprint 24, we established a baseline mutation testing infrastructure with:
- **Baseline kill rate**: ~83% (IR module)
- **8 targeted mutation tests** added to close gaps
- **Infrastructure**: cargo-mutants integrated, working

Sprint 26 builds on this foundation to achieve world-class mutation coverage (≥90%) across ALL core modules.

---

## Sprint Goals

1. **Run full mutation analysis** on all core modules (parser, IR, emitter, verifier)
2. **Identify surviving mutants** and analyze why tests didn't kill them
3. **Add targeted tests** to close gaps (using property tests where applicable)
4. **Achieve ≥90% mutation kill rate** project-wide
5. **Document baseline and improvements** for future reference

---

## Tickets (EXTREME TDD)

### ✅ RASH-2600: Kick off full mutation analysis
**Priority**: CRITICAL
**Status**: ✅ COMPLETE

**Requirements**:
- Run `cargo mutants` on all core modules
- Exclude test files, property tests, generators, bin utilities
- Timeout: 60s per mutant
- Jobs: 4 (parallel execution)

**Execution**:
```bash
# Full mutation run (all core modules)
cargo mutants \
  --exclude 'src/bash_parser/tests.rs' \
  --exclude 'src/bash_parser/property_tests.rs' \
  --exclude 'src/bash_parser/generators.rs' \
  --exclude 'src/bash_transpiler/tests.rs' \
  --exclude 'src/bin/*' \
  --exclude 'tests/*' \
  --timeout 60 \
  --jobs 4 \
  2>&1 | tee /tmp/mutants-full.log
```

**Results**:
- ✅ **1909 mutants** found
- ✅ Baseline build in progress (as of 2025-10-10)
- Output logged to: `/tmp/mutants-full.log`

---

### 🔲 RASH-2601: Parser mutation analysis
**Priority**: CRITICAL
**Status**: PENDING (waiting for RASH-2600 baseline)

**Requirements**:
- Analyze parser mutation results
- Identify surviving mutants in `src/bash_parser/`
- Categorize by mutant type (arithmetic, boundary, logic, etc.)
- Document gap analysis

**Tests** (to be written after analysis):
- `test_parser_mutation_baseline_documented`
- `test_all_parser_mutants_cataloged`
- `test_kill_rate_measured_accurately`

**Acceptance**:
- [ ] Baseline kill rate documented
- [ ] All surviving mutants analyzed
- [ ] Gap analysis complete (why mutants survived)

---

### 🔲 RASH-2602: Close parser mutation gaps
**Priority**: CRITICAL
**Status**: PENDING (depends on RASH-2601)

**Requirements**:
- Add unit tests for each surviving mutant
- Use property tests for patterns (e.g., boundary conditions)
- Target ≥90% kill rate for parser module
- RED-GREEN-REFACTOR cycle for each test

**Tests**:
- `test_parser_mutation_kill_rate_improved`
- `proptest_parser_edge_cases_covered`
- `test_no_trivial_mutants_survive`

**Acceptance**:
- [ ] ≥90% mutation kill rate on parser
- [ ] All critical mutants killed
- [ ] Property tests added for recurring patterns

---

### 🔲 RASH-2603: IR module mutation coverage
**Priority**: HIGH
**Status**: PENDING

**Requirements**:
- Current baseline: ~83% (from Sprint 24)
- Close gaps to reach ≥90%
- Focus on IR conversion logic

**Tests**:
- `test_ir_mutation_coverage_complete`
- `test_ir_conversion_mutants_killed`
- `proptest_ir_transformations_verified`

**Acceptance**:
- [ ] ≥90% mutation kill rate on IR
- [ ] All IR conversions tested
- [ ] Documentation updated

---

### 🔲 RASH-2604: Emitter module mutation coverage
**Priority**: HIGH
**Status**: PENDING

**Requirements**:
- Run cargo-mutants on `src/emitter/`
- Shell code generation must be verified
- Target ≥90% kill rate
- Verify POSIX compliance is maintained

**Tests**:
- `test_emitter_mutation_coverage`
- `test_shell_output_mutations_detected`
- `proptest_emitter_determinism`

**Acceptance**:
- [ ] ≥90% mutation kill rate on emitter
- [ ] All shell emission tested
- [ ] Output validation complete (ShellCheck passes)

---

### 🔲 RASH-2605: Verifier module mutation coverage
**Priority**: MEDIUM
**Status**: PENDING

**Requirements**:
- Run cargo-mutants on `src/verifier/`
- Verification logic must be sound
- Target ≥90% kill rate
- Test both false positives and false negatives

**Tests**:
- `test_verifier_mutation_complete`
- `test_verification_rules_tested`
- `proptest_verifier_correctness`

**Acceptance**:
- [ ] ≥90% mutation kill rate on verifier
- [ ] All verification paths tested
- [ ] False positive/negative tests added

---

## Current Progress (as of 2025-10-10)

### Mutation Testing Baseline

**Command running**:
```bash
cargo mutants --exclude 'src/bash_parser/tests.rs' \
  --exclude 'src/bash_parser/property_tests.rs' \
  --exclude 'src/bash_parser/generators.rs' \
  --exclude 'src/bash_transpiler/tests.rs' \
  --exclude 'src/bin/*' \
  --exclude 'tests/*' \
  --timeout 60 --jobs 4
```

**Results** (in progress):
- **Mutants found**: 1909
- **Baseline build**: ✅ Running
- **Expected completion**: ~30-60 minutes (depending on test suite speed)

### Test Coverage (v1.0.0)
- **Total tests**: 756/756 passing (100%)
- **Property tests**: 52 properties (~26,000+ cases)
- **Coverage**: 85.36% (core modules)
- **Complexity**: Median 1.0, Top 15 (excellent)

---

## Quality Metrics

| Module | Current Kill Rate | Target | Status |
|--------|------------------|--------|--------|
| **Parser** | TBD | ≥90% | 🔄 Analysis pending |
| **IR** | ~83% (Sprint 24) | ≥90% | 🟡 Needs improvement |
| **Emitter** | TBD | ≥90% | 🔄 Analysis pending |
| **Verifier** | TBD | ≥90% | 🔄 Analysis pending |
| **Overall** | TBD | ≥90% | 🎯 Sprint goal |

---

## Mutation Testing Strategy

### Phase 1: Analysis (Week 1, Days 1-3)
1. ✅ Run full mutation suite (RASH-2600)
2. 🔲 Analyze results by module
3. 🔲 Categorize surviving mutants
4. 🔲 Identify patterns (boundary conditions, error paths, edge cases)

### Phase 2: Gap Closure (Week 1, Days 4-7)
1. 🔲 Add unit tests for trivial mutants
2. 🔲 Add property tests for patterns
3. 🔲 Verify kill rate improves
4. 🔲 Rerun mutation suite

### Phase 3: Verification (Week 2, Days 1-3)
1. 🔲 Achieve ≥90% kill rate
2. 🔲 Document remaining survivors (if any)
3. 🔲 Update quality gates
4. 🔲 Create Sprint 26 completion report

### Phase 4: Integration (Week 2, Days 4-5)
1. 🔲 Update CI/CD with mutation testing
2. 🔲 Add mutation score to quality dashboard
3. 🔲 Document process for future sprints
4. 🔲 Release v1.0.1 with improved test coverage

---

## Expected Outcomes

### Immediate (End of Sprint 26)
- ✅ **≥90% mutation kill rate** across all core modules
- ✅ **Comprehensive gap analysis** documented
- ✅ **Targeted tests added** for all surviving mutants
- ✅ **Quality dashboard updated** with mutation score

### Long-term (Future Sprints)
- Mutation testing in CI/CD pipeline
- Mutation score maintained at ≥90%
- Test suite confidence significantly improved
- Zero-defects policy reinforced

---

## Tools and Commands

### Run mutation testing (full suite)
```bash
cargo mutants \
  --exclude 'src/bash_parser/tests.rs' \
  --exclude 'src/bash_parser/property_tests.rs' \
  --exclude 'src/bash_parser/generators.rs' \
  --exclude 'src/bash_transpiler/tests.rs' \
  --exclude 'src/bin/*' \
  --exclude 'tests/*' \
  --timeout 60 \
  --jobs 4
```

### Run mutation testing (specific module)
```bash
# Parser module
cargo mutants --in src/bash_parser --timeout 60 --jobs 2

# IR module
cargo mutants --in src/ir --timeout 60 --jobs 2

# Emitter module
cargo mutants --in src/emitter --timeout 60 --jobs 2

# Verifier module
cargo mutants --in src/verifier --timeout 60 --jobs 2
```

### Analyze results
```bash
# View summary
cargo mutants --list

# View detailed output
cargo mutants --list-files

# Check kill rate
grep "caught\|missed\|timeout" mutants.out | sort | uniq -c
```

---

## References

- **Sprint 24 Report**: `.quality/sprint24-complete.md` (83% IR baseline)
- **Mutation Testing Tool**: https://github.com/sourcefrog/cargo-mutants
- **EXTREME TDD Principles**: RED-GREEN-REFACTOR with zero tolerance
- **Quality Gates**: `roadmap.yaml` (≥90% mutation score)

---

## Success Criteria

- [ ] ≥90% mutation kill rate achieved project-wide
- [ ] All core modules (parser, IR, emitter, verifier) at ≥90%
- [ ] Gap analysis documented
- [ ] Targeted tests added for all surviving mutants
- [ ] Quality dashboard updated
- [ ] Sprint 26 completion report written
- [ ] v1.0.1 released with improved mutation coverage

---

**Status**: 🔄 IN PROGRESS (Baseline analysis running)
**Last Updated**: 2025-10-10
**Next Steps**: Await baseline completion, begin gap analysis

🤖 Generated with [Claude Code](https://claude.com/claude-code)
