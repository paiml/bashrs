//! Unit tests for purification/control_flow.rs — purify_control_flow method.
//!
//! Tests all 7 control flow variants (If, While, Until, For, ForCStyle, Case, Select)
//! plus the default fallthrough branch for non-control-flow statements.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::bash_parser::ast::*;

/// Construct a Purifier with default options.
fn make_purifier() -> Purifier {
    Purifier::new(PurificationOptions::default())
}

/// Helper: echo command used as a simple body statement.
fn echo_cmd(msg: &str) -> BashStmt {
    BashStmt::Command {
        name: "echo".to_string(),
        args: vec![BashExpr::Literal(msg.to_string())],
        redirects: vec![],
        span: Span::dummy(),
    }
}

// ===== 1. If — basic then-block =====

#[test]
fn test_purify_control_flow_if_basic() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::If {
        condition: BashExpr::Literal("true".to_string()),
        then_block: vec![echo_cmd("hello")],
        elif_blocks: vec![],
        else_block: None,
        span: Span::new(1, 0, 3, 0),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::If {
            condition,
            then_block,
            elif_blocks,
            else_block,
            span,
        } => {
            assert!(matches!(condition, BashExpr::Literal(s) if s == "true"));
            assert_eq!(then_block.len(), 1);
            assert!(elif_blocks.is_empty());
            assert!(else_block.is_none());
            assert_eq!(span.start_line, 1);
        }
        other => panic!("Expected If, got {:?}", other),
    }
}

// ===== 2. If — with elif blocks =====

#[test]
fn test_purify_control_flow_if_with_elif() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::If {
        condition: BashExpr::Test(Box::new(TestExpr::IntEq(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("1".to_string()),
        ))),
        then_block: vec![echo_cmd("one")],
        elif_blocks: vec![
            (
                BashExpr::Test(Box::new(TestExpr::IntEq(
                    BashExpr::Variable("x".to_string()),
                    BashExpr::Literal("2".to_string()),
                ))),
                vec![echo_cmd("two")],
            ),
            (
                BashExpr::Test(Box::new(TestExpr::IntEq(
                    BashExpr::Variable("x".to_string()),
                    BashExpr::Literal("3".to_string()),
                ))),
                vec![echo_cmd("three")],
            ),
        ],
        else_block: None,
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::If {
            elif_blocks,
            else_block,
            ..
        } => {
            assert_eq!(elif_blocks.len(), 2, "Should preserve both elif blocks");
            assert!(else_block.is_none());
        }
        other => panic!("Expected If, got {:?}", other),
    }
}

// ===== 3. If — with else block =====

#[test]
fn test_purify_control_flow_if_with_else() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::If {
        condition: BashExpr::Literal("false".to_string()),
        then_block: vec![echo_cmd("then")],
        elif_blocks: vec![],
        else_block: Some(vec![echo_cmd("else")]),
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::If { else_block, .. } => {
            let else_body = else_block.expect("else_block should be Some");
            assert_eq!(else_body.len(), 1);
        }
        other => panic!("Expected If, got {:?}", other),
    }
}

// ===== 4. If — with elif AND else =====

#[test]
fn test_purify_control_flow_if_elif_and_else() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::If {
        condition: BashExpr::Literal("cond1".to_string()),
        then_block: vec![echo_cmd("branch1")],
        elif_blocks: vec![(
            BashExpr::Literal("cond2".to_string()),
            vec![echo_cmd("branch2")],
        )],
        else_block: Some(vec![echo_cmd("fallback")]),
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::If {
            then_block,
            elif_blocks,
            else_block,
            ..
        } => {
            assert_eq!(then_block.len(), 1);
            assert_eq!(elif_blocks.len(), 1);
            assert!(else_block.is_some());
            assert_eq!(else_block.unwrap().len(), 1);
        }
        other => panic!("Expected If, got {:?}", other),
    }
}

// ===== 5. While — basic =====

#[test]
fn test_purify_control_flow_while_basic() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::While {
        condition: BashExpr::Test(Box::new(TestExpr::IntLt(
            BashExpr::Variable("i".to_string()),
            BashExpr::Literal("10".to_string()),
        ))),
        body: vec![echo_cmd("looping")],
        span: Span::new(5, 0, 8, 0),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::While {
            condition,
            body,
            span,
        } => {
            assert!(matches!(condition, BashExpr::Test(_)));
            assert_eq!(body.len(), 1);
            assert_eq!(span.start_line, 5);
        }
        other => panic!("Expected While, got {:?}", other),
    }
}

// ===== 6. While — with multiple body statements =====

#[test]
fn test_purify_control_flow_while_multi_body() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::While {
        condition: BashExpr::Literal("true".to_string()),
        body: vec![echo_cmd("step1"), echo_cmd("step2"), echo_cmd("step3")],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::While { body, .. } => {
            assert_eq!(body.len(), 3, "All body statements should be preserved");
        }
        other => panic!("Expected While, got {:?}", other),
    }
}

// ===== 7. Until — basic =====

#[test]
fn test_purify_control_flow_until_basic() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Until {
        condition: BashExpr::Test(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "/tmp/done".to_string(),
        )))),
        body: vec![echo_cmd("waiting")],
        span: Span::new(10, 0, 13, 0),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::Until {
            condition,
            body,
            span,
        } => {
            assert!(matches!(condition, BashExpr::Test(_)));
            assert_eq!(body.len(), 1);
            assert_eq!(span.start_line, 10);
        }
        other => panic!("Expected Until, got {:?}", other),
    }
}

// ===== 8. Until — with empty body =====

#[test]
fn test_purify_control_flow_until_empty_body() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Until {
        condition: BashExpr::Literal("false".to_string()),
        body: vec![],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::Until { body, .. } => {
            assert!(body.is_empty(), "Empty body should stay empty");
        }
        other => panic!("Expected Until, got {:?}", other),
    }
}

// ===== 9. For — basic =====

#[test]
fn test_purify_control_flow_for_basic() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::For {
        variable: "item".to_string(),
        items: BashExpr::Array(vec![
            BashExpr::Literal("a".to_string()),
            BashExpr::Literal("b".to_string()),
            BashExpr::Literal("c".to_string()),
        ]),
        body: vec![echo_cmd("processing")],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::For {
            variable,
            items,
            body,
            ..
        } => {
            assert_eq!(variable, "item");
            assert!(matches!(items, BashExpr::Array(arr) if arr.len() == 3));
            assert_eq!(body.len(), 1);
        }
        other => panic!("Expected For, got {:?}", other),
    }
}

// ===== 10. For — preserves variable name and span =====

#[test]
fn test_purify_control_flow_for_preserves_variable_and_span() {
    let mut purifier = make_purifier();
    let span = Span::new(20, 4, 25, 0);
    let stmt = BashStmt::For {
        variable: "file".to_string(),
        items: BashExpr::Glob("*.txt".to_string()),
        body: vec![echo_cmd("found")],
        span,
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::For {
            variable,
            items,
            span: result_span,
            ..
        } => {
            assert_eq!(variable, "file");
            assert!(matches!(items, BashExpr::Glob(s) if s == "*.txt"));
            assert_eq!(result_span, span);
        }
        other => panic!("Expected For, got {:?}", other),
    }
}

// ===== 11. ForCStyle — basic =====

#[test]
fn test_purify_control_flow_for_c_style_basic() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::ForCStyle {
        init: "i=0".to_string(),
        condition: "i<10".to_string(),
        increment: "i++".to_string(),
        body: vec![echo_cmd("iteration")],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::ForCStyle {
            init,
            condition,
            increment,
            body,
            ..
        } => {
            assert_eq!(init, "i=0");
            assert_eq!(condition, "i<10");
            assert_eq!(increment, "i++");
            assert_eq!(body.len(), 1);
        }
        other => panic!("Expected ForCStyle, got {:?}", other),
    }
}

// ===== 12. ForCStyle — init/condition/increment preserved as-is =====

#[test]

include!("control_flow_tests_tests_purify_2.rs");
