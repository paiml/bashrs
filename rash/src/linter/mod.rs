//! bashrs Native Linter
//!
//! Provides ShellCheck-equivalent linting for both:
//! - Ingested shell scripts (Bash → Rust conversion)
//! - Generated shell scripts (Rust → Shell transpilation)
//!
//! This linter operates at the AST level, providing deeper semantic analysis
//! than token-based linters.

pub mod diagnostic;
pub mod rules;
pub mod output;

pub use diagnostic::{Diagnostic, Fix, LintResult, Severity, Span};
pub use rules::lint_shell;

#[cfg(test)]
mod tests;
