# Test Generator Guide

## Overview

The Rash Test Generator automatically creates comprehensive test suites for bash-to-rust transpiled code, including:
- **Unit Tests**: Branch coverage, edge cases, error cases
- **Property Tests**: Determinism, idempotency, bounds checking
- **Doctests**: Extracted from comments and usage examples
- **Mutation Config**: Complexity-based .cargo-mutants.toml generation

## Quick Start

```rust
use bashrs::test_generator::TestGenerator;
use bashrs::bash_parser::BashParser;

// Parse your bash script
let bash_code = r#"
#!/bin/bash
# Example: factorial(5) => 120
factorial() {
    local n=$1
    if [ $n -le 1 ]; then
        echo 1
        return
    fi
    local prev=$(factorial $((n - 1)))
    echo $((n * prev))
}
"#;

let parser = BashParser::new();
let ast = parser.parse(bash_code)?;

// Generate comprehensive test suite
let mut generator = TestGenerator::new();
let test_suite = generator.generate(&ast)?;

// Get Rust test code
let rust_tests = test_suite.to_rust_code();
println!("{}", rust_tests);

// Get mutation config
let config = generator.generate_mutation_config(&ast)?;
std::fs::write(".cargo-mutants.toml", config)?;
```

## Generated Output

### Unit Tests

```rust
#[test]
fn test_factorial_if_then_branch() {
    // Test if-then branch
    factorial(5);
}

#[test]
fn test_factorial_edge_case_empty_string() {
    // Test with empty string input
    factorial("");
}

#[test]
fn test_factorial_edge_case_negative() {
    // Test with negative value
    factorial(-1);
}

#[test]
#[should_panic(expected = "Invalid input")]
fn test_factorial_error_invalid_input() {
    factorial("invalid");
}
```

### Property Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_factorial_determinism(
        arg0 in 0..=100,
    ) {
        // Test determinism: same input → same output
        let result1 = factorial(arg0);
        let result2 = factorial(arg0);
        prop_assert_eq!(result1, result2);
    }
}

proptest! {
    #[test]
    fn prop_factorial_bounds_0_100(
        arg0 in -10..=110,
    ) {
        // Test bounds: result in range [0, 100]
        let result = factorial(arg0);
        prop_assert!(result >= 0);
        prop_assert!(result <= 100);
    }
}
```

### Doctests

```rust
/// # Examples
///
/// ```
/// use crate::factorial;
/// factorial(5)
/// assert_eq!(result, 120);
/// ```
```

### Mutation Configuration

```toml
# Generated mutation test configuration
# Auto-generated based on code complexity analysis

timeout = 65
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
```

## Comment Patterns for Doctests

The doctest generator recognizes these patterns:

### Arrow Syntax
```bash
# Example: factorial(5) => 120
```

### Multi-line Usage + Output
```bash
# Usage: greet("Alice")
# Output: Hello, Alice!
```

### Case-Insensitive
```bash
# EXAMPLE: process(data) => result
# example: validate(input) => true
```

## Coverage Analysis

The test generator tracks coverage metrics:

```rust
let coverage = test_suite.coverage_report();
println!("Line coverage: {:.1}%", coverage.line_coverage());
println!("Branch coverage: {:.1}%", coverage.branch_coverage());
println!("Uncovered paths: {}", coverage.uncovered_count());
```

## Quality Gates

The generator enforces quality thresholds:

```rust
let options = TestGenOptions {
    target_coverage: 0.80,           // 80% minimum
    target_mutation_score: 0.85,     // 85% minimum
    generate_unit_tests: true,
    generate_property_tests: true,
    generate_doctests: true,
    generate_mutation_config: true,
};

let mut generator = TestGenerator::with_options(options);
```

## Complexity-Based Configuration

The mutation config generator analyzes:

1. **Cyclomatic Complexity**: Identifies critical paths (>10)
2. **Function Count**: Scales parallel jobs (2/4/8)
3. **Loop Count**: Increases timeout (10s per loop)
4. **Arithmetic Operations**: Enables arithmetic mutation operators
5. **Branch Count**: Enables relational/boolean operators

### Example: Simple Function
```bash
simple_func() {
    echo "Hello"
}
```
**Generated config**: timeout=65s, jobs=2

### Example: Complex Function
```bash
complex_func() {
    for i in {1..10}; do
        if [ $i -gt 5 ]; then
            echo "High"
        elif [ $i -gt 3 ]; then
            echo "Medium"
        else
            echo "Low"
        fi
    done
}
```
**Generated config**: timeout=75s, jobs=4, marked as critical

## Integration with Transpiler

```rust
use bashrs::{transpile, Config};
use bashrs::test_generator::TestGenerator;

// 1. Transpile bash to rust
let bash_code = std::fs::read_to_string("script.sh")?;
let rust_code = transpile(&bash_code, Config::default())?;

// 2. Generate tests
let ast = bash_parser::parse(&bash_code)?;
let mut generator = TestGenerator::new();
let test_suite = generator.generate(&ast)?;

// 3. Write test file
std::fs::write(
    "tests/generated_tests.rs",
    test_suite.to_rust_code()
)?;

// 4. Write mutation config
std::fs::write(
    ".cargo-mutants.toml",
    generator.generate_mutation_config(&ast)?
)?;

// 5. Run tests
std::process::Command::new("cargo")
    .args(&["test"])
    .status()?;

// 6. Run mutation tests
std::process::Command::new("cargo")
    .args(&["mutants"])
    .status()?;
```

## Advanced Features

### Targeted Test Generation

Generate tests for specific uncovered paths:

```rust
let uncovered = coverage.uncovered_paths();
let targeted_tests = generator.generate_targeted_tests(&uncovered)?;
```

### Custom Assertion Strategies

```rust
use bashrs::test_generator::Assertion;

let custom_test = UnitTest {
    name: "test_custom".to_string(),
    test_fn: "my_function()".to_string(),
    assertions: vec![
        Assertion::True {
            condition: "result.is_ok()".to_string(),
        },
        Assertion::Equals {
            actual: "result.unwrap()".to_string(),
            expected: "42".to_string(),
        },
    ],
};
```

### Property Test Generators

```rust
use bashrs::test_generator::{Generator, PropertyTest, Property};

let custom_property = PropertyTest {
    name: "prop_custom".to_string(),
    property: Property::Determinism,
    generators: vec![
        Generator::Integer { min: 0, max: 1000 },
        Generator::String { pattern: "[a-z]{5,15}".to_string() },
    ],
    test_cases: 100,
};
```

## Best Practices

1. **Document Examples**: Add `# Example:` comments for automatic doctest generation
2. **Target Coverage**: Aim for ≥80% line coverage
3. **Run Mutation Tests**: Target ≥85% mutation score
4. **Review Generated Tests**: Inspect and refine before committing
5. **Iterative Generation**: Use uncovered path feedback to add tests

## Statistics

**Current Implementation:**
- **2,732 lines** of test generator code
- **40 tests** for the generator itself
- **752 total tests** in the project
- **100% Sprint completion** (Sprints 1-3)

## Next Steps

- Integrate with CI/CD pipelines
- Add support for more bash constructs
- Enhance property inference from comments
- Generate benchmark tests for performance-critical code
