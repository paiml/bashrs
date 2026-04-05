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
