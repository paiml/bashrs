fn test_purify_control_flow_for_c_style_preserves_strings() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::ForCStyle {
        init: "x=100".to_string(),
        condition: "x>0".to_string(),
        increment: "x-=10".to_string(),
        body: vec![],
        span: Span::new(30, 0, 35, 0),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::ForCStyle {
            init,
            condition,
            increment,
            body,
            span,
        } => {
            assert_eq!(init, "x=100");
            assert_eq!(condition, "x>0");
            assert_eq!(increment, "x-=10");
            assert!(body.is_empty());
            assert_eq!(span.start_line, 30);
        }
        other => panic!("Expected ForCStyle, got {:?}", other),
    }
}

// ===== 13. Case — basic with one arm =====

#[test]
fn test_purify_control_flow_case_basic() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Case {
        word: BashExpr::Variable("action".to_string()),
        arms: vec![CaseArm {
            patterns: vec!["start".to_string()],
            body: vec![echo_cmd("starting")],
        }],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::Case {
            word, arms, span, ..
        } => {
            assert!(matches!(word, BashExpr::Variable(v) if v == "action"));
            assert_eq!(arms.len(), 1);
            assert_eq!(arms[0].patterns, vec!["start".to_string()]);
            assert_eq!(arms[0].body.len(), 1);
            assert_eq!(span, Span::dummy());
        }
        other => panic!("Expected Case, got {:?}", other),
    }
}

// ===== 14. Case — multiple arms with multiple patterns =====

#[test]
fn test_purify_control_flow_case_multiple_arms() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Case {
        word: BashExpr::Variable("ext".to_string()),
        arms: vec![
            CaseArm {
                patterns: vec!["*.txt".to_string(), "*.md".to_string()],
                body: vec![echo_cmd("text")],
            },
            CaseArm {
                patterns: vec!["*.rs".to_string()],
                body: vec![echo_cmd("rust")],
            },
            CaseArm {
                patterns: vec!["*".to_string()],
                body: vec![echo_cmd("unknown")],
            },
        ],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::Case { arms, .. } => {
            assert_eq!(arms.len(), 3);
            assert_eq!(arms[0].patterns.len(), 2);
            assert_eq!(arms[1].patterns, vec!["*.rs".to_string()]);
            assert_eq!(arms[2].patterns, vec!["*".to_string()]);
        }
        other => panic!("Expected Case, got {:?}", other),
    }
}

// ===== 15. Case — arm patterns are cloned, not purified =====

#[test]
fn test_purify_control_flow_case_patterns_cloned() {
    let mut purifier = make_purifier();
    let stmt = BashStmt::Case {
        word: BashExpr::Literal("hello".to_string()),
        arms: vec![CaseArm {
            patterns: vec!["h*".to_string(), "H*".to_string()],
            body: vec![],
        }],
        span: Span::dummy(),
    };

    let result = purifier.purify_control_flow(&stmt).expect("should succeed");

    match result {
        BashStmt::Case { arms, .. } => {
            assert_eq!(arms[0].patterns, vec!["h*".to_string(), "H*".to_string()]);
            assert!(arms[0].body.is_empty());
        }
        other => panic!("Expected Case, got {:?}", other),
    }
}

// ===== 16. Select — basic =====

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

include!("control_flow_tests_tests_purify.rs");
