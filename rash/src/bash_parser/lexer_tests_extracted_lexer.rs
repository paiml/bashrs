
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

include!("lexer_tests_extracted_lexer_lexer.rs");
