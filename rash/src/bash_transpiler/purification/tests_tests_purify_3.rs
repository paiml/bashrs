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

include!("tests_tests_purify_2.rs");
