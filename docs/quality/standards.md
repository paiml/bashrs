# Bashrs Quality Standards

**Project**: Bashrs (Rash) - Rust-to-Shell Transpiler
**Methodology**: EXTREME TDD with Zero Tolerance Quality Gates
**Inspired by**: paiml-mcp-agent-toolkit quality enforcement

## Overview

Bashrs enforces extreme quality standards through automated gates and zero-tolerance policies. As a safety-critical transpiler generating shell code, quality is not optional—it's mandatory.

## Critical Invariants

These invariants MUST be maintained at all times:

1. **POSIX Compliance**: Every generated script must pass `shellcheck -s sh`
2. **Determinism**: Same Rust input must produce byte-identical shell output
3. **Safety**: No user input can escape proper quoting in generated scripts
4. **Performance**: Generated install.sh must execute in <100ms for minimal scripts
5. **Code Size**: Runtime overhead should not exceed 20 lines of shell boilerplate

## Zero SATD Policy

### Definition

Self-Admitted Technical Debt (SATD) includes any comment containing:
- `TODO`, `FIXME`, `HACK`, `XXX`, `BUG`
- `KLUDGE`, `REFACTOR`, `DEPRECATED`, `WORKAROUND`
- Phrases like "temporary", "for now", "quick fix"

### Enforcement

Quality gates automatically detect and reject SATD:

```bash
# Pre-commit hook blocks SATD
make lint  # Will fail if SATD detected
```

### Exceptions

**Zero exceptions**. All SATD must be:
1. Converted to GitHub issues with proper tracking
2. Removed from code immediately
3. Added to roadmap.yaml if it's a planned feature
4. Documented in ROADMAP.md if it's a known limitation

### Rationale

In a transpiler that generates shell code, "temporary" solutions can become permanent security vulnerabilities. We maintain zero tolerance to prevent technical debt from accumulating.

## Complexity Limits

### Thresholds

| Metric | Maximum | Enforcement | Rationale |
|--------|---------|-------------|-----------|
| Cyclomatic Complexity | 10 | Hard fail | Simple functions are testable functions |
| Cognitive Complexity | 15 | Hard fail | Humans must understand the code |
| Function Length | 80 lines | Warning | Short functions are focused functions |
| File Length | 800 lines | Warning | Modular organization |
| Nesting Depth | 4 levels | Hard fail | Deep nesting indicates design issues |

### Current Status (v0.9.2)

- **Median cyclomatic**: 1.0 (excellent)
- **Median cognitive**: 0.0 (excellent)
- **Top function complexity**: 15 (within limits)
- **Achievement**: 96% complexity reduction in Sprint 7

### Measurement

We use pmat for complexity analysis:

```bash
# Analyze complexity
make complexity

# Detailed report
pmat analyze complexity src/ --max-cyclomatic 10
```

### Refactoring

When complexity exceeds limits, apply EXTREME TDD:

1. **RED**: Write failing tests for current behavior
2. **REFACTOR**: Extract helper functions
3. **GREEN**: Ensure all tests still pass
4. **VERIFY**: Run complexity analysis

Example from Sprint 7 (TICKET-4001):
```
convert_stmt: cognitive 61 → 1 (97% reduction)
```

## Documentation Requirements

### Mandatory Documentation

Every **public** item requires:

1. **Summary**: One-line description
2. **Details**: Extended explanation if needed
3. **Examples**: At least one usage example (as doctest)
4. **Panics**: Conditions that cause panics
5. **Errors**: Possible error returns
6. **Safety**: For any unsafe code (we have zero, but if added)

### Coverage Target

- **Minimum**: 75% documentation coverage
- **Public API**: 100% documentation coverage
- **Current**: See coverage reports in .quality/

### Doctest Requirements

All examples must be executable:

```rust
/// Transpiles Rust code to POSIX shell
///
/// # Example
/// ```
/// use rash::transpile;
///
/// let rust_code = r#"fn main() { println!("Hello"); }"#;
/// let shell_code = transpile(rust_code).unwrap();
/// assert!(shell_code.contains("echo"));
/// ```
pub fn transpile(code: &str) -> Result<String> {
    // ...
}
```

Run doctests:
```bash
make test-doc
```

## Test Coverage

### Requirements

| Category | Minimum | Current | Status |
|----------|---------|---------|--------|
| Core modules | 85% | 85.36% | ✅ |
| Total project | 80% | 82.18% | ✅ |
| Function coverage | 85% | 88.65% | ✅ |
| Region coverage | 85% | 86.88% | ✅ |

### Test Types

1. **Unit tests**: 599 tests, 100% pass rate
2. **Property tests**: 52 properties (~26,000+ cases)
3. **Integration tests**: 19 tests
4. **ShellCheck tests**: 24 validation tests
5. **Determinism tests**: 11 idempotence tests
6. **Unicode tests**: 11 tests
7. **Mutation tests**: 8 targeted tests (Sprint 24)

### Running Tests

```bash
# Fast test suite (<3 minutes)
make test-fast

# Comprehensive suite
make test-all

# Coverage report
make coverage
```

## Property-Based Testing

### Requirements

- **Minimum properties**: 50 (current: 52)
- **Cases per property**: 500
- **Total test cases**: 26,000+
- **Timeout per property**: 30 seconds

### Property Categories

1. **Determinism**: Same input → same output
2. **Idempotence**: Multiple runs → same result
3. **Shell safety**: No injection vulnerabilities
4. **POSIX compliance**: Generated code is valid shell
5. **Semantic equivalence**: Rust behavior preserved

### Example Property Test

```rust
proptest! {
    #[test]
    fn prop_deterministic_output(code in ".*") {
        let output1 = transpile(&code);
        let output2 = transpile(&code);
        prop_assert_eq!(output1, output2);
    }
}
```

## Mutation Testing

### Requirements

- **Target kill rate**: ≥90%
- **Current baseline**: ~83% (IR module, Sprint 24)
- **Modules to test**: parser, IR, emitter, verifier

### Strategy

Following Five Whys analysis (see FIVE_WHYS_ANALYSIS.md if created):

1. Run mutation tests per module (not all tests)
2. Use smart test filtering
3. Focus on critical paths first
4. Document surviving mutants

### Running Mutation Tests

```bash
# Run on specific module
make mutants TARGET=src/parser/

# Full suite (expensive, run manually)
make mutants-full
```

## Security Requirements

### Zero Unsafe Code

- **Current unsafe blocks**: 0
- **Maximum allowed**: 0
- **Enforcement**: Quality gate fails if unsafe detected

### Shell Injection Prevention

All user input must be properly escaped:

```rust
// ✅ CORRECT: Proper escaping
let escaped = shell_escape::escape(Cow::from(user_input));
emit!("echo {}", escaped);

// ❌ WRONG: Direct interpolation
emit!("echo {}", user_input);  // FAILS security check
```

### Dependency Auditing

```bash
# Run security audit
make audit

# Check for supply chain attacks
cargo deny check
```

## Performance Requirements

### Benchmarks

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Simple transpile | <10ms | 19.1µs | ✅ (523x better) |
| Complex transpile | <100ms | ~50µs | ✅ |
| Memory usage | <100MB | ~10MB | ✅ |
| Output overhead | <20 lines | ~15 lines | ✅ |

### Running Benchmarks

```bash
# Run all benchmarks
make bench

# Specific benchmark
cargo bench --bench transpile_performance
```

## ShellCheck Validation

### Requirements

- **Severity**: error level
- **Shell**: POSIX sh (`-s sh`)
- **Pass rate**: 100%
- **Test count**: 24 validation tests

### Running ShellCheck

```bash
# Validate all examples
make test-shellcheck

# Validate specific file
shellcheck -s sh output.sh
```

## Determinism Verification

### Requirements

- **Test iterations**: 10 per test
- **Byte-identical**: Yes
- **Test count**: 11 idempotence tests

### Running Determinism Tests

```bash
# Verify determinism
make test-determinism
```

## Quality Gate Integration

### Pre-commit Hooks

Automatically enforced on every commit:

```bash
# Install hooks
.git/hooks/pre-commit

# Manual check
make validate
```

Blocks commits with:
- SATD comments
- High complexity
- Lint errors
- Test failures
- Formatting issues

### CI/CD Pipeline

GitHub Actions runs:

```yaml
- format check (rustfmt)
- lint (clippy -D warnings)
- test (all test types)
- coverage (cargo llvm-cov)
- shellcheck validation
- determinism verification
- performance benchmarks
```

### Quality Scoring

Based on weighted metrics:

```toml
[scoring]
complexity_weight = 0.30    # Critical for maintainability
coverage_weight = 0.15      # Comprehensive testing
satd_weight = 0.25          # Zero tolerance
dead_code_weight = 0.15     # No unused code
documentation_weight = 0.05 # Focus on critical APIs
performance_weight = 0.10   # Must be fast
```

**Minimum passing score**: 90/100
**Current score**: 98/100 (A+)

## Toyota Way Principles

### Jidoka (自働化) - Build Quality In

- Zero defects policy
- EXTREME TDD methodology
- Quality gates enforced automatically

**Enforcement**: Pre-commit hooks + CI/CD gates

### Hansei (反省) - Reflection

- Five Whys analysis for every bug
- Sprint retrospectives
- Root cause documentation

**Enforcement**: Required for all P0/P1 issues

### Kaizen (改善) - Continuous Improvement

- Weekly quality reviews
- Complexity reduction sprints
- Performance optimization

**Enforcement**: Metrics tracked per sprint

### Genchi Genbutsu (現地現物) - Go and See

- Dogfooding on real projects
- Test on actual shell interpreters (sh, dash, ash, busybox)
- Profile real workloads

**Enforcement**: Required before major releases

## Escalation

Quality violations block:

1. **Local commits**: Pre-commit hook rejects
2. **CI builds**: Quality gate fails
3. **Releases**: Checklist verification fails

### Override Process

No overrides without:

1. Architecture review and approval
2. Documented exception with rationale
3. Remediation plan with timeline
4. Additional monitoring/testing

## Continuous Improvement

### Metrics Tracking

Track quality trends per sprint:

- Complexity metrics
- Coverage improvements
- Test count growth
- Performance benchmarks
- Defect density

### Sprint Reviews

Weekly quality reviews:

```bash
# Generate quality report
make quality-report

# Identify hotspots
pmat analyze lint-hotspot --top-files 5
```

## Related Documentation

- [ROADMAP.yaml](../../ROADMAP.yaml) - Sprint planning and achievements
- [CLAUDE.md](../../CLAUDE.md) - Development guidelines and Toyota Way
- [pmat-quality.toml](../../pmat-quality.toml) - Quality gate configuration
- [.pmat-gates.toml](../../.pmat-gates.toml) - Gate enforcement settings

## Summary

Bashrs maintains EXTREME quality standards because:

1. **Safety-critical**: Generates executable shell code
2. **Security-sensitive**: Must prevent injection attacks
3. **Correctness-critical**: Must preserve Rust semantics
4. **Production-ready**: Used in real bootstrap installers

Quality is not optional—it's the foundation of everything we build.
