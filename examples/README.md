# Rash Examples

This directory contains example Rash programs demonstrating various features and use cases.

## Important Note

These examples are written in Rash syntax, which is a subset of Rust designed to be transpiled to shell scripts. They are **not** meant to be compiled as regular Rust programs.

## Building Examples

To transpile an example to a shell script:

```bash
# From the project root
cargo run --bin rash -- build examples/basic/hello_world.rs -o hello.sh

# Make it executable and run
chmod +x hello.sh
./hello.sh
```

## Example Categories

### Basic Examples
- `basic/hello_world.rs` - Simplest possible Rash program
- `basic/variables.rs` - Variable usage and string escaping
- `basic/functions.rs` - Function calls and command execution

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
    cargo run --bin rash -- build "$example" -o "examples_output/$name.sh"
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
//! cargo run --bin rash -- build examples/category/name.rs -o name.sh
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