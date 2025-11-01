# Pre-Commit Quality Gates

## Overview

bashrs enforces **NASA-level quality standards** through automated pre-commit hooks. These hooks prevent defects from entering the codebase by validating code quality, complexity, and test coverage before each commit.

## Quality Gates

### 1. Clippy (Zero Tolerance)
**Tool**: `cargo clippy --lib -- -D warnings`
**Enforcement**: **BLOCKING** (commit fails if warnings found)
**Target**: Zero warnings

Checks for:
- Code style violations
- Potential bugs
- Performance issues
- Idiomatic Rust usage

**Fix automatically**:
```bash
cargo clippy --lib --fix --allow-dirty
```

### 2. Performance Lints
**Tool**: `cargo clippy --release -- -W clippy::perf`
**Enforcement**: **BLOCKING** (commit fails if issues found)
**Target**: No performance regressions

Checks for:
- Inefficient algorithms
- Unnecessary allocations
- Suboptimal data structures

### 3. Test Suite
**Tool**: `cargo test --lib`
**Enforcement**: **BLOCKING** (commit fails if tests fail)
**Target**: 100% pass rate, >85% coverage

Ensures:
- All unit tests pass
- No regressions introduced
- Property tests pass (100+ cases per module)

### 4. Code Complexity (pmat)
**Tool**: `pmat analyze complexity --max-complexity 15`
**Enforcement**: **WARNING** (does not block, but warns)
**Target**: All functions <15 cyclomatic complexity (ideal: <10)

Checks for:
- High complexity functions (>15)
- Nested control flow
- Functions needing refactoring

**View full report**:
```bash
pmat analyze complexity
```

**Best practice**: Refactor functions >10 complexity using EXTREME TDD:
1. RED: Write property tests
2. GREEN: Extract helper functions
3. REFACTOR: Reduce complexity
4. PROPERTY: Add 100+ generative tests
5. MUTATION: Run cargo-mutants

### 5. Technical Debt (pmat SATD)
**Tool**: `pmat analyze satd --path rash/src`
**Enforcement**: **WARNING** (does not block, but warns)
**Target**: No Critical or High priority SATD violations

Checks for:
- TODO/FIXME comments in production code
- Self-Admitted Technical Debt
- Critical security issues
- High-priority defects

**Severity Levels**:
- **Critical**: Security vulnerabilities, must fix immediately
- **High**: Defects requiring attention
- **Medium**: Design improvements
- **Low**: Nice-to-have enhancements

**View full report**:
```bash
pmat analyze satd --path rash/src
```

## Hook Installation

The pre-commit hook is tracked in the repository at `scripts/hooks/pre-commit.sh` and linked into `.git/hooks/`.

**Installation** (run once after cloning):
```bash
ln -sf ../../scripts/hooks/pre-commit.sh .git/hooks/pre-commit
```

**Verification**:
```bash
.git/hooks/pre-commit  # Should run all quality gates
```

The hook is tracked in git so the team can maintain it collaboratively.

## Hook Output Example

```
🔍 bashrs Pre-commit Quality Gates
====================================

📊 Quality Gate Checks
----------------------
  1. Clippy (zero warnings)... ✅
  2. Performance lints... ✅
  3. Test suite... ✅
  4. Code complexity (max: 10)... ⚠️  (18 functions >10)
     Goal: Refactor high-complexity functions with EXTREME TDD
     Run: pmat analyze complexity
  5. Technical debt (zero tolerance goal)... ⚠️  (1 critical, 3 high)
     Production code has Critical/High SATD - should fix soon
     Run: pmat analyze satd --path rash/src
  6. Code formatting... ✅
  7. Documentation sync... ✅

Quality Gate Summary
--------------------
✅ All quality gates passed!

Commit is ready to proceed.
```

**Note**: Warnings (⚠️) display progress toward quality goals but don't block commits.

## Bypassing Hooks (Emergency Only)

In exceptional circumstances, you can bypass hooks:
```bash
git commit --no-verify -m "emergency fix"
```

**⚠️ WARNING**: Only use `--no-verify` for critical hotfixes. All bypassed commits must be fixed immediately after deployment.

## Hook Execution Time

- **Fast path** (<10s): Clippy, tests, complexity check
- **Normal workflow**: Fits within developer flow
- **Parallel execution**: Multiple checks run concurrently

## Quality Metrics Tracked

| Metric | Target | Enforcement |
|--------|--------|-------------|
| Clippy warnings | 0 | **Blocking** |
| Performance issues | 0 | **Blocking** |
| Test pass rate | 100% | **Blocking** |
| Code formatting | 100% | **Blocking** |
| Code coverage | >85% | CI only |
| Cyclomatic complexity | <10 | Warning (goal) |
| SATD Critical/High | 0 | Warning (track) |

**Enforcement Levels**:
- **Blocking**: Commit fails if check fails (clippy, tests, performance, formatting)
- **Warning**: Displayed but doesn't block (complexity, SATD - track progress toward goals)
- **CI only**: Checked in continuous integration, not pre-commit

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
Pre-commit hooks **prevent defects** rather than finding them later.

### 反省 (Hansei) - Reflection
Warnings prompt reflection: "Should I refactor this before committing?"

### 改善 (Kaizen) - Continuous Improvement
Quality gates evolve based on lessons learned from defects.

### アンドン (Andon) - Stop the Line
**Blocking failures** stop the line - fix before proceeding.

## Integration with EXTREME TDD

Pre-commit hooks enforce the EXTREME TDD workflow:
1. **RED**: Tests must pass (checked by hook)
2. **GREEN**: Clippy must pass (checked by hook)
3. **REFACTOR**: Complexity must be reasonable (checked by hook)
4. **PROPERTY**: Property tests included in test suite
5. **MUTATION**: CI-level check (too slow for pre-commit)

## Future Enhancements

Planned additions to pre-commit hooks:
- **Coverage check**: Fail if coverage drops below 85%
- **Mutation testing**: Sample-based fast mutation checks
- **Security scanning**: Detect known vulnerabilities
- **License compliance**: Ensure all dependencies are MIT/Apache2

## Troubleshooting

### Hook Too Slow
If pre-commit hooks take >30s:
1. Check if incremental compilation is working
2. Run `cargo clean` to reset build cache
3. Consider using `cargo-watch` for faster feedback

### False Positives in Complexity
If complexity warnings are incorrect:
1. Review pmat complexity calculation
2. Consider extracting helper functions anyway
3. Document why complexity is acceptable (with comment)

### SATD Warnings for Valid TODOs
Mark intentional TODOs with context:
```rust
// TODO(ISSUE-123): Implement feature X after API stabilizes
```

## References

- CLAUDE.md - Development guidelines
- REPL-DEBUGGER-ROADMAP.yaml - Quality targets
- docs/completions/REPL-016-001-COMPLETE.md - Performance targets

---

**Remember**: Pre-commit hooks are the first line of defense. They save hours of debugging by catching issues early.

**Quality is not negotiable.** 🎯
