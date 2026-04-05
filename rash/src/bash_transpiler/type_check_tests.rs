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
    assert_eq!(ty, Some(ShellType::Array(Box::new(ShellType::String))));
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
    assert_eq!(checker.context().lookup("port"), Some(&ShellType::Integer));
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
    assert_eq!(checker.context().lookup("count"), Some(&ShellType::Integer));
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

include!("type_check_tests_tests_declare_with.rs");
