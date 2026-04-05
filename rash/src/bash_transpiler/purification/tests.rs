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

include!("tests_incl2.rs");
