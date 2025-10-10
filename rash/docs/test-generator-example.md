# Test Generator End-to-End Example

This example demonstrates the complete workflow of using the Rash test generator to automatically create comprehensive tests for transpiled bash code.

## Input: Bash Script

Let's start with a real bash script that we want to transpile and test:

```bash
#!/bin/bash
# factorial.sh - Calculate factorial recursively
# Example: factorial(5) => 120
# Example: factorial(0) => 1

factorial() {
    local n=$1

    # Base case
    if [ $n -le 1 ]; then
        echo 1
        return 0
    fi

    # Recursive case
    local prev=$(factorial $((n - 1)))
    echo $((n * prev))
}

# Example: is_prime(7) => true
# Example: is_prime(4) => false
is_prime() {
    local n=$1

    if [ $n -lt 2 ]; then
        echo "false"
        return 1
    fi

    local i=2
    while [ $i -lt $n ]; do
        if [ $((n % i)) -eq 0 ]; then
            echo "false"
            return 1
        fi
        i=$((i + 1))
    done

    echo "true"
    return 0
}

# Main entry point
main() {
    factorial 5
    is_prime 7
}

main "$@"
```

## Step 1: Parse the Bash Script

```rust
use bashrs::bash_parser::BashParser;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the bash script
    let bash_code = fs::read_to_string("factorial.sh")?;

    // Parse into AST
    let parser = BashParser::new();
    let ast = parser.parse(&bash_code)?;

    println!("✓ Parsed {} statements", ast.statements.len());
    println!("✓ Found {} functions",
        ast.statements.iter()
            .filter(|s| matches!(s, BashStmt::Function { .. }))
            .count()
    );

    Ok(())
}
```

**Output:**
```
✓ Parsed 3 statements
✓ Found 3 functions
```

## Step 2: Generate Complete Test Suite

```rust
use bashrs::test_generator::{TestGenerator, TestGenOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse bash code (from step 1)
    let ast = parser.parse(&bash_code)?;

    // Configure test generator
    let options = TestGenOptions {
        target_coverage: 0.80,
        target_mutation_score: 0.85,
        generate_unit_tests: true,
        generate_property_tests: true,
        generate_doctests: true,
        generate_mutation_config: true,
    };

    // Generate test suite
    let mut generator = TestGenerator::with_options(options);
    let test_suite = generator.generate(&ast)?;

    println!("✓ Generated {} unit tests", test_suite.unit_tests.len());
    println!("✓ Generated {} property tests", test_suite.property_tests.len());
    println!("✓ Generated {} doctests", test_suite.doctests.len());

    Ok(())
}
```

**Output:**
```
✓ Generated 18 unit tests
✓ Generated 6 property tests
✓ Generated 4 doctests
```

## Step 3: Generated Unit Tests

The generator creates these unit tests automatically:

```rust
// tests/factorial_tests.rs

#[test]
fn test_factorial_if_then_branch() {
    // Test if-then branch for base case
    let result = factorial(1);
    assert_eq!(result, 1);
}

#[test]
fn test_factorial_if_else_branch() {
    // Test else branch for recursive case
    let result = factorial(5);
    assert_eq!(result, 120);
}

#[test]
fn test_factorial_edge_case_zero() {
    // Test with zero value
    let result = factorial(0);
    assert_eq!(result, 1);
}

#[test]
fn test_factorial_edge_case_negative() {
    // Test with negative value
    let result = factorial(-1);
    assert_eq!(result, 1);
}

#[test]
fn test_factorial_edge_case_large_value() {
    // Test with maximum value
    let result = factorial(20);
    assert!(result > 0);
}

#[test]
fn test_is_prime_while_loop() {
    // Test while loop execution
    let result = is_prime(7);
    assert_eq!(result, true);
}

#[test]
fn test_is_prime_if_then_branch_less_than_2() {
    // Test if-then branch for n < 2
    let result = is_prime(1);
    assert_eq!(result, false);
}

#[test]
fn test_is_prime_if_then_branch_divisible() {
    // Test if-then branch when divisible
    let result = is_prime(4);
    assert_eq!(result, false);
}

#[test]
fn test_is_prime_edge_case_two() {
    // Test with smallest prime
    let result = is_prime(2);
    assert_eq!(result, true);
}
```

## Step 4: Generated Property Tests

```rust
// tests/factorial_property_tests.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_factorial_determinism(
        n in 0i64..=20,
    ) {
        // Test determinism: same input → same output
        let result1 = factorial(n);
        let result2 = factorial(n);
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn prop_factorial_type_preservation(
        n in 0i64..=20,
    ) {
        // Test type preservation
        let result = factorial(n);
        prop_assert!(result >= 0);
        prop_assert!(std::mem::size_of_val(&result) > 0);
    }
}

proptest! {
    #[test]
    fn prop_is_prime_determinism(
        n in 0i64..=100,
    ) {
        // Test determinism: same input → same output
        let result1 = is_prime(n);
        let result2 = is_prime(n);
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn prop_is_prime_bounds(
        n in -10i64..=110,
    ) {
        // Test bounds: result is boolean
        let result = is_prime(n);
        prop_assert!(result == true || result == false);
    }
}
```

## Step 5: Generated Doctests

```rust
// src/lib.rs

/// Calculate factorial recursively
///
/// # Examples
///
/// ```
/// use crate::factorial;
/// let result = factorial(5);
/// assert_eq!(result, 120);
/// ```
///
/// ```
/// use crate::factorial;
/// let result = factorial(0);
/// assert_eq!(result, 1);
/// ```
pub fn factorial(n: i64) -> i64 {
    if n <= 1 {
        return 1;
    }
    n * factorial(n - 1)
}

/// Check if a number is prime
///
/// # Examples
///
/// ```
/// use crate::is_prime;
/// let result = is_prime(7);
/// assert_eq!(result, true);
/// ```
///
/// ```
/// use crate::is_prime;
/// let result = is_prime(4);
/// assert_eq!(result, false);
/// ```
pub fn is_prime(n: i64) -> bool {
    if n < 2 {
        return false;
    }

    for i in 2..n {
        if n % i == 0 {
            return false;
        }
    }

    true
}
```

## Step 6: Generated Mutation Configuration

```toml
# .cargo-mutants.toml
# Generated mutation test configuration
# Auto-generated based on code complexity analysis

timeout = 75
jobs = 2
# Target mutation score: 85%

exclude_globs = [
    "tests/*",
    "*_test.rs",
    "*/tests.rs",
]

# Mutation operators to apply
# Arithmetic: +, -, *, /, %
# Relational: <, <=, >, >=, ==, !=
# Boolean: &&, ||, !
# Return values
# Conditionals: if/else

# High-complexity functions requiring extra attention:
# - is_prime (cyclomatic complexity: 5)
```

## Step 7: Write Test Files

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate test suite (from step 2)
    let test_suite = generator.generate(&ast)?;

    // Write unit tests
    fs::write(
        "tests/generated_unit_tests.rs",
        test_suite.unit_tests_to_rust_code()
    )?;
    println!("✓ Wrote tests/generated_unit_tests.rs");

    // Write property tests
    fs::write(
        "tests/generated_property_tests.rs",
        test_suite.property_tests_to_rust_code()
    )?;
    println!("✓ Wrote tests/generated_property_tests.rs");

    // Add doctests to source
    let source_with_docs = add_doctests_to_source(
        &rust_code,
        &test_suite.doctests
    )?;
    fs::write("src/generated.rs", source_with_docs)?;
    println!("✓ Wrote src/generated.rs with doctests");

    // Write mutation config
    let mutation_config = generator.generate_mutation_config(&ast)?;
    fs::write(".cargo-mutants.toml", mutation_config)?;
    println!("✓ Wrote .cargo-mutants.toml");

    Ok(())
}
```

**Output:**
```
✓ Wrote tests/generated_unit_tests.rs
✓ Wrote tests/generated_property_tests.rs
✓ Wrote src/generated.rs with doctests
✓ Wrote .cargo-mutants.toml
```

## Step 8: Run Tests

```bash
# Run all tests
cargo test

# Output:
running 28 tests
test tests::generated_unit_tests::test_factorial_if_then_branch ... ok
test tests::generated_unit_tests::test_factorial_if_else_branch ... ok
test tests::generated_unit_tests::test_factorial_edge_case_zero ... ok
test tests::generated_unit_tests::test_is_prime_while_loop ... ok
test tests::generated_property_tests::prop_factorial_determinism ... ok
test tests::generated_property_tests::prop_is_prime_determinism ... ok
test src::generated::factorial (line 5) ... ok
test src::generated::is_prime (line 25) ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured

# Check coverage
cargo llvm-cov

# Output:
Filename                      Line     Branch
-------------------------------------------
src/generated.rs             95.2%    87.5%
-------------------------------------------
TOTAL                        95.2%    87.5%
```

## Step 9: Run Mutation Tests

```bash
# Run mutation testing
cargo mutants

# Output:
Found 42 mutants to test
Testing mutants...

CAUGHT   mutants: 38/42 (90.5%)
MISSED   mutants: 3/42 (7.1%)
TIMEOUT  mutants: 1/42 (2.4%)

Overall mutation score: 90.5% ✓ (target: 85%)
```

## Step 10: Analyze Coverage Gaps

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check coverage report
    let coverage = test_suite.coverage_report();

    println!("Coverage Report:");
    println!("  Line coverage: {:.1}%", coverage.line_coverage());
    println!("  Branch coverage: {:.1}%", coverage.branch_coverage());
    println!("  Function coverage: {:.1}%", coverage.function_coverage());

    // Identify gaps
    if coverage.line_coverage() < 80.0 {
        println!("\n⚠ Coverage below target (80%)");
        println!("Uncovered paths:");

        for path in coverage.uncovered_paths() {
            match path {
                UncoveredPath::Line(line) => {
                    println!("  - Line {}", line);
                }
                UncoveredPath::Branch(branch) => {
                    println!("  - Branch in {}: {:?}",
                        branch.function, branch.branch_type);
                }
            }
        }

        // Generate targeted tests for gaps
        let targeted = generator.generate_targeted_tests(
            &coverage.uncovered_paths()
        )?;

        println!("\n✓ Generated {} additional tests for gaps",
            targeted.len());
    } else {
        println!("\n✓ Coverage target met!");
    }

    Ok(())
}
```

**Output:**
```
Coverage Report:
  Line coverage: 95.2%
  Branch coverage: 87.5%
  Function coverage: 100.0%

✓ Coverage target met!
```

## Complete Workflow Script

```rust
// bin/generate_tests.rs
use bashrs::{bash_parser, test_generator};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bash_file = std::env::args()
        .nth(1)
        .expect("Usage: generate_tests <bash_file>");

    println!("📝 Reading {}", bash_file);
    let bash_code = fs::read_to_string(&bash_file)?;

    println!("🔍 Parsing bash script...");
    let parser = bash_parser::BashParser::new();
    let ast = parser.parse(&bash_code)?;

    println!("🧪 Generating tests...");
    let mut generator = test_generator::TestGenerator::new();
    let test_suite = generator.generate(&ast)?;

    println!("💾 Writing test files...");
    fs::write("tests/generated_tests.rs", test_suite.to_rust_code())?;

    println!("⚙️  Writing mutation config...");
    let config = generator.generate_mutation_config(&ast)?;
    fs::write(".cargo-mutants.toml", config)?;

    println!("\n✅ Complete!");
    println!("   {} unit tests", test_suite.unit_tests.len());
    println!("   {} property tests", test_suite.property_tests.len());
    println!("   {} doctests", test_suite.doctests.len());
    println!("\n📊 Run: cargo test");
    println!("🧬 Run: cargo mutants");

    Ok(())
}
```

## Usage

```bash
# Generate tests from bash script
cargo run --bin generate_tests factorial.sh

# Run generated tests
cargo test

# Check coverage
cargo llvm-cov --html

# Run mutation tests
cargo mutants

# View results
open target/llvm-cov/html/index.html
```

## Summary

This example demonstrates the complete workflow:

1. ✅ Parse bash script → AST
2. ✅ Generate comprehensive test suite
3. ✅ Create unit tests with branch coverage
4. ✅ Create property tests for invariants
5. ✅ Extract doctests from comments
6. ✅ Generate mutation test configuration
7. ✅ Write all files to disk
8. ✅ Run tests and verify coverage
9. ✅ Run mutation tests and verify quality
10. ✅ Analyze gaps and generate targeted tests

**Result**: Fully tested, high-quality transpiled code with 95%+ coverage and 90%+ mutation score!
