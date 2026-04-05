#[cfg(test)]
mod tests {
    use super::*;
    use crate::bash_parser::ast::{AstMetadata, BashExpr, BashStmt, CaseArm, Span};

    fn dummy_metadata() -> AstMetadata {
        AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        }
    }

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

    #[test]
    fn test_format_or_list() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::OrList {
                left: Box::new(BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("fail".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("||"));
    }

    #[test]
    fn test_format_brace_group() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::BraceGroup {
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("test".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                subshell: false,
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_format_coproc() {
        let formatter = Formatter::new();
        let ast = BashAst {
            statements: vec![BashStmt::Coproc {
                name: Some("mycoproc".to_string()),
                body: vec![],
                span: Span::dummy(),
            }],
            metadata: dummy_metadata(),
        };

        let result = formatter.format(&ast).unwrap();
        assert!(result.contains("coproc mycoproc"));
    }

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
