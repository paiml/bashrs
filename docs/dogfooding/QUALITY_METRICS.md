# bashrs Quality Metrics Dashboard

**Generated**: 2025-11-25
**Version**: v6.39.0
**Methodology**: Automated collection + manual verification

## Executive Summary

bashrs maintains high quality standards through comprehensive testing, extensive linter coverage, and continuous self-validation (dogfooding).

## Test Corpus Metrics

### Unit & Integration Tests

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Total tests | 7,708 | >5,000 | ✅ Exceeds |
| Pass rate | 100% | 100% | ✅ Met |
| Property tests | 500 cases/property | >100 | ✅ Exceeds |
| Test execution | <5 min (fast) | <5 min | ✅ Met |

### Code Coverage

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Line coverage | 91.22% | >85% | ✅ Exceeds |
| Function coverage | 95.85% | >90% | ✅ Exceeds |
| Region coverage | 90.66% | >85% | ✅ Exceeds |

### Synthetic Test Corpus (verificar)

| Metric | Value | Notes |
|--------|-------|-------|
| Programs generated | 33 | Exhaustive strategy |
| Parse success rate | 100% | All programs lint without parse errors |
| Feature coverage | 8 types | assignments, echo, if, for, functions, arithmetic, pipes, env vars |

## Codebase Metrics

### Size & Complexity

| Metric | Value |
|--------|-------|
| Lines of Rust code | 199,845 |
| Binary size (release) | 4.6 MB |
| Direct dependencies | 45 |
| Workspace crates | 3 |

### Linter Coverage

| Metric | Value |
|--------|-------|
| Linter rule files | 381 |
| ShellCheck rules mapped | 325 |
| bashrs-specific rules | 56+ |
| Rule categories | 6 (DET, IDEM, SEC, SC, MAKE, CONFIG) |

## Dogfooding Metrics

### Self-Validation Results

| Target | Files | Errors | Warnings | Infos |
|--------|-------|--------|----------|-------|
| Shell scripts | 75 | 2,515 | 5,914 | 1,963 |
| Makefile | 1 | 2 | 51 | 0 |
| **Total** | **76** | **2,517** | **5,965** | **1,963** |

### Error Distribution (Top 10)

| Code | Description | Count | Category |
|------|-------------|-------|----------|
| SEC010 | Path traversal risk | 1,187 | Security |
| SC2086 | Unquoted variable | 2,199 | Quoting |
| SC2154 | Unassigned variable | 833 | Variables |
| SC2046 | Word splitting | 654 | Quoting |
| DET001 | $RANDOM usage | 424 | Determinism |
| SC2111 | ksh function syntax | 359 | Portability |
| DET002 | Timestamp usage | 352 | Determinism |
| IDEM001 | Non-idempotent op | 308 | Idempotency |
| SC2155 | Declare/assign split | 220 | Best practice |
| IDEM002 | File operation | 148 | Idempotency |

### Comparison with Industry

| Metric | bashrs | depyler | Industry Avg |
|--------|--------|---------|--------------|
| Files validated | 76 | 27 | ~20 |
| Issues documented | 10,445 | 5 | ~50 |
| Test count | 7,708 | ~1,000 | ~500 |
| Coverage | 91% | ~80% | ~70% |
| CI/CD dogfooding | Yes | Yes | Rare |

## Quality Gates

### Pre-commit Checks

| Gate | Status | Enforcement |
|------|--------|-------------|
| Clippy (zero warnings) | ✅ | Blocking |
| Performance lints | ✅ | Blocking |
| Test suite (fast) | ✅ | Blocking |
| Code complexity (<10) | ✅ | Blocking |
| Technical debt | ⚠️ | Warning |
| Formatting | ✅ | Blocking |
| Documentation sync | ✅ | Blocking |
| Book examples | ✅ | Blocking |
| Matrix test | ✅ | Blocking |

### CI/CD Validation

| Workflow | Trigger | Duration |
|----------|---------|----------|
| CI (main) | Push/PR | ~5 min |
| Coverage | Push/PR | ~10 min |
| Dogfooding | Push/PR + Weekly | ~3 min |
| Book validation | Push/PR | ~2 min |

## Trend Analysis

### Quality Score Evolution

```
v6.35.0: 118.0/134 (88.1%) - Grade A+
v6.38.0: 127.0/134 (94.8%) - Grade A+
v6.39.0: 127.0/134 (94.8%) - Grade A+ (maintained)
```

### Test Count Growth

```
v6.30.0: ~5,000 tests
v6.35.0: ~6,500 tests
v6.39.0: 7,708 tests (+54% from v6.30.0)
```

## Recommendations

### Short-term (Next Sprint)

1. **Fix SC2299 false positive** - Documented in ISSUE-002
2. **Reduce SEC010 warnings** - Add path validation to examples
3. **Improve DET001 detection** - Better handling of $RANDOM in tests

### Medium-term (Next Quarter)

1. **Expand verificar corpus** - Target 10,000 synthetic programs
2. **Mutation testing baseline** - Establish >80% kill rate
3. **Performance benchmarks** - Add renacer golden traces

### Long-term (Roadmap)

1. **100% linter rule coverage** - Map all 800+ ShellCheck rules
2. **Zero warnings in dogfood** - Fix all self-detected issues
3. **Automated purification** - Self-apply purification to scripts

## Reproducibility

All metrics can be regenerated:

```bash
# Test count
cargo test --workspace 2>&1 | grep "test result"

# Coverage
make coverage

# Dogfooding
make dogfood

# Synthetic tests
cd ../verificar && cargo run -- generate --language bash --count 10000 --output json
```

---

**Dashboard maintained by**: bashrs CI/CD pipeline
**Last updated**: 2025-11-25
**Next scheduled update**: Weekly (Monday 00:00 UTC)
