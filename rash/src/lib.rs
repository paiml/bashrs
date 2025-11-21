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

pub mod ast;
pub mod bash_parser;
pub mod bash_quality; // NEW: Bash quality tools (test, coverage, format, score)
pub mod bash_transpiler;
pub mod build_rs; // NEW: build.rs integration with auto-discovery (v7.1 - Issue #25)
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
#[cfg(not(target_arch = "wasm32"))]
pub mod compiler;
pub mod config; // NEW: Shell config file management (v7.0)
#[cfg(not(target_arch = "wasm32"))]
pub mod container;
pub mod emitter;
pub mod formal;
pub mod formatter;
pub mod ir;
pub mod linter;
pub mod make_parser; // NEW: Makefile parsing and purification
pub mod models;
pub mod repl; // NEW: Interactive REPL with integrated debugger (v7.0 - Phase 0)
pub mod services;
pub mod stdlib;
pub mod test_generator;
pub mod tracing; // NEW: Tracing infrastructure for bash purification/linting (v7.0 - Phase 1)
pub mod transpiler; // NEW: Builder API for programmatic transpilation (v7.1 - Issue #25)
pub mod validation;
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
