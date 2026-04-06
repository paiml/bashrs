//! Tests extracted from codegen.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::bash_parser::codegen::*;
use crate::bash_parser::BashParser;

// ============================================================================
// Statement Generation Tests
// ============================================================================

#[test]
fn test_convert_c_increment_prefix_increment() {
    let output = convert_c_increment_to_posix("++i");
    assert_eq!(output, "i=$((i+1))");
}

#[test]
fn test_convert_c_increment_postfix_decrement() {
    let output = convert_c_increment_to_posix("i--");
    assert_eq!(output, "i=$((i-1))");
}

#[test]
fn test_convert_c_increment_prefix_decrement() {
    let output = convert_c_increment_to_posix("--i");
    assert_eq!(output, "i=$((i-1))");
}

#[test]
fn test_convert_c_increment_plus_equals() {
    let output = convert_c_increment_to_posix("i+=2");
    assert_eq!(output, "i=$((i+2))");
}

#[test]
fn test_convert_c_increment_minus_equals() {
    let output = convert_c_increment_to_posix("i-=3");
    assert_eq!(output, "i=$((i-3))");
}

#[test]
fn test_convert_c_increment_assignment() {
    let output = convert_c_increment_to_posix("i=i+1");
    assert_eq!(output, "i=i+1");
}

#[test]
fn test_convert_c_increment_fallback() {
    let output = convert_c_increment_to_posix("something_else");
    assert_eq!(output, ":something_else");
}

// ============================================================================
// extract_var_name Coverage
// ============================================================================

#[test]
fn test_extract_var_name_with_dollar() {
    assert_eq!(extract_var_name("$i"), "i");
    assert_eq!(extract_var_name("$var"), "var");
}

#[test]
fn test_extract_var_name_without_dollar() {
    assert_eq!(extract_var_name("i"), "i");
    assert_eq!(extract_var_name("count"), "count");
}

// ============================================================================
// strip_quotes Coverage
// ============================================================================

#[test]
fn test_strip_quotes_double() {
    assert_eq!(strip_quotes("\"value\""), "value");
}

#[test]
fn test_strip_quotes_single() {
    assert_eq!(strip_quotes("'value'"), "value");
}

#[test]
fn test_strip_quotes_mixed() {
    assert_eq!(strip_quotes("\"value'"), "value");
}

#[test]
fn test_strip_quotes_none() {
    assert_eq!(strip_quotes("value"), "value");
}

// ============================================================================
// generate_condition Coverage
// ============================================================================

#[test]
fn test_generate_condition_test() {
    let expr = BashExpr::Test(Box::new(TestExpr::FileExists(BashExpr::Literal(
        "f".to_string(),
    ))));
    let output = generate_condition(&expr);
    assert!(output.contains("-e"));
}

#[test]
fn test_generate_condition_non_test() {
    let expr = BashExpr::Literal("true".to_string());
    let output = generate_condition(&expr);
    assert_eq!(output, "true");
}

// ============================================================================
// Comment shebang filtering
// ============================================================================

#[test]
fn test_generate_comment_shebang_filtered() {
    let stmt = BashStmt::Comment {
        text: "!/bin/bash".to_string(),
        span: Span::dummy(),
    };
    let output = generate_statement(&stmt);
    assert_eq!(output, "");
}

#[test]
fn test_generate_comment_shebang_with_space_filtered() {
    let stmt = BashStmt::Comment {
        text: " !/bin/sh".to_string(),
        span: Span::dummy(),
    };
    let output = generate_statement(&stmt);
    assert_eq!(output, "");
}

#[test]
fn test_generate_comment_normal() {
    let stmt = BashStmt::Comment {
        text: "This is a normal comment".to_string(),
        span: Span::dummy(),
    };
    let output = generate_statement(&stmt);
    assert_eq!(output, "# This is a normal comment");
}

#[cfg(test)]
mod test_issue_64 {
use crate::bash_parser::codegen::generate_purified_bash;
use crate::bash_parser::BashParser;

#[test]
fn test_ISSUE_64_single_quoted_ansi_codes() {
    // RED phase: Test single-quoted ANSI escape sequences
    let input = r#"RED='\033[0;31m'"#;
    let mut parser = BashParser::new(input).expect("Failed to parse");
    let ast = parser.parse().expect("Failed to parse");
    let output = generate_purified_bash(&ast);

    // Single quotes should be preserved for escape sequences
    assert!(
        output.contains("RED='\\033[0;31m'"),
        "Output should preserve single quotes around escape sequences: {}",
        output
    );
}

#[test]
fn test_ISSUE_64_single_quoted_literal() {
    let input = "echo 'Hello World'";
    let mut parser = BashParser::new(input).expect("Failed to parse");
    let ast = parser.parse().expect("Failed to parse");
    let output = generate_purified_bash(&ast);

    // Single quotes should be preserved
    assert!(
        output.contains("'Hello World'"),
        "Output should preserve single quotes: {}",
        output
    );
}

#[test]
fn test_ISSUE_64_assignment_with_single_quotes() {
    let input = "x='value'";
    let mut parser = BashParser::new(input).expect("Failed to parse");
    let ast = parser.parse().expect("Failed to parse");
    let output = generate_purified_bash(&ast);

    // For simple alphanumeric strings, quotes are optional in purified output
    // Both x=value and x='value' are correct POSIX shell
    // The important thing is it parses without error
    assert!(
        output.contains("x=value") || output.contains("x='value'"),
        "Output should contain valid assignment: {}",
        output
    );
}
} // mod test_issue_64

#[test]
fn test_ELIF_001_basic_elif_preserved() {
    let input = r#"if [ "$1" = "a" ]; then
echo alpha
elif [ "$1" = "b" ]; then
echo beta
else
echo unknown
fi"#;
    let mut parser = BashParser::new(input).expect("parser");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(
        output.contains("elif"),
        "elif should be preserved in output: {output}"
    );
    assert!(
        output.contains("echo alpha"),
        "then branch preserved: {output}"
    );
    assert!(
        output.contains("echo beta"),
        "elif branch preserved: {output}"
    );
    assert!(
        output.contains("echo unknown"),
        "else branch preserved: {output}"
    );
}

#[test]
fn test_ELIF_002_multiple_elif_preserved() {
    let input = r#"if [ "$1" = "a" ]; then
echo alpha
elif [ "$1" = "b" ]; then
echo beta
elif [ "$1" = "c" ]; then
echo gamma
else
echo unknown
fi"#;
    let mut parser = BashParser::new(input).expect("parser");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    let elif_count = output.matches("elif").count();
    assert_eq!(
        elif_count, 2,
        "should have 2 elif branches, got {elif_count}: {output}"
    );
}

#[test]
fn test_ELIF_003_elif_no_else() {
    let input = r#"if [ "$1" = "a" ]; then
echo alpha
elif [ "$1" = "b" ]; then
echo beta
fi"#;
    let mut parser = BashParser::new(input).expect("parser");
    let ast = parser.parse().expect("parse");
    let output = generate_purified_bash(&ast);
    assert!(output.contains("elif"), "elif preserved: {output}");
    assert!(!output.contains("else"), "no else block: {output}");
}
