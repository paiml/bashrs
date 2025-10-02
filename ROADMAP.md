# Rash (bashrs) Extreme Quality Roadmap

## ✅ INITIAL STATUS: Quality Baseline Established
**Achievement**: Project compiles and test suite functional!
- ✅ All 449 tests passing (422 lib + 19 integration + 8 doc tests)
- ✅ Build system operational (release binary: 3.7MB)
- ✅ Codebase: 22,445 lines of Rust code
- ✅ Fixed critical naming issues (rash→bashrs transition)
- ✅ Property-based testing framework in place (proptest)
- ✅ Makefile with quality gates infrastructure
- ⚠️ Coverage metrics not yet measured
- ⚠️ Known bugs from CLAUDE.md need addressing

## Current Status: Pre-Sprint | Quality Gate Establishment

### 🎯 Project Goals (Derived from CLAUDE.md)
Rash is a **Rust-to-Shell transpiler** with these critical invariants:
1. **POSIX compliance**: Every generated script must pass `shellcheck -s sh`
2. **Determinism**: Same Rust input must produce byte-identical shell output
3. **Safety**: No user input can escape proper quoting in generated scripts
4. **Performance**: Generated install.sh must execute in <100ms for minimal scripts
5. **Code size**: Runtime overhead should not exceed 20 lines of shell boilerplate

### 📊 Baseline Metrics (Sprint 0)

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| **Test Suite** | 449 passing, 2 ignored | 600+ passing, 0 ignored | 🟡 Good foundation |
| **Coverage** | Unknown | >85% line coverage | 🔴 Must measure |
| **Binary Size** | 3.7MB | <3MB minimal, <6MB full | 🟡 Acceptable |
| **LOC** | 22,445 | Reduce complexity | 🟡 Needs analysis |
| **ShellCheck** | Not validated | 100% pass rate | 🔴 Must test |
| **Property Tests** | ~20 tests | 100+ properties | 🟡 Good start |
| **Determinism** | 1 test | Comprehensive suite | 🔴 Critical gap |
| **Performance** | Not benchmarked | <10ms transpile | 🔴 Must measure |

### 🐛 Known Critical Issues (from CLAUDE.md)

**Priority #1 - Correctness**:
1. **Control flow statements generate non-idempotent shell code**
   - Impact: Generated scripts may behave differently on repeated runs
   - Root cause: Unknown (needs investigation)
   - Tests: Missing comprehensive idempotence property tests

2. **String escaping fails with unicode inputs**
   - Impact: Security vulnerability - injection vector
   - Root cause: Unknown (needs investigation)
   - Tests: Missing unicode property-based tests

3. **Verification framework doesn't catch all injection vectors**
   - Impact: Critical security gap
   - Root cause: Incomplete validation rules
   - Tests: Need adversarial fuzzing

---

## 🚀 Sprint Plan - EXTREME TDD Methodology

### Sprint 0: Quality Gate Establishment (CURRENT)
**Goal**: Establish comprehensive quality baselines and quality gates
**Duration**: Until complete
**Philosophy**: 自働化 (Jidoka) - Build quality in from the start

#### Tasks:
- [x] Fix test compilation issues (bashrs naming)
- [x] Verify all tests pass
- [ ] **CRITICAL**: Measure test coverage with `cargo llvm-cov`
- [ ] **CRITICAL**: Run ShellCheck validation on all generated scripts
- [ ] **CRITICAL**: Establish performance baselines with criterion
- [ ] Document quality gate thresholds in `.quality/baseline.json`
- [ ] Create quality gate enforcement in Makefile
- [ ] Set up continuous integration quality checks

#### Quality Gates (Must Pass Before Sprint 1):
```yaml
coverage:
  minimum: 85%
  target: 90%

shellcheck:
  pass_rate: 100%
  severity: error

tests:
  passing: 100%
  property_tests: 20+

performance:
  transpile_simple: <10ms
  transpile_complex: <100ms
  memory_peak: <10MB

determinism:
  byte_identical: 100%
  runs: 10

security:
  injection_tests: 100%
  unicode_tests: 100%
  fuzzing_hours: 1hr minimum
```

#### Deliverables:
- [ ] Coverage report: `coverage-report.txt` (>85%)
- [ ] ShellCheck results: All examples + fixtures pass
- [ ] Performance baselines: `benchmarks/baseline.json`
- [ ] Quality gate config: `.quality/baseline.json`
- [ ] Documentation: `docs/quality-gates.md`

---

### Sprint 1: Critical Bug Fixes with EXTREME TDD
**Status**: Pending Sprint 0 completion
**Duration**: Until all critical bugs resolved
**Philosophy**: 反省 (Hansei) - Fix before adding

#### TICKET-1001: Fix Control Flow Idempotence ⚠️ CRITICAL
**Priority**: P0 - Blocking production use

**RED Phase** (Tests First):
```rust
// Property: Control flow must be idempotent
#[proptest]
fn prop_if_else_idempotent(condition: bool, then_code: String, else_code: String) {
    let source = format!(r#"
        fn main() {{
            if {} {{
                {}
            }} else {{
                {}
            }}
        }}
    "#, condition, then_code, else_code);

    let config = Config::default();
    let shell1 = transpile(&source, config.clone()).unwrap();

    // Run script twice, capture state changes
    let result1 = execute_shell(&shell1);
    let result2 = execute_shell(&shell1);

    // Property: Second run must produce identical state
    prop_assert_eq!(result1, result2);
}

// Property: While loops must be bounded and idempotent
#[proptest]
fn prop_while_loop_idempotent(#[strategy(0..10u32)] max_iterations: u32) {
    // Similar idempotence test for while loops
}

// Property: For loops must be deterministic
#[proptest]
fn prop_for_loop_deterministic(#[strategy(0..100u32)] range: u32) {
    // Test for loop determinism
}
```

**GREEN Phase** (Fix Implementation):
1. Investigate emitter/mod.rs control flow generation
2. Identify non-idempotent patterns
3. Fix shell code generation
4. Verify all tests pass

**REFACTOR Phase**:
1. Extract control flow emitter into separate module
2. Add comprehensive documentation
3. Run complexity analysis

**Quality Gates**:
- [ ] 15+ property tests for control flow idempotence
- [ ] 100% pass rate on generated scripts (run 10x each)
- [ ] ShellCheck clean on all control flow examples
- [ ] Cyclomatic complexity <15 for control flow emitter

#### TICKET-1002: Fix Unicode String Escaping ⚠️ CRITICAL
**Priority**: P0 - Security vulnerability

**RED Phase**:
```rust
// Property: All unicode must be safely escaped
#[proptest]
fn prop_unicode_safe_escaping(unicode_str: String) {
    let source = format!(r#"
        fn main() {{
            let msg = "{}";
            echo(msg);
        }}
    "#, unicode_str);

    let shell = transpile(&source, Config::default()).unwrap();

    // Property: Must pass shellcheck
    prop_assert!(shellcheck_passes(&shell));

    // Property: Must not allow injection
    prop_assert!(!allows_injection(&shell, &unicode_str));
}

// Specific test cases for known issues
#[test]
fn test_unicode_emoji_escaping() {
    assert_safe_transpile("Hello 👋 World! 🦀");
}

#[test]
fn test_unicode_arabic_escaping() {
    assert_safe_transpile("مرحبا بالعالم");
}

#[test]
fn test_unicode_chinese_escaping() {
    assert_safe_transpile("你好世界");
}

#[test]
fn test_unicode_combining_characters() {
    assert_safe_transpile("e\u{0301}"); // é as e + combining acute
}
```

**GREEN Phase**:
1. Investigate emitter/shell_escape.rs (or equivalent)
2. Add comprehensive unicode handling
3. Update quoting logic for all unicode ranges
4. Verify tests pass

**Quality Gates**:
- [ ] 100% pass rate on unicode property tests
- [ ] Specific tests for emoji, RTL, combining chars
- [ ] Fuzzing with unicode inputs (10,000+ cases)
- [ ] ShellCheck clean on all unicode examples

#### TICKET-1003: Complete Verification Framework ⚠️ CRITICAL
**Priority**: P0 - Security framework incomplete

**RED Phase**:
```rust
// Property: Verification must catch command injection
#[proptest]
fn prop_verification_catches_injection(malicious_input: String) {
    // Generate potentially malicious inputs
    let attempts = vec![
        format!("{}; rm -rf /", malicious_input),
        format!("{}$(whoami)", malicious_input),
        format!("{}| cat /etc/passwd", malicious_input),
        format!("{}`reboot`", malicious_input),
    ];

    for attempt in attempts {
        let source = format!(r#"
            fn main() {{
                let user_input = "{}";
                exec("echo {{}}", user_input);
            }}
        "#, attempt);

        let shell = transpile(&source, Config::paranoid()).unwrap();

        // Property: Must be properly quoted
        prop_assert!(properly_quoted(&shell, &attempt));

        // Property: Must not allow code execution
        prop_assert!(!allows_code_execution(&shell));
    }
}
```

**GREEN Phase**:
1. Audit validation/mod.rs verification rules
2. Implement missing injection detection
3. Add comprehensive quoting validation
4. Add glob expansion protection

**Quality Gates**:
- [ ] 50+ adversarial injection tests
- [ ] 100% detection rate on OWASP injection patterns
- [ ] Fuzzing with command injection payloads
- [ ] Security audit documentation

---

### Sprint 2: Property-Based Testing Enhancement
**Status**: Pending Sprint 1
**Goal**: Achieve >100 property-based tests covering all critical properties

#### TICKET-2001: Determinism Properties
**Extreme TDD**: Write 20+ property tests for deterministic transpilation

```rust
// Property: Same input always produces same output
#[proptest]
fn prop_transpilation_deterministic(source: ValidRustProgram) {
    let outputs: Vec<String> = (0..10)
        .map(|_| transpile(&source, Config::default()).unwrap())
        .collect();

    // All outputs must be byte-identical
    prop_assert!(outputs.windows(2).all(|w| w[0] == w[1]));
}

// Property: Hash of output must be stable
#[proptest]
fn prop_output_hash_stable(source: ValidRustProgram) {
    let hash1 = blake3::hash(transpile(&source, Config::default()).unwrap().as_bytes());
    let hash2 = blake3::hash(transpile(&source, Config::default()).unwrap().as_bytes());
    prop_assert_eq!(hash1, hash2);
}
```

#### TICKET-2002: POSIX Compliance Properties
**Extreme TDD**: Verify all generated scripts are POSIX-compliant

```rust
// Property: All outputs must pass shellcheck
#[proptest]
fn prop_shellcheck_passes(source: ValidRustProgram) {
    let shell = transpile(&source, Config::default()).unwrap();
    prop_assert!(shellcheck_passes(&shell));
}

// Property: Scripts must run in dash (strictest POSIX shell)
#[proptest]
fn prop_runs_in_dash(source: ValidRustProgram) {
    let shell = transpile(&source, Config::default()).unwrap();
    prop_assert!(dash_can_parse(&shell));
}
```

#### Quality Gates:
- [ ] 100+ property-based tests total
- [ ] Coverage of all critical properties (determinism, safety, POSIX)
- [ ] Fuzzing integration (cargo-fuzz or proptest)
- [ ] Property test documentation

---

### Sprint 3: Performance Optimization
**Status**: Pending Sprint 2
**Goal**: Meet <10ms transpilation target

#### TICKET-3001: Transpilation Performance
**Benchmarking First** (Extreme TDD for performance):

```rust
// Criterion benchmarks
fn bench_simple_transpilation(c: &mut Criterion) {
    c.bench_function("transpile_hello_world", |b| {
        b.iter(|| transpile(HELLO_WORLD_SOURCE, Config::default()))
    });
}

fn bench_complex_transpilation(c: &mut Criterion) {
    c.bench_function("transpile_installer", |b| {
        b.iter(|| transpile(INSTALLER_SOURCE, Config::default()))
    });
}
```

**Optimization Targets**:
1. Parser optimization (syn usage)
2. IR optimization passes
3. Emitter string concatenation
4. Memory allocations reduction

**Quality Gates**:
- [ ] Simple programs: <10ms (p50), <15ms (p99)
- [ ] Complex programs: <100ms (p50), <150ms (p99)
- [ ] Memory: <10MB peak for all cases
- [ ] Zero regressions in existing benchmarks

---

### Sprint 4: Advanced Features with Formal Verification
**Status**: Pending Sprint 3
**Goal**: Add pattern matching and loops with correctness proofs

#### TICKET-4001: Pattern Matching Support
**TDD with Formal Specs**:

```rust
// Formal specification in comments
/// # Correctness Property
/// Pattern matching must be:
/// 1. Exhaustive (all cases handled)
/// 2. Mutually exclusive (no overlapping patterns)
/// 3. Deterministic (same input → same branch)
///
/// # Shell Mapping
/// match expr {
///     pattern1 => code1,
///     pattern2 => code2,
///     _ => default
/// }
/// →
/// case "$expr" in
///     pattern1) code1 ;;
///     pattern2) code2 ;;
///     *) default ;;
/// esac

#[test]
fn test_exhaustive_match() {
    // Test that missing wildcard causes error
}

#[proptest]
fn prop_match_deterministic(value: String, patterns: Vec<String>) {
    // Property: Same value always takes same branch
}
```

---

### Sprint 5: Quality Gates & CI/CD Hardening
**Status**: Pending Sprint 4
**Goal**: Production-ready quality enforcement

#### Automated Quality Gates:
1. **Pre-commit hooks**:
   - rustfmt check
   - clippy with -D warnings
   - Property test quick check (100 iterations)

2. **CI Pipeline**:
   - Full test suite (all 600+ tests)
   - Coverage enforcement (>85%)
   - ShellCheck on all examples
   - Performance regression detection
   - Fuzzing (1 hour per PR)

3. **Release Gates**:
   - All tests passing
   - Coverage >90%
   - Security audit clean (cargo-audit)
   - Performance benchmarks within 5% of baseline
   - Documentation complete
   - ShellCheck clean on all examples

---

## 🎯 Success Criteria (Project Completion)

### Correctness
- ✅ All 600+ tests passing
- ✅ Coverage >90%
- ✅ 100+ property-based tests
- ✅ Zero known security vulnerabilities
- ✅ 100% ShellCheck pass rate
- ✅ Formal verification for critical paths

### Performance
- ✅ <10ms transpilation (simple)
- ✅ <100ms transpilation (complex)
- ✅ <10MB memory peak
- ✅ <3MB binary (minimal build)

### Quality
- ✅ Cyclomatic complexity <15 per function
- ✅ Cognitive complexity <20 per function
- ✅ Zero clippy warnings
- ✅ 100% documentation coverage for public API

### Production Readiness
- ✅ CI/CD pipeline with quality gates
- ✅ Fuzzing infrastructure
- ✅ Security audit documentation
- ✅ Performance monitoring
- ✅ Release process documented

---

## 📚 Documentation Requirements

Each sprint must produce:
1. **Technical Design Document** - Architecture and implementation details
2. **Test Plan** - Property tests, unit tests, integration tests
3. **Quality Report** - Coverage, performance, complexity metrics
4. **Security Analysis** - Threat model and mitigations
5. **User Documentation** - Examples and API docs

---

## 🔄 Continuous Improvement (改善 - Kaizen)

After each sprint:
1. Run `make kaizen` to collect metrics
2. Review quality trends
3. Identify technical debt
4. Plan improvements for next sprint
5. Update quality baselines

---

## 🚨 Emergency Procedures

If quality gates fail:
1. **STOP** - Do not proceed to next sprint
2. **ANALYZE** - Root cause analysis (現地現物 - Go and see)
3. **FIX** - Address root cause, not symptoms
4. **VERIFY** - Ensure fix and add regression tests
5. **DOCUMENT** - Update documentation and procedures

---

**Last Updated**: 2025-10-02
**Status**: Sprint 0 - Quality Gate Establishment
**Next Milestone**: Complete Sprint 0 quality baselines
