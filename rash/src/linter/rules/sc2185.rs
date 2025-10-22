// SC2185: Some problems with loop
#[allow(unused_imports)]
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Context-sensitive, requires AST
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2185_placeholder() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
}
