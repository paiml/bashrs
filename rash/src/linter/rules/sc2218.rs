// SC2218: Prefer [[ ]] instead of [ ] (placeholder - style recommendation)
#[allow(unused_imports)]
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    // Placeholder: Style recommendation to prefer [[ ]] over [ ]
    // Would need to:
    // 1. Detect [ ] usage (not file test flags)
    // 2. Recommend [[ ]] for bash scripts (check shebang)
    // 3. Explain benefits: no word splitting, pattern matching, etc.
    // 4. Avoid flagging POSIX sh scripts where [[ ]] not available
    // Implementation deferred pending style preference configuration
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2218_placeholder() {
        let code = r#"[ "$x" = "y" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2218_double_bracket_ok() {
        let code = r#"[[ "$x" = "y" ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2218_file_test() {
        let code = r#"[ -f "$file" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2218_numeric_test() {
        let code = r#"[ "$count" -gt 5 ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2218_string_test() {
        let code = r#"[ "$status" = "ok" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2218_posix_sh() {
        let code = r#"#!/bin/sh
[ "$x" = "y" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // POSIX sh can't use [[]]
    }
    #[test]
    fn test_sc2218_bash() {
        let code = r#"#!/bin/bash
[ "$x" = "y" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - could recommend [[]]
    }
    #[test]
    fn test_sc2218_empty_test() {
        let code = r#"[ -z "$var" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2218_comment() {
        let code = r#"# [ "$x" = "y" ]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2218_no_test() {
        let code = r#"echo "no test here""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
