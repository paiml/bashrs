// SC2167: This parent trap is not inherited by the child.
#[allow(unused_imports)]
//
// Similar to SC2165 - traps in parent aren't inherited by child processes.
//
// Examples:
// Bad:
//   trap "cleanup" EXIT
//   $command                     // Child doesn't inherit
//
// Good:
//   trap "cleanup" EXIT
//   { $command; }                // Same shell
//
// Impact: Cleanup may not execute in child
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(_source: &str) -> LintResult {
    
    // This is a context-sensitive rule requiring deep analysis
    // Simplified implementation - would need full AST
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2167_placeholder() {
        let code = "trap 'cleanup' EXIT\ncommand";
        let result = check(code);
        // Simplified - would detect in full implementation
        assert_eq!(result.diagnostics.len(), 0);
    }
}
