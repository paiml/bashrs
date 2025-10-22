// SC2195: Pattern will never match
#[allow(unused_imports)]
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Complex pattern matching
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2195_placeholder() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
}
