# bashrs Strategic Options - November 2, 2025

**Context**: v6.27.1 released, Issue #5 complete, 6021 tests passing
**Decision Point**: Choose next strategic direction

---

## Option 1: Complete Shell-Specific Rule Filtering (HIGH IMPACT)

### Overview

**Objective**: Implement shell-specific rule filtering to fully leverage shell type detection infrastructure built in v6.27.0-6.27.1.

**Status**: Foundation complete (50%), filtering not yet implemented

### What This Means

Currently: Shell type is detected but all rules run universally
Proposed: Rules filtered based on detected shell type

**Example Impact**:
```rust
// v6.27.1 (current)
lint_shell_with_path(zsh_file) → runs all 357 rules

// v6.28.0 (proposed)
lint_shell_with_path(zsh_file) → runs zsh-appropriate rules only
                               → skips bash-only rules
                               → adds zsh-specific rules
```

### Implementation Plan (EXTREME TDD)

#### Phase 1: Rule Classification (Sprint 1 - 2 days)
- **Task**: Classify all 357 rules by shell compatibility
- **Output**: `ShellCompatibility` enum per rule
  - `Universal` - applies to all shells
  - `BashOnly` - bash-specific
  - `ZshOnly` - zsh-specific
  - `ShOnly` - POSIX sh only
- **Tests**: 357 classification tests
- **Deliverable**: `docs/RULE-SHELL-COMPATIBILITY.md`

#### Phase 2: Filtering Engine (Sprint 2 - 3 days)
- **Task**: Implement rule filtering in linter
- **API**: `filter_rules_for_shell(shell_type, all_rules) -> Vec<Rule>`
- **Tests**: 20 filtering tests (RED → GREEN → REFACTOR)
- **Property tests**: 10 new tests ensuring filtering correctness
- **Deliverable**: `rash/src/linter/rule_filter.rs`

#### Phase 3: Zsh-Specific Rules (Sprint 3 - 5 days)
- **Task**: Add 20 zsh-specific linter rules
- **Examples**:
  - `ZSH001`: Detect zsh array syntax issues
  - `ZSH002`: Flag bash-isms in zsh files
  - `ZSH003`: Zsh glob qualifier validation
  - `ZSH004`: Parameter expansion flag validation
  - `ZSH005-020`: Additional zsh idioms
- **Tests**: 200 tests (10 per rule)
- **Deliverable**: `rash/src/linter/rules/zsh_*.rs`

#### Phase 4: Integration & Validation (Sprint 4 - 2 days)
- **Task**: End-to-end validation
- **Tests**: 50 integration tests (real .zshrc files)
- **Benchmarking**: Performance impact of filtering
- **Documentation**: Update book with zsh rules
- **Deliverable**: v6.28.0 release

### Benefits ✅

1. **User Experience**: Fewer false positives, more relevant errors
2. **Performance**: Skip irrelevant rules (faster linting)
3. **Accuracy**: Shell-specific idiom detection
4. **Adoption**: Full zsh support attracts 70%+ of developers
5. **Market Position**: Only linter with shell-specific filtering

### Risks ⚠️

- **Complexity**: Rule classification requires domain expertise
- **Maintenance**: Rules must stay synchronized with shells
- **Testing**: 357 + 20 + filtering = significant test burden

### Effort Estimate

- **Duration**: 12 days (4 sprints)
- **Tests**: 537 new tests (357 classification + 20 zsh × 10 + integration)
- **Lines of Code**: ~3000 lines
- **Release**: v6.28.0 (minor version)

### Success Criteria

- [ ] All 357 rules classified by shell compatibility
- [ ] 20 new zsh-specific rules implemented
- [ ] Filtering reduces irrelevant errors by ≥80%
- [ ] Performance impact <5% (benchmarked)
- [ ] Zero regressions (6021+ tests still passing)
- [ ] Property tests ensure filtering correctness
- [ ] Book documentation complete

### Why Choose This?

**Rationale**: Completes the vision started in v6.27.0. Shell type detection is currently "detection only" - this makes it actionable.

**Impact**: High user value (fewer false positives) + market differentiation (unique feature)

**Alignment**: Builds on today's work (Issue #5), momentum is strong

---

## Option 2: Bash Manual Validation (SYSTEMATIC COMPLETION)

### Overview

**Objective**: Complete systematic validation of GNU Bash Manual transformations (currently 20% complete, 24/122 tasks).

**Status**: BASH-INGESTION-ROADMAP.yaml exists, 87/90 tasks completed

### What This Means

Systematically validate bashrs handles every construct in the GNU Bash Manual.

**Current**: 97% coverage (87/90 tasks), 3 blocked by parser feature
**Proposed**: 100% coverage, all edge cases documented

### Implementation Plan (EXTREME TDD + REPL Verification)

#### Workflow (from BASH-INGESTION-ROADMAP.yaml)
1. **RED**: Write failing test for bash construct
2. **GREEN**: Implement transformation
3. **REFACTOR**: Clean code (<10 complexity)
4. **REPL VERIFICATION**: Test interactively
5. **PROPERTY**: Add property tests
6. **MUTATION**: Verify ≥90% kill rate
7. **PMAT**: Quality analysis
8. **DOCUMENT**: Update roadmap

#### Remaining Tasks (3 blocked)

All active tasks complete! Only 3 blocked by P0-POSITIONAL-PARAMETERS parser feature:
- Positional parameters ($1, $2, $3, etc.)
- Shift command
- Set -- reassignment

**These require parser enhancement** - not simple transformations

#### New Direction: Edge Cases & Property Tests

Since active tasks are complete, focus on:

1. **Property tests for existing transforms** (87 tasks × 5 tests = 435 new property tests)
2. **Edge case documentation** (mutation testing reveals gaps)
3. **Performance optimization** (profile hot paths)
4. **REPL ergonomics** (improve interactive experience)

### Benefits ✅

1. **Completeness**: 100% Bash Manual coverage
2. **Confidence**: Every edge case tested
3. **Quality**: 435 new property tests
4. **Documentation**: Comprehensive transform catalog
5. **Benchmark**: Reference implementation for bash transformations

### Risks ⚠️

- **Diminishing Returns**: 97% → 100% may not justify effort
- **Parser Blocker**: 3 tasks require significant parser work
- **Low User Impact**: Most users don't hit edge cases

### Effort Estimate

- **Duration**: 15-20 days
- **Tests**: 435 new property tests + edge cases
- **Lines of Code**: ~1500 lines (mostly tests)
- **Release**: v6.29.0 (minor version)

### Success Criteria

- [ ] All 87 active tasks have property tests
- [ ] All edge cases documented
- [ ] Mutation testing ≥90% kill rate across transforms
- [ ] Performance profiling complete
- [ ] REPL ergonomics improved
- [ ] 100% Bash Manual coverage (if parser unblocked)

### Why Choose This?

**Rationale**: Achieve completeness, create definitive reference implementation

**Impact**: Medium user value (edge cases rare) + high technical reputation

**Alignment**: Systematic approach fits EXTREME TDD culture

---

## Option 3: Mutation Testing Excellence (QUALITY ASSURANCE)

### Overview

**Objective**: Achieve ≥90% mutation kill rate across entire codebase (currently running, ~2323 mutants project-wide).

**Status**: Mutation tests running in background, infrastructure ready

### What This Means

Use mutation testing to find weak tests and strengthen test suite quality.

**Current**: Unit tests exist, quality unknown
**Proposed**: Every test proven effective by mutation testing

### Implementation Plan (Sprint 26 - Deferred from ROADMAP)

#### Phase 1: Baseline Assessment (Week 1)
- **Task**: Run full mutation suite, measure baseline
- **Tool**: `cargo mutants --all`
- **Output**: Mutation score per module
- **Deliverable**: `docs/MUTATION-BASELINE-2025-11-02.md`

#### Phase 2: Critical Path First (Week 2-3)
- **Focus**: shell_type.rs, linter core, cli/bench.rs
- **Target**: ≥95% kill rate on new code
- **Method**: Add tests for surviving mutants
- **Tests**: ~500 new tests estimated

#### Phase 3: Coverage Expansion (Week 4-5)
- **Focus**: Parser, transformer, emitter modules
- **Target**: ≥90% kill rate overall
- **Method**: Property tests for complex logic
- **Tests**: ~800 new tests estimated

#### Phase 4: Maintenance & CI (Week 6)
- **Task**: Integrate mutation testing into CI
- **Tool**: `cargo mutants` in pre-release gate
- **Monitor**: Track mutation score over time
- **Deliverable**: Automated mutation testing workflow

### Benefits ✅

1. **Test Quality**: Prove every test is effective
2. **Bug Prevention**: Find weak tests before bugs do
3. **Confidence**: Mathematical proof of test coverage
4. **CI/CD**: Prevent test quality regression
5. **Industry Leading**: Few projects achieve ≥90%

### Risks ⚠️

- **Time**: Mutation testing is compute-intensive
- **Effort**: ~1300 new tests estimated
- **Diminishing Returns**: 85% → 90% costs more than 0% → 85%

### Effort Estimate

- **Duration**: 6 weeks (6 sprints)
- **Tests**: ~1300 new tests
- **Lines of Code**: ~4000 lines (tests)
- **Release**: v6.30.0 (minor version)

### Success Criteria

- [ ] Baseline mutation score documented
- [ ] Critical path (shell_type, linter) ≥95% kill rate
- [ ] Overall codebase ≥90% kill rate
- [ ] CI/CD integration complete
- [ ] Mutation score tracked over time
- [ ] Documentation updated

### Why Choose This?

**Rationale**: Achieve industry-leading test quality, prevent future regressions

**Impact**: Low immediate user value + high long-term quality assurance

**Alignment**: Doubles down on EXTREME TDD, proves methodology

---

## Decision Matrix

| Criterion | Option 1: Shell Filtering | Option 2: Bash Manual | Option 3: Mutation Testing |
|-----------|---------------------------|----------------------|---------------------------|
| **User Impact** | ⭐⭐⭐⭐⭐ (High) | ⭐⭐⭐ (Medium) | ⭐⭐ (Low) |
| **Market Differentiation** | ⭐⭐⭐⭐⭐ (Unique) | ⭐⭐⭐ (Good) | ⭐⭐ (Internal) |
| **Technical Complexity** | ⭐⭐⭐⭐ (High) | ⭐⭐⭐ (Medium) | ⭐⭐⭐⭐⭐ (Very High) |
| **Time to Value** | 12 days | 20 days | 42 days |
| **Builds on Today** | ✅ Yes (Issue #5) | 🟡 Parallel | 🟡 Parallel |
| **Risk** | Low | Medium (parser blocker) | Low |
| **Tests Added** | 537 | 435 | 1300 |
| **User-Facing Features** | ✅ Yes | 🟡 Indirect | ❌ No |

---

## Recommendation

### 🏆 Option 1: Complete Shell-Specific Rule Filtering

**Rationale**:
1. ✅ **Highest user impact** - Directly improves linting experience for 70%+ users
2. ✅ **Unique market position** - No other linter has shell-specific filtering
3. ✅ **Builds on momentum** - Completes Issue #5 work from today
4. ✅ **Clear deliverable** - v6.28.0 with 20 zsh-specific rules
5. ✅ **Reasonable effort** - 12 days vs 42 days for mutation testing

**Why Not Others**:
- **Option 2**: Bash Manual is 97% complete, diminishing returns
- **Option 3**: Mutation testing is valuable but internal-facing, low user impact

### Implementation Roadmap (v6.28.0)

**Week 1 (Sprint 1)**: Rule classification (357 rules)
**Week 2 (Sprint 2)**: Filtering engine + tests
**Week 3 (Sprint 3)**: Zsh-specific rules (20 rules, 200 tests)
**Week 4 (Sprint 4)**: Integration, validation, release

**Deliverable**: v6.28.0 - "Complete Zsh Support"

---

## Alternative Sequencing

If maximum value is desired:

1. **v6.28.0** (Option 1): Shell-specific filtering - 12 days
2. **v6.29.0** (Option 3): Mutation testing excellence - 42 days
3. **v6.30.0** (Option 2): Bash Manual completion - 20 days

**Total**: 74 days to complete all three

---

## Metrics for Success

Regardless of option chosen, measure:

- **Test count**: Target 7000+ tests (from 6021)
- **Property tests**: Target 700+ (from 648)
- **User adoption**: Track crates.io downloads
- **GitHub stars**: Measure community interest
- **Issue velocity**: Keep issues ≤24hr response
- **Release cadence**: Maintain quality, not speed

---

## Conclusion

bashrs is at a strategic inflection point with three viable paths:

1. **User Value** (Option 1) - Recommended
2. **Completeness** (Option 2) - Second priority
3. **Quality Assurance** (Option 3) - Long-term investment

**Next Step**: Decide on option, create Sprint 1 tickets, begin implementation Monday.

---

**Prepared**: 2025-11-02
**For**: bashrs v6.27.1 strategic planning
**Status**: Ready for decision
