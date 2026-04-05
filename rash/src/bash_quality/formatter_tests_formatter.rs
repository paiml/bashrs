// formatter.rs - Bash script formatter
// Following ruchy design patterns for code formatting
use crate::bash_parser::ast::{ArithExpr, BashAst, BashExpr, BashStmt, TestExpr};
use crate::bash_quality::formatter_config::FormatterConfig;
use anyhow::Result;


impl Formatter {
    #[test]
    fn test_formatter_new() {
        let formatter = Formatter::new();
        assert_eq!(formatter.config.indent_width, 2);
        assert!(!formatter.config.use_tabs);
    }

    #[test]
    fn test_formatter_default() {
        let formatter = Formatter::default();
        assert_eq!(formatter.config.indent_width, 2);
    }

    #[test]
    fn test_formatter_with_config() {
        let config = FormatterConfig {
            indent_width: 4,
            ..Default::default()
        };
        let formatter = Formatter::with_config(config);
        assert_eq!(formatter.config.indent_width, 4);
    }

    #[test]
    fn test_set_source() {
        let mut formatter = Formatter::new();
        assert!(formatter.source.is_none());
        formatter.set_source("echo hello");
        assert!(formatter.source.is_some());
        assert_eq!(formatter.source.unwrap(), "echo hello");
    }

    #[test]
    fn test_format_assignment() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("value".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert_eq!(result, "VAR=value");
    }

    #[test]
    fn test_format_exported_assignment() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("value".to_string()),
                exported: true,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("export "));
        assert!(result.contains("VAR=value"));
    }

    #[test]
    fn test_format_comment() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Comment {
                text: " This is a comment".to_string(),
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert_eq!(result, "# This is a comment");
    }

    #[test]
    fn test_format_command() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![
                    BashExpr::Literal("hello".to_string()),
                    BashExpr::Variable("name".to_string()),
                ],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("echo"));
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_format_function() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "greet".to_string(),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("hello".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("greet() {"));
        assert!(result.contains("  echo hello"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_format_function_not_normalized() {
        let config = FormatterConfig {
            normalize_functions: false,
            ..Default::default()
        };
        let formatter = Formatter::with_config(config);

        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "test".to_string(),
                body: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("function test()"));
    }

    #[test]
    fn test_format_function_space_before_brace() {
        let config = FormatterConfig {
            space_before_brace: false,
            ..Default::default()
        };
        let formatter = Formatter::with_config(config);

        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "test".to_string(),
                body: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("test(){"));
    }

    #[test]
    fn test_format_if() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Test(Box::new(TestExpr::IntEq(
                    BashExpr::Variable("x".to_string()),
                    BashExpr::Literal("1".to_string()),
                ))),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("yes".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                elif_blocks: vec![],
                else_block: None,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("if"));
        assert!(result.contains("then"));
        assert!(result.contains("fi"));
    }

    #[test]
    fn test_format_if_else() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Literal("true".to_string()),
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
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("else"));
    }

    #[test]
    fn test_format_if_elif() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Literal("true".to_string()),
                then_block: vec![],
                elif_blocks: vec![(BashExpr::Literal("false".to_string()), vec![])],
                else_block: None,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("elif"));
    }

    #[test]
    fn test_format_if_inline_then() {
        let config = FormatterConfig {
            inline_then: false,
            ..Default::default()
        };
        let formatter = Formatter::with_config(config);

        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Literal("true".to_string()),
                then_block: vec![],
                elif_blocks: vec![],
                else_block: None,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("\nthen"));
    }

    #[test]
    fn test_format_while() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::While {
                condition: BashExpr::Literal("true".to_string()),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("loop".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("while"));
        assert!(result.contains("do"));
        assert!(result.contains("done"));
    }

    #[test]
    fn test_format_until() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Until {
                condition: BashExpr::Literal("false".to_string()),
                body: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("until"));
        assert!(result.contains("do"));
        assert!(result.contains("done"));
    }

    #[test]
    fn test_format_for() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::For {
                variable: "i".to_string(),
                items: BashExpr::Literal("1 2 3".to_string()),
                body: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("for i in"));
        assert!(result.contains("do"));
        assert!(result.contains("done"));
    }

    #[test]
    fn test_format_for_cstyle() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::ForCStyle {
                init: "i=0".to_string(),
                condition: "i<10".to_string(),
                increment: "i++".to_string(),
                body: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("for (("));
        assert!(result.contains("i=0"));
        assert!(result.contains("i<10"));
        assert!(result.contains("i++"));
    }

    #[test]
    fn test_format_return() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Return {
                code: None,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert_eq!(result, "return");
    }

    #[test]
    fn test_format_return_with_code() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Return {
                code: Some(BashExpr::Literal("0".to_string())),
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert_eq!(result, "return 0");
    }

    #[test]
    fn test_format_case() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Case {
                word: BashExpr::Variable("x".to_string()),
                arms: vec![CaseArm {
                    patterns: vec!["a".to_string()],
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("a".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    }],
                }],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("case"));
        assert!(result.contains("esac"));
        assert!(result.contains(";;"));
    }

    #[test]
    fn test_format_pipeline() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Pipeline {
                commands: vec![
                    BashStmt::Command {
                        name: "ls".to_string(),
                        args: vec![],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                    BashStmt::Command {
                        name: "grep".to_string(),
                        args: vec![BashExpr::Literal("foo".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                ],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("ls | grep"));
    }

    #[test]
    fn test_format_and_list() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::AndList {
                left: Box::new(BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("ok".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("&&"));
    }


        include!("formatter_part3_incl2.rs");
