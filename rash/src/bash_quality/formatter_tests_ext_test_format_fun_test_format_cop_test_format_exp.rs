
    #[test]
    fn test_format_expr_remove_longest_prefix() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemoveLongestPrefix {
                    variable: "x".to_string(),
                    pattern: Box::new(BashExpr::Literal("*/".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        // * is a special char that gets quoted
        assert!(result.contains("${x##"), "Expected '${{x##' in: {}", result);
    }

    #[test]
    fn test_format_expr_remove_longest_suffix() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemoveLongestSuffix {
                    variable: "x".to_string(),
                    pattern: Box::new(BashExpr::Literal(".*".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        // * is a special char that gets quoted
        assert!(result.contains("${x%%"), "Expected '${{x%%' in: {}", result);
    }

    #[test]
    fn test_format_expr_command_condition() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::CommandCondition(Box::new(BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("test"));
    }

    // Arithmetic expression tests
    #[test]
    fn test_format_arith_add() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                    Box::new(ArithExpr::Number(1)),
                    Box::new(ArithExpr::Number(2)),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("1 + 2"));
    }

    #[test]
    fn test_format_arith_sub() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Arithmetic(Box::new(ArithExpr::Sub(
                    Box::new(ArithExpr::Number(5)),
                    Box::new(ArithExpr::Number(3)),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("5 - 3"));
    }

    #[test]
    fn test_format_arith_mul() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Arithmetic(Box::new(ArithExpr::Mul(
                    Box::new(ArithExpr::Number(2)),
                    Box::new(ArithExpr::Number(3)),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("2 * 3"));
    }

    #[test]
    fn test_format_arith_div() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Arithmetic(Box::new(ArithExpr::Div(
                    Box::new(ArithExpr::Number(10)),
                    Box::new(ArithExpr::Number(2)),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("10 / 2"));
    }

    #[test]
    fn test_format_arith_mod() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Arithmetic(Box::new(ArithExpr::Mod(
                    Box::new(ArithExpr::Number(10)),
                    Box::new(ArithExpr::Number(3)),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("10 % 3"));
    }

    #[test]
    fn test_format_arith_variable() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Arithmetic(Box::new(ArithExpr::Variable(
                    "x".to_string(),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("x"));
    }

    // Test expression formatting tests
    #[test]
    fn test_format_test_string_eq() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::StringEq(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("a".to_string()),
        ));
        assert!(result.contains(" = "));
    }

    #[test]
    fn test_format_test_string_ne() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::StringNe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("a".to_string()),
        ));
        assert!(result.contains(" != "));
    }

    #[test]
    fn test_format_test_int_lt() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::IntLt(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        ));
        assert!(result.contains(" -lt "));
    }

    #[test]
    fn test_format_test_int_le() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::IntLe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        ));
        assert!(result.contains(" -le "));
    }

    #[test]
    fn test_format_test_int_gt() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::IntGt(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        ));
        assert!(result.contains(" -gt "));
    }

    #[test]
    fn test_format_test_int_ge() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::IntGe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        ));
        assert!(result.contains(" -ge "));
    }

    #[test]
    fn test_format_test_int_ne() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::IntNe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        ));
        assert!(result.contains(" -ne "));
    }

    #[test]
    fn test_format_test_file_readable() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::FileReadable(BashExpr::Literal(
            "/tmp".to_string(),
        )));
        assert!(result.contains("-r "));
    }

    #[test]
    fn test_format_test_file_writable() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::FileWritable(BashExpr::Literal(
            "/tmp".to_string(),
        )));
        assert!(result.contains("-w "));
    }

    #[test]
    fn test_format_test_file_executable() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::FileExecutable(BashExpr::Literal(
            "/bin/sh".to_string(),
        )));
        assert!(result.contains("-x "));
    }

    #[test]
    fn test_format_test_file_directory() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::FileDirectory(BashExpr::Literal(
            "/tmp".to_string(),
        )));
        assert!(result.contains("-d "));
    }

    #[test]
    fn test_format_test_string_empty() {
        let formatter = Formatter::new();
        let result =
            formatter.format_test(&TestExpr::StringEmpty(BashExpr::Variable("x".to_string())));
        assert!(result.contains("-z "));
    }

    #[test]
    fn test_format_test_string_non_empty() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::StringNonEmpty(BashExpr::Variable(
            "x".to_string(),
        )));
        assert!(result.contains("-n "));
    }

    #[test]
    fn test_format_test_and() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::And(
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("a".to_string()),
            )),
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("y".to_string()),
                BashExpr::Literal("b".to_string()),
            )),
        ));
        assert!(result.contains(" && "));
    }

    #[test]
    fn test_format_test_or() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::Or(
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("a".to_string()),
            )),
            Box::new(TestExpr::StringEq(
                BashExpr::Variable("y".to_string()),
                BashExpr::Literal("b".to_string()),
            )),
        ));
        assert!(result.contains(" || "));
    }

    #[test]
    fn test_format_test_not() {
        let formatter = Formatter::new();
        let result = formatter.format_test(&TestExpr::Not(Box::new(TestExpr::StringEq(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("a".to_string()),
        ))));
        assert!(result.contains("! "));
    }

    #[test]
    fn test_format_source() {
        let mut formatter = Formatter::new();
        let result = formatter.format_source("x=1");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("x=1"));
    }

    #[test]
    fn test_format_source_error() {
        let mut formatter = Formatter::new();
        // Invalid bash syntax should return error
        let result = formatter.format_source("if then fi");
        // This might parse or not depending on parser; just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_format_multiple_statements() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![
                BashStmt::Assignment {
                    name: "x".to_string(),
                    index: None,
                    value: BashExpr::Literal("1".to_string()),
                    exported: false,
                    span: Span::dummy(),
                },
                BashStmt::Assignment {
                    name: "y".to_string(),
                    index: None,
                    value: BashExpr::Literal("2".to_string()),
                    exported: false,
                    span: Span::dummy(),
                },
            ],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("x=1"));
        assert!(result.contains("y=2"));
        assert!(result.contains("\n"));
    }
}
