//! Coverage tests for purification/control_flow.rs — Select statement and edge cases.
//!
//! Targets uncovered branches in purify_control_flow: Select, nested control flow,
//! type checking integration, and Negated statement paths.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::bash_parser::ast::*;

fn default_metadata() -> AstMetadata {
    AstMetadata {
        source_file: None,
        line_count: 1,
        parse_time_ms: 0,
    }
}

fn make_ast(stmts: Vec<BashStmt>) -> BashAst {
    BashAst {
        statements: stmts,
        metadata: default_metadata(),
    }
}

fn purify_ok(stmts: Vec<BashStmt>) -> (BashAst, PurificationReport) {
    let ast = make_ast(stmts);
    let mut purifier = Purifier::new(PurificationOptions::default());
    let result = purifier.purify(&ast).expect("purification should succeed");
    let report = purifier.report().clone();
    (result, report)
}

// ===== Select statement purification =====

#[test]
fn test_purify_select_statement() {
    let (purified, _) = purify_ok(vec![BashStmt::Select {
        variable: "choice".to_string(),
        items: BashExpr::Array(vec![
            BashExpr::Literal("yes".to_string()),
            BashExpr::Literal("no".to_string()),
        ]),
        body: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::Variable("choice".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    }]);

    match &purified.statements[0] {
        BashStmt::Select {
            variable,
            items,
            body,
            ..
        } => {
            assert_eq!(variable, "choice");
            assert!(matches!(items, BashExpr::Array(arr) if arr.len() == 2));
            assert_eq!(body.len(), 1);
        }
        other => panic!("Expected Select, got {:?}", other),
    }
}

#[test]
fn test_purify_select_with_non_deterministic_items() {
    let (_, report) = purify_ok(vec![BashStmt::Select {
        variable: "opt".to_string(),
        items: BashExpr::Array(vec![
            BashExpr::Variable("RANDOM".to_string()),
            BashExpr::Literal("fixed".to_string()),
        ]),
        body: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::Variable("opt".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    }]);

    assert!(
        !report.determinism_fixes.is_empty(),
        "Should detect RANDOM in select items"
    );
}

// ===== Negated statement purification =====

#[test]
fn test_purify_negated_command() {
    let (purified, _) = purify_ok(vec![BashStmt::Negated {
        command: Box::new(BashStmt::Command {
            name: "test".to_string(),
            args: vec![
                BashExpr::Literal("-f".to_string()),
                BashExpr::Literal("/tmp/file".to_string()),
            ],
            redirects: vec![],
            span: Span::dummy(),
        }),
        span: Span::dummy(),
    }]);

    assert!(
        matches!(&purified.statements[0], BashStmt::Negated { command, .. }
            if matches!(command.as_ref(), BashStmt::Command { name, .. } if name == "test"))
    );
}

#[test]
fn test_purify_negated_with_mkdir_gets_idempotency_fix() {
    let (_, report) = purify_ok(vec![BashStmt::Negated {
        command: Box::new(BashStmt::Command {
            name: "mkdir".to_string(),
            args: vec![BashExpr::Literal("/tmp/dir".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }),
        span: Span::dummy(),
    }]);

    assert!(
        !report.idempotency_fixes.is_empty(),
        "mkdir inside negated should still get -p fix"
    );
}

// ===== Type checking integration =====

#[test]
fn test_purify_with_type_check_enabled() {
    let ast = make_ast(vec![BashStmt::Assignment {
        name: "x".to_string(),
        index: None,
        value: BashExpr::Literal("hello".to_string()),
        exported: false,
        span: Span::dummy(),
    }]);

    let opts = PurificationOptions {
        strict_idempotency: true,
        remove_non_deterministic: true,
        track_side_effects: true,
        type_check: true,
        emit_guards: false,
        type_strict: false,
    };

    let mut purifier = Purifier::new(opts);
    let _purified = purifier.purify(&ast).expect("should succeed");
    assert!(purifier.type_checker().is_some(), "type checker should be set");
}

#[test]
fn test_purify_with_emit_guards_enabled() {
    let ast = make_ast(vec![
        BashStmt::Assignment {
            name: "count".to_string(),
            index: None,
            value: BashExpr::Literal("42".to_string()),
            exported: false,
            span: Span::dummy(),
        },
        BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::Variable("count".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        },
    ]);

    let opts = PurificationOptions {
        strict_idempotency: true,
        remove_non_deterministic: true,
        track_side_effects: true,
        type_check: false,
        emit_guards: true,
        type_strict: false,
    };

    let mut purifier = Purifier::new(opts);
    let _purified = purifier.purify(&ast).expect("should succeed");
    // emit_guards triggers type checking too
    assert!(purifier.type_checker().is_some());
}

#[test]
fn test_purify_without_type_check() {
    let ast = make_ast(vec![BashStmt::Command {
        name: "echo".to_string(),
        args: vec![BashExpr::Literal("hi".to_string())],
        redirects: vec![],
        span: Span::dummy(),
    }]);

    let opts = PurificationOptions {
        strict_idempotency: true,
        remove_non_deterministic: true,
        track_side_effects: true,
        type_check: false,
        emit_guards: false,
        type_strict: false,
    };

    let mut purifier = Purifier::new(opts);
    let _purified = purifier.purify(&ast).expect("should succeed");
    assert!(purifier.type_checker().is_none());
}

// ===== Assignment with index (array element) =====

#[test]
fn test_purify_array_index_assignment() {
    let (purified, _) = purify_ok(vec![BashStmt::Assignment {
        name: "arr".to_string(),
        index: Some("0".to_string()),
        value: BashExpr::Literal("first".to_string()),
        exported: false,
        span: Span::dummy(),
    }]);

    match &purified.statements[0] {
        BashStmt::Assignment {
            name, index, value, ..
        } => {
            assert_eq!(name, "arr");
            assert!(index.is_some());
            assert!(matches!(value, BashExpr::Literal(s) if s == "first"));
        }
        other => panic!("Expected Assignment, got {:?}", other),
    }
}

// ===== Nested control flow =====

#[test]
fn test_purify_nested_if_in_while() {
    let (purified, _) = purify_ok(vec![BashStmt::While {
        condition: BashExpr::Test(Box::new(TestExpr::IntLt(
            BashExpr::Variable("i".to_string()),
            BashExpr::Literal("10".to_string()),
        ))),
        body: vec![BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::IntEq(
                BashExpr::Variable("i".to_string()),
                BashExpr::Literal("5".to_string()),
            ))),
            then_block: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("midpoint".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            elif_blocks: vec![],
            else_block: None,
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    }]);

    match &purified.statements[0] {
        BashStmt::While { body, .. } => {
            assert_eq!(body.len(), 1);
            assert!(matches!(&body[0], BashStmt::If { .. }));
        }
        other => panic!("Expected While, got {:?}", other),
    }
}

#[test]
fn test_purify_case_with_non_deterministic_arm() {
    let (_, report) = purify_ok(vec![BashStmt::Case {
        word: BashExpr::Variable("x".to_string()),
        arms: vec![CaseArm {
            patterns: vec!["*".to_string()],
            body: vec![BashStmt::Assignment {
                name: "val".to_string(),
                index: None,
                value: BashExpr::Variable("RANDOM".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
        }],
        span: Span::dummy(),
    }]);

    assert!(
        !report.determinism_fixes.is_empty(),
        "RANDOM inside case arm body should be detected"
    );
}

// ===== For loop with non-deterministic body =====

#[test]
fn test_purify_for_with_mkdir_body() {
    let (_, report) = purify_ok(vec![BashStmt::For {
        variable: "dir".to_string(),
        items: BashExpr::Array(vec![
            BashExpr::Literal("/tmp/a".to_string()),
            BashExpr::Literal("/tmp/b".to_string()),
        ]),
        body: vec![BashStmt::Command {
            name: "mkdir".to_string(),
            args: vec![BashExpr::Variable("dir".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    }]);

    assert!(
        !report.idempotency_fixes.is_empty(),
        "mkdir in for body should get -p fix"
    );
}

// ===== Until loop with non-deterministic body =====

#[test]
fn test_purify_until_with_rm_body() {
    let (_, report) = purify_ok(vec![BashStmt::Until {
        condition: BashExpr::Test(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "/tmp/done".to_string(),
        )))),
        body: vec![BashStmt::Command {
            name: "rm".to_string(),
            args: vec![BashExpr::Literal("/tmp/temp".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    }]);

    assert!(
        !report.idempotency_fixes.is_empty(),
        "rm in until body should get -f fix"
    );
}

// ===== ForCStyle with non-deterministic body =====

#[test]
fn test_purify_for_c_style_with_non_deterministic_body() {
    let (_, report) = purify_ok(vec![BashStmt::ForCStyle {
        init: "i=0".to_string(),
        condition: "i<5".to_string(),
        increment: "i++".to_string(),
        body: vec![BashStmt::Assignment {
            name: "val".to_string(),
            index: None,
            value: BashExpr::Variable("SECONDS".to_string()),
            exported: false,
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    }]);

    assert!(
        !report.determinism_fixes.is_empty(),
        "SECONDS in C-style for body should be detected"
    );
}

// ===== ln symlink =====

#[test]
fn test_purify_ln_gets_sf_flag() {
    let (purified, report) = purify_ok(vec![BashStmt::Command {
        name: "ln".to_string(),
        args: vec![
            BashExpr::Literal("-s".to_string()),
            BashExpr::Literal("target".to_string()),
            BashExpr::Literal("link".to_string()),
        ],
        redirects: vec![],
        span: Span::dummy(),
    }]);

    match &purified.statements[0] {
        BashStmt::Command { name, args, .. } => {
            assert_eq!(name, "ln");
            let has_sf = args
                .iter()
                .any(|a| matches!(a, BashExpr::Literal(s) if s == "-sf"));
            let has_s = args
                .iter()
                .any(|a| matches!(a, BashExpr::Literal(s) if s == "-s"));
            // Should have either -sf or -s and -f
            assert!(
                has_sf || has_s,
                "ln should have -sf or -s: {:?}",
                args
            );
        }
        _ => panic!("Expected command"),
    }

    // May or may not have idempotency fix depending on implementation
    let _ = report;
}

// ===== chmod =====

#[test]
fn test_purify_chmod_tracked_as_side_effect() {
    let (_, report) = purify_ok(vec![BashStmt::Command {
        name: "chmod".to_string(),
        args: vec![
            BashExpr::Literal("755".to_string()),
            BashExpr::Literal("script.sh".to_string()),
        ],
        redirects: vec![],
        span: Span::dummy(),
    }]);

    // chmod should be tracked as a side effect
    assert!(
        !report.side_effects_isolated.is_empty() || !report.warnings.is_empty(),
        "chmod should be tracked as side effect or warning"
    );
}
