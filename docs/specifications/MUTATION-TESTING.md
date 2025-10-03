# Mutation Testing Specification for Rash

**Version**: 1.0
**Status**: Draft
**Target**: ≥90% mutation kill rate
**Tool**: cargo-mutants

---

## Executive Summary

Mutation testing validates that Rash's **527+ tests** are actually effective by deliberately introducing bugs ("mutations") and checking if tests catch them. This specification defines how to implement, run, and maintain mutation testing for the Rash transpiler with a target **≥90% mutation kill rate**.

---

## The Problem Mutation Testing Solves

You can have **99.4% test pass rate** and still have ineffective tests:

```rust
// Transpiler code
pub fn transpile(source: &str, config: Config) -> Result<String> {
    let ast = parse(source)?;
    let ir = convert_to_ir(&ast)?;
    emit_shell(&ir, &config)
}

// Test with high coverage but weak assertions
#[test]
fn test_transpile() {
    let source = "fn main() { let x = 1; }";
    transpile(source, Config::default());  // ❌ No assertion!
}
```

**Test metrics say**: ✅ 527/530 tests passing
**Reality**: This test catches nothing!

Mutation testing finds these **weak tests** by mutating code and seeing if tests fail.

---

## How Mutation Testing Works for Rash

### The Process

1. **Baseline**: Run all tests → 527/530 passing
2. **Mutate**: Change transpiler code (e.g., change `==` to `!=`)
3. **Test**: Run tests again
4. **Result**:
   - Tests **fail** → Mutation **killed** ✅ (good test!)
   - Tests **pass** → Mutation **survived** ❌ (weak test!)

### Rash-Specific Example Mutations

#### Parser Mutations

```rust
// Original parser code
pub fn convert_match_stmt(expr_match: &syn::ExprMatch) -> Result<Stmt> {
    let scrutinee = convert_expr(&expr_match.expr)?;
    let arms = convert_arms(&expr_match.arms)?;
    Ok(Stmt::Match { scrutinee, arms })
}

// Mutation 1: Remove scrutinee conversion (should fail tests)
pub fn convert_match_stmt(expr_match: &syn::ExprMatch) -> Result<Stmt> {
    let scrutinee = Expr::Literal(Literal::I32(0));  // Mutated: constant
    let arms = convert_arms(&expr_match.arms)?;
    Ok(Stmt::Match { scrutinee, arms })
}

// Mutation 2: Always return error
pub fn convert_match_stmt(expr_match: &syn::ExprMatch) -> Result<Stmt> {
    Err(Error::Validation("unsupported".into()))  // Mutated
}
```

**Good test** (catches both mutations):

```rust
#[test]
fn test_match_expression_parsing() {
    let source = r#"
        fn main() {
            let x = 2;
            match x {
                1 => { let y = 10; }
                _ => { let y = 0; }
            }
        }
    "#;

    // Should successfully transpile
    let result = transpile(source, Config::default());
    assert!(result.is_ok());  // Kills mutation 2

    // Generated code should contain case statement with $x
    let shell = result.unwrap();
    assert!(shell.contains("case \"$x\" in"));  // Kills mutation 1
}
```

#### IR Conversion Mutations

```rust
// Original IR conversion
impl IrConverter {
    pub fn convert_range(&self, start: &Expr, end: &Expr, inclusive: bool) -> Result<(ShellValue, ShellValue)> {
        let start_val = self.convert_expr_to_value(start)?;
        let mut end_val = self.convert_expr_to_value(end)?;

        if !inclusive {
            // For exclusive range (0..3), adjust to inclusive (0..2)
            if let ShellValue::String(s) = &end_val {
                if let Ok(n) = s.parse::<i32>() {
                    end_val = ShellValue::String((n - 1).to_string());
                }
            }
        }

        Ok((start_val, end_val))
    }
}

// Mutation 1: Remove exclusive range adjustment
if !inclusive {
    // Deleted: No adjustment
}

// Mutation 2: Wrong adjustment
end_val = ShellValue::String((n + 1).to_string());  // + instead of -

// Mutation 3: Invert condition
if inclusive {  // Should be !inclusive
    end_val = ShellValue::String((n - 1).to_string());
}
```

**Good test** (catches all mutations):

```rust
#[test]
fn test_exclusive_range_adjustment() {
    let source = "fn main() { for i in 0..3 { let x = i; } }";
    let result = transpile(source, Config::default()).unwrap();

    // Exclusive range 0..3 should generate seq 0 2 (not 0 3)
    assert!(result.contains("seq 0 2"));  // Kills mutations 1 & 2
    assert!(!result.contains("seq 0 3"));  // Kills mutation 3
}

#[test]
fn test_inclusive_range_no_adjustment() {
    let source = "fn main() { for i in 0..=3 { let x = i; } }";
    let result = transpile(source, Config::default()).unwrap();

    // Inclusive range 0..=3 should generate seq 0 3
    assert!(result.contains("seq 0 3"));  // Kills mutation 3
}
```

#### Emitter Mutations

```rust
// Original emitter code
pub fn emit_case_statement(&self, scrutinee: &ShellValue, arms: &[CaseArm]) -> Result<String> {
    let mut output = String::new();
    let scrutinee_str = self.emit_shell_value(scrutinee)?;

    writeln!(output, "case {} in", scrutinee_str)?;

    for arm in arms {
        let pattern = match &arm.pattern {
            CasePattern::Literal(lit) => lit.clone(),
            CasePattern::Wildcard => "*".to_string(),
        };

        writeln!(output, "    {})", pattern)?;
        self.emit_ir(&mut output, &arm.body)?;
        writeln!(output, "    ;;")?;
    }

    writeln!(output, "esac")?;
    Ok(output)
}

// Mutation 1: Remove wildcard handling
CasePattern::Wildcard => "".to_string(),  // Empty instead of *

// Mutation 2: Remove ;; terminators
// writeln!(output, "    ;;")?;  // Deleted

// Mutation 3: Remove esac
// writeln!(output, "esac")?;  // Deleted
```

**Good test** (catches all mutations):

```rust
#[test]
fn test_case_statement_structure() {
    let source = "fn main() { match x { 1 => {}, _ => {} } }";
    let result = transpile(source, Config::default()).unwrap();

    // Must have proper case structure
    assert!(result.contains("case"));
    assert!(result.contains("*)"));  // Kills mutation 1
    assert_eq!(result.matches(";;").count(), 2);  // Kills mutation 2
    assert!(result.contains("esac"));  // Kills mutation 3
}
```

---

## Setting Up Mutation Testing

### Installation

```bash
cargo install cargo-mutants
```

### Basic Usage

```bash
# Run mutation testing on entire workspace
cargo mutants

# Run on specific package
cargo mutants -p bashrs

# Run on specific file (e.g., parser)
cargo mutants --file rash/src/services/parser.rs

# Show what would be mutated without running tests
cargo mutants --list

# Run with specific test threads
cargo mutants --test-threads=8
```

### Configuration

Create `.cargo/mutants.toml`:

```toml
# Mutation testing configuration for Rash

# Timeout per mutant (10 minutes for slow transpiler tests)
timeout = 600

# Exclude patterns
exclude_globs = [
    "**/tests/**",
    "**/*_test.rs",
    "**/testing/**",
    "**/bin/**",
    "**/examples/**",
]

# Additional test arguments
test_args = ["--lib"]

# Exclude specific functions (if needed for acceptable mutations)
exclude_re = [
    ".*::fmt",           # Debug implementations
    ".*::clone",         # Clone implementations
    "rash_.*",          # Runtime functions
]
```

---

## Common Rash Mutation Types

### 1. Pattern Matching Mutations

```rust
// Original
Pattern::Literal(lit) => Ok(CasePattern::Literal(lit.to_string())),
Pattern::Wildcard => Ok(CasePattern::Wildcard),

// Mutations
Pattern::Literal(lit) => Ok(CasePattern::Wildcard),  // Always wildcard
Pattern::Wildcard => Ok(CasePattern::Literal("".into())),  // Wrong type
```

**Test that kills**:

```rust
#[test]
fn test_pattern_types_preserved() {
    let source = "fn main() { match x { 1 => {}, _ => {} } }";
    let result = transpile(source, Config::default()).unwrap();

    assert!(result.contains("1)"));   // Literal preserved
    assert!(result.contains("*)"));   // Wildcard preserved
}
```

### 2. Validation Mutations

```rust
// Original
if arms.is_empty() {
    return Err(RashError::ValidationError("Match must have at least one arm".into()));
}

// Mutations
if !arms.is_empty() { }  // Inverted condition
if arms.len() > 0 { }    // Changed check
// Deleted entirely
```

**Test that kills**:

```rust
#[test]
fn test_empty_match_rejected() {
    let source = "fn main() { match x {} }";
    let result = transpile(source, Config::default());

    assert!(result.is_err());  // Must reject
    assert!(matches!(result.unwrap_err(), Error::Validation(_)));
}

#[test]
fn test_nonempty_match_accepted() {
    let source = "fn main() { match x { _ => {} } }";
    let result = transpile(source, Config::default());

    assert!(result.is_ok());  // Must accept
}
```

### 3. Control Flow Mutations

```rust
// Original
for arm in arms {
    self.emit_arm(output, arm)?;
}

// Mutations
for arm in &arms[0..1] { }  // Only first arm
// Loop deleted entirely
```

**Test that kills**:

```rust
#[test]
fn test_all_match_arms_emitted() {
    let source = r#"fn main() { match x {
        1 => { let a = 1; }
        2 => { let b = 2; }
        _ => { let c = 0; }
    } }"#;

    let result = transpile(source, Config::default()).unwrap();

    // All three arms must be present
    assert!(result.contains("1)"));
    assert!(result.contains("2)"));
    assert!(result.contains("*)"));
    assert!(result.contains("a=1"));
    assert!(result.contains("b=2"));
    assert!(result.contains("c=0"));
}
```

### 4. Error Propagation Mutations

```rust
// Original
let ast = parse(source)?;
let ir = convert_to_ir(&ast)?;
emit_shell(&ir, &config)

// Mutations
let ast = parse(source).unwrap_or_default();  // No propagation
let ir = convert_to_ir(&ast).ok().unwrap();   // Panic instead
```

**Test that kills**:

```rust
#[test]
fn test_parse_error_propagates() {
    let source = "invalid rust syntax {{{";
    let result = transpile(source, Config::default());

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Parse(_)));
}
```

---

## Rash Mutation Testing Strategy

### Target: 90% Kill Rate

```
Mutation Score = (Killed Mutants / Total Mutants) × 100%

Rash target: ≥ 90%
```

### Priority Modules

Focus mutation testing on critical transpiler components:

1. **High Priority** (Run frequently):
   - `rash/src/services/parser.rs` - AST conversion
   - `rash/src/ir/mod.rs` - IR generation
   - `rash/src/emitter/posix.rs` - Shell code emission
   - `rash/src/validation/pipeline.rs` - Validation logic

2. **Medium Priority** (Weekly):
   - `rash/src/ast/restricted.rs` - AST definitions
   - `rash/src/ir/shell_ir.rs` - IR definitions

3. **Low Priority** (Pre-release):
   - `rash/src/models/*.rs` - Data structures
   - `rash/src/cli/*.rs` - CLI handlers

### Running Mutation Tests

```bash
# Full mutation test suite (slow - ~30-60 minutes)
make mutants

# Quick mutation test (critical modules only - ~10 minutes)
make mutants-quick

# Specific module
cargo mutants --file rash/src/services/parser.rs
```

### Makefile Integration

```makefile
# Add to Makefile

.PHONY: mutants mutants-quick

mutants: ## Run full mutation testing suite
	@echo "Running mutation tests (this will take 30-60 minutes)..."
	cargo mutants --test-threads=8

mutants-quick: ## Run mutation tests on critical modules only
	@echo "Running quick mutation tests on critical modules..."
	cargo mutants --file rash/src/services/parser.rs
	cargo mutants --file rash/src/ir/mod.rs
	cargo mutants --file rash/src/emitter/posix.rs
	cargo mutants --file rash/src/validation/pipeline.rs

mutants-report: ## Generate mutation testing report
	cargo mutants --json > .quality/mutation-report.json
	@echo "Mutation report saved to .quality/mutation-report.json"
```

---

## Expected Mutation Test Output

### Example Run

```
Testing mutants:
rash/src/services/parser.rs:498: replace convert_match_stmt -> Result<Stmt> with Ok(Stmt::Noop)
    CAUGHT in 0.8s by test_edge_case_07_match_expressions

rash/src/ir/mod.rs:200: replace convert_expr_to_value -> Result<ShellValue> with Ok(ShellValue::String("".into()))
    CAUGHT in 0.3s by test_transpile_basic

rash/src/emitter/posix.rs:350: replace emit_case_statement -> Result<()> with Ok(())
    CAUGHT in 0.5s by test_case_statement_structure

rash/src/validation/pipeline.rs:328: replace != with ==
    CAUGHT in 0.2s by test_validation_rejects_empty_match

rash/src/services/parser.rs:535: replace Pattern::Wildcard with Pattern::Literal(Literal::I32(0))
    MISSED in 1.2s

Summary:
  Tested: 450 mutants
  Caught: 412 mutants (91.6%)
  Missed: 35 mutants (7.8%)
  Timeout: 3 mutants (0.7%)
  Unviable: 12 mutants (2.7%)
```

### Interpreting Results

- **Caught** ✅: Test suite detected the mutation (good!)
- **Missed** ❌: Test suite didn't detect mutation (add test!)
- **Timeout** ⚠️: Test took >10 minutes (possibly infinite loop)
- **Unviable**: Mutation wouldn't compile (ignored in score)

---

## Improving Mutation Kill Rate

### Strategy 1: Test All Pattern Types

```rust
#[test]
fn test_all_pattern_types() {
    // Literal pattern
    let lit_source = "fn main() { match x { 1 => {} } }";
    assert!(transpile(lit_source, Config::default()).is_ok());

    // Wildcard pattern
    let wild_source = "fn main() { match x { _ => {} } }";
    assert!(transpile(wild_source, Config::default()).is_ok());

    // Multiple patterns
    let multi_source = "fn main() { match x { 1 => {}, 2 => {}, _ => {} } }";
    let result = transpile(multi_source, Config::default()).unwrap();
    assert!(result.contains("1)"));
    assert!(result.contains("2)"));
    assert!(result.contains("*)"));
}
```

### Strategy 2: Test Boundary Conditions

```rust
#[test]
fn test_range_boundaries() {
    // Exclusive range 0..3 should be seq 0 2
    let excl = "fn main() { for i in 0..3 {} }";
    let result = transpile(excl, Config::default()).unwrap();
    assert!(result.contains("seq 0 2"));
    assert!(!result.contains("seq 0 3"));

    // Inclusive range 0..=3 should be seq 0 3
    let incl = "fn main() { for i in 0..=3 {} }";
    let result = transpile(incl, Config::default()).unwrap();
    assert!(result.contains("seq 0 3"));
}
```

### Strategy 3: Test Error Cases

```rust
#[test]
fn test_error_cases_comprehensive() {
    // Parse errors
    assert!(transpile("invalid", Config::default()).is_err());

    // Validation errors
    assert!(transpile("fn not_main() {}", Config::default()).is_err());

    // Empty match
    assert!(transpile("fn main() { match x {} }", Config::default()).is_err());
}
```

---

## Acceptable Surviving Mutations

Some mutations are acceptable to miss:

### 1. Debug/Display Implementations

```rust
impl fmt::Debug for ShellIR {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)  // OK if mutated
    }
}
```

**Acceptable**: Tests shouldn't depend on debug output.

### 2. Internal Helper Formatting

```rust
fn escape_variable_name(name: &str) -> String {
    // Mutations to exact escaping logic may be acceptable
    // if ShellCheck validation still passes
    name.replace("-", "_")
}
```

**Acceptable if**: ShellCheck tests still pass.

### 3. Performance Optimizations

```rust
fn should_inline(&self) -> bool {
    self.size < INLINE_THRESHOLD  // Mutation to threshold OK
}
```

**Acceptable**: Result is same, just different performance.

---

## Integration with Quality Gates

### Quality Gate Threshold

```bash
# In Makefile quality-gate target
quality-gate: test coverage mutants
	@echo "Checking mutation score..."
	@SCORE=$$(cargo mutants --json | jq '.caught / .total * 100')
	@if [ $$(echo "$$SCORE < 90" | bc) -eq 1 ]; then \
		echo "❌ Mutation score $$SCORE% below target 90%"; \
		exit 1; \
	fi
	@echo "✅ All quality gates passed"
```

### CI/CD Integration

```yaml
# .github/workflows/mutation.yml
name: Mutation Testing

on:
  pull_request:
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday

jobs:
  mutants:
    runs-on: ubuntu-latest
    timeout-minutes: 120

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable

      - name: Install cargo-mutants
        run: cargo install cargo-mutants

      - name: Run mutation tests
        run: cargo mutants --test-threads=4

      - name: Check mutation score
        run: |
          SCORE=$(cargo mutants --json | jq '.caught / .total * 100')
          echo "Mutation score: $SCORE%"
          if (( $(echo "$SCORE < 90" | bc -l) )); then
            echo "❌ Mutation score $SCORE% below target 90%"
            exit 1
          fi
          echo "✅ Mutation score $SCORE% meets target"

      - name: Upload mutation report
        uses: actions/upload-artifact@v3
        with:
          name: mutation-report
          path: mutants.out/
```

---

## Performance Optimization

Mutation testing is slow. Optimize:

### 1. Parallel Execution

```bash
# Use all CPU cores
cargo mutants --test-threads=$(nproc)
```

### 2. Incremental Testing

```bash
# Only test changed files (for PRs)
git diff --name-only main | grep '\.rs$' | while read file; do
    cargo mutants --file "$file"
done
```

### 3. Timeout Configuration

```bash
# Set appropriate timeout (transpiler tests are slower)
cargo mutants --timeout=600  # 10 minutes per mutant
```

### 4. Baseline Filtering

```toml
# .cargo/mutants.toml
exclude_globs = [
    "**/tests/**",
    "**/testing/**",
    "**/*_test.rs",
    "**/bin/**",
]
```

---

## Mutation Testing Best Practices for Rash

### 1. Run Strategically

```bash
# Every commit: Skip (too slow)
# Every PR: Quick critical modules
# Weekly: Full mutation suite
# Before release: Full + report
```

### 2. Focus on High-Value Code

```bash
# Priority order
1. Parser (converts Rust to AST)
2. IR converter (AST to ShellIR)
3. Emitter (ShellIR to shell code)
4. Validation (safety checks)
```

### 3. Track Metrics Over Time

```bash
# Save mutation scores with git tags
git tag -a v0.6.0-mutations -m "Mutation score: 91.6%"
```

### 4. Target 90%, Not 100%

- **90%**: ✅ Excellent test quality
- **95%**: ⚠️ Very good, diminishing returns
- **100%**: ❌ Not practical (acceptable mutations exist)

### 5. Combine with Other Metrics

```bash
make quality-gate  # Runs:
# - cargo test (527/530 passing)
# - cargo coverage (85.36% core)
# - cargo mutants (≥90% kill rate)
# - shellcheck validation
```

---

## Expected Rash Mutation Metrics

### Baseline Targets

| Module | Expected Mutants | Target Kill Rate | Priority |
|--------|------------------|------------------|----------|
| Parser | ~150 | ≥92% | Critical |
| IR Converter | ~120 | ≥90% | Critical |
| Emitter | ~100 | ≥90% | Critical |
| Validation | ~60 | ≥95% | Critical |
| AST | ~40 | ≥85% | Medium |
| Models | ~30 | ≥80% | Low |
| **Total** | **~500** | **≥90%** | **Overall** |

### Quality Gate Integration

```
Rash Quality Metrics (v0.6.0)
├── Tests: 527/530 passing (99.4%)
├── Coverage: 85.36% core, 82.18% total
├── Mutation: ≥90% kill rate
├── ShellCheck: 24/24 passing
└── Performance: 19.1µs transpile

All metrics must pass for release.
```

---

## Summary

Mutation testing validates Rash's test effectiveness:

- **Purpose**: Ensure 527+ tests actually catch bugs
- **Target**: ≥90% mutation kill rate
- **Tool**: `cargo-mutants`
- **Integration**: Weekly CI runs, pre-release gates
- **Benefit**: Confidence that tests are effective

### Mutation Testing in Context

| Metric | What it measures | Rash target | Status |
|--------|------------------|-------------|--------|
| Test pass rate | Tests not failing | 99.4% | ✅ |
| Line coverage | Lines executed | ≥85% | ✅ |
| Mutation score | Test effectiveness | ≥90% | 🔄 TBD |
| ShellCheck | POSIX compliance | 100% | ✅ |

### Implementation Roadmap

1. **Sprint 20**: Setup (1-2 hours)
   - Install cargo-mutants
   - Create .cargo/mutants.toml
   - Add Makefile targets

2. **Sprint 21**: Baseline (2-3 hours)
   - Run initial mutation tests
   - Document baseline score
   - Identify weak tests

3. **Sprint 22**: Improve (3-4 hours)
   - Add tests to kill surviving mutants
   - Reach ≥90% target
   - Document acceptable mutations

4. **Sprint 23**: Integrate (1 hour)
   - Add CI/CD workflow
   - Add quality gate check
   - Document process

---

## Further Reading

- [cargo-mutants documentation](https://mutants.rs/)
- [Mutation Testing: A Comprehensive Survey](https://ieeexplore.ieee.org/document/6963470)
- [PIT Mutation Testing](https://pitest.org/) - Java mutation testing reference
- Rash mutation config: `.cargo/mutants.toml` (to be created)
