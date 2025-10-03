# Rash Full Testing & Quality Specification
## Modern Type-Safe Shell Transpiler Testing Infrastructure

**Version:** 1.0.0  
**Date:** October 2, 2025  
**Authors:** Rash Core Team  
**Status:** RFC - Request for Comments

---

## Executive Summary

This specification defines a comprehensive testing and quality assurance framework for Rash, a Rust-to-POSIX shell transpiler. Drawing inspiration from modern language ecosystems (Deno, Ruby), we establish four core quality commands that enable rigorous verification of transpiled shell scripts while maintaining the safety guarantees of the Rust source.

**Core Quality Commands:**
- `rash coverage` - Measure code coverage with line/branch/function granularity
- `rash score` - Quantitative code quality metrics and maintainability index
- `rash lint` - Static analysis with ShellCheck integration
- `rash compile` - Safe transpilation with verification pipeline

This framework enforces cyclomatic complexity <10 per function and employs extreme Test-Driven Development (TDD) with property-based testing to ensure correctness.

---

## 1. Research Foundation & Academic Context

### 1.1 Shell Script Static Analysis

Recent academic research demonstrates shell scripts can be statically analyzed despite their dynamic nature. Vasilakis et al. (2025) propose semantics-driven static analysis for Unix shell programs, enabling pre-runtime verification previously unavailable for shell environments. Their work addresses the "pervasive dynamicity" and opaque polyglot commands that have historically prevented formal analysis.

**Key Research Citations:**

1. **Vasilakis, N., Lazarek, L., Jung, S-H., et al.** (2025). "From Ahead-of- to Just-in-Time and Back Again: Static Analysis for Unix Shell Programs." *HotOS XX Conference*. [Forthcoming]
   - Establishes feasibility of static analysis for shell scripts
   - Proposes effect analysis for filesystem interactions
   - Introduces type systems centered around regular types for pipe-and-filter computations

2. **McCabe, T.J.** (1976). "A Complexity Measure." *IEEE Transactions on Software Engineering*, SE-2(4), 308-320.
   - Defines cyclomatic complexity metric: M = E - N + 2P
   - Establishes threshold of 10 for maintainable code
   - NIST235 confirms 10 as appropriate limit, with 15 acceptable only for experienced teams with formal design processes

3. **Claessen, K., & Hughes, J.** (2000). "QuickCheck: A Lightweight Tool for Random Testing of Haskell Programs." *ICFP 2000*.
   - Foundational work on property-based testing
   - Establishes automated test case generation from properties
   - Demonstrates effectiveness in compiler verification

### 1.2 Code Coverage for Shell Scripts

ShellSpec, released in 2019, provides full-featured BDD testing for POSIX shells with code coverage, mocking, and parallel execution capabilities. Bashcov uses SimpleCov to generate HTML coverage reports, automatically caching and merging results across test suites.

**Technical Approach:** Coverage tools leverage bash's `BASH_XTRACEFD` variable and `set -x` tracing to capture execution paths. Dynamic analysis through instrumented runs provides practical coverage measurement, though static analysis remains desirable for test-independent verification.

### 1.3 Cyclomatic Complexity Metrics

ShellMetrics (2023) measures NLOC (Non-comment Lines of Code), LLOC (Logical Lines of Code), and CCN (Cyclomatic Complexity Number) for bash, mksh, yash, and zsh scripts. The tool itself maintains CCN <10, demonstrating feasibility of low-complexity shell scripting.

Cyclomatic complexity M = E - N + P, where E=edges, N=nodes, P=connected components in control flow graph. For structured programs with single entry/exit: M = decision_points + 1.

---

## 2. Architecture Overview

### 2.1 Design Principles

1. **Zero Runtime Dependencies**: All generated scripts are pure POSIX shell
2. **Deterministic Output**: Same input → identical transpiled code
3. **Verifiable Correctness**: Formal verification pipeline with property-based tests
4. **Incremental Complexity**: Every function maintains CCN <10
5. **Fail-Fast Philosophy**: Errors caught at compile time, not runtime

### 2.2 Tool Pipeline

```
┌─────────────────┐
│  Rust Source    │
│  (.rs files)    │
└────────┬────────┘
         │
         ▼
    ┌────────┐
    │ Parser │ ◄──── AST Verification
    └────┬───┘
         │
         ▼
  ┌──────────────┐
  │  Transpiler  │ ◄──── Type Safety Layer
  └──────┬───────┘
         │
         ▼
┌──────────────────┐
│  POSIX Shell     │
│  (.sh files)     │
└────────┬─────────┘
         │
    ┌────┴────┬────────┬────────┬─────────┐
    │         │        │        │         │
    ▼         ▼        ▼        ▼         ▼
┌────────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐
│Coverage│ │Score │ │ Lint │ │ Test │ │Verify│
└────────┘ └──────┘ └──────┘ └──────┘ └──────┘
```

### 2.3 Quality Dimensions Matrix

| Dimension | Metric | Target | Tool | Enforcement |
|-----------|--------|--------|------|-------------|
| Coverage | Line Coverage | ≥90% | `rash coverage` | CI Gate |
| Coverage | Branch Coverage | ≥85% | `rash coverage` | CI Gate |
| Coverage | Function Coverage | 100% | `rash coverage` | CI Gate |
| Complexity | Cyclomatic Complexity | <10 per function | `rash score` | Compile Error |
| Complexity | LLOC | <50 per function | `rash score` | Warning |
| Quality | ShellCheck Score | 0 issues | `rash lint` | Compile Error |
| Safety | Injection Vulnerabilities | 0 | `rash lint` | Compile Error |
| Correctness | Property Tests Pass | 100% | `rash test` | CI Gate |

---

## 3. Component Specifications

### 3.1 rash coverage

**Command:** `rash coverage [options] <script.rs>`

**Purpose:** Measure code coverage of transpiled shell scripts through dynamic analysis with kcov integration.

#### 3.1.1 Technical Implementation

**Coverage Measurement Strategy:**

1. **Instrumentation Phase**
   - Parse Rust AST to identify all executable statements
   - Generate line number mappings: Rust line → Shell line
   - Insert tracing hooks at function boundaries
   - Create execution manifest for verification

2. **Execution Phase**
   - Leverage bash `BASH_XTRACEFD` for execution tracing
   - Capture function entry/exit events
   - Record branch evaluation outcomes
   - Monitor subprocess creation

3. **Analysis Phase**
   - Correlate trace data with source maps
   - Calculate coverage metrics:
     - Line coverage: executed_lines / total_lines
     - Branch coverage: taken_branches / total_branches
     - Function coverage: called_functions / total_functions
   - Generate HTML reports via modified SimpleCov

**Integration with kcov:**
```bash
rash coverage src/main.rs
# Compiles to: /tmp/rash/main.sh
# Executes: kcov --bash-dont-parse-binary-dir coverage/ /tmp/rash/main.sh
# Outputs: coverage/index.html with source mappings
```

**Data Structures:**
```rust
struct CoverageReport {
    line_coverage: f64,        // Percentage
    branch_coverage: f64,      // Percentage
    function_coverage: f64,    // Percentage
    uncovered_lines: Vec<LineInfo>,
    uncovered_branches: Vec<BranchInfo>,
    source_map: SourceMap,     // Rust → Shell mapping
}

struct SourceMap {
    rust_to_shell: HashMap<RustLocation, ShellLocation>,
    shell_to_rust: HashMap<ShellLocation, RustLocation>,
}
```

**Output Format:**
- **Console:** Summary table with pass/fail indicators
- **HTML:** Interactive source view with heat maps
- **JSON:** Machine-readable for CI/CD integration
- **LCOV:** For Codecov/Coveralls integration

**CLI Options:**
```bash
rash coverage --format=html        # HTML report (default)
rash coverage --format=json        # JSON for CI
rash coverage --format=lcov        # LCOV for external tools
rash coverage --min-coverage=90    # Fail if below threshold
rash coverage --include-stdlib     # Include stdlib coverage
rash coverage --exclude=test/*     # Exclude patterns
```

#### 3.1.2 Test Strategy

**Unit Tests:**
- Test SourceMap bidirectional mapping correctness
- Verify line counting accuracy
- Validate branch identification in AST

**Integration Tests:**
- Full coverage measurement on sample programs
- Edge cases: empty functions, nested conditionals
- Property test: coverage ∈ [0, 100]

**Complexity Target:** Each function CCN ≤ 7

---

### 3.2 rash score

**Command:** `rash score [options] <script.rs>`

**Purpose:** Calculate comprehensive code quality metrics including cyclomatic complexity, maintainability index, and LLOC.

#### 3.2.1 Metrics Calculated

1. **Cyclomatic Complexity (CCN)**
   - Formula: M = E - N + 2P (McCabe, 1976)
   - Per-function calculation
   - Project aggregate: weighted average by LLOC
   - **Enforcement:** Compilation fails if any function CCN >10

2. **Maintainability Index (MI)**
   - Formula: MI = 171 - 5.2×ln(V) - 0.23×G - 16.2×ln(LOC)
   - Where: V=Halstead Volume, G=Cyclomatic Complexity, LOC=Lines of Code
   - Range: 0-100 (100 = most maintainable)
   - Target: MI ≥ 70

3. **Logical Lines of Code (LLOC)**
   - Excludes: comments, blank lines, braces-only lines
   - Per-function and project totals
   - Target: ≤50 LLOC per function

4. **Code Duplication**
   - Token-based comparison (≥6 token sequences)
   - Reports duplicated blocks and percentage
   - Target: <3% duplication

5. **Comment Ratio**
   - Comments / (Comments + Code)
   - Target: 15-25% (too low = unclear, too high = over-documented)

#### 3.2.2 Score Calculation Algorithm

```rust
fn calculate_score(ast: &RustAST) -> QualityScore {
    let functions = extract_functions(ast);
    let mut scores = Vec::new();
    
    for func in functions {
        let cfg = build_control_flow_graph(func);
        let ccn = calculate_cyclomatic_complexity(&cfg);
        
        if ccn > 10 {
            return Err(ComplexityError {
                function: func.name,
                complexity: ccn,
                threshold: 10,
            });
        }
        
        let lloc = count_logical_lines(func);
        let halstead = calculate_halstead_metrics(func);
        let mi = calculate_maintainability_index(lloc, ccn, halstead);
        
        scores.push(FunctionScore { func.name, ccn, lloc, mi });
    }
    
    QualityScore {
        functions: scores,
        aggregate: calculate_aggregate(&scores),
        duplication: detect_duplication(ast),
    }
}
```

#### 3.2.3 Report Format

**Console Output:**
```
Code Quality Score Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Project: rash-transpiler v1.0.0

Overall Score: 87/100 (Good)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Metrics:
  Cyclomatic Complexity (avg):  6.2   ✓ (target: <10)
  Maintainability Index:       78.5   ✓ (target: ≥70)
  Code Duplication:             1.8%  ✓ (target: <3%)
  Comment Ratio:               18.3%  ✓ (target: 15-25%)

Functions by Complexity:
┌────────────────────┬──────┬──────┬──────┐
│ Function           │ CCN  │ LLOC │  MI  │
├────────────────────┼──────┼──────┼──────┤
│ parse_expression   │  8   │  45  │ 72.1 │
│ transpile_match    │  7   │  38  │ 75.3 │
│ emit_function      │  4   │  22  │ 82.6 │
│ validate_types     │  6   │  34  │ 76.9 │
└────────────────────┴──────┴──────┴──────┘

Warnings: 0  |  Errors: 0
```

**JSON Output:**
```json
{
  "project": "rash-transpiler",
  "version": "1.0.0",
  "timestamp": "2025-10-02T10:15:30Z",
  "overall_score": 87,
  "metrics": {
    "cyclomatic_complexity": {
      "average": 6.2,
      "max": 8,
      "threshold": 10,
      "status": "pass"
    },
    "maintainability_index": 78.5,
    "code_duplication_percent": 1.8,
    "comment_ratio": 18.3
  },
  "functions": [
    {
      "name": "parse_expression",
      "loc": 52,
      "lloc": 45,
      "ccn": 8,
      "mi": 72.1,
      "halstead_volume": 458.3
    }
  ]
}
```

#### 3.2.4 Test Strategy

**Property-Based Tests:**
- ∀ function f: CCN(f) ≥ 1 (at minimum, entry/exit nodes)
- ∀ function f: MI(f) inversely correlates with CCN(f)
- Duplication detection is symmetric: dup(A,B) = dup(B,A)

**Unit Tests:**
- CFG construction correctness
- Edge/node counting for known graphs
- Halstead metric calculation validation

**Complexity Target:** Each scoring function CCN ≤ 6

---

### 3.3 rash lint

**Command:** `rash lint [options] <script.rs>`

**Purpose:** Static analysis combining Rust's type safety with ShellCheck integration for the transpiled output.

#### 3.3.1 Multi-Layer Linting Architecture

**Layer 1: Rust-Level Analysis**
- Leverage rustc's borrow checker
- Custom lint passes for shell-specific patterns
- Detect unsafe patterns before transpilation

**Layer 2: Transpilation Validation**
- Verify POSIX compliance of generated code
- Ensure no dynamic evaluation (`eval`, `source` of variables)
- Validate proper quoting and escaping

**Layer 3: ShellCheck Integration**
- ShellCheck is an open-source static analysis tool that detects bugs, portability issues, and syntax errors in shell scripts
- Run on transpiled output
- Zero tolerance policy: all issues must be resolved

#### 3.3.2 Custom Lint Rules

**Security Rules:**
1. **No Command Injection (RASH-S001)**
   ```rust
   // BAD: User input in command
   let user_input = std::env::var("USER_FILE")?;
   sh::exec!("cat {}", user_input); // LINT ERROR
   
   // GOOD: Validated and escaped
   let user_input = sh::validate_path(std::env::var("USER_FILE")?)?;
   sh::exec!("cat {}", sh::escape(user_input));
   ```

2. **No Eval of Dynamic Content (RASH-S002)**
   ```rust
   // Forbidden: eval-like constructs never allowed
   sh::eval(runtime_string); // COMPILE ERROR
   ```

3. **Proper Error Handling (RASH-E001)**
   ```rust
   // BAD: Ignoring errors
   sh::exec!("important_command");
   
   // GOOD: Explicit error handling
   sh::exec!("important_command")?;
   // or
   if !sh::exec!("optional_command").is_ok() {
       sh::warn("Optional command failed");
   }
   ```

**Portability Rules:**
1. **POSIX Compliance (RASH-P001)**
   - No bashisms unless explicitly declared
   - Detect GNU-specific commands
   - Warn on non-portable flags

2. **Dependency Declaration (RASH-P002)**
   - All external commands must be declared
   - Generate dependency manifest
   - Runtime availability checks

#### 3.3.3 ShellCheck Integration

**Configuration:**
```yaml
# .rash-lint.yml
shellcheck:
  severity: error
  exclude: []
  shell: sh        # Force POSIX compliance
  enable:
    - require-variable-braces
    - quote-safe-variables
    
custom_rules:
  - RASH-S001: error
  - RASH-S002: error
  - RASH-E001: warning
  - RASH-P001: warning
```

**Integration Flow:**
```rust
fn lint_file(rust_file: &Path) -> LintResult {
    // Stage 1: Rust analysis
    let rust_errors = rustc_lint(rust_file)?;
    
    // Stage 2: Transpile with validation
    let shell_script = transpile_with_validation(rust_file)?;
    
    // Stage 3: ShellCheck
    let shellcheck_output = run_shellcheck(&shell_script)?;
    
    // Aggregate results
    combine_lint_results(rust_errors, shellcheck_output)
}
```

#### 3.3.4 Output Format

```
Linting: src/deploy.rs
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✓ Rust Analysis: No issues
✓ Transpilation: Valid POSIX shell
✗ ShellCheck: 2 issues found

Errors:
  deploy.rs:45:12 - RASH-S001 (security)
    Potential command injection vulnerability
    │ 
  45│     sh::exec!("rm -rf {}", user_path);
    │                           ^^^^^^^^^
    │ 
    Help: Use sh::validate_path() or sh::escape()

  deploy.rs:67:8 - SC2086 (shellcheck)
    Double quote to prevent globbing and word splitting
    │ 
  67│     mv $tmp_file /etc/config
    │        ^^^^^^^^^
    │ 
    Transpiled to: mv $tmp_file /etc/config
    Help: Change to: mv "$tmp_file" /etc/config

Summary: 2 errors, 0 warnings
Status: FAILED
```

#### 3.3.5 Test Strategy

**Security Test Suite:**
- Injection attempts: SQL, command, path traversal
- Property test: ∀ input, no `eval` in output
- Fuzzing with malicious inputs

**Portability Tests:**
- Test on: dash, busybox sh, bash, zsh
- Verify POSIX compliance on all platforms
- Property test: output ∈ POSIX subset

**Complexity Target:** Each lint rule CCN ≤ 5

---

### 3.4 rash compile

**Command:** `rash compile [options] <script.rs>`

**Purpose:** Safe transpilation from Rust to POSIX shell with comprehensive verification pipeline.

#### 3.4.1 Compilation Pipeline

**Phase 1: Parse & Validate**
```rust
fn parse_rust_source(file: &Path) -> Result<RustAST> {
    let ast = rustc_parse(file)?;
    validate_rash_subset(&ast)?;  // Ensure only safe subset used
    Ok(ast)
}
```

**Phase 2: Type Checking**
```rust
fn type_check(ast: &RustAST) -> Result<TypedAST> {
    let types = infer_types(ast)?;
    verify_shell_compatibility(types)?;  // Shell has no types at runtime
    Ok(TypedAST { ast, types })
}
```

**Phase 3: IR Generation**
```rust
fn generate_ir(typed_ast: TypedAST) -> Result<IR> {
    let ir = lower_to_ir(typed_ast)?;
    optimize_for_shell(&ir)?;
    verify_ir_invariants(&ir)?;
    Ok(ir)
}
```

**Phase 4: Code Generation**
```rust
fn emit_shell(ir: &IR) -> Result<ShellScript> {
    let shell = emit_posix_shell(ir)?;
    validate_syntax(&shell)?;
    beautify_output(&shell)?;
    Ok(shell)
}
```

**Phase 5: Verification**
```rust
fn verify_output(rust: &Path, shell: &ShellScript) -> Result<()> {
    run_shellcheck(shell)?;
    verify_complexity(shell)?;     // All functions CCN <10
    verify_security(shell)?;       // No injection risks
    property_test(rust, shell)?;   // Behavioral equivalence
    Ok(())
}
```

#### 3.4.2 Supported Rust Subset

**Core Language Features:**
```rust
// ✓ Variables and basic types
let x = 42;
let name = "Alice";
let is_active = true;

// ✓ Control flow
if condition { ... } else { ... }
match value { ... }
for item in items { ... }
while condition { ... }

// ✓ Functions
fn process(input: &str) -> Result<String> { ... }

// ✓ String manipulation
format!("Hello, {}", name)
input.trim().to_uppercase()

// ✓ File operations
std::fs::read_to_string(path)?
std::fs::write(path, contents)?

// ✓ Process execution
sh::exec!("command", args)?
sh::pipe!(cmd1, cmd2, cmd3)?

// ✗ NOT SUPPORTED (compilation error)
// - Heap allocations (Vec, Box, Rc)
// - Async/await
// - Traits and generics
// - Closures (except simple cases)
// - FFI
```

#### 3.4.3 Optimization Passes

1. **Dead Code Elimination**
   - Remove unused functions
   - Eliminate unreachable branches

2. **Constant Folding**
   - Evaluate constant expressions at compile time
   - Reduce shell arithmetic overhead

3. **Function Inlining**
   - Inline small functions (<5 LLOC)
   - Only if doesn't increase calling function's CCN

4. **Command Coalescing**
   - Combine multiple simple commands
   - Reduce subprocess overhead

#### 3.4.4 Error Messages

```
error[RASH-C001]: unsupported feature
  --> src/main.rs:12:5
   |
12 |     let numbers: Vec<i32> = vec![1, 2, 3];
   |                  ^^^^^^^^ heap allocation not supported
   |
   = note: Shell scripts have no heap; use fixed arrays
   = help: Replace with: let numbers = [1, 2, 3];

error[RASH-C002]: complexity too high
  --> src/deploy.rs:45:1
   |
45 | fn deploy_application(...) {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^ cyclomatic complexity: 14
   |
   = note: Maximum allowed: 10
   = help: Split into smaller functions:
           - fn validate_inputs() -> Result<()>
           - fn prepare_environment() -> Result<()>
           - fn execute_deployment() -> Result<()>
```

#### 3.4.5 Property-Based Compilation Tests

**Correctness Properties:**

1. **Determinism**
   ```rust
   property: ∀ rust_file,
       compile(rust_file) == compile(rust_file)
   ```

2. **Idempotence**
   ```rust
   property: ∀ rust_file,
       let s1 = compile(rust_file);
       let s2 = compile(compile(rust_file));
       s1 == s2
   ```

3. **POSIX Compliance**
   ```rust
   property: ∀ rust_file,
       let shell = compile(rust_file);
       ∀ posix_shell in [dash, bash, busybox],
           can_execute(posix_shell, shell)
   ```

4. **Complexity Guarantee**
   ```rust
   property: ∀ rust_file,
       let shell = compile(rust_file);
       ∀ function in shell,
           ccn(function) <= 10
   ```

5. **Security Guarantee**
   ```rust
   property: ∀ rust_file,
       let shell = compile(rust_file);
       ¬contains(shell, "eval") ∧
       ¬contains(shell, unquoted_expansion) ∧
       ∀ command in shell,
           is_sanitized(command)
   ```

#### 3.4.6 Test Strategy

**Unit Tests:** Each compilation phase independently
**Integration Tests:** End-to-end compilation
**Property Tests:** All correctness properties above
**Fuzzing:** Random valid Rust programs
**Regression Tests:** Previously found bugs

**Complexity Target:** Each compiler phase CCN ≤ 8

---

## 4. Test-Driven Development Methodology

### 4.1 TDD Philosophy for Compiler Development

TDD leads to more modularized, flexible, and extensible code by requiring developers to think in terms of small, independently testable units. For compiler development, this manifests as:

1. **Red Phase:** Write failing test for desired transpilation behavior
2. **Green Phase:** Implement minimal code to pass test
3. **Refactor Phase:** Improve code while maintaining tests and CCN <10

### 4.2 Property-Based Testing Integration

Property-based testing verifies system behavior against general properties rather than specific examples, automatically generating test cases to validate invariants across all valid inputs.

**Example Properties:**

```rust
#[quickcheck]
fn transpiled_code_is_shellcheck_clean(rust_code: ValidRustProgram) -> bool {
    let shell = rash::compile(rust_code);
    shellcheck::analyze(shell).errors.is_empty()
}

#[quickcheck]
fn all_functions_below_complexity_threshold(rust_code: ValidRustProgram) -> bool {
    let shell = rash::compile(rust_code);
    shell.functions().all(|f| f.cyclomatic_complexity() <= 10)
}

#[quickcheck]
fn output_is_deterministic(rust_code: ValidRustProgram) -> bool {
    let output1 = rash::compile(rust_code.clone());
    let output2 = rash::compile(rust_code);
    output1 == output2
}
```

### 4.3 Test Organization

```
tests/
├── unit/
│   ├── parser/
│   │   ├── test_expression_parsing.rs
│   │   ├── test_statement_parsing.rs
│   │   └── test_error_recovery.rs
│   ├── transpiler/
│   │   ├── test_type_lowering.rs
│   │   ├── test_control_flow.rs
│   │   └── test_function_emission.rs
│   └── analysis/
│       ├── test_complexity_calculation.rs
│       ├── test_coverage_tracking.rs
│       └── test_security_analysis.rs
├── integration/
│   ├── test_full_compilation.rs
│   ├── test_multi_file_projects.rs
│   └── test_stdlib_integration.rs
├── property/
│   ├── test_compilation_properties.rs
│   ├── test_correctness_properties.rs
│   └── test_security_properties.rs
└── regression/
    ├── test_issue_001_unicode.rs
    ├── test_issue_042_nested_loops.rs
    └── test_issue_089_error_messages.rs
```

### 4.4 Testing Tools & Framework

**Core Testing Stack:**
- `cargo test` - Standard Rust testing
- `quickcheck` / `proptest` - Property-based testing
- `criterion` - Performance benchmarking
- `kcov` - Code coverage measurement
- `insta` - Snapshot testing for AST/IR

**Shell Testing:**
- ShellSpec - BDD-style shell tests
- Bats - Bash Automated Testing System
- shellcheck - Static analysis verification

### 4.5 Continuous Integration

```yaml
# .github/workflows/ci.yml
name: Rash CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run unit tests
        run: cargo test --lib
      
      - name: Run integration tests
        run: cargo test --test '*'
      
      - name: Run property tests
        run: cargo test --test property -- --ignored
      
      - name: Check code coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --output-dir coverage
      
      - name: Verify coverage threshold
        run: |
          coverage=$(grep -oP 'line-rate="\K[^"]+' coverage/cobertura.xml)
          if (( $(echo "$coverage < 0.90" | bc -l) )); then
            echo "Coverage $coverage below 90% threshold"
            exit 1
          fi

  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Check complexity
        run: cargo run --bin rash-score -- --max-ccn=10 src/
      
      - name: Run linter
        run: cargo run --bin rash-lint -- src/
      
      - name: ShellCheck integration
        run: |
          cargo build --release
          ./target/release/rash compile examples/hello.rs -o /tmp/hello.sh
          shellcheck /tmp/hello.sh

  cross-platform:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        shell: [dash, bash, busybox]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - name: Test on ${{ matrix.shell }}
        run: |
          cargo build --release
          ./target/release/rash test --shell=${{ matrix.shell }}
```

---

## 5. Implementation Roadmap

### 5.1 Project Tracking

**Note on pmat:** Research did not identify "pmat" as a standard testing or project management tool. If this refers to a specific internal tool or methodology, please clarify. This roadmap uses industry-standard tools:

- **GitHub Projects** - Kanban boards and sprint planning
- **GitHub Issues** - Ticket tracking with labels
- **GitHub Actions** - CI/CD automation
- **Conventional Commits** - Semantic versioning integration

**Alternative Recommendation:** For Rust projects, consider `cargo-make` for task automation and `cargo-deny` for dependency auditing.

### 5.2 Development Phases

#### Phase 1: Foundation (Weeks 1-4)

**Epic 1.1: Core Parser Infrastructure**

- [ ] **RASH-001** Setup Rust project with `nom` parser combinator
  - Acceptance: Can parse minimal Rust subset (functions, variables)
  - Tests: Property test for parser determinism
  - Complexity: Parser functions CCN ≤ 8

- [ ] **RASH-002** Implement AST representation
  - Acceptance: Complete AST for supported Rust subset
  - Tests: AST node creation/traversal unit tests
  - Complexity: Each node type CCN ≤ 5

- [ ] **RASH-003** Error recovery and reporting
  - Acceptance: User-friendly error messages with code snippets
  - Tests: Error message snapshot tests
  - Complexity: Error formatting CCN ≤ 6

**Epic 1.2: Testing Infrastructure**

- [ ] **RASH-004** Setup quickcheck property testing
  - Acceptance: Can generate random valid Rust programs
  - Tests: Generator produces only valid programs
  - Complexity: Generator CCN ≤ 7

- [ ] **RASH-005** Implement ShellCheck integration
  - Acceptance: Can invoke shellcheck and parse results
  - Tests: Mock shellcheck output parsing
  - Complexity: Integration layer CCN ≤ 5

- [ ] **RASH-006** Coverage tracking setup
  - Acceptance: Can measure line coverage with kcov
  - Tests: Coverage measurement accuracy tests
  - Complexity: Coverage tracker CCN ≤ 6

#### Phase 2: Core Transpilation (Weeks 5-8)

**Epic 2.1: Type System Integration**

- [ ] **RASH-007** Implement type inference
  - Acceptance: Infers types for all variables
  - Tests: Type inference property tests
  - Complexity: Inference engine CCN ≤ 9

- [ ] **RASH-008** Type compatibility validation
  - Acceptance: Rejects unsupported types (Vec, Box, etc.)
  - Tests: Error cases for each unsupported type
  - Complexity: Validator CCN ≤ 7

- [ ] **RASH-009** Shell type mapping
  - Acceptance: Maps Rust types to shell equivalents
  - Tests: Mapping correctness for all types
  - Complexity: Mapping logic CCN ≤ 5

**Epic 2.2: Code Generation**

- [ ] **RASH-010** Basic statement emission
  - Acceptance: Generates POSIX shell for simple statements
  - Tests: Snapshot tests for each statement type
  - Complexity: Each emitter CCN ≤ 8

- [ ] **RASH-011** Control flow emission
  - Acceptance: if/match/loop transpilation
  - Tests: Property test for control flow equivalence
  - Complexity: Each control structure CCN ≤ 9

- [ ] **RASH-012** Function emission
  - Acceptance: Functions with params and return values
  - Tests: Function call correctness tests
  - Complexity: Function emitter CCN ≤ 8

#### Phase 3: Quality Tooling (Weeks 9-12)

**Epic 3.1: rash coverage**

- [ ] **RASH-013** Source map generation
  - Acceptance: Accurate Rust ↔ Shell line mapping
  - Tests: Mapping accuracy on 100+ programs
  - Complexity: Mapper CCN ≤ 7

- [ ] **RASH-014** Coverage report generation
  - Acceptance: HTML reports with < 1% deviation from kcov
  - Tests: Compare against kcov baseline
  - Complexity: Reporter CCN ≤ 6

- [ ] **RASH-015** CLI integration
  - Acceptance: `rash coverage` matches spec
  - Tests: CLI flag handling tests
  - Complexity: CLI parser CCN ≤ 5

**Epic 3.2: rash score**

- [ ] **RASH-016** Cyclomatic complexity calculation
  - Acceptance: Matches ShellMetrics on test suite
  - Tests: Property test: CCN ≥ 1 for all functions
  - Complexity: Calculator CCN ≤ 6

- [ ] **RASH-017** Maintainability index
  - Acceptance: Implements MI formula correctly
  - Tests: Known MI values for reference programs
  - Complexity: MI calculator CCN ≤ 5

- [ ] **RASH-018** Quality report generation
  - Acceptance: Console and JSON output per spec
  - Tests: Report formatting tests
  - Complexity: Formatter CCN ≤ 6

**Epic 3.3: rash lint**

- [ ] **RASH-019** Custom lint rule engine
  - Acceptance: Detects all RASH-S*, RASH-E*, RASH-P* rules
  - Tests: Each rule tested independently
  - Complexity: Each rule CCN ≤ 5

- [ ] **RASH-020** ShellCheck integration
  - Acceptance: Zero false positives on valid code
  - Tests: ShellCheck baseline suite passes
  - Complexity: Integration CCN ≤ 5

- [ ] **RASH-021** Lint report aggregation
  - Acceptance: Combined report format per spec
  - Tests: Report formatting tests
  - Complexity: Aggregator CCN ≤ 6

#### Phase 4: Optimization & Hardening (Weeks 13-16)

**Epic 4.1: Compiler Optimizations**

- [ ] **RASH-022** Dead code elimination
  - Acceptance: Removes all unreachable code
  - Tests: Property test: output ⊆ input (semantically)
  - Complexity: Eliminator CCN ≤ 8

- [ ] **RASH-023** Constant folding
  - Acceptance: Evaluates compile-time constants
  - Tests: Correctness tests for arithmetic
  - Complexity: Folder CCN ≤ 7

- [ ] **RASH-024** Function inlining
  - Acceptance: Inlines small functions without increasing CCN
  - Tests: CCN before/after inlining invariant
  - Complexity: Inliner CCN ≤ 9

**Epic 4.2: Security Hardening**

- [ ] **RASH-025** Injection vulnerability scanner
  - Acceptance: Detects all OWASP Top 10 shell injection patterns
  - Tests: Penetration testing suite
  - Complexity: Scanner CCN ≤ 8

- [ ] **RASH-026** Automatic escaping
  - Acceptance: All user input properly escaped
  - Tests: Fuzzing with malicious inputs
  - Complexity: Escaper CCN ≤ 6

- [ ] **RASH-027** Security audit report
  - Acceptance: Generates comprehensive security report
  - Tests: Report completeness tests
  - Complexity: Reporter CCN ≤ 6

**Epic 4.3: Documentation & Polish**

- [ ] **RASH-028** User guide and tutorials
  - Acceptance: 90%+ user comprehension in testing
  - Tests: Documentation examples auto-tested
  - Complexity: N/A

- [ ] **RASH-029** API documentation
  - Acceptance: 100% public API documented
  - Tests: `cargo doc` completeness check
  - Complexity: N/A

- [ ] **RASH-030** Error message improvements
  - Acceptance: Error messages rated 4+/5 by users
  - Tests: Error message UX study
  - Complexity: N/A

### 5.3 Definition of Done (DoD)

For each ticket to be considered complete:

1. ✅ **Code Complete**
   - Implementation matches acceptance criteria
   - All functions have CCN ≤ 10
   - No compiler warnings

2. ✅ **Tests Pass**
   - Unit tests pass (≥90% coverage)
   - Integration tests pass
   - Property tests pass (≥1000 iterations)
   - Regression tests pass

3. ✅ **Documentation**
   - Public API documented
   - Inline comments for complex logic
   - CHANGELOG.md updated

4. ✅ **Code Review**
   - Peer reviewed (≥2 approvals)
   - All comments addressed
   - CI/CD green

5. ✅ **Quality Gates**
   - `rash score` passes (≥70 MI)
   - `rash lint` clean
   - `rash coverage` ≥90%

### 5.4 Sprint Structure

**Sprint Duration:** 2 weeks

**Sprint Ceremonies:**
- **Planning:** Select tickets from roadmap, estimate complexity
- **Daily Standup:** 15min sync (async for distributed teams)
- **Mid-Sprint Review:** Check progress, adjust if needed
- **Demo:** Show completed features
- **Retrospective:** Improve process

**Velocity Tracking:**
- Track story points per sprint
- Adjust estimates based on actual complexity
- Target: Complete 1-2 epics per sprint

---

## 6. Complexity Management Strategy

### 6.1 Enforcing CCN <10

**Compile-Time Enforcement:**
```rust
// In rash compiler:
fn validate_complexity(function: &Function) -> Result<()> {
    let cfg = build_cfg(function);
    let ccn = calculate_ccn(&cfg);
    
    if ccn > 10 {
        return Err(CompilerError::ComplexityExceeded {
            function: function.name.clone(),
            actual: ccn,
            max: 10,
            suggestion: suggest_refactoring(function),
        });
    }
    
    Ok(())
}
```

**Refactoring Strategies:**

1. **Extract Function**
   ```rust
   // Before (CCN = 12)
   fn deploy() -> Result<()> {
       if validate_env().is_ok() {
           if check_disk_space().is_ok() {
               if backup_data().is_ok() {
                   // ... more logic
               }
           }
       }
       Ok(())
   }
   
   // After (CCN = 3 + 2 + 2 + 2 = 4 each)
   fn deploy() -> Result<()> {
       validate_prerequisites()?;
       perform_deployment()?;
       Ok(())
   }
   
   fn validate_prerequisites() -> Result<()> {
       validate_env()?;
       check_disk_space()?;
       Ok(())
   }
   
   fn perform_deployment() -> Result<()> {
       backup_data()?;
       deploy_application()?;
       Ok(())
   }
   ```

2. **Replace Nested Conditionals with Early Returns**
   ```rust
   // Before (CCN = 8)
   fn process(x: i32) -> Result<String> {
       if x > 0 {
           if x < 100 {
               if x % 2 == 0 {
                   Ok("even".to_string())
               } else {
                   Ok("odd".to_string())
               }
           } else {
               Err("too large")
           }
       } else {
           Err("negative")
       }
   }
   
   // After (CCN = 5)
   fn process(x: i32) -> Result<String> {
       if x <= 0 { return Err("negative"); }
       if x >= 100 { return Err("too large"); }
       
       if x % 2 == 0 {
           Ok("even".to_string())
       } else {
           Ok("odd".to_string())
       }
   }
   ```

3. **Table-Driven Logic**
   ```rust
   // Before (CCN = 11)
   fn classify(status: i32) -> &'static str {
       if status == 200 { "OK" }
       else if status == 201 { "Created" }
       else if status == 400 { "Bad Request" }
       // ... 8 more cases
       else { "Unknown" }
   }
   
   // After (CCN = 2)
   fn classify(status: i32) -> &'static str {
       let table = [
           (200, "OK"),
           (201, "Created"),
           (400, "Bad Request"),
           // ... all cases
       ];
       
       table.iter()
           .find(|(code, _)| *code == status)
           .map(|(_, msg)| *msg)
           .unwrap_or("Unknown")
   }
   ```

### 6.2 Monitoring Complexity

**Pre-commit Hook:**
```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Checking cyclomatic complexity..."
cargo run --bin rash-score -- --max-ccn=10 src/

if [ $? -ne 0 ]; then
    echo "❌ Complexity check failed!"
    echo "Please refactor functions with CCN >10"
    exit 1
fi

echo "✅ Complexity check passed"
```

**CI Dashboard:**
```
Complexity Trends
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Average CCN:      6.4 → 6.2 📉 (-3%)
Max CCN:          9   → 9   ⚪
Functions >5:     12  → 10  📉 (-17%)
Functions >8:     2   → 1   📉 (-50%)
Functions >10:    0   → 0   ✅

Historical:
  Jan 2025:  5.8 ████████████░░░░░░░░
  Feb 2025:  6.1 █████████████░░░░░░░
  Mar 2025:  6.4 ██████████████░░░░░░
  Apr 2025:  6.2 █████████████░░░░░░░ ← Now
```

---

## 7. Bibliography & References

### 7.1 Academic Papers

1. Vasilakis, N., Lazarek, L., Jung, S-H., Lamprou, E., Li, Z., Narsipur, A., Zhao, E., Greenberg, M., Kallas, K., & Mamouras, K. (2025). "From Ahead-of- to Just-in-Time and Back Again: Static Analysis for Unix Shell Programs." *HotOS XX Conference*. [Forthcoming]

2. McCabe, T.J. (1976). "A Complexity Measure." *IEEE Transactions on Software Engineering*, SE-2(4), 308-320.

3. Claessen, K., & Hughes, J. (2000). "QuickCheck: A Lightweight Tool for Random Testing of Haskell Programs." *Proceedings of ICFP 2000*, 268-279.

4. Chen, Z., O'Connor, L., Keller, G., Klein, G., & Heiser, G. (2017). "The Cogent Case for Property-Based Testing." *Workshop on Programming Languages and Operating Systems (PLOS)*, 1-7. ACM.

5. Beck, K. (2003). *Test-Driven Development: By Example*. Addison-Wesley Professional.

6. Leroy, X. (2009). "Formal verification of a realistic compiler." *Communications of the ACM*, 52(7), 107-115. [CompCert]

### 7.2 Technical Standards

1. NIST Special Publication 500-235. (1992). "Structured Testing: A Testing Methodology Using the Cyclomatic Complexity Metric."

2. IEEE Standard 1008-1987. "IEEE Standard for Software Unit Testing."

3. POSIX.1-2017 (IEEE Std 1003.1-2017). "The Open Group Base Specifications Issue 7."

### 7.3 Tools & Frameworks

1. **ShellCheck** - https://www.shellcheck.net/
   Koalaman. (2023). ShellCheck: A static analysis tool for shell scripts.

2. **ShellSpec** - https://shellspec.info/
   Full-featured BDD unit testing framework for POSIX shells.

3. **Bashcov** - https://github.com/infertux/bashcov
   Code coverage analysis tool for Bash using SimpleCov.

4. **ShellMetrics** - https://github.com/shellspec/shellmetrics
   Cyclomatic Complexity Analyzer for shell scripts.

5. **kcov** - https://github.com/SimonKagstrom/kcov
   Code coverage tool for compiled languages and shell scripts.

### 7.4 Online Resources

1. Software Assurance Technology Center (SATC), NASA. "Cyclomatic Complexity in Software Quality."

2. Martin, R. C. (2008). *Clean Code: A Handbook of Agile Software Craftsmanship*. Prentice Hall.

3. Fowler, M. (1999). *Refactoring: Improving the Design of Existing Code*. Addison-Wesley.

---

## 8. Appendices

### Appendix A: Sample Configuration Files

**`.rashrc` - Project Configuration**
```toml
[project]
name = "my-deployment-tool"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]

[compilation]
target = "posix"          # Options: posix, bash, zsh
optimization_level = 2    # 0=none, 1=basic, 2=aggressive
strict_mode = true        # Fail on warnings

[quality]
max_cyclomatic_complexity = 10
min_maintainability_index = 70
min_coverage = 90
max_duplication = 3

[linting]
enable_all = true
shellcheck_severity = "error"
custom_rules = [
  "RASH-S001",
  "RASH-S002",
  "RASH-E001",
]

[testing]
framework = "shellspec"
property_test_iterations = 1000
test_shells = ["dash", "bash", "busybox"]
```

### Appendix B: Example Rash Program

```rust
//! Simple deployment script in Rash
use rash::prelude::*;

fn main() -> Result<()> {
    validate_environment()?;
    deploy_application()?;
    verify_deployment()?;
    Ok(())
}

fn validate_environment() -> Result<()> {
    // CCN = 3
    if !sh::command_exists("git")? {
        return Err(Error::MissingDependency("git"));
    }
    
    if !sh::has_disk_space("/var", "1G")? {
        return Err(Error::InsufficientSpace);
    }
    
    Ok(())
}

fn deploy_application() -> Result<()> {
    // CCN = 4
    sh::echo("Starting deployment...")?;
    
    let repo = "https://github.com/user/app.git";
    sh::exec!("git clone {}", sh::escape(repo))?;
    
    sh::cd("/var/www/app")?;
    sh::exec!("./build.sh")?;
    
    Ok(())
}

fn verify_deployment() -> Result<()> {
    // CCN = 2
    let status = sh::exec!("curl -f http://localhost:8080/health")?;
    
    if status.success() {
        sh::echo("✅ Deployment successful")?;
        Ok(())
    } else {
        Err(Error::HealthCheckFailed)
    }
}
```

**Transpiled Output (Generated Shell Script):**
```bash
#!/bin/sh
# Generated by Rash v1.0.0
# Source: deploy.rs
# DO NOT EDIT - changes will be overwritten

set -eu

main() {
    validate_environment || return $?
    deploy_application || return $?
    verify_deployment || return $?
    return 0
}

validate_environment() {
    # CCN: 3
    if ! command -v git >/dev/null 2>&1; then
        echo "Error: Missing dependency: git" >&2
        return 1
    fi
    
    _available=$(df -h /var | awk 'NR==2 {print $4}')
    if [ "${_available%%G*}" -lt 1 ]; then
        echo "Error: Insufficient disk space" >&2
        return 1
    fi
    
    return 0
}

deploy_application() {
    # CCN: 4
    echo "Starting deployment..."
    
    _repo="https://github.com/user/app.git"
    _escaped_repo=$(printf '%s' "$_repo" | sed 's/[^a-zA-Z0-9:\/._-]/\\&/g')
    
    git clone "$_escaped_repo" || return $?
    
    cd /var/www/app || return $?
    ./build.sh || return $?
    
    return 0
}

verify_deployment() {
    # CCN: 2
    if curl -f http://localhost:8080/health >/dev/null 2>&1; then
        echo "✅ Deployment successful"
        return 0
    else
        echo "Error: Health check failed" >&2
        return 1
    fi
}

main "$@"
```

### Appendix C: Glossary

- **AST** - Abstract Syntax Tree
- **CCN** - Cyclomatic Complexity Number
- **CFG** - Control Flow Graph
- **IR** - Intermediate Representation
- **LLOC** - Logical Lines of Code
- **MI** - Maintainability Index
- **POSIX** - Portable Operating System Interface
- **TDD** - Test-Driven Development

### Appendix D: Frequently Asked Questions

**Q: Why enforce CCN <10 so strictly?**  
A: Research shows functions with CCN >10 have significantly higher defect density and are harder to test, maintain, and understand. For critical shell scripts that often run with elevated privileges, this strictness prevents catastrophic errors.

**Q: How does Rash compare to existing shell frameworks?**  
A: Rash is a *transpiler*, not a framework. It provides compile-time guarantees (type safety, injection prevention) that runtime frameworks cannot offer. The output is pure POSIX shell with no dependencies.

**Q: What's the performance overhead?**  
A: Zero runtime overhead - transpiled scripts are standard shell code. Compilation adds ~100-500ms for typical programs, acceptable for development workflows.

**Q: Can I use Rash for existing shell scripts?**  
A: No. Rash requires writing in the Rust subset. However, you can call existing shell scripts via `sh::exec!()` with proper escaping.

**Q: What if I need a feature not in the safe subset?**  
A: File an issue to discuss. We may add it if it can be transpiled safely, or provide an escape hatch with explicit unsafety marking.

---

## 9. Conclusion

This specification establishes a rigorous foundation for developing Rash's testing and quality infrastructure. By combining:

- Academic research on shell script analysis
- Property-based testing for correctness
- Extreme TDD with complexity constraints
- Modern tooling (ShellCheck, coverage, metrics)

We create a transpiler that brings Rust's safety guarantees to the shell scripting domain. The enforced CCN <10 limit ensures every component remains comprehensible and maintainable.

**Next Steps:**

1. Community review of this specification (2 weeks)
2. Form working groups for each epic
3. Setup project infrastructure (GitHub, CI/CD)
4. Begin Phase 1 implementation

**Success Criteria:**

By end of Phase 4, Rash will:
- ✅ Transpile safe Rust subset to POSIX shell
- ✅ Achieve ≥90% test coverage
- ✅ Maintain CCN <10 in all code
- ✅ Pass 100% of property-based tests
- ✅ Generate ShellCheck-clean output

---

**Document Version Control:**

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 1.0.0 | 2025-10-02 | Initial specification | Rash Core Team |

**Review Requested From:**
- Security team (injection prevention)
- Performance team (optimization strategies)
- Documentation team (user guide approach)
- Testing team (property test design)

---

*This specification is living document and will evolve based on implementation experience and community feedback.*
