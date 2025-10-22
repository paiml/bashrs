// SC2189: Pipe before heredoc terminator
#[allow(unused_imports)]
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(_source: &str) -> LintResult {
    LintResult::new() // Complex heredoc parsing needed
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2189_placeholder() {
        assert_eq!(check("").diagnostics.len(), 0);
    }
}
