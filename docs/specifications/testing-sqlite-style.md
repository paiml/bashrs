# Bashrs Testing Specification: SQLite-Inspired Quality Standards v1.0

**Document ID**: BASHRS-SPEC-001
**Version**: 1.0
**Status**: ACTIVE
**Last Updated**: October 16, 2025
**Authors**: Bashrs Development Team

## Executive Summary

This specification defines comprehensive testing standards for bashrs (Rash) following SQLite's legendary testing methodology. SQLite achieves 100% branch coverage with 248.5 million test instances before each release. We adapt these principles for bashrs's mission: bidirectional shell safety through Rust transpilation and Makefile purification.

**Core Philosophy**: **Build Quality In (自働化 - Jidoka)** - Test exhaustively, catch defects immediately, achieve zero-defect releases.

## Research Foundation: SQLite Testing Excellence

### SQLite Testing By The Numbers

| Metric | SQLite Standard | Bashrs Target |
|--------|----------------|---------------|
| **Test Code Ratio** | 608:1 (test:source) | 100:1 minimum |
| **Branch Coverage** | 100% | 100% for parser/transpiler |
| **Test Instances** | 248.5M per release | 1M+ per release |
| **Test Frameworks** | TH3 + TCL + SLT | Rust built-in + proptest + mutants |
| **Mutation Score** | ~100% (inferred) | >90% measured |
| **CI Duration** | Hours of comprehensive testing | <30 min unit, <2 hr full suite |

### Key SQLite Testing Principles Adapted for Bashrs

1. **100% Branch Coverage**: Every code path must be tested
2. **Multi-Harness Testing**: Unit + Integration + Property + Mutation + Fuzz
3. **Boundary Value Testing**: Test limits and edge cases exhaustively
4. **Regression Prevention**: Every bug becomes a permanent test
5. **Deterministic Testing**: Tests must be reproducible and stable
6. **Performance Baseline**: Detect performance regressions automatically

## Bashrs-Specific Testing Challenges

### 1. **Bidirectional Transpilation Testing**

**Challenge**: Ensure correctness in both directions:
- **Rust → Shell**: Safe POSIX shell generation
- **Bash → Rust → Purified Bash**: Deterministic purification

**Solution**: Cross-verification testing
```rust
#[test]
fn test_bidirectional_roundtrip() {
    let rust_code = "fn test() { run_command(\"echo\", &[\"hello\"]); }";
    let shell_code = transpile_to_shell(rust_code);

    // Verify shell correctness
    assert!(shellcheck_passes(&shell_code));
    assert!(is_posix_compliant(&shell_code));
    assert!(is_deterministic(&shell_code));

    // Verify behavioral equivalence
    let rust_output = execute_rust(rust_code);
    let shell_output = execute_shell(&shell_code);
    assert_eq!(rust_output, shell_output);
}
```

### 2. **Makefile Parser Correctness**

**Challenge**: Parse GNU Makefile syntax with 100% accuracy

**Solution**: Comprehensive GNU Make manual coverage
```rust
// Test EVERY construct from GNU Make manual
#[test]
fn test_makefile_parser_coverage() {
    // 150 tasks from MAKE-INGESTION-ROADMAP.yaml
    for task in ROADMAP_TASKS {
        let input = task.input;
        let ast = parse_makefile(input).unwrap();

        // Verify parsing correctness
        assert_eq!(ast.items.len(), task.expected_items);
        assert_matches!(ast.items[0], task.expected_type);
    }
}
```

### 3. **Shell Safety Guarantees**

**Challenge**: Prove generated shell is injection-safe and idempotent

**Solution**: Property-based safety testing
```rust
#[quickcheck]
fn shell_output_always_safe(rust_input: ValidRustProgram) -> bool {
    let shell = transpile_to_shell(&rust_input.0);

    // Safety properties
    assert!(no_injection_vectors(&shell));
    assert!(all_vars_quoted(&shell));
    assert!(no_eval_or_exec(&shell));
    assert!(is_idempotent(&shell));

    true
}
```

## Bashrs Testing Framework: Five Pillars

### Pillar 1: Unit Tests - Feature Completeness

**Goal**: 100% coverage of all implemented features
**Framework**: Rust built-in `#[test]`
**Target**: 1000+ unit tests

#### Test Categories

1. **Parser Tests** (`rash/src/make_parser/tests.rs`)
   - Target: 100% of Makefile constructs (150 tasks)
   - Current: 14/150 tasks (9.33%), 1110 tests passing
   - Example: RULE-SYNTAX-001, VAR-BASIC-001, ECHO-001

2. **AST Tests** (`rash/src/ast/tests.rs`)
   - Target: All AST node types and transformations
   - Current: Comprehensive restricted AST tests
   - Focus: Type safety, immutability, ownership

3. **Transpiler Tests** (`rash/src/transpiler/tests.rs`)
   - Target: Rust → Shell correctness
   - Verify: POSIX compliance, safety, determinism
   - Example: String escaping, variable quoting, command generation

4. **Purifier Tests** (`rash/src/purifier/tests.rs`)
   - Target: Bash → Purified Bash correctness
   - Verify: Determinism, idempotency, behavioral equivalence
   - Example: Remove $RANDOM, convert `=` to `:=`, add `.PHONY`

#### Naming Convention

```rust
// Pattern: test_{TASK_ID}_{feature}_{scenario}
#[test]
fn test_RULE_SYNTAX_001_basic_rule_syntax() {
    // ARRANGE
    let makefile = "target: prerequisites\n\trecipe";

    // ACT
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT
    assert_eq!(ast.items.len(), 1);
    assert_matches!(ast.items[0], MakeItem::Target { .. });
}
```

### Pillar 2: Property-Based Tests - Edge Case Discovery

**Goal**: Discover unexpected edge cases through generative testing
**Framework**: `proptest`
**Target**: 100+ property tests, 10K+ generated test cases

#### Property Categories

1. **Parser Properties**
   ```rust
   proptest! {
       #[test]
       fn prop_parser_always_terminates(input in ".*") {
           // Parser must never hang or panic
           let _ = parse_makefile(&input);
       }

       #[test]
       fn prop_parsing_deterministic(input in valid_makefile_pattern()) {
           // Same input = same output
           let ast1 = parse_makefile(&input).unwrap();
           let ast2 = parse_makefile(&input).unwrap();
           assert_eq!(ast1, ast2);
       }
   }
   ```

2. **Transpiler Properties**
   ```rust
   proptest! {
       #[test]
       fn prop_transpile_preserves_semantics(
           rust_code in valid_rust_pattern()
       ) {
           let shell = transpile_to_shell(&rust_code);
           let rust_output = execute_rust(&rust_code);
           let shell_output = execute_shell(&shell);
           assert_eq!(rust_output, shell_output);
       }
   }
   ```

3. **Safety Properties**
   ```rust
   proptest! {
       #[test]
       fn prop_no_injection_possible(
           user_input in ".*",
           rust_code in valid_rust_pattern()
       ) {
           let shell = transpile_to_shell(&rust_code);
           // Even with malicious input, output is safe
           assert!(no_code_injection(&shell, &user_input));
       }
   }
   ```

### Pillar 3: Mutation Testing - Test Quality Verification

**Goal**: >90% mutation kill rate
**Framework**: `cargo-mutants`
**Target**: All critical code paths

#### Mutation Testing Workflow

```bash
# Run mutation testing on parser
cargo mutants --file rash/src/make_parser/parser.rs -- --lib

# Expected output:
# Killed: 48/53 mutants (90.6% kill rate) ✅
# Missed: 5/53 mutants (need better tests)
```

#### Mutation Test Examples

```rust
// Kill mutant: line 270 `recipe.push(...)` → `recipe.clear()`
#[test]
fn test_RECIPE_001_mut_recipe_push_must_happen() {
    let makefile = "build:\n\tcargo build";
    let ast = parse_makefile(makefile).unwrap();

    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 1); // Would fail if push() removed
            assert_eq!(recipe[0], "cargo build");
        }
        _ => panic!("Expected Target"),
    }
}
```

### Pillar 4: Integration Tests - Real-World Workflows

**Goal**: Test complete end-to-end workflows
**Framework**: `tests/` directory integration tests
**Target**: All major use cases

#### Integration Test Scenarios

1. **Rust → Shell Transpilation**
   ```rust
   #[test]
   fn integration_rust_to_shell_workflow() {
       // Full workflow: write Rust → transpile → shellcheck → execute
       let rust_file = "examples/install.rs";
       let shell_file = "target/install.sh";

       // Transpile
       let shell_code = transpile_file(rust_file).unwrap();
       fs::write(shell_file, &shell_code).unwrap();

       // Verify shellcheck passes
       assert!(shellcheck_command(shell_file).success());

       // Verify execution works
       assert!(execute_shell_file(shell_file).success());
   }
   ```

2. **Bash → Purified Bash Workflow**
   ```rust
   #[test]
   fn integration_bash_purification_workflow() {
       let messy_makefile = r#"
       RELEASE := $(shell date +%s)
       FILES := $(wildcard *.c)

       test:
           cargo test
       "#;

       // Parse → Analyze → Purify → Emit
       let ast = parse_makefile(messy_makefile).unwrap();
       let purified = purify_makefile(&ast).unwrap();

       // Verify purification
       assert!(!purified.contains("$(shell date"));
       assert!(!purified.contains("$(wildcard"));
       assert!(purified.contains(".PHONY: test"));
       assert!(purified.contains(":="));
   }
   ```

### Pillar 5: Regression Tests - Zero Defect Policy

**Goal**: 100% of bugs become permanent tests
**Framework**: Dedicated regression test suite
**Target**: Every GitHub issue, every bug report

#### Regression Test Structure

```rust
// tests/regression/github_issues.rs

#[test]
fn regression_issue_42_parser_hangs_on_empty_recipe() {
    // GitHub Issue #42: Parser infinite loop on empty recipe
    let makefile = "target:\n\n";

    // Must complete without hanging
    let result = parse_makefile(makefile);
    assert!(result.is_ok());
}

#[test]
fn regression_issue_103_variable_in_prerequisite() {
    // GitHub Issue #103: Variables in prerequisites not parsed
    let makefile = "target: $(DEPS)\n\techo done";

    let ast = parse_makefile(makefile).unwrap();
    match &ast.items[0] {
        MakeItem::Target { prerequisites, .. } => {
            assert_eq!(prerequisites[0], "$(DEPS)");
        }
        _ => panic!("Expected Target"),
    }
}
```

## Test Organization Structure

```
rash/
├── src/
│   ├── make_parser/
│   │   ├── parser.rs              # Parser implementation
│   │   └── tests.rs                # 1110 parser unit tests ✅
│   ├── ast/
│   │   ├── restricted.rs           # AST types
│   │   └── restricted_test.rs      # AST unit tests ✅
│   └── transpiler/
│       ├── mod.rs                  # Transpiler implementation
│       └── tests.rs                # Transpiler unit tests
├── tests/
│   ├── integration/
│   │   ├── rust_to_shell.rs        # Rust → Shell workflows
│   │   ├── bash_purification.rs    # Bash purification workflows
│   │   └── cli_commands.rs         # CLI integration tests
│   ├── properties/
│   │   ├── parser_properties.rs    # Parser property tests
│   │   ├── safety_properties.rs    # Safety guarantees
│   │   └── semantic_properties.rs  # Semantic equivalence
│   ├── regression/
│   │   ├── github_issues.rs        # All GitHub issues
│   │   └── bug_reports.rs          # All bug reports
│   └── benchmarks/
│       ├── parse_performance.rs    # Parser benchmarks
│       └── transpile_performance.rs # Transpiler benchmarks
└── mutants.out/                   # Mutation testing results
    ├── caught.txt                  # Killed mutants ✅
    ├── missed.txt                  # Surviving mutants ⚠️
    └── unviable.txt                # Invalid mutants
```

## Quality Gates and CI Integration

### Mandatory Quality Gates

1. **Unit Test Gate** - MUST PASS
   ```bash
   cargo test --lib
   # Target: 1000+ tests, 100% pass rate
   ```

2. **Property Test Gate** - MUST PASS
   ```bash
   cargo test --test properties -- --test-threads=1
   # Target: 10K+ generated cases, 100% pass rate
   ```

3. **Mutation Test Gate** - MUST PASS
   ```bash
   cargo mutants --file rash/src/make_parser/parser.rs -- --lib
   # Target: >90% kill rate
   ```

4. **Integration Test Gate** - MUST PASS
   ```bash
   cargo test --test integration
   # Target: All workflows functional
   ```

5. **Shellcheck Gate** - MUST PASS
   ```bash
   # All generated shell must pass shellcheck
   find target/generated -name "*.sh" -exec shellcheck -s sh {} \;
   ```

### CI Pipeline (GitHub Actions)

```yaml
name: SQLite-Style Quality Testing
on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run unit tests
        run: cargo test --lib --verbose
        timeout-minutes: 10

      - name: Check test count
        run: |
          TEST_COUNT=$(cargo test --lib -- --list | grep -c "test ")
          if [ "$TEST_COUNT" -lt 1000 ]; then
            echo "ERROR: Only $TEST_COUNT tests (target: 1000+)"
            exit 1
          fi

  property-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run property tests
        run: cargo test --test properties --release
        timeout-minutes: 30

      - name: Verify test case count
        run: |
          # Property tests should generate 10K+ cases
          cargo test --test properties -- --nocapture | \
            grep -E "^\[quickcheck\]" | wc -l

  mutation-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-mutants
        run: cargo install cargo-mutants

      - name: Run mutation tests
        run: |
          cargo mutants --file rash/src/make_parser/parser.rs -- --lib
          cargo mutants --file rash/src/ast/restricted.rs -- --lib
        timeout-minutes: 120

      - name: Check mutation score
        run: |
          KILL_RATE=$(grep -oP 'Killed: \K\d+\.\d+' mutants.out/outcomes.json)
          if (( $(echo "$KILL_RATE < 90.0" | bc -l) )); then
            echo "ERROR: Mutation score $KILL_RATE% (target: >90%)"
            exit 1
          fi

  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install shellcheck
        run: sudo apt-get install -y shellcheck

      - name: Run integration tests
        run: cargo test --test integration --verbose

      - name: Verify generated shell quality
        run: |
          find target/generated -name "*.sh" -exec shellcheck -s sh {} \;

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install llvm-cov
        run: cargo install cargo-llvm-cov

      - name: Generate coverage report
        run: cargo llvm-cov --html --output-dir target/coverage

      - name: Check coverage threshold
        run: |
          COVERAGE=$(cargo llvm-cov --json | jq '.data[0].totals.lines.percent')
          if (( $(echo "$COVERAGE < 85.0" | bc -l) )); then
            echo "ERROR: Coverage $COVERAGE% (target: >85%)"
            exit 1
          fi

      - name: Upload coverage report
        uses: actions/upload-artifact@v4
        with:
          name: coverage-report
          path: target/coverage/

  performance-regression:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run benchmarks
        run: cargo test --test benchmarks --release -- --nocapture

      - name: Compare with baseline
        run: |
          # Compare current performance with baseline
          python3 scripts/check_performance_regression.py \
            target/benchmarks/current.json \
            target/benchmarks/baseline.json
```

## Metrics and Success Criteria

### Current Status (Sprint 40)

| Category | Target | Current | Status |
|----------|--------|---------|--------|
| **Unit Tests** | 1000+ | 1110 | ✅ EXCEEDS |
| **Test Pass Rate** | 100% | 100% | ✅ PERFECT |
| **Parser Coverage** | 100% (150 tasks) | 9.33% (14 tasks) | 🔄 IN PROGRESS |
| **Mutation Score** | >90% | 92.6% (parser) | ✅ EXCEEDS |
| **Property Tests** | 100+ | 150+ | ✅ EXCEEDS |
| **Code Coverage** | >85% | TBD | 📊 MEASURE |
| **Integration Tests** | 50+ | TBD | 🎯 IMPLEMENT |

### Performance Baselines

```rust
const PERFORMANCE_THRESHOLDS: &[(&str, Duration)] = &[
    ("parse_simple_makefile", Duration::from_millis(1)),
    ("parse_complex_makefile", Duration::from_millis(10)),
    ("transpile_rust_to_shell", Duration::from_millis(5)),
    ("purify_bash_script", Duration::from_millis(10)),
    ("end_to_end_workflow", Duration::from_millis(100)),
];
```

## Implementation Roadmap

### Phase 1: Foundation (✅ CURRENT - Sprint 40)
- ✅ EXTREME TDD workflow established
- ✅ 1110 unit tests passing (100% pass rate)
- ✅ Parser mutation testing (92.6% kill rate)
- ✅ Property testing framework (150+ property tests)
- ✅ 14/150 Makefile parser tasks complete (9.33%)

### Phase 2: Expansion (🎯 NEXT - Sprints 41-50)
- 🎯 Complete Makefile parser (150/150 tasks = 100%)
- 🎯 Add integration test suite (50+ tests)
- 🎯 Implement transpiler tests (100+ tests)
- 🎯 Add purifier tests (100+ tests)
- 🎯 Achieve >85% code coverage

### Phase 3: Excellence (📅 FUTURE - Sprints 51-60)
- 📅 Fuzz testing integration
- 📅 Performance regression detection
- 📅 Cross-platform testing (Linux, macOS, Windows)
- 📅 SQLite-level test count (1M+ test instances)

## Toyota Way Principles in Testing

### 自働化 (Jidoka) - Build Quality In
- Tests written BEFORE implementation (RED phase)
- Mutation testing ensures test quality
- Property tests discover edge cases automatically
- CI gates prevent defects from merging

### 現地現物 (Genchi Genbutsu) - Go and See
- Test against real shells (dash, bash, sh)
- Verify on actual Alpine containers
- Profile real-world Makefiles
- Measure actual performance

### 反省 (Hansei) - Reflect and Improve
- Every bug becomes a regression test
- Failed tests trigger immediate investigation
- Mutation survivors reveal test gaps
- Regular test suite review and cleanup

### 改善 (Kaizen) - Continuous Improvement
- Test count increases with each sprint
- Coverage improves continuously
- Mutation score trends upward
- Performance baselines tighten over time

## Conclusion

This specification establishes bashrs's testing as a **first-class engineering discipline** following SQLite's legendary quality standards. By implementing these practices, we ensure:

1. **Zero-defect releases** through exhaustive testing
2. **Behavioral correctness** through property-based testing
3. **Test quality** through mutation testing
4. **Real-world reliability** through integration testing
5. **Performance stability** through regression detection

**The Goal**: Achieve SQLite-level confidence that bashrs is **production-ready and battle-tested**.

---

**Status**: APPROVED
**Implementation**: IN PROGRESS (Sprint 40)
**Next Review**: Sprint 50 (after Phase 2 completion)
**Target**: SQLite-level quality by v2.0.0
