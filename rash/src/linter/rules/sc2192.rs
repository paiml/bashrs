// SC2192: Array is empty
#[allow(unused_imports)]
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Requires state tracking
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2192_placeholder() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
}
