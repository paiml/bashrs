fn test_check_if_statement() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![BashStmt::If {
        condition: BashExpr::Test(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "/tmp".to_string(),
        )))),
        then_block: vec![BashStmt::Assignment {
            name: "found".to_string(),
            index: None,
            value: BashExpr::Literal("1".to_string()),
            exported: false,
            span: Span::dummy(),
        }],
        elif_blocks: vec![],
        else_block: None,
        span: Span::dummy(),
    }]);

    let diags = checker.check_ast(&ast);
    assert!(diags.is_empty());
    assert_eq!(checker.context().lookup("found"), Some(&ShellType::Integer));
}

#[test]
fn test_check_while_loop() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![BashStmt::While {
        condition: BashExpr::Test(Box::new(TestExpr::IntLt(
            BashExpr::Variable("i".to_string()),
            BashExpr::Literal("10".to_string()),
        ))),
        body: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::Variable("i".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    }]);

    let diags = checker.check_ast(&ast);
    assert!(diags.is_empty());
}

#[test]
fn test_check_pipeline() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![BashStmt::Pipeline {
        commands: vec![
            BashStmt::Command {
                name: "ls".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "grep".to_string(),
                args: vec![BashExpr::Literal("pattern".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            },
        ],
        span: Span::dummy(),
    }]);

    let diags = checker.check_ast(&ast);
    assert!(diags.is_empty());
}

#[test]
fn test_check_case_statement() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![BashStmt::Case {
        word: BashExpr::Variable("opt".to_string()),
        arms: vec![CaseArm {
            patterns: vec!["a".to_string()],
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("found a".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
        }],
        span: Span::dummy(),
    }]);

    let diags = checker.check_ast(&ast);
    assert!(diags.is_empty());
}

#[test]
fn test_integer_string_gradual_compatibility() {
    // Integer assigned to string-annotated variable should be OK (gradual)
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![
        BashStmt::Comment {
            text: " @type val: str".to_string(),
            span: Span::dummy(),
        },
        BashStmt::Assignment {
            name: "val".to_string(),
            index: None,
            value: BashExpr::Literal("42".to_string()),
            exported: false,
            span: Span::dummy(),
        },
    ]);

    let diags = checker.check_ast(&ast);
    // Integer → String is a gradual coercion, should not produce a mismatch
    assert!(diags.is_empty());
}

#[test]
fn test_infer_arithmetic_helper() {
    let checker = TypeChecker::new();
    let ty = checker.infer_arithmetic(&ArithExpr::Number(42));
    assert_eq!(ty, ShellType::Integer);
}

#[test]
fn test_infer_test_helper() {
    let checker = TypeChecker::new();
    let ty = checker.infer_test(&TestExpr::FileExists(BashExpr::Literal("/tmp".to_string())));
    assert_eq!(ty, ShellType::Boolean);
}

#[test]
fn test_default_type_checker() {
    let checker = TypeChecker::default();
    assert!(checker.diagnostics().is_empty());
}

#[test]
fn test_default_type_context() {
    let ctx = TypeContext::default();
    assert_eq!(ctx.scope_depth(), 1);
}

// ============================================================================
// StringInArithmetic Tests
// ============================================================================

#[test]
fn test_string_in_arithmetic_warns() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![
        BashStmt::Comment {
            text: " @type name: str".to_string(),
            span: Span::dummy(),
        },
        BashStmt::Assignment {
            name: "name".to_string(),
            index: None,
            value: BashExpr::Literal("hello".to_string()),
            exported: false,
            span: Span::dummy(),
        },
        BashStmt::Assignment {
            name: "result".to_string(),
            index: None,
            value: BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                Box::new(ArithExpr::Variable("name".to_string())),
                Box::new(ArithExpr::Number(1)),
            ))),
            exported: false,
            span: Span::dummy(),
        },
    ]);

    let diags = checker.check_ast(&ast);
    assert!(!diags.is_empty());
    assert!(matches!(
        diags[0].kind,
        DiagnosticKind::StringInArithmetic { .. }
    ));
}

#[test]
fn test_integer_in_arithmetic_no_warning() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![
        BashStmt::Comment {
            text: " @type count: int".to_string(),
            span: Span::dummy(),
        },
        BashStmt::Assignment {
            name: "count".to_string(),
            index: None,
            value: BashExpr::Literal("5".to_string()),
            exported: false,
            span: Span::dummy(),
        },
        BashStmt::Assignment {
            name: "result".to_string(),
            index: None,
            value: BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                Box::new(ArithExpr::Variable("count".to_string())),
                Box::new(ArithExpr::Number(1)),
            ))),
            exported: false,
            span: Span::dummy(),
        },
    ]);

    let diags = checker.check_ast(&ast);
    assert!(diags.is_empty());
}

// ============================================================================
// Annotation Hint Tests
// ============================================================================

#[test]
fn test_annotation_hint_preserved() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![
        BashStmt::Comment {
            text: " @type config: path".to_string(),
            span: Span::dummy(),
        },
        BashStmt::Assignment {
            name: "config".to_string(),
            index: None,
            value: BashExpr::Literal("/etc/app.conf".to_string()),
            exported: false,
            span: Span::dummy(),
        },
    ]);

    checker.check_ast(&ast);
    assert_eq!(checker.annotation_hint("config"), Some("path"));
}

#[test]
fn test_annotation_hint_missing_returns_none() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![BashStmt::Assignment {
        name: "x".to_string(),
        index: None,
        value: BashExpr::Literal("5".to_string()),
        exported: false,
        span: Span::dummy(),
    }]);

    checker.check_ast(&ast);
    assert_eq!(checker.annotation_hint("x"), None);
}

// ============================================================================
// Boolean Literal Inference Tests
// ============================================================================

#[test]
fn test_infer_true_as_boolean() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&BashExpr::Literal("true".to_string()));
    assert_eq!(ty, Some(ShellType::Boolean));
}

#[test]
fn test_infer_false_as_boolean() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&BashExpr::Literal("false".to_string()));
    assert_eq!(ty, Some(ShellType::Boolean));
}

#[test]
fn test_bool_annotation_with_true_literal_no_warning() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![
        BashStmt::Comment {
            text: " @type debug: bool".to_string(),
            span: Span::dummy(),
        },
        BashStmt::Assignment {
            name: "debug".to_string(),
            index: None,
            value: BashExpr::Literal("true".to_string()),
            exported: false,
            span: Span::dummy(),
        },
    ]);

    let diags = checker.check_ast(&ast);
    assert!(
        diags.is_empty(),
        "true should be compatible with bool annotation"
    );
}
