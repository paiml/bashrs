# Corpus-Driven Transpilation Quality Specification

**Version**: 1.0.0
**Date**: 2026-02-06
**Status**: Draft
**Methodology**: EXTREME TDD + Popperian Falsification + Toyota Production System

## Executive Summary

This specification defines three corpus repositories in the `paiml` GitHub organization for measuring and improving bashrs transpilation quality across three target formats: Bash (purified POSIX shell), Makefiles, and Dockerfiles. Each corpus serves as a **falsifiable test oracle** -- a curated collection of Rust DSL inputs paired with expected outputs that enables continuous, automated measurement of transpilation correctness.

**Targets**:
- 99% transpilation success rate across all three formats
- 95% test coverage on transpiled outputs (Rust source is testable; outputs are unit-verifiable)
- Zero regression tolerance (Andon cord / STOP THE LINE on any decrease)

**Repositories**:

| Repository | Format | Initial Corpus Size | Target Rate |
|---|---|---|---|
| `paiml/bashrs-corpus-bash` | POSIX shell (purified) | 200 programs | 99% |
| `paiml/bashrs-corpus-makefile` | GNU Make | 150 programs | 99% |
| `paiml/bashrs-corpus-dockerfile` | Dockerfile | 150 programs | 99% |

---

## 1. Theoretical Foundation

### 1.1 Popperian Falsification Applied to Transpiler Validation

Karl Popper's critical rationalism holds that scientific theories cannot be verified, only falsified (Popper, 1959). Applied to transpiler engineering, this means:

> A transpiler is not "correct" because it passes N tests. It is **not yet falsified** because no test in the corpus has demonstrated incorrect behavior.

Each corpus entry is a **potential falsifier**: a specific input-output pair that could demonstrate transpilation failure. The corpus grows monotonically -- entries are never removed, only added. A 99% transpilation rate means that fewer than 1% of potential falsifiers have succeeded in demonstrating a defect.

**Falsification Protocol**:
1. Every corpus entry MUST have an expected output (the "prediction")
2. Every transpilation run produces an actual output (the "observation")
3. Any mismatch between prediction and observation is a **falsification event**
4. Falsification events trigger STOP THE LINE (see Section 5)

> "In so far as a scientific statement speaks about reality, it must be falsifiable; and in so far as it is not falsifiable, it does not speak about reality." -- Popper, K. (1959). *The Logic of Scientific Discovery*. Routledge, p. 314.

### 1.2 The Cardinal Rule: Fix the Transpiler, Never the Corpus

**THIS IS THE MOST IMPORTANT PRINCIPLE IN THIS ENTIRE SPECIFICATION.**

When a corpus entry fails, there are exactly two possible responses:

| Response | Correct? | Rationale |
|----------|----------|-----------|
| Fix the transpiler so the entry passes | **YES** | The corpus found a real defect. The transpiler is the system under test. |
| Modify or remove the corpus entry to hide the failure | **NEVER** | This is scientific fraud -- destroying evidence that falsifies your hypothesis. |

The corpus is the **test oracle**. It represents ground truth. The transpiler is the **system under test**. When the system fails the oracle, you fix the system.

**Why this matters**: The natural human temptation when a test fails is to "fix the test." In corpus-driven development, this impulse must be actively resisted. A failing corpus entry is not a bug in the test -- it is a **discovered defect** in the transpiler. It is a gift. It tells you exactly where to improve.

**Analogy**: In manufacturing, when a part fails quality inspection, you fix the manufacturing process, not the inspection gauge. Toyota calls this "respect for the process" (Liker, 2004, Principle 6).

**Enforcement**:
- Corpus entries are **append-only**. Entries are NEVER removed or weakened.
- The `convergence.log` records the corpus size monotonically increasing.
- Code review MUST reject any PR that modifies expected outputs to match transpiler bugs.
- CI MUST flag any reduction in corpus entry count as a P0 violation.

### 1.3 The Infinite Corpus: What Happens at 100%

Reaching 100% on the current corpus does **not** mean the transpiler is correct. It means the current set of falsifiers has been exhausted. The correct response is to **add harder entries**.

**The corpus growth cycle**:

```
    ┌─────────────────────────────────────────────────────────┐
    │                                                         │
    ▼                                                         │
 [Add new corpus entries]                                     │
    │                                                         │
    ▼                                                         │
 [Run corpus → measure rate]                                  │
    │                                                         │
    ├── Rate < 99% ──► [Fix transpiler] ──► [Run again] ──┘  │
    │                                                         │
    └── Rate = 100% ──► [Add HARDER entries] ─────────────────┘
```

**When you reach 100% on the current corpus**:
1. **Celebrate briefly** -- you've exhausted this level of difficulty
2. **Immediately add new entries** from the next tier or new edge cases
3. **Target constructs not yet covered**: new Rust syntax, deeper nesting, more complex patterns
4. **Mine real-world scripts** for patterns not yet in the corpus
5. **Run mutation testing** to find transpiler code paths not exercised by any entry
6. **Never declare victory** -- the corpus is a living document that grows forever

**The asymptotic model**: In practice, each round of "reach 100%, add harder entries" follows a sigmoid curve. The transpiler improves rapidly at first (low-hanging fruit), then improvements slow as edge cases get harder. This is expected and healthy -- it means the corpus is doing its job of pushing the transpiler toward correctness.

> "The strength of a theory lies not in its ability to avoid falsification, but in its ability to survive increasingly severe tests." -- Lakatos, I. (1978). *The Methodology of Scientific Research Programmes*. Cambridge University Press, p. 33.

**Corpus size targets over time**:

| Milestone | Corpus Size | Expected Rate | Action | Status |
|-----------|------------|---------------|--------|--------|
| Initial   | 30 entries | ~85% | Establish baseline, fix obvious gaps | DONE (iter 1-2) |
| Iteration 5 | 100 entries | ~92% | Expanding construct coverage | DONE (iter 5: 85/85, 100%) |
| Iteration 8 | 150 entries | ~95% | Production patterns added | DONE (iter 8: 150/150, 100%) |
| Iteration 11 | 250 entries | ~97% | Deeper edge cases | DONE (iter 11: 250/250, 100%, bug #7 fixed) |
| Iteration 13 | 330 entries | ~98% | Expansion waves 3-4 | DONE (iter 13: 330/330, 100%) |
| Iteration 14 | 500 entries | ~99% | Full corpus target reached | DONE (iter 14: 500/500, 100%, bug #8 fixed) |
| Ongoing | 500+ entries | 99%+ | Continuous addition of harder entries forever | ONGOING |

The corpus has no maximum size. If you run out of ideas for new entries, run mutation testing -- every surviving mutant reveals a corpus gap.

### 1.4 Toyota Production System: Jidoka and Kaizen

The Toyota Production System (TPS) provides two principles directly applicable to corpus-driven quality (see also Section 1.2 -- the cardinal rule ensures Jidoka is applied to the transpiler, not the corpus):

**Jidoka (Autonomation)**: Build quality into the process by stopping the line when a defect is detected (Liker, 2004). In our context:
- Every CI run executes the full corpus
- Any falsification event halts the pipeline (Andon cord)
- No release proceeds until the corpus passes at 99%+

**Kaizen (Continuous Improvement)**: Improvement through small, incremental changes measured against objective baselines (Imai, 1986). In our context:
- Transpilation rate is tracked per-iteration (convergence log)
- Each iteration adds corpus entries or fixes transpilation defects
- The corpus grows, making the quality bar strictly monotonically increasing

> "The Toyota Way is about processes and results... Test every process, improve every process, and involve every worker." -- Liker, J. K. (2004). *The Toyota Way: 14 Management Principles*. McGraw-Hill, p. 37.

### 1.5 Mutation Testing as Second-Order Falsification

Mutation testing (DeMillo et al., 1978) provides **second-order falsification**: it tests whether the tests themselves are adequate. A mutant that survives indicates a gap in the test oracle.

Applied to corpus validation:
- Inject mutations into the transpiler (cargo-mutants)
- If a mutant produces different output but no corpus entry catches it, the corpus has a gap
- Target: 90% mutation kill rate on transpiler code

> "Mutation testing provides a systematic approach to evaluating test suite adequacy by introducing small syntactic changes to source code." -- DeMillo, R. A., Lipton, R. J., & Sayward, F. G. (1978). "Hints on Test Data Selection: Help for the Practicing Programmer." *IEEE Computer*, 11(4), 34-41.

---

## 2. Corpus Architecture

### 2.1 Registry Schema

Each corpus repository follows a standardized structure inspired by depyler's corpus registry pattern:

```
paiml/bashrs-corpus-{format}/
├── Cargo.toml                    # Workspace for Rust DSL test crate
├── .pmat-gates.toml              # Quality gate thresholds
├── .pmat-metrics.toml            # Performance budgets
├── corpus/
│   ├── registry.toml             # Corpus metadata registry
│   ├── tier-1-trivial/           # Simple constructs (10-20 LOC)
│   │   ├── 001-hello-world/
│   │   │   ├── input.rs          # Rust DSL source
│   │   │   ├── expected.{sh,Makefile,Dockerfile}
│   │   │   ├── metadata.toml     # Entry metadata
│   │   │   └── test.rs           # Verification test
│   │   └── ...
│   ├── tier-2-standard/          # Common patterns (20-100 LOC)
│   ├── tier-3-complex/           # Real-world programs (100-500 LOC)
│   ├── tier-4-adversarial/       # Edge cases, injection attempts
│   └── tier-5-production/        # Full production scripts
├── src/
│   ├── lib.rs                    # Registry + runner
│   └── registry.rs               # CorpusEntry, CorpusRegistry
├── tests/
│   └── convergence_tests.rs      # Automated convergence measurement
└── convergence.log               # Historical transpilation rates
```

### 2.2 Registry Entry Metadata

```toml
# corpus/tier-1-trivial/001-hello-world/metadata.toml
[entry]
name = "hello-world"
tier = 1
description = "Simple echo statement"
added = "2026-02-06"
author = "bashrs-team"

[quality]
target_rate = 1.0       # Must always transpile
tdg_score = 9.5         # Target code quality
grade = "A+"
complexity = 1          # Cyclomatic complexity of input

[verification]
shellcheck = true       # Output must pass shellcheck (bash corpus)
deterministic = true    # Two runs produce identical output
idempotent = true       # Safe to execute twice
has_unit_test = true    # Rust-side unit test exists
```

### 2.3 Tier System

| Tier | Description | Count (Bash) | Count (Make) | Count (Docker) | Target Rate |
|------|-------------|-------------|-------------|----------------|-------------|
| 1 - Trivial | Single constructs: echo, let, if | 50 | 40 | 40 | 100% |
| 2 - Standard | Common patterns: loops, functions, pipes | 60 | 40 | 40 | 99% |
| 3 - Complex | Multi-function programs, error handling | 40 | 30 | 30 | 98% |
| 4 - Adversarial | Injection vectors, Unicode, edge cases | 30 | 25 | 25 | 95% |
| 5 - Production | Real-world scripts from open source | 20 | 15 | 15 | 95% |
| **Total** | | **200** | **150** | **150** | **99%** |

Tier assignment follows the **principle of progressive difficulty** (Vygotsky, 1978): each tier builds on constructs validated in the previous tier, creating a zone of proximal development for the transpiler.

---

## 3. Corpus Specifications by Format

### 3.1 Bash Corpus (`paiml/bashrs-corpus-bash`)

**Purpose**: Validate Rust DSL -> purified POSIX shell transpilation.

**Tier 1 - Trivial Constructs** (50 entries):

| ID | Construct | Rust DSL | Expected POSIX sh |
|----|-----------|----------|-------------------|
| B-001 | Variable assignment | `let x = "hello";` | `x='hello'` |
| B-002 | Echo | `println!("hello");` | `echo 'hello'` |
| B-003 | Integer arithmetic | `let x = 5 + 3;` | `x=$((5 + 3))` |
| B-004 | If statement | `if x > 0 { ... }` | `if [ "$x" -gt 0 ]; then ... fi` |
| B-005 | For loop | `for i in 1..5 { ... }` | `for i in 1 2 3 4; do ... done` |
| ... | ... | ... | ... |
| B-050 | Exit code | `std::process::exit(1);` | `exit 1` |

**Tier 2 - Standard Patterns** (60 entries):

| ID | Pattern | Description |
|----|---------|-------------|
| B-051 | Function definition | Named functions with arguments |
| B-052 | Command substitution | `$(command)` patterns |
| B-053 | Pipe chains | Multi-stage pipelines |
| B-054 | File operations | `fs::read`, `fs::write` -> safe shell equivalents |
| B-055 | Error handling | `Result<T>` -> `|| { echo "error"; exit 1; }` |
| ... | ... | ... |
| B-110 | Complex pipe | 5+ stage pipeline with error propagation |

**Verification Requirements**:
- All outputs pass `shellcheck -s sh` (POSIX compliance)
- All outputs are deterministic (no `$RANDOM`, `$$`, timestamps)
- All outputs are idempotent (mkdir -p, rm -f, ln -sf)
- All variables quoted (injection prevention)

### 3.2 Makefile Corpus (`paiml/bashrs-corpus-makefile`)

**Purpose**: Validate Rust DSL -> GNU Makefile transpilation.

**Tier 1 - Trivial Constructs** (40 entries):

| ID | Construct | Rust DSL | Expected Makefile |
|----|-----------|----------|-------------------|
| M-001 | Variable | `let cc = "gcc";` | `CC := gcc` |
| M-002 | Multiple vars | `let cflags = "-O2 -Wall";` | `CFLAGS := -O2 -Wall` |
| M-003 | Simple target | `target("all", &["main.o"], &["$(CC) -o main main.o"]);` | `all: main.o\n\t$(CC) -o main main.o` |
| M-004 | Phony target | `phony_target("clean", &[], &["rm -f *.o"]);` | `.PHONY: clean\nclean:\n\trm -f *.o` |
| M-005 | Default goal | First target is default | `.DEFAULT_GOAL := all` |
| ... | ... | ... | ... |
| M-040 | Pattern rule | `%.o: %.c` pattern | Pattern rules with automatic variables |

**Tier 2 - Standard Patterns** (40 entries):

| ID | Pattern | Description |
|----|---------|-------------|
| M-041 | Multi-target | Multiple targets with shared prerequisites |
| M-042 | Conditional | `ifeq`/`ifdef` blocks from Rust conditionals |
| M-043 | Include | `include` directives |
| M-044 | Functions | `$(wildcard ...)`, `$(patsubst ...)` |
| M-045 | Recursive make | `$(MAKE) -C subdir` |
| ... | ... | ... |
| M-080 | Full C project | Complete build system with install/uninstall |

**Verification Requirements**:
- All outputs pass `bashrs make lint` (MAKE001-MAKE020 rules)
- Variables are uppercase (MAKE convention)
- Targets use `:=` (simply-expanded, deterministic)
- Tab characters used for recipes (GNU Make requirement)
- Phony targets declared with `.PHONY`

### 3.3 Dockerfile Corpus (`paiml/bashrs-corpus-dockerfile`)

**Purpose**: Validate Rust DSL -> Dockerfile transpilation.

**Tier 1 - Trivial Constructs** (40 entries):

| ID | Construct | Rust DSL | Expected Dockerfile |
|----|-----------|----------|---------------------|
| D-001 | FROM | `from_image("alpine", "3.18");` | `FROM alpine:3.18` |
| D-002 | WORKDIR | `workdir("/app");` | `WORKDIR /app` |
| D-003 | COPY | `copy(".", ".");` | `COPY . .` |
| D-004 | RUN | `run(&["apk add curl"]);` | `RUN apk add curl` |
| D-005 | USER | `user("65534");` | `USER 65534` |
| ... | ... | ... | ... |
| D-040 | HEALTHCHECK | `healthcheck("CMD curl -f http://localhost/");` | `HEALTHCHECK CMD curl -f http://localhost/` |

**Tier 2 - Standard Patterns** (40 entries):

| ID | Pattern | Description |
|----|---------|-------------|
| D-041 | Multi-stage | Builder + runtime stages |
| D-042 | RUN chaining | `&&` chaining with layer optimization |
| D-043 | ARG + ENV | Build args and environment variables |
| D-044 | COPY --from | Cross-stage copy |
| D-045 | ENTRYPOINT + CMD | Exec form with default args |
| ... | ... | ... |
| D-080 | Production Rust | Multi-stage Rust build with musl |

**Verification Requirements**:
- All outputs pass `bashrs dockerfile lint` (DOCKER001-DOCKER012 rules)
- No `:latest` tags (DOCKER002: pinned versions)
- USER directive present (DOCKER003: non-root)
- Minimal layers (RUN commands chained with `&&`)
- Exec form for ENTRYPOINT/CMD (no shell form)

---

## 4. Scoring System

### 4.1 100-Point Transpilation Quality Score

Adapted from depyler's Pareto single-shot scoring methodology (Gift, 2025):

| Category | Points | Weight | Description |
|----------|--------|--------|-------------|
| A. Transpilation Success | 40 | 40% | Does the input transpile without error? |
| B. Output Correctness | 25 | 25% | Does output match expected semantics? |
| C. Test Coverage | 15 | 15% | Are transpiled outputs verified by tests? |
| D. Lint Compliance | 10 | 10% | Does output pass format-specific linting? |
| E. Determinism | 10 | 10% | Is output byte-identical across runs? |

**Scoring Formula**:

```
Score = (A_success_ratio × 40)
      + (B_correct_ratio × 25)
      + (C_coverage_ratio × 15)
      + (D_lint_pass_ratio × 10)
      + (E_determinism_ratio × 10)
```

**Gateway Logic** (Popperian falsification barrier):
- If A < 24 (60% transpilation), B through E are scored as 0
- Rationale: A transpiler that fails to produce output cannot have correct, tested, or lint-clean output

**Grade Scale**:

| Grade | Score Range | Interpretation |
|-------|------------|----------------|
| A+ | 97-100 | Production-ready, fully validated |
| A | 90-96 | Near-production, minor gaps |
| B | 80-89 | Good quality, known limitations |
| C | 70-79 | Functional, significant gaps |
| D | 60-69 | Partially functional |
| F | < 60 | Not yet viable |

**Target**: Grade A+ (97+) for all three corpus repositories.

### 4.2 Per-Entry Scoring

Each corpus entry receives an individual score:

```toml
# Automated scoring output
[score]
transpiles = true         # +40 (A: success)
output_correct = true     # +25 (B: correctness)
has_test = true           # +15 (C: coverage)
lint_clean = true         # +10 (D: lint)
deterministic = true      # +10 (E: determinism)
total = 100               # Sum
grade = "A+"
```

### 4.3 Aggregate Scoring

The repository-level score is the weighted mean of all entry scores:

```
Repo_Score = Σ(entry_score × tier_weight) / Σ(tier_weight)
```

Where tier weights reflect difficulty:
- Tier 1: weight 1.0
- Tier 2: weight 1.5
- Tier 3: weight 2.0
- Tier 4: weight 2.5
- Tier 5: weight 3.0

This weighting ensures that production-quality programs contribute more to the overall score, following the Pareto principle: the hardest 20% of entries provide 40% of the quality signal (Juran, 1951).

---

## 5. Convergence Tracking and Kaizen Protocol

### 5.1 Convergence Log

Each corpus repository maintains a `convergence.log` tracking transpilation rate over iterations:

```
# convergence.log (ACTUAL DATA - updated 2026-02-06)
# iter | date       | total | pass | fail | rate   | delta  | score | grade | notes
  1    | 2026-02-06 |   30  |  26  |   4  | 86.7%  | +86.7  | ~85   | B     | Initial Tier 1: 4 falsifiers (D-006 u16, D-007/M-003/M-004 array refs)
  2    | 2026-02-06 |   30  |  30  |   0  | 100.0% | +13.3  | 99.2  | A+    | Fixed: u16 type, array/slice refs, reference exprs
  3    | 2026-02-06 |   55  |  54  |   1  | 98.2%  | -1.8   | ~98   | A+    | Tier 2 added: 1 falsifier (B-016 assignment expr)
  4    | 2026-02-06 |   55  |  55  |   0  | 100.0% | +1.8   | 99.5  | A+    | Fixed: SynExpr::Assign handler
  5    | 2026-02-06 |   85  |  85  |   0  | 100.0% |  0.0   | 99.1  | A+    | Tier 3 added: no falsifiers (sawtooth didn't dip)
  6    | 2026-02-06 |  110  | 101  |   9  | 91.8%  | -8.2   | 90.8  | A     | Tier 4 adversarial: 9 falsifiers (+=/-=/*=, eprintln!, target() arity)
  7    | 2026-02-06 |  110  | 110  |   0  | 100.0% | +8.2   | 99.0  | A+    | Fixed: compound assign, eprintln!, 2-arg target()
  8    | 2026-02-06 |  150  | 150  |   0  | 100.0% |  0.0   | 99.3  | A+    | Tier 5 production: no falsifiers (40 new entries)
  9    | 2026-02-06 |  200  | 200  |   0  | 100.0% |  0.0   | 99.5  | A+    | Expansion 1: 50 more entries, no falsifiers
  10   | 2026-02-06 |  250  | 249  |   1  | 99.6%  | -0.4   | 99.1  | A+    | Expansion 2: B-121 falsifier (CommandSubst in arithmetic)
  11   | 2026-02-06 |  250  | 250  |   0  | 100.0% | +0.4   | 99.5  | A+    | Fixed: emit_arithmetic_operand handles CommandSubst
  12   | 2026-02-06 |  290  | 290  |   0  | 100.0% |  0.0   | 99.6  | A+    | Expansion 3+4: 80 more entries, no falsifiers
  13   | 2026-02-06 |  330  | 330  |   0  | 100.0% |  0.0   | 99.6  | A+    | Expansion 4 confirmed: 330 entries, zero falsifiers
  14   | 2026-02-06 |  500  | 499  |   1  | 99.8%  | -0.2   | 99.5  | A+    | Expansion 5-7: B-171 falsifier (format! macro expr)
  15   | 2026-02-06 |  500  | 500  |   0  | 100.0% | +0.2   | 99.7  | A+    | Fixed: SynExpr::Macro handler for format!/vec! macros
```

**Final Corpus Composition**:
- **Bash**: 200 entries (B-001..B-200) — target: 200 ✅
- **Makefile**: 150 entries (M-001..M-150) — target: 150 ✅
- **Dockerfile**: 150 entries (D-001..D-150) — target: 150 ✅
- **Total**: 500 entries — target: 500 ✅

**Bugs Fixed (Transpiler Improvements)**:
1. **u16 type support** (D-006): Added `Type::U16`, `Literal::U16(u16)` to AST, parser, IR, all emitters
2. **Array/slice reference expressions** (D-007, M-003, M-004): Added `SynExpr::Array`, `SynExpr::Reference`, `SynType::Slice` handlers
3. **Assignment expressions** (B-016): Added `SynExpr::Assign` → `convert_assign_stmt()` in parser
4. **Compound assignment operators** (B-036/B-037/B-038): Desugar `+=`, `-=`, `*=`, `/=`, `%=` to binary expressions
5. **eprintln! macro** (B-039): Parser + `rash_eprintln` runtime function with `>&2` redirect
6. **2-arg target()** (M-026/M-027/M-028/M-029): Makefile `target()/phony_target()` now accept 2 or 3 args
7. **CommandSubst in arithmetic** (B-121): `emit_arithmetic_operand` now handles `ShellValue::CommandSubst` for function return values in `$((...))` expressions
8. **format! macro expression** (B-171): Added `SynExpr::Macro` handler in `convert_expr()` for `format!` and `vec!` macro expressions

### 5.2 Convergence Criteria

The transpiler is considered **converged at a given corpus level** when:

1. **Rate threshold**: Transpilation rate >= 99% for 3 consecutive iterations
2. **Stability**: Delta < 0.5% for 3 consecutive iterations (approaching asymptote)
3. **Corpus growth**: Corpus size >= initial target (200/150/150)
4. **No regressions**: No entry that previously passed has started failing

**CRITICAL: Convergence is temporary.** When convergence is reached, the corpus MUST be expanded with harder entries (see Section 1.3). Convergence at N entries triggers growth to N+50 entries. There is no final convergence -- only convergence at the current difficulty level.

This follows the statistical process control methodology of Shewhart (1931): a process is "in control" when variation falls within expected bounds over sustained measurement. But a controlled process operating within limits should be challenged with tighter limits.

> "A phenomenon will be said to be controlled when, through the use of past experience, we can predict, at least within limits, how the phenomenon may be expected to vary in the future." -- Shewhart, W. A. (1931). *Economic Control of Quality of Manufactured Product*. Van Nostrand, p. 6.

### 5.3 Regression Detection (Jidoka)

**Andon Cord Protocol**:

When CI detects a regression (an entry that previously passed now fails):

1. **STOP THE LINE**: Pipeline fails, no releases proceed
2. **Root cause analysis**: Five Whys applied to the regression
3. **Fix with EXTREME TDD**: RED -> GREEN -> REFACTOR cycle
4. **Regression test**: The failing entry becomes a permanent regression test
5. **Resume**: Only after full convergence suite passes

This implements Toyota's Jidoka principle: "stop and fix problems as they occur rather than pushing them down the line" (Ohno, 1988).

> "If a defective part or equipment malfunction is discovered, the affected machine automatically stops, and operators stop work and correct the problem." -- Ohno, T. (1988). *Toyota Production System: Beyond Large-Scale Production*. Productivity Press, p. 6.

---

## 6. Test Coverage Strategy

### 6.1 Dual-Layer Testing

The 95% coverage target is achieved through two complementary testing layers:

**Layer 1: Rust-Side Unit Tests (the Rust DSL source is testable)**

```rust
#[test]
fn test_corpus_B001_hello_world() {
    let rust_input = r#"fn main() { println!("hello"); }"#;
    let config = Config::default();
    let output = bashrs::transpile(rust_input, config).unwrap();

    assert!(output.contains("echo 'hello'"));
    assert!(!output.contains("$RANDOM"));  // Determinism
    assert!(!output.contains(":latest"));   // No latest tags (Docker)
}
```

**Layer 2: Output Verification Tests (the transpiled output is verifiable)**

```rust
#[test]
fn test_corpus_B001_output_quality() {
    let output = transpile_corpus_entry("tier-1-trivial/001-hello-world");

    // Structural verification
    assert!(output.starts_with("#!/bin/sh"));
    assert!(output.contains("set -euf"));

    // Lint verification
    let lint = bashrs::lint_shell(&output);
    assert_eq!(lint.errors.len(), 0, "No SEC/DET/IDEM violations");

    // Determinism verification
    let output2 = transpile_corpus_entry("tier-1-trivial/001-hello-world");
    assert_eq!(output, output2, "Transpilation must be deterministic");
}
```

### 6.2 Coverage Measurement

```bash
# Measure coverage of corpus test suite
cargo llvm-cov --package bashrs-corpus-bash --lcov --output-path lcov.info

# Target: 95% line coverage across:
# - Transpiler code exercised by corpus
# - Output verification tests
# - Registry and runner infrastructure
```

### 6.3 Property-Based Testing

Each tier includes property tests that generate random valid inputs within the tier's construct space:

```rust
proptest! {
    #[test]
    fn prop_tier1_always_transpiles(
        var_name in "[a-z][a-z0-9_]{0,10}",
        value in "[a-zA-Z0-9 ]{1,50}"
    ) {
        let input = format!(r#"fn main() {{ let {var_name} = "{value}"; }}"#);
        let result = bashrs::transpile(&input, Config::default());
        prop_assert!(result.is_ok(), "Tier 1 constructs must always transpile");
    }
}
```

### 6.4 Mutation Testing as Test Quality Validation

Following DeMillo et al. (1978), mutation testing validates that the corpus tests are meaningful:

```bash
# Run mutation testing on transpiler code
cargo mutants --file rash/src/emitter/posix.rs -- --test corpus

# Target: >=90% mutation kill rate
# Interpretation: 90% of transpiler mutations are caught by corpus tests
```

A surviving mutant indicates either:
1. A gap in the corpus (add a new entry targeting the uncaught mutation)
2. A redundancy in the transpiler (dead code that can be removed)

---

## 7. Compiler-in-the-Loop (CITL) Integration

### 7.1 What is CITL for bashrs?

Compiler-in-the-Loop (CITL) is a pattern from the depyler project (Gift, 2025) where the **compiler serves as an automated oracle** on every commit. In depyler, `rustc` is the compiler. In bashrs, **the bashrs linter IS the compiler**:

| Format | CITL "Compiler" | Rules Applied |
|--------|-----------------|---------------|
| Bash (POSIX shell) | `bashrs::linter::rules::lint_shell()` | SEC001-SEC008, DET001-DET003, IDEM001-IDEM003 |
| Makefile | `bashrs::linter::rules::lint_makefile()` | MAKE001-MAKE020 |
| Dockerfile | `bashrs::linter::rules::lint_dockerfile()` | DOCKER001-DOCKER012 |

We already have the compiler. The corpus runner already calls it (the "D: Lint Compliance" score). The unit tests on the transpiled output already close the loop. **CITL is not an external tool -- it is the combination of transpilation + linting + unit testing that already runs on every corpus entry.**

The key insight from depyler: the loop must run **on every commit**, failures must **block the commit**, and compiler errors must **generate new corpus entries**.

### 7.2 The CITL Loop: Every Commit, Every Entry

```
┌──────────────────────────────────────────────────────────────┐
│                    EVERY COMMIT                               │
│                                                               │
│  1. Transpile all corpus entries (Rust DSL → Bash/Make/Docker)│
│     │                                                         │
│  2. For each transpiled output, run THREE validators:         │
│     ├── Unit test: does output contain expected content?      │
│     ├── Lint (CITL): lint_shell / lint_makefile /             │
│     │   lint_dockerfile on the actual transpiled output       │
│     └── Determinism: transpile twice, byte-compare            │
│     │                                                         │
│  3. Score each entry (100-point system) and aggregate         │
│     │                                                         │
│  4. If any previously-passing entry now fails:                │
│     └── ANDON CORD → fix the TRANSPILER (Section 1.2)        │
│                                                               │
│  5. If rate = 100% on current corpus:                         │
│     └── ADD HARDER ENTRIES (Section 1.3)                      │
│                                                               │
│  6. Lint violations on transpiled output become NEW entries:  │
│     └── Violation → new corpus entry targeting that defect    │
└──────────────────────────────────────────────────────────────┘
```

### 7.3 Lint Violation → Corpus Entry Pipeline (Self-Improving Corpus)

When the bashrs linter flags a violation in transpiled output, that violation becomes a **new corpus entry**:

```
lint_shell(transpiled_output):
  SEC003: Unquoted variable in command at line 5

  → New corpus entry:
    id: "B-031"
    name: "unquoted-variable-in-command"
    description: "SEC003: variable used in command argument must be quoted"
    input: <the Rust DSL that produced the bad output>
    expected_output: <corrected output with proper quoting>
    lint_rule: "SEC003"
```

This creates a **self-improving cycle**: lint violations from CITL validation automatically generate new corpus entries, which drive transpiler fixes, which improve the rate. The corpus grows itself from linter feedback. This is the same pattern depyler uses with `rustc` errors, but our "compiler" is the bashrs linter.

### 7.4 Pre-Commit Hook Integration

Following the depyler pattern, corpus validation runs on every commit via pmat-managed hooks:

```bash
# .git/hooks/pre-commit (pmat-managed)
#!/bin/sh
set -euf

# Run corpus unit tests (<30s)
cargo test -p bashrs --lib -- corpus --quiet

# Full corpus integration tests on CI
# cargo test -p bashrs --test corpus_tests
```

On CI (GitHub Actions), the full corpus runs:

```yaml
- name: CITL Corpus Validation
  run: cargo test -p bashrs --test corpus_tests
```

### 7.5 Convergence Log Tracks Lint Pass Rate

The convergence log tracks the CITL (lint) pass rate alongside transpilation rate. The gap between them reveals "hidden invalidity" -- output that transpiles but violates lint rules:

```
# convergence.log
# iter | date       | total | transpile | lint_pass | rate   | lint_rate | notes
  1    | 2026-02-06 | 30    | 26        | 22        | 86.7%  | 73.3%    | Baseline: 4 AST gaps, 4 lint violations
  2    | 2026-02-13 | 30    | 30        | 28        | 100%   | 93.3%    | Fixed AST, 2 SEC rule violations remain
  3    | 2026-02-20 | 50    | 46        | 42        | 92.0%  | 84.0%    | Added 20 harder entries, rate dipped (healthy)
  4    | 2026-02-27 | 50    | 50        | 49        | 100%   | 98.0%    | Recovered, one DOCKER003 violation
  5    | 2026-03-06 | 80    | 76        | 72        | 95.0%  | 90.0%    | Added 30 more entries (Section 1.3)
```

---

## 8. Implementation Phases (Fix the Transpiler, Grow the Corpus)

### Phase 1: Infrastructure and Tier 1 Corpus (Weeks 1-3)

**Objective**: Establish repository structure, build runner infrastructure, populate Tier 1 entries.

**Deliverables**:
- Three GitHub repositories created with standardized structure
- `CorpusEntry` and `CorpusRegistry` types implemented
- Automated runner: `cargo test` transpiles all entries and compares output
- Convergence logging infrastructure
- 50 Bash + 40 Makefile + 40 Dockerfile Tier 1 entries
- CI integration (GitHub Actions)

**Falsification Checklist** (Popper):
- [ ] Can a syntactically valid Rust DSL input fail to transpile? (Expected: no for Tier 1)
- [ ] Can transpilation produce output that differs between runs? (Expected: no)
- [ ] Can transpiled Bash output fail shellcheck? (Expected: no for Tier 1)
- [ ] Can transpiled Makefile output violate MAKE001-MAKE020? (Expected: no for Tier 1)
- [ ] Can transpiled Dockerfile output violate DOCKER001-DOCKER012? (Expected: no for Tier 1)

**Quality Gates**:
- Tier 1 transpilation rate: 100%
- Test coverage: >= 90%
- Mutation kill rate: >= 80%

**Citations**:
- Repository structure follows depyler corpus pattern (Gift, 2025)
- Test naming: `test_<CORPUS_ID>_<feature>_<scenario>` per CLAUDE.md
- Jidoka: CI pipeline halts on any Tier 1 failure (Ohno, 1988)

### Phase 2: Tier 2-3 Population and Convergence (Weeks 4-8)

**Objective**: Add standard and complex constructs, drive transpilation rate to 95%+.

**Deliverables**:
- 60 Bash + 40 Makefile + 40 Dockerfile Tier 2 entries
- 40 Bash + 30 Makefile + 30 Dockerfile Tier 3 entries
- Convergence log showing monotonic improvement
- Transpiler fixes for failing entries (EXTREME TDD cycle per fix)
- Property tests for each tier

**Falsification Checklist** (Popper):
- [ ] Can a pipe chain with 5+ stages fail to transpile correctly? (Test it)
- [ ] Can a multi-stage Docker build lose cross-stage references? (Test it)
- [ ] Can a Makefile with pattern rules produce invalid syntax? (Test it)
- [ ] Can error handling in Rust DSL produce shell scripts that silently ignore errors? (Test it)
- [ ] Can transpiled functions have name collisions with POSIX builtins? (Test it)

**Quality Gates**:
- Overall transpilation rate: >= 95%
- No Tier 1 regressions (Jidoka)
- Test coverage: >= 93%
- Mutation kill rate: >= 85%
- Convergence delta trending toward 0 (Kaizen)

**Citations**:
- Progressive difficulty follows zone of proximal development (Vygotsky, 1978)
- Monotonic improvement tracking follows Kaizen methodology (Imai, 1986)
- Statistical process control for convergence detection (Shewhart, 1931)

### Phase 3: Adversarial and Production Corpus (Weeks 9-12)

**Objective**: Add adversarial edge cases and production scripts, reach 99% target.

**Deliverables**:
- 30 Bash + 25 Makefile + 25 Dockerfile Tier 4 (adversarial) entries
- 20 Bash + 15 Makefile + 15 Dockerfile Tier 5 (production) entries
- Security audit of transpiled outputs (no injection vectors)
- Full mutation testing pass (>= 90% kill rate)
- Convergence log showing 99%+ rate for 3+ iterations

**Adversarial Entry Categories**:

| Category | Examples | Purpose |
|----------|----------|---------|
| Injection | `"; rm -rf /`, `$({malicious})` | Verify escaping |
| Unicode | Bidi overrides, zero-width chars, emoji | Verify ASCII-safe output |
| Boundary | Empty strings, max-length args, null bytes | Stress edge cases |
| Ambiguity | Reserved words as identifiers, nested quotes | Verify disambiguation |
| Resource | Deep nesting, wide fan-out, large literals | Verify bounded output |

**Falsification Checklist** (Popper):
- [ ] Can any adversarial input produce shell injection in output? (MUST be false)
- [ ] Can Unicode bidi overrides in input survive to output? (MUST be false)
- [ ] Can a production-scale script exceed 10MB transpiled output? (MUST be false)
- [ ] Can any transpiled Dockerfile use `:latest` tag? (MUST be false)
- [ ] Can any transpiled Makefile use recursively-expanded `=` instead of `:=`? (Test it)

**Quality Gates**:
- Overall transpilation rate: >= 99% (target achieved)
- Test coverage: >= 95% (target achieved)
- Mutation kill rate: >= 90%
- Zero security violations in transpiled output
- Convergence stable (delta < 0.5% for 3 iterations)

**Citations**:
- Adversarial testing follows fuzzing methodology (Miller et al., 1990)
- Security verification follows OWASP testing guide (OWASP, 2023)
- Mutation testing adequacy criterion (DeMillo et al., 1978)

### Phase 4: Continuous Growth and Perpetual Falsification (Ongoing -- Never Ends)

**Objective**: The corpus never stops growing. When 100% is reached, add harder entries until the rate drops, then fix the transpiler again. Repeat forever.

**The cardinal rule applies here most urgently** (Section 1.2): the temptation to "declare victory" and stop adding entries is the single greatest risk to long-term quality. A static corpus decays into a regression suite -- necessary, but insufficient.

**Deliverables**:
- Automated corpus contribution pipeline (PR template for new entries)
- Monthly convergence report showing corpus SIZE growth (not just rate)
- Quarterly adversarial audit (new injection patterns, new CVEs)
- Mutation-testing-guided corpus expansion: every surviving mutant becomes a new entry
- Integration with pmat quality scoring
- **Minimum 10 new entries per month** (enforced by CI)

**Kaizen Cycle** (Toyota PDCA applied to corpus growth):
1. **Plan**: Run mutation testing to find untested transpiler code paths
2. **Do**: Write corpus entries targeting those paths (they WILL fail initially)
3. **Check**: Confirm the new entries fail (if they pass, the entry is too easy -- write harder ones)
4. **Act**: Fix the transpiler to pass the new entries, then go back to Plan

**The healthy cadence**:
- Rate drops when new entries are added (this is GOOD -- it means the corpus is challenging)
- Rate recovers as transpiler improves (this is the Kaizen improvement)
- Rate reaches 100% again (this means it's time for more entries)
- This cycle repeats indefinitely

**Citations**:
- PDCA cycle (Deming, 1986)
- Continuous improvement in manufacturing quality (Imai, 1986)
- Statistical process control for ongoing monitoring (Shewhart, 1931)
- "A static test suite is a dead test suite" -- adapted from Beck, K. (2002). *Test-Driven Development: By Example*. Addison-Wesley.

---

## 9. Quality Gate Configuration

### 8.1 `.pmat-gates.toml`

```toml
[quality]
min_coverage = 95.0
max_complexity = 10
max_cognitive_complexity = 15
min_tdg_score = 9.0

[gates]
block_on_coverage_drop = true
block_on_complexity_violation = true
block_on_satd = false
block_on_regression = true

[thresholds]
max_file_lines = 500
max_function_lines = 50
max_parameters = 5

[enforcement]
level = "error"     # "warn", "error", or "block"
```

### 8.2 `.pmat-metrics.toml`

```toml
[thresholds]
lint_ms = 5000
test_ms = 60000
coverage_ms = 120000
binary_size_kb = 10240

[staleness]
max_age_days = 7

[enforcement]
fail_on_stale = true
fail_on_performance_regression = true

[trend_analysis]
enabled = true
retention_days = 90

[quality_gates]
min_coverage = 95.0
min_mutation_score = 90.0
min_tdg_grade = "A"

[performance]
max_transpile_ms_per_entry = 100
max_memory_mb_per_entry = 10
```

---

## 10. CI/CD Integration

### 10.1 GitHub Actions Workflow

```yaml
# .github/workflows/corpus.yml
name: Corpus Validation
on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        corpus: [bash, makefile, dockerfile]
    steps:
      - uses: actions/checkout@v4
      - name: Run corpus tests (CITL loop)
        run: cargo test -p bashrs --test corpus_tests
      - name: Run lib corpus tests
        run: cargo test -p bashrs --lib -- corpus
      - name: Check convergence
        run: |
          RATE=$(cargo test -p bashrs --test corpus_tests -- --nocapture 2>&1 | grep "Rate:" | awk '{print $2}')
          echo "Transpilation rate: $RATE"
      - name: Update convergence log
        if: github.ref == 'refs/heads/main'
        run: cargo test -p bashrs --test corpus_tests -- --nocapture 2>&1 | tee convergence_output.txt
```

### 10.2 Andon Cord Integration

Any CI failure on the corpus triggers:
1. GitHub check fails (blocks merge)
2. Notification to maintainers
3. Issue auto-created with failing entry details
4. Release pipeline halted until resolution

### 10.3 Hugging Face Dataset Publishing

The corpus and convergence metrics are published to Hugging Face as open datasets on every release. This serves three purposes:
1. **Reproducibility**: Anyone can download and re-run the corpus against any bashrs version
2. **Training data**: The input/output pairs serve as training data for code generation models
3. **Benchmarking**: Other transpiler projects can compare against the bashrs corpus

**Hugging Face Repositories**:

| HF Dataset | Contents | Update Frequency |
|------------|----------|------------------|
| `paiml/bashrs-corpus-bash` | Rust DSL → POSIX shell pairs + scores | Every release + weekly snapshot |
| `paiml/bashrs-corpus-makefile` | Rust DSL → Makefile pairs + scores | Every release + weekly snapshot |
| `paiml/bashrs-corpus-dockerfile` | Rust DSL → Dockerfile pairs + scores | Every release + weekly snapshot |
| `paiml/bashrs-convergence` | Historical convergence logs, iteration metrics, scoring trends | Every commit to main |

**Dataset Schema** (Apache Parquet format):

```
corpus_entry.parquet:
  - id: string              # "B-001", "M-042", "D-015"
  - name: string            # "variable-assignment"
  - tier: int32             # 1-5
  - format: string          # "bash", "makefile", "dockerfile"
  - input_rust: string      # Rust DSL source code
  - expected_output: string # Ground truth expected output
  - actual_output: string   # What the transpiler actually produced
  - transpiled: bool        # Did it transpile without error?
  - output_correct: bool    # Does output match expected?
  - lint_clean: bool        # Does output pass linter (CITL)?
  - deterministic: bool     # Is output identical across runs?
  - score: float64          # 0-100 per-entry score
  - grade: string           # "A+", "A", "B", "C", "D", "F"
  - bashrs_version: string  # "6.59.0"
  - commit_sha: string      # Git commit that generated this result
  - date: string            # ISO 8601 date
```

**Publishing Workflow** (GitHub Actions):

```yaml
# .github/workflows/publish-corpus.yml
name: Publish Corpus to Hugging Face
on:
  push:
    branches: [main]
  release:
    types: [published]

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run corpus and generate parquet
        run: cargo test -p bashrs --test corpus_tests -- --nocapture
      - name: Export results to parquet
        run: cargo run --bin corpus-export -- --format parquet --output corpus_results.parquet
      - name: Push to Hugging Face
        env:
          HF_TOKEN: ${{ secrets.HF_TOKEN }}
        run: |
          pip install huggingface_hub
          python -c "
          from huggingface_hub import HfApi
          api = HfApi()
          api.upload_file(
              path_or_fileobj='corpus_results.parquet',
              path_in_repo='data/corpus_results.parquet',
              repo_id='paiml/bashrs-corpus-bash',
              repo_type='dataset',
              token='$HF_TOKEN'
          )
          "
```

**Model Publishing** (Oracle/CITL models):

When the bashrs oracle or CITL pattern library is retrained from corpus data, the updated model is also pushed to Hugging Face:

| HF Model | Contents | Update Frequency |
|----------|----------|------------------|
| `paiml/bashrs-oracle` | Error classification model trained on corpus failures | Monthly or on significant corpus growth |
| `paiml/bashrs-citl-patterns` | Lint violation → fix pattern library (BM25 index) | Weekly with corpus updates |

This follows the depyler pattern where the `depyler_oracle.apr` model is retrained after each overnight session and published for reproducibility.

**Benefits of Hugging Face Publishing**:
- **Open science**: Corpus is publicly available for peer review (Popperian transparency)
- **Version tracking**: Every dataset version is immutable and linked to a git commit
- **Training signal**: The input/output/score triples are directly usable as fine-tuning data
- **Community growth**: External contributors can propose new corpus entries via HF discussions

---

## 11. References

### Peer-Reviewed and Foundational

1. **DeMillo, R. A., Lipton, R. J., & Sayward, F. G.** (1978). "Hints on Test Data Selection: Help for the Practicing Programmer." *IEEE Computer*, 11(4), 34-41. DOI: 10.1109/C-M.1978.218136

2. **Deming, W. E.** (1986). *Out of the Crisis*. MIT Press. ISBN: 978-0262541152

3. **Imai, M.** (1986). *Kaizen: The Key to Japan's Competitive Success*. McGraw-Hill. ISBN: 978-0075543329

4. **Juran, J. M.** (1951). *Quality Control Handbook*. McGraw-Hill. (Source of the Pareto principle in quality management.)

5. **Liker, J. K.** (2004). *The Toyota Way: 14 Management Principles from the World's Greatest Manufacturer*. McGraw-Hill. ISBN: 978-0071392310

6. **Miller, B. P., Fredriksen, L., & So, B.** (1990). "An Empirical Study of the Reliability of UNIX Utilities." *Communications of the ACM*, 33(12), 32-44. DOI: 10.1145/96267.96279

7. **Ohno, T.** (1988). *Toyota Production System: Beyond Large-Scale Production*. Productivity Press. ISBN: 978-0915299140

8. **OWASP Foundation.** (2023). *OWASP Testing Guide v4.2*. https://owasp.org/www-project-web-security-testing-guide/

9. **Popper, K.** (1959). *The Logic of Scientific Discovery*. Routledge. ISBN: 978-0415278447

10. **Shewhart, W. A.** (1931). *Economic Control of Quality of Manufactured Product*. Van Nostrand. ISBN: 978-0873890762

11. **Vygotsky, L. S.** (1978). *Mind in Society: The Development of Higher Psychological Processes*. Harvard University Press. ISBN: 978-0674576292

12. **Lakatos, I.** (1978). *The Methodology of Scientific Research Programmes*. Cambridge University Press. ISBN: 978-0521280310. (Progressive falsification through increasingly severe tests.)

13. **Beck, K.** (2002). *Test-Driven Development: By Example*. Addison-Wesley. ISBN: 978-0321146533. (Test-first development; static test suites as a quality anti-pattern.)

### Project-Specific

14. **Gift, N.** (2025). "Depyler Corpus Registry and Convergence Methodology." Internal specification, paiml/depyler. (Corpus registry pattern, 100-point scoring system, multi-tier measurement.)

15. **bashrs CLAUDE.md** (2024-2026). Project development guidelines. (EXTREME TDD, STOP THE LINE, assert_cmd mandate, unwrap policy.)

---

## Appendix A: Falsification Summary Matrix

| Phase | Hypothesis | Falsification Test | Expected Result |
|-------|-----------|-------------------|-----------------|
| 1 | Tier 1 always transpiles | Run all 130 Tier 1 entries | 100% pass |
| 1 | Output is deterministic | Transpile each entry twice, byte-compare | Identical |
| 1 | Bash output passes shellcheck | `shellcheck -s sh` on all Bash outputs | Zero errors |
| 2 | Pipe chains preserve semantics | 5-stage pipe with known I/O | Correct output |
| 2 | Multi-stage Docker preserves stages | 3-stage build with cross-copy | All stages present |
| 2 | Makefile patterns expand correctly | `%.o: %.c` with 5 source files | All rules generated |
| 3 | No injection vectors in output | 30 adversarial inputs with shell metacharacters | All escaped |
| 3 | Unicode cannot bypass escaping | Bidi overrides, zero-width joiners | Stripped or quoted |
| 3 | Production scripts transpile | 50 real-world scripts | >= 95% pass |
| 4 | No regressions over time | Full corpus run weekly | Monotonic or stable |
| 4 | New entries do not break old ones | Add 10 entries, run full suite | Zero regressions |
| 4 | 100% rate is temporary | Add 50 harder entries after convergence | Rate drops, then recovers |
| 4 | Corpus grows forever | Measure corpus SIZE alongside rate | Monotonically increasing |

## Appendix B: Convergence Target Timeline (Sawtooth Pattern)

```
Rate
100%|          *           *              *                 *
    |         / \         / \            / \               / \
 99%|......../..\......../..\............/...\............./...\.... TARGET
    |       /    \      /    \         /     \           /     \
 95%|      /      \    /      \       /       \         /       \
    |     /        \  /        \     /         \       /         \
 90%|    /          \/          \   /           \     /           \
    |   /                        \ /             \   /             \
 80%|  /                          *               \ /               \
    | /                                            *                 ...
 70%|/
    +----+----+----+----+----+----+----+----+----+----+----+----+---->
    1    2    3    4    5    6    7    8    9    10   11   12   13  Iter

    Phase 1        Phase 2       Phase 3     Phase 4 (repeating sawtooth)
    (Tier 1)       (Tier 2-3)    (Tier 4-5)  (Add entries → rate drops → fix → recover)

Corpus size: 30   100  100  200  200  250  350  350  400  500  500  550  600
```

The convergence curve follows a **sawtooth pattern**, NOT a monotonic sigmoid. Each time 100% is reached, new harder entries are added, causing the rate to drop temporarily. The transpiler is then improved to recover. This is the healthy Kaizen cadence: perpetual challenge and improvement.

The corpus SIZE line is monotonically increasing. The RATE line oscillates as new challenges are introduced and overcome. A flat rate line at 100% for more than 2 iterations indicates the corpus has stopped growing -- this is an anti-pattern (see Appendix C).

## Appendix C: Anti-Patterns (What NOT to Do)

| Anti-Pattern | Why It's Wrong | Correct Response |
|---|---|---|
| **Modify corpus entry to match transpiler bug** | Destroys the falsifier. Hides the defect. Scientific fraud. | Fix the transpiler. The corpus is ground truth. |
| **Remove a failing corpus entry** | Evidence destruction. The entry revealed a real defect. | Fix the transpiler. Keep the entry forever. |
| **Stop adding entries after 100%** | Static corpus = static quality. New bugs will go undetected. | Add 50 harder entries immediately. |
| **Weaken expected output to be less specific** | Makes the test less effective at catching regressions. | Keep strict expectations. Fix the transpiler. |
| **Skip corpus entries in CI** | Defeats the purpose of automated quality enforcement. | Fix whatever is slow/broken. Run all entries always. |
| **Declare the transpiler "done"** | No transpiler is ever done. New Rust syntax, new edge cases. | Keep growing the corpus. Kaizen has no end. |
| **Blame the corpus when rate drops** | The corpus is the oracle. The transpiler is the SUT. | Rate drops are healthy -- they mean the corpus found defects. |
