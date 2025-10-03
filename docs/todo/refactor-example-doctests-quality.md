# Refactor Plan: Examples, Documentation, and Code Quality for Crates.io Release

## Overview
This document outlines a comprehensive refactoring plan to prepare Rash for publication on crates.io. The plan addresses code quality hotspots, documentation improvements, and the creation of idiomatic Rust examples following patterns from the sister project `paiml-mcp-agent-toolkit`.

## Phase 1: Address Code Complexity Hotspots

### Critical Complexity Issues (Top 5 Functions to Refactor)

1. **`ValidationPipeline::validate_expr`** (cyclomatic complexity: 32)
   - File: `./rash/src/validation/pipeline.rs`
   - Action: Break down into smaller, focused validation methods
   - Target complexity: < 10

2. **`setup_database_integration`** (cyclomatic complexity: 27)
   - File: `./tests/open_source/nodejs_project_bootstrap.rs`
   - Action: Extract database setup steps into helper functions
   - Target complexity: < 10

3. **`ProofInspector::generate_report`** (cyclomatic complexity: 25)
   - File: `./rash/src/formal/inspector.rs`
   - Action: Separate report generation logic by proof type
   - Target complexity: < 10

4. **`KeymapEngine::key_event_to_string`** (cyclomatic complexity: 25)
   - File: `./rash/src/playground/editor.rs`
   - Action: Use pattern matching with helper functions
   - Target complexity: < 10

5. **`install_framework_dependencies`** (cyclomatic complexity: 25)
   - File: `./tests/open_source/nodejs_project_bootstrap.rs`
   - Action: Create framework-specific installation modules
   - Target complexity: < 10

### High Complexity Files to Refactor

1. **`posix.rs`** (Total Cyclomatic: 256)
   - Break down large emit functions
   - Extract common patterns into helper functions
   - Add comprehensive documentation

2. **`restricted.rs`** (Total Cyclomatic: 240)
   - Simplify AST validation logic
   - Create type-specific validators
   - Improve error messages

3. **`pipeline.rs`** (Total Cyclomatic: 182)
   - Modularize validation stages
   - Implement builder pattern for pipeline configuration
   - Add validation rule documentation

## Phase 2: Add Documentation Tests

### Core Module Documentation

For each core module, add:
- Module-level documentation with examples
- Function-level documentation with code examples
- Error handling examples
- Integration examples

#### Priority Modules for Doc Tests:

1. **`rash/src/ast/`**
   ```rust
   //! # AST Module
   //! 
   //! This module provides the Abstract Syntax Tree representation for Rash.
   //! 
   //! ## Examples
   //! 
   //! ```rust
   //! use bashrs::ast::{RestrictedAst, Function};
   //! 
   //! # fn main() -> Result<(), Box<dyn std::error::Error>> {
   //! let ast = RestrictedAst::new(vec![
   //!     Function::new("main", vec![], vec![])
   //! ]);
   //! assert!(ast.validate().is_ok());
   //! # Ok(())
   //! # }
   //! ```
   ```

2. **`rash/src/emitter/`**
   - Document POSIX compliance guarantees
   - Add examples for each shell construct
   - Include safety examples

3. **`rash/src/verifier/`**
   - Document verification levels
   - Add examples of verification failures
   - Include property examples

4. **`rash/src/ir/`**
   - Document IR transformation process
   - Add optimization examples
   - Include effect analysis examples

## Phase 3: Create Idiomatic Rust Examples

### Example Structure (following paiml-mcp-agent-toolkit pattern)

```
examples/
├── basic/
│   ├── hello_world.rs          # Minimal transpilation example
│   ├── variables.rs            # Variable usage and escaping
│   └── functions.rs            # Function calls and arguments
├── control_flow/
│   ├── conditionals.rs         # If/else statements
│   ├── loops.rs                # For/while loops
│   └── pattern_matching.rs     # Match expressions
├── safety/
│   ├── injection_prevention.rs # Command injection prevention
│   ├── escaping.rs            # String escaping examples
│   └── validation.rs          # Input validation
├── advanced/
│   ├── optimization.rs         # Optimization levels
│   ├── verification.rs        # Verification examples
│   └── custom_emitter.rs      # Custom shell dialects
└── bootstrap/
    ├── minimal_installer.rs    # Minimal bootstrap script
    ├── package_manager.rs      # Package installation
    └── system_setup.rs        # System configuration

```

### Example Template

Each example should follow this structure:

```rust
//! # Example: [Name]
//! 
//! This example demonstrates [what it does].
//! 
//! ## Usage
//! 
//! ```bash
//! cargo run --example basic/hello_world
//! ```
//! 
//! ## Expected Output
//! 
//! The generated shell script will [describe output].

use bashrs::prelude::*;

#[bashrs::main]
fn main() {
    // Example implementation
    println!("Hello from Rash!");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example_output() {
        // Test the example produces expected output
    }
}
```

## Phase 4: Cargo Configuration for Examples

Update `Cargo.toml`:

```toml
[[example]]
name = "hello_world"
path = "examples/basic/hello_world.rs"

[[example]]
name = "injection_prevention"
path = "examples/safety/injection_prevention.rs"
required-features = ["verification"]

# ... more examples
```

## Phase 5: README and Documentation Enhancement

### README.md Improvements

1. **Add Badges**
   ```markdown
   [![Crates.io](https://img.shields.io/crates/v/bashrs.svg)](https://crates.io/crates/bashrs)
   [![Documentation](https://docs.rs/bashrs/badge.svg)](https://docs.rs/bashrs)
   [![License](https://img.shields.io/crates/l/bashrs.svg)](LICENSE)
   [![CI](https://github.com/paiml/bashrs/workflows/CI/badge.svg)](https://github.com/paiml/bashrs/actions)
   ```

2. **Quick Start Section**
   ```markdown
   ## Quick Start
   
   ```rust
   use bashrs::prelude::*;
   
   #[bashrs::main]
   fn main() {
       let name = env("USER");
       println!("Hello, {name}!");
   }
   ```
   
   Compile to shell:
   ```bash
   bashrs build hello.rs -o hello.sh
   ```
   ```

3. **Installation Methods**
   - From crates.io
   - From source
   - Using cargo-binstall
   - Container images

4. **Feature Matrix**
   - Supported Rust constructs
   - Shell compatibility table
   - Verification levels

## Phase 6: API Documentation

### Priority Documentation Areas

1. **Public API Surface**
   - Document all public types
   - Add examples to trait implementations
   - Include error handling patterns

2. **Builder APIs**
   ```rust
   /// Build a Rash transpilation pipeline
   /// 
   /// # Examples
   /// 
   /// ```rust
   /// use bashrs::builder::TranspilerBuilder;
   /// 
   /// let transpiler = TranspilerBuilder::new()
   ///     .optimization_level(OptimizationLevel::Size)
   ///     .verification_level(VerificationLevel::Strict)
   ///     .target_shell(ShellDialect::Posix)
   ///     .build()?;
   /// ```
   ```

3. **Error Types**
   - Document all error variants
   - Add recovery examples
   - Include common troubleshooting

## Phase 7: Integration Tests as Examples

Create integration tests that serve as examples:

```rust
// tests/examples/mod.rs
#[test]
fn example_bootstrap_script() {
    let source = include_str!("../../examples/bootstrap/minimal_installer.rs");
    let result = bashrs::transpile(source)?;
    
    // Verify POSIX compliance
    assert!(shellcheck::verify(&result.shell_script).is_ok());
    
    // Verify determinism
    let result2 = bashrs::transpile(source)?;
    assert_eq!(result.shell_script, result2.shell_script);
}
```

## Phase 8: Release Preparation

### Documentation Checklist

- [ ] All public APIs have doc comments
- [ ] All modules have module-level documentation
- [ ] Examples compile and run correctly
- [ ] Doc tests pass (`cargo test --doc`)
- [ ] README is comprehensive and accurate
- [ ] CHANGELOG.md is up to date
- [ ] API breaking changes are documented
- [ ] Migration guide for breaking changes

### Code Quality Checklist

- [ ] All complexity hotspots addressed (cyclomatic < 10)
- [ ] No clippy warnings
- [ ] Test coverage > 85%
- [ ] All TODO comments resolved or documented
- [ ] Consistent error handling patterns
- [ ] Performance benchmarks documented

### Example Quality Checklist

- [ ] Examples cover all major use cases
- [ ] Examples are self-contained
- [ ] Examples include error handling
- [ ] Examples have tests
- [ ] Examples follow Rust idioms
- [ ] Examples are referenced in documentation

## Success Metrics

1. **Documentation Coverage**: 100% of public APIs documented
2. **Example Coverage**: At least one example per major feature
3. **Code Complexity**: No function with cyclomatic complexity > 10
4. **Test Coverage**: Line coverage > 85%
5. **Doc Test Success**: All documentation examples compile and run
6. **User Feedback**: Clear path from README to working example

## Timeline

- Week 1: Address complexity hotspots
- Week 2: Add documentation tests
- Week 3: Create example suite
- Week 4: Polish and release preparation

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) - Official Rust API guidelines
- [Crates.io Publishing Guide](https://doc.rust-lang.org/cargo/reference/publishing.html) - Publishing documentation