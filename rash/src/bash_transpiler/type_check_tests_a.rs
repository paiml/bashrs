#![allow(clippy::unwrap_used)]

use super::*;
use crate::bash_parser::ast::*;

// ============================================================================
// TypeAnnotation Parsing Tests
// ============================================================================

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
