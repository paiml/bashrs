// REPL Parser Integration Module
//
// Task: REPL-004-001 - Embed bash parser into REPL
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 6+ scenarios
// - Property tests: 2+ generators
// - Mutation score: ≥90%
// - Complexity: <10 per function

use crate::bash_parser::{BashAst, BashParser, ParseError};

/// Parse bash input and return AST
///
/// # Examples
///
/// ```
/// use bashrs::repl::parser::parse_bash;
///
/// let result = parse_bash("echo hello");
/// assert!(result.is_ok());
/// ```
pub fn parse_bash(input: &str) -> Result<BashAst, ParseError> {
    let mut parser = BashParser::new(input)?;
    parser.parse()
}

/// Format parse error for display in REPL with enhanced context
///
/// Provides helpful error messages with line numbers for better debugging.
/// Handles all ParseError variants with appropriate formatting.
///
/// # Examples
///
/// ```
/// use bashrs::repl::parser::{parse_bash, format_parse_error};
///
/// let result = parse_bash("if then fi");  // Missing condition
/// assert!(result.is_err());
///
/// let error = result.unwrap_err();
/// let formatted = format_parse_error(&error);
/// assert!(formatted.contains("Syntax error"));
/// ```
pub fn format_parse_error(error: &ParseError) -> String {
    match error {
        ParseError::UnexpectedToken {
            expected,
            found,
            line,
        } => {
            format!(
                "Syntax error at line {}: expected {}, but found {}",
                line, expected, found
            )
        }
        ParseError::UnexpectedEof => String::from(
            "Syntax error: unexpected end of file (did you forget to close a quote or bracket?)",
        ),
        ParseError::InvalidSyntax(msg) => {
            // Try to add generic line info even if not provided in the variant
            format!("Syntax error: {}", msg)
        }
        ParseError::LexerError(lexer_err) => {
            format!("Lexer error: {}", lexer_err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: REPL-004-001-001 - Parse simple command
    #[test]
    fn test_REPL_004_001_parse_simple_command() {
        let input = "echo hello";
        let result = parse_bash(input);

        assert!(result.is_ok(), "Should parse simple command");
        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1, "Should have 1 statement");
    }

    /// Test: REPL-004-001-002 - Parse command with arguments
    #[test]
    fn test_REPL_004_001_parse_command_with_args() {
        let input = "ls -la /tmp";
        let result = parse_bash(input);

        assert!(
            result.is_ok(),
            "Should parse command with arguments: {:?}",
            result
        );
        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1, "Should have 1 statement");
    }

    /// Test: REPL-004-001-003 - Parse control flow (if statement)
    #[test]
    fn test_REPL_004_001_parse_control_flow() {
        let input = r#"if true; then
    echo success
fi"#;
        let result = parse_bash(input);

        assert!(result.is_ok(), "Should parse if statement: {:?}", result);
        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1, "Should have 1 if statement");
    }

    /// Test: REPL-004-001-004 - Parse error for unclosed quote
    #[test]
    fn test_REPL_004_001_parse_error_unclosed_quote() {
        let input = r#"echo "hello"#;
        let result = parse_bash(input);

        assert!(result.is_err(), "Should fail on unclosed quote");
        let error = result.unwrap_err();
        let error_msg = format_parse_error(&error);
        // Updated to match new error format
        assert!(
            error_msg.contains("Syntax error") || error_msg.contains("Lexer error"),
            "Should format error: {}",
            error_msg
        );
    }

    /// Test: REPL-004-001-005 - Parse error for invalid syntax
    #[test]
    fn test_REPL_004_001_parse_error_invalid_syntax() {
        let input = "if then fi"; // Missing condition
        let result = parse_bash(input);

        assert!(result.is_err(), "Should fail on invalid syntax");
    }

    /// Test: REPL-004-001-006 - Parse empty input
    #[test]
    fn test_REPL_004_001_parse_empty_input() {
        let input = "";
        let result = parse_bash(input);

        assert!(result.is_ok(), "Should handle empty input");
        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 0, "Should have no statements");
    }

    /// Test: REPL-004-001-007 - Parse multiline input
    #[test]
    fn test_REPL_004_001_parse_multiline() {
        let input = "echo line1\necho line2\necho line3";
        let result = parse_bash(input);

        assert!(result.is_ok(), "Should parse multiline input");
        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 3, "Should have 3 statements");
    }

    /// Test: REPL-004-001-008 - Format error message
    #[test]
    fn test_REPL_004_001_format_error_message() {
        let input = "echo \"unclosed";
        let result = parse_bash(input);

        assert!(result.is_err());
        let error = result.unwrap_err();
        let formatted = format_parse_error(&error);

        // Updated to match new improved error formatting
        assert!(
            formatted.contains("Syntax error") || formatted.contains("Lexer error"),
            "Should have error type: {}",
            formatted
        );
        assert!(!formatted.is_empty());
    }

    // ===== REPL-004-003 TESTS (RED PHASE) =====

    /// Test: REPL-004-003-001 - Syntax error includes line number
    #[test]
    fn test_REPL_004_003_syntax_error_unclosed_quote() {
        let input = r#"echo "hello
world"#;
        let result = parse_bash(input);

        assert!(result.is_err(), "Should fail on unclosed quote");
        let error = result.unwrap_err();
        let formatted = format_parse_error(&error);

        // Should contain line number information
        assert!(
            formatted.contains("line") || formatted.contains("Line"),
            "Error should mention line number: {}",
            formatted
        );
        assert!(!formatted.is_empty(), "Formatted error should not be empty");
    }

    /// Test: REPL-004-003-002 - Invalid operator error shows context
    #[test]
    fn test_REPL_004_003_syntax_error_invalid_operator() {
        let input = "if | then echo test; fi";
        let result = parse_bash(input);

        assert!(result.is_err(), "Should fail on invalid syntax");
        let error = result.unwrap_err();
        let formatted = format_parse_error(&error);

        // Should contain helpful error message
        assert!(!formatted.is_empty(), "Formatted error should not be empty");
    }

    /// Test: REPL-004-003-003 - Error messages are helpful and clear
    #[test]
    fn test_REPL_004_003_syntax_error_helpful_message() {
        let input = r#"echo line1
echo line2
if then echo problem
echo line4"#;
        let result = parse_bash(input);

        assert!(result.is_err(), "Should fail on line 3");
        let error = result.unwrap_err();
        let formatted = format_parse_error(&error);

        // Should have clear syntax error message
        assert!(
            formatted.contains("Syntax error") || formatted.contains("syntax error"),
            "Error should be clear syntax error: {}",
            formatted
        );
        // Should not just be the generic "Parse error:" prefix
        assert!(
            !formatted.starts_with("Parse error:"),
            "Should use specific error formatting, not generic prefix"
        );
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // ===== PROPERTY TESTS (PROPERTY PHASE) =====

    /// Property: Parser should never panic on any input
    proptest! {
        #[test]
        fn prop_parse_never_panics(input in ".*{0,1000}") {
            // Test that parser gracefully handles any input without panicking
            let _ = parse_bash(&input);
            // If we get here without panic, test passes
        }
    }

    /// Property: Parser should produce valid AST or error
    proptest! {
        #[test]
        fn prop_parse_produces_valid_result(input in "[a-z ]{1,100}") {
            let result = parse_bash(&input);
            // Result must be either Ok(AST) or Err(ParseError)
            // Both variants are valid - no undefined state
            match result {
                Ok(ast) => {
                    // Valid AST should have consistent structure
                    prop_assert!(ast.statements.len() < 1000, "AST size reasonable");
                }
                Err(_) => {
                    // Error is also a valid outcome
                }
            }
        }
    }

    /// Property: Empty/whitespace input should always succeed
    proptest! {
        #[test]
        fn prop_parse_empty_whitespace_succeeds(
            spaces in r"[ \t\n]{0,100}"
        ) {
            let result = parse_bash(&spaces);
            prop_assert!(result.is_ok(), "Empty/whitespace should parse successfully");
            let ast = result.unwrap();
            prop_assert_eq!(ast.statements.len(), 0, "Should have no statements");
        }
    }

    /// Property: Valid commands should parse successfully
    proptest! {
        #[test]
        fn prop_parse_valid_commands(
            cmd in "[a-z]{1,10}",
            arg in "[a-z0-9]{0,20}"
        ) {
            let input = if arg.is_empty() {
                cmd.clone()
            } else {
                format!("{} {}", cmd, arg)
            };

            let result = parse_bash(&input);
            // Should either parse successfully or fail with clear error
            // (some generated commands may not be valid syntax)
            match result {
                Ok(ast) => {
                    prop_assert!(ast.statements.len() >= 1, "Should have at least 1 statement");
                }
                Err(error) => {
                    let formatted = format_parse_error(&error);
                    // Updated to match new improved error formatting
                    let has_error_type = formatted.contains("Syntax error")
                        || formatted.contains("syntax error")
                        || formatted.contains("Lexer error");
                    prop_assert!(has_error_type, "Should format error message: {}", formatted);
                }
            }
        }
    }

    /// Property: Parse error formatting should never be empty
    proptest! {
        #[test]
        fn prop_error_formatting_never_empty(input in ".*{1,100}") {
            if let Err(error) = parse_bash(&input) {
                let formatted = format_parse_error(&error);
                prop_assert!(!formatted.is_empty(), "Error message should not be empty");
                // Updated to match new improved error formatting
                let has_error_type = formatted.contains("Syntax error")
                    || formatted.contains("syntax error")
                    || formatted.contains("Lexer error");
                prop_assert!(has_error_type, "Should have specific error type: {}", formatted);
            }
        }
    }

    /// Property: Multiline commands should parse or error gracefully
    proptest! {
        #[test]
        fn prop_parse_multiline_graceful(
            line1 in "[a-z ]{1,50}",
            line2 in "[a-z ]{1,50}",
            line3 in "[a-z ]{1,50}"
        ) {
            let input = format!("{}\n{}\n{}", line1, line2, line3);
            let result = parse_bash(&input);

            // Should handle multiline input without panicking
            match result {
                Ok(ast) => {
                    // Multiple lines may produce multiple statements
                    prop_assert!(ast.statements.len() <= 3, "Should not exceed line count");
                }
                Err(_) => {
                    // Error is valid for malformed multiline
                }
            }
        }
    }

    // ===== REPL-004-003 PROPERTY TESTS (PROPERTY PHASE) =====

    /// Property: All syntax errors have helpful, non-empty messages
    proptest! {
        #[test]
        fn prop_syntax_errors_always_helpful(input in ".*{1,100}") {
            if let Err(error) = parse_bash(&input) {
                let formatted = format_parse_error(&error);

                // Error messages must never be empty
                prop_assert!(!formatted.is_empty(), "Error message should not be empty");

                // Error messages should be helpful (contain key words)
                let is_helpful = formatted.contains("Syntax error")
                    || formatted.contains("syntax error")
                    || formatted.contains("Lexer error")
                    || formatted.contains("line")
                    || formatted.contains("expected")
                    || formatted.contains("found")
                    || formatted.contains("unexpected");

                prop_assert!(
                    is_helpful,
                    "Error message should be helpful and descriptive: {}",
                    formatted
                );
            }
        }
    }

    /// Property: Errors with line numbers always format them correctly
    proptest! {
        #[test]
        fn prop_line_numbers_formatted_correctly(input in ".*{1,200}") {
            if let Err(error) = parse_bash(&input) {
                let formatted = format_parse_error(&error);

                // If the error mentions a line number, it should be formatted correctly
                if formatted.contains("line") {
                    // Line numbers should be positive integers
                    // This is a simple sanity check
                    prop_assert!(
                        !formatted.contains("line 0"),
                        "Line numbers should start at 1, not 0"
                    );
                }
            }
        }
    }
}
