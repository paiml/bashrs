// SC2193: Literal space in glob
#[allow(unused_imports)]
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Pattern analysis needed
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2193_placeholder() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
}
