
    #[test]
    fn test_for_cstyle() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::ForCStyle {
            init: "i=0".to_string(),
            condition: "i<10".to_string(),
            increment: "i++".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("loop".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("echo"));
    }

    #[test]
    fn test_case_statement() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Case {
            word: BashExpr::Variable("opt".to_string()),
            arms: vec![CaseArm {
                patterns: vec!["a".to_string()],
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("option a".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("echo"));
    }

    #[test]
    fn test_pipeline() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Pipeline {
            commands: vec![
                BashStmt::Command {
                    name: "cat".to_string(),
                    args: vec![BashExpr::Literal("file".to_string())],
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
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_reads.contains("cat"));
        assert!(report.effects.file_reads.contains("grep"));
    }

    #[test]
    fn test_and_list() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::AndList {
            left: Box::new(BashStmt::Command {
                name: "test".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }),
            right: Box::new(BashStmt::Command {
                name: "echo".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("test"));
        assert!(report.effects.process_spawns.contains("echo"));
    }

    #[test]
    fn test_or_list() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::OrList {
            left: Box::new(BashStmt::Command {
                name: "false".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }),
            right: Box::new(BashStmt::Command {
                name: "true".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("false"));
        assert!(report.effects.process_spawns.contains("true"));
    }

    #[test]
    fn test_brace_group() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::BraceGroup {
            body: vec![BashStmt::Command {
                name: "pwd".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            subshell: false,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("pwd"));
    }

    #[test]
    fn test_coproc() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Coproc {
            name: Some("mycoproc".to_string()),
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_reads.contains("cat"));
    }

    #[test]
    fn test_function_definition() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Function {
            name: "myfunc".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("hello".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.functions.contains_key("myfunc"));
        let func = report.scope_info.functions.get("myfunc").unwrap();
        assert!(func.calls_detected.contains("echo"));
    }

    #[test]
    fn test_function_redefinition_error() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Function {
                name: "myfunc".to_string(),
                body: vec![],
                span: Span::dummy(),
            },
            BashStmt::Function {
                name: "myfunc".to_string(),
                body: vec![],
                span: Span::dummy(),
            },
        ]);

        let result = analyzer.analyze(&ast);
        assert!(matches!(
            result,
            Err(SemanticError::FunctionRedefinition(_))
        ));
    }

    #[test]
    fn test_return_statement() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Return {
            code: Some(BashExpr::Literal("0".to_string())),
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.is_pure());
    }

include!("semantic_tests_extracted_for_return.rs");
