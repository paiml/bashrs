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

use crate::bash_parser::{BashParser, ParseError, BashAst};

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

/// Format parse error for display in REPL
pub fn format_parse_error(error: &ParseError) -> String {
    format!("Parse error: {}", error)
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

        assert!(result.is_ok(), "Should parse command with arguments: {:?}", result);
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
        assert!(error_msg.contains("Parse error"), "Should format error");
    }

    /// Test: REPL-004-001-005 - Parse error for invalid syntax
    #[test]
    fn test_REPL_004_001_parse_error_invalid_syntax() {
        let input = "if then fi";  // Missing condition
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
        
        assert!(formatted.starts_with("Parse error:"));
        assert!(!formatted.is_empty());
    }
}
