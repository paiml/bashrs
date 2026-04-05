
    #[test]
    fn test_generate_purified_bash_if_with_else() {
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Test(Box::new(TestExpr::StringNonEmpty(BashExpr::Variable(
                    "x".to_string(),
                )))),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("yes".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                elif_blocks: vec![],
                else_block: Some(vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("no".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }]),
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("else"));
    }

    #[test]
    fn test_generate_purified_bash_for_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::For {
                variable: "i".to_string(),
                items: BashExpr::Array(vec![
                    BashExpr::Literal("1".to_string()),
                    BashExpr::Literal("2".to_string()),
                ]),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("i".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("for i in"));
        assert!(output.contains("do"));
        assert!(output.contains("done"));
    }

    #[test]
    fn test_generate_purified_bash_for_c_style() {
        let ast = BashAst {
            statements: vec![BashStmt::ForCStyle {
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
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("for ((i=0; i<10; i++))"));
        assert!(output.contains("do"));
        assert!(output.contains("done"));
    }

    #[test]
    fn test_generate_purified_bash_while_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::While {
                condition: BashExpr::Test(Box::new(TestExpr::IntLt(
                    BashExpr::Variable("i".to_string()),
                    BashExpr::Literal("10".to_string()),
                ))),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("i".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("while"));
        assert!(output.contains("do"));
        assert!(output.contains("done"));
    }

    #[test]
    fn test_generate_purified_bash_until_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::Until {
                condition: BashExpr::Test(Box::new(TestExpr::IntGe(
                    BashExpr::Variable("i".to_string()),
                    BashExpr::Literal("10".to_string()),
                ))),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("i".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        // Until is transformed to while with negated condition
        assert!(output.contains("while"));
        assert!(output.contains("!"));
    }

    #[test]
    fn test_generate_purified_bash_return() {
        let ast = BashAst {
            statements: vec![BashStmt::Return {
                code: Some(BashExpr::Literal("0".to_string())),
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("return 0"));
    }

    #[test]
    fn test_generate_purified_bash_return_without_code() {
        let ast = BashAst {
            statements: vec![BashStmt::Return {
                code: None,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("return"));
    }

    #[test]
    fn test_generate_purified_bash_case() {
        let ast = BashAst {
            statements: vec![BashStmt::Case {
                word: BashExpr::Variable("x".to_string()),
                arms: vec![
                    CaseArm {
                        patterns: vec!["a".to_string()],
                        body: vec![BashStmt::Command {
                            name: "echo".to_string(),
                            args: vec![BashExpr::Literal("A".to_string())],
                            redirects: vec![],
                            span: Span::dummy(),
                        }],
                    },
                    CaseArm {
                        patterns: vec!["b".to_string(), "c".to_string()],
                        body: vec![BashStmt::Command {
                            name: "echo".to_string(),
                            args: vec![BashExpr::Literal("B or C".to_string())],
                            redirects: vec![],
                            span: Span::dummy(),
                        }],
                    },
                ],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("case"));
        assert!(output.contains("esac"));
        assert!(output.contains(";;"));
        assert!(output.contains("b|c"));
    }

include!("generators_tests_extracted_generate_generate.rs");
