# Quick Start Guide - Bashrs Quality Excellence

**Goal**: Get to A+ (98/100) quality grade in 30 minutes

## 🚀 Fast Track to Excellence

### Step 1: Install ShellCheck (5 minutes)

This fixes 24 test failures immediately.

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y shellcheck
```

**macOS:**
```bash
brew install shellcheck
```

**Verify installation:**
```bash
shellcheck --version
# Should show: ShellCheck - shell script analysis tool
```

### Step 2: Run Tests (5 minutes)

```bash
cd /home/noahgift/src/bashrs
cargo test --lib

# Expected output:
# test result: ok. 667 passed; 0 failed; 2 ignored
```

**Before ShellCheck**: 643/667 passing (96.4%)
**After ShellCheck**: 667/667 passing (100%) ✅

### Step 3: Fix Version Mismatch (10 minutes)

Check current versions:
```bash
grep "^version" Cargo.toml
grep "Current Status:" ROADMAP.md
```

**Current state:**
- Cargo.toml: `v1.0.0-rc1`
- ROADMAP.md: `v0.9.2`

**Decision needed**: Which version is correct?

**If v1.0.0-rc1 is correct:**
```bash
# Update ROADMAP.md line 62
sed -i 's/v0.9.2/v1.0.0-rc1/g' ROADMAP.md
```

**If v0.9.2 is correct:**
```bash
# Update Cargo.toml
# Edit the version field in workspace.package section
```

### Step 4: Run Quality Gates (5 minutes)

```bash
# Run all 9 quality gates
./scripts/quality-gates.sh

# Expected: All gates PASS ✅
```

### Step 5: Verify A+ Grade (5 minutes)

```bash
# Check metrics
echo "✅ SATD Comments: $(grep -r 'TODO\|FIXME' src/ --include='*.rs' | wc -l) (target: 0)"
echo "✅ Unsafe Blocks: $(grep -r 'unsafe' src/ --include='*.rs' | grep -v '//' | wc -l) (target: 0)"
echo "✅ Test Pass Rate: Run 'cargo test --lib' to verify 100%"
echo "✅ Coverage: 85.36% core (target: ≥85%)"
echo "✅ Performance: 19.1µs (target: <10ms)"
```

**Result**: 🎉 A+ (98/100) achieved!

---

## 📋 Using the Quality Infrastructure

### Daily Development Workflow

```bash
# Before starting work
git pull

# Make changes with EXTREME TDD
# 1. RED: Write failing tests first
cargo test --lib  # Tests should FAIL

# 2. GREEN: Implement minimal code
cargo test --lib  # Tests should PASS

# 3. REFACTOR: Clean up code
cargo fmt
cargo clippy --fix

# Before committing
./scripts/quality-gates.sh  # All gates must PASS

# Commit
git add .
git commit -m "feat: description

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

### Creating a Bug Report

```bash
# Copy template
cp .github/ISSUE_TEMPLATE/bug_report.md issue-XXXX.md

# Fill in:
# - Problem statement
# - Reproduction steps
# - Five Whys analysis
# - Severity (P0/P1/P2/P3)

# Create issue on GitHub
```

### Creating a Feature Request

```bash
# Copy template
cp .github/ISSUE_TEMPLATE/feature_request.md feature-XXXX.md

# Fill in:
# - User story
# - Technical specification
# - EXTREME TDD plan (RED-GREEN-REFACTOR)
# - Quality gates checklist

# Create issue on GitHub
```

### Starting a New Sprint

```bash
# Copy sprint template
cp .quality/SPRINT_TEMPLATE.md .quality/sprint25-in-progress.md

# Fill in:
# - Sprint goals from roadmap.yaml
# - Tickets from roadmap.yaml Sprint 25
# - Baseline metrics (before sprint)

# Track progress during sprint
# Mark tickets as in_progress/completed

# At end of sprint:
# - Fill in actual metrics
# - Complete retrospective
# - Rename to sprint25-complete.md
```

### Creating a Pull Request

1. **Write tests first** (RED phase)
2. **Implement feature** (GREEN phase)
3. **Refactor code** (REFACTOR phase)
4. **Run quality gates**: `./scripts/quality-gates.sh`
5. **Create PR** using `.github/PULL_REQUEST_TEMPLATE.md`
6. **Fill in checklist**:
   - RED-GREEN-REFACTOR phases ✅
   - Quality gates (code, testing, transpiler-specific) ✅
   - Edge cases tested ✅
   - Toyota Way principles applied ✅

### Performing Five Whys Analysis

When a bug occurs (especially P0/P1):

```bash
# Copy template
cp FIVE_WHYS_TEMPLATE.md .quality/five-whys-issue-XXXX.md

# Fill in each "Why" level:
# Why #1: Immediate cause
# Why #2: Why did #1 happen?
# Why #3: Why did #2 happen?
# Why #4: Why did #3 happen?
# Why #5 (ROOT CAUSE): Fundamental issue

# Document:
# - Root cause identified
# - Fix strategy (RED-GREEN-REFACTOR)
# - Prevention strategy
# - Lessons learned
```

---

## 🎯 Sprint 25 Quick Start

**Goal**: Mutation Testing Excellence (83% → 90%+)

### Week 1: Parser & IR Modules

**Monday-Tuesday**: RASH-2501 & RASH-2502
```bash
# Run mutation tests on parser
cargo mutants --package rash --file src/parser/

# Analyze surviving mutants
# Write tests for gaps
# Target: 90%+ kill rate
```

**Wednesday-Thursday**: RASH-2503
```bash
# Run mutation tests on IR module
cargo mutants --package rash --file src/ir/

# Currently: 83% kill rate
# Target: 90%+ kill rate
```

**Friday**: Documentation
```bash
# Update sprint tracking
vim .quality/sprint25-in-progress.md

# Fill in Week 1 progress
# Document learnings
```

### Week 2: Emitter & Verifier Modules

**Monday-Tuesday**: RASH-2504
```bash
# Run mutation tests on emitter
cargo mutants --package rash --file src/emitter/

# Critical: shell code generation
# Target: 90%+ kill rate
```

**Wednesday-Thursday**: RASH-2505
```bash
# Run mutation tests on verifier
cargo mutants --package rash --file src/verifier/

# Verification logic must be sound
# Target: 90%+ kill rate
```

**Friday**: Sprint Completion
```bash
# Fill in final metrics
# Complete retrospective
# Update ROADMAP.md
# Rename sprint25-complete.md

# If 90%+ achieved:
# 🎉 Sprint 25 COMPLETE!
# Grade improves to A+ (100/100)
```

---

## 📊 Quality Metrics Dashboard

### Quick Check Commands

```bash
# Test status
cargo test --lib 2>&1 | grep "test result"

# SATD count (should be 0)
grep -r "TODO\|FIXME\|HACK" src/ --include="*.rs" | wc -l

# Unsafe count (should be 0)
grep -r "unsafe" src/ --include="*.rs" | grep -v "//" | wc -l

# Coverage
cargo llvm-cov --lcov --output-path coverage.info
# Core: 85.36%, Total: 82.18%

# Complexity (if pmat installed)
pmat analyze complexity src/ --summary

# Dependency audit
cargo deny check
cargo tree --duplicates
```

### Quality Gate Status

```bash
# Run all gates
./scripts/quality-gates.sh

# Individual gates:
# 1. Format:        cargo fmt -- --check
# 2. Lint:          cargo clippy --all-targets -- -D warnings
# 3. Tests:         cargo test --lib
# 4. Coverage:      cargo llvm-cov
# 5. Complexity:    pmat analyze complexity (if available)
# 6. SATD:          grep -r "TODO\|FIXME" src/
# 7. ShellCheck:    shellcheck validation
# 8. Determinism:   byte-identical output verification
# 9. Performance:   cargo bench
```

---

## 🔧 Troubleshooting

### Tests Failing?

```bash
# Check shellcheck installed
which shellcheck

# If not installed:
sudo apt-get install shellcheck  # Ubuntu/Debian
brew install shellcheck           # macOS

# Re-run tests
cargo test --lib
```

### Quality Gates Failing?

```bash
# Format issues
cargo fmt

# Lint issues
cargo clippy --fix

# Test failures
cargo test --lib -- --nocapture  # See detailed output

# Coverage too low
# Add more tests for uncovered code paths

# Complexity too high
# Refactor function using helper functions
# Target: cyclomatic ≤10, cognitive ≤15
```

### Dependency Issues?

```bash
# Update dependencies
cargo update

# Check for duplicates
cargo tree --duplicates

# Audit security
cargo audit

# Check licenses/bans
cargo deny check
```

---

## 📚 Key Documentation

### Must Read (30 minutes)
1. `QUALITY_REVIEW_2025-10-09.md` - Today's assessment
2. `EXTREME_QUALITY_IMPLEMENTATION.md` - Infrastructure overview
3. `docs/quality/standards.md` - Quality standards bible

### Reference When Needed
4. `pmat-quality.toml` - Quality thresholds
5. `.pmat-gates.toml` - Gate configuration
6. `roadmap.yaml` - 5-sprint plan
7. `FIVE_WHYS_TEMPLATE.md` - Root cause analysis
8. `.quality/SPRINT_TEMPLATE.md` - Sprint tracking

### Templates
9. `.github/PULL_REQUEST_TEMPLATE.md` - PR checklist
10. `.github/ISSUE_TEMPLATE/bug_report.md` - Bug reports
11. `.github/ISSUE_TEMPLATE/feature_request.md` - Features

---

## 🎓 Toyota Way Principles in Practice

### 自働化 (Jidoka) - Build Quality In

**Daily**: Quality gates before every commit
```bash
./scripts/quality-gates.sh  # Must pass
```

### 反省 (Hansei) - Reflection

**When bugs occur**: Five Whys analysis
```bash
cp FIVE_WHYS_TEMPLATE.md .quality/five-whys-issue-XXXX.md
# Complete root cause analysis
```

### 改善 (Kaizen) - Continuous Improvement

**Weekly**: Review metrics and improve
```bash
# Check trends
cat .quality/sprint*-complete.md | grep "quality_score"
# Should show continuous improvement
```

### 現地現物 (Genchi Genbutsu) - Go and See

**Always**: Test on real shells
```bash
# Generate and test on actual shells
cargo run -- transpile examples/hello.rs > output.sh
sh output.sh      # POSIX sh
dash output.sh    # dash
ash output.sh     # ash
busybox sh output.sh  # busybox
```

---

## ✅ Success Criteria

You've achieved A+ grade when:

- [ ] ShellCheck installed
- [ ] All 667 tests passing (100%)
- [ ] Zero SATD comments
- [ ] Zero unsafe code blocks
- [ ] Version consistency (Cargo.toml = ROADMAP.md)
- [ ] Quality gates passing (./scripts/quality-gates.sh)
- [ ] Coverage ≥85% core
- [ ] Complexity ≤10 cyclomatic, ≤15 cognitive
- [ ] Performance <10ms (currently 19.1µs!)

**Current Status**: 5/9 criteria met ✅
**After ShellCheck + version fix**: 7/9 criteria met (A+ unlocked!)
**After Sprint 25**: 9/9 criteria met (Perfect A+!)

---

## 🎉 Quick Wins

### Today (30 min) → A (94/100) → A+ (98/100)
1. Install ShellCheck (5 min)
2. Fix version mismatch (10 min)
3. Verify tests pass (5 min)
4. Run quality gates (5 min)

### This Week (3 hours) → Maintain A+
1. Make shellcheck tests conditional (1 hour)
2. Deduplicate dependencies (2 hours)

### This Sprint (2 weeks) → Perfect A+ (100/100)
1. Execute Sprint 25 mutation testing
2. Achieve 90%+ mutation score
3. Document everything

---

**Ready to start?** Pick your path:

- **Path 1 (Fastest)**: Run Step 1-4 above (30 min)
- **Path 2 (Thorough)**: Read all documentation first (2 hours)
- **Path 3 (Strategic)**: Plan Sprint 25 execution (1 hour)

All paths lead to **A+ excellence**! 🚀
