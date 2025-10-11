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
pub mod autofix;

pub use diagnostic::{Diagnostic, Fix, LintResult, Severity, Span};
pub use rules::lint_shell;
pub use autofix::{apply_fixes, apply_fixes_to_file, FixOptions, FixResult};

#[cfg(test)]
mod tests;
