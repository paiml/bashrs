// SC2177: 'time' only measures the first command in the pipeline.
#[allow(unused_imports)]
//
// Related to SC2176 - clarifies that time only measures first command.
//
// Examples:
// Bad:
//   time cmd1 | cmd2             // Only times cmd1
//
// Good:
//   time { cmd1 | cmd2; }        // Times entire pipeline
//
// Impact: Misleading timing measurements
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(_source: &str) -> LintResult {
    let result = LintResult::new();
    // This is covered by SC2176 - same pattern
    // Placeholder for compatibility
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2177_placeholder() {
        let code = "time cmd1 | cmd2";
        let result = check(code);
        // Covered by SC2176
        assert_eq!(result.diagnostics.len(), 0);
    }
}
