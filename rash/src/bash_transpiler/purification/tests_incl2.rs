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

include!("tests_incl2_incl2.rs");
