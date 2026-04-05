#[cfg(test)]
mod tests {
    use super::parser_arith::ArithToken;
    use super::*;
    #[test]
    fn test_arith_tok_015_octal_numbers() {
        let tokens = tokenize("077");
        assert_eq!(tokens, vec![ArithToken::Number(0o77)]);

        let tokens = tokenize("010");
        assert_eq!(tokens, vec![ArithToken::Number(8)]);
    }

    #[test]
    fn test_arith_tok_016_dollar_variable() {
        let tokens = tokenize("$var");
        assert_eq!(tokens, vec![ArithToken::Variable("var".to_string())]);

        let tokens = tokenize("$foo_bar");
        assert_eq!(tokens, vec![ArithToken::Variable("foo_bar".to_string())]);
    }

    #[test]
    fn test_arith_tok_017_bare_identifier_variable() {
        let tokens = tokenize("count");
        assert_eq!(tokens, vec![ArithToken::Variable("count".to_string())]);

        let tokens = tokenize("_private");
        assert_eq!(tokens, vec![ArithToken::Variable("_private".to_string())]);

        let tokens = tokenize("Var2");
        assert_eq!(tokens, vec![ArithToken::Variable("Var2".to_string())]);
    }

    #[test]
    fn test_arith_tok_018_whitespace_handling() {
        // Tabs, spaces, newlines should all be skipped
        let tokens = tokenize("  1\t+\n2  ");
        assert_eq!(
            tokens,
            vec![
                ArithToken::Number(1),
                ArithToken::Plus,
                ArithToken::Number(2),
            ]
        );
    }

    #[test]
    fn test_arith_tok_019_invalid_character_error() {
        let err = tokenize_err("1 @ 2");
        match err {
            ParseError::InvalidSyntax(msg) => {
                assert!(
                    msg.contains('@'),
                    "Error should mention the invalid char '@': {msg}"
                );
            }
            other => panic!("Expected InvalidSyntax, got: {other:?}"),
        }
    }

    #[test]
    fn test_arith_tok_020_complex_expression() {
        // Full real-world expression: x = (a + b) * c / 2
        let tokens = tokenize("x = (a + b) * c / 2");
        assert_eq!(
            tokens,
            vec![
                ArithToken::Variable("x".to_string()),
                ArithToken::Assign,
                ArithToken::LeftParen,
                ArithToken::Variable("a".to_string()),
                ArithToken::Plus,
                ArithToken::Variable("b".to_string()),
                ArithToken::RightParen,
                ArithToken::Multiply,
                ArithToken::Variable("c".to_string()),
                ArithToken::Divide,
                ArithToken::Number(2),
            ]
        );
    }

    #[test]
    fn test_arith_tok_021_single_token_inputs() {
        // Each single-char operator should produce exactly one token
        let cases: Vec<(&str, ArithToken)> = vec![
            ("+", ArithToken::Plus),
            ("-", ArithToken::Minus),
            ("*", ArithToken::Multiply),
            ("/", ArithToken::Divide),
            ("%", ArithToken::Modulo),
            ("(", ArithToken::LeftParen),
            (")", ArithToken::RightParen),
            ("?", ArithToken::Question),
            (":", ArithToken::Colon),
            ("^", ArithToken::BitXor),
            ("~", ArithToken::BitNot),
            (",", ArithToken::Comma),
        ];
        for (input, expected) in cases {
            let tokens = tokenize(input);
            assert_eq!(tokens, vec![expected], "Failed for input: {input:?}");
        }
    }

    #[test]
    fn test_arith_tok_022_dollar_empty_variable() {
        // $ followed by a non-alphanumeric char should yield an empty variable name
        let tokens = tokenize("$+");
        assert_eq!(
            tokens,
            vec![ArithToken::Variable(String::new()), ArithToken::Plus,]
        );
    }

    #[test]
    fn test_arith_tok_023_adjacent_operators_no_spaces() {
        let tokens = tokenize("1+2*3");
        assert_eq!(
            tokens,
            vec![
                ArithToken::Number(1),
                ArithToken::Plus,
                ArithToken::Number(2),
                ArithToken::Multiply,
                ArithToken::Number(3),
            ]
        );
    }

    #[test]
    fn test_arith_tok_024_zero_standalone() {
        // Just "0" without further digits is a standalone zero
        let tokens = tokenize("0");
        assert_eq!(tokens, vec![ArithToken::Number(0)]);
    }

    #[test]
    fn test_arith_tok_025_all_comparison_in_expression() {
        // Expression mixing several comparison operators
        let tokens = tokenize("a <= b >= c == d != e < f > g");
        assert_eq!(
            tokens,
            vec![
                ArithToken::Variable("a".to_string()),
                ArithToken::Le,
                ArithToken::Variable("b".to_string()),
                ArithToken::Ge,
                ArithToken::Variable("c".to_string()),
                ArithToken::Eq,
                ArithToken::Variable("d".to_string()),
                ArithToken::Ne,
                ArithToken::Variable("e".to_string()),
                ArithToken::Lt,
                ArithToken::Variable("f".to_string()),
                ArithToken::Gt,
                ArithToken::Variable("g".to_string()),
            ]
        );
    }
}

// ============================================================================
// Coverage Tests - C-style For Loop (FOR_C_STYLE_001-025)
// Comprehensive tests for parse_for_c_style and parse_for_c_style_from_content
// ============================================================================
mod for_c_style_tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    /// Helper: parse input and return (init, condition, increment, body_len)
    fn parse_c_for(input: &str) -> (String, String, String, usize) {
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::ForCStyle {
                init,
                condition,
                increment,
                body,
                ..
            } => (
                init.clone(),
                condition.clone(),
                increment.clone(),
                body.len(),
            ),
            other => panic!("Expected ForCStyle, got {other:?}"),
        }
    }

    #[test]
    fn test_FOR_C_STYLE_001_basic_loop() {
        let (init, cond, incr, body_len) = parse_c_for("for ((i=0; i<10; i++)); do echo $i; done");
        assert_eq!(init, "i=0");
        assert!(cond.contains("i") && cond.contains("10"));
        assert!(!incr.is_empty());
        assert!(body_len >= 1);
    }

    #[test]
    fn test_FOR_C_STYLE_002_identifier_tokens() {
        let (init, cond, incr, _) =
            parse_c_for("for ((count=0; count<5; count++)); do echo ok; done");
        assert!(init.contains("count"));
        assert!(cond.contains("count"));
        assert!(incr.contains("count"));
    }

    #[test]
    fn test_FOR_C_STYLE_003_number_tokens() {
        let (init, cond, _, _) = parse_c_for("for ((i=100; i<200; i++)); do echo $i; done");
        assert!(init.contains("100"));
        assert!(cond.contains("200"));
    }

    #[test]
    fn test_FOR_C_STYLE_004_assign_operator() {
        let (init, _, _, _) = parse_c_for("for ((i=0; i<10; i++)); do echo $i; done");
        assert!(init.contains("="));
        assert!(init.contains("i"));
        assert!(init.contains("0"));
    }

    #[test]
    fn test_FOR_C_STYLE_005_lt_operator() {
        let (_, cond, _, _) = parse_c_for("for ((i=0; i<10; i++)); do echo $i; done");
        assert!(cond.contains("<"));
    }

    #[test]
    fn test_FOR_C_STYLE_006_gt_operator() {
        let (_, cond, _, _) = parse_c_for("for ((i=10; i>0; i--)); do echo $i; done");
        assert!(cond.contains(">"));
    }

    #[test]
    fn test_FOR_C_STYLE_007_le_operator() {
        let (_, cond, _, _) = parse_c_for("for ((i=0; i<=10; i++)); do echo $i; done");
        assert!(cond.contains("<="));
    }

    #[test]
    fn test_FOR_C_STYLE_008_ge_operator() {
        let (_, cond, _, _) = parse_c_for("for ((i=10; i>=0; i--)); do echo $i; done");
        assert!(cond.contains(">="));
    }

    #[test]
    fn test_FOR_C_STYLE_009_eq_operator() {
        let (_, cond, _, _) = parse_c_for("for ((i=0; i==0; i++)); do echo ok; done");
        assert!(cond.contains("=="));
    }

    #[test]
    fn test_FOR_C_STYLE_010_ne_operator() {
        let (_, cond, _, _) = parse_c_for("for ((i=0; i!=10; i++)); do echo $i; done");
        assert!(cond.contains("!="));
    }

    #[test]
    fn test_FOR_C_STYLE_011_variable_with_dollar() {
        let (init, cond, _, _) = parse_c_for("for (($x=0; $x<10; x++)); do echo ok; done");
        assert!(init.contains("$x"));
        assert!(cond.contains("$x"));
    }

    #[test]
    fn test_FOR_C_STYLE_012_nested_parens_in_init() {
        let (init, _, _, _) = parse_c_for("for (((i)=0; i<10; i++)); do echo $i; done");
        assert!(init.contains("(i)"));
    }

    #[test]
    fn test_FOR_C_STYLE_013_nested_parens_in_condition() {
        let (_, cond, _, _) = parse_c_for("for ((i=0; (i)<10; i++)); do echo $i; done");
        assert!(cond.contains("(i)"));
    }

    #[test]
    fn test_FOR_C_STYLE_014_nested_parens_in_increment() {
        let (_, _, incr, _) = parse_c_for("for ((i=0; i<10; (i)++)); do echo $i; done");
        assert!(incr.contains("(i)"));
    }

    #[test]
    fn test_FOR_C_STYLE_015_semicolon_before_do() {
        // With explicit semicolon between )) and do
        let (init, cond, incr, _) = parse_c_for("for ((i=0; i<10; i++)); do echo $i; done");
        assert_eq!(init, "i=0");
        assert!(!cond.is_empty());
        assert!(!incr.is_empty());
    }

    #[test]
    fn test_FOR_C_STYLE_016_no_semicolon_before_do() {
        // No semicolon, newline separates )) and do
        let (init, cond, incr, _) = parse_c_for("for ((i=0; i<5; i++))\ndo\necho ok\ndone");
        assert_eq!(init, "i=0");
        assert!(!cond.is_empty());
        assert!(!incr.is_empty());
    }

    #[test]
    fn test_FOR_C_STYLE_017_newlines_around_do() {
        let (init, _, _, body_len) = parse_c_for("for ((i=0; i<3; i++))\n\ndo\n\necho $i\n\ndone");
        assert_eq!(init, "i=0");
        assert!(body_len >= 1);
    }

    #[test]
    fn test_FOR_C_STYLE_018_multiple_body_statements() {
        let (_, _, _, body_len) =
            parse_c_for("for ((i=0; i<3; i++)); do\necho $i\necho done_iter\necho third\ndone");
        assert!(body_len >= 3);
    }

    #[test]
    fn test_FOR_C_STYLE_019_body_with_assignment() {
        let (_, _, _, body_len) = parse_c_for("for ((i=0; i<3; i++)); do\nx=1\necho $x\ndone");
        assert!(body_len >= 2);
    }

    #[test]
    fn test_FOR_C_STYLE_020_complex_increment_expression() {
        let (_, _, incr, _) = parse_c_for("for ((i=0; i<100; i+=10)); do echo $i; done");
        // The increment should contain something representing i+=10
        assert!(!incr.is_empty());
    }

    #[test]
    fn test_FOR_C_STYLE_021_decrementing_loop() {
        let (init, cond, _, _) = parse_c_for("for ((i=10; i>0; i--)); do echo $i; done");
        assert!(init.contains("10"));
        assert!(cond.contains(">"));
    }

    #[test]
    fn test_FOR_C_STYLE_022_from_content_basic() {
        // This exercises parse_for_c_style_from_content via ArithmeticExpansion token
        // The lexer may combine ((...)) into a single token
        let input = "for ((x=1; x<5; x++)); do\necho $x\ndone";
        let (init, cond, incr, body_len) = parse_c_for(input);
        assert!(!init.is_empty());
        assert!(!cond.is_empty());
        assert!(!incr.is_empty());
        assert!(body_len >= 1);
    }

    #[test]
    fn test_FOR_C_STYLE_023_from_content_with_variables() {
        let input = "for ((n=0; n<max; n++)); do\necho $n\ndone";
        let (init, cond, incr, _) = parse_c_for(input);
        assert!(init.contains("n"));
        assert!(cond.contains("n") || cond.contains("max"));
        assert!(!incr.is_empty());
    }

    #[test]
    fn test_FOR_C_STYLE_024_single_body_command() {
        let (_, _, _, body_len) = parse_c_for("for ((i=0; i<1; i++)); do echo only; done");
        assert_eq!(body_len, 1);
    }

    #[test]
    fn test_FOR_C_STYLE_025_all_comparison_operators_together() {
        // Verify different operators parse correctly in separate loops
        let ops = vec![
            ("for ((i=0; i<10; i++)); do echo x; done", "<"),
            ("for ((i=0; i>0; i++)); do echo x; done", ">"),
            ("for ((i=0; i<=10; i++)); do echo x; done", "<="),
            ("for ((i=0; i>=0; i++)); do echo x; done", ">="),
            ("for ((i=0; i==0; i++)); do echo x; done", "=="),
            ("for ((i=0; i!=0; i++)); do echo x; done", "!="),
        ];
        for (input, expected_op) in ops {
            let (_, cond, _, _) = parse_c_for(input);
            assert!(
                cond.contains(expected_op),
                "Expected condition to contain '{expected_op}', got '{cond}' for input: {input}"
            );
        }
    }
}

// ============================================================================
// Coverage Tests - parse_arithmetic_expr (ARITH_EXPR_001-042)
// Comprehensive tests for all 15 precedence levels of arithmetic parsing
// ============================================================================
mod parse_arithmetic_expr_tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    /// Helper: parse an arithmetic expression string into ArithExpr
    fn parse_arith(input: &str) -> ArithExpr {
        let mut parser = BashParser::new("echo x").unwrap();
        parser.parse_arithmetic_expr(input).unwrap()
    }

    /// Helper: parse expecting an error
    fn parse_arith_err(input: &str) -> ParseError {
        let mut parser = BashParser::new("echo x").unwrap();
        parser.parse_arithmetic_expr(input).unwrap_err()
    }

    // ── Primary (Level 15) ────────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_001_number_literal() {
        assert_eq!(parse_arith("42"), ArithExpr::Number(42));
    }

    #[test]
    fn test_ARITH_EXPR_002_variable() {
        assert_eq!(parse_arith("x"), ArithExpr::Variable("x".to_string()));
    }

    #[test]
    fn test_ARITH_EXPR_003_parenthesized_expression() {
        assert_eq!(parse_arith("(7)"), ArithExpr::Number(7));
    }

    #[test]
    fn test_ARITH_EXPR_004_nested_parentheses() {
        assert_eq!(
            parse_arith("((1 + 2))"),
            ArithExpr::Add(
                Box::new(ArithExpr::Number(1)),
                Box::new(ArithExpr::Number(2)),
            )
        );
    }

    // ── Unary (Level 14) ─────────────────────────────────────────────

    #[test]
    fn test_ARITH_EXPR_005_unary_minus() {
        // -5 becomes Sub(Number(0), Number(5))
        assert_eq!(
            parse_arith("-5"),
            ArithExpr::Sub(
                Box::new(ArithExpr::Number(0)),
                Box::new(ArithExpr::Number(5)),
            )
        );
    }

    #[test]
    fn test_ARITH_EXPR_006_unary_plus() {
        // +5 passes through to Number(5)
        assert_eq!(parse_arith("+5"), ArithExpr::Number(5));
    }

    #[test]
    fn test_ARITH_EXPR_007_bitwise_not() {
        // ~x becomes Sub(Number(-1), Variable("x"))
        assert_eq!(
            parse_arith("~x"),
            ArithExpr::Sub(
                Box::new(ArithExpr::Number(-1)),
                Box::new(ArithExpr::Variable("x".to_string())),
            )
        );
    }
}
