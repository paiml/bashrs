
    /// Test: REPL-004-002-008 - Display case statement AST
    #[test]
    fn test_REPL_004_002_display_case_statement() {
        use crate::bash_parser::ast::CaseArm;

        let ast = BashAst {
            statements: vec![BashStmt::Case {
                word: BashExpr::Variable("choice".to_string()),
                arms: vec![
                    CaseArm {
                        patterns: vec!["yes".to_string(), "y".to_string()],
                        body: vec![],
                    },
                    CaseArm {
                        patterns: vec!["no".to_string(), "n".to_string()],
                        body: vec![],
                    },
                ],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 5,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("Case statement (2 arms)"));
    }

    /// Test: REPL-004-002-009 - Display until loop AST
    #[test]
    fn test_REPL_004_002_display_until_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::Until {
                condition: BashExpr::Literal("false".to_string()),
                body: vec![BashStmt::Command {
                    name: "work".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: dummy_span(),
                }],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 3,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("Until loop (1 statements)"));
    }

    /// Test: REPL-004-002-010 - Display return statement AST
    #[test]
    fn test_REPL_004_002_display_return_statement() {
        let ast = BashAst {
            statements: vec![BashStmt::Return {
                code: Some(BashExpr::Literal("0".to_string())),
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("Return statement"));
    }

    /// Test: REPL-004-002-011 - Display comment AST
    #[test]
    fn test_REPL_004_002_display_comment() {
        let ast = BashAst {
            statements: vec![BashStmt::Comment {
                text: "This is a test comment\nwith multiple lines".to_string(),
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 2,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("Comment: This is a test comment"));
    }

    /// Test: REPL-004-002-012 - Display pipeline AST
    #[test]
    fn test_REPL_004_002_display_pipeline() {
        let ast = BashAst {
            statements: vec![BashStmt::Pipeline {
                commands: vec![
                    BashStmt::Command {
                        name: "cat".to_string(),
                        args: vec![BashExpr::Literal("file.txt".to_string())],
                        redirects: vec![],
                        span: dummy_span(),
                    },
                    BashStmt::Command {
                        name: "grep".to_string(),
                        args: vec![BashExpr::Literal("pattern".to_string())],
                        redirects: vec![],
                        span: dummy_span(),
                    },
                    BashStmt::Command {
                        name: "wc".to_string(),
                        args: vec![BashExpr::Literal("-l".to_string())],
                        redirects: vec![],
                        span: dummy_span(),
                    },
                ],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("Pipeline (3 commands)"));
    }

    /// Test: REPL-004-002-013 - Display AndList AST
    #[test]
    fn test_REPL_004_002_display_and_list() {
        let ast = BashAst {
            statements: vec![BashStmt::AndList {
                left: Box::new(BashStmt::Command {
                    name: "true".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: dummy_span(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("success".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                }),
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("AndList (&&)"));
    }

    /// Test: REPL-004-002-014 - Display OrList AST
    #[test]
    fn test_REPL_004_002_display_or_list() {
        let ast = BashAst {
            statements: vec![BashStmt::OrList {
                left: Box::new(BashStmt::Command {
                    name: "false".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: dummy_span(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("fallback".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                }),
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("OrList (||)"));
    }

    /// Test: REPL-004-002-015 - Display BraceGroup AST
    #[test]
    fn test_REPL_004_002_display_brace_group() {
        let ast = BashAst {
            statements: vec![BashStmt::BraceGroup {
                body: vec![
                    BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("inside brace".to_string())],
                        redirects: vec![],
                        span: dummy_span(),
                    },
                    BashStmt::Command {
                        name: "pwd".to_string(),
                        args: vec![],
                        redirects: vec![],
                        span: dummy_span(),
                    },
                ],
                subshell: false,
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 2,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("BraceGroup (2 statements)"));
    }

    /// Test: REPL-004-002-016 - Display function AST
    #[test]
    fn test_REPL_004_002_display_function() {
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "my_function".to_string(),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("Hello".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                }],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 3,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("Function: my_function (1 statements)"));
    }

    /// Test: REPL-004-002-017 - Display C-style for loop AST
    #[test]
    fn test_REPL_004_002_display_c_style_for_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::ForCStyle {
                init: "i=0".to_string(),
                condition: "i<10".to_string(),
                increment: "i++".to_string(),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("i".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                }],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 3,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("C-style for loop: i=0; i<10; i++ (1 statements)"));
    }

    /// Test: REPL-004-002-018 - Display if with elif and else
    #[test]
    fn test_REPL_004_002_display_if_with_elif_else() {
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Literal("test1".to_string()),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("first".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                }],
                elif_blocks: vec![
                    (BashExpr::Literal("test2".to_string()), vec![]),
                    (BashExpr::Literal("test3".to_string()), vec![]),
                ],
                else_block: Some(vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("default".to_string())],
                    redirects: vec![],
                    span: dummy_span(),
                }]),
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 7,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        assert!(output.contains("If statement"));
        assert!(output.contains("then: 1 statements"));
        assert!(output.contains("elif: 2 branches"));
        assert!(output.contains("else: present"));
    }

    /// Test: REPL-004-002-019 - Display command with no args
    #[test]
    fn test_REPL_004_002_display_command_no_args() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "pwd".to_string(),
                args: vec![],
                redirects: vec![],
                span: dummy_span(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let output = format_ast(&ast);
        // Command with no args should not have "(args: X)"
        assert!(output.contains("Command: pwd"));
        assert!(!output.contains("(args: 0)"));

