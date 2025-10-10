//! Lint rules for shell script analysis

pub mod sc2086;
pub mod sc2046;
pub mod sc2116;

use crate::linter::LintResult;

/// Lint a shell script and return all diagnostics
pub fn lint_shell(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Parse the shell script
    // For now, we'll use a simple token-based approach
    // In production, this would use the bash_parser AST

    // Run all rules
    result.merge(sc2086::check(source));
    result.merge(sc2046::check(source));
    result.merge(sc2116::check(source));

    result
}
