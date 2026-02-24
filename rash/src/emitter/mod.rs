//! # Shell Code Emitter Module
//!
//! This module is responsible for generating shell scripts from the Intermediate
//! Representation (IR). It ensures that all generated shell code is safe,
//! deterministic, and compliant with the target shell dialect.
//!
//! ## Features
//!
//! - **POSIX Compliance**: Generates portable shell scripts that work across different shells
//! - **Safety Guarantees**: Proper escaping and quoting to prevent injection attacks
//! - **Deterministic Output**: Same input always produces identical output
//! - **Multiple Dialects**: Support for POSIX sh, Bash, and other shell variants
//!
//! ## Safety Note
//! Emitter operations use fallible methods with proper error handling.
//! Production code MUST NOT use unwrap() (Cloudflare-class defect prevention).
//!
//! ## Architecture
//!
//! The emitter consists of:
//! - **Escape Module**: Handles string escaping and shell-safe formatting
//! - **POSIX Emitter**: Generates POSIX-compliant shell code
//! - **Dialect Support**: Extensible architecture for different shell dialects
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust
//! use bashrs::emitter::emit;
//! use bashrs::ir::{ShellIR, ShellValue, Command};
//! use bashrs::ir::effects::EffectSet;
//! use bashrs::models::Config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a simple echo command
//! let ir = ShellIR::Exec {
//!     cmd: Command {
//!         program: "echo".to_string(),
//!         args: vec![ShellValue::String("Hello, World!".to_string())],
//!     },
//!     effects: EffectSet::pure(),
//! };
//!
//! // Generate shell code
//! let config = Config::default();
//! let shell_code = emit(&ir, &config)?;
//!
//! assert!(shell_code.contains("echo 'Hello, World!'"));
//! # Ok(())
//! # }
//! ```
//!
//! ### Variable Assignment
//!
//! ```rust
//! use bashrs::emitter::emit;
//! use bashrs::ir::{ShellIR, ShellValue};
//! use bashrs::ir::effects::EffectSet;
//! use bashrs::models::Config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let ir = ShellIR::Let {
//!     name: "USERNAME".to_string(),
//!     value: ShellValue::String("alice".to_string()),
//!     effects: EffectSet::pure(),
//! };
//!
//! let config = Config::default();
//! let shell_code = emit(&ir, &config)?;
//!
//! assert!(shell_code.contains("USERNAME=") && shell_code.contains("alice"));
//! # Ok(())
//! # }
//! ```
//!
//! ### Safe String Escaping
//!
//! ```rust
//! use bashrs::emitter::emit;
//! use bashrs::ir::{ShellIR, ShellValue, Command};
//! use bashrs::ir::effects::EffectSet;
//! use bashrs::models::Config;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Even with special characters, output is safe
//! let ir = ShellIR::Exec {
//!     cmd: Command {
//!         program: "echo".to_string(),
//!         args: vec![ShellValue::String("Hello $USER; rm -rf /".to_string())],
//!     },
//!     effects: EffectSet::pure(),
//! };
//!
//! let config = Config::default();
//! let shell_code = emit(&ir, &config)?;
//!
//! // Special characters are safely escaped
//! assert!(shell_code.contains("'Hello $USER; rm -rf /'"));
//! # Ok(())
//! # }
//! ```

// All expect() calls in dockerfile emitter are guarded by preceding bounds
// checks or is_some() guards — safe code-generation invariants.
#[allow(clippy::expect_used)]
pub mod dockerfile;
pub mod escape;
#[allow(clippy::expect_used)] // Makefile emitter uses expect() for code-generation invariants
pub mod makefile;
#[allow(clippy::expect_used)] // POSIX emitter uses expect() for code-generation invariants
pub mod posix;
pub mod trace;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod posix_tests;

#[cfg(test)]
#[path = "emitter_coverage_tests.rs"]
mod emitter_coverage_tests;

#[cfg(test)]
#[path = "makefile_coverage_tests.rs"]
mod makefile_coverage_tests;

#[cfg(test)]
#[path = "posix_coverage_tests.rs"]
mod posix_coverage_tests;

pub use posix::PosixEmitter;

use crate::ir::ShellIR;
use crate::models::{Config, Result};

pub use trace::{DecisionTrace, TranspilerDecision};

/// Emit shell code from IR based on target dialect
///
/// This function selects the appropriate emitter based on the configured
/// shell dialect and generates the corresponding shell code.
///
/// # Arguments
///
/// * `ir` - The intermediate representation to emit
/// * `config` - Configuration specifying target dialect and options
///
/// # Returns
///
/// A `Result` containing the generated shell code as a string, or an error
/// if emission fails.
///
/// # Examples
///
/// ```rust
/// use bashrs::emitter::emit;
/// use bashrs::ir::ShellIR;
/// use bashrs::models::{Config, config::ShellDialect};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let ir = ShellIR::Noop;
///
/// // Target POSIX shell
/// let mut config = Config::default();
/// config.target = ShellDialect::Posix;
///
/// let shell_code = emit(&ir, &config)?;
/// // The generated code includes boilerplate for safety
/// assert!(shell_code.contains("main()"));
/// # Ok(())
/// # }
/// ```
pub fn emit(ir: &ShellIR, config: &Config) -> Result<String> {
    match config.target {
        crate::models::config::ShellDialect::Posix => {
            let emitter = PosixEmitter::new(config.clone());
            emitter.emit(ir)
        }
        crate::models::config::ShellDialect::Bash => {
            // For now, use POSIX emitter for Bash too
            let emitter = PosixEmitter::new(config.clone());
            emitter.emit(ir)
        }
        _ => {
            // Default to POSIX for other dialects
            let emitter = PosixEmitter::new(config.clone());
            emitter.emit(ir)
        }
    }
}

/// Emit shell code from IR and return the decision trace for fault localization.
pub fn emit_with_trace(ir: &ShellIR, config: &Config) -> Result<(String, DecisionTrace)> {
    let emitter = PosixEmitter::new(config.clone());
    let output = emitter.emit(ir)?;
    let trace = emitter.take_trace();
    Ok((output, trace))
}
