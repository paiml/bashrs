// SC2175: Quote this to prevent word splitting.
#[allow(unused_imports)]
//
// Unquoted variables undergo word splitting and globbing.
//
// Examples:
// Bad:
//   for arg in $@; do            // Unquoted $@
//   echo $var                     // Unquoted variable
//
// Good:
//   for arg in "$@"; do           // Quoted
//   echo "$var"                   // Quoted
//
// Impact: Word splitting, unexpected behavior
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(_source: &str) -> LintResult {
    
    // This overlaps with SC2068, SC2086 - already implemented
    // Placeholder for compatibility
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2175_placeholder() {
        let code = "echo $var";
        let result = check(code);
        // Covered by other rules
        assert_eq!(result.diagnostics.len(), 0);
    }
}
