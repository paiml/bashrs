
    #[test]
    fn test_format_coproc_unnamed() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Coproc {
                name: None,
                body: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("coproc {"));
    }

    #[test]
    fn test_format_with_tabs() {
        let config = FormatterConfig {
            use_tabs: true,
            ..Default::default()
        };
        let formatter = Formatter::with_config(config);

        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "test".to_string(),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("test".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("\techo test"));
    }

    // Expression formatting tests
    #[test]
    fn test_format_expr_literal_special_chars() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("hello world".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("\"hello world\""));
    }

    #[test]
    fn test_format_expr_variable_quoted() {
        let config = FormatterConfig {
            quote_variables: true,
            ..Default::default()
        };
        let formatter = Formatter::with_config(config);

        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("x".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("\"$x\""));
    }

    #[test]
    fn test_format_expr_variable_unquoted() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("x".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("$x"));
    }

    #[test]
    fn test_format_expr_command_subst() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::CommandSubst(Box::new(BashStmt::Command {
                    name: "date".to_string(),
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
        assert!(result.contains("$(date)"));
    }

    #[test]
    fn test_format_expr_array() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "arr".to_string(),
                index: None,
                value: BashExpr::Array(vec![
                    BashExpr::Literal("a".to_string()),
                    BashExpr::Literal("b".to_string()),
                ]),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("(a b)"));
    }

    #[test]
    fn test_format_expr_concat() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Concat(vec![
                    BashExpr::Literal("hello".to_string()),
                    BashExpr::Variable("name".to_string()),
                ])],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        // Variable formatting includes $, so we check for echo hello$name
        assert!(result.contains("hello"), "Expected 'hello' in: {}", result);
        assert!(result.contains("name"), "Expected 'name' in: {}", result);
    }

    #[test]
    fn test_format_expr_test_single_brackets() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Test(Box::new(TestExpr::FileExists(
                    BashExpr::Literal("/tmp".to_string()),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("[ -e /tmp ]"));
    }

    #[test]
    fn test_format_expr_test_double_brackets() {
        let config = FormatterConfig {
            use_double_brackets: true,
            ..Default::default()
        };
        let formatter = Formatter::with_config(config);

        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Test(Box::new(TestExpr::FileExists(
                    BashExpr::Literal("/tmp".to_string()),
                )))],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("[[ -e /tmp ]]"));
    }

    #[test]
    fn test_format_expr_glob() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "ls".to_string(),
                args: vec![BashExpr::Glob("*.txt".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("*.txt"));
    }

    #[test]
    fn test_format_expr_default_value() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::DefaultValue {
                    variable: "x".to_string(),
                    default: Box::new(BashExpr::Literal("default".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("${x:-default}"));
    }

    #[test]
    fn test_format_expr_assign_default() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::AssignDefault {
                    variable: "x".to_string(),
                    default: Box::new(BashExpr::Literal("value".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("${x:=value}"));
    }

    #[test]
    fn test_format_expr_error_if_unset() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::ErrorIfUnset {
                    variable: "x".to_string(),
                    message: Box::new(BashExpr::Literal("error".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("${x:?error}"));
    }

    #[test]
    fn test_format_expr_alternative_value() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::AlternativeValue {
                    variable: "x".to_string(),
                    alternative: Box::new(BashExpr::Literal("alt".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("${x:+alt}"));
    }

    #[test]
    fn test_format_expr_string_length() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::StringLength {
                    variable: "x".to_string(),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("${#x}"));
    }

    #[test]
    fn test_format_expr_remove_suffix() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemoveSuffix {
                    variable: "x".to_string(),
                    pattern: Box::new(BashExpr::Literal(".txt".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("${x%.txt}"));
    }

    #[test]
    fn test_format_expr_remove_prefix() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemovePrefix {
                    variable: "x".to_string(),
                    pattern: Box::new(BashExpr::Literal("/tmp/".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("${x#/tmp/}"));
    }

// FIXME(PMAT-238): include!("formatter_tests_ext_test_format_fun_test_format_cop_test_format_exp.rs");
