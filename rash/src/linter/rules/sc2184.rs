// SC2184: Quote arguments to unset (simpler than SC2149)
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Covered by SC2149
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2184_placeholder() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
}
