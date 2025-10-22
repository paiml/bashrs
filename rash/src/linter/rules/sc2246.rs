// SC2246: Word is of the form "A B C" - quote or use array (placeholder - complex pattern)
#[allow(unused_imports)]
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    // Placeholder: Complex rule requiring semantic analysis of strings vs arrays
    // Would need to detect multi-word strings that should be arrays
    // Implementation deferred pending type inference
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2246_placeholder() {
        let code = r#"list="one two three""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2246_array_ok_placeholder() {
        let code = "list=(one two three)";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2246_comment_placeholder() {
        let code = "# comment";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2246_no_code_placeholder() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2246_simple_placeholder() {
        let code = "echo test";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2246_single_word_placeholder() {
        let code = r#"var="word""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2246_quoted_phrase_placeholder() {
        let code = r#"msg="Hello World""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2246_command_list_placeholder() {
        let code = r#"commands="ls grep awk""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - should be array
    }
    #[test]
    fn test_sc2246_for_loop_placeholder() {
        let code = r#"for x in "a b c"; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2246_assignment_placeholder() {
        let code = "files=\"*.txt *.log\"";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
}
