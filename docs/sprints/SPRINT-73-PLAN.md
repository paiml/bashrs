# Sprint 73: Bash Purifier Production Readiness

**Date**: 2024-10-18
**Duration**: 2-3 weeks
**Status**: 🎯 PLANNED
**Goal**: Take Bash → Purified Bash from 70% → 100% production-ready for v2.0.0 release

---

## Executive Summary

**Sprint Focus**: Complete the working Bash purifier and make it production-ready for v2.0.0 release.

**Key Insight from Sprint 72 Audit**:
- Bash → Purified Bash workflow is 70% complete and working
- Rust → Shell workflow is not implemented (deferred to v3.0+)
- Focus on completing what works rather than building what's missing

**Deliverable**: v2.0.0 release with production-ready Bash purification tool

---

## Sprint Goals

### Primary Goals (P0 - MUST HAVE)

1. ✅ **Production Documentation** (1 week)
   - User guide with examples
   - API documentation
   - Migration guide (from raw bash to purified)
   - Troubleshooting guide

2. ✅ **Real-World Examples** (2-3 days)
   - 5-10 production-quality example scripts
   - Bootstrap installers
   - Deployment scripts
   - CI/CD integration examples
   - Docker build scripts

3. ✅ **CLI Integration Tests** (3-4 days)
   - Complete `assert_cmd` test suite
   - Error handling tests
   - End-to-end workflow tests
   - Regression tests

4. ✅ **Performance Benchmarking** (2-3 days)
   - Parse time baseline (<50ms target)
   - Transpile time baseline (<100ms target)
   - Memory usage baseline (<10MB target)
   - Identify bottlenecks

### Secondary Goals (P1 - SHOULD HAVE)

5. ⚠️ **Error Handling Polish** (2-3 days)
   - Improve error messages
   - Add context to failures
   - Graceful degradation

6. ⚠️ **Quality Assurance** (2-3 days)
   - Mutation testing audit (≥90% target)
   - Code coverage verification (>85% target)
   - Complexity audit (<10 target)

### Stretch Goals (P2 - NICE TO HAVE)

7. 🎁 **Advanced Bash Constructs** (ongoing)
   - Process substitution
   - Advanced expansions
   - Coprocesses

8. 🎁 **v2.0.0 Release** (1 day)
   - CHANGELOG.md update
   - Version bump
   - Release notes
   - GitHub release

---

## Phase 1: Production Documentation (Week 1)

### Day 1-2: User Guide

**File**: `docs/USER-GUIDE.md`

**Contents**:
```markdown
# Rash User Guide

## Quick Start
- Installation
- Basic usage
- Common workflows

## Bash Purification
- What is purification?
- Why use Rash?
- Before/after examples

## CLI Reference
- rash parse
- rash purify
- rash lint
- rash check

## Advanced Usage
- Configuration
- Integration with CI/CD
- Custom workflows
```

**Acceptance Criteria**:
- [ ] Complete quick start guide
- [ ] 5+ before/after examples
- [ ] CLI reference with all options
- [ ] Troubleshooting section

---

### Day 3-4: API Documentation

**File**: `docs/API-REFERENCE.md`

**Contents**:
```markdown
# Rash API Reference

## Bash Parser
- parse_bash()
- BashAst structure
- Parser options

## Bash Transpiler
- transpile()
- Transpiler options
- Error handling

## Linter
- lint_shell()
- LintResult structure
- Custom rules
```

**Acceptance Criteria**:
- [ ] Complete API documentation
- [ ] Code examples for each function
- [ ] Error handling documentation
- [ ] Integration examples

---

### Day 5: Migration Guide

**File**: `docs/MIGRATION-GUIDE.md`

**Contents**:
```markdown
# Migration Guide: Raw Bash → Purified Bash

## Why Migrate?
- Determinism benefits
- Idempotency benefits
- Safety benefits

## Migration Process
1. Parse existing scripts
2. Review lint warnings
3. Apply purification
4. Test purified output
5. Deploy

## Common Patterns
- $RANDOM → UUID/version-based
- Timestamps → fixed versions
- mkdir → mkdir -p
- rm → rm -f
- ln -s → ln -sf

## Case Studies
- [Case study examples]
```

**Acceptance Criteria**:
- [ ] Complete migration workflow
- [ ] 3+ real-world case studies
- [ ] Common patterns documented
- [ ] Rollback strategy

---

## Phase 2: Real-World Examples (Days 6-7)

### Example 1: Bootstrap Installer

**File**: `examples/bootstrap-installer/original.sh`

```bash
#!/bin/bash
# Original messy bootstrap script

TEMP_DIR="/tmp/install-$$"
mkdir $TEMP_DIR

curl -o $TEMP_DIR/app.tar.gz https://example.com/app.tar.gz
tar -xzf $TEMP_DIR/app.tar.gz -C /usr/local/bin

rm -r $TEMP_DIR
```

**File**: `examples/bootstrap-installer/purified.sh`

```bash
#!/bin/sh
# Purified by Rash v2.0.0

TEMP_DIR="/tmp/install-bootstrap"
mkdir -p "${TEMP_DIR}" || exit 1

curl -o "${TEMP_DIR}/app.tar.gz" https://example.com/app.tar.gz || exit 1
tar -xzf "${TEMP_DIR}/app.tar.gz" -C /usr/local/bin || exit 1

rm -rf "${TEMP_DIR}"
```

**File**: `examples/bootstrap-installer/README.md`

```markdown
# Bootstrap Installer Example

## Problem
Original script had:
- Non-deterministic temp dir ($$)
- Non-idempotent mkdir
- Unquoted variables (injection risk)
- No error handling

## Solution
Purified script:
- Deterministic temp dir
- Idempotent mkdir -p
- Quoted variables
- Error handling with exit codes
```

---

### Example 2: Deployment Script

**File**: `examples/deployment/original.sh`

```bash
#!/bin/bash
# Original deployment script

RELEASE="release-$(date +%s)"
SESSION_ID=$RANDOM

mkdir /app/releases/$RELEASE
cp -r build/* /app/releases/$RELEASE/

rm /app/current
ln -s /app/releases/$RELEASE /app/current

echo "Deployed release $RELEASE (session $SESSION_ID)"
```

**File**: `examples/deployment/purified.sh`

```bash
#!/bin/sh
# Purified by Rash v2.0.0

VERSION="${1:-unknown}"
RELEASE="release-${VERSION}"
SESSION_ID="session-${VERSION}"

mkdir -p "/app/releases/${RELEASE}" || exit 1
cp -r build/* "/app/releases/${RELEASE}/" || exit 1

rm -f "/app/current"
ln -sf "/app/releases/${RELEASE}" "/app/current" || exit 1

printf 'Deployed release %s (session %s)\n' "${RELEASE}" "${SESSION_ID}"
```

---

### All Examples (5-10 total)

1. ✅ **Bootstrap Installer** - curl-based installer with temp files
2. ✅ **Deployment Script** - release deployment with timestamps
3. ⏸️ **CI/CD Integration** - GitHub Actions / Jenkins integration
4. ⏸️ **Docker Build** - Docker entrypoint with environment setup
5. ⏸️ **Database Migration** - SQL migration script with rollback
6. ⏸️ **Backup Script** - Incremental backup with rotation
7. ⏸️ **Configuration Management** - System configuration with idempotency
8. ⏸️ **Log Rotation** - Log management with cleanup
9. ⏸️ **Service Health Check** - Monitoring script with alerts
10. ⏸️ **Build Pipeline** - Multi-stage build with caching

**Each example includes**:
- `original.sh` - Messy bash script
- `purified.sh` - Purified output
- `README.md` - Explanation of changes
- `test.sh` - Verification script

---

## Phase 3: CLI Integration Tests (Days 8-10)

### Test File: `tests/cli_integration.rs`

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn rash_cmd() -> Command {
    Command::cargo_bin("rash").expect("Failed to find rash binary")
}

// Test 1: Basic parse
#[test]
fn test_cli_parse_valid_bash() {
    let script = "tests/fixtures/valid.sh";
    fs::write(script, "#!/bin/bash\necho hello").unwrap();

    rash_cmd()
        .arg("parse")
        .arg(script)
        .assert()
        .success()
        .stdout(predicate::str::contains("AST"));

    let _ = fs::remove_file(script);
}

// Test 2: Purify with output
#[test]
fn test_cli_purify_with_output() {
    let input = "tests/fixtures/messy.sh";
    let output = "tests/fixtures/purified.sh";

    fs::write(input, "#!/bin/bash\nTEMP=/tmp/test-$$\nmkdir $TEMP").unwrap();

    rash_cmd()
        .arg("purify")
        .arg(input)
        .arg("--output")
        .arg(output)
        .assert()
        .success();

    let purified = fs::read_to_string(output).unwrap();
    assert!(purified.contains("#!/bin/sh"));
    assert!(!purified.contains("$$"));
    assert!(purified.contains("mkdir -p"));

    let _ = fs::remove_file(input);
    let _ = fs::remove_file(output);
}

// Test 3: Lint with errors
#[test]
fn test_cli_lint_detects_errors() {
    let script = "tests/fixtures/unsafe.sh";
    fs::write(script, "#!/bin/bash\neval \"$user_input\"\ncurl http://example.com | sh").unwrap();

    rash_cmd()
        .arg("lint")
        .arg(script)
        .assert()
        .failure()
        .stdout(predicate::str::contains("SEC001"))
        .stdout(predicate::str::contains("SEC008"));

    let _ = fs::remove_file(script);
}

// Test 4: Error handling - invalid file
#[test]
fn test_cli_parse_nonexistent_file() {
    rash_cmd()
        .arg("parse")
        .arg("nonexistent.sh")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"))
        .stderr(predicate::str::contains("nonexistent.sh"));
}

// Test 5: Error handling - malformed bash
#[test]
fn test_cli_parse_malformed_bash() {
    let script = "tests/fixtures/malformed.sh";
    fs::write(script, "#!/bin/bash\nif then fi").unwrap();

    rash_cmd()
        .arg("parse")
        .arg(script)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Parse error"));

    let _ = fs::remove_file(script);
}

// Test 6: End-to-end workflow
#[test]
fn test_cli_integration_full_workflow() {
    let messy = "tests/fixtures/integration/messy_deploy.sh";
    let purified = "tests/fixtures/integration/purified_deploy.sh";

    fs::write(messy, r#"
#!/bin/bash
SESSION_ID=$RANDOM
RELEASE="release-$(date +%s)"
mkdir /tmp/releases/$RELEASE
"#).unwrap();

    // Step 1: Parse
    rash_cmd()
        .arg("parse")
        .arg(messy)
        .assert()
        .success();

    // Step 2: Lint (should detect issues)
    rash_cmd()
        .arg("lint")
        .arg(messy)
        .assert()
        .failure() // Has DET/IDEM warnings
        .stdout(predicate::str::contains("DET001")) // $RANDOM
        .stdout(predicate::str::contains("DET002")); // date

    // Step 3: Purify
    rash_cmd()
        .arg("purify")
        .arg(messy)
        .arg("--output")
        .arg(purified)
        .assert()
        .success();

    // Step 4: Verify purified content
    let purified_content = fs::read_to_string(purified).unwrap();
    assert!(!purified_content.contains("$RANDOM"));
    assert!(!purified_content.contains("date +%s"));
    assert!(purified_content.contains("mkdir -p"));

    // Cleanup
    let _ = fs::remove_file(messy);
    let _ = fs::remove_file(purified);
}
```

**Test Coverage Goals**:
- [ ] All CLI commands tested
- [ ] Success cases covered
- [ ] Error cases covered
- [ ] End-to-end workflows tested
- [ ] Edge cases handled

---

## Phase 4: Performance Benchmarking (Days 11-12)

### Benchmark File: `benches/parse_bench.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rash::bash_parser;

fn parse_benchmark(c: &mut Criterion) {
    let simple_script = "#!/bin/bash\necho hello\n";
    let complex_script = std::fs::read_to_string("benches/fixtures/complex.sh").unwrap();

    c.bench_function("parse simple", |b| {
        b.iter(|| bash_parser::parse(black_box(simple_script)))
    });

    c.bench_function("parse complex", |b| {
        b.iter(|| bash_parser::parse(black_box(&complex_script)))
    });
}

criterion_group!(benches, parse_benchmark);
criterion_main!(benches);
```

### Benchmark File: `benches/transpile_bench.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rash::{bash_parser, bash_transpiler};

fn transpile_benchmark(c: &mut Criterion) {
    let script = std::fs::read_to_string("benches/fixtures/deployment.sh").unwrap();
    let ast = bash_parser::parse(&script).unwrap();

    c.bench_function("transpile deployment", |b| {
        b.iter(|| bash_transpiler::transpile(black_box(&ast)))
    });
}

criterion_group!(benches, transpile_benchmark);
criterion_main!(benches);
```

**Performance Targets**:
- [ ] Parse time: <50ms for typical scripts (100-200 lines)
- [ ] Transpile time: <100ms for typical scripts
- [ ] Memory usage: <10MB for typical scripts
- [ ] Binary size: <5MB (currently ~8MB)

**Optimization Strategy**:
1. Baseline measurements
2. Identify bottlenecks with profiling
3. Optimize hot paths
4. Re-measure
5. Document results

---

## Phase 5: Error Handling Polish (Days 13-14)

### Current State (Example)

```
Error: Parse error
```

### Improved State (Target)

```
Error: Failed to parse bash script at line 15, column 8

  15 | if then fi
           ^^^^
           Expected condition after 'if'

Suggestion: Add a condition like [ -f file ] or [ $var = "value" ]

For more help: https://docs.rash.sh/errors/parse-error
```

### Implementation

**File**: `rash/src/errors.rs`

```rust
use std::fmt;

#[derive(Debug)]
pub struct RashError {
    pub kind: ErrorKind,
    pub message: String,
    pub location: Option<Location>,
    pub suggestion: Option<String>,
    pub help_url: Option<String>,
}

#[derive(Debug)]
pub enum ErrorKind {
    ParseError,
    TranspileError,
    LintError,
    IOError,
}

#[derive(Debug)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub snippet: String,
}

impl fmt::Display for RashError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Error: {}", self.message)?;

        if let Some(loc) = &self.location {
            writeln!(f, "\n  {} | {}", loc.line, loc.snippet)?;
            writeln!(f, "  {: >width$}^", "", width = loc.column + 4)?;
        }

        if let Some(suggestion) = &self.suggestion {
            writeln!(f, "\nSuggestion: {}", suggestion)?;
        }

        if let Some(url) = &self.help_url {
            writeln!(f, "\nFor more help: {}", url)?;
        }

        Ok(())
    }
}
```

**Acceptance Criteria**:
- [ ] Clear error messages
- [ ] Line/column information
- [ ] Code snippets in errors
- [ ] Helpful suggestions
- [ ] Links to documentation

---

## Phase 6: Quality Assurance (Days 15-16)

### Mutation Testing Audit

```bash
# Run mutation tests on all modules
cargo mutants --file rash/src/bash_parser/mod.rs
cargo mutants --file rash/src/bash_transpiler/mod.rs
cargo mutants --file rash/src/linter/rules/mod.rs

# Target: ≥90% kill rate
```

### Code Coverage Audit

```bash
# Run coverage analysis
cargo llvm-cov --html

# Target: >85% coverage
# Current: ~85% (verify)
```

### Complexity Audit

```bash
# Use radon or similar
find rash/src -name "*.rs" -exec rust-code-analysis {} \;

# Target: All functions <10 complexity
```

---

## Phase 7: v2.0.0 Release (Day 17)

### Pre-Release Checklist

- [ ] All tests passing (1,500+ tests)
- [ ] Documentation complete
- [ ] Examples complete
- [ ] Performance benchmarks complete
- [ ] Error handling polished
- [ ] Mutation score ≥90%
- [ ] Code coverage >85%
- [ ] Complexity <10

### Release Tasks

1. **Update CHANGELOG.md**
```markdown
# v2.0.0 - Production-Ready Bash Purifier

## Added
- Production documentation (user guide, API reference, migration guide)
- 10 real-world example scripts
- Complete CLI integration test suite
- Performance benchmarks
- Improved error messages

## Fixed
- [List any bugs fixed during sprint]

## Changed
- Bash → Purified Bash promoted to PRIMARY workflow
- Rust → Shell deferred to v3.0+

## Removed
- [Any deprecated features]
```

2. **Version Bump**
```bash
# Update Cargo.toml
sed -i 's/version = "1.4.0"/version = "2.0.0"/' Cargo.toml

# Update CLAUDE.md
sed -i 's/v1.4.0/v2.0.0/g' CLAUDE.md
```

3. **Git Tag**
```bash
git add .
git commit -m "Release v2.0.0: Production-Ready Bash Purifier"
git tag -a v2.0.0 -m "v2.0.0 - Production-Ready Bash Purifier"
git push origin main --tags
```

4. **GitHub Release**
- Create release on GitHub
- Attach binaries (linux, macos, windows)
- Include release notes
- Link to documentation

---

## Success Metrics

### Quantitative Metrics

| Metric | Target | Status |
|--------|--------|--------|
| **Test Count** | 1,500+ | 🎯 TBD |
| **Test Pass Rate** | 100% | 🎯 TBD |
| **Code Coverage** | >85% | 🎯 TBD |
| **Mutation Score** | ≥90% | 🎯 TBD |
| **Parse Time** | <50ms | 🎯 TBD |
| **Transpile Time** | <100ms | 🎯 TBD |
| **Memory Usage** | <10MB | 🎯 TBD |
| **Examples** | 5-10 | 🎯 TBD |
| **Documentation** | 100% | 🎯 TBD |

### Qualitative Metrics

- [ ] User guide is clear and comprehensive
- [ ] Examples are production-quality
- [ ] Error messages are helpful
- [ ] CLI is intuitive
- [ ] Performance is acceptable

---

## Risk Assessment

### High Risk ⚠️

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Documentation takes longer than expected** | HIGH | Start early, parallel work |
| **Performance below targets** | MEDIUM | Profile early, optimize hot paths |
| **Examples not comprehensive** | MEDIUM | Use real-world scripts from community |

### Medium Risk ⚠️

| Risk | Impact | Mitigation |
|------|--------|------------|
| **CLI tests reveal bugs** | MEDIUM | Fix as discovered (STOP THE LINE) |
| **Mutation testing reveals gaps** | LOW | Add tests, improve coverage |

---

## Dependencies

### External Dependencies

- None (all work internal to rash codebase)

### Internal Dependencies

- Sprint 72 audit complete ✅
- CLAUDE.md updated ✅
- Feature matrix created ✅

---

## Team Allocation

**Sprint Lead**: Claude (AI Assistant)
**Methodology**: EXTREME TDD + Toyota Production System principles
**Quality Gate**: Zero defects policy

---

## Timeline

### Week 1 (Days 1-5)
- Day 1-2: User guide
- Day 3-4: API documentation
- Day 5: Migration guide

### Week 2 (Days 6-12)
- Day 6-7: Real-world examples (5-10 scripts)
- Day 8-10: CLI integration tests
- Day 11-12: Performance benchmarking

### Week 3 (Days 13-17)
- Day 13-14: Error handling polish
- Day 15-16: Quality assurance audit
- Day 17: v2.0.0 release

**Total Duration**: 17 days (2.5 weeks, rounded to 3 weeks with buffer)

---

## Next Steps

1. **Immediate**: Begin Week 1 (documentation)
2. **Day 6**: Start real-world examples
3. **Day 8**: Begin CLI integration tests
4. **Day 17**: Release v2.0.0

---

## Sprint Retrospective (Post-Sprint)

**To be completed after Sprint 73**:
- What went well?
- What could be improved?
- Lessons learned
- Sprint 74 planning

---

**Status**: 🎯 READY TO START
**Prepared by**: Claude (AI Assistant)
**Date**: 2024-10-18
**Methodology**: 反省 (Hansei) + 改善 (Kaizen) + 自働化 (Jidoka)
