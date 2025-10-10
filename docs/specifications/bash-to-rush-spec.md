I'll research this systematically, starting with understanding the Rash codebase and then exploring formal methods for shell script verification and transpilation.# Bash-to-Rash Transpilation with Formal Verification: A Research Roadmap

## Executive Summary

This research proposes a bidirectional transpilation workflow between bash scripts and Rash (Rust-subset) with formal correctness guarantees, achieving 80-100% test coverage through **extreme Test-Driven Development** (TDD), mutation testing, and property-based testing. The workflow ingests legacy bash scripts, transpiles them to Rash with verification, applies "purification" transformations, and optionally regenerates deterministic bash with correctness proofs.

**Key Innovation**: Unlike prior shell verification workfocusing on static analysis of existing scripts, this system provides **bidirectional semantic equivalence** with mechanical verification via differential testingat the transpiler boundary.

---

## Related Academic Work

### Shell Script Verification

Recent work by Vasilakis et al. (HotOS 2025) introduces static analysis for Unix shell programs, leveraging formal semantics and symbolic execution to detect correctness issues before runtime. This represents the first successful academic attempt at shell script static verification after multiple failed approaches.

ABASH (Mazurak & Zdancewic, 2007) pioneered static analysis for bash through abstract interpretation, handling shell variable expansion and external program interfaces via signature-based modeling.

### Compiler Correctness & Differential Testing

CrossLangFuzzer (2025) demonstrates differential testing across JVM compilers using a universal intermediate representation, detecting 24 confirmed bugs through cross-language semantic equivalence checks.

Differential testing of formal verification frameworks (Utting et al., 2022) validates that high-level Isabelle/HOL models match Java compiler implementations, achieving 99.76% test pass rate through systematic boundary-value testing and translation validation.

Comprehensive surveys reveal that Randomized Differential Testing (RDT) is more effective for general bugs, while Different Optimization Levels (DOL) excels at optimization-related defects. Equivalence Modulo Inputs (EMI) provides complementary coverage.

### Property-Based Testing

Hypothesis introduces shrinking strategies that transparently mutate failing test cases to minimal reproducible examples, storing rich structural information about generated test data.

QuickCheck (Claessen & Hughes, 2000) established property-based testing for Haskell, automatically generating random inputs and simplifying counterexamples through user-defined generators.

### Mutation Testing for Language Tools

Hariri et al. (2016) evaluate mutation testing effects at compiler IR level, demonstrating that mutation operators expose inadequate test suites for compiler optimization phases.

The language mutation problem (2022) applies mutation testing to interpreters via language product lines, avoiding source code modification and recompilation by creating runtime language variants.

---

## Proposed Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                    BASH-TO-RASH WORKFLOW                      │
└───────────────────────────────────────────────────────────────┘

Phase 1: Ingestion & Parsing
    ┌─────────────────┐
    │  Legacy Bash    │────────────────────────────────┐
    │   Scripts       │                                │
    └────────┬────────┘                                │
             │                                         │
             ▼                                         │
    ┌─────────────────┐                                │
    │ Formal Parser   │ (ShellCheck + Morbig)         │
    │  + AST Gen      │                                │
    └────────┬────────┘                                │
             │                                         │
             ▼                                         │
    ┌─────────────────┐                                │
    │ Semantic        │ (Effects, Variable Scopes)     │
    │ Analysis        │                                │
    └────────┬────────┘                                │
             │                                         │
             │                                         │
Phase 2: Bash→Rash Transpilation                      │
             │                                         │
             ▼                                         │
    ┌─────────────────┐                                │
    │   Pattern       │ (Template Matching)            │
    │   Matching      │                                │
    └────────┬────────┘                                │
             │                                         │
             ▼                                         │
    ┌─────────────────┐                                │
    │ Rash IR         │ (Type-Safe AST)                │
    │ Generation      │                                │
    └────────┬────────┘                                │
             │                                         │
             ▼                                         │
    ┌─────────────────┐                                │
    │ Purification    │ (Idempotency, Determinism)     │
    │ Transform       │                                │
    └────────┬────────┘                                │
             │                                         │
             ▼                                         │
    ┌─────────────────┐                                │
    │  Rash Source    │                                │
    │  + Test Suite   │ (80-100% Coverage)             │
    └────────┬────────┘                                │
             │                                         │
             │                                         │
Phase 3: Verification                                  │
             │                                         │
             ├──────────────┐                          │
             │              │                          │
             ▼              ▼                          │
    ┌──────────────┐  ┌──────────────┐                │
    │ Property     │  │ Differential │                │
    │ Tests (PBT)  │  │ Testing      │◄───────────────┘
    └──────┬───────┘  └──────┬───────┘
           │                 │
           ▼                 ▼
    ┌──────────────────────────────┐
    │   Mutation Testing           │ (Cosmic Ray + Custom)
    │   - AST Mutators             │
    │   - Type Signature Mutators  │
    │   - Effect Mutators          │
    └──────────┬───────────────────┘
               │
               ▼
    ┌──────────────────────────────┐
    │  Clippy + Custom Lints       │ (CCN <10, SATD=0)
    └──────────┬───────────────────┘
               │
               │
Phase 4: Rash→Bash (Optional)
               │
               ▼
    ┌──────────────────────────────┐
    │  Deterministic Bash Gen      │
    │  + Equivalence Proof         │
    └──────────┬───────────────────┘
               │
               ▼
    ┌──────────────────────────────┐
    │  POSIX-Compliant Output      │
    │  (ShellCheck Verified)       │
    └──────────────────────────────┘
```

---

## Testing Infrastructure: Extreme TDD + PMAT Integration

### 1. Extreme Test-Driven Development Protocol

Following Toyota Way principles from PMAT:

```yaml
# .pmat/workflow.yaml - Sprint Definition
sprint:
  id: "bash-rash-001"
  name: "Bash Variable Expansion Transpiler"

  pre_commit:
    - pmat analyze complexity --fail-on-violation --max-complexity 10
    - pmat analyze satd --fail-on-violation
    - cargo test --all-features
    - cargo clippy -- -D warnings

  quality_gates:
    complexity_threshold: 10  # McCabe CCN
    satd_tolerance: 0         # Zero technical debt
    coverage_minimum: 80      # Line coverage
    mutation_score: 0.85      # 85% mutants killed

  roadmap:
    tasks:
      - id: "T001"
        name: "Parse simple variable expansion"
        test_first: true
        prerequisites: []
        verification:
          - type: "property_test"
            property: "parse(gen_bash_var()) preserves semantics"
          - type: "mutation_test"
            operators: ["ArithmeticOperator", "BooleanOperator"]
          - type: "differential_test"
            oracle: "bash execution"
```

### 2. Property-Based Testing Specification

Using Hypothesis for Python components, proptest for Rust:

```rust
// rash-transpiler/tests/properties.rs
use proptest::prelude::*;

proptest! {
    /// Property: Transpilation preserves script semantics
    /// Rationale: ∀ bash_script, exec(bash_script) ≡ exec(transpile(bash_script))
    #[test]
    fn transpilation_semantic_equivalence(
        bash_script in bash_script_generator()
    ) {
        let rash_code = transpile(&bash_script)?;
        let bash_output = execute_bash(&bash_script)?;
        let rash_output = execute_rash(&rash_code)?;

        prop_assert_eq!(
            normalize_output(bash_output),
            normalize_output(rash_output)
        );
    }

    /// Property: Purification maintains idempotency
    /// Rationale: ∀ script, ∃ state s.t. exec(script, s) = exec(exec(script, s), s)
    #[test]
    fn purification_idempotency(
        script in rash_script_generator()
    ) {
        let purified = purify(script.clone())?;
        let state = create_test_state();

        let result1 = execute_with_state(&purified, &state)?;
        let state_after = result1.final_state;
        let result2 = execute_with_state(&purified, &state_after)?;

        prop_assert_eq!(result1, result2);
    }

    /// Property: Determinism across executions
    /// Rationale: ∀ script, exec(script) always produces same output
    #[test]
    fn determinism_property(
        script in deterministic_bash_generator()
    ) {
        let outputs: Vec<_> = (0..100)
            .map(|_| execute_bash(&script))
            .collect();

        prop_assert!(outputs.windows(2).all(|w| w[0] == w[1]));
    }
}

fn bash_script_generator() -> impl Strategy<Value = String> {
    prop::collection::vec(bash_statement_gen(), 1..20)
        .prop_map(|stmts| stmts.join("\n"))
}
```

### 3. Mutation Testing Configuration

```toml
# cosmic_ray_config.toml (for Python transpiler components)
[cosmic-ray]
module-path = "rash_parser"
timeout = 30.0
exclude-modules = ["tests", "benchmarks"]

test-command = "pytest -x tests/"

[cosmic-ray.distributor]
name = "local"

[cosmic-ray.operators]
exclude = [
    "core/RemoveDecorator",  # Preserve type annotations
]

# Custom operators for AST mutations
include-custom = [
    "ast/SwapBinaryOperator",
    "ast/ChangeVariableScope",
    "ast/ModifyTypeSignature"
]
```

```rust
// Custom Rust mutation testing via cargo-mutants
// .cargo-mutants.toml
[[mutants]]
# Mutate arithmetic operations in code generator
include = ["src/codegen/*.rs"]
operators = [
    "arithmetic",      # + → -, * → /, etc.
    "relational",      # < → <=, == → !=
    "boolean",         # && → ||
]

[[mutants]]
# Mutate AST transformations
include = ["src/transpiler/*.rs"]
operators = [
    "return_value",    # Change function returns
    "call_args",       # Modify function call arguments
]

timeout_multiplier = 3
minimum_test_time_ms = 100
```

### 4. Differential Testing Oracle

```rust
// tests/differential.rs
use std::process::Command;

#[test]
fn differential_test_suite() {
    let corpus = load_bash_corpus("tests/corpus/*.sh");

    for (name, bash_script) in corpus {
        // Execute original bash
        let bash_result = execute_bash_sandboxed(&bash_script)
            .expect(&format!("Bash execution failed: {}", name));

        // Transpile to Rash
        let rash_code = transpile(&bash_script)
            .expect(&format!("Transpilation failed: {}", name));

        // Execute Rash version
        let rash_result = execute_rash_sandboxed(&rash_code)
            .expect(&format!("Rash execution failed: {}", name));

        // Compare outputs (stdout, stderr, exit code, filesystem effects)
        assert_eq!(
            bash_result.stdout, rash_result.stdout,
            "stdout mismatch for {}", name
        );
        assert_eq!(
            bash_result.stderr, rash_result.stderr,
            "stderr mismatch for {}", name
        );
        assert_eq!(
            bash_result.exit_code, rash_result.exit_code,
            "exit code mismatch for {}", name
        );
        assert_fs_effects_eq!(
            bash_result.fs_effects, rash_result.fs_effects,
            "filesystem effects mismatch for {}", name
        );
    }
}

fn execute_bash_sandboxed(script: &str) -> Result<ExecutionResult> {
    // Use Docker + seccomp to sandbox execution
    Command::new("docker")
        .args(&[
            "run", "--rm", "--network=none",
            "--security-opt", "seccomp=strict.json",
            "bash:5.1", "bash", "-c", script
        ])
        .output()
        .map(ExecutionResult::from)
}
```

---

## PMAT YAML Roadmapping: Implementation Phases

### Phase 1: Parser Foundation (Weeks 1-4)

```yaml
epic: "E1-Parser"
owner: "team-transpiler"

research_questions:
  - "Can we parse 95% of real-world bash scripts from GitHub corpus?"
  - "What subset of POSIX shell features are representable in Rash IR?"
  - "What is the average parsing latency for 1-10k LOC scripts?"

milestones:
  - id: "M1.1"
    name: "Formal Grammar Implementation"
    deliverables:
      - "ANTLR4 grammar for bash subset"
      - "AST with position tracking"
      - "Property tests: parse(gen()) always succeeds"
    verification:
      - pmat analyze complexity --max 10
      - cargo test --all-features
      - hypothesis test --hypothesis-profile ci

  - id: "M1.2"
    name: "Semantic Analysis"
    deliverables:
      - "Variable scope resolution"
      - "Command effect tracking"
      - "Differential tests vs bash -n"
    quality_gate:
      coverage: 90
      mutation_score: 0.85

  - id: "M1.3"
    name: "Corpus Evaluation"
    deliverables:
      - "GitHub corpus (10k scripts)"
      - "Parse success rate report"
      - "Failure categorization"
    metrics:
      parse_success_rate: ">= 0.95"
      avg_latency_ms: "<= 500"
```

### Phase 2: Rash IR & Transpiler (Weeks 5-12)

```yaml
epic: "E2-Transpiler"
dependencies: ["E1-Parser"]

research_questions:
  - "Can we prove semantic equivalence for core bash constructs?"
  - "What is the overhead of safety checks in generated Rash code?"
  - "How do we handle non-deterministic bash features (e.g., $RANDOM)?"

tasks:
  - id: "T2.1"
    name: "Pattern-Based Translation Rules"
    tdd_protocol:
      1. Write property: "translate(bash_if) → rash_if preserves condition"
      2. Write failing test
      3. Implement minimal translator
      4. Verify with mutation testing
      5. Commit only if: CCN < 10, SATD = 0, coverage >= 80%

  - id: "T2.2"
    name: "Type Inference Engine"
    deliverables:
      - "Hindley-Milner-style inference for shell vars"
      - "Effect system for command invocations"
      - "Proof of soundness (Coq/Lean optional)"
    verification:
      - 1000+ property tests
      - Differential testing vs bash type errors
      - Mutation testing: 90% kill rate

  - id: "T2.3"
    name: "Purification Transformations"
    deliverables:
      - "Idempotency analysis"
      - "Determinism rewriting (remove $RANDOM, $$, etc.)"
      - "Side-effect isolation"
    metrics:
      idempotency_violations: "= 0 for purified scripts"
      determinism_score: ">= 0.95"
```

### Phase 3: Verification Suite (Weeks 13-16)

```yaml
epic: "E3-Verification"
dependencies: ["E2-Transpiler"]

research_questions:
  - "What mutation operators expose gaps in transpiler correctness?"
  - "Can we mechanically verify the transpiler using Coq/Isabelle?"
  - "What is the false positive rate of our differential oracle?"

tasks:
  - id: "T3.1"
    name: "Property-Based Test Suite"
    deliverables:
      - "50+ properties covering all bash constructs"
      - "Custom generators for valid bash syntax"
      - "Shrinking strategies for minimal counterexamples"
    acceptance_criteria:
      - "10,000+ test cases per property"
      - "Average shrinking to <10 lines for failures"

  - id: "T3.2"
    name: "Mutation Testing Infrastructure"
    deliverables:
      - "Custom mutation operators for transpiler"
      - "CI pipeline with parallel mutation execution"
      - "Mutation score tracking dashboard"
    targets:
      mutation_score: ">= 0.90"
      execution_time: "<= 30 minutes on GitHub Actions"

  - id: "T3.3"
    name: "Differential Oracle"
    deliverables:
      - "Sandboxed bash execution environment"
      - "Filesystem diff tool"
      - "Output normalization (timestamps, PIDs, etc.)"
    metrics:
      false_positive_rate: "<= 0.01"
      corpus_coverage: ">= 0.95"
```

### Phase 4: Rash→Bash Back-Transpilation (Weeks 17-20)

```yaml
epic: "E4-BackTranspile"
dependencies: ["E3-Verification"]

research_questions:
  - "Can we generate shell-check-clean bash from arbitrary Rash?"
  - "What is the code size overhead of safety checks?"
  - "Do round-trip translations preserve semantics?"

tasks:
  - id: "T4.1"
    name: "Code Generation with Proofs"
    deliverables:
      - "Rash→Bash codegen with equivalence proof"
      - "ShellCheck integration"
      - "POSIX compliance verification"
    verification:
      - Round-trip test: bash → rash → bash ≡ bash
      - ShellCheck score: 0 warnings

  - id: "T4.2"
    name: "Optimization Pass"
    deliverables:
      - "Dead code elimination"
      - "Constant folding"
      - "Redundant check removal"
    metrics:
      code_size_overhead: "<= 20% vs handwritten"
      performance_overhead: "<= 10% execution time"
```

---

## Continuous Integration & Quality Enforcement

```yaml
# .github/workflows/quality-gate.yml
name: Extreme TDD Quality Gate

on: [push, pull_request]

jobs:
  quality-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # PMAT static analysis
      - name: Install PMAT
        run: cargo install pmat

      - name: Check Complexity
        run: pmat analyze complexity --fail-on-violation --max 10

      - name: Check SATD
        run: pmat analyze satd --fail-on-violation

      # Unit tests
      - name: Run Tests
        run: cargo test --all-features -- --nocapture

      # Property-based tests
      - name: Property Tests
        run: cargo test --test properties --release
        env:
          PROPTEST_CASES: 10000

      # Mutation testing (cached)
      - name: Mutation Testing
        run: |
          cargo install cargo-mutants
          cargo mutants --timeout 600 --jobs 4 --in-diff

      # Differential testing
      - name: Differential Oracle
        run: cargo test --test differential --release

      # Coverage
      - name: Coverage Report
        run: make coverage
```

---

## Key Innovations & Research Contributions

1. **Bidirectional Semantic Preservation**: Unlike static analyzers, this system provides mechanical proofs of equivalence across transpilation boundaries.

2. **Purification as Program Transformation**: Formalizes shell script "cleaning" as type-preserving transformations with idempotency and determinism guarantees.

3. **Mutation Testing for Transpilers**: Novel mutation operators targeting AST transformations, type inference, and effect system correctness.

4. **PMAT-Integrated Workflow**: Demonstrates Toyota Way principles (Jidoka, Kaizen) in language tool development, achieving <10 CCN complexity and zero technical debt.

5. **Corpus-Driven Validation**: Empirical evaluation against 10k+ real-world bash scripts from GitHub, measuring parse success rate, semantic equivalence, and performance overhead.

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Test Coverage | ≥ 80% | make coverage (llvm-cov) |
| Mutation Score | ≥ 85% | cargo-mutants |
| Property Test Cases | 10k+ per property | proptest |
| Cyclomatic Complexity | < 10 per function | PMAT |
| SATD Comments | 0 | PMAT |
| Parse Success Rate | ≥ 95% | Corpus eval |
| Semantic Equivalence | ≥ 95% | Differential tests |
| False Positive Rate | ≤ 1% | Manual audit |
| Clippy Warnings | 0 | cargo clippy |

---

## References & Further Reading

This roadmap synthesizes multiple research threads:

- **Shell Verification**: Vasilakis et al. HotOS 2025, Mazurak & Zdancewic PLAS 2007
- **Compiler Correctness**: CrossLangFuzzer 2025, Chen et al. ISSTA 2016
- **Property Testing**: Hypothesis & QuickCheck
- **Mutation Testing**: Hariri et al. ISSRE 2016, Language Mutation Problem 2022
- **Quality Frameworks**: PMAT toolkit with Toyota Way principles

**Next Steps**: Begin Phase 1 implementation, validate with community review, iterate based on corpus evaluation results.
