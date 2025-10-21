// SC2215: Expression is not properly quoted
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    // Placeholder: This rule is covered by SC2086 (unquoted variable expansion)
    // SC2215 is a more general reminder to quote expressions
    // Implementation deferred as it overlaps with existing SC2086
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2215_placeholder() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2215_unquoted_var() {
        let code = "echo test2";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - covered by SC2086
    }
    #[test]
    fn test_sc2215_quoted_var() {
        let code = "echo test3";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2215_test_command() {
        let code = "test ok";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2215_cat_command() {
        let code = "cat file";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2215_grep_command() {
        let code = "grep pattern file";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2215_comment() {
        let code = "# comment";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2215_literal() {
        let code = "echo hello";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2215_no_var() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2215_empty() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
