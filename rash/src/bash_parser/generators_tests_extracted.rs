#[cfg(test)]
mod tests {
    use super::*;
    use proptest::strategy::ValueTree;

    proptest! {
        #[test]
        fn test_generates_valid_identifiers(id in bash_identifier()) {
            // Should start with letter or underscore
            assert!(id.chars().next().unwrap().is_alphabetic() || id.starts_with('_'));
            // Should be reasonable length
            assert!(id.len() <= 16);
        }

        #[test]
        fn test_generates_valid_expressions(expr in bash_expr(2)) {
            // All expressions should be constructible
            match expr {
                BashExpr::Literal(s) => assert!(!s.is_empty() || s.is_empty()),
                BashExpr::Variable(v) => assert!(!v.is_empty()),
                BashExpr::Array(items) => assert!(items.len() <= 3),
                BashExpr::Arithmetic(_) => {},
                _ => {}
            }
        }

        #[test]
        fn test_generates_valid_statements(stmt in bash_stmt(2)) {
            // All statements should be constructible
            match stmt {
                BashStmt::Assignment { name, .. } => assert!(!name.is_empty()),
                BashStmt::Command { name, .. } => assert!(!name.is_empty()),
                BashStmt::Function { name, body, .. } => {
                    assert!(!name.is_empty());
                    assert!(!body.is_empty());
                }
                _ => {}
            }
        }

        #[test]
        fn test_generates_valid_scripts(script in bash_script()) {
            // Scripts should have at least one statement
            assert!(!script.statements.is_empty());
            assert!(script.statements.len() <= 10);
        }

        /// 🔴 RED: Property test for unique function names
        /// TICKET-6002: bash_script() should generate scripts with unique function names
        #[test]
        fn test_generated_scripts_have_unique_function_names(script in bash_script()) {
            use std::collections::HashSet;

            // Collect all function names
            let mut function_names = HashSet::new();
            let mut duplicate_found = false;
            let mut duplicate_name = String::new();

            for stmt in &script.statements {
                if let BashStmt::Function { name, .. } = stmt {
                    if !function_names.insert(name.clone()) {
                        // Duplicate found!
                        duplicate_found = true;
                        duplicate_name = name.clone();
                        break;
                    }
                }
            }

            prop_assert!(
                !duplicate_found,
                "Generated script has duplicate function name: '{}'. \
                All function names in a script must be unique. \
                Function names found: {:?}",
                duplicate_name,
                function_names
            );
        }
    }

    // ============== generate_purified_bash tests ==============

    #[test]
    fn test_generate_purified_bash_empty() {
        let ast = BashAst {
            statements: vec![],
            metadata: AstMetadata {
                source_file: None,
                line_count: 0,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.starts_with("#!/bin/sh\n"));
    }

    #[test]
    fn test_generate_purified_bash_command() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("hello".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("echo hello"));
    }

    #[test]
    fn test_generate_purified_bash_assignment() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "FOO".to_string(),
                index: None,
                value: BashExpr::Literal("bar".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("FOO=bar"));
    }

    #[test]
    fn test_generate_purified_bash_exported_assignment() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "PATH".to_string(),
                index: None,
                value: BashExpr::Literal("/usr/bin".to_string()),
                exported: true,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("export PATH=/usr/bin"));
    }

    #[test]
    fn test_generate_purified_bash_comment() {
        let ast = BashAst {
            statements: vec![BashStmt::Comment {
                text: "This is a comment".to_string(),
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("# This is a comment"));
    }

    #[test]
    fn test_generate_purified_bash_function() {
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "my_func".to_string(),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("hello".to_string())],
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
        assert!(output.contains("my_func() {"));
        assert!(output.contains("echo hello"));
        assert!(output.contains("}"));
    }

    #[test]
    fn test_generate_purified_bash_if_statement() {
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
                else_block: None,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("if"));
        assert!(output.contains("then"));
        assert!(output.contains("fi"));
    }

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

    #[test]
    fn test_generate_purified_bash_pipeline() {
        let ast = BashAst {
            statements: vec![BashStmt::Pipeline {
                commands: vec![
                    BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("hello".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                    BashStmt::Command {
                        name: "grep".to_string(),
                        args: vec![BashExpr::Literal("h".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
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
        assert!(output.contains("echo hello | grep h"));
    }

    #[test]
    fn test_generate_purified_bash_and_list() {
        let ast = BashAst {
            statements: vec![BashStmt::AndList {
                left: Box::new(BashStmt::Command {
                    name: "true".to_string(),
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
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("true && echo ok"));
    }

    #[test]
    fn test_generate_purified_bash_or_list() {
        let ast = BashAst {
            statements: vec![BashStmt::OrList {
                left: Box::new(BashStmt::Command {
                    name: "false".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("failed".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("false || echo failed"));
    }

    #[test]
    fn test_generate_purified_bash_brace_group() {
        let ast = BashAst {
            statements: vec![BashStmt::BraceGroup {
                body: vec![
                    BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("a".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                    BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("b".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                ],
                subshell: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("{"));
        assert!(output.contains("}"));
    }

    #[test]
    fn test_generate_purified_bash_coproc_with_name() {
        let ast = BashAst {
            statements: vec![BashStmt::Coproc {
                name: Some("mycoproc".to_string()),
                body: vec![BashStmt::Command {
                    name: "cat".to_string(),
                    args: vec![],
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
        assert!(output.contains("coproc mycoproc"));
    }

    #[test]
    fn test_generate_purified_bash_coproc_without_name() {
        let ast = BashAst {
            statements: vec![BashStmt::Coproc {
                name: None,
                body: vec![BashStmt::Command {
                    name: "cat".to_string(),
                    args: vec![],
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
        assert!(output.contains("coproc { cat; }"));
    }

    // ============== generate_expr tests ==============

    #[test]
    fn test_generate_expr_literal_simple() {
        let expr = BashExpr::Literal("hello".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "hello");
    }

    #[test]
    fn test_generate_expr_literal_with_space() {
        let expr = BashExpr::Literal("hello world".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "'hello world'");
    }

    #[test]
    fn test_generate_expr_literal_with_dollar() {
        let expr = BashExpr::Literal("$HOME".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "'$HOME'");
    }

    #[test]
    fn test_generate_expr_variable() {
        let expr = BashExpr::Variable("FOO".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "\"$FOO\"");
    }

    #[test]
    fn test_generate_expr_array() {
        let expr = BashExpr::Array(vec![
            BashExpr::Literal("a".to_string()),
            BashExpr::Literal("b".to_string()),
        ]);
        let output = generate_expr(&expr);
        assert_eq!(output, "a b");
    }

    #[test]
    fn test_generate_expr_arithmetic() {
        let expr = BashExpr::Arithmetic(Box::new(ArithExpr::Add(
            Box::new(ArithExpr::Number(1)),
            Box::new(ArithExpr::Number(2)),
        )));
        let output = generate_expr(&expr);
        assert_eq!(output, "$((1 + 2))");
    }

    #[test]
    fn test_generate_expr_command_subst() {
        let expr = BashExpr::CommandSubst(Box::new(BashStmt::Command {
            name: "date".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        }));
        let output = generate_expr(&expr);
        assert_eq!(output, "$(date)");
    }

    #[test]
    fn test_generate_expr_concat() {
        let expr = BashExpr::Concat(vec![
            BashExpr::Literal("prefix_".to_string()),
            BashExpr::Variable("VAR".to_string()),
        ]);
        let output = generate_expr(&expr);
        assert!(output.contains("prefix_"));
        assert!(output.contains("\"$VAR\""));
    }

    #[test]
    fn test_generate_expr_glob() {
        let expr = BashExpr::Glob("*.txt".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "*.txt");
    }

    #[test]
    fn test_generate_expr_default_value() {
        let expr = BashExpr::DefaultValue {
            variable: "FOO".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FOO:-default}"));
    }

    #[test]
    fn test_generate_expr_assign_default() {
        let expr = BashExpr::AssignDefault {
            variable: "FOO".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FOO:=default}"));
    }

    #[test]
    fn test_generate_expr_error_if_unset() {
        let expr = BashExpr::ErrorIfUnset {
            variable: "FOO".to_string(),
            message: Box::new(BashExpr::Literal("error".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FOO:?error}"));
    }

    #[test]
    fn test_generate_expr_alternative_value() {
        let expr = BashExpr::AlternativeValue {
            variable: "FOO".to_string(),
            alternative: Box::new(BashExpr::Literal("alt".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FOO:+alt}"));
    }

    #[test]
    fn test_generate_expr_string_length() {
        let expr = BashExpr::StringLength {
            variable: "FOO".to_string(),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${#FOO}"));
    }

    #[test]
    fn test_generate_expr_remove_suffix() {
        let expr = BashExpr::RemoveSuffix {
            variable: "FILE".to_string(),
            pattern: Box::new(BashExpr::Literal(".txt".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FILE%.txt}"));
    }

    #[test]
    fn test_generate_expr_remove_prefix() {
        let expr = BashExpr::RemovePrefix {
            variable: "PATH".to_string(),
            pattern: Box::new(BashExpr::Literal("*/".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${PATH#*/}"));
    }

    #[test]
    fn test_generate_expr_remove_longest_prefix() {
        let expr = BashExpr::RemoveLongestPrefix {
            variable: "PATH".to_string(),
            pattern: Box::new(BashExpr::Literal("*/".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${PATH##*/}"));
    }

    #[test]
    fn test_generate_expr_remove_longest_suffix() {
        let expr = BashExpr::RemoveLongestSuffix {
            variable: "FILE".to_string(),
            pattern: Box::new(BashExpr::Literal(".*".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FILE%%.*}"));
    }

    #[test]
    fn test_generate_expr_command_condition() {
        let expr = BashExpr::CommandCondition(Box::new(BashStmt::Command {
            name: "test".to_string(),
            args: vec![
                BashExpr::Literal("-f".to_string()),
                BashExpr::Literal("file".to_string()),
            ],
            redirects: vec![],
            span: Span::dummy(),
        }));
        let output = generate_expr(&expr);
        assert!(output.contains("test -f file"));
    }

    // ============== generate_arith_expr tests ==============

    #[test]
    fn test_generate_arith_expr_number() {
        let expr = ArithExpr::Number(42);
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "42");
    }

    #[test]
    fn test_generate_arith_expr_variable() {
        let expr = ArithExpr::Variable("x".to_string());
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "x");
    }

    #[test]
    fn test_generate_arith_expr_add() {
        let expr = ArithExpr::Add(
            Box::new(ArithExpr::Number(1)),
            Box::new(ArithExpr::Number(2)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "1 + 2");
    }

    #[test]
    fn test_generate_arith_expr_sub() {
        let expr = ArithExpr::Sub(
            Box::new(ArithExpr::Number(5)),
            Box::new(ArithExpr::Number(3)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "5 - 3");
    }

    #[test]
    fn test_generate_arith_expr_mul() {
        let expr = ArithExpr::Mul(
            Box::new(ArithExpr::Number(2)),
            Box::new(ArithExpr::Number(3)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "2 * 3");
    }

    #[test]
    fn test_generate_arith_expr_div() {
        let expr = ArithExpr::Div(
            Box::new(ArithExpr::Number(6)),
            Box::new(ArithExpr::Number(2)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "6 / 2");
    }

    #[test]
    fn test_generate_arith_expr_mod() {
        let expr = ArithExpr::Mod(
            Box::new(ArithExpr::Number(7)),
            Box::new(ArithExpr::Number(3)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "7 % 3");
    }

    // ============== generate_test_expr tests ==============

    #[test]
    fn test_generate_test_expr_string_eq() {
        let expr = TestExpr::StringEq(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("y".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("= y"));
    }

    #[test]
    fn test_generate_test_expr_string_ne() {
        let expr = TestExpr::StringNe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("y".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("!= y"));
    }

    #[test]
    fn test_generate_test_expr_int_eq() {
        let expr = TestExpr::IntEq(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("-eq 5"));
    }

    #[test]
    fn test_generate_test_expr_int_ne() {
        let expr = TestExpr::IntNe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("-ne 5"));
    }

    #[test]
    fn test_generate_test_expr_int_lt() {
        let expr = TestExpr::IntLt(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("-lt 5"));
    }

    #[test]
    fn test_generate_test_expr_int_le() {
        let expr = TestExpr::IntLe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("-le 5"));
    }

    #[test]
    fn test_generate_test_expr_int_gt() {
        let expr = TestExpr::IntGt(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("-gt 5"));
    }

    #[test]
    fn test_generate_test_expr_int_ge() {
        let expr = TestExpr::IntGe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("-ge 5"));
    }

    #[test]
    fn test_generate_test_expr_file_exists() {
        let expr = TestExpr::FileExists(BashExpr::Literal("/tmp".to_string()));
        let output = generate_test_expr(&expr);
        assert!(output.contains("-e /tmp"));
    }

    #[test]
    fn test_generate_test_expr_file_readable() {
        let expr = TestExpr::FileReadable(BashExpr::Literal("/tmp".to_string()));
        let output = generate_test_expr(&expr);
        assert!(output.contains("-r /tmp"));
    }

    #[test]
    fn test_generate_test_expr_file_writable() {
        let expr = TestExpr::FileWritable(BashExpr::Literal("/tmp".to_string()));
        let output = generate_test_expr(&expr);
        assert!(output.contains("-w /tmp"));
    }

    #[test]
    fn test_generate_test_expr_file_executable() {
        let expr = TestExpr::FileExecutable(BashExpr::Literal("/tmp".to_string()));
        let output = generate_test_expr(&expr);
        assert!(output.contains("-x /tmp"));
    }

    #[test]
    fn test_generate_test_expr_file_directory() {
        let expr = TestExpr::FileDirectory(BashExpr::Literal("/tmp".to_string()));
        let output = generate_test_expr(&expr);
        assert!(output.contains("-d /tmp"));
    }

    #[test]
    fn test_generate_test_expr_string_empty() {
        let expr = TestExpr::StringEmpty(BashExpr::Variable("x".to_string()));
        let output = generate_test_expr(&expr);
        assert!(output.contains("-z"));
    }

    #[test]
    fn test_generate_test_expr_string_non_empty() {
        let expr = TestExpr::StringNonEmpty(BashExpr::Variable("x".to_string()));
        let output = generate_test_expr(&expr);
        assert!(output.contains("-n"));
    }

    #[test]
    fn test_generate_test_expr_and() {
        let expr = TestExpr::And(
            Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
            Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("&&"));
    }

    #[test]
    fn test_generate_test_expr_or() {
        let expr = TestExpr::Or(
            Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
            Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("||"));
    }

    #[test]
    fn test_generate_test_expr_not() {
        let expr = TestExpr::Not(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "a".to_string(),
        ))));
        let output = generate_test_expr(&expr);
        assert!(output.contains("!"));
    }

    // ============== negate_condition tests ==============

    #[test]
    fn test_negate_condition_test() {
        let expr = BashExpr::Test(Box::new(TestExpr::IntGt(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        )));
        let output = negate_condition(&expr);
        assert!(output.contains("!"));
    }

    #[test]
    fn test_negate_condition_other() {
        let expr = BashExpr::Variable("x".to_string());
        let output = negate_condition(&expr);
        assert!(output.starts_with("!"));
    }

    // ============== generate_test_condition tests ==============

    #[test]
    fn test_generate_test_condition_all_types() {
        // Test all test condition variants
        let tests = vec![
            (
                TestExpr::StringEq(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("b".to_string()),
                ),
                "=",
            ),
            (
                TestExpr::StringNe(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("b".to_string()),
                ),
                "!=",
            ),
            (
                TestExpr::IntEq(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("1".to_string()),
                ),
                "-eq",
            ),
            (
                TestExpr::IntNe(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("1".to_string()),
                ),
                "-ne",
            ),
            (
                TestExpr::IntLt(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("1".to_string()),
                ),
                "-lt",
            ),
            (
                TestExpr::IntLe(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("1".to_string()),
                ),
                "-le",
            ),
            (
                TestExpr::IntGt(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("1".to_string()),
                ),
                "-gt",
            ),
            (
                TestExpr::IntGe(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("1".to_string()),
                ),
                "-ge",
            ),
            (
                TestExpr::FileExists(BashExpr::Literal("f".to_string())),
                "-e",
            ),
            (
                TestExpr::FileReadable(BashExpr::Literal("f".to_string())),
                "-r",
            ),
            (
                TestExpr::FileWritable(BashExpr::Literal("f".to_string())),
                "-w",
            ),
            (
                TestExpr::FileExecutable(BashExpr::Literal("f".to_string())),
                "-x",
            ),
            (
                TestExpr::FileDirectory(BashExpr::Literal("f".to_string())),
                "-d",
            ),
            (
                TestExpr::StringEmpty(BashExpr::Variable("x".to_string())),
                "-z",
            ),
            (
                TestExpr::StringNonEmpty(BashExpr::Variable("x".to_string())),
                "-n",
            ),
        ];

        for (expr, expected) in tests {
            let output = generate_test_condition(&expr);
            assert!(
                output.contains(expected),
                "Expected '{}' in output: {}",
                expected,
                output
            );
        }
    }

    #[test]
    fn test_generate_test_condition_and_or_not() {
        let and_expr = TestExpr::And(
            Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
            Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
        );
        let and_output = generate_test_condition(&and_expr);
        assert!(and_output.contains("&&"));

        let or_expr = TestExpr::Or(
            Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
            Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
        );
        let or_output = generate_test_condition(&or_expr);
        assert!(or_output.contains("||"));

        let not_expr = TestExpr::Not(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "a".to_string(),
        ))));
        let not_output = generate_test_condition(&not_expr);
        assert!(not_output.contains("!"));
    }

    // ============== generate_condition tests ==============

    #[test]
    fn test_generate_condition_with_test() {
        let expr = BashExpr::Test(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "/tmp".to_string(),
        ))));
        let output = generate_condition(&expr);
        assert!(output.contains("-e /tmp"));
    }

    #[test]
    fn test_generate_condition_with_other() {
        let expr = BashExpr::Variable("x".to_string());
        let output = generate_condition(&expr);
        assert_eq!(output, "\"$x\"");
    }

    // ============== BASH_KEYWORDS tests ==============

    #[test]
    fn test_bash_keywords_contains_expected() {
        assert!(BASH_KEYWORDS.contains(&"if"));
        assert!(BASH_KEYWORDS.contains(&"then"));
        assert!(BASH_KEYWORDS.contains(&"else"));
        assert!(BASH_KEYWORDS.contains(&"fi"));
        assert!(BASH_KEYWORDS.contains(&"for"));
        assert!(BASH_KEYWORDS.contains(&"while"));
        assert!(BASH_KEYWORDS.contains(&"do"));
        assert!(BASH_KEYWORDS.contains(&"done"));
        assert!(BASH_KEYWORDS.contains(&"case"));
        assert!(BASH_KEYWORDS.contains(&"esac"));
    }

    // ============== Strategy function type tests ==============

    #[test]
    fn test_bash_string_generates_valid_output() {
        use proptest::test_runner::TestRunner;
        let strategy = bash_string();
        let mut runner = TestRunner::default();

        // Generate a few values to verify the strategy works
        for _ in 0..5 {
            let value = strategy.new_tree(&mut runner).unwrap().current();
            assert!(value.len() <= 20);
            // Valid characters only
            assert!(value
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == ' '));
        }
    }

    #[test]
    fn test_bash_integer_generates_valid_range() {
        use proptest::test_runner::TestRunner;
        let strategy = bash_integer();
        let mut runner = TestRunner::default();

        for _ in 0..10 {
            let value = strategy.new_tree(&mut runner).unwrap().current();
            assert!(value >= -1000);
            assert!(value < 1000);
        }
    }

    #[test]
    fn test_bash_variable_name_generates_valid() {
        use proptest::test_runner::TestRunner;
        let strategy = bash_variable_name();
        let mut runner = TestRunner::default();

        for _ in 0..5 {
            let value = strategy.new_tree(&mut runner).unwrap().current();
            assert!(!value.is_empty());
            // Should be one of the known variable names
            let valid_names = vec![
                "FOO", "BAR", "PATH", "HOME", "USER", "x", "y", "status", "result",
            ];
            assert!(valid_names.contains(&value.as_str()));
        }
    }

    #[test]
    fn test_bash_test_expr_generates_valid() {
        use proptest::test_runner::TestRunner;
        let strategy = bash_test_expr();
        let mut runner = TestRunner::default();

        // Just verify it generates without panic
        for _ in 0..5 {
            let _value = strategy.new_tree(&mut runner).unwrap().current();
        }
    }
}
