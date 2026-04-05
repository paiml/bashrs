#[cfg(test)]
mod tests {
    use super::*;

    fn make_ast(statements: Vec<BashStmt>) -> BashAst {
        BashAst {
            statements,
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        }
    }

    #[test]
    fn test_variable_tracking() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Assignment {
            name: "FOO".to_string(),
            index: None,
            value: BashExpr::Literal("bar".to_string()),
            exported: false,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.variables.contains_key("FOO"));
    }

    #[test]
    fn test_exported_variable_tracking() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Assignment {
            name: "PATH".to_string(),
            index: None,
            value: BashExpr::Literal("/usr/bin".to_string()),
            exported: true,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.variables.get("PATH").unwrap().exported);
        assert!(report.effects.env_modifications.contains("PATH"));
    }

    #[test]
    fn test_effect_tracking() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Command {
            name: "curl".to_string(),
            args: vec![BashExpr::Literal("http://example.com".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.network_access);
    }

    #[test]
    fn test_effect_tracker_is_pure() {
        let tracker = EffectTracker::new();
        assert!(tracker.is_pure());

        let mut impure = EffectTracker::new();
        impure.network_access = true;
        assert!(!impure.is_pure());
    }

    #[test]
    fn test_effect_tracker_default() {
        let tracker = EffectTracker::default();
        assert!(tracker.is_pure());
    }

    #[test]
    fn test_file_read_commands() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Command {
            name: "cat".to_string(),
            args: vec![BashExpr::Literal("file.txt".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_reads.contains("cat"));
    }

    #[test]
    fn test_file_write_commands() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Command {
            name: "rm".to_string(),
            args: vec![BashExpr::Literal("file.txt".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_writes.contains("rm"));
    }

    #[test]
    fn test_network_commands() {
        for cmd in &["wget", "nc", "telnet", "ssh"] {
            let mut analyzer = SemanticAnalyzer::new();
            let ast = make_ast(vec![BashStmt::Command {
                name: cmd.to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }]);

            let report = analyzer.analyze(&ast).unwrap();
            assert!(
                report.effects.network_access,
                "Command {} should enable network_access",
                cmd
            );
        }
    }

    #[test]
    fn test_if_statement() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::StringNonEmpty(BashExpr::Variable(
                "VAR".to_string(),
            )))),
            then_block: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("yes".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            elif_blocks: vec![(
                BashExpr::Test(Box::new(TestExpr::StringEmpty(BashExpr::Literal(
                    "".to_string(),
                )))),
                vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("elif".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
            )],
            else_block: Some(vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("no".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }]),
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("echo"));
    }

    #[test]
    fn test_while_loop() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::While {
            condition: BashExpr::Literal("true".to_string()),
            body: vec![BashStmt::Command {
                name: "sleep".to_string(),
                args: vec![BashExpr::Literal("1".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("sleep"));
    }

    #[test]
    fn test_until_loop() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Until {
            condition: BashExpr::Literal("false".to_string()),
            body: vec![BashStmt::Command {
                name: "wait".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("wait"));
    }

    #[test]
    fn test_for_loop() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::For {
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
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("echo"));
    }

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
}
