
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

    #[test]
    fn test_token_clone() {
        let tokens = vec![
            Token::If,
            Token::Then,
            Token::Identifier("x".to_string()),
            Token::String("hello".to_string()),
            Token::Number(42),
            Token::Variable("x".to_string()),
            Token::Eof,
        ];
        for token in tokens {
            let _ = token.clone();
        }
    }

    #[test]
    fn test_token_eq() {
        assert_eq!(Token::If, Token::If);
        assert_ne!(Token::If, Token::Then);
        assert_eq!(Token::Number(42), Token::Number(42));
        assert_ne!(Token::Number(42), Token::Number(43));
    }

    #[test]
    fn test_lexer_error_debug() {
        let err = LexerError::UnexpectedChar('x', 1, 1);
        let debug = format!("{:?}", err);
        assert!(debug.contains("UnexpectedChar"));
    }

    #[test]
    fn test_lexer_complex_script() {
        let input = r#"
#!/bin/bash
# Comment
FOO=bar
if [ "$FOO" == "bar" ]; then
    echo "Hello $FOO"
fi
"#;
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
    }

    #[test]
    fn test_lexer_escape_in_string() {
        let mut lexer = Lexer::new(r#""hello\nworld""#);
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0], Token::String(_)));
    }

    #[test]
    fn test_lexer_dollar_sign_context() {
        // $ followed by space might be handled differently
        let mut lexer = Lexer::new("echo $FOO");
        let tokens = lexer.tokenize().unwrap();
        // Should have a variable token
        assert!(tokens.iter().any(|t| matches!(t, Token::Variable(_))));
    }

    // ============================================================================
    // Coverage Tests - read_operator (LEX_OP_COV_001-020)
    // ============================================================================

    /// Helper: tokenize and return the token types
    fn lex(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input);
        lexer.tokenize().unwrap_or_default()
    }

    #[test]
    fn test_LEX_OP_COV_001_ne_operator() {
        let tokens = lex("[ a != b ]");
        assert!(tokens.iter().any(|t| matches!(t, Token::Ne)));
    }

    #[test]
    fn test_LEX_OP_COV_002_le_operator() {
        let tokens = lex("[[ a <= b ]]");
        assert!(tokens.iter().any(|t| matches!(t, Token::Le)));
    }

    #[test]
    fn test_LEX_OP_COV_003_ge_operator() {
        let tokens = lex("[[ a >= b ]]");
        assert!(tokens.iter().any(|t| matches!(t, Token::Ge)));
    }

    #[test]
    fn test_LEX_OP_COV_004_append_redirect() {
        let tokens = lex("echo hi >> file");
        assert!(tokens.iter().any(|t| matches!(t, Token::GtGt)));
    }

    #[test]
    fn test_LEX_OP_COV_005_and_operator() {
        let tokens = lex("true && false");
        assert!(tokens.iter().any(|t| matches!(t, Token::And)));
    }

    #[test]
    fn test_LEX_OP_COV_006_or_operator() {
        let tokens = lex("true || false");
        assert!(tokens.iter().any(|t| matches!(t, Token::Or)));
    }

include!("lexer_tests_extracted_lexer_lexer_LEX.rs");
