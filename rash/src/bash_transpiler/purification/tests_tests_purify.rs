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
