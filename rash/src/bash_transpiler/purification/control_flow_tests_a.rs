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
fn test_purify_control_flow_select_basic() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Select {
        variable: "opt".to_string(),
        items: BashExpr::Array(vec![
            BashExpr::Literal("yes".to_string()),
            BashExpr::Literal("no".to_string()),
            BashExpr::Literal("maybe".to_string()),
        ]),
        body: vec![echo_cmd("selected")],
        span: Span::new(40, 0, 45, 0),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::Select {
            variable,
            items,
            body,
            span,
        } => {
            assert_eq!(variable, "opt");
            assert!(matches!(items, BashExpr::Array(arr) if arr.len() == 3));
            assert_eq!(body.len(), 1);
            assert_eq!(span.start_line, 40);
        }
        other => panic!("Expected Select, got {:?}", other),
    }
}

// ===== 17. Select — preserves variable name =====

#[test]
fn test_purify_control_flow_select_preserves_variable() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Select {
        variable: "my_choice".to_string(),
        items: BashExpr::Literal("a b c".to_string()),
        body: vec![],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::Select { variable, body, .. } => {
            assert_eq!(variable, "my_choice");
            assert!(body.is_empty());
        }
        other => panic!("Expected Select, got {:?}", other),
    }
}

// ===== 18. Default branch — Comment passes through =====

#[test]
fn test_purify_control_flow_default_comment_passthrough() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Comment {
        text: "# this is a comment".to_string(),
        span: Span::new(50, 0, 50, 20),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::Comment { text, span } => {
            assert_eq!(text, "# this is a comment");
            assert_eq!(span.start_line, 50);
        }
        other => panic!("Expected Comment passthrough, got {:?}", other),
    }
}

// ===== 19. Default branch — Assignment passes through =====

#[test]
fn test_purify_control_flow_default_assignment_passthrough() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Assignment {
        name: "x".to_string(),
        index: None,
        value: BashExpr::Literal("42".to_string()),
        exported: false,
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    // The default branch clones the statement unchanged
    assert_eq!(result, stmt);
}

// ===== 20. Default branch — Command passes through =====

#[test]
fn test_purify_control_flow_default_command_passthrough() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Command {
        name: "ls".to_string(),
        args: vec![BashExpr::Literal("-la".to_string())],
        redirects: vec![],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    assert_eq!(result, stmt);
}

// ===== 21. If — purifies non-deterministic variable in condition =====

#[test]
fn test_purify_control_flow_if_nondet_condition() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::If {
        condition: BashExpr::Variable("RANDOM".to_string()),
        then_block: vec![echo_cmd("random path")],
        elif_blocks: vec![],
        else_block: None,
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::If { condition, .. } => {
            // RANDOM should be replaced with "0" (determinism fix)
            assert!(
                matches!(condition, BashExpr::Literal(ref s) if s == "0"),
                "RANDOM in condition should be replaced with literal 0, got {:?}",
                condition
            );
        }
        other => panic!("Expected If, got {:?}", other),
    }
}

// ===== 22. While — purifies non-deterministic variable in condition =====

#[test]
fn test_purify_control_flow_while_nondet_condition() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::While {
        condition: BashExpr::Variable("SECONDS".to_string()),
        body: vec![echo_cmd("tick")],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::While { condition, .. } => {
            assert!(
                matches!(condition, BashExpr::Literal(ref s) if s == "0"),
                "SECONDS in condition should be replaced with literal 0, got {:?}",
                condition
            );
        }
        other => panic!("Expected While, got {:?}", other),
    }
}

// ===== 23. For — purifies non-deterministic items expression =====

#[test]
fn test_purify_control_flow_for_nondet_items() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::For {
        variable: "val".to_string(),
        items: BashExpr::Variable("BASHPID".to_string()),
        body: vec![echo_cmd("item")],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::For { items, .. } => {
            assert!(
                matches!(items, BashExpr::Literal(ref s) if s == "0"),
                "BASHPID in for items should be replaced, got {:?}",
                items
            );
        }
        other => panic!("Expected For, got {:?}", other),
    }
}

// ===== 24. Case — purifies non-deterministic word expression =====

#[test]
fn test_purify_control_flow_case_nondet_word() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Case {
        word: BashExpr::Variable("PPID".to_string()),
        arms: vec![CaseArm {
            patterns: vec!["*".to_string()],
            body: vec![echo_cmd("catch-all")],
        }],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::Case { word, .. } => {
            assert!(
                matches!(word, BashExpr::Literal(ref s) if s == "0"),
                "PPID in case word should be replaced, got {:?}",
                word
            );
        }
        other => panic!("Expected Case, got {:?}", other),
    }
}

// ===== 25. If — purifies elif conditions and bodies =====

#[test]
fn test_purify_control_flow_if_purifies_elif_bodies() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::If {
        condition: BashExpr::Literal("true".to_string()),
        then_block: vec![echo_cmd("then")],
        elif_blocks: vec![(
            BashExpr::Variable("RANDOM".to_string()),
            vec![BashStmt::Assignment {
                name: "val".to_string(),
                index: None,
                value: BashExpr::Variable("SECONDS".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
        )],
        else_block: None,
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::If { elif_blocks, .. } => {
            assert_eq!(elif_blocks.len(), 1);
            // The elif condition (RANDOM) should be replaced
            let (elif_cond, elif_body) = &elif_blocks[0];
            assert!(
                matches!(elif_cond, BashExpr::Literal(s) if s == "0"),
                "RANDOM in elif condition should be replaced, got {:?}",
                elif_cond
            );
            // The elif body assignment value (SECONDS) should be replaced
            match &elif_body[0] {
                BashStmt::Assignment { value, .. } => {
                    assert!(
                        matches!(value, BashExpr::Literal(s) if s == "0"),
                        "SECONDS in elif body should be replaced, got {:?}",
                        value
                    );
                }
                other => panic!("Expected Assignment in elif body, got {:?}", other),
            }
        }
        other => panic!("Expected If, got {:?}", other),
    }
}

// ===== 26. If — purifies else body =====

#[test]
fn test_purify_control_flow_if_purifies_else_body() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::If {
        condition: BashExpr::Literal("cond".to_string()),
        then_block: vec![echo_cmd("then")],
        elif_blocks: vec![],
        else_block: Some(vec![BashStmt::Assignment {
            name: "r".to_string(),
            index: None,
            value: BashExpr::Variable("RANDOM".to_string()),
            exported: false,
            span: Span::dummy(),
        }]),
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::If { else_block, .. } => {
            let else_body = else_block.expect("else_block should be Some");
            match &else_body[0] {
                BashStmt::Assignment { value, .. } => {
                    assert!(
                        matches!(value, BashExpr::Literal(s) if s == "0"),
                        "RANDOM in else body should be replaced, got {:?}",
                        value
                    );
                }
                other => panic!("Expected Assignment in else body, got {:?}", other),
            }
        }
        other => panic!("Expected If, got {:?}", other),
    }
}

// ===== 27. Case — empty arms =====

#[test]
fn test_purify_control_flow_case_empty_arms() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Case {
        word: BashExpr::Literal("test".to_string()),
        arms: vec![],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::Case { arms, .. } => {
            assert!(arms.is_empty());
        }
        other => panic!("Expected Case, got {:?}", other),
    }
}

// ===== 28. Select — purifies non-deterministic items =====

#[test]
fn test_purify_control_flow_select_nondet_items() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Select {
        variable: "choice".to_string(),
        items: BashExpr::Variable("RANDOM".to_string()),
        body: vec![echo_cmd("chosen")],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::Select { items, .. } => {
            assert!(
                matches!(items, BashExpr::Literal(ref s) if s == "0"),
                "RANDOM in select items should be replaced, got {:?}",
                items
            );
        }
        other => panic!("Expected Select, got {:?}", other),
    }
}

// ===== 29. Default branch — Return passes through =====

#[test]
fn test_purify_control_flow_default_return_passthrough() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Return {
        code: Some(BashExpr::Literal("0".to_string())),
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    // Return is not a control flow variant handled by purify_control_flow,
    // so it falls through to the default branch which clones the statement
    assert_eq!(result, stmt);
}

// ===== 30. Default branch — Function passes through =====

#[test]
fn test_purify_control_flow_default_function_passthrough() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Function {
        name: "my_func".to_string(),
        body: vec![echo_cmd("inside function")],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    assert_eq!(result, stmt);
}

// ===== 31. ForCStyle — body with nested control flow =====

#[test]
fn test_purify_control_flow_for_c_style_nested_if_body() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::ForCStyle {
        init: "j=0".to_string(),
        condition: "j<3".to_string(),
        increment: "j++".to_string(),
        body: vec![BashStmt::If {
            condition: BashExpr::Literal("true".to_string()),
            then_block: vec![echo_cmd("inner")],
            elif_blocks: vec![],
            else_block: None,
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::ForCStyle { body, .. } => {
            assert_eq!(body.len(), 1);
            assert!(matches!(&body[0], BashStmt::If { .. }));
        }
        other => panic!("Expected ForCStyle, got {:?}", other),
    }
}

// ===== 32. Until — purifies condition with non-deterministic variable =====

#[test]
fn test_purify_control_flow_until_nondet_condition() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Until {
        condition: BashExpr::Variable("RANDOM".to_string()),
        body: vec![echo_cmd("retry")],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::Until { condition, .. } => {
            assert!(
                matches!(condition, BashExpr::Literal(ref s) if s == "0"),
                "RANDOM in until condition should be replaced, got {:?}",
                condition
            );
        }
        other => panic!("Expected Until, got {:?}", other),
    }
}

// ===== 33. Span is preserved through all control flow variants =====

