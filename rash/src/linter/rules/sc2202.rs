// SC2202: Order sensitivity (e.g., redirects)
#[allow(unused_imports)]
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Complex ordering analysis
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2202_placeholder() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
}
