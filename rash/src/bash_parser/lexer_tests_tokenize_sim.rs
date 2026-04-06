
use super::*;

#[test]
fn test_tokenize_simple_assignment() {
    let mut lexer = Lexer::new("FOO=bar");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0], Token::Identifier("FOO".to_string()));
    assert_eq!(tokens[1], Token::Assign);
    assert_eq!(tokens[2], Token::Identifier("bar".to_string()));
}

#[test]
fn test_tokenize_if_statement() {
    let mut lexer = Lexer::new("if [ $x == 1 ]; then");
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0], Token::If);
    assert_eq!(tokens[1], Token::LeftBracket);
    assert!(matches!(tokens[2], Token::Variable(_)));
}

#[test]
fn test_tokenize_string() {
    let mut lexer = Lexer::new(r#""hello world""#);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0], Token::String("hello world".to_string()));
}

#[test]
fn test_tokenize_comment() {
    let mut lexer = Lexer::new("# This is a comment");
    let tokens = lexer.tokenize().unwrap();

    assert!(matches!(tokens[0], Token::Comment(_)));
}

// EXTREME TDD - RED Phase: Test for date +FORMAT support
// This test is EXPECTED TO FAIL until lexer enhancement is implemented
#[test]
fn test_lexer_plus_in_command_args() {
    let input = "date +%s";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    // Expected tokens: [Identifier("date"), Identifier("+%s"), Eof]
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0], Token::Identifier("date".to_string()));
    assert_eq!(tokens[1], Token::Identifier("+%s".to_string()));
    assert_eq!(tokens[2], Token::Eof);
}

#[test]
fn test_lexer_date_format_quoted() {
    let input = r#"date '+%Y-%m-%d %H:%M:%S'"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    // Expected tokens: [Identifier("date"), String("+%Y-%m-%d %H:%M:%S"), Eof]
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0], Token::Identifier("date".to_string()));
    assert_eq!(tokens[1], Token::String("+%Y-%m-%d %H:%M:%S".to_string()));
    assert_eq!(tokens[2], Token::Eof);
}

#[test]
fn test_lexer_plus_in_various_contexts() {
    // Test +%Y%m%d%H%M%S format
    let input = "date +%Y%m%d%H%M%S";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[1], Token::Identifier("+%Y%m%d%H%M%S".to_string()));

    // Test bare +x flag
    let input2 = "some_cmd +x";
    let mut lexer2 = Lexer::new(input2);
    let tokens2 = lexer2.tokenize().unwrap();
    assert_eq!(tokens2[1], Token::Identifier("+x".to_string()));
}

#[test]
fn test_lexer_arithmetic_expansion_basic() {
    let input = "y=$((x + 1))";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    // Expected: [Identifier("y"), Assign, ArithmeticExpansion("x + 1"), Eof]
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0], Token::Identifier("y".to_string()));
    assert_eq!(tokens[1], Token::Assign);
    assert_eq!(tokens[2], Token::ArithmeticExpansion("x + 1".to_string()));
    assert_eq!(tokens[3], Token::Eof);
}

#[test]
fn test_lexer_arithmetic_expansion_complex() {
    let input = "sum=$((a + b))";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[2], Token::ArithmeticExpansion("a + b".to_string()));

    let input2 = "diff=$((a - b))";
    let mut lexer2 = Lexer::new(input2);
    let tokens2 = lexer2.tokenize().unwrap();

    assert_eq!(tokens2[2], Token::ArithmeticExpansion("a - b".to_string()));
}

#[test]
fn test_lexer_arithmetic_expansion_nested_parens() {
    let input = "result=$(((a + b) * c))";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(
        tokens[2],
        Token::ArithmeticExpansion("(a + b) * c".to_string())
    );
}

// ============================================================================
// Token Display Tests
// ============================================================================

#[test]
fn test_token_display_if() {
    assert_eq!(format!("{}", Token::If), "if");
}

#[test]
fn test_token_display_then() {
    assert_eq!(format!("{}", Token::Then), "then");
}

#[test]
fn test_token_display_identifier() {
    assert_eq!(
        format!("{}", Token::Identifier("foo".to_string())),
        "Identifier(foo)"
    );
}

#[test]
fn test_token_display_string() {
    assert_eq!(
        format!("{}", Token::String("hello".to_string())),
        "String(hello)"
    );
}

#[test]
fn test_token_display_number() {
    assert_eq!(format!("{}", Token::Number(42)), "Number(42)");
}

#[test]
fn test_token_display_variable() {
    assert_eq!(format!("{}", Token::Variable("x".to_string())), "$x");
}

#[test]
fn test_token_display_arithmetic() {
    assert_eq!(
        format!("{}", Token::ArithmeticExpansion("1+2".to_string())),
        "$((1+2)"
    );
}

#[test]
fn test_token_display_command_sub() {
    assert_eq!(
        format!("{}", Token::CommandSubstitution("ls".to_string())),
        "$(ls)"
    );
}

#[test]
fn test_token_display_comment() {
    assert_eq!(format!("{}", Token::Comment("test".to_string())), "#test");
}

#[test]
fn test_token_display_eof() {
    assert_eq!(format!("{}", Token::Eof), "EOF");
}

#[test]
fn test_token_display_other() {
    // Other tokens use Debug format
    let output = format!("{}", Token::Semicolon);
    assert!(output.contains("Semicolon"));
}

// ============================================================================
// LexerError Tests
// ============================================================================
include!("lexer_tests_extracted_lexer.rs");
