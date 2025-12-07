//! # Rash - Rust to Shell Transpiler
// Allow uppercase TASK_IDs in test function names (EXTREME TDD naming convention)
#![cfg_attr(test, allow(non_snake_case))]
// Allow unwrap in test code - tests should panic on unexpected conditions
#![cfg_attr(test, allow(clippy::unwrap_used))]
// Allow indexing in test code - tests should panic on out-of-bounds
#![cfg_attr(test, allow(clippy::indexing_slicing))]
// Allow absurd extreme comparisons (defensive test assertions like usize >= 0)
// TODO(v2.1.0): Clean up these assertions - Issue #TBD
#![allow(clippy::absurd_extreme_comparisons)]
// Allow multiple crate versions - transitive dependencies from different crates
#![allow(clippy::multiple_crate_versions)]
//!
//! Rash is a Rust-to-POSIX shell script transpiler that generates safe, deterministic,
//! and verifiable shell scripts from a restricted Rust subset.
//!
//! ## Features
//!
//! - **POSIX Compliance**: Generated scripts work on sh, dash, bash, and ash
//! - **Safety**: Injection attack prevention, proper quoting, verified output
//! - **Determinism**: Same input always produces identical output
//! - **ShellCheck Integration**: All output passes shellcheck validation
//!
//! ## Quick Start
//!
//! ```rust
//! use bashrs::{transpile, Config};
//!
//! let rust_code = r#"
//!     fn main() {
//!         let greeting = "Hello, World!";
//!         echo(greeting);
//!     }
//!
//!     fn echo(msg: &str) {}
//! "#;
//!
//! let shell_script = transpile(rust_code, Config::default()).unwrap();
//! assert!(shell_script.contains("#!/bin/sh"));
//! ```
//!
//! ## Main Functions
//!
//! - [`transpile`]: Convert Rust code to shell script
//! - [`check`]: Validate Rust code without generating output
//!
//! ## Configuration
//!
//! ```rust
//! use bashrs::{Config, transpile};
//! use bashrs::models::{ShellDialect, VerificationLevel};
//!
//! let config = Config {
//!     target: ShellDialect::Posix,
//!     verify: VerificationLevel::Strict,
//!     optimize: true,
//!     ..Config::default()
//! };
//!
//! let rust_code = "fn main() { let x = 42; }";
//! let result = transpile(rust_code, config);
//! assert!(result.is_ok());
//! ```

/// Abstract syntax tree types and validation
pub mod ast;
/// Bash script parsing and AST generation
pub mod bash_parser;
/// Bash quality tools (test generation, coverage, formatting, scoring)
pub mod bash_quality;
/// Bash script transpilation and purification
pub mod bash_transpiler;
/// build.rs integration with auto-discovery
pub mod build_rs;
/// Command-line interface for bashrs
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
/// Rust compiler integration for transpilation
#[cfg(not(target_arch = "wasm32"))]
pub mod compiler;
/// Shell configuration file management and analysis
pub mod config;
/// Container and sandbox support
#[cfg(not(target_arch = "wasm32"))]
pub mod container;
/// Shell script code emission
pub mod emitter;
/// Formal verification and proof generation
pub mod formal;
/// Shell script formatting
pub mod formatter;
/// Quality gate configuration and enforcement
pub mod gates;
/// Intermediate representation for transpilation
pub mod ir;
/// Shell script linting with ShellCheck-equivalent rules
pub mod linter;
/// Makefile parsing and purification
pub mod make_parser;
/// Configuration types and error handling
pub mod models;
/// Quality gates with rich reporting and fault localization (ML-001 to ML-012)
pub mod quality;
/// Interactive REPL with integrated debugger
pub mod repl;
/// Parser and compiler services
pub mod services;
/// Standard library function mappings
pub mod stdlib;
/// Test case generation from shell scripts
pub mod test_generator;
/// Tracing infrastructure for diagnostics and debugging
pub mod tracing;
/// Builder API for programmatic transpilation
pub mod transpiler;
/// Type system with taint tracking for injection safety
pub mod types;
/// AST and output validation
pub mod validation;
/// Output verification and shellcheck integration
pub mod verifier;

#[cfg(test)]
pub mod testing;

// WebAssembly support (Phase 0: Feasibility Study)
#[cfg(feature = "wasm")]
pub mod wasm;

pub use models::{Config, Error, Result};
pub use transpiler::Transpiler;

/// Transpile Rust source code to POSIX shell script.
///
/// This is the main entry point for the Rash transpiler. It takes Rust source code
/// and converts it to a POSIX-compliant shell script with full validation.
///
/// # Arguments
///
/// * `input` - Rust source code as a string
/// * `config` - Configuration options for transpilation
///
/// # Returns
///
/// * `Ok(String)` - Generated shell script
/// * `Err(Error)` - Transpilation error (parse, validation, emission)
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use bashrs::{transpile, Config};
///
/// let rust_code = r#"
///     fn main() {
///         let message = "Hello from Rash";
///         echo(message);
///     }
///     fn echo(msg: &str) {}
/// "#;
///
/// let shell_script = transpile(rust_code, Config::default()).unwrap();
/// assert!(shell_script.contains("#!/bin/sh"));
/// assert!(shell_script.contains("message="));
/// ```
///
/// ## With Custom Configuration
///
/// ```rust
/// use bashrs::{transpile, Config};
/// use bashrs::models::{ShellDialect, VerificationLevel};
///
/// let config = Config {
///     target: ShellDialect::Bash,
///     verify: VerificationLevel::Paranoid,
///     optimize: true,
///     ..Config::default()
/// };
///
/// let rust_code = "fn main() { let x = 1 + 2; }";
/// let result = transpile(rust_code, config);
/// assert!(result.is_ok());
/// ```
///
/// ## Variable Assignment
///
/// ```rust
/// use bashrs::{transpile, Config};
///
/// let rust_code = r#"
///     fn main() {
///         let name = "Alice";
///         let age = 30;
///         let active = true;
///     }
/// "#;
///
/// let shell_script = transpile(rust_code, Config::default()).unwrap();
/// assert!(shell_script.contains("name="));
/// assert!(shell_script.contains("age="));
/// ```
///
/// ## Function Calls
///
/// ```rust
/// use bashrs::{transpile, Config};
///
/// let rust_code = r#"
///     fn main() {
///         greet("World");
///     }
///     fn greet(name: &str) {}
/// "#;
///
/// let shell_script = transpile(rust_code, Config::default()).unwrap();
/// assert!(shell_script.contains("greet"));
/// ```
///
/// # Errors
///
/// Returns `Err` if:
/// - Input cannot be parsed as valid Rust
/// - AST validation fails (unsupported features)
/// - IR generation fails
/// - Shell code emission fails
/// - Output validation fails (shellcheck, safety checks)
pub fn transpile(input: &str, config: Config) -> Result<String> {
    let validation_pipeline = validation::pipeline::ValidationPipeline::new(&config);

    let ast = services::parser::parse(input)?;
    ast::validate(&ast)?;
    validation_pipeline.validate_ast(&ast)?;

    let ir = ir::from_ast(&ast)?;
    validation_pipeline.validate_ir(&ir)?;

    let optimized = ir::optimize(ir, &config)?;
    let shell_code = emitter::emit(&optimized, &config)?;

    validation_pipeline.validate_output(&shell_code)?;

    Ok(shell_code)
}

/// Check if the given Rust code is valid for transpilation without generating output.
///
/// This function parses and validates the input Rust code but does not generate
/// shell script output. Useful for quick validation or IDE integration.
///
/// # Arguments
///
/// * `input` - Rust source code as a string
///
/// # Returns
///
/// * `Ok(())` - Code is valid and can be transpiled
/// * `Err(Error)` - Code has syntax or validation errors
///
/// # Examples
///
/// ## Valid Code
///
/// ```rust
/// use bashrs::check;
///
/// let valid_code = r#"
///     fn main() {
///         let x = 42;
///         let y = "hello";
///     }
/// "#;
///
/// assert!(check(valid_code).is_ok());
/// ```
///
/// ## Invalid Syntax
///
/// ```rust
/// use bashrs::check;
///
/// let invalid_code = "fn main( { }"; // Missing closing paren
/// assert!(check(invalid_code).is_err());
/// ```
///
/// ## Unsupported Features
///
/// ```rust
/// use bashrs::check;
///
/// let unsupported = r#"
///     fn main() {
///         let v = vec![1, 2, 3]; // Vec not supported in v1.0
///     }
/// "#;
///
/// // This may fail validation depending on the restricted subset
/// let result = check(unsupported);
/// ```
///
/// # Use Cases
///
/// - **IDE Integration**: Fast syntax checking without full transpilation
/// - **CI/CD Validation**: Verify Rash code before transpilation
/// - **Development**: Quick feedback loop for code validity
pub fn check(input: &str) -> Result<()> {
    let ast = services::parser::parse(input)?;
    ast::validate(&ast)?;
    Ok(())
}
