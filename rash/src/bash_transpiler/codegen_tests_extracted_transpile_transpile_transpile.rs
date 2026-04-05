
    #[test]
    fn test_transpile_test_file_readable() {
        let test = TestExpr::FileReadable(BashExpr::Literal("/tmp/file".to_string()));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("metadata"));
        assert!(result.contains("readonly"));
    }

    #[test]
    fn test_transpile_test_file_writable() {
        let test = TestExpr::FileWritable(BashExpr::Literal("/tmp/file".to_string()));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("metadata"));
        assert!(result.contains("readonly"));
    }

    #[test]
    fn test_transpile_test_file_executable() {
        let test = TestExpr::FileExecutable(BashExpr::Literal("/tmp/file".to_string()));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test(&test).unwrap();
        assert!(result.contains("is_executable"));
    }

    // Expression direct tests
    #[test]
    fn test_transpile_expr_glob() {
        let expr = BashExpr::Glob("*.txt".to_string());
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("glob"));
        assert!(result.contains("*.txt"));
    }

    #[test]
    fn test_transpile_expr_default_value() {
        let expr = BashExpr::DefaultValue {
            variable: "x".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("unwrap_or"));
    }

    #[test]
    fn test_transpile_expr_assign_default() {
        let expr = BashExpr::AssignDefault {
            variable: "x".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("get_or_insert"));
    }

    #[test]
    fn test_transpile_expr_error_if_unset() {
        let expr = BashExpr::ErrorIfUnset {
            variable: "x".to_string(),
            message: Box::new(BashExpr::Literal("error".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("expect"));
    }

    #[test]
    fn test_transpile_expr_alternative_value() {
        let expr = BashExpr::AlternativeValue {
            variable: "x".to_string(),
            alternative: Box::new(BashExpr::Literal("alt".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("as_ref"));
        assert!(result.contains("map"));
    }

    #[test]
    fn test_transpile_expr_string_length() {
        let expr = BashExpr::StringLength {
            variable: "x".to_string(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains(".len()"));
    }

    #[test]
    fn test_transpile_expr_remove_suffix() {
        let expr = BashExpr::RemoveSuffix {
            variable: "x".to_string(),
            pattern: Box::new(BashExpr::Literal(".txt".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("strip_suffix"));
    }

    #[test]
    fn test_transpile_expr_remove_prefix() {
        let expr = BashExpr::RemovePrefix {
            variable: "x".to_string(),
            pattern: Box::new(BashExpr::Literal("/tmp/".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("strip_prefix"));
    }

    #[test]
    fn test_transpile_expr_remove_longest_prefix() {
        let expr = BashExpr::RemoveLongestPrefix {
            variable: "x".to_string(),
            pattern: Box::new(BashExpr::Literal("/*/".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("rsplit_once"));
    }

    #[test]
    fn test_transpile_expr_remove_longest_suffix() {
        let expr = BashExpr::RemoveLongestSuffix {
            variable: "x".to_string(),
            pattern: Box::new(BashExpr::Literal(".*".to_string())),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("split_once"));
    }

    #[test]
    fn test_transpile_expr_array() {
        let expr = BashExpr::Array(vec![
            BashExpr::Literal("a".to_string()),
            BashExpr::Literal("b".to_string()),
        ]);
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("vec!"));
    }

    #[test]
    fn test_transpile_expr_concat() {
        let expr = BashExpr::Concat(vec![
            BashExpr::Literal("hello".to_string()),
            BashExpr::Variable("name".to_string()),
        ]);
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("format!"));
    }

    #[test]
    fn test_transpile_expr_command_subst() {
        let stmt = BashStmt::Command {
            name: "ls".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        };
        let expr = BashExpr::CommandSubst(Box::new(stmt));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_transpile_expr_command_condition() {
        let stmt = BashStmt::Command {
            name: "test".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        };
        let expr = BashExpr::CommandCondition(Box::new(stmt));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("success"));
    }

    #[test]
    fn test_CODEGEN_COV_001_if_with_elif() {
        let stmt = BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::StringEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ))),
            then_block: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("one".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            elif_blocks: vec![(
                BashExpr::Test(Box::new(TestExpr::StringEq(
                    BashExpr::Variable("x".to_string()),
                    BashExpr::Literal("2".to_string()),
                ))),
                vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("two".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
            )],
            else_block: None,
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("else if"));
    }

    #[test]
    fn test_CODEGEN_COV_002_for_c_style() {
        let stmt = BashStmt::ForCStyle {
            init: "i=0".to_string(),
            condition: "i<10".to_string(),
            increment: "i++".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("C-style for loop"));
    }

    #[test]
    fn test_CODEGEN_COV_003_case_statement() {
        let stmt = BashStmt::Case {
            word: BashExpr::Variable("opt".to_string()),
            arms: vec![
                CaseArm {
                    patterns: vec!["start".to_string()],
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("starting".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    }],
                },
                CaseArm {
                    patterns: vec!["*".to_string()],
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("default".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    }],
                },
            ],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("match"));
        assert!(result.contains("start"));
    }

    #[test]
    fn test_CODEGEN_COV_004_pipeline() {
        let stmt = BashStmt::Pipeline {
            commands: vec![
                BashStmt::Command {
                    name: "cat".to_string(),
                    args: vec![BashExpr::Literal("file.txt".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
                BashStmt::Command {
                    name: "grep".to_string(),
                    args: vec![BashExpr::Literal("pattern".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
            ],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("|"));
    }

    #[test]
    fn test_CODEGEN_COV_005_and_list() {
        let stmt = BashStmt::AndList {
            left: Box::new(BashStmt::Command {
                name: "test".to_string(),
                args: vec![BashExpr::Literal("-f".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }),
            right: Box::new(BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("exists".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("&&"));
    }

    #[test]
    fn test_CODEGEN_COV_006_or_list() {
        let stmt = BashStmt::OrList {
            left: Box::new(BashStmt::Command {
                name: "test".to_string(),
                args: vec![BashExpr::Literal("-f".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }),
            right: Box::new(BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("missing".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("||"));
    }

    #[test]
    fn test_CODEGEN_COV_007_brace_group() {
        let stmt = BashStmt::BraceGroup {
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("inside".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            subshell: false,
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_CODEGEN_COV_008_coproc_named() {
        let stmt = BashStmt::Coproc {
            name: Some("myproc".to_string()),
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("coproc myproc"));
    }

    #[test]
    fn test_CODEGEN_COV_009_coproc_unnamed() {
        let stmt = BashStmt::Coproc {
            name: None,
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("coproc -"));
    }

    #[test]
    fn test_CODEGEN_COV_010_select() {
        let stmt = BashStmt::Select {
            variable: "opt".to_string(),
            items: BashExpr::Array(vec![
                BashExpr::Literal("a".to_string()),
                BashExpr::Literal("b".to_string()),
            ]),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("opt".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_statement(&stmt).unwrap();
        assert!(result.contains("select opt"));
    }

    #[test]
    fn test_CODEGEN_COV_011_expr_arithmetic() {
        let expr = BashExpr::Arithmetic(Box::new(ArithExpr::Add(
            Box::new(ArithExpr::Number(5)),
            Box::new(ArithExpr::Number(3)),
        )));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("5") && result.contains("3"));
    }

    #[test]
    fn test_CODEGEN_COV_012_expr_test() {
        let expr = BashExpr::Test(Box::new(TestExpr::StringEq(
            BashExpr::Literal("a".to_string()),
            BashExpr::Literal("b".to_string()),
        )));
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_expression(&expr).unwrap();
        assert!(result.contains("=="));
    }

    #[test]
    fn test_CODEGEN_COV_013_test_expression_fallback() {
        // Non-Test expr in test position falls through to transpile_expression
        let expr = BashExpr::Literal("true".to_string());
        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let result = transpiler.transpile_test_expression(&expr).unwrap();
        assert!(result.contains("true"));
    }
}
