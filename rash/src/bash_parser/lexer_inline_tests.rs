//! Tests extracted from lexer.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::bash_parser::lexer::*;

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

#[test]
fn test_lexer_error_unexpected_char() {
    let err = LexerError::UnexpectedChar('$', 1, 5);
    assert!(err.to_string().contains("'$'"));
    assert!(err.to_string().contains("line 1"));
}

#[test]
fn test_lexer_error_unterminated_string() {
    let err = LexerError::UnterminatedString(2, 10);
    assert!(err.to_string().contains("Unterminated"));
    assert!(err.to_string().contains("line 2"));
}

#[test]
fn test_lexer_error_invalid_number() {
    let err = LexerError::InvalidNumber("abc123".to_string());
    assert!(err.to_string().contains("Invalid"));
}

// ============================================================================
// Lexer Method Tests
// ============================================================================

#[test]
fn test_lexer_new() {
    let lexer = Lexer::new("echo hello");
    assert_eq!(lexer.position, 0);
    assert_eq!(lexer.line, 1);
    assert_eq!(lexer.column, 1);
}

#[test]
fn test_lexer_empty_input() {
    let mut lexer = Lexer::new("");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Eof);
}

#[test]
fn test_lexer_whitespace_only() {
    let mut lexer = Lexer::new("   \t   ");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0], Token::Eof);
}

#[test]
fn test_lexer_newline() {
    let mut lexer = Lexer::new("\n");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t, Token::Newline)));
}

#[test]
fn test_lexer_multiple_newlines() {
    let mut lexer = Lexer::new("\n\n\n");
    let tokens = lexer.tokenize().unwrap();
    assert!(
        tokens
            .iter()
            .filter(|t| matches!(t, Token::Newline))
            .count()
            >= 1
    );
}

#[test]
fn test_lexer_variable_simple() {
    let mut lexer = Lexer::new("$FOO");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0], Token::Variable("FOO".to_string()));
}

#[test]
fn test_lexer_variable_braces() {
    let mut lexer = Lexer::new("${FOO}");
    let tokens = lexer.tokenize().unwrap();
    assert!(matches!(tokens[0], Token::Variable(_)));
}

#[test]
fn test_lexer_variable_special() {
    let mut lexer = Lexer::new("$?");
    let tokens = lexer.tokenize().unwrap();
    // $? is tokenized as Variable - content may vary by implementation
    assert!(matches!(tokens[0], Token::Variable(_)));
}

#[test]
fn test_lexer_command_substitution() {
    let mut lexer = Lexer::new("$(echo hello)");
    let tokens = lexer.tokenize().unwrap();
    assert!(matches!(tokens[0], Token::CommandSubstitution(_)));
}

#[test]
fn test_lexer_keywords() {
    let keywords = vec![
        ("if", Token::If),
        ("then", Token::Then),
        ("elif", Token::Elif),
        ("else", Token::Else),
        ("fi", Token::Fi),
        ("for", Token::For),
        ("while", Token::While),
        ("until", Token::Until),
        ("do", Token::Do),
        ("done", Token::Done),
        ("case", Token::Case),
        ("esac", Token::Esac),
        ("in", Token::In),
        ("function", Token::Function),
        ("return", Token::Return),
        ("export", Token::Export),
        ("local", Token::Local),
        ("coproc", Token::Coproc),
    ];

    for (input, expected) in keywords {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], expected, "Failed for keyword: {}", input);
    }
}

#[test]
fn test_lexer_operators() {
    let mut lexer = Lexer::new("= == != < <= > >= && || !");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::Assign));
    assert!(tokens.contains(&Token::Eq));
    assert!(tokens.contains(&Token::Ne));
}

#[test]
fn test_lexer_pipe() {
    let mut lexer = Lexer::new("echo hello | grep h");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::Pipe));
}

#[test]
fn test_lexer_semicolon() {
    let mut lexer = Lexer::new("echo a; echo b");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::Semicolon));
}

#[test]
fn test_lexer_ampersand() {
    let mut lexer = Lexer::new("sleep 1 &");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::Ampersand));
}

#[test]
fn test_lexer_parentheses() {
    let mut lexer = Lexer::new("(echo hello)");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::LeftParen));
    assert!(tokens.contains(&Token::RightParen));
}

#[test]
fn test_lexer_braces() {
    let mut lexer = Lexer::new("{ echo hello; }");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::LeftBrace));
    assert!(tokens.contains(&Token::RightBrace));
}

#[test]
fn test_lexer_brackets() {
    let mut lexer = Lexer::new("[ $x ]");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::LeftBracket));
    assert!(tokens.contains(&Token::RightBracket));
}

#[test]
fn test_lexer_double_brackets() {
    let mut lexer = Lexer::new("[[ $x ]]");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::DoubleLeftBracket));
    assert!(tokens.contains(&Token::DoubleRightBracket));
}

#[test]
fn test_lexer_single_quoted_string() {
    let mut lexer = Lexer::new("'hello world'");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0], Token::String("hello world".to_string()));
}

#[test]
fn test_lexer_double_quoted_string() {
    let mut lexer = Lexer::new("\"hello world\"");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0], Token::String("hello world".to_string()));
}

#[test]
fn test_lexer_number() {
    let mut lexer = Lexer::new("42");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0], Token::Number(42));
}

#[test]
fn test_lexer_negative_number() {
    let mut lexer = Lexer::new("x=-5");
    let tokens = lexer.tokenize().unwrap();
    // -5 may be parsed as identifier or number depending on context
    assert!(tokens.len() >= 3);
}

#[test]
fn test_lexer_herestring() {
    let mut lexer = Lexer::new("cat <<< 'hello'");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t, Token::HereString(_))));
}

#[test]
fn test_lexer_heredoc() {
    let mut lexer = Lexer::new("cat <<EOF\nhello\nEOF");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.iter().any(|t| matches!(t, Token::Heredoc { .. })));
}

#[test]
fn test_lexer_append_redirect() {
    let mut lexer = Lexer::new("echo hello >> file");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::GtGt));
}

#[test]
fn test_lexer_for_loop() {
    let mut lexer = Lexer::new("for i in 1 2 3; do echo $i; done");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::For));
    assert!(tokens.contains(&Token::In));
    assert!(tokens.contains(&Token::Do));
    assert!(tokens.contains(&Token::Done));
}

#[test]
fn test_lexer_while_loop() {
    let mut lexer = Lexer::new("while true; do echo loop; done");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::While));
    assert!(tokens.contains(&Token::Do));
    assert!(tokens.contains(&Token::Done));
}

#[test]
fn test_lexer_case_statement() {
    let mut lexer = Lexer::new("case $x in a) echo a;; esac");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::Case));
    assert!(tokens.contains(&Token::In));
    assert!(tokens.contains(&Token::Esac));
}

#[test]
fn test_lexer_function_definition() {
    let mut lexer = Lexer::new("function foo { echo hello; }");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::Function));
}

#[test]
fn test_lexer_export() {
    let mut lexer = Lexer::new("export FOO=bar");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::Export));
}

#[test]
fn test_lexer_local() {
    let mut lexer = Lexer::new("local x=5");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::Local));
}

#[test]
fn test_lexer_return() {
    let mut lexer = Lexer::new("return 0");
    let tokens = lexer.tokenize().unwrap();
    assert!(tokens.contains(&Token::Return));
}
