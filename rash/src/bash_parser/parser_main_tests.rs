#[cfg(test)]
mod tests {
    use super::parser_arith::ArithToken;
    use super::*;
    #[test]
    fn test_parse_simple_assignment() {
        let mut parser = BashParser::new("FOO=bar").unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(ast.statements[0], BashStmt::Assignment { .. }));
    }

    #[test]
    fn test_parse_if_statement() {
        let input = r#"
if [ $x == 1 ]; then
    echo "one"
fi
"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        assert!(ast
            .statements
            .iter()
            .any(|s| matches!(s, BashStmt::If { .. })));
    }

    // Issue #93: Test inline if/then/else/fi with command condition
    #[test]
    fn test_issue_93_inline_if_with_command_condition() {
        // This is the exact pattern from issue #93 that was failing
        let input = r#"if grep -q "pattern" "$file"; then echo "found"; else echo "not found"; fi"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast.statements.len(),
            1,
            "Should parse single inline if statement"
        );
        match &ast.statements[0] {
            BashStmt::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                // The condition should be a CommandCondition
                assert!(
                    matches!(condition, BashExpr::CommandCondition(_)),
                    "Condition should be CommandCondition, got {:?}",
                    condition
                );

                // Should have then block
                assert!(!then_block.is_empty(), "Should have then block");

                // Should have else block
                assert!(else_block.is_some(), "Should have else block");
            }
            _ => panic!("Expected If statement, got {:?}", ast.statements[0]),
        }
    }

    // Issue #93: Test inline if with grep -q pattern
    #[test]
    fn test_issue_93_inline_if_grep_pattern() {
        let input = r#"if grep -q "MAX_QUEUE_DEPTH.*=.*3" "$BRIDGE"; then pass "1: found"; else fail "1: not found"; fi"#;
        let mut parser = BashParser::new(input).unwrap();
        let result = parser.parse();

        // This should NOT fail with "expected Then, found Identifier"
        assert!(
            result.is_ok(),
            "Parser should handle inline if/grep pattern, got: {:?}",
            result
        );
    }

    // Issue #93: Test while loop with command condition (simple case)
    #[test]
    fn test_issue_93_while_with_command_condition() {
        // Use a simpler while condition that doesn't have redirects
        let input = r#"
while grep -q "pattern" file.txt; do
    echo "found"
done
"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        assert!(
            ast.statements
                .iter()
                .any(|s| matches!(s, BashStmt::While { .. })),
            "Should parse while with command condition"
        );
    }

    #[test]
    fn test_parse_function() {
        let input = r#"
function greet() {
    echo "Hello"
}
"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        assert!(ast
            .statements
            .iter()
            .any(|s| matches!(s, BashStmt::Function { .. })));
    }

    // BUG-011: Function with subshell body
    #[test]
    fn test_parse_function_subshell_body() {
        let input = "myfunc() ( echo subshell )";

        let mut parser = BashParser::new(input).unwrap();
        let ast = parser
            .parse()
            .expect("Should parse function with subshell body");
        assert!(
            ast.statements
                .iter()
                .any(|s| matches!(s, BashStmt::Function { .. })),
            "Should find function statement"
        );
    }

    #[test]
    fn test_glob_bracket_pattern() {
        // Basic bracket glob
        let input = "echo [abc].txt";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().expect("Should parse [abc].txt");
        assert!(matches!(&ast.statements[0], BashStmt::Command { args, .. } if !args.is_empty()));

        // Negated bracket glob [!abc]
        let input2 = "echo [!abc].txt";
        let mut parser2 = BashParser::new(input2).unwrap();
        parser2.parse().expect("Should parse [!abc].txt");
    }

    // BUG-018: Test coproc syntax
    #[test]
    fn test_parse_coproc() {
        // Named coproc
        let input = "coproc myproc { cat; }";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().expect("Should parse named coproc");
        assert!(matches!(
            &ast.statements[0],
            BashStmt::Coproc {
                name: Some(n),
                ..
            } if n == "myproc"
        ));

        // Anonymous coproc
        let input2 = "coproc { cat; }";
        let mut parser2 = BashParser::new(input2).unwrap();
        let ast2 = parser2.parse().expect("Should parse anonymous coproc");
        assert!(matches!(
            &ast2.statements[0],
            BashStmt::Coproc { name: None, .. }
        ));
    }

    // RED PHASE: Arithmetic expansion tests
    #[test]
    fn test_parse_arithmetic_basic() {
        let input = "y=$((x + 1))";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            BashStmt::Assignment { name, value, .. } => {
                assert_eq!(name, "y");
                match value {
                    BashExpr::Arithmetic(arith) => match arith.as_ref() {
                        ArithExpr::Add(left, right) => {
                            assert!(matches!(left.as_ref(), ArithExpr::Variable(v) if v == "x"));
                            assert!(matches!(right.as_ref(), ArithExpr::Number(1)));
                        }
                        _ => panic!("Expected Add expression"),
                    },
                    _ => panic!("Expected Arithmetic expression, got {:?}", value),
                }
            }
            _ => panic!("Expected Assignment statement"),
        }
    }

    #[test]
    fn test_parse_arithmetic_complex() {
        let input = "result=$(((a + b) * c))";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            BashStmt::Assignment { name, value, .. } => {
                assert_eq!(name, "result");
                match value {
                    BashExpr::Arithmetic(arith) => {
                        // Should be: Mul(Add(a, b), c)
                        match arith.as_ref() {
                            ArithExpr::Mul(left, right) => {
                                assert!(matches!(left.as_ref(), ArithExpr::Add(_, _)));
                                assert!(
                                    matches!(right.as_ref(), ArithExpr::Variable(v) if v == "c")
                                );
                            }
                            _ => panic!("Expected Mul expression at top level"),
                        }
                    }
                    _ => panic!("Expected Arithmetic expression"),
                }
            }
            _ => panic!("Expected Assignment statement"),
        }
    }

    #[test]
    fn test_parse_arithmetic_precedence() {
        let input = "z=$((a + b * c))";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            BashStmt::Assignment { name, value, .. } => {
                assert_eq!(name, "z");
                match value {
                    BashExpr::Arithmetic(arith) => {
                        // Should be: Add(a, Mul(b, c)) - multiplication has higher precedence
                        match arith.as_ref() {
                            ArithExpr::Add(left, right) => {
                                assert!(
                                    matches!(left.as_ref(), ArithExpr::Variable(v) if v == "a")
                                );
                                assert!(matches!(right.as_ref(), ArithExpr::Mul(_, _)));
                            }
                            _ => panic!("Expected Add expression at top level"),
                        }
                    }
                    _ => panic!("Expected Arithmetic expression"),
                }
            }
            _ => panic!("Expected Assignment statement"),
        }
    }

    // ============================================================================
    // Coverage Tests - Error Handling
    // ============================================================================

    #[test]
    fn test_parse_error_unexpected_eof() {
        let input = "if true; then";
        let mut parser = BashParser::new(input).unwrap();
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::UnexpectedEof;
        assert_eq!(format!("{}", err), "Unexpected end of file");

        let err2 = ParseError::InvalidSyntax("bad syntax".to_string());
        assert!(format!("{}", err2).contains("bad syntax"));

        let err3 = ParseError::UnexpectedToken {
            expected: "Then".to_string(),
            found: "Else".to_string(),
            line: 5,
        };
        assert!(format!("{}", err3).contains("Then"));
        assert!(format!("{}", err3).contains("Else"));
        assert!(format!("{}", err3).contains("5"));
    }

    // ============================================================================
    // Coverage Tests - While and Until Loops
    // ============================================================================

    #[test]
    fn test_parse_while_basic() {
        let input = "while [ $x -lt 10 ]; do echo $x; done";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(matches!(&ast.statements[0], BashStmt::While { .. }));
    }

    #[test]
    fn test_parse_until_basic() {
        let input = "until [ $x -ge 10 ]; do echo $x; done";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(matches!(&ast.statements[0], BashStmt::Until { .. }));
    }

    // ============================================================================
    // Coverage Tests - For Loops
    // ============================================================================

    #[test]
    fn test_parse_for_in_loop() {
        let input = "for i in 1 2 3; do echo $i; done";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(matches!(&ast.statements[0], BashStmt::For { .. }));
    }

    #[test]
    fn test_parse_for_c_style_basic() {
        let input = "for ((i=0; i<10; i++)); do echo $i; done";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(matches!(&ast.statements[0], BashStmt::ForCStyle { .. }));
    }

    #[test]
    fn test_parse_for_c_style_with_spaces() {
        let input = "for (( i = 0; i < 5; i += 1 )); do echo $i; done";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(matches!(&ast.statements[0], BashStmt::ForCStyle { .. }));
    }

    // ============================================================================
    // Coverage Tests - C-style For Loop Parser (FORCSTYLE_COV_001-015)
    // ============================================================================

    /// Helper: parse C-style for loop and return (init, condition, increment)
    fn parse_for_c_style_parts(input: &str) -> (String, String, String) {
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::ForCStyle {
                init,
                condition,
                increment,
                ..
            } => (init.clone(), condition.clone(), increment.clone()),
            other => panic!("Expected ForCStyle, got {other:?}"),
        }
    }

    #[test]
    fn test_FORCSTYLE_COV_001_le_operator() {
        let (_, cond, _) = parse_for_c_style_parts("for ((i=0; i<=10; i++)); do echo $i; done");
        assert!(cond.contains("<="));
    }

    #[test]
    fn test_FORCSTYLE_COV_002_ge_operator() {
        let (_, cond, _) = parse_for_c_style_parts("for ((i=10; i>=0; i--)); do echo $i; done");
        assert!(cond.contains(">="));
    }

    #[test]
    fn test_FORCSTYLE_COV_003_eq_operator() {
        let (_, cond, _) = parse_for_c_style_parts("for ((i=0; i==0; i++)); do echo $i; done");
        assert!(cond.contains("=="));
    }

    #[test]
    fn test_FORCSTYLE_COV_004_ne_operator() {
        let (_, cond, _) = parse_for_c_style_parts("for ((i=0; i!=10; i++)); do echo $i; done");
        assert!(cond.contains("!="));
    }

    #[test]
    fn test_FORCSTYLE_COV_005_gt_operator() {
        let (_, cond, _) = parse_for_c_style_parts("for ((i=10; i>0; i--)); do echo $i; done");
        assert!(cond.contains(">"));
    }

    #[test]
    fn test_FORCSTYLE_COV_006_variable_token() {
        let (init, _, _) = parse_for_c_style_parts("for (($i=0; $i<10; i++)); do echo $i; done");
        assert!(init.contains("$i"));
    }

    #[test]
    fn test_FORCSTYLE_COV_007_no_semicolon_before_do() {
        // No semicolon between )) and do
        let (init, cond, incr) =
            parse_for_c_style_parts("for ((i=0; i<10; i++))\ndo\necho $i\ndone");
        assert_eq!(init, "i=0");
        assert!(cond.contains("i<10") || cond.contains("i <10") || cond.contains("i< 10"));
        assert!(!incr.is_empty());
    }

    #[test]
    fn test_FORCSTYLE_COV_008_semicolon_before_do() {
        // Explicit semicolon between )) and do
        let (init, _, _) = parse_for_c_style_parts("for ((i=0; i<10; i++)); do echo $i; done");
        assert_eq!(init, "i=0");
    }

    #[test]
    fn test_FORCSTYLE_COV_009_nested_parentheses() {
        // Nested parens in arithmetic
        let (init, _, _) = parse_for_c_style_parts("for (((i)=0; i<10; i++)); do echo $i; done");
        assert!(init.contains("(i)"));
    }

    #[test]
    fn test_FORCSTYLE_COV_010_number_tokens() {
        let (init, cond, incr) =
            parse_for_c_style_parts("for ((i=0; i<100; i++)); do echo $i; done");
        assert!(init.contains("0"));
        assert!(cond.contains("100"));
        assert!(!incr.is_empty());
    }

    #[test]
    fn test_FORCSTYLE_COV_011_multiline_body() {
        let input = "for ((i=0; i<3; i++))\ndo\necho $i\necho done_iter\ndone";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::ForCStyle { body, .. } => {
                assert!(body.len() >= 2);
            }
            other => panic!("Expected ForCStyle, got {other:?}"),
        }
    }

    #[test]
    fn test_FORCSTYLE_COV_012_from_content_variant() {
        // This tests the `parse_for_c_style_from_content` path via ArithmeticExpansion token
        // When the lexer pre-parses ((init;cond;incr)) as a single ArithmeticExpansion token
        let input = "for ((x=1; x<5; x++)); do\necho $x\ndone";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::ForCStyle {
                init,
                condition,
                increment,
                ..
            } => {
                assert!(!init.is_empty());
                assert!(!condition.is_empty());
                assert!(!increment.is_empty());
            }
            other => panic!("Expected ForCStyle, got {other:?}"),
        }
    }

    #[test]
    fn test_FORCSTYLE_COV_013_assign_token() {
        // Tests the Token::Assign (=) path in the content reader
        let (init, _, _) = parse_for_c_style_parts("for ((i=0; i<10; i++)); do echo ok; done");
        assert!(init.contains("=") || init.contains("0"));
    }
}
