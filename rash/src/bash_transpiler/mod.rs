//! Bash-to-Rash Transpiler
//!
//! Converts parsed Bash AST to Rash (Rust) source code.
//! Implements pattern-based translation with semantic preservation.
//!
//! ## Design Principles
//! - Semantic equivalence: Generated Rash must have same behavior as bash
//! - Safety: All string handling uses proper escaping
//! - Idempotency: Prefer idempotent operations where possible

pub mod codegen;
pub mod patterns;
pub mod purification;
pub mod test_generator;

pub use codegen::{BashToRashTranspiler, TranspileOptions};
pub use purification::{PurificationOptions, PurificationReport, Purifier};
pub use test_generator::{TestGenerator, TestGeneratorOptions};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranspileError {
    #[error("Unsupported bash construct: {0}")]
    UnsupportedConstruct(String),

    #[error("Code generation failed: {0}")]
    CodeGenFailed(String),

    #[error("Invalid bash syntax cannot be transpiled: {0}")]
    InvalidSyntax(String),
}

pub type TranspileResult<T> = Result<T, TranspileError>;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod purification_property_tests;
