#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pipeline() -> ValidationPipeline {
        ValidationPipeline {
            level: ValidationLevel::Strict,
            strict_mode: true,
        }
    }

    // Issue #94: Table formatting strings should not be flagged
    #[test]
    fn test_issue_94_table_formatting_ok() {
        let pipeline = create_test_pipeline();
        // Table formatting with multiple pipes is NOT a command injection
        let result =
            pipeline.validate_string_literal("|     whisper.apr      |     whisper.cpp      |");
        assert!(
            result.is_ok(),
            "Table formatting with pipes should NOT be flagged: {:?}",
            result
        );
    }

    #[test]
    fn test_issue_94_pipe_border_ok() {
        let pipeline = create_test_pipeline();
        // Table borders are OK
        let result = pipeline.validate_string_literal("| col1 | col2 | col3 |");
        assert!(result.is_ok(), "Table border should not be flagged");
    }

    #[test]
    fn test_issue_94_single_pipe_border_ok() {
        let pipeline = create_test_pipeline();
        // Single pipe at start or end is OK (table border)
        let result = pipeline.validate_string_literal("| some content");
        assert!(result.is_ok(), "Leading pipe should not be flagged");
    }

    #[test]
    fn test_issue_94_pipe_to_command_flagged() {
        let pipeline = create_test_pipeline();
        // Actual command pipe should still be flagged
        let result = pipeline.validate_string_literal("cat file | rm -rf /");
        assert!(result.is_err(), "Actual command pipe should be flagged");
    }

    #[test]
    fn test_issue_94_semicolon_in_quoted_string_allowed() {
        let pipeline = create_test_pipeline();
        // Bare semicolons inside double-quoted strings are safe (echo "a; b" is not injection)
        let result = pipeline.validate_string_literal("cmd1; cmd2");
        assert!(
            result.is_ok(),
            "Bare semicolons in double-quoted strings are not injection: {:?}",
            result,
        );
    }

    #[test]
    fn test_issue_94_quote_escape_semicolon_flagged() {
        let pipeline = create_test_pipeline();
        // Quote-escape + semicolon IS injection (breaks out of quotes)
        let result = pipeline.validate_string_literal("value'; rm -rf /");
        assert!(result.is_err(), "Quote-escape semicolon should be flagged");
    }

    #[test]
    fn test_issue_94_command_substitution_still_flagged() {
        let pipeline = create_test_pipeline();
        // Command substitution should still be flagged
        let result = pipeline.validate_string_literal("$(dangerous_command)");
        assert!(result.is_err(), "Command substitution should be flagged");
    }

    // Issue #95: exec() arguments should allow shell operators
    #[test]
    fn test_issue_95_exec_with_pipe_allowed() {
        let pipeline = create_test_pipeline();
        // Pipes in exec() are expected - this is the whole point of exec()
        let result = pipeline.validate_string_literal_in_exec("ldd /usr/bin/foo | grep cuda");
        assert!(
            result.is_ok(),
            "Pipe in exec() should be allowed: {:?}",
            result
        );
    }

    #[test]
    fn test_issue_95_exec_with_and_allowed() {
        let pipeline = create_test_pipeline();
        // AND operator in exec() is expected
        let result = pipeline.validate_string_literal_in_exec("cmd1 && cmd2");
        assert!(
            result.is_ok(),
            "AND operator in exec() should be allowed: {:?}",
            result
        );
    }

    #[test]
    fn test_issue_95_exec_with_or_allowed() {
        let pipeline = create_test_pipeline();
        // OR operator in exec() is expected
        let result = pipeline.validate_string_literal_in_exec("cmd1 || cmd2");
        assert!(
            result.is_ok(),
            "OR operator in exec() should be allowed: {:?}",
            result
        );
    }

    #[test]
    fn test_issue_95_exec_with_semicolon_allowed() {
        let pipeline = create_test_pipeline();
        // Semicolon in exec() is expected
        let result = pipeline.validate_string_literal_in_exec("cmd1; cmd2");
        assert!(
            result.is_ok(),
            "Semicolon in exec() should be allowed: {:?}",
            result
        );
    }

    #[test]
    fn test_issue_95_exec_shellshock_still_blocked() {
        let pipeline = create_test_pipeline();
        // Shellshock attacks should STILL be blocked even in exec()
        let result = pipeline.validate_string_literal_in_exec("() { :; }; echo pwned");
        assert!(
            result.is_err(),
            "Shellshock in exec() should still be blocked"
        );
    }

    #[test]
    fn test_issue_95_exec_command_substitution_blocked() {
        let pipeline = create_test_pipeline();
        // Command substitution in exec() is blocked (potential injection vector)
        let result = pipeline.validate_string_literal_in_exec("echo $(whoami)");
        assert!(
            result.is_err(),
            "Command substitution in exec() should be blocked"
        );
    }

    #[test]
    fn test_issue_95_non_exec_pipe_still_flagged() {
        let pipeline = create_test_pipeline();
        // Non-exec strings with pipes should still be flagged
        let result = pipeline.validate_string_literal("cat file | rm -rf /");
        assert!(result.is_err(), "Pipe in non-exec string should be flagged");
    }

    #[test]
    fn test_issue_95_complex_exec_command() {
        let pipeline = create_test_pipeline();
        // Complex shell commands should work in exec()
        let result = pipeline.validate_string_literal_in_exec(
            "ldd /usr/local/bin/main 2>/dev/null | grep -i blas | head -1",
        );
        assert!(
            result.is_ok(),
            "Complex pipeline in exec() should be allowed: {:?}",
            result
        );
    }
}
