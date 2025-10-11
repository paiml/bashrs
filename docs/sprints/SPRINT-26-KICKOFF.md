# Sprint 26: Mutation Testing Excellence - Kickoff

**Status**: READY TO START
**Created**: 2025-10-11
**Goal**: Achieve ≥90% mutation kill rate across all core modules
**Duration**: 5-7 working days (~35-50 hours)
**Priority**: HIGH
**Prerequisites**: v1.2.1 released and stable ✅

---

## Executive Summary

Sprint 26 is a comprehensive mutation testing effort to validate the quality of our 808-test suite and achieve world-class mutation coverage (≥90% kill rate) across all core modules.

### Why Now?

**Perfect Timing**:
- ✅ v1.2.1 just released (100% auto-fix success rate)
- ✅ 808 tests passing (100% pass rate)
- ✅ 88.5% code coverage
- ✅ Stable codebase (3 consecutive successful releases: v1.1.0, v1.2.0, v1.2.1)
- ✅ Sprint 2-3 momentum complete (good stopping point for quality audit)

**Strategic Value**:
- Validates test suite effectiveness (do our 808 tests actually catch bugs?)
- Identifies gaps in test coverage (where are we vulnerable?)
- Builds confidence before adding new features (macro support, parallel execution)
- Establishes quality baseline for future sprints

---

## Current State Analysis

### Test Suite Metrics (v1.2.1)
```
Total Tests:        808/808 passing (100%)
Auto-Fix Tests:     8/8 passing (100%)
Linter Tests:       48/48 passing (100%)
Property Tests:     52 properties (~26,000+ cases)
Code Coverage:      88.5% lines, 85.6% regions, 90.4% functions
Complexity:         Median 1.0, Top 15 (excellent)
Performance:        19.1µs transpile, <2ms lint
```

### Mutation Testing Baseline
```
Mutants Identified: 2323 (up from 1909 estimate)
Mutants Tested:     3 (linter only, from v1.2.0/v1.2.1)
Kill Rate:          Unknown (full baseline not yet run)
Target:             ≥90% kill rate project-wide
```

**Why More Mutants?**
- v1.1.0: Added native linter (+48 tests, ~500 LOC)
- v1.2.0: Added auto-fix (+5 tests, ~200 LOC)
- v1.2.1: Added conflict resolution (+3 tests, ~100 LOC)
- Total: 2323 mutants across all modules

---

## Sprint 26 Execution Plan

### Phase 1: Baseline Analysis (Day 1-2, ~10 hours)

**Day 1: Run Mutation Analysis**
```bash
# Full mutation baseline (3-5 hours runtime)
cargo mutants \
  --exclude 'src/bash_parser/tests.rs' \
  --exclude 'src/bash_parser/property_tests.rs' \
  --exclude 'src/bash_parser/generators.rs' \
  --exclude 'src/bash_transpiler/tests.rs' \
  --exclude 'src/bin/*' \
  --exclude 'tests/*' \
  --timeout 60 \
  --jobs 4 \
  --output-dir mutants-sprint26 \
  2>&1 | tee mutants-sprint26.log
```

**Day 2: Analyze Results**
- Parse mutants-sprint26/outcomes.json
- Categorize survivors by module:
  - Parser (`src/bash_parser/`)
  - IR (`src/ir/`)
  - Emitter (`src/emitter/`)
  - Verifier (`src/verifier/`)
  - Linter (`src/linter/`)
- Categorize by mutation type:
  - Arithmetic operators (+/- to */÷)
  - Comparison operators (>/< to ==/!=)
  - Boolean logic (&&/|| to !/⊥)
  - Boundary conditions (+1/-1 mutations)
  - Return value mutations (Default::default())

**Deliverable**: `docs/sprints/sprint-26-baseline-analysis.md`

---

### Phase 2: Gap Identification (Day 2-3, ~8 hours)

**Tasks**:
1. For each surviving mutant:
   - Identify the function/line
   - Understand why tests didn't catch it
   - Categorize: missing test / weak assertion / untested branch
2. Group survivors by pattern:
   - "Arithmetic boundary" → property test candidate
   - "Error path" → unit test candidate
   - "Edge case" → integration test candidate
3. Prioritize by impact:
   - Critical: Parser, IR conversion, Emitter (core transpiler logic)
   - High: Verifier, Linter (quality and safety)
   - Medium: Utilities, helpers

**Deliverable**: `docs/sprints/sprint-26-gap-analysis.md`

---

### Phase 3: Test Writing (Day 3-5, ~20-30 hours)

**Strategy**: EXTREME TDD with RED-GREEN-REFACTOR

**Module Priority Order**:
1. **Parser** (Target: ≥90% kill rate)
   - Most critical (entry point, syntax tree)
   - Add unit tests for arithmetic mutations
   - Add property tests for boundary conditions
   - Add integration tests for complex syntax

2. **IR** (Target: ≥90% kill rate, currently ~83%)
   - Core transpiler logic
   - Add tests for IR conversion edge cases
   - Add property tests for IR transformations
   - Focus on existing 17% gap from Sprint 24

3. **Emitter** (Target: ≥90% kill rate)
   - Shell code generation
   - Add tests for shell syntax variations
   - Add property tests for determinism
   - Verify ShellCheck compliance

4. **Verifier** (Target: ≥90% kill rate)
   - Safety validation
   - Add tests for false positive/negative cases
   - Add property tests for validation rules

5. **Linter** (Target: ≥95% kill rate)
   - Newest module (v1.1.0-v1.2.1)
   - Should have high kill rate already (recent TDD work)
   - Close any remaining gaps

**Test Template** (for each survivor):
```rust
#[test]
fn test_kill_mutant_<module>_<function>_<line>() {
    // Arrange: Set up scenario that exercises the mutated line

    // Act: Execute the code

    // Assert: Verify behavior that would change with mutation
    // (This assertion MUST fail if the mutant survives)
}
```

**Deliverable**: 50-100 new targeted tests (estimate)

---

### Phase 4: Verification (Day 6, ~6 hours)

**Tasks**:
1. Re-run full mutation suite
2. Verify ≥90% kill rate achieved
3. Document remaining survivors (if any)
4. Analyze cost/benefit of killing final survivors

**Acceptance Criteria**:
- [ ] Overall kill rate ≥90%
- [ ] Parser kill rate ≥90%
- [ ] IR kill rate ≥90%
- [ ] Emitter kill rate ≥90%
- [ ] Verifier kill rate ≥90%
- [ ] Linter kill rate ≥95% (stretch goal)

**Deliverable**: `docs/sprints/sprint-26-results.md`

---

### Phase 5: Documentation & Release (Day 7, ~4 hours)

**Tasks**:
1. Update ROADMAP.md with Sprint 26 completion
2. Update CHANGELOG.md for v1.2.2 (if releasing)
3. Create Sprint 26 completion report
4. Update quality dashboard
5. Document mutation testing process for future sprints

**Release Decision**:
- **Option A**: Release v1.2.2 (quality improvement release)
  - Bump version to 1.2.2
  - CHANGELOG: "50-100 targeted tests added, ≥90% mutation coverage achieved"
  - Publish to crates.io
- **Option B**: Don't release (internal quality improvement only)
  - Keep as v1.2.1
  - Update internal quality metrics
  - Save release for next feature (v1.3.0 or v1.4.0)

**Deliverable**: Sprint 26 completion report + optional v1.2.2 release

---

## Tools and Scripts

### Helper Scripts (to be created)

**`scripts/mutants-analyze.sh`**:
```bash
#!/bin/bash
# Analyze mutation results and generate report

MUTANTS_DIR="mutants-sprint26"

echo "=== Mutation Testing Results ==="
echo ""
echo "Total Mutants: $(jq '.[] | length' $MUTANTS_DIR/outcomes.json)"
echo "Caught: $(cat $MUTANTS_DIR/caught.txt | wc -l)"
echo "Missed: $(cat $MUTANTS_DIR/missed.txt | wc -l)"
echo "Timeout: $(cat $MUTANTS_DIR/timeout.txt | wc -l)"
echo "Unviable: $(cat $MUTANTS_DIR/unviable.txt | wc -l)"
echo ""
echo "Kill Rate: $(calculate_kill_rate)"
```

**`scripts/mutants-by-module.sh`**:
```bash
#!/bin/bash
# Group survivors by module

for module in bash_parser ir emitter verifier linter; do
    echo "=== $module ==="
    grep "src/$module/" mutants-sprint26/missed.txt | wc -l
done
```

---

## Risk Mitigation

### Risk 1: Time Overrun
**Mitigation**:
- Prioritize critical modules first (parser, IR, emitter)
- Set time boxes for each phase
- Accept <90% if time-boxed approach limits progress

### Risk 2: Trivial Mutants
**Mitigation**:
- Identify "equivalent mutants" (semantically identical code)
- Document as "acceptable survivors" if cost > benefit
- Focus on meaningful mutations

### Risk 3: Test Suite Slowdown
**Mitigation**:
- Monitor test execution time
- Optimize slow tests if needed
- Consider test parallelization if suite exceeds 60s

---

## Success Metrics

### Primary Metrics
- [ ] ≥90% mutation kill rate achieved
- [ ] All core modules at ≥90%
- [ ] Gap analysis documented
- [ ] 50-100 targeted tests added

### Secondary Metrics
- [ ] Test suite still passes in <60s
- [ ] Code coverage maintained (≥88.5%)
- [ ] No regressions introduced
- [ ] Mutation testing process documented

---

## References

- **Sprint 24 Report**: `.quality/sprint24-complete.md` (83% IR baseline)
- **Sprint 26 Specification**: `docs/specifications/sprint-26-mutation-testing.md`
- **Mutation Testing Tool**: https://github.com/sourcefrog/cargo-mutants
- **EXTREME TDD Principles**: RED-GREEN-REFACTOR with zero tolerance

---

## Kickoff Checklist

Before starting Sprint 26:

- [x] v1.2.1 released and stable
- [x] 808 tests passing (100%)
- [x] cargo-mutants installed (v25.3.1)
- [x] 2323 mutants identified
- [x] Sprint 26 kickoff document created
- [ ] Dedicated time block scheduled (5-7 days)
- [ ] Mutation analysis scripts created
- [ ] Team notified (if applicable)
- [ ] Ready to commit to quality work

---

**Status**: 🟢 READY TO START (all prerequisites met)
**Next Action**: Schedule dedicated 5-7 day time block
**Expected Completion**: 7 working days after kickoff

🤖 Generated with [Claude Code](https://claude.com/claude-code)
