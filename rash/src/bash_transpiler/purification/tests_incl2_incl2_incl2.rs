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

include!("tests_incl2_incl2_incl2_incl2.rs");
