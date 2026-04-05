
    #[test]
    fn test_return_without_code() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Return {
            code: None,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.is_pure());
    }

    #[test]
    fn test_comment() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Comment {
            text: "# This is a comment".to_string(),
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.is_pure());
    }

    #[test]
    fn test_command_substitution() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Assignment {
            name: "OUT".to_string(),
            index: None,
            value: BashExpr::CommandSubst(Box::new(BashStmt::Command {
                name: "date".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            })),
            exported: false,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("date"));
    }

    #[test]
    fn test_concat_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Assignment {
            name: "X".to_string(),
            index: None,
            value: BashExpr::Concat(vec![
                BashExpr::Literal("a".to_string()),
                BashExpr::Variable("B".to_string()),
            ]),
            exported: false,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.variables.contains_key("X"));
    }

    #[test]
    fn test_default_value_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("set".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::DefaultValue {
                    variable: "VAR".to_string(),
                    default: Box::new(BashExpr::Literal("default".to_string())),
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
    fn test_assign_default_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::AssignDefault {
                variable: "NEWVAR".to_string(),
                default: Box::new(BashExpr::Literal("value".to_string())),
            }],
            redirects: vec![],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.variables.contains_key("NEWVAR"));
        let var = report.scope_info.variables.get("NEWVAR").unwrap();
        assert!(var.assigned);
        assert!(var.used);
    }

    #[test]
    fn test_assign_default_existing_var() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("original".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::AssignDefault {
                    variable: "VAR".to_string(),
                    default: Box::new(BashExpr::Literal("new".to_string())),
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
    fn test_error_if_unset_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("set".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::ErrorIfUnset {
                    variable: "VAR".to_string(),
                    message: Box::new(BashExpr::Literal("VAR is unset".to_string())),
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
    fn test_alternative_value_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("set".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::AlternativeValue {
                    variable: "VAR".to_string(),
                    alternative: Box::new(BashExpr::Literal("alt".to_string())),
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
    fn test_string_length_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "STR".to_string(),
                index: None,
                value: BashExpr::Literal("hello".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::StringLength {
                    variable: "STR".to_string(),
                }],
                redirects: vec![],
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let var = report.scope_info.variables.get("STR").unwrap();
        assert!(var.used);
    }

include!("semantic_tests_extracted_for_return_remove.rs");
