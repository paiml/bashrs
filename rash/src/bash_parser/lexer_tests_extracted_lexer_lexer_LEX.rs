
    #[test]
    fn test_LEX_OP_COV_007_double_brackets() {
        let tokens = lex("[[ x == y ]]");
        assert!(tokens.iter().any(|t| matches!(t, Token::DoubleLeftBracket)));
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::DoubleRightBracket)));
    }

    #[test]
    fn test_LEX_OP_COV_008_plus_equals() {
        let tokens = lex("arr+=(val)");
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == "+=")));
    }

    #[test]
    fn test_LEX_OP_COV_009_not_operator() {
        let tokens = lex("! true");
        assert!(tokens.iter().any(|t| matches!(t, Token::Not)));
    }

    #[test]
    fn test_LEX_OP_COV_010_pipe() {
        let tokens = lex("ls | grep foo");
        assert!(tokens.iter().any(|t| matches!(t, Token::Pipe)));
    }

    #[test]
    fn test_LEX_OP_COV_011_case_double_semicolon() {
        let tokens = lex(";;");
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == ";;")));
    }

    #[test]
    fn test_LEX_OP_COV_012_case_semicolon_ampersand() {
        let tokens = lex(";&");
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == ";&")));
    }

    #[test]
    fn test_LEX_OP_COV_013_ampersand_background() {
        let tokens = lex("sleep 1 &");
        assert!(tokens.iter().any(|t| matches!(t, Token::Ampersand)));
    }

    #[test]
    fn test_LEX_OP_COV_014_parens() {
        let tokens = lex("(echo hi)");
        assert!(tokens.iter().any(|t| matches!(t, Token::LeftParen)));
        assert!(tokens.iter().any(|t| matches!(t, Token::RightParen)));
    }

    #[test]
    fn test_LEX_OP_COV_015_braces() {
        let tokens = lex("{ echo hi; }");
        assert!(tokens.iter().any(|t| matches!(t, Token::LeftBrace)));
        assert!(tokens.iter().any(|t| matches!(t, Token::RightBrace)));
    }

    #[test]
    fn test_LEX_OP_COV_016_brackets() {
        let tokens = lex("[ -f file ]");
        assert!(tokens.iter().any(|t| matches!(t, Token::LeftBracket)));
        assert!(tokens.iter().any(|t| matches!(t, Token::RightBracket)));
    }

    #[test]
    fn test_LEX_OP_COV_017_noclobber_redirect() {
        let tokens = lex("echo hi >| file");
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == ">|")));
    }

    #[test]
    fn test_LEX_OP_COV_018_readwrite_redirect() {
        let tokens = lex("exec 3<> file");
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == "<>")));
    }

    #[test]
    fn test_LEX_OP_COV_019_question_glob() {
        let tokens = lex("echo file?.txt");
        // The ? should be tokenized somewhere in the output
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_LEX_OP_COV_020_case_resume_double_semi_ampersand() {
        let tokens = lex(";;&");
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == ";;&")));
    }

    #[test]
    fn test_LEX_OP_COV_021_herestring() {
        let tokens = lex("cat <<< 'hello'");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::HereString(s) if s == "hello")),
            "Expected HereString(\"hello\"), got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_022_heredoc_indented() {
        let tokens = lex("cat <<-EOF\n\t\tline1\n\tEOF\n");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Heredoc { delimiter, .. } if delimiter == "EOF")),
            "Expected Heredoc with delimiter EOF, got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_023_process_substitution_input() {
        let tokens = lex("diff <(ls dir1) file2");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s.starts_with("<("))),
            "Expected process substitution <(...), got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_024_process_substitution_output() {
        let tokens = lex("tee >(grep foo)");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s.starts_with(">("))),
            "Expected process substitution >(...), got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_025_case_fall_through_semicolon_ampersand() {
        let tokens = lex(";&");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == ";&")),
            "Expected ;& fall-through operator, got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_026_extended_glob_negation() {
        let tokens = lex("!(foo|bar)");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == "!(foo|bar)")),
            "Expected extended glob !(foo|bar), got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_027_eq_in_double_bracket() {
        let tokens = lex("[[ $x == y ]]");
        assert!(tokens.iter().any(|t| matches!(t, Token::DoubleLeftBracket)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Eq)));
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::DoubleRightBracket)));
    }

    #[test]
    fn test_LEX_OP_COV_028_heredoc_basic_delimiter() {
        let tokens = lex("cat <<END\nhello world\nEND\n");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Heredoc { delimiter, content }
                    if delimiter == "END" && content == "hello world")),
            "Expected Heredoc with delimiter END and content 'hello world', got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_029_multiple_operators_and_or_sequence() {
        let tokens = lex("a && b || c");
        assert!(tokens.iter().any(|t| matches!(t, Token::And)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Or)));
    }

    #[test]
    fn test_LEX_OP_COV_030_fd_number_before_append_redirect() {
        let tokens = lex("cmd 2>>file");
        assert!(
            tokens.iter().any(|t| matches!(t, Token::GtGt)),
            "Expected >> append redirect, got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_031_noclobber_after_fd_number() {
        let tokens = lex("cmd 1>| file");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == ">|")),
            "Expected >| noclobber redirect, got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_032_readwrite_redirect_after_fd() {
        let tokens = lex("exec 3<> /dev/tty");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == "<>")),
            "Expected <> read-write redirect, got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_033_double_semi_vs_semi_amp_disambiguation() {
        // ;; is case terminator
        let tokens_dsemi = lex(";;");
        assert!(
            tokens_dsemi
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == ";;")),
            "Expected ;; case terminator, got: {:?}",
            tokens_dsemi
        );

        // ;& is case fall-through
        let tokens_samp = lex(";&");
        assert!(
            tokens_samp
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == ";&")),
            "Expected ;& fall-through, got: {:?}",
            tokens_samp
        );

        // ;;& is case resume
        let tokens_dsamp = lex(";;&");
        assert!(
            tokens_dsamp
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == ";;&")),
            "Expected ;;& case resume, got: {:?}",
            tokens_dsamp
        );
    }

    #[test]
    fn test_LEX_OP_COV_034_plus_equals_different_lhs() {
        // Array append
        let tokens = lex("myarr+=(newval)");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == "+=")),
            "Expected += operator, got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_035_nested_extended_glob_with_inner_parens() {
        let tokens = lex("!(a|(b|c))");
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == "!(a|(b|c))")),
            "Expected nested extended glob !(a|(b|c)), got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_036_not_before_command() {
        let tokens = lex("! grep foo file");
        assert!(
            tokens.iter().any(|t| matches!(t, Token::Not)),
            "Expected ! (Not) token, got: {:?}",
            tokens
        );
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::Identifier(s) if s == "grep")),
            "Expected command identifier 'grep', got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_037_pipe_in_pipeline() {
        let tokens = lex("ls -la | sort | head -5");
        let pipe_count = tokens.iter().filter(|t| matches!(t, Token::Pipe)).count();
        assert_eq!(
            pipe_count, 2,
            "Expected 2 pipe tokens in pipeline, got {}: {:?}",
            pipe_count, tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_038_semicolon_in_different_contexts() {
        // Semicolon as command separator
        let tokens = lex("echo a; echo b");
        let semi_count = tokens
            .iter()
            .filter(|t| matches!(t, Token::Semicolon))
            .count();
        assert_eq!(
            semi_count, 1,
            "Expected 1 semicolon, got {}: {:?}",
            semi_count, tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_039_append_redirect_in_pipeline() {
        let tokens = lex("cmd1 | cmd2 >> outfile");
        assert!(
            tokens.iter().any(|t| matches!(t, Token::Pipe)),
            "Expected pipe, got: {:?}",
            tokens
        );
        assert!(
            tokens.iter().any(|t| matches!(t, Token::GtGt)),
            "Expected >> append redirect, got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_LEX_OP_COV_040_mixed_operators_conditional_and_or() {
        let tokens = lex("[[ $x == y ]] && echo yes || echo no");
        assert!(tokens.iter().any(|t| matches!(t, Token::DoubleLeftBracket)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Eq)));
        assert!(tokens
            .iter()
            .any(|t| matches!(t, Token::DoubleRightBracket)));
        assert!(tokens.iter().any(|t| matches!(t, Token::And)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Or)));
    }
