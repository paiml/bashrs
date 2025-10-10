# Bash-to-Rash Test Generation Specification

**Version**: 1.0.0
**Status**: Draft
**Target**: Automatic generation of Rust code + comprehensive tests from bash scripts

## 1. Overview

This specification extends the bash-to-rash transpiler to automatically generate:
1. **Rust code** (from bash)
2. **Unit tests** (for each function/construct)
3. **Property tests** (invariants and properties)
4. **Mutation test configuration** (targeting critical paths)
5. **Doctests** (from bash comments)

### 1.1 Goals

- **Complete test coverage**: ≥80% line coverage, ≥85% mutation score
- **Automatic generation**: No manual test writing required
- **Property verification**: Generate properties from bash semantics
- **Documentation**: Doctests derived from bash comments
- **Quality gates**: All generated code passes clippy, rustfmt, and pmat

### 1.2 Success Criteria

```rust
// Input: Bash script with comments
let bash_script = r#"
#!/bin/bash
# Calculates factorial of a number
# Usage: factorial N
# Returns: N!
function factorial() {
    local n=$1
    if [ $n -le 1 ]; then
        echo 1
    else
        local prev=$(factorial $((n - 1)))
        echo $((n * prev))
    fi
}
factorial 5  # Should output 120
"#;

// Output: Complete Rust module with tests
// - factorial.rs (implementation)
// - factorial_tests.rs (unit tests)
// - factorial_properties.rs (property tests)
// - factorial_mutations.toml (mutation config)
// All with ≥80% coverage, ≥85% mutation score
```

## 2. Architecture

### 2.1 Pipeline Overview

```
┌─────────────┐
│ Bash Script │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│  Bash Parser    │ (existing)
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│ Semantic        │ (existing)
│ Analysis        │
└──────┬──────────┘
       │
       ├─────────────────┬──────────────────┬──────────────────┐
       ▼                 ▼                  ▼                  ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Rust Code    │  │ Unit Tests   │  │ Property     │  │ Doctests     │
│ Generator    │  │ Generator    │  │ Generator    │  │ Generator    │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                  │                  │
       ▼                 ▼                  ▼                  ▼
┌──────────────────────────────────────────────────────────────┐
│                    Code Formatter                             │
│              (rustfmt + clippy + pmat)                        │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
                  ┌─────────────────┐
                  │ Generated        │
                  │ Rust Module      │
                  │ with Tests       │
                  └─────────────────┘
```

### 2.2 Component Design

#### 2.2.1 Test Generator Core

```rust
pub struct TestGenerator {
    options: TestGenOptions,
    coverage_tracker: CoverageTracker,
    mutation_analyzer: MutationAnalyzer,
}

pub struct TestGenOptions {
    /// Generate unit tests for each function
    pub generate_unit_tests: bool,

    /// Generate property tests using proptest
    pub generate_property_tests: bool,

    /// Generate mutation test configuration
    pub generate_mutation_config: bool,

    /// Generate doctests from comments
    pub generate_doctests: bool,

    /// Target coverage percentage (0-100)
    pub target_coverage: f64,

    /// Target mutation score (0-100)
    pub target_mutation_score: f64,

    /// Number of property test cases
    pub property_test_cases: usize,
}

impl Default for TestGenOptions {
    fn default() -> Self {
        Self {
            generate_unit_tests: true,
            generate_property_tests: true,
            generate_mutation_config: true,
            generate_doctests: true,
            target_coverage: 80.0,
            target_mutation_score: 85.0,
            property_test_cases: 1000,
        }
    }
}
```

#### 2.2.2 Unit Test Generator

```rust
pub struct UnitTestGenerator {
    /// Tracks which code paths have been covered
    coverage_tracker: CoverageTracker,
}

impl UnitTestGenerator {
    /// Generate unit tests for a bash function
    pub fn generate_tests(&self, function: &BashStmt) -> Vec<UnitTest> {
        // 1. Extract function signature and body
        // 2. Identify branches (if/else, case)
        // 3. Generate tests for each branch
        // 4. Generate edge case tests (empty input, max values, etc.)
        // 5. Generate error case tests
        // 6. Ensure ≥80% coverage
    }
}

pub struct UnitTest {
    pub name: String,
    pub test_fn: String,
    pub assertions: Vec<Assertion>,
}

pub enum Assertion {
    Equals { actual: String, expected: String },
    NotEquals { actual: String, expected: String },
    True { condition: String },
    False { condition: String },
    Panic { expected_message: String },
}
```

#### 2.2.3 Property Test Generator

```rust
pub struct PropertyTestGenerator {
    /// Identifies invariants and properties from bash semantics
    property_analyzer: PropertyAnalyzer,
}

impl PropertyTestGenerator {
    /// Generate property tests for a bash function
    pub fn generate_properties(&self, function: &BashStmt) -> Vec<PropertyTest> {
        // Properties to verify:
        // 1. Determinism: same input → same output
        // 2. Idempotency: f(f(x)) == f(x) for idempotent operations
        // 3. Commutativity: f(a, b) == f(b, a) where applicable
        // 4. Bounds: output within expected range
        // 5. Type preservation: string → string, int → int
        // 6. Side effects: file operations, env vars
    }
}

pub struct PropertyTest {
    pub name: String,
    pub property: Property,
    pub generators: Vec<Generator>,
    pub test_cases: usize,
}

pub enum Property {
    Determinism,
    Idempotency,
    Commutativity,
    Bounds { min: i64, max: i64 },
    TypePreservation,
    NoSideEffects,
    SideEffectsTracked { effects: Vec<SideEffect> },
}

pub enum Generator {
    Integer { min: i64, max: i64 },
    String { pattern: String },
    Path { valid: bool },
    Custom { generator_fn: String },
}
```

#### 2.2.4 Doctest Generator

```rust
pub struct DoctestGenerator;

impl DoctestGenerator {
    /// Extract doctests from bash comments
    pub fn generate_doctests(&self, ast: &BashAst) -> Vec<Doctest> {
        // 1. Parse comments above functions
        // 2. Extract usage examples
        // 3. Convert to Rust doctest format
        // 4. Add expected output assertions
    }
}

pub struct Doctest {
    pub function_name: String,
    pub example: String,
    pub expected_output: String,
}
```

#### 2.2.5 Mutation Test Config Generator

```rust
pub struct MutationConfigGenerator {
    mutation_analyzer: MutationAnalyzer,
}

impl MutationConfigGenerator {
    /// Generate mutation test configuration
    pub fn generate_config(&self, ast: &BashAst) -> MutationConfig {
        // 1. Identify critical code paths
        // 2. Configure mutation operators
        // 3. Set timeouts based on function complexity
        // 4. Exclude non-critical code
    }
}

pub struct MutationConfig {
    pub operators: Vec<MutationOperator>,
    pub timeout: u64,
    pub parallel_jobs: usize,
    pub target_score: f64,
}

pub enum MutationOperator {
    ArithmeticOp,     // +, -, *, /
    RelationalOp,     // ==, !=, <, >
    BooleanOp,        // &&, ||, !
    ReturnValue,      // replace with defaults
    Conditional,      // invert conditions
}
```

## 3. Test Generation Strategies

### 3.1 Unit Test Generation

#### 3.1.1 Branch Coverage

For each branch in the bash script, generate a test:

```bash
# Input bash
if [ $x -gt 10 ]; then
    echo "large"
else
    echo "small"
fi
```

Generated tests:
```rust
#[test]
fn test_branch_x_greater_than_10() {
    let x = 11;
    let result = function_name(x);
    assert_eq!(result, "large");
}

#[test]
fn test_branch_x_less_than_or_equal_10() {
    let x = 10;
    let result = function_name(x);
    assert_eq!(result, "small");
}

#[test]
fn test_branch_boundary_x_equals_10() {
    let x = 10;
    let result = function_name(x);
    assert_eq!(result, "small");
}
```

#### 3.1.2 Edge Case Generation

Automatically generate edge cases:
- Empty strings
- Zero/negative numbers
- Maximum values (i64::MAX, etc.)
- Special characters in strings
- Missing files/directories

```rust
#[test]
fn test_edge_case_empty_string() {
    let result = function_name("");
    assert!(result.is_err() || result == "");
}

#[test]
fn test_edge_case_max_value() {
    let result = function_name(i64::MAX);
    assert!(result.is_ok());
}
```

#### 3.1.3 Error Case Generation

For operations that can fail:
```rust
#[test]
#[should_panic(expected = "File not found")]
fn test_error_missing_file() {
    function_name("/nonexistent/file");
}

#[test]
fn test_error_invalid_input() {
    let result = function_name("invalid");
    assert!(result.is_err());
}
```

### 3.2 Property Test Generation

#### 3.2.1 Determinism Property

Every bash function should be deterministic (unless it uses $RANDOM, etc.):

```rust
proptest! {
    #[test]
    fn prop_determinism(input in any::<String>()) {
        let result1 = function_name(&input);
        let result2 = function_name(&input);
        prop_assert_eq!(result1, result2);
    }
}
```

#### 3.2.2 Idempotency Property

For idempotent operations (mkdir -p, etc.):

```rust
proptest! {
    #[test]
    fn prop_idempotency(input in any::<PathBuf>()) {
        let result1 = function_name(&input);
        let result2 = function_name(&input);
        prop_assert_eq!(result1, result2);
    }
}
```

#### 3.2.3 Bounds Property

For numeric functions:

```rust
proptest! {
    #[test]
    fn prop_bounds(n in 0i64..100i64) {
        let result = factorial(n);
        prop_assert!(result >= 1);
        prop_assert!(result <= i64::MAX);
    }
}
```

#### 3.2.4 Type Preservation

```rust
proptest! {
    #[test]
    fn prop_type_preservation(input in any::<String>()) {
        let result = function_name(&input);
        prop_assert!(result.is_some());
    }
}
```

### 3.3 Doctest Generation

#### 3.3.1 From Bash Comments

Extract examples from bash comments:

```bash
#!/bin/bash
# Reverses a string
#
# Example:
#   reverse_string "hello"
#   # outputs: olleh
function reverse_string() {
    echo "$1" | rev
}
```

Generated doctest:
```rust
/// Reverses a string
///
/// # Examples
///
/// ```
/// # use crate::reverse_string;
/// let result = reverse_string("hello");
/// assert_eq!(result, "olleh");
/// ```
pub fn reverse_string(s: &str) -> String {
    s.chars().rev().collect()
}
```

#### 3.3.2 From Function Usage

Extract usage from actual function calls:

```bash
factorial 5  # Should output 120
```

Generated doctest:
```rust
/// # Examples
///
/// ```
/// # use crate::factorial;
/// let result = factorial(5);
/// assert_eq!(result, 120);
/// ```
```

## 4. Coverage Analysis

### 4.1 Coverage Tracking

```rust
pub struct CoverageTracker {
    lines_covered: HashSet<usize>,
    branches_covered: HashSet<BranchId>,
    total_lines: usize,
    total_branches: usize,
}

impl CoverageTracker {
    pub fn coverage_percentage(&self) -> f64 {
        (self.lines_covered.len() as f64 / self.total_lines as f64) * 100.0
    }

    pub fn branch_coverage(&self) -> f64 {
        (self.branches_covered.len() as f64 / self.total_branches as f64) * 100.0
    }

    pub fn is_sufficient(&self, target: f64) -> bool {
        self.coverage_percentage() >= target
    }
}
```

### 4.2 Iterative Test Generation

If coverage is insufficient, generate additional tests:

```rust
impl TestGenerator {
    pub fn generate_until_coverage_met(&mut self, ast: &BashAst) -> TestSuite {
        let mut tests = self.generate_initial_tests(ast);

        while !self.coverage_tracker.is_sufficient(self.options.target_coverage) {
            // Identify uncovered code paths
            let uncovered = self.coverage_tracker.uncovered_paths();

            // Generate targeted tests
            let additional = self.generate_targeted_tests(&uncovered);
            tests.extend(additional);

            // Update coverage
            self.coverage_tracker.analyze(&tests);
        }

        tests
    }
}
```

## 5. Mutation Testing Configuration

### 5.1 Automatic Mutation Config

```rust
impl MutationConfigGenerator {
    pub fn generate_config(&self, ast: &BashAst) -> String {
        let mut config = String::from("# Generated mutation test configuration\n\n");

        // Analyze critical paths
        let critical_functions = self.identify_critical_functions(ast);

        config.push_str(&format!("timeout = {}\n", self.calculate_timeout(ast)));
        config.push_str(&format!("jobs = {}\n", self.optimal_parallelism()));
        config.push_str("\n[[examine]]\npaths = [\n");

        for func in critical_functions {
            config.push_str(&format!("    \"{}\",\n", func));
        }

        config.push_str("]\n\n");
        config.push_str("[operators]\n");
        config.push_str("arithmetic = true\n");
        config.push_str("relational = true\n");
        config.push_str("boolean = true\n");

        config
    }
}
```

### 5.2 Critical Path Identification

```rust
impl MutationAnalyzer {
    fn identify_critical_functions(&self, ast: &BashAst) -> Vec<String> {
        let mut critical = Vec::new();

        for stmt in &ast.statements {
            match stmt {
                BashStmt::Function { name, body, .. } => {
                    let complexity = self.calculate_complexity(body);
                    let has_branches = self.has_control_flow(body);
                    let has_side_effects = self.has_side_effects(body);

                    if complexity > 5 || has_branches || has_side_effects {
                        critical.push(name.clone());
                    }
                }
                _ => {}
            }
        }

        critical
    }
}
```

## 6. Output Structure

### 6.1 Generated File Organization

```
generated/
├── src/
│   ├── lib.rs                    # Main module
│   ├── factorial.rs              # Generated function
│   ├── reverse_string.rs         # Generated function
│   └── utils.rs                  # Helper functions
│
├── tests/
│   ├── factorial_test.rs         # Unit tests
│   ├── factorial_properties.rs   # Property tests
│   ├── reverse_string_test.rs
│   └── integration_test.rs       # Integration tests
│
├── .cargo-mutants.toml           # Mutation config
├── Cargo.toml                    # Dependencies
└── README.md                     # Generated documentation
```

### 6.2 Complete Module Generation

```rust
pub struct GeneratedModule {
    pub name: String,
    pub source_code: String,
    pub unit_tests: String,
    pub property_tests: String,
    pub doctests: Vec<Doctest>,
    pub mutation_config: String,
    pub readme: String,
}

impl GeneratedModule {
    pub fn write_to_disk(&self, base_path: &Path) -> Result<()> {
        // Write src/
        let src_path = base_path.join("src");
        fs::create_dir_all(&src_path)?;
        fs::write(src_path.join(&format!("{}.rs", self.name)), &self.source_code)?;

        // Write tests/
        let test_path = base_path.join("tests");
        fs::create_dir_all(&test_path)?;
        fs::write(test_path.join(&format!("{}_test.rs", self.name)), &self.unit_tests)?;
        fs::write(test_path.join(&format!("{}_properties.rs", self.name)), &self.property_tests)?;

        // Write mutation config
        fs::write(base_path.join(".cargo-mutants.toml"), &self.mutation_config)?;

        // Write README
        fs::write(base_path.join("README.md"), &self.readme)?;

        Ok(())
    }

    pub fn verify_quality(&self) -> QualityReport {
        // Run rustfmt
        let fmt_result = self.run_rustfmt();

        // Run clippy
        let clippy_result = self.run_clippy();

        // Check coverage
        let coverage = self.measure_coverage();

        // Run mutation tests
        let mutation_score = self.run_mutation_tests();

        QualityReport {
            fmt_passed: fmt_result.is_ok(),
            clippy_passed: clippy_result.is_ok(),
            coverage_percentage: coverage,
            mutation_score,
            meets_quality_gates: coverage >= 80.0 && mutation_score >= 85.0,
        }
    }
}
```

## 7. Quality Gates

### 7.1 Automated Verification

```rust
pub struct QualityGate {
    pub min_coverage: f64,
    pub min_mutation_score: f64,
    pub require_clippy_clean: bool,
    pub require_rustfmt: bool,
}

impl QualityGate {
    pub fn verify(&self, module: &GeneratedModule) -> Result<QualityReport> {
        let report = module.verify_quality();

        if !report.meets_quality_gates {
            return Err(QualityError::InsufficientQuality {
                coverage: report.coverage_percentage,
                mutation_score: report.mutation_score,
                required_coverage: self.min_coverage,
                required_mutation: self.min_mutation_score,
            });
        }

        Ok(report)
    }
}
```

### 7.2 Quality Report

```rust
pub struct QualityReport {
    pub fmt_passed: bool,
    pub clippy_passed: bool,
    pub coverage_percentage: f64,
    pub mutation_score: f64,
    pub meets_quality_gates: bool,
    pub suggestions: Vec<String>,
}

impl QualityReport {
    pub fn display(&self) -> String {
        format!(
            r#"
Quality Report
==============
Formatting: {}
Clippy: {}
Coverage: {:.1}%
Mutation Score: {:.1}%
Quality Gates: {}

{}
"#,
            if self.fmt_passed { "✅ PASS" } else { "❌ FAIL" },
            if self.clippy_passed { "✅ PASS" } else { "❌ FAIL" },
            self.coverage_percentage,
            self.mutation_score,
            if self.meets_quality_gates { "✅ PASS" } else { "❌ FAIL" },
            self.suggestions.join("\n")
        )
    }
}
```

## 8. Implementation Roadmap

### Sprint 1: Core Test Generation (Week 1)
- [ ] Implement `TestGenerator` core
- [ ] Implement `UnitTestGenerator`
  - [ ] Branch coverage tests
  - [ ] Edge case generation
  - [ ] Error case generation
- [ ] Basic coverage tracking
- [ ] Quality gate: ≥80% coverage

### Sprint 2: Property Testing (Week 2)
- [ ] Implement `PropertyTestGenerator`
  - [ ] Determinism property
  - [ ] Idempotency property
  - [ ] Bounds property
  - [ ] Type preservation
- [ ] Custom generator synthesis
- [ ] Property verification

### Sprint 3: Doctest & Mutation (Week 3)
- [ ] Implement `DoctestGenerator`
  - [ ] Comment parsing
  - [ ] Example extraction
  - [ ] Rust doctest generation
- [ ] Implement `MutationConfigGenerator`
  - [ ] Critical path identification
  - [ ] Operator configuration
  - [ ] Timeout calculation
- [ ] Quality gate: ≥85% mutation score

### Sprint 4: Integration & Polish (Week 4)
- [ ] Complete module generation
- [ ] File system integration
- [ ] Quality verification automation
- [ ] CLI tool integration
- [ ] Documentation and examples
- [ ] Canonical demonstration (book)

## 9. Success Metrics

### 9.1 Quantitative Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Test Coverage | ≥80% | llvm-cov |
| Mutation Score | ≥85% | cargo-mutants |
| Clippy Warnings | 0 | cargo clippy |
| Format Compliance | 100% | rustfmt |
| Complexity | <10 | pmat |
| Property Tests | ≥5 per function | proptest |
| Doctests | ≥1 per function | cargo test --doc |

### 9.2 Qualitative Metrics

- [ ] All generated code compiles without errors
- [ ] All generated tests pass on first run
- [ ] Generated doctests are readable and useful
- [ ] Mutation tests catch real bugs
- [ ] Property tests find edge cases automatically

## 10. Example: Complete Generation

### Input Bash Script

```bash
#!/bin/bash
# Fibonacci calculator
# Computes the nth Fibonacci number
#
# Usage: fibonacci N
# Example: fibonacci 10
# Output: 55

function fibonacci() {
    local n=$1

    # Base cases
    if [ $n -le 0 ]; then
        echo 0
        return
    fi

    if [ $n -eq 1 ]; then
        echo 1
        return
    fi

    # Recursive case
    local a=$(fibonacci $((n - 1)))
    local b=$(fibonacci $((n - 2)))
    echo $((a + b))
}

# Main
fibonacci "${1:-10}"
```

### Generated Output

#### `src/fibonacci.rs`
```rust
/// Fibonacci calculator
/// Computes the nth Fibonacci number
///
/// # Examples
///
/// ```
/// use fibonacci::fibonacci;
/// assert_eq!(fibonacci(10), 55);
/// ```
pub fn fibonacci(n: i64) -> i64 {
    // Base cases
    if n <= 0 {
        return 0;
    }

    if n == 1 {
        return 1;
    }

    // Recursive case
    let a = fibonacci(n - 1);
    let b = fibonacci(n - 2);
    a + b
}
```

#### `tests/fibonacci_test.rs`
```rust
use fibonacci::fibonacci;

#[test]
fn test_base_case_zero() {
    assert_eq!(fibonacci(0), 0);
}

#[test]
fn test_base_case_one() {
    assert_eq!(fibonacci(1), 1);
}

#[test]
fn test_recursive_case_small() {
    assert_eq!(fibonacci(5), 5);
}

#[test]
fn test_recursive_case_medium() {
    assert_eq!(fibonacci(10), 55);
}

#[test]
fn test_negative_input() {
    assert_eq!(fibonacci(-1), 0);
}

#[test]
fn test_boundary_value() {
    assert_eq!(fibonacci(2), 1);
}
```

#### `tests/fibonacci_properties.rs`
```rust
use fibonacci::fibonacci;
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_determinism(n in 0i64..20i64) {
        let result1 = fibonacci(n);
        let result2 = fibonacci(n);
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn prop_monotonic_increasing(n in 1i64..20i64) {
        let fib_n = fibonacci(n);
        let fib_n_plus_1 = fibonacci(n + 1);
        prop_assert!(fib_n_plus_1 >= fib_n);
    }

    #[test]
    fn prop_golden_ratio(n in 10i64..20i64) {
        let fib_n = fibonacci(n) as f64;
        let fib_n_minus_1 = fibonacci(n - 1) as f64;
        let ratio = fib_n / fib_n_minus_1;
        let golden = 1.618033988749895;
        prop_assert!((ratio - golden).abs() < 0.01);
    }
}
```

## 11. References

- Property-Based Testing: [proptest documentation](https://docs.rs/proptest/)
- Mutation Testing: [cargo-mutants documentation](https://github.com/sourcefrog/cargo-mutants)
- Coverage: [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- Quality: PMAT framework

---

**Next Steps**:
1. Review and approve specification
2. Implement Sprint 1 (core test generation)
3. Validate with canonical bash examples
4. Create "book" with demonstrations
