#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_SEC019_001_unquoted_variable_detected() {
        let script = "echo $user_input";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC019");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.contains("injection risk"));
        assert!(diag.message.contains("user_input"));
    }

    #[test]
    fn test_SEC019_002_quoted_variable_safe() {
        let script = r#"echo "$user_input""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Quoted variables are safe");
    }

    #[test]
    fn test_SEC019_003_single_quoted_safe() {
        let script = "echo '$user_input'";
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Single quotes prevent expansion"
        );
    }

    #[test]
    fn test_SEC019_004_brace_expansion_unquoted() {
        let script = "echo ${user_input}";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.message.contains("user_input"));
    }

    #[test]
    fn test_SEC019_005_brace_expansion_quoted() {
        let script = r#"echo "${user_input}""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_SEC019_006_multiple_unquoted_variables() {
        let script = "echo $var1 $var2 $var3";
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            3,
            "Should detect all 3 unquoted variables"
        );
    }

    #[test]
    fn test_SEC019_007_special_variables_ignored() {
        let script = "echo $? $# $$ $@ $*";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Special variables are safe");
    }

    #[test]
    fn test_SEC019_008_arithmetic_expansion_safe() {
        let script = "result=$((x + y))";
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Arithmetic expansions are safe"
        );
    }

    #[test]
    fn test_SEC019_009_test_context_safe() {
        let script = "[[ $var == value ]]";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0, "Variables in [[ ]] are safe");
    }

    #[test]
    fn test_SEC019_010_command_in_dangerous_context() {
        let script = "rm -rf $directory";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert!(diag.message.contains("directory"));
        assert_eq!(diag.severity, Severity::Warning);
    }
}
