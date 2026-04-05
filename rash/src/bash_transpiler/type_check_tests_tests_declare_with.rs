fn test_declare_with_assignment() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![BashStmt::Command {
        name: "declare".to_string(),
        args: vec![
            BashExpr::Literal("-i".to_string()),
            BashExpr::Literal("count=0".to_string()),
        ],
        redirects: vec![],
        span: Span::dummy(),
    }]);

    checker.check_ast(&ast);
    assert_eq!(checker.context().lookup("count"), Some(&ShellType::Integer));
}

// ============================================================================
// declare -i name=value Integration Tests
// ============================================================================

#[test]
fn test_declare_i_name_equals_value_tracks_type() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![BashStmt::Command {
        name: "declare".to_string(),
        args: vec![
            BashExpr::Literal("-i".to_string()),
            BashExpr::Literal("counter=0".to_string()),
        ],
        redirects: vec![],
        span: Span::dummy(),
    }]);

    checker.check_ast(&ast);
    assert_eq!(
        checker.context().lookup("counter"),
        Some(&ShellType::Integer)
    );
}

#[test]
fn test_declare_i_then_string_assign_warns() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![
        BashStmt::Command {
            name: "declare".to_string(),
            args: vec![
                BashExpr::Literal("-i".to_string()),
                BashExpr::Literal("counter=0".to_string()),
            ],
            redirects: vec![],
            span: Span::dummy(),
        },
        BashStmt::Assignment {
            name: "counter".to_string(),
            index: None,
            value: BashExpr::Literal("not_a_number".to_string()),
            exported: false,
            span: Span::new(3, 0, 3, 0),
        },
    ]);

    let diags = checker.check_ast(&ast);
    assert!(
        !diags.is_empty(),
        "string assigned to declare -i var should warn"
    );
    assert!(matches!(diags[0].kind, DiagnosticKind::TypeMismatch { .. }));
}

// ============================================================================
// Type Annotation + Assignment Integration Tests
// ============================================================================

#[test]
fn test_comment_annotation_sets_variable_type() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![
        BashStmt::Comment {
            text: " @type port: int".to_string(),
            span: Span::dummy(),
        },
        BashStmt::Assignment {
            name: "port".to_string(),
            index: None,
            value: BashExpr::Literal("8080".to_string()),
            exported: false,
            span: Span::dummy(),
        },
    ]);

    checker.check_ast(&ast);
    assert_eq!(checker.context().lookup("port"), Some(&ShellType::Integer));
}

#[test]
fn test_annotation_mismatch_produces_warning() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![
        BashStmt::Comment {
            text: " @type port: int".to_string(),
            span: Span::dummy(),
        },
        BashStmt::Assignment {
            name: "port".to_string(),
            index: None,
            value: BashExpr::Array(vec![BashExpr::Literal("a".to_string())]),
            exported: false,
            span: Span::new(5, 0, 5, 20),
        },
    ]);

    let diags = checker.check_ast(&ast);
    assert!(!diags.is_empty());
    assert!(matches!(diags[0].kind, DiagnosticKind::TypeMismatch { .. }));
    assert_eq!(diags[0].severity, Severity::Warning);
}

#[test]
fn test_annotation_compatible_no_diagnostic() {
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
    ]);

    let diags = checker.check_ast(&ast);
    assert!(diags.is_empty());
}

// ============================================================================
// Gradual Typing Tests — Untyped Variables Produce No Errors
// ============================================================================

#[test]
fn test_gradual_untyped_variable_no_error() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![BashStmt::Assignment {
        name: "x".to_string(),
        index: None,
        value: BashExpr::Variable("y".to_string()),
        exported: false,
        span: Span::dummy(),
    }]);

    let diags = checker.check_ast(&ast);
    assert!(
        diags.is_empty(),
        "gradual typing: untyped var should not produce errors"
    );
}

#[test]
fn test_gradual_fully_untyped_script() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![
        BashStmt::Assignment {
            name: "x".to_string(),
            index: None,
            value: BashExpr::Literal("hello".to_string()),
            exported: false,
            span: Span::dummy(),
        },
        BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::Variable("x".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        },
    ]);

    let diags = checker.check_ast(&ast);
    assert!(
        diags.is_empty(),
        "fully untyped script should produce no diagnostics"
    );
}

// ============================================================================
// Function Signature Tests
// ============================================================================

#[test]
fn test_function_param_annotation() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![
        BashStmt::Comment {
            text: " @param port: int".to_string(),
            span: Span::dummy(),
        },
        BashStmt::Comment {
            text: " @returns: exit_code".to_string(),
            span: Span::dummy(),
        },
        BashStmt::Function {
            name: "start_server".to_string(),
            body: vec![],
            span: Span::dummy(),
        },
    ]);

    checker.check_ast(&ast);
    let sig = checker.context().lookup_function("start_server").unwrap();
    assert_eq!(sig.params.len(), 1);
    assert_eq!(sig.params[0].0, "port");
    assert_eq!(sig.params[0].1, ShellType::Integer);
    assert_eq!(sig.return_type, Some(ShellType::ExitCode));
}

#[test]
fn test_function_scope_isolation() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![BashStmt::Function {
        name: "myfunc".to_string(),
        body: vec![BashStmt::Assignment {
            name: "local_var".to_string(),
            index: None,
            value: BashExpr::Literal("42".to_string()),
            exported: false,
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    }]);

    checker.check_ast(&ast);
    // local_var should not be visible in the outer scope
    assert_eq!(checker.context().lookup("local_var"), None);
}

// ============================================================================
// Guard Generation Tests
// ============================================================================

#[test]
fn test_generate_integer_guard() {
    let guard = generate_integer_guard("port");
    assert!(guard.contains("port"));
    assert!(guard.contains("*[!0-9]*"));
    assert!(guard.contains("type error"));
    assert!(guard.contains("exit 1"));
}

#[test]
fn test_generate_path_guard() {
    let guard = generate_path_guard("config_path");
    assert!(guard.contains("config_path"));
    assert!(guard.contains("/*|./*|../*"));
    assert!(guard.contains("type error"));
}

#[test]
fn test_generate_nonempty_guard() {
    let guard = generate_nonempty_guard("name");
    assert!(guard.contains("name"));
    assert!(guard.contains("-z"));
    assert!(guard.contains("type error"));
}

#[test]
fn test_generate_guard_for_integer_type() {
    let guard = generate_guard_for_type("x", &ShellType::Integer, None);
    assert!(guard.is_some());
    assert!(guard.unwrap().contains("*[!0-9]*"));
}

#[test]
fn test_generate_guard_for_string_type() {
    let guard = generate_guard_for_type("x", &ShellType::String, None);
    assert!(guard.is_some());
    assert!(guard.unwrap().contains("-z"));
}

#[test]
fn test_generate_guard_for_path_type() {
    let guard = generate_guard_for_type("x", &ShellType::String, Some("path"));
    assert!(guard.is_some());
    let g = guard.unwrap();
    assert!(g.contains("/*|./*|../*"));
    assert!(g.contains("type error: x must be a path"));
}

#[test]
fn test_generate_guard_for_boolean_type() {
    let guard = generate_guard_for_type("x", &ShellType::Boolean, None);
    assert!(guard.is_none());
}

// ============================================================================
// Diagnostic Display Tests
// ============================================================================

#[test]
fn test_diagnostic_kind_display() {
    let kind = DiagnosticKind::TypeMismatch {
        expected: ShellType::Integer,
        actual: ShellType::String,
    };
    let display = format!("{}", kind);
    assert!(display.contains("integer"));
    assert!(display.contains("string"));
}

#[test]
fn test_severity_display() {
    assert_eq!(format!("{}", Severity::Error), "error");
    assert_eq!(format!("{}", Severity::Warning), "warning");
    assert_eq!(format!("{}", Severity::Info), "info");
}

#[test]
fn test_diagnostic_display() {
    let diag = TypeDiagnostic {
        span: Span::new(10, 5, 10, 20),
        kind: DiagnosticKind::UndeclaredVariable {
            name: "x".to_string(),
        },
        severity: Severity::Warning,
        message: "variable x is undeclared".to_string(),
    };
    let display = format!("{}", diag);
    assert!(display.contains("10"));
    assert!(display.contains("warning"));
}

// ============================================================================
// parse_type_name Tests
// ============================================================================

#[test]
fn test_parse_type_name_all_variants() {
    assert_eq!(parse_type_name("int"), Some(ShellType::Integer));
    assert_eq!(parse_type_name("integer"), Some(ShellType::Integer));
    assert_eq!(parse_type_name("str"), Some(ShellType::String));
    assert_eq!(parse_type_name("string"), Some(ShellType::String));
    assert_eq!(parse_type_name("bool"), Some(ShellType::Boolean));
    assert_eq!(parse_type_name("boolean"), Some(ShellType::Boolean));
    assert_eq!(parse_type_name("path"), Some(ShellType::String)); // path subtype
    assert_eq!(parse_type_name("fd"), Some(ShellType::FileDescriptor));
    assert_eq!(parse_type_name("exit_code"), Some(ShellType::ExitCode));
    assert_eq!(
        parse_type_name("array"),
        Some(ShellType::Array(Box::new(ShellType::String)))
    );
    assert_eq!(parse_type_name("nonexistent"), None);
}

// ============================================================================
// Complex AST Walk Tests
// ============================================================================

#[test]

include!("type_check_tests_tests_check_if.rs");
