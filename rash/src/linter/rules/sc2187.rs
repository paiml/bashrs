// SC2187: Ash scripts checked as Bash
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Metadata-based, requires shebang analysis
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2187_placeholder() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
}
