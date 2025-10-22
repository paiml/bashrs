// SC2221: This ;; will cause the next case to fall through (placeholder - requires AST)
#[allow(unused_imports)]
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    // Placeholder: Complex rule requiring case statement AST parsing
    // Would need to:
    // 1. Parse case statement structure
    // 2. Identify case terminators (;; vs ;;& vs ;;&)
    // 3. Detect unintended fallthrough patterns
    // 4. Track control flow between cases
    // Implementation deferred pending AST-based case parser
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2221_placeholder() {
        let code = r#"case $x in a) echo 1;; esac"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2221_fallthrough_placeholder() {
        let code = r#"case $x in a) echo 1;;& esac"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2221_multiple_cases_placeholder() {
        let code = r#"case $x in a) echo 1;; b) echo 2;; esac"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2221_comment_placeholder() {
        let code = "# case";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2221_no_case_placeholder() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2221_empty_placeholder() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2221_nested_case_placeholder() {
        let code = "case $a in x) case $b in y) echo z;; esac;; esac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2221_no_terminator_placeholder() {
        let code = "case $x in a) echo 1 esac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - syntax error
    }
    #[test]
    fn test_sc2221_pattern_placeholder() {
        let code = "case $x in [a-z]) echo 1;; esac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2221_multiline_placeholder() {
        let code = "case $x in\n  a) echo 1;;\n  b) echo 2;;\nesac";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
}
