# Sprint 117: 85% ShellCheck Coverage Milestone

**Status**: 🟢 IN PROGRESS
**Sprint ID**: SPRINT-117
**Goal**: Reach 85% ShellCheck SC2xxx series coverage (255/300 rules)
**Duration**: ~3-4 hours
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR)
**Current Coverage**: 240/300 (80.0%)
**Target Coverage**: 255/300 (85.0%)

## Sprint Overview

Implement the next batch of 15 ShellCheck-equivalent linter rules (SC2251-SC2265) to achieve the 85% coverage milestone. This continues the systematic expansion following Sprints 114-116.

## Current Status

**Before Sprint 117**:
- ✅ 240 active linter rules (80.0% coverage)
- ✅ 3,945 tests passing (100% pass rate)
- ✅ v5.0.0 released successfully
- ✅ Zero regressions maintained
- ✅ All quality gates passing

**After Sprint 117 (Target)**:
- 🎯 255 active linter rules (85.0% coverage)
- 🎯 ~4,095 tests passing (150+ new tests)
- 🎯 Zero regressions maintained
- 🎯 Ready for v5.1.0 release

## Rules to Implement (SC2251-SC2265)

Based on ShellCheck's SC2xxx series progression, targeting the next 15 rules:

### Batch 1: SC2251-SC2255 (5 rules)
**Focus**: Test expressions and bracket safety

1. **SC2251**: TBD - Test expression validation
2. **SC2252**: TBD - Bracket usage patterns
3. **SC2253**: TBD - Glob pattern handling
4. **SC2254**: TBD - Quote safety
5. **SC2255**: TBD - Variable expansion

### Batch 2: SC2256-SC2260 (5 rules)
**Focus**: Command safety and execution

6. **SC2256**: TBD - Command validation
7. **SC2257**: TBD - Execution context
8. **SC2258**: TBD - Process handling
9. **SC2259**: TBD - Redirection safety
10. **SC2260**: TBD - Pipeline patterns

### Batch 3: SC2261-SC2265 (5 rules)
**Focus**: Syntax and portability

11. **SC2261**: TBD - Syntax validation
12. **SC2262**: TBD - Portability issues
13. **SC2263**: TBD - Best practices
14. **SC2264**: TBD - Error handling
15. **SC2265**: TBD - Code quality

**Note**: Specific rule descriptions will be researched and documented during implementation. Rules are implemented based on actual ShellCheck wiki documentation.

## Implementation Plan

### Phase 1: Research & Design (30-45 minutes)
- [ ] Research SC2251-SC2265 from ShellCheck wiki
- [ ] Document each rule with examples
- [ ] Design regex patterns and detection logic
- [ ] Identify potential edge cases
- [ ] Plan test coverage strategy

### Phase 2: Implementation - Batch 1 (60-75 minutes)
**Rules**: SC2251-SC2255

For each rule:
1. **RED Phase** (10 min):
   - Write 10 failing tests
   - Cover: basic case, edge cases, false positives, severity
   - Tests must fail initially

2. **GREEN Phase** (8 min):
   - Implement rule in `rash/src/linter/rules/sc2XXX.rs`
   - Add to `mod.rs` integration
   - All tests must pass

3. **REFACTOR Phase** (2 min):
   - Clean up code
   - Ensure complexity <10
   - Add documentation

### Phase 3: Implementation - Batch 2 (60-75 minutes)
**Rules**: SC2256-SC2260

Same EXTREME TDD process as Batch 1.

### Phase 4: Implementation - Batch 3 (60-75 minutes)
**Rules**: SC2261-SC2265

Same EXTREME TDD process as Batch 1 and 2.

### Phase 5: Integration & Verification (30-45 minutes)
- [ ] Run full test suite: `cargo test --lib`
- [ ] Verify 255 rules registered
- [ ] Check 85% coverage calculation
- [ ] Run clippy: `cargo clippy --all-targets`
- [ ] Verify zero regressions
- [ ] Update rule count in all documentation

### Phase 6: Documentation (20-30 minutes)
- [ ] Update CHANGELOG.md with Sprint 117 achievements
- [ ] Update ROADMAP.yaml with completion status
- [ ] Create SPRINT-117-COMPLETE.md
- [ ] Document any technical challenges
- [ ] Update rule coverage metrics

## Success Criteria

### Must Have
- [x] All 15 rules implemented (SC2251-SC2265)
- [ ] 150+ comprehensive tests (10 per rule minimum)
- [ ] 100% test pass rate (no failures)
- [ ] 85.0% ShellCheck coverage verified
- [ ] Zero regressions (all existing 3,945 tests pass)
- [ ] Complexity <10 on all functions
- [ ] Zero clippy warnings

### Quality Gates
- [ ] Test coverage: 100% on new rules
- [ ] Code complexity: <10 (median 2-5)
- [ ] Performance: <2ms per rule
- [ ] Documentation: Complete for all rules

## Expected Deliverables

### Code Files (15 new)
```
rash/src/linter/rules/sc2251.rs
rash/src/linter/rules/sc2252.rs
rash/src/linter/rules/sc2253.rs
rash/src/linter/rules/sc2254.rs
rash/src/linter/rules/sc2255.rs
rash/src/linter/rules/sc2256.rs
rash/src/linter/rules/sc2257.rs
rash/src/linter/rules/sc2258.rs
rash/src/linter/rules/sc2259.rs
rash/src/linter/rules/sc2260.rs
rash/src/linter/rules/sc2261.rs
rash/src/linter/rules/sc2262.rs
rash/src/linter/rules/sc2263.rs
rash/src/linter/rules/sc2264.rs
rash/src/linter/rules/sc2265.rs
rash/src/linter/rules/mod.rs (updated)
```

### Documentation
```
docs/sprints/SPRINT-117-PLAN.md (this file)
docs/sprints/SPRINT-117-COMPLETE.md
CHANGELOG.md (updated)
ROADMAP.yaml (updated)
```

### Metrics
- **Rules Added**: 15
- **Tests Added**: 150+
- **Total Tests**: ~4,095
- **Coverage**: 85.0% (255/300 rules)
- **Code Added**: ~3,500 lines (rules + tests)
- **Documentation**: ~1,000 lines

## Testing Strategy

### Test Template (10 tests per rule)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 1. Basic detection
    #[test]
    fn test_scXXXX_detects_basic_case() { }

    // 2. Multiple occurrences
    #[test]
    fn test_scXXXX_detects_multiple() { }

    // 3. No false positives
    #[test]
    fn test_scXXXX_ignores_valid_code() { }

    // 4. Edge case 1
    #[test]
    fn test_scXXXX_edge_case_1() { }

    // 5. Edge case 2
    #[test]
    fn test_scXXXX_edge_case_2() { }

    // 6. Context sensitivity
    #[test]
    fn test_scXXXX_context_aware() { }

    // 7. Quote handling
    #[test]
    fn test_scXXXX_quote_handling() { }

    // 8. Severity validation
    #[test]
    fn test_scXXXX_severity() { }

    // 9. Message validation
    #[test]
    fn test_scXXXX_message_text() { }

    // 10. Integration
    #[test]
    fn test_scXXXX_integration() { }
}
```

## Risk Management

### Known Risks
1. **ShellCheck Wiki Access**: Mitigated by manual research if needed
2. **Regex Complexity**: Rust regex limitations (no lookahead/backreferences)
3. **False Positives**: Comprehensive negative test cases
4. **Time Overrun**: Batch implementation allows checkpoints

### Mitigation Strategies
- Break into 3 batches of 5 rules each
- Checkpoint after each batch (verify tests pass)
- Document challenges as they arise
- Use established patterns from Sprints 114-116

## Toyota Way Principles

### 🚨 Jidoka (自働化) - Build Quality In
- EXTREME TDD: RED → GREEN → REFACTOR
- Zero defects: 100% test pass rate required
- Quality gates: Complexity <10, zero clippy warnings

### 🔍 Hansei (反省) - Reflection
- Document technical challenges
- Identify patterns for future sprints
- Improve test templates based on learnings

### 📈 Kaizen (改善) - Continuous Improvement
- Systematic rule expansion
- Consistent test coverage (10+ tests per rule)
- Zero regression maintenance

### 🎯 Genchi Genbutsu (現地現物) - Direct Observation
- Research actual ShellCheck wiki documentation
- Test against real-world shell scripts
- Verify with shellcheck tool output

## Timeline

**Total Estimated Time**: 3-4 hours

- **Phase 1** (Research): 30-45 min
- **Phase 2** (Batch 1): 60-75 min
- **Phase 3** (Batch 2): 60-75 min
- **Phase 4** (Batch 3): 60-75 min
- **Phase 5** (Integration): 30-45 min
- **Phase 6** (Documentation): 20-30 min

**Checkpoints**:
- After Batch 1: 5 rules, ~50 tests
- After Batch 2: 10 rules, ~100 tests
- After Batch 3: 15 rules, ~150 tests

## Next Steps After Sprint 117

### Immediate (Sprint 118)
**Option 1**: Continue to 90% milestone (SC2266-SC2280)
- 15 more rules
- Target: 270/300 (90.0% coverage)
- Duration: ~3-4 hours

**Option 2**: Release v5.1.0 with 85% coverage
- Comprehensive testing
- Performance benchmarks
- Documentation review
- Publish to crates.io

### Future Sprints
- **Sprint 119-120**: Reach 95% coverage (285/300 rules)
- **Sprint 121-122**: Complete 100% SC2xxx coverage (300/300 rules)
- **Sprint 123+**: Expand to other ShellCheck series (SC1xxx, SC3xxx)

## References

- **Previous Sprints**: SPRINT-114-PLAN.md, SPRINT-115-PLAN.md, SPRINT-116-PLAN.md
- **ShellCheck Wiki**: https://www.shellcheck.net/wiki/
- **ROADMAP.yaml**: Project roadmap with sprint history
- **CLAUDE.md**: EXTREME TDD methodology guide

---

**Sprint Start**: 2025-10-23
**Sprint Owner**: EXTREME TDD Team
**Methodology**: RED → GREEN → REFACTOR
**Quality Standard**: Zero Defects, 100% Test Pass Rate
