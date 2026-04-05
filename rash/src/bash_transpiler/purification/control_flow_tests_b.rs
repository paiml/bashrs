//! Unit tests for purification/control_flow.rs — purify_control_flow method.
//!
//! Tests all 7 control flow variants (If, While, Until, For, ForCStyle, Case, Select)
//! plus the default fallthrough branch for non-control-flow statements.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::bash_parser::ast::*;

/// Construct a Purifier with default options.
#[test]
fn test_purify_control_flow_span_preservation() {
    let mut purifier = make_purifier();
    let original_span = Span::new(99, 5, 120, 10);

    // Test span preservation for While
    let while_stmt = BashStmt::While {
        condition: BashExpr::Literal("true".to_string()),
        body: vec![],
        span: original_span,
    };
    let result = purifier
        .purify_control_flow(&while_stmt)
        .expect("should succeed");
    match result {
        BashStmt::While { span, .. } => assert_eq!(span, original_span),
        _ => panic!("Expected While"),
    }

    // Test span preservation for Until
    let until_stmt = BashStmt::Until {
        condition: BashExpr::Literal("false".to_string()),
        body: vec![],
        span: original_span,
    };
    let result = purifier
        .purify_control_flow(&until_stmt)
        .expect("should succeed");
    match result {
        BashStmt::Until { span, .. } => assert_eq!(span, original_span),
        _ => panic!("Expected Until"),
    }

    // Test span preservation for Case
    let case_stmt = BashStmt::Case {
        word: BashExpr::Literal("x".to_string()),
        arms: vec![],
        span: original_span,
    };
    let result = purifier
        .purify_control_flow(&case_stmt)
        .expect("should succeed");
    match result {
        BashStmt::Case { span, .. } => assert_eq!(span, original_span),
        _ => panic!("Expected Case"),
    }
}
