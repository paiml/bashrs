
    #[test]
    fn test_remove_suffix_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "FILE".to_string(),
                index: None,
                value: BashExpr::Literal("test.txt".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemoveSuffix {
                    variable: "FILE".to_string(),
                    pattern: Box::new(BashExpr::Literal(".txt".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let var = report.scope_info.variables.get("FILE").unwrap();
        assert!(var.used);
    }

    #[test]
    fn test_remove_prefix_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "PATH".to_string(),
                index: None,
                value: BashExpr::Literal("/usr/local/bin".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemovePrefix {
                    variable: "PATH".to_string(),
                    pattern: Box::new(BashExpr::Literal("/usr/".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let var = report.scope_info.variables.get("PATH").unwrap();
        assert!(var.used);
    }

    #[test]
    fn test_remove_longest_prefix() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("aaa/bbb/ccc".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemoveLongestPrefix {
                    variable: "VAR".to_string(),
                    pattern: Box::new(BashExpr::Literal("*/".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let var = report.scope_info.variables.get("VAR").unwrap();
        assert!(var.used);
    }

    #[test]
    fn test_remove_longest_suffix() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("aaa/bbb/ccc".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemoveLongestSuffix {
                    variable: "VAR".to_string(),
                    pattern: Box::new(BashExpr::Literal("/*".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let var = report.scope_info.variables.get("VAR").unwrap();
        assert!(var.used);
    }

    #[test]
    fn test_command_condition() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::If {
            condition: BashExpr::CommandCondition(Box::new(BashStmt::Command {
                name: "grep".to_string(),
                args: vec![
                    BashExpr::Literal("-q".to_string()),
                    BashExpr::Literal("pattern".to_string()),
                    BashExpr::Literal("file".to_string()),
                ],
                redirects: vec![],
                span: Span::dummy(),
            })),
            then_block: vec![],
            elif_blocks: vec![],
            else_block: None,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_reads.contains("grep"));
    }

    #[test]
    fn test_glob_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Command {
            name: "ls".to_string(),
            args: vec![BashExpr::Glob("*.txt".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("ls"));
    }

    #[test]
    fn test_arithmetic_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "X".to_string(),
                index: None,
                value: BashExpr::Literal("5".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Assignment {
                name: "Y".to_string(),
                index: None,
                value: BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                    Box::new(ArithExpr::Variable("X".to_string())),
                    Box::new(ArithExpr::Number(10)),
                ))),
                exported: false,
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let x = report.scope_info.variables.get("X").unwrap();
        assert!(x.used);
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Assignment {
            name: "RESULT".to_string(),
            index: None,
            value: BashExpr::Arithmetic(Box::new(ArithExpr::Mod(
                Box::new(ArithExpr::Div(
                    Box::new(ArithExpr::Mul(
                        Box::new(ArithExpr::Sub(
                            Box::new(ArithExpr::Number(10)),
                            Box::new(ArithExpr::Number(2)),
                        )),
                        Box::new(ArithExpr::Number(3)),
                    )),
                    Box::new(ArithExpr::Number(4)),
                )),
                Box::new(ArithExpr::Number(5)),
            ))),
            exported: false,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.variables.contains_key("RESULT"));
    }

    #[test]
    fn test_test_expressions_comparison() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::And(
                Box::new(TestExpr::StringEq(
                    BashExpr::Literal("a".to_string()),
                    BashExpr::Literal("a".to_string()),
                )),
                Box::new(TestExpr::Or(
                    Box::new(TestExpr::StringNe(
                        BashExpr::Literal("x".to_string()),
                        BashExpr::Literal("y".to_string()),
                    )),
                    Box::new(TestExpr::Not(Box::new(TestExpr::StringEmpty(
                        BashExpr::Literal("test".to_string()),
                    )))),
                )),
            ))),
            then_block: vec![],
            elif_blocks: vec![],
            else_block: None,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.is_pure());
    }

    #[test]
    fn test_test_expressions_integer() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::And(
                Box::new(TestExpr::IntEq(
                    BashExpr::Literal("1".to_string()),
                    BashExpr::Literal("1".to_string()),
                )),
                Box::new(TestExpr::And(
                    Box::new(TestExpr::IntNe(
                        BashExpr::Literal("1".to_string()),
                        BashExpr::Literal("2".to_string()),
                    )),
                    Box::new(TestExpr::And(
                        Box::new(TestExpr::IntLt(
                            BashExpr::Literal("1".to_string()),
                            BashExpr::Literal("2".to_string()),
                        )),
                        Box::new(TestExpr::And(
                            Box::new(TestExpr::IntLe(
                                BashExpr::Literal("1".to_string()),
                                BashExpr::Literal("1".to_string()),
                            )),
                            Box::new(TestExpr::And(
                                Box::new(TestExpr::IntGt(
                                    BashExpr::Literal("2".to_string()),
                                    BashExpr::Literal("1".to_string()),
                                )),
                                Box::new(TestExpr::IntGe(
                                    BashExpr::Literal("2".to_string()),
                                    BashExpr::Literal("2".to_string()),
                                )),
                            )),
                        )),
                    )),
                )),
            ))),
            then_block: vec![],
            elif_blocks: vec![],
            else_block: None,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.is_pure());
    }

    #[test]
    fn test_test_expressions_file() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::And(
                Box::new(TestExpr::FileExists(BashExpr::Literal("/tmp".to_string()))),
                Box::new(TestExpr::And(
                    Box::new(TestExpr::FileReadable(BashExpr::Literal(
                        "/tmp".to_string(),
                    ))),
                    Box::new(TestExpr::And(
                        Box::new(TestExpr::FileWritable(BashExpr::Literal(
                            "/tmp".to_string(),
                        ))),
                        Box::new(TestExpr::And(
                            Box::new(TestExpr::FileExecutable(BashExpr::Literal(
                                "/tmp".to_string(),
                            ))),
                            Box::new(TestExpr::FileDirectory(BashExpr::Literal(
                                "/tmp".to_string(),
                            ))),
                        )),
                    )),
                )),
            ))),
            then_block: vec![],
            elif_blocks: vec![],
            else_block: None,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_reads.contains("/tmp"));
    }

    #[test]
    fn test_infer_type_integer() {
        let analyzer = SemanticAnalyzer::new();
        let expr = BashExpr::Literal("42".to_string());
        assert_eq!(analyzer.infer_type(&expr), InferredType::Integer);
    }

    #[test]
    fn test_infer_type_string() {
        let analyzer = SemanticAnalyzer::new();
        let expr = BashExpr::Literal("hello".to_string());
        assert_eq!(analyzer.infer_type(&expr), InferredType::String);
    }

    #[test]
    fn test_infer_type_array() {
        let analyzer = SemanticAnalyzer::new();
        let expr = BashExpr::Array(vec![BashExpr::Literal("a".to_string())]);
        assert_eq!(analyzer.infer_type(&expr), InferredType::Array);
    }

    #[test]
    fn test_infer_type_arithmetic() {
        let analyzer = SemanticAnalyzer::new();
        let expr = BashExpr::Arithmetic(Box::new(ArithExpr::Number(5)));
        assert_eq!(analyzer.infer_type(&expr), InferredType::Integer);
    }

    #[test]
    fn test_infer_type_unknown() {
        let analyzer = SemanticAnalyzer::new();
        let expr = BashExpr::Variable("X".to_string());
        assert_eq!(analyzer.infer_type(&expr), InferredType::Unknown);
    }

    #[test]
    fn test_semantic_analyzer_default() {
        let analyzer = SemanticAnalyzer::default();
        assert!(analyzer.global_scope.variables.is_empty());
    }

    #[test]
    fn test_var_info_fields() {
        let var = VarInfo {
            name: "TEST".to_string(),
            exported: true,
            assigned: true,
            used: false,
            inferred_type: InferredType::String,
        };
        assert_eq!(var.name, "TEST");
        assert!(var.exported);
        assert!(var.assigned);
        assert!(!var.used);
        assert_eq!(var.inferred_type, InferredType::String);
    }

    #[test]
    fn test_function_info_fields() {
        let mut calls = HashSet::new();
        calls.insert("echo".to_string());
        let func = FunctionInfo {
            name: "myfunc".to_string(),
            parameter_count: 2,
            calls_detected: calls,
        };
        assert_eq!(func.name, "myfunc");
        assert_eq!(func.parameter_count, 2);
        assert!(func.calls_detected.contains("echo"));
    }

    #[test]
    fn test_scope_info_with_parent() {
        let parent = ScopeInfo {
            variables: HashMap::new(),
            functions: HashMap::new(),
            parent: None,
        };
        let child = ScopeInfo {
            variables: HashMap::new(),
            functions: HashMap::new(),
            parent: Some(Box::new(parent)),
        };
        assert!(child.parent.is_some());
    }
