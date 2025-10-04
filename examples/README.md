# Rash Examples

This directory contains example Rash programs demonstrating various features and use cases.

## Important Note

These examples are written in Rash syntax, which is a subset of Rust designed to be transpiled to shell scripts. They are **not** meant to be compiled as regular Rust programs.

## Building Examples

To transpile an example to a shell script:

```bash
# From the project root
cargo run --bin bashrs -- build examples/basic/hello_world.rs -o hello.sh

# Make it executable and run
chmod +x hello.sh
./hello.sh
```

## Example Categories

### Basic Examples
- `basic/hello_world.rs` - Simplest possible Rash program
- `basic/variables.rs` - Variable usage and string escaping
- `basic/functions.rs` - Function calls and command execution

### Book Examples

These examples correspond to the official Rash book chapters and are fully tested.

#### Chapter 2: Variables (10 examples) ✅
- `ch02_variables/ex01_basic_string.rs` - Basic string variable assignment
- `ch02_variables/ex02_integer_variables.rs` - Integer literals
- `ch02_variables/ex03_multiple_strings.rs` - Multiple variable declarations
- `ch02_variables/ex04_string_interpolation.rs` - String interpolation patterns
- `ch02_variables/ex05_special_chars.rs` - Special character escaping ($, ", *)
- `ch02_variables/ex06_boolean_values.rs` - Boolean literals (true/false)
- `ch02_variables/ex07_paths_with_spaces.rs` - Path handling with spaces
- `ch02_variables/ex08_environment_style.rs` - Environment variable patterns
- `ch02_variables/ex09_version_numbers.rs` - Version number handling
- `ch02_variables/ex10_unicode.rs` - Unicode support

#### Chapter 3: Functions (12 examples) ✅
- `ch03_functions/ex01_basic_function.rs` - No-parameter function
- `ch03_functions/ex02_function_with_params.rs` - Single parameter
- `ch03_functions/ex03_multiple_params.rs` - Multiple parameters
- `ch03_functions/ex04_nested_calls.rs` - Function calling function
- `ch03_functions/ex05_function_composition.rs` - Chained function calls
- `ch03_functions/ex06_conditional_execution.rs` - Functions with if statements
- `ch03_functions/ex07_helper_functions.rs` - Utility helpers
- `ch03_functions/ex08_installer_pattern.rs` - Real-world installer stages
- `ch03_functions/ex09_utility_functions.rs` - String/file utilities
- `ch03_functions/ex10_string_operations.rs` - String manipulation
- `ch03_functions/ex11_file_operations.rs` - File I/O operations
- `ch03_functions/ex12_download_verify.rs` - Download with verification

#### Chapter 4: Control Flow (15 examples) ⚠️
**Note**: Some examples expose transpiler bugs and are pending fixes.

Working examples (7/15):
- `ch04_control_flow/ex01_basic_if.rs` - Basic if with integer comparison ✅
- `ch04_control_flow/ex02_if_else.rs` - If-else with boolean ✅
- `ch04_control_flow/ex03_if_elif_else.rs` - If-elif-else chain ✅
- `ch04_control_flow/ex04_integer_comparisons.rs` - Comparison operators ✅
- `ch04_control_flow/ex09_nested_if.rs` - Nested conditionals ✅
- `ch04_control_flow/ex11_early_return.rs` - Early return pattern ✅
- `ch04_control_flow/ex14_boolean_variables.rs` - Boolean conditions ✅

Known issues (8/15 - transpiler bugs):
- `ch04_control_flow/ex05_string_comparison.rs` - String equality (Bug: uses -eq instead of =)
- `ch04_control_flow/ex06_logical_and.rs` - AND operator (Bug: IR generation error)
- `ch04_control_flow/ex07_logical_or.rs` - OR operator (Bug: IR generation error)
- `ch04_control_flow/ex08_not_operator.rs` - NOT operator (Bug: operator not transpiled)
- `ch04_control_flow/ex10_conditional_calls.rs` - Function dispatch (Bug: string comparison)
- `ch04_control_flow/ex12_guard_clauses.rs` - Guard pattern (Bug: logical operators)
- `ch04_control_flow/ex13_complex_logic.rs` - Complex conditions (Bug: logical operators)
- `ch04_control_flow/ex15_installer_logic.rs` - Installer logic (Bug: string comparison)

See `TEST_RESULTS.md` for detailed bug reports.

### Control Flow
- `control_flow/conditionals.rs` - If/else statements
- `control_flow/loops.rs` - Bounded loops (requires `loops` feature)
- `control_flow/pattern_matching.rs` - Match expressions (TODO)

### Safety Features
- `safety/injection_prevention.rs` - Command injection prevention
- `safety/escaping.rs` - String escaping mechanisms
- `safety/validation.rs` - Input validation (TODO)

### Advanced Examples
- `advanced/optimization.rs` - Optimization levels (TODO)
- `advanced/verification.rs` - Verification examples (TODO)
- `advanced/custom_emitter.rs` - Custom shell dialects (TODO)

### Bootstrap Scripts
- `bootstrap/minimal_installer.rs` - Minimal development environment setup
- `bootstrap/package_manager.rs` - Package installation (TODO)
- `bootstrap/system_setup.rs` - System configuration (TODO)

## Running All Examples

To transpile all examples:

```bash
# Create output directory
mkdir -p examples_output

# Transpile all examples
for example in examples/**/*.rs; do
    name=$(basename "$example" .rs)
    cargo run --bin bashrs -- build "$example" -o "examples_output/$name.sh"
done
```

## Example Template

When creating new examples, follow this template:

```rust
//! # Example: [Name]
//! 
//! Brief description of what this example demonstrates.
//! 
//! ## Usage
//! 
//! ```bash
//! cargo run --bin bashrs -- build examples/category/name.rs -o name.sh
//! ```
//! 
//! ## Expected Output
//! 
//! Description of what the generated shell script will do.

#[rash::main]
fn main() {
    // Example implementation
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_example() {
        // Test that verifies the example
    }
}
```

## Contributing

When adding new examples:
1. Place them in the appropriate category directory
2. Follow the naming convention: `snake_case.rs`
3. Include comprehensive documentation
4. Add a test to ensure the example remains valid
5. Update this README with the new example