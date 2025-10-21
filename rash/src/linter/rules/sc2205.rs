// SC2205: Array append (covered by SC2179)
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Covered by SC2179
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2205_placeholder() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
}
