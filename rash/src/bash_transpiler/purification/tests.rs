    use super::*;

    #[test]
    fn test_purify_removes_random_variable() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "value".to_string(),
                index: None,
                value: BashExpr::Variable("RANDOM".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        // RANDOM should be replaced with deterministic value
        assert_eq!(purified.statements.len(), 1);
        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => {
                assert!(matches!(value, BashExpr::Literal(_)));
            }
            _ => panic!("Expected assignment"),
        }

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_mkdir_idempotency_warning() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "mkdir".to_string(),
                args: vec![BashExpr::Literal("/tmp/test".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().idempotency_fixes.is_empty());
    }

    #[test]
    fn test_purify_preserves_deterministic_code() {
        let ast = BashAst {
            statements: vec![
                BashStmt::Assignment {
                    name: "FOO".to_string(),
                    index: None,
                    value: BashExpr::Literal("bar".to_string()),
                    exported: false,
                    span: Span::dummy(),
                },
                BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("FOO".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
            ],
            metadata: AstMetadata {
                source_file: None,
                line_count: 2,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        // Deterministic code should be unchanged
        assert_eq!(purified.statements.len(), ast.statements.len());
        assert!(purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_PHASE2_001_mkdir_gets_p_flag() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "mkdir".to_string(),
                args: vec![BashExpr::Literal("/app/releases".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).expect("purification should succeed");

        // Should produce a single mkdir -p command
        assert_eq!(purified.statements.len(), 1);
        match &purified.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "mkdir");
                let has_p_flag = args
                    .iter()
                    .any(|arg| matches!(arg, BashExpr::Literal(s) if s == "-p"));
                assert!(has_p_flag, "mkdir should have -p flag: {args:?}");
            }
            other => panic!("Expected Command, got: {other:?}"),
        }

        assert!(
            !purifier.report().idempotency_fixes.is_empty(),
            "Should report idempotency fix"
        );
    }

    #[test]
    fn test_PHASE2_002_mkdir_p_integration() {
        use crate::bash_parser::codegen::generate_purified_bash;

        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "mkdir".to_string(),
                args: vec![BashExpr::Literal("/opt/app".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).expect("purification should succeed");
        let generated_code = generate_purified_bash(&purified);

        // Generated code should have mkdir -p
        assert!(
            generated_code.contains("mkdir") && generated_code.contains("-p"),
            "Generated code should have mkdir -p: {}",
            generated_code
        );
    }

    // ============== PurificationOptions tests ==============

    #[test]
    fn test_purification_options_default() {
        let opts = PurificationOptions::default();
        assert!(opts.strict_idempotency);
        assert!(opts.remove_non_deterministic);
        assert!(opts.track_side_effects);
    }

    #[test]
    fn test_purification_options_clone() {
        let opts = PurificationOptions {
            strict_idempotency: false,
            remove_non_deterministic: true,
            track_side_effects: false,
            type_check: false,
            emit_guards: false,
            type_strict: false,
        };
        let cloned = opts.clone();
        assert!(!cloned.strict_idempotency);
        assert!(cloned.remove_non_deterministic);
        assert!(!cloned.track_side_effects);
    }

    #[test]
    fn test_purification_options_debug() {
        let opts = PurificationOptions::default();
        let debug_str = format!("{:?}", opts);
        assert!(debug_str.contains("strict_idempotency"));
        assert!(debug_str.contains("remove_non_deterministic"));
    }

    // ============== PurificationReport tests ==============

    #[test]
    fn test_purification_report_new() {
        let report = PurificationReport::new();
        assert!(report.idempotency_fixes.is_empty());
        assert!(report.determinism_fixes.is_empty());
        assert!(report.side_effects_isolated.is_empty());
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn test_purification_report_clone() {
        let mut report = PurificationReport::new();
        report.idempotency_fixes.push("fix1".to_string());
        report.warnings.push("warn1".to_string());
        let cloned = report.clone();
        assert_eq!(cloned.idempotency_fixes.len(), 1);
        assert_eq!(cloned.warnings.len(), 1);
    }

    #[test]
    fn test_purification_report_debug() {
        let report = PurificationReport::new();
        let debug_str = format!("{:?}", report);
        assert!(debug_str.contains("idempotency_fixes"));
    }

    // ============== PurificationError tests ==============

    #[test]
    fn test_purification_error_non_deterministic() {
        let err = PurificationError::NonDeterministicConstruct("$RANDOM".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("non-deterministic"));
        assert!(msg.contains("$RANDOM"));
    }

    #[test]
    fn test_purification_error_non_idempotent() {
        let err = PurificationError::NonIdempotentSideEffect("mkdir /tmp".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("idempotent"));
    }

    #[test]
    fn test_purification_error_debug() {
        let err = PurificationError::NonDeterministicConstruct("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NonDeterministicConstruct"));
    }

    // ============== Purifier non-deterministic variable tests ==============

    #[test]
    fn test_purify_removes_seconds_variable() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "time".to_string(),
                index: None,
                value: BashExpr::Variable("SECONDS".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => {
                assert!(matches!(value, BashExpr::Literal(s) if s == "0"));
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_purify_removes_bashpid_variable() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "pid".to_string(),
                index: None,
                value: BashExpr::Variable("BASHPID".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => {
                assert!(matches!(value, BashExpr::Literal(_)));
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_purify_removes_ppid_variable() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "parent".to_string(),
                index: None,
                value: BashExpr::Variable("PPID".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => {
                assert!(matches!(value, BashExpr::Literal(_)));
            }
            _ => panic!("Expected assignment"),
        }
    }

    // ============== Purifier strict mode tests ==============

    #[test]
    fn test_purify_strict_mode_rejects_random() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::Variable("RANDOM".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let opts = PurificationOptions {
            strict_idempotency: true,
            remove_non_deterministic: false,
            track_side_effects: false,
            type_check: false,
            emit_guards: false,
            type_strict: false,
        };

        let mut purifier = Purifier::new(opts);
        let result = purifier.purify(&ast);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            PurificationError::NonDeterministicConstruct(_)
        ));
    }

    // ============== Command purification tests ==============

    #[test]
    fn test_purify_rm_adds_force_flag() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "rm".to_string(),
                args: vec![BashExpr::Literal("/tmp/file".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "rm");
                assert!(args
                    .iter()
                    .any(|a| matches!(a, BashExpr::Literal(s) if s == "-f")));
            }
            _ => panic!("Expected command"),
        }

        assert!(!purifier.report().idempotency_fixes.is_empty());
    }

    #[test]
    fn test_purify_rm_keeps_existing_force_flag() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "rm".to_string(),
                args: vec![
                    BashExpr::Literal("-f".to_string()),
                    BashExpr::Literal("/tmp/file".to_string()),
                ],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Command { args, .. } => {
                // Should not have duplicate -f flags
                let f_count = args
                    .iter()
                    .filter(|a| matches!(a, BashExpr::Literal(s) if s == "-f"))
                    .count();
                assert_eq!(f_count, 1);
            }
            _ => panic!("Expected command"),
        }
    }

    #[test]
    fn test_purify_echo_unchanged() {
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "echo");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected command"),
        }
    }

    #[test]
    fn test_purify_cp_generates_warning() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "cp".to_string(),
                args: vec![
                    BashExpr::Literal("src".to_string()),
                    BashExpr::Literal("dst".to_string()),
                ],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().warnings.is_empty());
        assert!(purifier.report().warnings[0].contains("cp"));
    }

    #[test]
    fn test_purify_mv_generates_warning() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "mv".to_string(),
                args: vec![
                    BashExpr::Literal("src".to_string()),
                    BashExpr::Literal("dst".to_string()),
                ],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().warnings.is_empty());
        assert!(purifier.report().warnings[0].contains("mv"));
    }

    #[test]
    fn test_purify_unknown_command_tracks_side_effect() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "custom_cmd".to_string(),
                args: vec![BashExpr::Literal("arg1".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().side_effects_isolated.is_empty());
    }

    // ============== Function purification tests ==============

    #[test]
    fn test_purify_function() {
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Function { name, body, .. } => {
                assert_eq!(name, "my_func");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected function"),
        }
    }

    // ============== If statement purification tests ==============

    #[test]
    fn test_purify_if_statement() {
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        assert!(matches!(&purified.statements[0], BashStmt::If { .. }));
    }

    #[test]
    fn test_purify_if_with_elif_and_else() {
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Test(Box::new(TestExpr::IntEq(
                    BashExpr::Variable("x".to_string()),
                    BashExpr::Literal("1".to_string()),
                ))),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("one".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                elif_blocks: vec![(
                    BashExpr::Test(Box::new(TestExpr::IntEq(
                        BashExpr::Variable("x".to_string()),
                        BashExpr::Literal("2".to_string()),
                    ))),
                    vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("two".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    }],
                )],
                else_block: Some(vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("other".to_string())],
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::If {
                elif_blocks,
                else_block,
                ..
            } => {
                assert_eq!(elif_blocks.len(), 1);
                assert!(else_block.is_some());
            }
            _ => panic!("Expected if statement"),
        }
    }

    // ============== Loop purification tests ==============

    #[test]
    fn test_purify_while_loop() {
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        assert!(matches!(&purified.statements[0], BashStmt::While { .. }));
    }

    #[test]
    fn test_purify_until_loop() {
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        assert!(matches!(&purified.statements[0], BashStmt::Until { .. }));
    }

    #[test]
    fn test_purify_for_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::For {
                variable: "item".to_string(),
                items: BashExpr::Array(vec![
                    BashExpr::Literal("a".to_string()),
                    BashExpr::Literal("b".to_string()),
                ]),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("item".to_string())],
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        assert!(matches!(&purified.statements[0], BashStmt::For { .. }));
    }

    #[test]
    fn test_purify_for_c_style_loop() {
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        assert!(matches!(
            &purified.statements[0],
            BashStmt::ForCStyle { .. }
        ));
    }

    // ============== Case statement purification tests ==============

    #[test]
    fn test_purify_case_statement() {
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
                        patterns: vec!["*".to_string()],
                        body: vec![BashStmt::Command {
                            name: "echo".to_string(),
                            args: vec![BashExpr::Literal("default".to_string())],
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Case { arms, .. } => {
                assert_eq!(arms.len(), 2);
            }
            _ => panic!("Expected case statement"),
        }
    }

    // ============== Return statement purification tests ==============

    #[test]
    fn test_purify_return_with_code() {
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Return { code, .. } => {
                assert!(code.is_some());
            }
            _ => panic!("Expected return statement"),
        }
    }

    #[test]
    fn test_purify_return_without_code() {
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Return { code, .. } => {
                assert!(code.is_none());
            }
            _ => panic!("Expected return statement"),
        }
    }

    // ============== Comment purification tests ==============

    #[test]
    fn test_purify_comment_unchanged() {
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Comment { text, .. } => {
                assert_eq!(text, "This is a comment");
            }
            _ => panic!("Expected comment"),
        }
    }

    // ============== Pipeline purification tests ==============

    #[test]
    fn test_purify_pipeline() {
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Pipeline { commands, .. } => {
                assert_eq!(commands.len(), 2);
            }
            _ => panic!("Expected pipeline"),
        }
    }

    // ============== AndList/OrList purification tests ==============

    #[test]
    fn test_purify_and_list() {
        let ast = BashAst {
            statements: vec![BashStmt::AndList {
                left: Box::new(BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![
                        BashExpr::Literal("-f".to_string()),
                        BashExpr::Literal("file".to_string()),
                    ],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("exists".to_string())],
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        assert!(matches!(&purified.statements[0], BashStmt::AndList { .. }));
    }

    #[test]
    fn test_purify_or_list() {
        let ast = BashAst {
            statements: vec![BashStmt::OrList {
                left: Box::new(BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![
                        BashExpr::Literal("-f".to_string()),
                        BashExpr::Literal("file".to_string()),
                    ],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("not found".to_string())],
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        assert!(matches!(&purified.statements[0], BashStmt::OrList { .. }));
    }

    // ============== BraceGroup purification tests ==============

    #[test]
    fn test_purify_brace_group() {
        let ast = BashAst {
            statements: vec![BashStmt::BraceGroup {
                body: vec![
                    BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("one".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                    BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("two".to_string())],
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::BraceGroup { body, .. } => {
                assert_eq!(body.len(), 2);
            }
            _ => panic!("Expected brace group"),
        }
    }

    // ============== Coproc purification tests ==============

    #[test]
    fn test_purify_coproc() {
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Coproc { name, body, .. } => {
                assert_eq!(name.as_deref(), Some("mycoproc"));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected coproc"),
        }
    }

    // ============== Expression purification tests ==============

    #[test]
    fn test_purify_command_substitution() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "output".to_string(),
                index: None,
                value: BashExpr::CommandSubst(Box::new(BashStmt::Command {
                    name: "date".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                })),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _purified = purifier.purify(&ast).unwrap();

        // Should generate a warning about command substitution
        assert!(!purifier.report().warnings.is_empty());
        assert!(purifier.report().warnings[0].contains("Command substitution"));
    }

    #[test]
    fn test_purify_array() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "arr".to_string(),
                index: None,
                value: BashExpr::Array(vec![
                    BashExpr::Literal("a".to_string()),
                    BashExpr::Variable("RANDOM".to_string()),
                ]),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        // RANDOM should be replaced
        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => match value {
                BashExpr::Array(items) => {
                    assert_eq!(items.len(), 2);
                    assert!(matches!(&items[1], BashExpr::Literal(_)));
                }
                _ => panic!("Expected array"),
            },
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_purify_concat() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::Concat(vec![
                    BashExpr::Literal("prefix_".to_string()),
                    BashExpr::Variable("RANDOM".to_string()),
                ]),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _purified = purifier.purify(&ast).unwrap();

        // RANDOM in concat should be replaced
        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_literal_unchanged() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::Literal("hello".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => {
                assert!(matches!(value, BashExpr::Literal(s) if s == "hello"));
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_purify_glob_unchanged() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "files".to_string(),
                index: None,
                value: BashExpr::Glob("*.txt".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => {
                assert!(matches!(value, BashExpr::Glob(s) if s == "*.txt"));
            }
            _ => panic!("Expected assignment"),
        }
    }

    // ============== Default value expression tests ==============

    #[test]
    fn test_purify_default_value() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::DefaultValue {
                    variable: "FOO".to_string(),
                    default: Box::new(BashExpr::Literal("default".to_string())),
                },
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => {
                assert!(matches!(value, BashExpr::DefaultValue { .. }));
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_purify_default_value_with_non_deterministic_var() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::DefaultValue {
                    variable: "RANDOM".to_string(),
                    default: Box::new(BashExpr::Literal("0".to_string())),
                },
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_assign_default() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::AssignDefault {
                    variable: "RANDOM".to_string(),
                    default: Box::new(BashExpr::Literal("0".to_string())),
                },
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_error_if_unset() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::ErrorIfUnset {
                    variable: "RANDOM".to_string(),
                    message: Box::new(BashExpr::Literal("error".to_string())),
                },
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_alternative_value() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::AlternativeValue {
                    variable: "RANDOM".to_string(),
                    alternative: Box::new(BashExpr::Literal("alt".to_string())),
                },
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_string_length() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "len".to_string(),
                index: None,
                value: BashExpr::StringLength {
                    variable: "RANDOM".to_string(),
                },
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_remove_suffix() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::RemoveSuffix {
                    variable: "RANDOM".to_string(),
                    pattern: Box::new(BashExpr::Literal("*".to_string())),
                },
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_remove_prefix() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::RemovePrefix {
                    variable: "RANDOM".to_string(),
                    pattern: Box::new(BashExpr::Literal("*".to_string())),
                },
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_remove_longest_prefix() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::RemoveLongestPrefix {
                    variable: "RANDOM".to_string(),
                    pattern: Box::new(BashExpr::Literal("*".to_string())),
                },
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_remove_longest_suffix() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::RemoveLongestSuffix {
                    variable: "RANDOM".to_string(),
                    pattern: Box::new(BashExpr::Literal("*".to_string())),
                },
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_command_condition() {
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::CommandCondition(Box::new(BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![
                        BashExpr::Literal("-f".to_string()),
                        BashExpr::Literal("file".to_string()),
                    ],
                    redirects: vec![],
                    span: Span::dummy(),
                })),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("ok".to_string())],
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        assert!(matches!(&purified.statements[0], BashStmt::If { .. }));
    }

    // ============== Test expression purification tests ==============

    #[test]
    fn test_purify_test_all_comparison_types() {
        let tests = vec![
            TestExpr::StringEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("y".to_string()),
            ),
            TestExpr::StringNe(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("y".to_string()),
            ),
            TestExpr::IntEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            TestExpr::IntNe(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            TestExpr::IntLt(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            TestExpr::IntLe(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            TestExpr::IntGt(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            TestExpr::IntGe(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
        ];

        for test in tests {
            let ast = BashAst {
                statements: vec![BashStmt::If {
                    condition: BashExpr::Test(Box::new(test)),
                    then_block: vec![],
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

            let mut purifier = Purifier::new(PurificationOptions::default());
            let result = purifier.purify(&ast);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_purify_test_file_tests() {
        let tests = vec![
            TestExpr::FileExists(BashExpr::Literal("/tmp".to_string())),
            TestExpr::FileReadable(BashExpr::Literal("/tmp".to_string())),
            TestExpr::FileWritable(BashExpr::Literal("/tmp".to_string())),
            TestExpr::FileExecutable(BashExpr::Literal("/tmp".to_string())),
            TestExpr::FileDirectory(BashExpr::Literal("/tmp".to_string())),
        ];

        for test in tests {
            let ast = BashAst {
                statements: vec![BashStmt::If {
                    condition: BashExpr::Test(Box::new(test)),
                    then_block: vec![],
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

            let mut purifier = Purifier::new(PurificationOptions::default());
            let result = purifier.purify(&ast);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_purify_test_string_tests() {
        let tests = vec![
            TestExpr::StringEmpty(BashExpr::Variable("x".to_string())),
            TestExpr::StringNonEmpty(BashExpr::Variable("x".to_string())),
        ];

        for test in tests {
            let ast = BashAst {
                statements: vec![BashStmt::If {
                    condition: BashExpr::Test(Box::new(test)),
                    then_block: vec![],
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

            let mut purifier = Purifier::new(PurificationOptions::default());
            let result = purifier.purify(&ast);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_purify_test_logical_operators() {
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Test(Box::new(TestExpr::And(
                    Box::new(TestExpr::StringNonEmpty(BashExpr::Variable(
                        "x".to_string(),
                    ))),
                    Box::new(TestExpr::Or(
                        Box::new(TestExpr::IntGt(
                            BashExpr::Variable("y".to_string()),
                            BashExpr::Literal("0".to_string()),
                        )),
                        Box::new(TestExpr::Not(Box::new(TestExpr::FileExists(
                            BashExpr::Literal("/tmp".to_string()),
                        )))),
                    )),
                ))),
                then_block: vec![],
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let result = purifier.purify(&ast);
        assert!(result.is_ok());
    }

    // ============== Arithmetic purification tests ==============

    #[test]
    fn test_purify_arithmetic_all_operators() {
        let ops = vec![
            ArithExpr::Add(
                Box::new(ArithExpr::Number(1)),
                Box::new(ArithExpr::Number(2)),
            ),
            ArithExpr::Sub(
                Box::new(ArithExpr::Number(5)),
                Box::new(ArithExpr::Number(3)),
            ),
            ArithExpr::Mul(
                Box::new(ArithExpr::Number(2)),
                Box::new(ArithExpr::Number(3)),
            ),
            ArithExpr::Div(
                Box::new(ArithExpr::Number(6)),
                Box::new(ArithExpr::Number(2)),
            ),
            ArithExpr::Mod(
                Box::new(ArithExpr::Number(7)),
                Box::new(ArithExpr::Number(3)),
            ),
        ];

        for op in ops {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
                    index: None,
                    value: BashExpr::Arithmetic(Box::new(op)),
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let mut purifier = Purifier::new(PurificationOptions::default());
            let result = purifier.purify(&ast);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_purify_arithmetic_with_random_variable() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "result".to_string(),
                index: None,
                value: BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                    Box::new(ArithExpr::Variable("RANDOM".to_string())),
                    Box::new(ArithExpr::Number(1)),
                ))),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        // RANDOM should be replaced with 0
        assert!(!purifier.report().determinism_fixes.is_empty());

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => match value {
                BashExpr::Arithmetic(arith) => match arith.as_ref() {
                    ArithExpr::Add(left, _) => {
                        assert!(matches!(left.as_ref(), ArithExpr::Number(0)));
                    }
                    _ => panic!("Expected Add"),
                },
                _ => panic!("Expected Arithmetic"),
            },
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_purify_arithmetic_number_unchanged() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::Arithmetic(Box::new(ArithExpr::Number(42))),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => match value {
                BashExpr::Arithmetic(arith) => {
                    assert!(matches!(arith.as_ref(), ArithExpr::Number(42)));
                }
                _ => panic!("Expected Arithmetic"),
            },
            _ => panic!("Expected assignment"),
        }
    }

    // ============== Report accessor test ==============

    #[test]
    fn test_purifier_report_accessor() {
        let mut purifier = Purifier::new(PurificationOptions::default());

        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::Variable("RANDOM".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let _ = purifier.purify(&ast).unwrap();

        let report = purifier.report();
        assert!(!report.determinism_fixes.is_empty());
    }

    // ============== Exported assignment test ==============

    #[test]
    fn test_purify_exported_assignment() {
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

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        match &purified.statements[0] {
            BashStmt::Assignment { exported, .. } => {
                assert!(*exported);
            }
            _ => panic!("Expected assignment"),
        }
    }
