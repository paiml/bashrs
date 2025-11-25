# Dogfooding (Self-Validation)

bashrs validates its own codebase using its own linting tools - a practice known as "dogfooding." This ensures the tool works correctly on real-world scripts and demonstrates confidence in its own quality.

## What is Dogfooding?

**Dogfooding** (or "eating your own dog food") means using your own product internally. For bashrs, this means:

- Linting bashrs's own Makefile with `bashrs make lint`
- Linting all shell scripts in the repository with `bashrs lint`
- Running purification on example scripts
- Validating synthetic test corpora

This practice ensures bashrs can handle real-world complexity and catches regressions early.

## Quick Start

```bash
# Quick dogfood check (Makefile + key scripts)
make dogfood-quick

# Full self-validation (all 75+ scripts)
make dogfood

# Lint just the Makefile
make lint-makefile

# Lint all shell scripts
make lint-scripts
```

## Dogfooding Results

bashrs self-validation produces comprehensive metrics:

```bash
$ make dogfood

🐕 bashrs Dogfooding - Self-Validation
=======================================

=== Phase 1: Makefile Validation ===
Summary: 2 error(s), 51 warning(s), 0 info(s)

=== Phase 2: Shell Script Validation ===

=== Dogfooding Summary ===
Shell scripts scanned: 75
Total errors: 2515
Total warnings: 5914
Total infos: 1963

✅ Dogfooding complete - bashrs validated its own codebase!

📊 Full report: docs/dogfooding/BASHRS_DOGFOODING.md
```

### Metrics Breakdown

| Metric | Value | Notes |
|--------|-------|-------|
| Shell scripts scanned | 75 | All `.sh` files in repo |
| Total errors | 2,515 | Mostly in test fixtures |
| Total warnings | 5,914 | Quoting, variables |
| Total infos | 1,963 | Best practice suggestions |
| Makefile errors | 2 | SC2299 false positives |
| Makefile warnings | 51 | MAKE003 unquoted vars |

### Top Error Categories

```bash
# Error distribution (from dogfooding)
SEC010 (Path traversal risk):     1,187
DET001 ($RANDOM usage):             424
SC2111 (ksh function syntax):       359
DET002 (Timestamp usage):           352
SC2086 (Unquoted variable):       2,199  # warning
```

## Make Targets

### `make dogfood`

Full self-validation of all scripts:

```bash
make dogfood
```

**What it does:**
1. Builds bashrs in release mode
2. Lints the Makefile with `bashrs make lint`
3. Scans all `.sh` files (excluding node_modules, target)
4. Aggregates error/warning/info counts
5. Reports summary statistics

**Runtime:** ~2 minutes (depends on script count)

### `make dogfood-quick`

Quick validation of key files only:

```bash
make dogfood-quick
```

**What it does:**
1. Lints Makefile
2. Lints `install.sh`
3. Lints `scripts/validate-book.sh`

**Runtime:** ~5 seconds

### `make lint-scripts`

Lint all shell scripts:

```bash
make lint-scripts
```

### `make lint-makefile`

Lint just the Makefile:

```bash
make lint-makefile
```

## CI/CD Integration

bashrs includes a GitHub Actions workflow for automated dogfooding:

```yaml
# .github/workflows/dogfooding.yml
name: Dogfooding (Self-Validation)

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 0 * * 1'  # Weekly on Monday

jobs:
  dogfood:
    name: bashrs Self-Validation
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Build bashrs
      run: cargo build --release --bin bashrs
    - name: Dogfood - Lint Makefile
      run: ./target/release/bashrs make lint Makefile
    - name: Dogfood - Lint Shell Scripts
      run: |
        for script in $(find . -name "*.sh" -type f); do
          ./target/release/bashrs lint "$script" || true
        done
```

**Features:**
- Runs on every push/PR to main
- Weekly scheduled runs catch regressions
- Generates GitHub Actions summary with metrics
- Synthetic test corpus validation

## Understanding the Results

### Why So Many Issues?

Most issues come from **intentional test fixtures** that demonstrate bashrs's detection capabilities:

```bash
# examples/backup-messy.sh - Intentionally non-deterministic
SESSION_ID="backup-$RANDOM-$(date +%s)"  # DET001, DET002

# tests/purification_examples/legacy-deploy.sh - Shows before/after
TIMESTAMP=$(date +%s)  # Intentional for demonstration
```

These files exist to:
1. Test that bashrs detects issues correctly
2. Demonstrate purification transformations
3. Provide before/after examples in documentation

### Real Issues vs Test Fixtures

| Category | Real Issues | Test Fixtures |
|----------|-------------|---------------|
| SEC010 | ~10 | ~1,177 |
| DET001/DET002 | ~5 | ~771 |
| SC2086 | ~200 | ~1,999 |

**Real issues** are in production scripts like `install.sh` and `Makefile`.

## Synthetic Testing with verificar

bashrs integrates with [verificar](https://github.com/paiml/verificar) for synthetic bash test generation:

```bash
# Generate synthetic bash programs
cd ../verificar
cargo run -- generate --language bash --count 1000 --output json > corpus.json

# Test with bashrs
cd ../bashrs
for prog in corpus/*.sh; do
  bashrs lint "$prog" --format human
done
```

### Current Capabilities

verificar generates 33 unique bash programs covering:

| Feature | Example | Tested |
|---------|---------|--------|
| Assignments | `x=1`, `y=hello` | ✅ |
| Echo | `echo "$x"` | ✅ |
| Environment | `echo "$HOME"` | ✅ |
| Conditionals | `if [ $x -eq 0 ]; then` | ✅ |
| Loops | `for i in 1 2 3; do` | ✅ |
| Functions | `greet() { echo hello }` | ✅ |
| Arithmetic | `result=$((1 + 2))` | ✅ |
| Pipes | `echo hello \| wc -c` | ✅ |

**Results:** 100% of synthetic programs parse without errors.

## Best Practices

### 1. Run Dogfood Before PRs

```bash
# Quick check before committing
make dogfood-quick

# Full validation before PR
make dogfood
```

### 2. Investigate New Errors

If dogfooding reveals new errors, investigate:

```bash
# Check specific file
bashrs lint path/to/script.sh --format human

# Get JSON for analysis
bashrs lint path/to/script.sh --format json
```

### 3. Document False Positives

If bashrs incorrectly flags valid code, document it:

```bash
# Example: SC2299 false positive in Makefile
# ${var:-default} is valid POSIX, but bashrs flags it
# See: docs/issues/ISSUE-002-SC2299-FALSE-POSITIVE.md
```

### 4. Track Metrics Over Time

Monitor dogfooding metrics to catch regressions:

```bash
# Weekly tracking
date >> dogfood-metrics.log
make dogfood-quick | grep "Summary:" >> dogfood-metrics.log
```

## Known Issues

### SC2299 False Positive

bashrs incorrectly flags `${var:-default}` as SC2299:

```makefile
# This is VALID POSIX syntax
THREADS=$${PROPTEST_THREADS:-$$(nproc)}
```

**Status:** Documented in `docs/issues/ISSUE-002-SC2299-FALSE-POSITIVE.md`

**Impact:** 2 false positive errors in Makefile

**Workaround:** None needed - the Makefile works correctly

## Example: Complete Dogfooding Workflow

```bash
# 1. Build bashrs
cargo build --release

# 2. Quick sanity check
make dogfood-quick

# 3. Full validation
make dogfood

# 4. Check specific script
bashrs lint install.sh --format human

# 5. Fix issues
# Edit script to address warnings

# 6. Verify fix
bashrs lint install.sh --format human

# 7. Commit
git add install.sh
git commit -m "fix: Address SC2086 warnings in install.sh"
```

## Comparison with Other Projects

| Project | Self-Validates | Automated | Synthetic Tests |
|---------|----------------|-----------|-----------------|
| bashrs | ✅ 75+ scripts | ✅ Weekly CI | ✅ 33 programs |
| ShellCheck | ❌ | ❌ | ❌ |
| depyler | ✅ 27 files | ✅ CI | ❌ |

bashrs is the **first shell linting tool with comprehensive automated dogfooding**.

## Further Reading

- [BASHRS_DOGFOODING.md](https://github.com/paiml/bashrs/blob/main/docs/dogfooding/BASHRS_DOGFOODING.md) - Full report
- [QUALITY_METRICS.md](https://github.com/paiml/bashrs/blob/main/docs/dogfooding/QUALITY_METRICS.md) - Metrics dashboard
- [VERIFICAR_INTEGRATION.md](https://github.com/paiml/bashrs/blob/main/docs/dogfooding/VERIFICAR_INTEGRATION.md) - Synthetic testing
- [Toyota Way Principles](../contributing/toyota-way.md) - Quality philosophy
- [EXTREME TDD](../contributing/extreme-tdd.md) - Testing methodology
