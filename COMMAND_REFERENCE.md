# Bashrs Command Reference Card

Quick reference for daily development with EXTREME TDD methodology.

## 🚀 Quick Setup (First Time - 30 min)

```bash
# 1. Install ShellCheck (5 min)
sudo apt-get install shellcheck     # Ubuntu/Debian
# or
brew install shellcheck              # macOS

# 2. Verify tests pass (5 min)
cd /home/noahgift/src/bashrs
cargo test --lib
# Expected: 667/667 passing

# 3. Run quality gates (5 min)
./scripts/quality-gates.sh
# All 9 gates should PASS

# 4. You're ready! Grade: A+ (98/100) ✅
```

---

## 📋 Daily Development Workflow

### Before Starting Work
```bash
# Pull latest changes
git pull

# Check current status
git status
cargo test --lib --quiet
```

### RED Phase (Write Failing Tests)
```bash
# Write tests FIRST in appropriate test file
# Tests should FAIL initially

cargo test test_your_feature_name
# Expected: FAILED (RED) ❌
```

### GREEN Phase (Minimal Implementation)
```bash
# Write minimal code to pass tests

cargo test test_your_feature_name
# Expected: ok. 1 passed (GREEN) ✅
```

### REFACTOR Phase (Clean Up)
```bash
# Refactor for quality

# Format code
cargo fmt

# Fix lint issues
cargo clippy --fix

# Re-run tests (must still pass)
cargo test --lib
```

### Before Committing
```bash
# Run all quality gates (MANDATORY)
./scripts/quality-gates.sh

# If all pass, commit
git add .
git commit -m "feat: your feature description

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## 🧪 Testing Commands

### Run All Tests
```bash
# Full test suite
cargo test --lib

# Fast tests only
cargo test --lib --quiet

# With detailed output
cargo test --lib -- --nocapture

# Specific test
cargo test test_name
```

### Property Tests
```bash
# Run all property tests
cargo test --test property_tests

# With specific case count
PROPTEST_CASES=1000 cargo test --test property_tests
```

### ShellCheck Validation
```bash
# Run shellcheck tests
cargo test test_shellcheck

# Manual shellcheck
cargo run -- transpile examples/hello.rs > /tmp/output.sh
shellcheck -s sh /tmp/output.sh
```

### Determinism Tests
```bash
# Run determinism verification
cargo test test_deterministic

# Manual check
cargo run -- transpile examples/hello.rs > /tmp/out1.sh
cargo run -- transpile examples/hello.rs > /tmp/out2.sh
diff /tmp/out1.sh /tmp/out2.sh  # Should be identical
```

---

## 📊 Quality Checks

### Format
```bash
# Check formatting
cargo fmt -- --check

# Auto-format
cargo fmt
```

### Lint
```bash
# Check with clippy
cargo clippy --all-targets --all-features

# With warnings as errors
cargo clippy --all-targets --all-features -- -D warnings

# Auto-fix
cargo clippy --fix
```

### Coverage
```bash
# Generate HTML coverage report
cargo llvm-cov --html --open

# Generate LCOV for CI
cargo llvm-cov --lcov --output-path coverage.info

# Summary only
cargo llvm-cov --summary-only
```

### Complexity
```bash
# If pmat installed
pmat analyze complexity src/

# Detailed report
pmat analyze complexity src/ --detailed

# Specific file
pmat analyze complexity src/parser/mod.rs
```

### SATD Check
```bash
# Check for technical debt comments
grep -r "TODO\|FIXME\|HACK\|XXX" src/ --include="*.rs"

# Count (should be 0)
grep -r "TODO\|FIXME\|HACK\|XXX" src/ --include="*.rs" | wc -l
```

### Unsafe Code Check
```bash
# Find unsafe blocks
grep -r "unsafe" src/ --include="*.rs" | grep -v "//"

# Count (should be 0)
grep -r "unsafe" src/ --include="*.rs" | grep -v "//" | wc -l
```

---

## 🎯 Quality Gates

### Run All Gates
```bash
# Automated 9-gate check
./scripts/quality-gates.sh

# Expected output:
# ✓ Format check passed
# ✓ Lint check passed
# ✓ Test suite passed
# ✓ Coverage check passed
# ✓ Complexity check passed
# ✓ SATD check passed
# ✓ ShellCheck validation passed
# ✓ Determinism check passed
# ✓ Performance check passed
# ✓ All quality gates passed! ✓
```

### Individual Gates
```bash
# 1. Format
cargo fmt -- --check

# 2. Lint
cargo clippy --all-targets --all-features -- -D warnings

# 3. Tests
cargo test --lib

# 4. Coverage
cargo llvm-cov --summary-only

# 5. Complexity (if pmat installed)
pmat analyze complexity src/ --max-cyclomatic 10

# 6. SATD
grep -r "TODO\|FIXME\|HACK" src/ --include="*.rs" | wc -l

# 7. ShellCheck
cargo run -- transpile examples/hello.rs | shellcheck -s sh -

# 8. Determinism
# (use determinism test commands above)

# 9. Performance
cargo bench
```

---

## 🏗️ Build Commands

### Development Build
```bash
# Debug build
cargo build

# Fast debug build
cargo build --profile dev-fast
```

### Release Build
```bash
# Optimized release
cargo build --release

# Minimal size build
cargo build --profile min-size

# Check binary size
ls -lh target/release/rash
# or
ls -lh target/min-size/rash
```

### Clean Build
```bash
# Clean all artifacts
cargo clean

# Full rebuild
cargo clean && cargo build --release
```

---

## 📦 Dependencies

### Update
```bash
# Update dependencies
cargo update

# Check outdated
cargo outdated

# Tree view
cargo tree
```

### Audit
```bash
# Security audit
cargo audit

# Check licenses/bans
cargo deny check

# Find duplicates
cargo tree --duplicates
```

### Add/Remove
```bash
# Add dependency
cargo add dependency-name

# Add dev dependency
cargo add --dev dev-dependency-name

# Remove
cargo rm dependency-name
```

---

## 🔍 Analysis Commands

### Code Statistics
```bash
# If tokei installed
tokei src/

# Line count
find src/ -name "*.rs" -exec wc -l {} + | tail -1
```

### Benchmark
```bash
# Run all benchmarks
cargo bench

# Specific benchmark
cargo bench transpile

# Save baseline
cargo bench -- --save-baseline main

# Compare to baseline
cargo bench -- --baseline main
```

### Documentation
```bash
# Generate docs
cargo doc --no-deps --open

# Doc tests only
cargo test --doc
```

---

## 🐛 Debugging

### Verbose Output
```bash
# Verbose build
cargo build -vv

# Verbose test
cargo test -- --nocapture --test-threads=1

# Show stdout
cargo test -- --show-output
```

### Environment Variables
```bash
# Enable backtrace
RUST_BACKTRACE=1 cargo test

# Full backtrace
RUST_BACKTRACE=full cargo test

# Logging
RUST_LOG=debug cargo run
```

---

## 📝 Documentation Workflow

### Create Bug Report
```bash
# Copy template
cp .github/ISSUE_TEMPLATE/bug_report.md issue-XXXX.md

# Fill in:
# - Problem statement
# - Reproduction steps
# - Five Whys analysis
# - Environment info
```

### Create Feature Request
```bash
# Copy template
cp .github/ISSUE_TEMPLATE/feature_request.md feature-XXXX.md

# Fill in:
# - User story
# - Technical spec
# - EXTREME TDD plan
# - Acceptance criteria
```

### Start Sprint
```bash
# Copy sprint template
cp .quality/SPRINT_TEMPLATE.md .quality/sprint25-in-progress.md

# Fill in:
# - Sprint goals
# - Tickets
# - Baseline metrics
```

### Five Whys Analysis
```bash
# Copy template
cp FIVE_WHYS_TEMPLATE.md .quality/five-whys-issue-XXXX.md

# Complete all sections:
# - Five Whys (to root cause)
# - Fix strategy
# - Prevention
```

---

## 🚀 Release Workflow

### Pre-release Checks
```bash
# All quality gates pass
./scripts/quality-gates.sh

# All tests pass
cargo test --lib

# No warnings
cargo clippy --all-targets -- -D warnings

# Version updated
grep "^version" Cargo.toml
```

### Build Release
```bash
# Build optimized binary
cargo build --release

# Check size
ls -lh target/release/rash

# Test binary
target/release/rash --version
target/release/rash transpile examples/hello.rs
```

### Publish
```bash
# Dry run
cargo publish --dry-run

# Actual publish (be careful!)
cargo publish
```

---

## 💡 Pro Tips

### Fast Iteration
```bash
# Watch mode (requires cargo-watch)
cargo watch -x test

# Fast check without tests
cargo check

# Parallel jobs
cargo build -j8
```

### Incremental Compilation
```bash
# Already enabled in .cargo/config.toml
# But you can verify:
echo $CARGO_INCREMENTAL  # Should be 1
```

### Cache Management
```bash
# Clean cargo cache
rm -rf ~/.cargo/registry/cache

# Clean build cache
cargo clean

# Clean coverage cache
rm -rf target/llvm-cov-target
```

---

## 🎓 Learning Resources

### Quick Reference
```bash
# This file
cat COMMAND_REFERENCE.md

# Quick start
cat QUICK_START_GUIDE.md

# Quality standards
cat docs/quality/standards.md
```

### Complete Docs
```bash
# Index
cat INDEX_QUALITY_DOCS.md

# Quality review
cat QUALITY_REVIEW_2025-10-09.md

# Infrastructure
cat EXTREME_QUALITY_IMPLEMENTATION.md
```

### Help Commands
```bash
# Cargo help
cargo --help
cargo test --help
cargo build --help

# Rash help
cargo run -- --help
```

---

## 🆘 Troubleshooting

### Tests Failing?
```bash
# 1. Check ShellCheck installed
which shellcheck

# 2. Run verbose
cargo test -- --nocapture

# 3. Run single test
cargo test failing_test_name -- --nocapture

# 4. Clean and rebuild
cargo clean && cargo test
```

### Quality Gates Failing?
```bash
# 1. Format code
cargo fmt

# 2. Fix lint
cargo clippy --fix

# 3. Check specific gate
./scripts/quality-gates.sh

# 4. Review output for specific failure
```

### Build Errors?
```bash
# 1. Update Rust
rustup update

# 2. Clean build
cargo clean

# 3. Update deps
cargo update

# 4. Check Cargo.toml
cat Cargo.toml
```

---

## 📞 Getting Help

- **Quick Start**: `QUICK_START_GUIDE.md`
- **Quality Issues**: `QUALITY_REVIEW_2025-10-09.md`
- **Standards**: `docs/quality/standards.md`
- **Templates**: `.github/ISSUE_TEMPLATE/`
- **All Docs**: `INDEX_QUALITY_DOCS.md`

---

**Last Updated**: 2025-10-09
**Status**: Complete and tested
**Grade**: A (94/100) → A+ (98/100) after quick fixes
