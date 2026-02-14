#![allow(clippy::unwrap_used)]

use super::*;
use crate::bash_parser::ast::*;

// ============================================================================
// TypeAnnotation Parsing Tests
// ============================================================================

#[test]
fn test_parse_type_annotation_int() {
    let ann = parse_type_annotation(" @type port: int").unwrap();
    assert_eq!(ann.name, "port");
    assert_eq!(ann.shell_type, ShellType::Integer);
    assert!(!ann.is_return);
    assert!(!ann.is_param);
}

#[test]
fn test_parse_type_annotation_str() {
    let ann = parse_type_annotation(" @type name: str").unwrap();
    assert_eq!(ann.name, "name");
    assert_eq!(ann.shell_type, ShellType::String);
}

#[test]
fn test_parse_type_annotation_string_alias() {
    let ann = parse_type_annotation(" @type name: string").unwrap();
    assert_eq!(ann.shell_type, ShellType::String);
}

#[test]
fn test_parse_type_annotation_integer_alias() {
    let ann = parse_type_annotation(" @type count: integer").unwrap();
    assert_eq!(ann.shell_type, ShellType::Integer);
}

#[test]
fn test_parse_type_annotation_bool() {
    let ann = parse_type_annotation(" @type flag: bool").unwrap();
    assert_eq!(ann.shell_type, ShellType::Boolean);
}

#[test]
fn test_parse_type_annotation_path() {
    // Path is a string subtype
    let ann = parse_type_annotation(" @type config_path: path").unwrap();
    assert_eq!(ann.shell_type, ShellType::String);
}

#[test]
fn test_parse_type_annotation_array() {
    let ann = parse_type_annotation(" @type items: array").unwrap();
    assert_eq!(
        ann.shell_type,
        ShellType::Array(Box::new(ShellType::String))
    );
}

#[test]
fn test_parse_type_annotation_fd() {
    let ann = parse_type_annotation(" @type logfd: fd").unwrap();
    assert_eq!(ann.shell_type, ShellType::FileDescriptor);
}

#[test]
fn test_parse_type_annotation_exit_code() {
    let ann = parse_type_annotation(" @type result: exit_code").unwrap();
    assert_eq!(ann.shell_type, ShellType::ExitCode);
}

#[test]
fn test_parse_type_annotation_unknown_type() {
    let result = parse_type_annotation(" @type x: custom_type");
    assert!(result.is_none());
}

#[test]
fn test_parse_type_annotation_no_annotation() {
    assert!(parse_type_annotation(" this is a regular comment").is_none());
}

#[test]
fn test_parse_type_annotation_empty() {
    assert!(parse_type_annotation("").is_none());
}

#[test]
fn test_parse_param_annotation() {
    let ann = parse_type_annotation(" @param port: int").unwrap();
    assert_eq!(ann.name, "port");
    assert_eq!(ann.shell_type, ShellType::Integer);
    assert!(ann.is_param);
    assert!(!ann.is_return);
}

#[test]
fn test_parse_returns_annotation() {
    let ann = parse_type_annotation(" @returns: int").unwrap();
    assert_eq!(ann.shell_type, ShellType::Integer);
    assert!(ann.is_return);
    assert!(!ann.is_param);
}

// ============================================================================
// TypeContext Scope Tests
// ============================================================================

#[test]
fn test_type_context_set_and_lookup() {
    let mut ctx = TypeContext::new();
    ctx.set_type("port", ShellType::Integer);
    assert_eq!(ctx.lookup("port"), Some(&ShellType::Integer));
}

#[test]
fn test_type_context_lookup_missing() {
    let ctx = TypeContext::new();
    assert_eq!(ctx.lookup("unknown"), None);
}

#[test]
fn test_type_context_scope_push_pop() {
    let mut ctx = TypeContext::new();
    ctx.set_type("outer", ShellType::String);

    ctx.push_scope();
    ctx.set_type("inner", ShellType::Integer);
    assert_eq!(ctx.lookup("inner"), Some(&ShellType::Integer));
    assert_eq!(ctx.lookup("outer"), Some(&ShellType::String));

    ctx.pop_scope();
    assert_eq!(ctx.lookup("inner"), None);
    assert_eq!(ctx.lookup("outer"), Some(&ShellType::String));
}

#[test]
fn test_type_context_shadowing() {
    let mut ctx = TypeContext::new();
    ctx.set_type("x", ShellType::String);

    ctx.push_scope();
    ctx.set_type("x", ShellType::Integer);
    assert_eq!(ctx.lookup("x"), Some(&ShellType::Integer));

    ctx.pop_scope();
    assert_eq!(ctx.lookup("x"), Some(&ShellType::String));
}

#[test]
fn test_type_context_scope_depth() {
    let mut ctx = TypeContext::new();
    assert_eq!(ctx.scope_depth(), 1);

    ctx.push_scope();
    assert_eq!(ctx.scope_depth(), 2);

    ctx.push_scope();
    assert_eq!(ctx.scope_depth(), 3);

    ctx.pop_scope();
    assert_eq!(ctx.scope_depth(), 2);
}

#[test]
fn test_type_context_cannot_pop_last_scope() {
    let mut ctx = TypeContext::new();
    ctx.pop_scope();
    assert_eq!(ctx.scope_depth(), 1);
}

#[test]
fn test_type_context_function_sig() {
    let mut ctx = TypeContext::new();
    ctx.set_function_sig(
        "start",
        FunctionSig {
            params: vec![("port".to_string(), ShellType::Integer)],
            return_type: Some(ShellType::ExitCode),
        },
    );

    let sig = ctx.lookup_function("start").unwrap();
    assert_eq!(sig.params.len(), 1);
    assert_eq!(sig.params[0].0, "port");
    assert_eq!(sig.params[0].1, ShellType::Integer);
    assert_eq!(sig.return_type, Some(ShellType::ExitCode));
}

// ============================================================================
// Expression Type Inference Tests
// ============================================================================

fn make_ast(stmts: Vec<BashStmt>) -> BashAst {
    BashAst {
        statements: stmts,
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    }
}

#[test]
fn test_infer_string_literal() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&BashExpr::Literal("hello".to_string()));
    assert_eq!(ty, Some(ShellType::String));
}

#[test]
fn test_infer_integer_literal() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&BashExpr::Literal("42".to_string()));
    assert_eq!(ty, Some(ShellType::Integer));
}

#[test]
fn test_infer_negative_integer_literal() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&BashExpr::Literal("-5".to_string()));
    assert_eq!(ty, Some(ShellType::Integer));
}

#[test]
fn test_infer_arithmetic_expr() {
    let mut checker = TypeChecker::new();
    let arith = ArithExpr::Add(
        Box::new(ArithExpr::Number(1)),
        Box::new(ArithExpr::Number(2)),
    );
    let ty = checker.infer_expr(&BashExpr::Arithmetic(Box::new(arith)));
    assert_eq!(ty, Some(ShellType::Integer));
}

#[test]
fn test_infer_command_subst() {
    let mut checker = TypeChecker::new();
    let cmd = BashStmt::Command {
        name: "date".to_string(),
        args: vec![],
        redirects: vec![],
        span: Span::dummy(),
    };
    let ty = checker.infer_expr(&BashExpr::CommandSubst(Box::new(cmd)));
    assert_eq!(ty, Some(ShellType::String));
}

#[test]
fn test_infer_array() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&BashExpr::Array(vec![
        BashExpr::Literal("a".to_string()),
        BashExpr::Literal("b".to_string()),
    ]));
    assert_eq!(
        ty,
        Some(ShellType::Array(Box::new(ShellType::String)))
    );
}

#[test]
fn test_infer_test_expr() {
    let mut checker = TypeChecker::new();
    let test = TestExpr::FileExists(BashExpr::Literal("/tmp".to_string()));
    let ty = checker.infer_expr(&BashExpr::Test(Box::new(test)));
    assert_eq!(ty, Some(ShellType::Boolean));
}

#[test]
fn test_infer_concat() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&BashExpr::Concat(vec![
        BashExpr::Literal("hello".to_string()),
        BashExpr::Literal("world".to_string()),
    ]));
    assert_eq!(ty, Some(ShellType::String));
}

#[test]
fn test_infer_string_length() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&BashExpr::StringLength {
        variable: "x".to_string(),
    });
    assert_eq!(ty, Some(ShellType::Integer));
}

#[test]
fn test_infer_variable_after_assignment() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![BashStmt::Assignment {
        name: "port".to_string(),
        index: None,
        value: BashExpr::Literal("8080".to_string()),
        exported: false,
        span: Span::dummy(),
    }]);

    checker.check_ast(&ast);
    assert_eq!(
        checker.context().lookup("port"),
        Some(&ShellType::Integer)
    );
}

#[test]
fn test_infer_unknown_variable_returns_none() {
    let mut checker = TypeChecker::new();
    let ty = checker.infer_expr(&BashExpr::Variable("unknown".to_string()));
    assert_eq!(ty, None);
}

// ============================================================================
// Declare Statement Type Extraction Tests
// ============================================================================

#[test]
fn test_declare_i_sets_integer() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![BashStmt::Command {
        name: "declare".to_string(),
        args: vec![
            BashExpr::Literal("-i".to_string()),
            BashExpr::Literal("count".to_string()),
        ],
        redirects: vec![],
        span: Span::dummy(),
    }]);

    checker.check_ast(&ast);
    assert_eq!(
        checker.context().lookup("count"),
        Some(&ShellType::Integer)
    );
}

#[test]
fn test_declare_a_sets_array() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![BashStmt::Command {
        name: "declare".to_string(),
        args: vec![
            BashExpr::Literal("-a".to_string()),
            BashExpr::Literal("items".to_string()),
        ],
        redirects: vec![],
        span: Span::dummy(),
    }]);

    checker.check_ast(&ast);
    assert_eq!(
        checker.context().lookup("items"),
        Some(&ShellType::Array(Box::new(ShellType::String)))
    );
}

#[test]
fn test_declare_uppercase_a_sets_assoc_array() {
    let mut checker = TypeChecker::new();
    let ast = make_ast(vec![BashStmt::Command {
        name: "declare".to_string(),
        args: vec![
            BashExpr::Literal("-A".to_string()),
            BashExpr::Literal("map".to_string()),
        ],
        redirects: vec![],
        span: Span::dummy(),
    }]);

    checker.check_ast(&ast);
    assert_eq!(
        checker.context().lookup("map"),
        Some(&ShellType::AssocArray {
            key: Box::new(ShellType::String),
            value: Box::new(ShellType::String),
        })
    );
}

#[test]
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
    assert_eq!(
        checker.context().lookup("count"),
        Some(&ShellType::Integer)
    );
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
    assert_eq!(
        checker.context().lookup("port"),
        Some(&ShellType::Integer)
    );
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
    assert!(matches!(
        diags[0].kind,
        DiagnosticKind::TypeMismatch { .. }
    ));
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
    assert!(diags.is_empty(), "gradual typing: untyped var should not produce errors");
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
    assert!(diags.is_empty(), "fully untyped script should produce no diagnostics");
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
    let ast = make_ast(vec![
        BashStmt::Function {
            name: "myfunc".to_string(),
            body: vec![BashStmt::Assignment {
                name: "local_var".to_string(),
                index: None,
                value: BashExpr::Literal("42".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        },
    ]);

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
        condition: BashExpr::Test(Box::new(TestExpr::FileExists(
            BashExpr::Literal("/tmp".to_string()),
        ))),
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
    assert_eq!(
        checker.context().lookup("found"),
        Some(&ShellType::Integer)
    );
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
    assert!(diags.is_empty(), "true should be compatible with bool annotation");
}
