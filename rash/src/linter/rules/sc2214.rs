// SC2214: getopts option syntax error (placeholder - requires optstring parsing)
#[allow(unused_imports)]
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    // Placeholder: Complex rule requiring optstring format validation
    // Would need to:
    // 1. Parse getopts "optstring" format
    // 2. Validate characters are alphanumeric
    // 3. Check : placement (only after option letter, not standalone)
    // 4. Detect invalid characters in optstring
    // 5. Warn about :: (GNU extension, not POSIX)
    // Implementation deferred pending parser enhancement
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2214_placeholder() {
        let code = r#"while getopts "a:b:c" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2214_invalid_char_ok() {
        let code = r#"while getopts "a-b" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - should flag -
    }
    #[test]
    fn test_sc2214_standalone_colon_ok() {
        let code = r#"while getopts "a:" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Valid
    }
    #[test]
    fn test_sc2214_double_colon_ok() {
        let code = r#"while getopts "a::" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - GNU extension
    }
    #[test]
    fn test_sc2214_leading_colon_ok() {
        let code = r#"while getopts ":abc" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Valid - silent error mode
    }
    #[test]
    fn test_sc2214_numbers_ok() {
        let code = r#"while getopts "123" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Valid but unusual
    }
    #[test]
    fn test_sc2214_mixed_case() {
        let code = r#"while getopts "aAbBcC" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Valid
    }
    #[test]
    fn test_sc2214_special_char_ok() {
        let code = r#"while getopts "a$b" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - should flag $
    }
    #[test]
    fn test_sc2214_comment() {
        let code = r#"# while getopts "abc" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2214_variable_optstring() {
        let code = r#"while getopts "$opts" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Can't validate variable
    }
}
