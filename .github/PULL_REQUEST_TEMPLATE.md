# Pull Request - EXTREME TDD Checklist

## Ticket Reference
**Ticket**: RASH-XXXX
**Sprint**: Sprint XX
**Type**: [Feature / Bug Fix / Refactor / Documentation]

## Description
[Clear description of what this PR does]

### Problem
[What problem does this solve?]

### Solution
[How does this PR solve it?]

## RED-GREEN-REFACTOR Checklist

### RED Phase ✅
- [ ] Wrote failing tests FIRST
- [ ] Tests fail for the right reason
- [ ] Tests cover all requirements
- [ ] Property tests added (if applicable)
- [ ] Edge cases identified and tested

### GREEN Phase ✅
- [ ] Minimal implementation to pass tests
- [ ] All tests now passing
- [ ] No test modifications (only implementation changed)
- [ ] No extra features beyond requirements

### REFACTOR Phase ✅
- [ ] Code refactored for clarity
- [ ] Complexity within limits (≤10 cyclomatic, ≤15 cognitive)
- [ ] Extracted helper functions where needed
- [ ] Documentation added/updated
- [ ] All tests still passing after refactor

## Quality Gates

### Code Quality ✅
- [ ] `cargo fmt` - Code formatted correctly
- [ ] `cargo clippy` - No warnings
- [ ] Complexity ≤10 (cyclomatic), ≤15 (cognitive)
- [ ] No SATD comments (TODO, FIXME, HACK, etc.)
- [ ] No dead code introduced
- [ ] No unsafe code (unless absolutely necessary + justified)

### Testing ✅
- [ ] All tests passing (100% pass rate)
- [ ] Unit tests added for new code
- [ ] Property tests added (if applicable)
- [ ] Integration tests updated (if needed)
- [ ] Doc tests working
- [ ] Test coverage ≥85% for core modules

### Transpiler-Specific ✅
- [ ] ShellCheck validation passes (`-s sh`)
- [ ] Generated code is POSIX compliant
- [ ] Determinism verified (byte-identical output)
- [ ] No shell injection vulnerabilities
- [ ] Proper escaping for all user input
- [ ] Output overhead <20 lines boilerplate

### Performance ✅
- [ ] No performance regressions
- [ ] Benchmarks run (if applicable)
- [ ] Transpile time <50µs (simple cases)
- [ ] Memory usage reasonable (<100MB)

### Documentation ✅
- [ ] Public APIs documented
- [ ] Examples added to docs
- [ ] ROADMAP.md updated (if sprint complete)
- [ ] CHANGELOG.md updated (if user-facing change)
- [ ] Architecture docs updated (if design change)

## Edge Cases

### Tested Edge Cases ✅
- [ ] Empty input
- [ ] Deeply nested structures (100+ levels if recursive)
- [ ] Unicode/special characters
- [ ] Large inputs (stress testing)
- [ ] Error conditions
- [ ] Boundary values (min/max)

### Edge Case Results
[List any interesting findings or behaviors]

## Breaking Changes
- [ ] No breaking changes
- [ ] Breaking changes documented below with migration guide

**Breaking Changes** (if any):
```
[Describe breaking changes and how to migrate]
```

## Performance Impact

### Benchmarks
```
Before: XXX µs
After:  YYY µs
Change: ±Z% [improvement/regression]
```

### Memory
```
Before: XX MB
After:  YY MB
Change: ±Z MB
```

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- [ ] Zero defects policy followed
- [ ] Quality built in, not inspected in
- [ ] Automated quality checks passed

**Evidence**: [How quality was built in]

### 反省 (Hansei) - Reflection
- [ ] Five Whys applied (if fixing bug)
- [ ] Root cause documented
- [ ] Lessons learned captured

**Root Cause** (if bug fix): [Brief explanation]

### 改善 (Kaizen) - Continuous Improvement
- [ ] Complexity reduced or maintained
- [ ] Test coverage improved
- [ ] Process improvements identified

**Improvements**: [What got better]

### 現地現物 (Genchi Genbutsu) - Go and See
- [ ] Dogfooded on real examples
- [ ] Tested on actual shell interpreters (sh, dash, ash, busybox)
- [ ] Measured real performance

**Testing Environment**: [Where was this tested]

## Verification Commands

Run these commands to verify the PR:

```bash
# Format check
cargo fmt -- --check

# Lint check
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
make test-all

# Coverage check
make coverage

# Complexity check (if pmat installed)
pmat analyze complexity src/

# ShellCheck validation
make test-shellcheck

# Determinism check
make test-determinism

# Full quality gates
./scripts/quality-gates.sh
```

## Test Output

### Unit Tests
```
[Paste test output showing all tests pass]
```

### Property Tests
```
[Paste property test output]
```

### Coverage Report
```
Coverage: XX.XX%
Core modules: XX.XX%
```

## Mutation Testing (if run)
```
Mutation score: XX%
Mutants killed: XX/YY
```

## Reviewer Checklist

### Code Review ✅
- [ ] Code is readable and maintainable
- [ ] Logic is sound and correct
- [ ] Error handling is comprehensive
- [ ] Edge cases are covered
- [ ] No obvious bugs or security issues

### Test Review ✅
- [ ] Tests are meaningful (not just for coverage)
- [ ] Tests follow GIVEN-WHEN-THEN pattern
- [ ] Property tests cover important invariants
- [ ] Tests are fast (<3 min total)

### Design Review ✅
- [ ] Design is simple and clear
- [ ] Follows existing patterns
- [ ] No unnecessary complexity
- [ ] Proper separation of concerns
- [ ] Adheres to SOLID principles

## Additional Context

### Related Issues
- Closes #XXX
- Related to #YYY

### Dependencies
- Depends on PR #ZZZ
- Blocks PR #AAA

### Screenshots/Examples
[If applicable, add examples of generated shell code or other visual evidence]

### Migration Guide (if breaking)
```
[Step-by-step migration instructions]
```

## Post-Merge Actions
- [ ] Update version number (if needed)
- [ ] Tag release (if needed)
- [ ] Update documentation site (if needed)
- [ ] Announce in changelog (if user-facing)

---

## Acknowledgments
[Thank anyone who helped with this PR]

## Sign-off

**Author**: @username
**Reviewer**: @reviewer
**Quality Score**: [A+/A/A-/etc.]

---

By submitting this PR, I confirm that:
- [ ] I have followed the EXTREME TDD methodology (RED-GREEN-REFACTOR)
- [ ] All quality gates have passed
- [ ] I have tested this on real examples
- [ ] I have documented all changes
- [ ] I am ready for code review

**Zero Tolerance**: This PR contains no SATD comments, no quality violations, and maintains 100% test pass rate.
