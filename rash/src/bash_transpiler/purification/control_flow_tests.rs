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
