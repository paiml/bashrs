//! bashrs Native Linter
//!
//! Provides ShellCheck-equivalent linting for both:
//! - Ingested shell scripts (Bash → Rust conversion)
//! - Generated shell scripts (Rust → Shell transpilation)
//!
//! This linter operates at the AST level, providing deeper semantic analysis
//! than token-based linters.
//!
//! ## Safety Note
//! Linter rules use unwrap() on regex captures after pattern matching.
//! These are checked invariants - if the regex matched, the capture exists.
//! This is a performance optimization for hot paths (linting 1000s of lines).
//! Some rules are placeholders with unused variables/statics during development.
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod autofix;
pub mod citl;
pub mod diagnostic;
pub mod ignore_file;
pub mod make_preprocess;
pub mod output;
pub mod rule_registry;
pub mod rules;
pub mod shell_compatibility;
pub mod shell_type;
pub mod suppression;

pub use autofix::{apply_fixes, apply_fixes_to_file, FixOptions, FixResult};
pub use citl::{CitlDiagnostic, CitlExport, CitlSpan, CitlSuggestion, CitlSummary};
pub use diagnostic::{Diagnostic, Fix, FixSafetyLevel, LintResult, Severity, Span};
pub use ignore_file::{IgnoreFile, IgnoreResult};
pub use rule_registry::{get_rule_compatibility, should_apply_rule, RuleMetadata};
pub use rules::{lint_shell, lint_shell_with_path};
pub use shell_compatibility::ShellCompatibility;
pub use shell_type::{detect_shell_type, ShellType};
pub use suppression::{Suppression, SuppressionManager, SuppressionType};

#[cfg(test)]
mod tests;
