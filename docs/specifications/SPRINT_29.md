# Sprint 29: Mutation Testing - Full Coverage

**Sprint:** 29
**Focus:** Achieve ≥90% mutation kill rate project-wide
**Priority:** P2_MEDIUM
**Duration:** 4-6 hours
**Philosophy:** 自働化 (Jidoka) - Build quality in through comprehensive mutation testing

---

## Overview

Expand mutation testing from the IR module (96.6% kill rate) to cover the entire codebase, achieving ≥90% mutation kill rate across all critical modules: parser, emitter, and AST.

### Current Baseline

**IR Module (Sprint 26):**
- Kill rate: 96.6% (28/29 mutants)
- Status: ✅ Exceeds ≥90% target

**is_string_value Function (Sprint 26.1):**
- Kill rate: 100% (3/3 mutants)
- Status: ✅ Perfect score

**Other Modules:**
- Parser: Not yet tested
- Emitter: Not yet tested
- AST: Not yet tested
- Stdlib: Not yet tested
- Converter: Not yet tested

---

## Goals

### Primary Goal
Achieve ≥90% mutation kill rate on critical modules:
1. **Parser module** (`rash/src/parser/`)
2. **Emitter module** (`rash/src/emitter/`)
3. **AST module** (`rash/src/ast/`)

### Secondary Goals
- Document mutation testing methodology
- Establish baseline for remaining modules
- Create targeted tests for surviving mutants
- Maintain 100% test pass rate

### Success Criteria
- [ ] Parser module ≥90% kill rate
- [ ] Emitter module ≥90% kill rate
- [ ] AST module ≥90% kill rate
- [ ] All 857+ tests passing (100%)
- [ ] Zero regressions introduced
- [ ] Completed in 4-6 hours
- [ ] Toyota Way principles applied

---

## Scope

### In Scope
1. **Mutation Testing Execution**
   - Run `cargo mutants` on parser module
   - Run `cargo mutants` on emitter module
   - Run `cargo mutants` on AST module
   - Collect kill rate baselines

2. **Targeted Test Development**
   - Analyze surviving mutants
   - Apply Five Whys analysis
   - Write tests that kill specific mutants
   - Verify mutation kill improvement

3. **Documentation**
   - Record baseline kill rates
   - Document surviving mutants
   - Explain root causes
   - Track improvement metrics

### Out of Scope
- Bash parser module (property test heavy, separate analysis needed)
- Stdlib module (covered by emitter runtime tests)
- Utility modules (low risk, defer to later sprint)
- Refactoring unrelated to mutation testing

---

## Technical Design

### Phase 1: Baseline Establishment (1-1.5 hours)

**Parser Module Baseline:**
```bash
cargo mutants --no-shuffle --timeout 120 --jobs 2 \
  --file 'rash/src/parser/mod.rs' \
  2>&1 | tee /tmp/mutants-parser.log
```

**Emitter Module Baseline:**
```bash
cargo mutants --no-shuffle --timeout 120 --jobs 2 \
  --file 'rash/src/emitter/mod.rs' \
  --file 'rash/src/emitter/posix.rs' \
  2>&1 | tee /tmp/mutants-emitter.log
```

**AST Module Baseline:**
```bash
cargo mutants --no-shuffle --timeout 120 --jobs 2 \
  --file 'rash/src/ast/mod.rs' \
  2>&1 | tee /tmp/mutants-ast.log
```

**Expected Output:**
- Kill rate percentage for each module
- List of surviving mutants with line numbers
- Total mutants tested per module

### Phase 2: Analysis (1-2 hours)

For each surviving mutant:

**Five Whys Analysis:**
1. **Why did the mutant survive?**
   - Existing tests don't check behavior affected by mutation

2. **Why don't tests check this behavior?**
   - Test is too indirect / checks wrong assertion

3. **Why was the test indirect?**
   - Missing understanding of what code does

4. **Why was understanding missing?**
   - Code path not explicitly tested

5. **Root cause?**
   - Missing targeted test for specific behavior

**Categorize Mutants:**
- **Category A:** Missing test coverage (add new test)
- **Category B:** Weak assertions (strengthen existing test)
- **Category C:** Dead code (consider removal or document why)
- **Category D:** Acceptable survival (edge case, document)

### Phase 3: Test Development (2-3 hours)

**Test Strategy:**

1. **Direct Behavioral Tests**
   ```rust
   // BAD: Indirect test (checks overall success)
   #[test]
   fn test_parse_function() {
       let result = parse("fn foo() {}");
       assert!(result.is_ok()); // Doesn't check WHAT was parsed
   }

   // GOOD: Direct test (checks specific behavior)
   #[test]
   fn test_parse_function_name() {
       let result = parse("fn foo() {}").unwrap();
       assert_eq!(result.name, "foo"); // Checks specific field
   }
   ```

2. **Mutation-Killing Tests**
   ```rust
   // Example: Killing a mutant that replaces && with ||
   #[test]
   fn test_requires_both_conditions() {
       // This input makes && and || behave differently
       let result = check_value("123.5"); // parses as f64, not i64

       // With &&: true && false = false (correct)
       // With ||: true || false = true (wrong!)
       assert!(!result.is_string); // Catches the || mutant
   }
   ```

3. **Follow Sprint 26 Patterns**
   - Use inputs that expose logic differences
   - Assert on specific behavior, not general success
   - Test edge cases that reveal mutation

### Phase 4: Verification (0.5 hours)

**Re-run Mutation Testing:**
```bash
# Verify improvements
cargo mutants --no-shuffle --timeout 120 --jobs 2 \
  --file 'rash/src/parser/mod.rs' \
  --file 'rash/src/emitter/mod.rs' \
  --file 'rash/src/ast/mod.rs' \
  2>&1 | tee /tmp/mutants-final.log
```

**Success Check:**
- [ ] Parser ≥90% kill rate
- [ ] Emitter ≥90% kill rate
- [ ] AST ≥90% kill rate
- [ ] All tests passing (100%)

---

## Testing Strategy

### EXTREME TDD Adaptation for Mutation Testing

Unlike typical RED-GREEN-REFACTOR, mutation testing follows:

**BASELINE → ANALYZE → TARGET → VERIFY**

1. **BASELINE Phase**
   - Run mutation testing on target modules
   - Record kill rates and surviving mutants
   - Commit baseline report

2. **ANALYZE Phase**
   - Apply Five Whys to each surviving mutant
   - Categorize mutants (A, B, C, D)
   - Identify root causes

3. **TARGET Phase**
   - Write tests that kill Category A mutants
   - Strengthen tests for Category B mutants
   - Document Category C & D decisions
   - Run tests to verify they pass

4. **VERIFY Phase**
   - Re-run mutation testing
   - Confirm kill rate improvement
   - Commit improvements

### Test Expectations

**Before Sprint 29:**
- Total tests: 857
- IR module: 96.6% kill rate
- is_string_value: 100% kill rate
- Other modules: Unknown

**After Sprint 29:**
- Total tests: 857 + N (new mutation-killing tests)
- Parser module: ≥90% kill rate
- Emitter module: ≥90% kill rate
- AST module: ≥90% kill rate
- All tests: 100% pass rate

**Estimated New Tests:** 10-20 targeted mutation tests

---

## Risk Assessment

### Risk 1: Long Mutation Testing Runtime
**Probability:** HIGH
**Impact:** MEDIUM
**Mitigation:**
- Use `--jobs 2` to parallelize
- Use `--timeout 120` to prevent hangs
- Test modules separately
- Run during breaks

### Risk 2: Low Baseline Kill Rates (<70%)
**Probability:** MEDIUM
**Impact:** HIGH
**Mitigation:**
- Focus on highest-impact mutants first
- Accept 80-85% as interim milestone
- Document reasons for acceptance
- Plan follow-up sprint if needed

### Risk 3: Test Interference
**Probability:** LOW
**Impact:** MEDIUM
**Mitigation:**
- Run full test suite after each test addition
- Watch for unintended side effects
- Use `cargo test --lib` to isolate

### Risk 4: Dead Code Discoveries
**Probability:** MEDIUM
**Impact:** LOW
**Mitigation:**
- Document why code exists (even if not tested)
- Consider removal only if truly unused
- Defer removal to separate cleanup sprint

---

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- **Mutation testing** catches inadequate tests
- **Targeted test development** ensures quality
- **≥90% kill rate** validates test effectiveness
- **Zero defects policy** maintained

### 現地現物 (Genchi Genbutsu) - Direct Observation
- **Analyze actual mutants** that survived
- **Measure real kill rates** with cargo mutants
- **Test real behavior** affected by mutations
- **Verify with re-run** to confirm improvement

### 反省 (Hansei) - Reflection
- **Five Whys analysis** for each surviving mutant
- **Root cause identification** before writing tests
- **Learn from Sprint 26** patterns and techniques
- **Document lessons** for future sprints

### 改善 (Kaizen) - Continuous Improvement
- **Expand from IR** (96.6%) to entire codebase
- **Pursue ≥90%** across all critical modules
- **Improve test quality** not just quantity
- **Establish baseline** for future improvements

---

## Deliverables

### Code Changes
1. **Test Files**
   - `rash/src/parser/tests.rs` - New mutation-killing tests
   - `rash/src/emitter/tests.rs` - New mutation-killing tests
   - `rash/src/ast/tests.rs` - New mutation-killing tests

2. **Documentation**
   - `.quality/sprint29-baseline.md` - Baseline kill rates
   - `.quality/sprint29-analysis.md` - Mutant analysis
   - `.quality/sprint29-complete.md` - Final report

### Metrics
- Baseline kill rates (per module)
- Final kill rates (per module)
- Improvement delta (percentage points)
- Tests added count
- Mutants killed count

### Reports
1. **Baseline Report**
   - Kill rates before improvements
   - Surviving mutants list
   - Categorization breakdown

2. **Analysis Report**
   - Five Whys for top mutants
   - Root cause summary
   - Test strategy decisions

3. **Completion Report**
   - Final kill rates
   - Tests added
   - Lessons learned
   - Next steps

---

## Acceptance Criteria

### Functional Requirements
- [ ] Parser module mutation tested
- [ ] Emitter module mutation tested
- [ ] AST module mutation tested
- [ ] Surviving mutants analyzed
- [ ] Category A mutants have new tests
- [ ] Category B mutants have improved tests
- [ ] Category C & D mutants documented

### Quality Requirements
- [ ] Parser ≥90% kill rate OR documented acceptance
- [ ] Emitter ≥90% kill rate OR documented acceptance
- [ ] AST ≥90% kill rate OR documented acceptance
- [ ] All tests passing (100%)
- [ ] Zero regressions
- [ ] Zero clippy warnings

### Process Requirements
- [ ] BASELINE phase complete
- [ ] ANALYZE phase complete
- [ ] TARGET phase complete
- [ ] VERIFY phase complete
- [ ] Five Whys documented for key mutants
- [ ] Toyota Way principles applied

### Documentation Requirements
- [ ] Baseline report created
- [ ] Analysis report created
- [ ] Completion report created
- [ ] ROADMAP updated
- [ ] Sprint 29 marked complete

---

## Timeline

### Estimated Duration: 4-6 hours

**Phase 1: BASELINE (1-1.5 hours)**
- Run mutation testing on 3 modules
- Collect and analyze output
- Create baseline report

**Phase 2: ANALYZE (1-2 hours)**
- Apply Five Whys to surviving mutants
- Categorize mutants
- Document root causes
- Plan test strategy

**Phase 3: TARGET (2-3 hours)**
- Write new mutation-killing tests
- Strengthen existing tests
- Run tests to verify they pass
- Document Category C & D decisions

**Phase 4: VERIFY (0.5 hours)**
- Re-run mutation testing
- Confirm kill rate improvements
- Create completion report
- Update ROADMAP

---

## Success Metrics

### Primary Metrics
- **Parser Kill Rate:** ≥90% (or documented acceptance ≥80%)
- **Emitter Kill Rate:** ≥90% (or documented acceptance ≥80%)
- **AST Kill Rate:** ≥90% (or documented acceptance ≥80%)
- **Test Pass Rate:** 100%

### Secondary Metrics
- **Tests Added:** 10-20 targeted mutation tests
- **Mutants Killed:** Category A + Category B
- **Documentation:** 3 reports (baseline, analysis, completion)
- **Time:** Completed in ≤6 hours

### Quality Metrics
- **Zero Regressions:** No existing tests break
- **Zero Errors:** All new tests pass first try (or minimal fixes)
- **Zero Warnings:** cargo clippy clean
- **Toyota Way:** All 4 principles applied and documented

---

## Follow-Up Actions

**If ≥90% achieved on all modules:**
- Mark Sprint 29 as COMPLETE
- Consider Sprint 31 (Parser Enhancement) next
- Document mutation testing best practices

**If 80-89% achieved:**
- Mark Sprint 29 as SUBSTANTIAL_COMPLETE
- Document reasons for acceptance
- Plan Sprint 29.1 for remaining gaps

**If <80% achieved:**
- Analyze why kill rate is low
- Consider if module needs refactoring
- Plan extended Sprint 29 or break into sub-sprints

---

## References

- Sprint 26: Mutation Testing Excellence (IR module baseline)
- Sprint 26.1: Perfect Mutation Kill Rate (is_string_value)
- `.quality/sprint26-complete.md` - Techniques and patterns
- `docs/specifications/MUTATION_TESTING.md` - Methodology

---

**Specification Created:** 2025-10-14
**Sprint Status:** READY TO START
**Estimated Start:** Upon approval
**Philosophy:** 自働化 (Jidoka) - Build quality in through mutation testing
**Target:** ≥90% kill rate across parser, emitter, and AST modules
