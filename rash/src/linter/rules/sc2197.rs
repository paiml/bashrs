// SC2197: Glob doesn't match
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Runtime behavior, can't detect statically
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2197_placeholder() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
}
