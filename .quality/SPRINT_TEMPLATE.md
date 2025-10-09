# Sprint XX: [Sprint Name] - EXTREME TDD

**Sprint ID**: sprint-XX
**Duration**: [X weeks/days]
**Goal**: [One sentence sprint goal]
**Status**: [Planning / In Progress / Complete]
**Quality Grade Target**: A+ (≥95)

## Sprint Overview

### Objectives
1. [Primary objective]
2. [Secondary objective]
3. [Tertiary objective]

### Success Criteria
- [ ] All tickets completed (RED-GREEN-REFACTOR)
- [ ] All tests passing (100% pass rate)
- [ ] Quality gates passed (complexity, coverage, SATD)
- [ ] Performance benchmarks met
- [ ] Documentation updated

## Tickets

### TICKET-XXXX: [Ticket Title]
**Priority**: [Critical/High/Medium/Low]
**Status**: [TODO/In Progress/Done]
**Duration**: [Estimated hours]

#### Requirements
- [ ] [Requirement 1]
- [ ] [Requirement 2]
- [ ] [Requirement 3]

#### Tests (RED Phase)
Write these tests FIRST - they should FAIL:

```rust
#[test]
fn test_requirement_1() {
    // RED: This test should fail initially
    todo!("Implement requirement 1");
}

#[test]
fn test_requirement_2() {
    // RED: This test should fail initially
    todo!("Implement requirement 2");
}

proptest! {
    #[test]
    fn prop_requirement_properties(input in generate_test_input()) {
        // RED: Property test should fail initially
        todo!("Implement property verification");
    }
}
```

#### Implementation (GREEN Phase)
Once tests are RED, implement minimal code to make them GREEN:

```rust
// GREEN: Minimal implementation
pub fn new_feature() -> Result<Output> {
    // Just enough code to pass tests
    todo!()
}
```

#### Refactoring (REFACTOR Phase)
Once tests are GREEN, refactor for quality:

- [ ] Extract helper functions
- [ ] Simplify complex logic
- [ ] Add documentation
- [ ] Optimize performance
- [ ] Check complexity metrics

#### Acceptance Criteria
- [ ] All unit tests pass
- [ ] All property tests pass
- [ ] Complexity ≤10 (cyclomatic)
- [ ] Coverage ≥85%
- [ ] No SATD comments
- [ ] Documentation complete
- [ ] ShellCheck passes (if applicable)
- [ ] Determinism verified (if applicable)

#### Verification Commands
```bash
# Run tests
cargo test --test ticket_xxxx

# Check complexity
pmat analyze complexity src/path/to/module.rs

# Check coverage
cargo llvm-cov --html --open

# Run ShellCheck (if generating shell code)
make test-shellcheck

# Run determinism tests (if applicable)
make test-determinism

# Run property tests
cargo test --test property_tests -- proptest_ticket_xxxx
```

---

### TICKET-YYYY: [Next Ticket Title]
[Same structure as above]

---

## Quality Metrics Tracking

### Before Sprint
```yaml
metrics:
  test_count: XXX
  test_pass_rate: 100%
  property_tests: XX
  coverage_core: XX.XX%
  complexity_median: X.X
  mutation_score: XX%
  quality_score: XX/100
  grade: [Letter grade]
```

### After Sprint (Target)
```yaml
metrics:
  test_count: XXX  # Should increase
  test_pass_rate: 100%  # Must maintain
  property_tests: XX  # Should increase
  coverage_core: XX.XX%  # Should maintain or increase
  complexity_median: X.X  # Should maintain or decrease
  mutation_score: XX%  # Should maintain or increase
  quality_score: XX/100  # Should maintain or increase
  grade: A+  # Target
```

### Actual After Sprint
```yaml
metrics:
  test_count: [actual]
  test_pass_rate: [actual]
  property_tests: [actual]
  coverage_core: [actual]
  complexity_median: [actual]
  mutation_score: [actual]
  quality_score: [actual]
  grade: [actual]
```

## Performance Benchmarks

### Baseline
```
benchmark_name: XXX µs ± YY µs
another_benchmark: XXX ms ± YY ms
```

### Target
```
benchmark_name: ≤XXX µs (no regression)
another_benchmark: ≤XXX ms (or XX% improvement)
```

### Actual
```
benchmark_name: [actual result]
another_benchmark: [actual result]
```

## Toyota Way Application

### 自働化 (Jidoka) - Build Quality In
**Applied**:
- [ ] EXTREME TDD followed (RED-GREEN-REFACTOR)
- [ ] Zero defects policy maintained
- [ ] Quality gates enforced

**Evidence**:
- All tests written before implementation
- No SATD comments introduced
- All quality gates passed

### 反省 (Hansei) - Reflection
**Applied**:
- [ ] Five Whys analysis for any issues
- [ ] Root causes documented
- [ ] Lessons learned captured

**Issues Found**:
1. [Issue description] → [Root cause] → [Fix implemented]

### 改善 (Kaizen) - Continuous Improvement
**Applied**:
- [ ] Complexity reduced or maintained
- [ ] Test coverage improved
- [ ] Process improvements identified

**Improvements Made**:
1. [Improvement 1]
2. [Improvement 2]

### 現地現物 (Genchi Genbutsu) - Go and See
**Applied**:
- [ ] Dogfooded on real examples
- [ ] Tested on actual shell interpreters
- [ ] Measured real-world performance

**Observations**:
1. [Observation 1]
2. [Observation 2]

## Sprint Retrospective

### What Went Well ✅
- [Success 1]
- [Success 2]
- [Success 3]

### What Went Wrong ❌
- [Issue 1] → [How we fixed it]
- [Issue 2] → [How we fixed it]

### What We Learned 📚
- [Learning 1]
- [Learning 2]
- [Learning 3]

### What We'll Do Differently Next Sprint 🔄
- [Change 1]
- [Change 2]
- [Change 3]

## Technical Debt

### Debt Added This Sprint
- **None** (zero tolerance policy) ✅

### Debt Resolved This Sprint
- [Debt item 1] - [Ticket that resolved it]
- [Debt item 2] - [Ticket that resolved it]

## Blockers Encountered

### Blocker 1: [Description]
**Impact**: [High/Medium/Low]
**Resolution**: [How it was resolved]
**Time Lost**: [Hours/Days]
**Prevention**: [How to prevent in future]

## Dependencies

### External Dependencies
- [Dependency 1] - [Status]
- [Dependency 2] - [Status]

### Internal Dependencies
- [Module 1] - [Status]
- [Module 2] - [Status]

## Documentation Updates

### Files Updated
- [ ] ROADMAP.md - Sprint completion documented
- [ ] CHANGELOG.md - Changes documented
- [ ] API documentation - New features documented
- [ ] Test documentation - New tests documented
- [ ] Architecture docs - Changes documented

### New Documentation Created
- [ ] [Document 1] - [Purpose]
- [ ] [Document 2] - [Purpose]

## Release Notes (if applicable)

### Version: vX.Y.Z

#### New Features
- [Feature 1]
- [Feature 2]

#### Improvements
- [Improvement 1]
- [Improvement 2]

#### Bug Fixes
- [Fix 1]
- [Fix 2]

#### Breaking Changes
- **None** (or list if any)

#### Migration Guide
- [Steps if breaking changes]

## CI/CD Status

### Build Status
- [ ] All builds passing
- [ ] No warnings
- [ ] No clippy violations

### Test Status
- [ ] Unit tests: XXX/XXX passing
- [ ] Property tests: XX properties
- [ ] Integration tests: passing
- [ ] ShellCheck tests: passing
- [ ] Determinism tests: passing

### Coverage Status
- [ ] Core modules: ≥85%
- [ ] Total project: ≥80%
- [ ] Uploaded to Codecov

### Quality Gates
- [ ] Complexity: ≤10
- [ ] SATD: 0
- [ ] Dead code: <0.5%
- [ ] Mutation score: ≥90% (or baseline documented)

## Commit History

### Sprint Commits
```
git log --oneline --since="[sprint start date]" --until="[sprint end date]"
```

### Atomic Commits per Ticket
- [Commit hash] - TICKET-XXXX: [description]
- [Commit hash] - TICKET-YYYY: [description]

## Sprint Velocity

### Planned vs Actual
- **Planned tickets**: [N]
- **Completed tickets**: [M]
- **Velocity**: [M/N * 100]%

### Time Tracking
- **Estimated time**: [X hours]
- **Actual time**: [Y hours]
- **Variance**: [Y-X hours] ([variance %])

## Next Sprint Planning

### Carryover Items
- [Item 1] - [Reason not completed]
- [Item 2] - [Reason not completed]

### Proposed Focus
- [Focus area 1]
- [Focus area 2]

### Risk Assessment
- **High Risk**: [Risk 1] - [Mitigation strategy]
- **Medium Risk**: [Risk 2] - [Mitigation strategy]

## Stakeholder Communication

### Status Summary
[One paragraph summary for stakeholders]

### Demo Items
1. [Feature 1] - [Demo script/video]
2. [Feature 2] - [Demo script/video]

### Questions for Stakeholders
1. [Question 1]
2. [Question 2]

---

## Appendix

### Code Statistics
```bash
# Lines of code
tokei src/

# Test to code ratio
[Calculate ratio]

# Documentation coverage
cargo doc --no-deps
```

### Benchmark Data
```bash
# Run benchmarks
cargo bench

# Compare to baseline
[Comparison results]
```

### Coverage Report
```bash
# Generate coverage
make coverage

# View report
[Link to coverage report]
```

---

**Sprint Start**: YYYY-MM-DD
**Sprint End**: YYYY-MM-DD
**Sprint Leader**: [Name]
**Reviewers**: [Names]
**Status**: [Complete/Incomplete]
**Quality Grade Achieved**: [A+/A/A-/etc.]

---

## Sign-off

- [ ] All acceptance criteria met
- [ ] All quality gates passed
- [ ] Documentation complete
- [ ] Sprint retrospective complete
- [ ] Next sprint planned

**Approved by**: [Name]
**Date**: YYYY-MM-DD
