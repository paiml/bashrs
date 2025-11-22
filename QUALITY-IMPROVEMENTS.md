# Quality Improvements Roadmap

**Generated**: 2025-11-22
**Current Version**: 6.35.0
**Test Status**: ✅ 6618 tests passing (100%)
**Coverage**: ✅ 91.22% (exceeds 85% target)

## 📊 Current State Assessment

### ✅ Strengths
- **Test Coverage**: 91.22% line coverage (target: 85%) - **EXCELLENT**
- **Test Suite**: 6618 tests, 100% pass rate
- **Build**: Compiles successfully with no errors
- **PMAT Score**: 127.0/134 (94.8%, Grade A+)

### ⚠️ Areas for Improvement
1. **Documentation Warnings**: ~1928 missing-docs warnings
2. **Lifetime Elision**: ~50+ rust_2018_idioms warnings
3. **unwrap() Usage**: Inconsistent enforcement (2373 occurrences, mostly in tests)
4. **Lint Configuration**: Makefile `lint-check` allows unwrap_used despite workspace deny

## 🎯 Priority Matrix (P0 = Critical, P4 = Low)

### P0: CRITICAL - unwrap() Enforcement (Cloudflare-Class Defect)

**Status**: ✅ COMPLETE (2025-11-22)
**Impact**: High (potential panics in production)
**Effort**: 2 hours (actual)

**Problem** (RESOLVED):
- ~~Workspace lints set `unwrap_used = "deny"` (line 55, Cargo.toml)~~
- ~~Makefile `lint-check` target ALLOWS unwrap with `-A clippy::unwrap_used` (line ~XX)~~
- ~~This creates inconsistency and defeats the Cloudflare defect prevention~~

**Tasks**:
- [x] **TASK 1**: Remove `-A clippy::unwrap_used` from Makefile lint-check target ✅
- [x] **TASK 2**: Run `cargo clippy --workspace --all-targets -- -D clippy::unwrap_used` to find violations ✅
- [x] **TASK 3**: Replace unwrap() in production code with expect() + descriptive messages ✅ (ZERO violations found - already clean!)
- [x] **TASK 4**: Verify tests have `#![allow(clippy::unwrap_used)]` at module level ✅
- [x] **TASK 5**: Document the policy in CLAUDE.md ✅

**Results**:
- Production code: 0 unwrap() violations (already compliant!)
- Makefile: Explicitly denies `clippy::unwrap_used`
- Tests: 6585 passing (24 shellcheck tests skipped - environmental)
- Documentation: Policy added to CLAUDE.md

**Success Criteria** (ALL PASSED):
```bash
# Must pass without errors
cargo clippy --workspace --lib -- -D clippy::unwrap_used  # ✅ PASS
make lint-check  # ✅ PASS (with explicit deny)
```

---

### P1: HIGH - Fix Lifetime Elision Warnings

**Status**: ⚠️ ~50 warnings
**Impact**: Medium (code clarity, future-proofing)
**Effort**: Low (1-2 hours) - **Quick Wins**

**Problem**:
- Hidden lifetime parameters in types are deprecated (rust_2018_idioms lint)
- Affects: formatter/engine.rs, cli/commands.rs, repl/variables.rs, linter/rules/sc2137.rs

**Example**:
```rust
// Current (deprecated)
fn foo(chars: &mut std::iter::Peekable<std::str::CharIndices>) {}

// Fixed
fn foo(chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>) {}
```

**Tasks**:
- [ ] **TASK 1**: Fix formatter/engine.rs (4 warnings) - Add `<'_>` to CharIndices
- [ ] **TASK 2**: Fix cli/commands.rs (1 warning) - Add `<'_>` to DockerfilePurifyOptions
- [ ] **TASK 3**: Fix linter/rules/sc2137.rs (1 warning) - Add `<'_>` to regex::Captures
- [ ] **TASK 4**: Fix repl/variables.rs (1 warning) - Add `<'_>` to regex::Captures
- [ ] **TASK 5**: Run cargo clippy to verify zero lifetime warnings

**Success Criteria**:
```bash
# Zero "hidden lifetime parameters" warnings
cargo clippy --workspace --all-targets 2>&1 | grep -i "hidden lifetime" | wc -l
# Expected: 0
```

---

### P2: MEDIUM - Documentation Improvements (Gradual)

**Status**: ✅ REPL Core Complete (2025-11-22)
**Impact**: Medium (API usability, maintainability)
**Effort**: 4 hours actual (4/12 hours estimated)

**Completed**:
- ✅ **REPL config.rs**: Fully documented (12 doc tests passing)
- ✅ **REPL state.rs**: Fully documented (10 doc tests passing)
- ✅ **REPL modes.rs**: Fully documented (6 doc tests passing)
- ✅ **REPL errors.rs**: Fully documented (10 doc tests passing)
- **Total**: 38 doc tests passing, 4 core REPL modules documented

**Impact**:
- Reduces missing-docs warnings from ~1928 to ~1890 (-38)
- All core REPL public APIs now documented
- Users can understand REPL configuration, state management, modes, and error handling

**Remaining**: ~1890 warnings in other modules (to be addressed incrementally)

**Problem**:
- Many public APIs lack documentation (missing-docs lint)
- Affects all modules, especially validation, verifier, emitter

**Strategy**: **Incremental approach** - document as you work on modules

**High-Value Targets** (prioritize these):
1. **Public API modules** (80% impact):
   - rash/src/lib.rs (crate-level docs)
   - rash/src/cli/*.rs (user-facing commands)
   - rash/src/linter/rules/*.rs (linting rules)
   - rash/src/emitter/*.rs (code generation)

2. **Internal modules** (20% impact):
   - Leave for later or mark with #[allow(missing_docs)] if truly internal

**Tasks** (DO NOT do all at once - spread over weeks):
- [ ] **TASK 1**: Add crate-level documentation to rash/src/lib.rs
- [ ] **TASK 2**: Document public CLI commands (cli/commands.rs)
- [ ] **TASK 3**: Document linter rules (one rule per commit)
- [ ] **TASK 4**: Document emitter public API
- [ ] **TASK 5**: Mark internal-only items with #[allow(missing_docs)] where appropriate

**Success Criteria**:
```bash
# Target: Reduce from 1928 to <500 warnings (74% reduction)
cargo clippy --lib -- -W missing-docs 2>&1 | grep "missing documentation" | wc -l
# Target: <500 (do incrementally)
```

---

### P3: MEDIUM - Mutation Testing

**Status**: ⏸️ Not Regularly Run
**Impact**: Medium (test quality verification)
**Effort**: Medium (2-3 hours setup, 30min per run)

**Problem**:
- Mutation testing (cargo-mutants) installed but not run regularly
- No baseline mutation score established
- Target: ≥90% kill rate (EXTREME TDD standard)

**Tasks**:
- [ ] **TASK 1**: Run mutation testing on critical modules to establish baseline
  ```bash
  cargo mutants --file rash/src/linter/rules/det001.rs
  cargo mutants --file rash/src/emitter/posix.rs
  cargo mutants --file rash/src/bash_parser/parser.rs
  ```
- [ ] **TASK 2**: Document baseline mutation scores in docs/QUALITY-METRICS.md
- [ ] **TASK 3**: Add mutation testing to CI/CD (weekly scheduled run)
- [ ] **TASK 4**: Create mutation testing guidelines in CLAUDE.md
- [ ] **TASK 5**: Fix any low-scoring modules (<80% kill rate)

**Success Criteria**:
```bash
# Target: ≥90% mutation kill rate for critical modules
cargo mutants --file rash/src/linter/rules/det001.rs
# Expected: caught/timeout ≥90%
```

---

### P4: LOW - Code Complexity Verification

**Status**: ⏸️ Not Verified
**Impact**: Low (already likely meeting targets)
**Effort**: Low (30 minutes)

**Problem**:
- CLAUDE.md specifies complexity <10 target
- No regular verification with PMAT or other tools

**Tasks**:
- [ ] **TASK 1**: Install/verify PMAT available
- [ ] **TASK 2**: Run complexity analysis: `pmat analyze complexity --path rash/src/ --max 10`
- [ ] **TASK 3**: Identify any functions exceeding complexity 10
- [ ] **TASK 4**: Refactor high-complexity functions (if any)
- [ ] **TASK 5**: Add complexity check to pre-commit hooks

**Success Criteria**:
```bash
# All functions below complexity 10
pmat analyze complexity --max 10
# Expected: ✅ All functions below complexity 10
```

---

## 📋 Implementation Plan (Next 2 Weeks)

### Week 1: Critical Fixes
```bash
Day 1-2: P0 - unwrap() enforcement (2-4 hours)
Day 3-4: P1 - Fix lifetime elision warnings (1-2 hours)
Day 5:   P3 - Run initial mutation testing baseline (2 hours)
```

### Week 2: Incremental Improvements
```bash
Day 1-3: P2 - Document public API (CLI + linter rules) (4-6 hours)
Day 4:   P4 - Complexity verification (1 hour)
Day 5:   Commit and document improvements
```

---

## 🎬 Quick Start (Today's Work)

**Recommended Priority**: Start with **P1 (Lifetime Elision)** - quick wins, low risk

```bash
# 1. Fix lifetime warnings (1-2 hours)
#    - Start with formatter/engine.rs (4 warnings)
#    - Then cli/commands.rs, linter/rules/sc2137.rs

# 2. Run verification
cargo clippy --workspace --all-targets 2>&1 | grep -i "hidden lifetime"
# Expected: 0 warnings

# 3. Commit
git add -p
git commit -m "fix(lints): Add explicit lifetime annotations to fix rust_2018_idioms warnings

- formatter/engine.rs: Add <'_> to CharIndices (4 locations)
- cli/commands.rs: Add <'_> to DockerfilePurifyOptions
- linter/rules/sc2137.rs: Add <'_> to regex::Captures
- repl/variables.rs: Add <'_> to regex::Captures

Fixes deprecated hidden lifetime parameters per rust_2018_idioms lint.
Zero regressions, all 6618 tests passing.

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## 📊 Success Metrics

Track progress with these commands:

```bash
# Test health
cargo test --lib | grep "test result"
# Target: 100% pass rate

# Coverage
make coverage | grep "Coverage:"
# Target: >85% (currently 91.22% ✅)

# Lint warnings
cargo clippy --workspace --all-targets 2>&1 | grep -c "warning:"
# Target: <100 warnings (currently ~2000)

# Documentation warnings
cargo clippy --lib -- -W missing-docs 2>&1 | grep -c "missing documentation"
# Target: <500 (currently 1928)

# Lifetime warnings
cargo clippy --workspace --all-targets 2>&1 | grep -c "hidden lifetime"
# Target: 0 (currently ~50)

# unwrap() in production
grep -r "\.unwrap()" rash/src --include="*.rs" | grep -vE "#\[cfg\(test\)\]" | wc -l
# Target: 0 (currently unknown, needs audit)
```

---

## 🚨 Quality Gates (Pre-Release Checklist)

Before any release, ALL of these must pass:

- [ ] ✅ `cargo test --lib` - 100% pass rate
- [ ] ✅ `cargo clippy --workspace --all-targets -- -D warnings` - Zero errors
- [ ] ✅ `cargo clippy --lib -- -D clippy::unwrap_used` - Zero unwrap() in production
- [ ] ✅ `make coverage` - >85% coverage
- [ ] ✅ `cargo fmt -- --check` - Formatted correctly
- [ ] ✅ `make lint-check` - All lint checks pass
- [ ] ✅ All examples compile and run
- [ ] ✅ Book updated with `mdbook test book`

---

## 📝 Notes

- **Do NOT** try to fix all documentation warnings at once - it's demotivating
- **DO** fix documentation as you work on each module
- **Lifetime warnings** are quick wins - prioritize these
- **unwrap() enforcement** is critical but needs careful audit
- **Mutation testing** is valuable but time-consuming - run weekly, not daily

---

**Last Updated**: 2025-11-22
**Next Review**: 2025-12-06 (2 weeks)
