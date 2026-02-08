# Corpus-Driven Transpilation Quality Specification

**Version**: 2.0.0
**Date**: 2026-02-07
**Status**: Draft (v2 — research design for quantifiable correctness)
**Methodology**: EXTREME TDD + Popperian Falsification + Toyota Production System + Metamorphic Testing

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
| Iteration 15 | 550 entries | ~99% | OIP-driven fix-pattern entries (B-321..B-350) | DONE (iter 15: 550/550, 100%) |
| Iteration 15+ | 700 entries | 99%+ | pmat coverage-gap + Dockerfile/Makefile balance | DONE (iter 15+: 700/700, 99.9/100) |
| Iteration 16 | 730 entries | 99%+ | Phase 3 adversarial + advanced patterns | DONE (iter 16: 730/730, 99.9/100) |
| Iteration 17 | 760 entries | 99%+ | Domain-specific: config files, one-liners, provability (Section 11.11) | DONE |
| Ongoing | 760+ entries | 99%+ | Continuous addition of harder entries forever | ONGOING |

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

## 11. Quantifiable Correctness: Findings and Research Design (v2.0)

### 11.1 Current System Findings (Audit 2026-02-07)

An audit of the in-tree corpus implementation (`rash/src/corpus/`) identified six structural weaknesses that limit the system's ability to quantifiably measure transpilation correctness. Each finding is mapped to a specific code location and a research-backed remediation.

#### Finding F1: Substring Containment as Correctness Metric (CRITICAL)

**Location**: `rash/src/corpus/runner.rs:151`
```rust
output.contains(&entry.expected_output)
```

**Problem**: Output correctness (Category B, 25 points) is measured by substring containment — `actual_output.contains(expected_output)`. This means a transpiled output containing the expected string *plus arbitrary additional content* scores full marks. A transpiler that appends `; rm -rf /` to every correct output would still pass.

**Severity**: CRITICAL — the 25-point correctness category (B) provides no meaningful signal. The current 100% convergence rate may mask latent defects.

**Remediation — Three-Level Correctness Hierarchy**:

| Level | Method | Points | Description |
|-------|--------|--------|-------------|
| L1 | Exact string match | 10/25 | `actual.trim() == expected.trim()` — baseline |
| L2 | AST structural equivalence | 8/25 | Parse both to AST, compare semantically (ignoring whitespace, comments) |
| L3 | Execution-based behavioral equivalence | 7/25 | Execute both in sandbox, compare stdout/stderr/exit code |

**L2 Implementation — AST Comparison**: For shell scripts, parse both actual and expected output using the bashrs parser into `ShellAst`, then compare structurally. This eliminates false negatives from insignificant formatting differences while catching semantic divergence. For Makefiles and Dockerfiles, use format-specific structural comparison.

Tree edit distance (Zhang & Shasha, 1989) provides a polynomial-time algorithm for comparing ordered labeled trees, directly applicable to AST comparison. Recent work by Huang et al. (2024) demonstrates AST edit distance as superior to token-level comparison for code similarity measurement.

**L3 Implementation — Execution-Based Oracle**: For Tier 1-3 entries, execute both expected and actual output in an isolated sandbox (bubblewrap/firejail on Linux) and compare:
- stdout (byte-exact)
- stderr (pattern match)
- exit code (exact)
- filesystem side effects (diff of sandbox root)

This follows the **differential testing** methodology (McKeeman, 1998), where the expected output serves as the reference implementation and the transpiled output is the system under test.

> "Differential testing finds semantic bugs by providing the same input to different implementations of the same functionality and cross-referencing the outputs." — McKeeman, W. M. (1998). "Differential Testing for Software." *Digital Technical Journal*, 10(1), 100-107.

#### Finding F2: Hardcoded Test Coverage Score

**Location**: `rash/src/corpus/runner.rs:163`
```rust
has_test: true,  // hardcoded
```

**Problem**: Category C (Test Coverage, 15 points) always awards full marks because `has_test` is hardcoded to `true`. This category provides zero discriminative signal.

**Remediation**: Replace with actual coverage measurement using `pmat query --coverage` integration (see Section 11.3). Each corpus entry should measure whether the transpiler code paths exercised by that entry are covered by unit tests:

```
C_score = (covered_transpiler_lines_for_entry / total_transpiler_lines_for_entry) × 15
```

This requires per-entry LLVM coverage tracing, achievable via `cargo llvm-cov report --json` with test-name filtering.

#### Finding F3: Two Disconnected Oracle Systems

**Locations**:
- In-tree k-NN oracle: `rash/src/quality/oracle.rs` (1858 lines, 73-feature vector, k=5)
- Standalone Random Forest oracle: `bashrs-oracle/src/lib.rs` (696 lines, 100 trees via `aprender`)

**Problem**: Two independent ML systems classify transpilation errors but are not connected to each other or to the corpus runner. The in-tree oracle uses k-NN with 15 error categories; the standalone oracle uses Random Forest with 24 categories. Neither feeds classification results back into the corpus scoring system. Neither is trained on real corpus failure data.

**Remediation — Unified Oracle Architecture**:

```
┌─────────────────────────────────────────────────────┐
│                 Unified Oracle                        │
│                                                       │
│  ┌──────────┐   ┌──────────────┐   ┌─────────────┐  │
│  │ k-NN     │   │ Random       │   │ Ensemble     │  │
│  │ (fast,   │──▶│ Forest       │──▶│ Voter        │  │
│  │  online) │   │ (accurate,   │   │ (majority    │  │
│  │          │   │  batch)      │   │  vote)       │  │
│  └──────────┘   └──────────────┘   └─────────────┘  │
│        ▲               ▲                   │          │
│        │               │                   ▼          │
│  ┌──────────────────────────────┐  ┌──────────────┐  │
│  │ Corpus Failure Training Data │  │ Fix Pattern   │  │
│  │ (real failures, not synthetic)│  │ Recommender  │  │
│  └──────────────────────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────┘
```

Ensemble classification combining k-NN and Random Forest improves prediction accuracy over either alone. Breiman (2001) demonstrated that Random Forests achieve lower generalization error through ensemble diversity, and combining with instance-based learners (k-NN) provides complementary bias-variance tradeoffs (Dietterich, 2000).

#### Finding F4: Synthetic Training Data

**Location**: `bashrs-oracle/src/lib.rs` — `Corpus::generate_synthetic(5000)`

**Problem**: The standalone oracle trains on 5000 synthetically generated examples, not on real corpus failures. The synthetic generator creates plausible-looking feature vectors with random labels, meaning the model learns artificial correlations rather than real failure patterns.

**Remediation**: Train exclusively on real corpus failure data. Every falsification event (corpus entry failure) generates a training example:

```rust
TrainingExample {
    features: FeatureVector::extract(&diagnostic, &source_code),
    label: error_category,  // manually classified on first occurrence
    corpus_entry_id: "B-036",
    transpiler_version: "6.60.0",
    fix_applied: "compound_assign_desugar",
}
```

With 500 corpus entries and 8 historical bugs (see Section 5.1), the current real training set is small. **Active learning** (Settles, 2012) addresses this by selecting the most informative examples for labeling: run the oracle on new corpus entries, and prioritize manual labeling of entries where the oracle is least confident.

#### Finding F5: No Cross-Validation or Held-Out Test Set

**Problem**: Neither oracle system uses cross-validation or a held-out test set. Model accuracy is unmeasured. The in-tree k-NN uses bootstrap patterns (PAT-001..PAT-015) as a fallback but never validates their accuracy against held-out data.

**Remediation**: Implement k-fold cross-validation (k=5) on the real corpus failure dataset. Report precision, recall, and F1-score per error category. Maintain a 20% held-out test set that is never used during training — only for final accuracy measurement.

**Target Metrics** (based on software defect prediction literature):
- Accuracy: ≥80% (Malhotra, 2015 reports 75-85% for Random Forest on NASA datasets)
- F1-score: ≥0.75 per category
- AUC-ROC: ≥0.80

#### Finding F6: No Execution-Based Behavioral Equivalence

**Problem**: No corpus entry is ever *executed*. Correctness is entirely syntactic (string match or lint pass). A transpiled script could be syntactically correct but behaviorally wrong (e.g., an off-by-one in a loop range, incorrect variable scoping, wrong exit code).

**Remediation**: See Section 11.2 for the execution-based oracle design.

---

### 11.2 Execution-Based Oracle Design (Behavioral Equivalence)

The **test oracle problem** (Barr et al., 2015) is the fundamental challenge of determining whether a program's output is correct. For transpilers, the oracle problem is acute: the expected *behavior* of the output program must match the input program's semantics, but behavior is not directly observable from syntax alone.

We propose a **three-tier oracle** that progressively strengthens correctness guarantees:

#### Tier A: Reference Execution Oracle (Differential Testing)

For each corpus entry, maintain a **reference execution trace**:

```toml
# corpus/tier-2-standard/B-052/execution.toml
[execution]
stdin = ""
argv = []
env = { HOME = "/tmp/test", PATH = "/usr/bin" }

[expected]
stdout = "hello world\n"
stderr = ""
exit_code = 0
files_created = ["output.txt"]
files_content = { "output.txt" = "result\n" }
```

The transpiled output is executed in an identical sandbox and all observable effects are compared. This is **differential testing** (McKeeman, 1998) where the expected execution trace is the reference oracle.

**Sandbox Requirements**:
- Filesystem isolation (tmpfs mount, no host access)
- Network isolation (no outbound connections)
- Time budget: 5s per entry (kill on timeout)
- Resource limits: 64MB memory, 1MB stdout
- Deterministic environment (fixed PATH, HOME, locale, timezone)

#### Tier B: Metamorphic Testing Oracle

**Metamorphic testing** (Chen et al., 2018) alleviates the oracle problem by defining **metamorphic relations** (MRs) — properties that must hold across related inputs, even when individual outputs cannot be independently verified.

**Metamorphic Relations for Shell Transpilation**:

| MR ID | Relation | Description |
|-------|----------|-------------|
| MR-1 | **Determinism** | `transpile(X) == transpile(X)` — same input always produces same output |
| MR-2 | **Monotonicity** | Adding a no-op line to input does not change output semantics |
| MR-3 | **Commutativity** | Reordering independent variable assignments does not change behavior |
| MR-4 | **Idempotency** | `transpile(purify(X)) == transpile(X)` — purification is idempotent |
| MR-5 | **Subsumption** | If `transpile(A)` succeeds and B is a simplification of A, `transpile(B)` must succeed |
| MR-6 | **Composition** | `transpile(A; B) ≡ transpile(A); transpile(B)` for independent statements |
| MR-7 | **Negation** | `transpile(if P then A else B)` must swap branches when P is negated |

> "A central element [of metamorphic testing] is a set of metamorphic relations, which are necessary properties of the target function or algorithm in relation to multiple inputs and their expected outputs." — Chen, T. Y. et al. (2018). "Metamorphic Testing: A Review of Challenges and Opportunities." *ACM Computing Surveys*, 51(1), Article 4.

**Implementation**: For each corpus entry, generate follow-up test cases by applying MR transformations. Verify that the metamorphic relation holds between the source and follow-up outputs. This multiplies the effective corpus size without requiring new expected outputs.

**Coverage Amplification**: 500 corpus entries × 7 MRs = 3,500 effective test cases.

#### Tier C: N-Version Oracle (Cross-Shell Validation)

Execute transpiled POSIX shell output across multiple shell interpreters:

| Shell | Version | Purpose |
|-------|---------|---------|
| dash | 0.5.12+ | POSIX reference (strict) |
| bash | 5.2+ | Most common (permissive) |
| busybox ash | 1.36+ | Minimal POSIX (embedded) |
| zsh --emulate sh | 5.9+ | Diversity check |

If all four shells produce identical output, correctness confidence is high. Any divergence indicates either:
1. A POSIX compliance bug in the transpiled output (the transpiler must be fixed)
2. A shell interpreter bug (rare, document and exclude)

This follows the **N-version programming** principle (Avizienis, 1985): fault detection through diversity.

> "The N-version programming approach is based on the assumption that the probability of identical errors in independently developed implementations of the same specification is small." — Avizienis, A. (1985). "The N-Version Approach to Fault-Tolerant Software." *IEEE Transactions on Software Engineering*, SE-11(12), 1491-1501.

#### Quantifiable Correctness Metrics

The revised scoring system replaces the current string-containment metric with a multi-dimensional correctness measurement:

| Metric | Formula | Target |
|--------|---------|--------|
| **Syntactic Correctness** | `exact_match_count / total_entries` | ≥99% |
| **Structural Equivalence** | `ast_equivalent_count / total_entries` | ≥99% |
| **Behavioral Equivalence** | `execution_match_count / executable_entries` | ≥95% |
| **Metamorphic Consistency** | `mr_hold_count / (entries × mr_count)` | ≥98% |
| **Cross-Shell Consistency** | `all_shells_agree_count / executable_entries` | ≥90% |
| **Oracle Precision** | `correct_classifications / total_classifications` | ≥80% |
| **Oracle Recall** | `detected_faults / total_faults` | ≥85% |
| **Mutation Kill Rate** | `killed_mutants / total_mutants` | ≥90% |

---

### 11.3 Research Design: Improving Makefile, Bash, and Dockerfile Quality

#### 11.3.1 Bash Quality Improvement

**Current State**: 200 entries (B-001..B-200), 100% convergence, 8 transpiler bugs found and fixed.

**Gap Analysis**:
1. No execution-based verification — all correctness is syntactic
2. No coverage of interactive constructs (read, select, trap)
3. No heredoc/herestring transpilation testing
4. No pipeline error propagation testing (`set -o pipefail` semantics)

**Research Protocol**:

| Phase | Action | Metric | Target |
|-------|--------|--------|--------|
| R1 | Add execution traces for Tier 1-2 entries (90 entries) | Behavioral match rate | ≥95% |
| R2 | Add metamorphic relations MR-1 through MR-7 | MR violation rate | <2% |
| R3 | Cross-shell validation (dash, bash, ash, zsh) | Agreement rate | ≥90% |
| R4 | Add 50 entries for interactive/heredoc/pipeline constructs | Transpilation rate after additions | measure drop |
| R5 | Train oracle on real B-series failures | Classification F1 | ≥0.75 |

**Bash-Specific Metamorphic Relations**:
- **MR-B1**: Quoting transformation — `$var` → `"$var"` must not change behavior
- **MR-B2**: Arithmetic equivalence — `$((x+1))` ≡ `$((x + 1))`
- **MR-B3**: Function inlining — inlining a single-use function must preserve behavior
- **MR-B4**: Pipe to process substitution — `cmd1 | cmd2` ≡ `cmd2 <(cmd1)` for stdin readers

#### 11.3.2 Makefile Quality Improvement

**Current State**: 150 entries (M-001..M-150), 100% convergence.

**Gap Analysis**:
1. No validation of Make's rebuild semantics (timestamp-based dependency resolution)
2. No testing of parallel make (`-j` flag) safety
3. No recursive vs non-recursive make pattern testing
4. No validation of automatic variable expansion (`$@`, `$<`, `$^`, `$?`)

**Research Protocol**:

| Phase | Action | Metric | Target |
|-------|--------|--------|--------|
| R1 | Add execution traces with `make -n` dry-run comparison | Command sequence match | ≥98% |
| R2 | Add parallel-safety test entries (`make -j4` vs `make -j1`) | Output equivalence | ≥95% |
| R3 | Add 30 entries for automatic variables and pattern rules | Transpilation rate | measure drop |
| R4 | Validate rebuild semantics (touch file, re-make, verify minimal rebuild) | Correct rebuild count | 100% |
| R5 | Cross-validate with GNU Make 4.3+ and bmake | Agreement rate | ≥85% |

**Makefile-Specific Metamorphic Relations**:
- **MR-M1**: Target reordering — reordering independent targets must not change build output
- **MR-M2**: Variable expansion — `:=` (simply-expanded) must be equivalent to `=` for non-recursive definitions
- **MR-M3**: Phony equivalence — `.PHONY: clean` must produce same behavior whether declared or not (for recipes without file output)

#### 11.3.3 Dockerfile Quality Improvement

**Current State**: 150 entries (D-001..D-150), 100% convergence.

**Gap Analysis**:
1. No image build verification (transpiled Dockerfiles are never built)
2. No layer count optimization measurement
3. No multi-platform build testing (arm64 vs amd64)
4. No BuildKit-specific feature testing (cache mounts, secret mounts)

**Research Protocol**:

| Phase | Action | Metric | Target |
|-------|--------|--------|--------|
| R1 | Add `docker build --no-cache` verification for Tier 1-2 | Build success rate | ≥95% |
| R2 | Measure layer count vs expected layer count | Layer count delta | ≤1 per entry |
| R3 | Add 25 entries for BuildKit features (cache mounts, secrets, heredocs) | Transpilation rate | measure drop |
| R4 | Hadolint cross-validation (run both bashrs and hadolint, compare) | Agreement rate | ≥90% |
| R5 | Multi-platform build matrix (amd64, arm64) | Build success rate | ≥90% |

**Dockerfile-Specific Metamorphic Relations**:
- **MR-D1**: Layer merging — combining two `RUN` commands with `&&` must produce same filesystem
- **MR-D2**: Stage reordering — reordering independent build stages must produce same final image
- **MR-D3**: ARG default override — `--build-arg` overriding default must propagate correctly

---

### 11.4 Revised 100-Point Scoring System (v2)

The original scoring system (Section 4) is updated to replace weak metrics with quantifiable measurements:

| Category | v1 (Current) | v2 (Proposed) | Change |
|----------|-------------|---------------|--------|
| A. Transpilation Success | 40 pts — transpiles without error | 30 pts — transpiles without error | -10 pts (still critical but overweighted) |
| B. Output Correctness | 25 pts — `output.contains()` | 25 pts — L1 exact (10) + L2 AST (8) + L3 execution (7) | Decomposed into 3 levels |
| C. Test Coverage | 15 pts — hardcoded `true` | 15 pts — actual LLVM coverage per entry | Real measurement |
| D. Lint Compliance | 10 pts — lint pass/fail | 10 pts — lint pass/fail (unchanged) | No change |
| E. Determinism | 10 pts — transpile twice, compare | 10 pts — transpile twice, compare (unchanged) | No change |
| **F. Metamorphic Consistency** | — | **5 pts** — MR-1 through MR-7 hold | **NEW** |
| **G. Cross-Shell Agreement** | — | **5 pts** — all reference shells agree | **NEW** |
| **Total** | **100 pts** | **100 pts** | Rebalanced |

**v2 Scoring Formula**:
```
Score = (A × 30)
      + (B_L1 × 10 + B_L2 × 8 + B_L3 × 7)
      + (C_coverage × 15)
      + (D_lint × 10)
      + (E_determinism × 10)
      + (F_metamorphic × 5)
      + (G_cross_shell × 5)
```

**Gateway Logic** (updated):
- If A < 18 (60% transpilation): B through G score 0
- If B_L1 < 6 (60% exact match): B_L2 and B_L3 score 0

---

### 11.5 Oracle Unification and ML Pipeline

#### 11.5.1 Feature Alignment

The in-tree oracle uses a 73-feature vector (20 lexical + 25 structural + 28 semantic) but only 24 dimensions for k-NN distance calculation. The standalone oracle uses `aprender` with an opaque feature matrix. These must be aligned:

**Unified Feature Schema** (32 features):

| Feature Group | Count | Features |
|---------------|-------|----------|
| Lexical | 8 | line_count, token_count, avg_line_length, max_line_length, comment_ratio, blank_ratio, string_literal_count, numeric_literal_count |
| Structural | 10 | nesting_depth, branch_count, loop_count, function_count, pipe_count, redirect_count, subshell_count, command_count, variable_ref_count, assignment_count |
| Semantic | 8 | has_shebang, uses_set_e, uses_set_u, has_trap, uses_eval, uses_source, has_heredoc, uses_arithmetic |
| Quality | 6 | lint_violation_count, lint_severity_max, determinism_score, idempotency_score, quoting_ratio, shellcheck_issue_count |

#### 11.5.2 Training Pipeline

```
Corpus Run (500 entries)
    │
    ├── Passing entries → negative examples (no fault)
    │
    └── Failing entries → positive examples
            │
            ├── Extract 32-feature vector
            ├── Label: error_category (24 categories)
            ├── Label: fix_pattern (15 patterns)
            │
            ▼
    ┌─────────────────┐
    │  Train/Test Split │
    │  (80/20, stratified) │
    └─────────────────┘
            │
            ├──▶ k-NN (k=5, online, fast)
            ├──▶ Random Forest (100 trees, batch, accurate)
            │
            ▼
    ┌─────────────────┐
    │  Ensemble Voter   │
    │  (weighted majority)│
    └─────────────────┘
            │
            ▼
    ┌─────────────────┐
    │  5-Fold CV Report │
    │  P/R/F1 per class │
    └─────────────────┘
```

#### 11.5.3 Drift Detection

Both oracles include drift detection, but they measure different things. Unify on a single drift metric:

```
drift_score = |accuracy_window_recent - accuracy_window_historical|
```

Where `accuracy_window_recent` is the classification accuracy over the last 50 corpus runs and `accuracy_window_historical` is the accuracy over the preceding 200 runs. If `drift_score > 0.10` (10% accuracy drop), trigger model retraining.

This follows the concept drift detection methodology from Gama et al. (2014): "A survey on concept drift adaptation."

---

### 11.6 Implementation Roadmap (v2 Enhancements)

| Phase | Work | Duration | Key Metric |
|-------|------|----------|------------|
| V2-1 | Replace `output.contains()` with exact match (L1) | 1 week | Measure how many entries currently pass exact match |
| V2-2 | Add AST structural comparison (L2) for bash entries | 2 weeks | AST equivalence rate across B-001..B-200 |
| V2-3 | Add execution traces for Tier 1-2 entries (L3) | 3 weeks | Behavioral match rate ≥95% |
| V2-4 | Implement 7 metamorphic relations | 2 weeks | MR violation rate <2% |
| V2-5 | Cross-shell execution (dash, bash, ash, zsh) | 2 weeks | Agreement rate ≥90% |
| V2-6 | Unify oracle systems into ensemble | 3 weeks | Classification F1 ≥0.75 |
| V2-7 | Replace synthetic training with real corpus failures | 1 week | Training set from 8+ real bugs |
| V2-8 | Implement real coverage measurement (replace hardcoded `has_test`) | 1 week | Coverage score variance >0 |
| V2-9 | Makefile execution verification (`make -n`) | 2 weeks | Command sequence match ≥98% |
| V2-10 | Dockerfile build verification (`docker build`) | 2 weeks | Build success rate ≥95% |

**Total estimated effort**: 19 weeks (can be parallelized to ~10 weeks with 2 developers)

---

### 11.7 Aprender Integration: Model Compilation and Provability

The `aprender` crate (../aprender) provides the ML infrastructure for the unified oracle. Key capabilities discovered via `pmat query`:

#### 11.7.1 Core API for Corpus Oracle

**Estimator trait** (`src/traits.rs`):
```rust
pub trait Estimator {
    fn fit(&mut self, x: &Matrix<f32>, y: &Vector<f32>) -> Result<()>;
    fn predict(&self, x: &Matrix<f32>) -> Vector<f32>;
    fn score(&self, x: &Matrix<f32>, y: &Vector<f32>) -> f32;
}
```

**RandomForestClassifier** (`examples/random_forest_iris.rs`):
```rust
let mut rf = RandomForestClassifier::new(100)  // 100 trees
    .with_max_depth(10)
    .with_random_state(42);  // deterministic training
rf.fit(&x_train, &y_train)?;
let predictions = rf.predict(&x_test);
let accuracy = rf.score(&x_test, &y_test);
```

**Classification metrics** (`src/metrics/classification.rs`):
- `accuracy(y_pred, y_true) -> f32`
- `precision(y_pred, y_true, Average::Macro) -> f32`
- `recall(y_pred, y_true, Average::Macro) -> f32`
- `f1_score(y_pred, y_true, Average::Weighted) -> f32`
- `evaluate_classification(y_pred, y_true) -> HashMap<String, f32>` — full report

**Cross-validation** (`src/model_selection/mod.rs`):
- `CrossValidationResult { scores: Vec<f32> }` — k-fold CV
- `cross_validate(&model, &x, &y, &kfold) -> Result<CrossValidationResult>`

#### 11.7.2 Poka-Yoke Quality Gates (APR-POKA-001)

Aprender implements Toyota's Poka-yoke (mistake-proofing) as a first-class concept:

**PokaYoke trait** (`src/format/validation.rs`):
```rust
pub trait PokaYoke {
    fn poka_yoke_validate(&self) -> PokaYokeResult;
    fn quality_score(&self) -> u8 { self.poka_yoke_validate().score }
}
```

**Jidoka gate in .apr format** (`src/format/core_io.rs`):
- `save()` refuses to write models with `quality_score == 0` (APR-POKA-001)
- Models are serialized as `.apr` files with MessagePack metadata, zstd compression, CRC32 checksums
- Quality score is embedded in the file header — consumers can verify before loading

**Application to corpus oracle**: The corpus oracle model should implement `PokaYoke` with gates for:
1. Minimum training accuracy (≥80%)
2. Minimum F1-score per category (≥0.60)
3. Training data size (≥50 real failure examples)
4. Cross-validation score variance (<0.15)

If any gate fails, `save()` refuses to persist the model — Jidoka stops the line at the ML level.

#### 11.7.3 Drift Detection for Oracle Monitoring

Aprender provides two drift detection mechanisms:

**DriftDetector trait** (`src/online/drift.rs`):
```rust
pub trait DriftDetector: Send + Sync {
    fn add_element(&mut self, error: bool);     // feed prediction outcomes
    fn detected_change(&self) -> DriftStatus;   // check for drift
}
```

**RollingDriftMonitor** (`src/metrics/drift.rs`):
- Maintains reference + current windows
- Statistical distance measures between windows
- `RetrainingTrigger`: combines multiple drift signals, requires N consecutive detections

**Application**: After each corpus run, feed oracle classification outcomes into `RollingDriftMonitor`. When drift is detected (corpus failures shift in character), trigger model retraining from updated failure data.

#### 11.7.4 Model Persistence and Versioning

**`.apr` format** (`src/format/core_io.rs`):
- Binary format: Header (64B) + MessagePack metadata + zstd payload + CRC32
- AES-256-GCM encryption option for sensitive models
- Embedded metadata: model type, training date, quality score, feature names

**Corpus oracle model lifecycle**:
```
Train on corpus failures → PokaYoke validate → Save as .apr
    → Embed in bashrs binary (include_bytes!)
    → Load at runtime for error classification
    → Monitor with DriftDetector
    → Retrain when drift detected
```

---

### 11.8 Formal Schema Enforcement for Output Formats

Each target format (Bash, Makefile, Dockerfile) has a formal grammar or specification that transpiled outputs must conform to. Schema enforcement ensures outputs are not just syntactically plausible but grammatically valid according to the authoritative specification.

#### 11.8.1 POSIX Shell Grammar (Bash Output)

**Authoritative spec**: IEEE Std 1003.1-2017 (POSIX.1), Shell Command Language (Section 2)

**Grammar enforcement layers**:

| Layer | Validator | What It Checks | Pass Criteria |
|-------|-----------|----------------|---------------|
| L1: Lexical | bashrs parser (`ShellAst`) | Token stream is valid | Parses without error |
| L2: Syntactic | `shellcheck -s sh` | POSIX grammar compliance | Zero errors (SC-level "error") |
| L3: Semantic | bashrs linter (SEC/DET/IDEM rules) | Security, determinism, idempotency | Zero violations |
| L4: Behavioral | Cross-shell execution (dash, bash, ash) | Runtime equivalence | All shells agree |

**POSIX grammar productions enforced** (subset):

```
complete_command : list separator_op
               | list
               ;
list            : list separator_op and_or
               | and_or
               ;
and_or          : pipeline
               | and_or AND_IF linebreak pipeline
               | and_or OR_IF linebreak pipeline
               ;
pipeline        : pipe_sequence
               | Bang pipe_sequence
               ;
```

**Corpus enforcement**: Every transpiled shell script MUST parse successfully against the POSIX grammar. The bashrs parser already produces `ShellAst` — we add a `validate_posix_grammar(ast: &ShellAst) -> Vec<GrammarViolation>` function that checks:
- No bashisms (process substitution `<()`, arrays, `[[ ]]`)
- Correct quoting (all variable expansions in double quotes)
- Valid here-document delimiters
- Correct `case` pattern syntax
- Proper arithmetic expansion `$(())`

#### 11.8.2 GNU Make Grammar (Makefile Output)

**Authoritative spec**: GNU Make Manual, 4.4 (2023), Section 3.7 "How `make` Reads a Makefile"

**Grammar enforcement layers**:

| Layer | Validator | What It Checks | Pass Criteria |
|-------|-----------|----------------|---------------|
| L1: Lexical | Tab-vs-space detection | Recipe lines use tabs | Zero space-indented recipes |
| L2: Syntactic | `make -n --warn-undefined-variables` | Valid Make syntax | Zero warnings |
| L3: Semantic | bashrs Makefile linter (MAKE001-MAKE020) | Best practices | Zero violations |
| L4: Behavioral | `make -n` dry-run comparison | Command sequence | Matches expected |

**Makefile grammar schema** (key rules):

```
makefile     : (rule | assignment | directive | comment | empty_line)*
rule         : targets ':' prerequisites '\n' recipe
targets      : target (' ' target)*
prerequisites: prerequisite (' ' prerequisite)*
recipe       : ('\t' command '\n')+
assignment   : variable assignment_op value
assignment_op: ':=' | '?=' | '+=' | '='
directive    : 'include' | 'ifeq' | 'ifdef' | 'define' | '.PHONY' | ...
```

**Schema violations detectable at parse time**:
- Recipe lines not starting with tab character
- Undefined variable references (`:=` without prior definition)
- Circular dependency detection
- `.PHONY` targets with file-producing recipes
- Recursive vs simply-expanded variable misuse

#### 11.8.3 Dockerfile Grammar (Dockerfile Output)

**Authoritative spec**: Dockerfile reference, Docker Engine v25+ (2024)

**Grammar enforcement layers**:

| Layer | Validator | What It Checks | Pass Criteria |
|-------|-----------|----------------|---------------|
| L1: Lexical | Instruction keyword recognition | Valid instructions only | All lines are valid instructions |
| L2: Syntactic | bashrs Dockerfile parser | Correct argument format | Parses without error |
| L3: Semantic | bashrs Dockerfile linter (DOCKER001-012) + Hadolint | Best practices | Zero violations |
| L4: Behavioral | `docker build --no-cache` | Builds successfully | Exit code 0 |

**Dockerfile grammar schema** (key rules):

```
dockerfile   : (instruction | comment | empty_line)*
instruction  : FROM from_args
             | RUN run_args
             | COPY copy_args
             | WORKDIR path
             | ENV env_args
             | EXPOSE port_spec
             | USER user_spec
             | CMD exec_or_shell
             | ENTRYPOINT exec_or_shell
             | ARG arg_spec
             | LABEL label_args
             | HEALTHCHECK healthcheck_args
             | ...
from_args    : ['--platform=' platform] image [':' tag | '@' digest] ['AS' name]
exec_or_shell: exec_form | shell_form
exec_form    : '[' string (',' string)* ']'
shell_form   : string
```

**Schema violations detectable at parse time**:
- `FROM` not as first instruction (multi-stage: each stage starts with FROM)
- `:latest` tag (DOCKER002 — must pin version)
- Shell form for `ENTRYPOINT`/`CMD` (exec form required)
- Missing `USER` directive (DOCKER003 — non-root enforcement)
- `ADD` instead of `COPY` for local files (DOCKER004)

#### 11.8.4 Schema Validation Integration with Corpus Scoring

Add a **Schema Conformance** check to each corpus entry's scoring:

```rust
fn check_schema_conformance(output: &str, format: CorpusFormat) -> SchemaResult {
    match format {
        CorpusFormat::Bash => {
            let ast = parse_posix_shell(output)?;
            let violations = validate_posix_grammar(&ast);
            SchemaResult { valid: violations.is_empty(), violations }
        }
        CorpusFormat::Makefile => {
            let ast = parse_makefile(output)?;
            let violations = validate_make_grammar(&ast);
            SchemaResult { valid: violations.is_empty(), violations }
        }
        CorpusFormat::Dockerfile => {
            let ast = parse_dockerfile(output)?;
            let violations = validate_dockerfile_grammar(&ast);
            SchemaResult { valid: violations.is_empty(), violations }
        }
    }
}
```

Schema conformance becomes a **hard gate**: if `valid == false`, the entry scores 0 on categories B through G regardless of other results. This is stronger than the existing gateway logic — a syntactically invalid output cannot be correct, tested, or deterministic.

#### 11.8.5 Aprender Model for Grammar Error Classification

Train a `RandomForestClassifier` via aprender to classify grammar violations by root cause:

| Category | Description | Fix Pattern |
|----------|-------------|-------------|
| GRAM-001 | Missing quoting in expansion | Add double quotes around `${}` |
| GRAM-002 | Bashism in POSIX output | Replace `[[ ]]` with `[ ]` |
| GRAM-003 | Tab/space confusion in Makefile | Ensure recipe lines use `\t` |
| GRAM-004 | Shell form in Dockerfile CMD | Convert to exec form `["cmd", "arg"]` |
| GRAM-005 | Undefined variable reference | Add `:=` assignment before use |
| GRAM-006 | Invalid POSIX arithmetic | Replace bash-specific `(( ))` with `$(( ))` |
| GRAM-007 | Missing FROM in Dockerfile | Add `FROM` as first instruction |
| GRAM-008 | Circular Make dependency | Reorder targets |

The classifier uses the 32-feature unified schema (Section 11.5.1) plus 4 grammar-specific features:
- `grammar_violation_count`: total violations
- `grammar_violation_severity`: max severity
- `format_type`: bash=0, makefile=1, dockerfile=2
- `nesting_at_violation`: AST depth at first violation

Training data comes from real corpus grammar failures, following the same pipeline as Section 11.5.2. The model is persisted as `.apr` with Poka-yoke validation (APR-POKA-001) ensuring minimum quality before deployment.

### 11.9 OIP-Driven Corpus Generation

Organizational Intelligence Platform (OIP) provides automated mining of real fix patterns from git history across an entire GitHub organization. This section defines how OIP outputs are systematically converted into corpus entries, ensuring the corpus reflects **real defects** rather than hypothetical edge cases.

#### 11.9.1 Mining Methodology

OIP analyzes commit history to classify fix patterns into 18 defect categories:

```bash
# Extract training data from a single repo
oip extract-training-data --repo . --max-commits 500

# Analyze an entire GitHub organization
oip analyze --org paiml

# Output: classified fix commits with defect categories, severity, and code diffs
```

**Key insight**: Every bug fix in the transpiler's history represents a real-world failure mode. Each fix should generate 1-3 corpus entries that would **catch the regression** if the bug were reintroduced.

#### 11.9.2 Defect Category to Corpus Entry Mapping

OIP's 18 defect categories map to specific corpus entry patterns:

| OIP Category | Frequency (bashrs) | Corpus Entry Pattern | Example |
|---|---|---|---|
| ASTTransform | 62 | Parser/emitter correctness: heredoc, brackets, brace groups, command substitution | B-321..B-330 |
| OperatorPrecedence | 6 | Arithmetic parenthesization, operator associativity | B-331..B-335 |
| SecurityVulnerabilities | 24 | Quoting, injection prevention, special character handling | B-336..B-340 |
| IdempotencyViolation | 8 | `mkdir -p`, atomic writes, lock files, existence checks | B-341..B-345 |
| ComprehensionBugs | 8 | Iterator patterns, accumulation, filtering, early exit | B-346..B-350 |
| ConfigurationErrors | 7 | Env var handling, default values, path construction | Future entries |
| IntegrationFailures | 3 | Cross-shell compatibility, version-specific behavior | Future entries |
| FalsePositives | 5 | Linter rules triggering on valid code (SC2171, MAKE016) | Linter corpus |

#### 11.9.3 Fix-Driven Entry Generation Protocol

For each OIP-detected fix commit:

1. **Extract the fix diff**: Identify what changed in the transpiler
2. **Identify the input that triggered the bug**: Reconstruct the Rust DSL input
3. **Determine the correct output**: What the transpiler should produce post-fix
4. **Create 1-3 corpus entries**:
   - **Entry A**: The exact regression case (minimal reproducer)
   - **Entry B**: A generalized variant (different values, same pattern)
   - **Entry C**: An edge case variant (boundary conditions)

**Example** (from Issue #59 — nested quotes in command substitution):

```
Fix commit: "fix: handle nested quotes inside command substitution"
OIP category: ASTTransform
Severity: P1

→ Corpus entry B-321:
  Input:  fn main() { let out = command_output("echo \"hello\""); }
  Output: out=$(echo "hello")
  Tests:  Nested quoting preserved through transpilation
```

#### 11.9.4 Org-Wide Pattern Analysis

Running `oip analyze --org paiml` across 28 repositories reveals cross-project defect patterns applicable to bashrs:

| Cross-Project Pattern | Source Repos | bashrs Relevance |
|---|---|---|
| Off-by-one in range iteration | depyler, aprender | `for i in $(seq)` boundary values |
| String escaping in code generation | depyler, decy | Quote handling in shell output |
| Precedence errors in expression trees | depyler, decy | Arithmetic parenthesization |
| Missing error path handling | trueno, aprender | Shell `set -e` interaction |

These patterns inform corpus entries that test **cross-cutting concerns** — defect classes that appear in multiple transpiler projects and are likely to recur.

#### 11.9.5 Continuous OIP Integration

OIP analysis should be re-run periodically to discover new fix patterns:

- **Per-release**: `oip extract-training-data --repo . --since <last-release-tag>`
- **Monthly**: `oip analyze --org paiml` for cross-project patterns
- **On regression**: Immediate `oip extract-training-data` on the fix commit to generate corpus entries

Each OIP run produces a training data file (JSON) that is processed into corpus entries following the protocol in Section 11.9.3. The corpus grows monotonically (Section 1.2 — append-only rule) with each OIP cycle adding 10-30 entries.

### 11.10 Cross-Project Techniques from depyler

The `depyler` Python-to-Rust transpiler (same org) has developed three corpus-driven ML techniques that are directly applicable to bashrs. This section defines how each technique adapts to shell transpilation.

> "Standing on the shoulders of sister projects is not reuse—it is organizational learning." — Adapted from Nonaka & Takeuchi (1995), *The Knowledge-Creating Company*.

#### 11.10.1 Tarantula Fault Localization for Transpiler Decisions

**Source**: `depyler-oracle/src/tarantula_corpus.rs` (Jones & Harrold, 2005)

Tarantula assigns a **suspiciousness score** to each transpiler decision based on how strongly it correlates with corpus failures. In depyler, this identified `async_await` as the #1 priority (suspiciousness 0.946) when intuition suggested other features.

**Adaptation to bashrs**:

Each corpus entry's transpilation produces a **decision trace** — the sequence of emitter choices made:

```rust
struct TranspilerDecision {
    /// e.g., "emit_for_range", "emit_if_condition", "emit_arithmetic"
    decision_type: String,
    /// e.g., "seq_inclusive", "test_bracket", "dollar_paren_paren"
    choice: String,
    /// Line in the Rust DSL input
    source_span: (usize, usize),
}
```

Tarantula scoring formula (Jones & Harrold, 2005):

```
suspiciousness(d) = (failed(d) / total_failed) / ((failed(d) / total_failed) + (passed(d) / total_passed))
```

Where `failed(d)` = number of failing corpus entries that exercised decision `d`, and `passed(d)` = number of passing entries that exercised it.

**Expected output** (run periodically on corpus):

```
Decision                    Suspiciousness   Impact    Priority
────────────────────────────────────────────────────────────────
emit_nested_arithmetic      0.89             HIGH      Fix first
emit_string_in_conditional  0.72             MEDIUM    Fix second
emit_heredoc_expansion      0.68             MEDIUM    Investigate
emit_brace_group            0.45             LOW       Monitor
emit_simple_assignment      0.02             NONE      Stable
```

Decisions with suspiciousness > 0.7 trigger automatic corpus entry generation (Section 11.9.3) targeting the suspicious code path with adversarial inputs.

#### 11.10.2 CITL (Compiler-in-the-Loop) Pattern Mining

**Source**: `depyler-oracle/src/corpus_citl.rs` (entrenar `DecisionCITL`)

CITL closes the feedback loop between transpiler output and downstream validation. In depyler, the "compiler" is `rustc` — transpiled Rust that fails `cargo check` generates training signal. In bashrs, the "compilers" are **shellcheck** and **/bin/sh execution**.

**CITL feedback loop for bashrs**:

```
┌────────────────────┐     ┌──────────────────┐     ┌────────────────────┐
│  Rust DSL Input    │────►│  bashrs Transpile │────►│  POSIX Shell Output│
│  (corpus entry)    │     │  (decision trace) │     │  (generated .sh)   │
└────────────────────┘     └──────────────────┘     └────────────────────┘
                                                            │
                           ┌────────────────────────────────┼──────────────┐
                           │                                │              │
                           ▼                                ▼              ▼
                    ┌──────────────┐              ┌──────────────┐  ┌────────────┐
                    │  shellcheck  │              │  sh -c exec  │  │  dash exec │
                    │  (lint gate) │              │  (B3 behav.) │  │  (G cross) │
                    └──────────────┘              └──────────────┘  └────────────┘
                           │                                │              │
                           └────────────────────────────────┼──────────────┘
                                                            │
                                                            ▼
                                                 ┌──────────────────┐
                                                 │  PatternStore    │
                                                 │  (BM25 + Dense)  │
                                                 │  error → fix map │
                                                 └──────────────────┘
```

**Pattern store schema**:

```rust
struct ShellFixPattern {
    /// Shellcheck error code or execution failure type
    error_signal: String,        // e.g., "SC2086", "B3_timeout", "G_dash_fail"
    /// Transpiler decision that caused the error
    causal_decision: String,     // e.g., "emit_unquoted_variable"
    /// Fix applied to the transpiler
    fix_type: String,            // e.g., "add_double_quotes"
    /// Confidence (0.0-1.0) from Tarantula suspiciousness
    confidence: f64,
    /// Corpus entries that demonstrated this pattern
    evidence_ids: Vec<String>,   // e.g., ["B-042", "B-189", "B-336"]
}
```

**Training cycle**:

1. Run full corpus → collect all B3/D/G failures
2. Extract decision traces from failing entries
3. Match failure signals to decisions via Tarantula (Section 11.10.1)
4. Build `ShellFixPattern` entries for each error→decision→fix triple
5. On next transpilation, query PatternStore for known fixes when a decision is about to be made
6. Log suggestions to convergence log for human review

#### 11.10.3 Graph-Aware Corpus with Call Context

**Source**: `depyler-oracle/src/graph_corpus.rs` (depyler-graph `VectorizedFailure`)

Depyler enriches each corpus failure with **call graph context** — the in-degree, out-degree, callers, and callees of the function where the failure occurred. Functions with high connectivity (many callers) are higher priority because a fix has greater blast radius.

**Adaptation to bashrs**:

The Rust DSL inputs define functions. Each corpus entry can be enriched with graph context:

```rust
struct ShellGraphContext {
    /// Function being transpiled
    function_name: String,
    /// Number of call sites in the corpus (how many entries call this function)
    corpus_call_count: usize,
    /// Functions this function calls
    callees: Vec<String>,
    /// Functions that call this function
    callers: Vec<String>,
    /// Whether this function is in the "hot path" (called by >5 entries)
    is_high_connectivity: bool,
}
```

**Prioritization formula**:

```
priority(f) = suspiciousness(f) × log2(1 + corpus_call_count(f))
```

A function that is both suspicious (high failure correlation) AND highly connected (many callers) gets top priority. This prevents fixing obscure one-off patterns when high-impact shared functions have bugs.

**Example application**:

| Function | Suspiciousness | Call Count | Priority | Action |
|----------|---------------|------------|----------|--------|
| `emit_arithmetic` | 0.89 | 45 | 4.94 | Fix immediately |
| `emit_for_range` | 0.72 | 38 | 3.97 | Fix next |
| `emit_heredoc` | 0.68 | 3 | 1.36 | Defer |
| `emit_assignment` | 0.02 | 120 | 0.14 | Stable |

#### 11.10.4 Weak Supervision and Error Deduplication

**Source**: `depyler-oracle/src/corpus_extract.rs`

Depyler deduplicates training errors by hashing `(error_code, message)` and tracks extraction cycles. This prevents the same shellcheck warning from inflating training data.

**Adaptation to bashrs**:

```rust
struct ShellTrainingError {
    /// Shellcheck code or execution failure type
    error_code: String,
    /// Error message (normalized — paths and line numbers stripped)
    message: String,
    /// Deduplication hash
    hash: u64,
    /// Which corpus run discovered this error
    cycle: u32,
    /// Risk classification (programmatic labeling)
    risk: RiskLevel,  // HIGH, MEDIUM, LOW
}

enum RiskLevel {
    /// Security-relevant (injection, unquoted expansion in eval)
    High,
    /// Correctness-relevant (wrong output, behavioral mismatch)
    Medium,
    /// Style/lint (shellcheck warnings that don't affect behavior)
    Low,
}
```

**Programmatic labeling rules** (weak supervision à la Snorkel, Ratner et al. 2017):

| Rule | Condition | Label |
|------|-----------|-------|
| SEC_RULE | error_code matches SEC001-SEC008 | HIGH |
| B3_FAIL | entry has B3 behavioral failure | HIGH |
| G_FAIL | entry has cross-shell disagreement (sh vs dash) | MEDIUM |
| LINT_ONLY | only shellcheck style warnings, B3 passes | LOW |
| QUOTING | error_code is SC2086 (unquoted variable) | MEDIUM |

This automated triage ensures fix effort is directed at high-risk failures first, following the Pareto principle (Juran, 1951): 80% of user-visible defects come from 20% of error categories.

#### 11.10.5 Multi-Corpus Convergence Dashboard

**Source**: depyler `improve-converge.md` (17 iterations tracked)

Depyler tracks per-tier compile rates across 5 independent corpora at each iteration, with root cause analysis tables. Bashrs should adopt the same granular tracking.

**Proposed convergence table format**:

| Iteration | Date | Bash (350) | Makefile (150) | Dockerfile (150) | Total | Score | Notes |
|-----------|------|-----------|---------------|------------------|-------|-------|-------|
| 14 | 2026-02-07 | 349/350 | 150/150 | 150/150 | 649/650 | 99.9 | B-143 only failure |
| 15 | 2026-02-08 | 349/350 | 150/150 | 150/150 | 649/650 | 99.9 | +30 OIP entries |
| 16 | TBD | ? | ? | ? | ? | ? | CITL-driven entries |

Each iteration records:
- **Per-format pass rates** (not just aggregate)
- **New entries added** (append-only count)
- **Failures fixed** (transpiler changes)
- **Root cause** for any new failures introduced

This enables detection of **format-specific regressions** — a Makefile fix that accidentally breaks Bash entries would be immediately visible.

#### 11.10.6 Implementation Roadmap

| Phase | Technique | Effort | Prerequisite | Expected Impact |
|-------|-----------|--------|-------------|-----------------|
| 1 | Error deduplication + weak supervision (11.10.4) | 1 week | None | Prioritized fix backlog |
| 2 | Decision tracing in emitter (11.10.1 prerequisite) | 2 weeks | None | Enables Tarantula + CITL |
| 3 | Tarantula fault localization (11.10.1) | 1 week | Phase 2 | Data-driven prioritization |
| 4 | CITL pattern store (11.10.2) | 2 weeks | Phases 2-3 | Automated fix suggestions |
| 5 | Graph-aware prioritization (11.10.3) | 1 week | Phase 3 | Impact-weighted triage |
| 6 | Convergence dashboard (11.10.5) | 3 days | None | Regression visibility |

Phase 1 and Phase 6 are independent and can start immediately. Phases 2-5 are sequential.

### 11.11 Domain-Specific Corpus Categories

The corpus must cover three domain-specific categories that standard tier progression misses. These represent real-world usage patterns where shell scripts are most commonly written and maintained, and where transpiler correctness has the highest practical impact.

#### 11.11.1 Category A: Shell Configuration Files (bashrc/zshrc/profile)

**Motivation**: Shell config files (`.bashrc`, `.zshrc`, `.profile`, `/etc/environment`) are the most-edited shell scripts in existence. Every developer maintains at least one. They have unique patterns:

- **PATH manipulation**: Append/prepend directories, deduplication, conditional addition
- **Alias definitions**: Simple and complex aliases with quoting challenges
- **Environment exports**: `export VAR=value` chains, conditional exports
- **Prompt customization**: PS1/PS2 with escape sequences and dynamic content
- **Conditional tool setup**: `if command -v tool >/dev/null; then ... fi`
- **Source/dot inclusion**: `. ~/.bashrc.d/*.sh` sourcing patterns
- **Shell options**: `set -o`, `shopt -s`, `setopt` configuration
- **History configuration**: HISTSIZE, HISTFILESIZE, HISTCONTROL

**Corpus Entry Pattern**: Rust DSL representing config-style shell constructs. The transpiler should emit clean, idempotent config blocks suitable for inclusion in rc files.

**Unique Quality Requirements**:
- **Idempotent**: Sourcing the config twice must be safe (no duplicate PATH entries)
- **Non-destructive**: Config blocks must not overwrite user state (use `${VAR:-default}`)
- **POSIX-portable**: Must work when sourced by sh, bash, zsh, and dash

**Entry Range**: B-371..B-380

#### 11.11.2 Category B: Shell One-Liners (bash/sh/zsh)

**Motivation**: Shell one-liners are the most common ad-hoc shell usage. They compress complex operations into single pipeline expressions. The transpiler must produce output that captures the *intent* of these patterns even when the Rust DSL input is multi-statement.

**Key Patterns**:
- **Pipeline chains**: `cmd1 | cmd2 | cmd3` — data flows through filters
- **Find-exec patterns**: `find . -name '*.log' -exec rm {} \;`
- **Xargs composition**: `cmd | xargs -I{} other {}`
- **Process substitution**: `diff <(cmd1) <(cmd2)`
- **Inline conditionals**: `test -f file && source file`
- **Redirect chains**: `cmd > out 2>&1`, `cmd 2>/dev/null`
- **Sort-uniq pipelines**: `cmd | sort | uniq -c | sort -rn | head`
- **Awk/sed transforms**: Text processing in single expressions
- **Subshell grouping**: `(cd dir && cmd)` to avoid directory pollution
- **Arithmetic expansion**: Complex `$((...))` expressions

**Corpus Entry Pattern**: Rust DSL that expresses operations typically solved by one-liners. The transpiled output should demonstrate that the transpiler can produce compact, idiomatic shell.

**Unique Quality Requirements**:
- **Behavioral equivalence**: The multi-statement Rust DSL must produce shell output that achieves the same result as the canonical one-liner
- **Pipeline safety**: No unquoted variables in pipe chains
- **Error propagation**: `set -o pipefail` equivalent semantics where applicable

**Entry Range**: B-381..B-390

#### 11.11.3 Category C: Provability Corpus (Restricted Rust → Verified Shell)

**Motivation**: The provability corpus contains entries where the Rust source is **restricted to a formally verifiable subset** — pure functions, no I/O, no unsafe, no panics. This subset can be:

1. **Verified by Miri**: Rust's mid-level IR interpreter can prove absence of undefined behavior
2. **Verified by property tests**: Exhaustive/random testing over the input domain
3. **Verified by symbolic execution**: For simple arithmetic, the Rust and shell outputs can be proven equivalent

**Restricted Rust Subset** (allowed constructs):
- Pure functions (`fn f(x: i32) -> i32`)
- Integer arithmetic (`+`, `-`, `*`, `/`, `%`)
- Boolean logic (`&&`, `||`, `!`)
- Conditionals (`if`/`else`)
- Bounded loops (`for i in 0..n`, `while i < n`)
- Local variables only (no globals, no statics, no heap)
- No I/O, no `println!`, no `eprintln!`
- No `unsafe`, no `unwrap`, no `expect`, no `panic!`

**Provability Chain**:
```
Rust source (restricted subset)
  │
  ├── Miri verification: cargo miri run (proves no UB)
  ├── Property test: proptest over input domain
  │
  ▼
Shell output (transpiled)
  │
  ├── Behavioral test: sh -c "$script" produces same result
  ├── Cross-shell: sh, dash, bash agree
  │
  ▼
Equivalence: Rust output ≡ Shell output (for all tested inputs)
```

**Why This Matters**: The provability corpus establishes a **trusted kernel** — a set of entries where correctness is not just tested but *proven*. This kernel serves as the foundation for confidence in the transpiler. If the transpiler is correct on provably-correct Rust, we have high confidence it's correct on general Rust.

**Corpus Entry Pattern**: Pure Rust functions with known-correct outputs. Expected shell output is derived from the Rust semantics (not observed from the transpiler). This makes the corpus truly **falsifying** — it can catch transpiler bugs that other entries cannot.

**Unique Quality Requirements**:
- **Miri-clean**: `cargo miri run` passes on the Rust source (no UB)
- **Deterministic**: Pure functions produce identical output every run
- **Exhaustively testable**: Small input domains allow full enumeration
- **No shell-isms**: Output must not rely on shell-specific behavior (e.g., string-as-boolean)

**Entry Range**: B-391..B-400

#### 11.11.4 Cross-Category Quality Matrix

| Property | Config (A) | One-liner (B) | Provability (C) |
|----------|-----------|--------------|----------------|
| Idempotent | REQUIRED | N/A | REQUIRED |
| POSIX | REQUIRED | REQUIRED | REQUIRED |
| Deterministic | REQUIRED | REQUIRED | REQUIRED |
| Miri-verifiable | N/A | N/A | REQUIRED |
| Cross-shell | REQUIRED | REQUIRED | REQUIRED |
| Shellcheck-clean | REQUIRED | REQUIRED | REQUIRED |
| Pipeline-safe | N/A | REQUIRED | N/A |

---

## 12. References

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

### v2 References (Quantifiable Correctness)

14. **Avizienis, A.** (1985). "The N-Version Approach to Fault-Tolerant Software." *IEEE Transactions on Software Engineering*, SE-11(12), 1491-1501. DOI: 10.1109/TSE.1985.232116. (N-version programming for fault detection through implementation diversity.)

15. **Barr, E. T., Harman, M., McMinn, P., Shahbaz, M., & Yoo, S.** (2015). "The Oracle Problem in Software Testing: A Survey." *IEEE Transactions on Software Engineering*, 41(5), 507-525. DOI: 10.1109/TSE.2014.2372785. (Comprehensive taxonomy of test oracle approaches including specified, derived, implicit, and human oracles.)

16. **Breiman, L.** (2001). "Random Forests." *Machine Learning*, 45(1), 5-32. DOI: 10.1023/A:1010933404324. (Foundational paper on Random Forest ensemble method; demonstrates lower generalization error through bagging and feature subsampling.)

17. **Chen, T. Y., Kuo, F.-C., Liu, H., Poon, P.-L., Towey, D., Tse, T. H., & Zhou, Z. Q.** (2018). "Metamorphic Testing: A Review of Challenges and Opportunities." *ACM Computing Surveys*, 51(1), Article 4. DOI: 10.1145/3143561. (Definitive survey on metamorphic testing for alleviating the oracle problem; defines metamorphic relations as necessary properties across related test inputs.)

18. **Dietterich, T. G.** (2000). "Ensemble Methods in Machine Learning." *Multiple Classifier Systems (MCS 2000)*, LNCS 1857, 1-15. Springer. DOI: 10.1007/3-540-45014-9_1. (Theoretical basis for combining k-NN with Random Forest; bias-variance decomposition of ensemble error.)

19. **Gama, J., Žliobaitė, I., Bifet, A., Pechenizkiy, M., & Bouchachia, A.** (2014). "A Survey on Concept Drift Adaptation." *ACM Computing Surveys*, 46(4), Article 44. DOI: 10.1145/2523813. (Concept drift detection methods for monitoring oracle accuracy degradation over time.)

20. **Huang, K., et al.** (2024). "Revisiting Code Similarity Evaluation with Abstract Syntax Tree Edit Distance." *arXiv preprint* arXiv:2404.08817. (Demonstrates AST edit distance as superior to token-level comparison for measuring code structural equivalence.)

21. **Malhotra, R.** (2015). "A Systematic Review of Machine Learning Techniques for Software Fault Prediction." *Applied Soft Computing*, 27, 504-518. DOI: 10.1016/j.asoc.2014.11.023. (Meta-analysis showing Random Forest and ensemble methods achieve 75-85% accuracy on software defect prediction benchmarks including NASA datasets.)

22. **McKeeman, W. M.** (1998). "Differential Testing for Software." *Digital Technical Journal*, 10(1), 100-107. (Seminal work on using multiple implementations as cross-referencing oracles; directly applicable to cross-shell validation of transpiled output.)

23. **Settles, B.** (2012). *Active Learning*. Synthesis Lectures on Artificial Intelligence and Machine Learning. Morgan & Claypool. ISBN: 978-1608457250. (Active learning for efficient labeling of corpus failure examples when training data is scarce.)

24. **Zhang, K. & Shasha, D.** (1989). "Simple Fast Algorithms for the Editing Distance Between Trees and Related Problems." *SIAM Journal on Computing*, 18(6), 1245-1262. DOI: 10.1137/0218082. (Polynomial-time algorithm for tree edit distance; basis for AST structural comparison in Level 2 correctness measurement.)

25. **Chen, J., Patra, J., Pradel, M., Xiong, Y., Zhang, H., Hao, D., & Zhang, L.** (2020). "A Survey of Compiler Testing." *ACM Computing Surveys*, 53(1), Article 4. DOI: 10.1145/3363562. (Survey of compiler testing techniques including differential testing, metamorphic testing, and EMI; relevant methodology for transpiler validation.)

### v2.1 References (Cross-Project Techniques, Section 11.10)

26. **Jones, J. A. & Harrold, M. J.** (2005). "Empirical Evaluation of the Tarantula Automatic Fault-Localization Technique." *Proceedings of the 20th IEEE/ACM International Conference on Automated Software Engineering (ASE)*, 273-282. DOI: 10.1145/1101908.1101949. (Tarantula suspiciousness scoring for fault localization; applied to transpiler decision tracing in Section 11.10.1.)

27. **Zeller, A.** (2002). "Isolating Cause-Effect Chains from Computer Programs." *Proceedings of the 10th ACM SIGSOFT Symposium on Foundations of Software Engineering (FSE)*, 1-10. DOI: 10.1145/587051.587053. (Delta debugging and cause-effect chain isolation; theoretical basis for CITL pattern mining in Section 11.10.2.)

28. **Ratner, A., Bach, S. H., Ehrenberg, H., Fries, J., Wu, S., & Ré, C.** (2017). "Snorkel: Rapid Training Data Creation with Weak Supervision." *Proceedings of the VLDB Endowment*, 11(3), 269-282. DOI: 10.14778/3157794.3157797. (Programmatic labeling functions for weak supervision; applied to error risk classification in Section 11.10.4.)

29. **Nonaka, I. & Takeuchi, H.** (1995). *The Knowledge-Creating Company: How Japanese Companies Create the Dynamics of Innovation*. Oxford University Press. ISBN: 978-0195092691. (Organizational knowledge transfer; basis for cross-project technique adoption in Section 11.10.)

### Project-Specific

30. **Gift, N.** (2025). "Depyler Corpus Registry and Convergence Methodology." Internal specification, paiml/depyler. (Corpus registry pattern, 100-point scoring system, multi-tier measurement.)

31. **Gift, N.** (2026). "Depyler Oracle: CITL Pattern Mining, Tarantula Fault Localization, and Graph-Aware Corpus." Internal implementation, paiml/depyler `crates/depyler-oracle/`. (Source implementations for Sections 11.10.1-11.10.3.)

32. **bashrs CLAUDE.md** (2024-2026). Project development guidelines. (EXTREME TDD, STOP THE LINE, assert_cmd mandate, unwrap policy.)

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

Corpus size: 30   100  100  200  200  250  350  350  400  500  500  550  600  620
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
