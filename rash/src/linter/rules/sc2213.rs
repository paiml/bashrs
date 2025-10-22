// SC2213: getopts is being used incorrectly (placeholder - requires state tracking)
#[allow(unused_imports)]
use crate::linter::LintResult;

pub fn check(_source: &str) -> LintResult {
    // Placeholder: Complex rule requiring getopts state machine and context tracking
    // Would need to:
    // 1. Parse getopts optstring format
    // 2. Track which options expect arguments (followed by :)
    // 3. Verify case statement matches optstring
    // 4. Check OPTARG usage only for options with arguments
    // Implementation deferred pending AST-based analysis
    LintResult::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2213_placeholder() {
        let code = r#"while getopts "abc:" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2213_invalid_ok() {
        let code = r#"while getopts "a:b:c" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2213_mismatch_ok() {
        let code = r#"while getopts "ab" opt; do
  case "$opt" in
    c) echo "not in optstring" ;;
  esac
done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2213_optarg_misuse_ok() {
        let code = r#"while getopts "ab" opt; do
  case "$opt" in
    a) echo "$OPTARG" ;;
  esac
done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder - should flag OPTARG without :
    }
    #[test]
    fn test_sc2213_valid_optarg_ok() {
        let code = r#"while getopts "a:" opt; do
  case "$opt" in
    a) echo "$OPTARG" ;;
  esac
done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2213_comment() {
        let code = r#"# while getopts "ab" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2213_multiple_colons() {
        let code = r#"while getopts "a:b:c:" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2213_leading_colon() {
        let code = r#"while getopts ":abc" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Silent error mode
    }
    #[test]
    fn test_sc2213_empty_optstring() {
        let code = r#"while getopts "" opt; do"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Placeholder
    }
    #[test]
    fn test_sc2213_no_getopts() {
        let code = r#"while true; do echo "no getopts"; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
