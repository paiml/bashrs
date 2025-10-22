// SC2241: Exit status can only be 0-255 (placeholder - overlaps SC2151/SC2152)
#[allow(unused_imports)]
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    // Placeholder: This rule is covered by SC2151 (return) and SC2152 (exit)
    // Exit status validation is already handled by existing rules
    // Implementation deferred as it duplicates existing functionality
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2241_placeholder() {
        let code = "exit 0";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - covered by SC2152
    }
    #[test]
    fn test_sc2241_return_placeholder() {
        let code = "return 42";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - covered by SC2151
    }
    #[test]
    fn test_sc2241_comment_placeholder() {
        let code = "# comment";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2241_no_code_placeholder() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2241_simple_placeholder() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2241_large_exit_placeholder() {
        let code = "exit 256";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - covered by SC2152
    }
    #[test]
    fn test_sc2241_negative_placeholder() {
        let code = "return -1";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - covered by SC2151
    }
    #[test]
    fn test_sc2241_variable_placeholder() {
        let code = "exit $status";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2241_success_placeholder() {
        let code = "exit 0";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2241_failure_placeholder() {
        let code = "exit 1";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
}
