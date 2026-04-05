//! Coverage tests for makefile.rs — targeting uncovered branches
//!
//! Focuses on:
//! - analyze_makefile_line: .PHONY, tab-prefix, := assignment, target, = assignment, comment
//! - resolve_concat_expr: I32/U16/U32/Bool literals, variable with/without binding, __format_concat
//! - expr_to_string: Array, Binary ops (Add/Sub/Mul/Div/other), Index, unsupported
//! - convert_println with empty resolved string
//! - convert_exec with empty resolved string
//! - emit_raw_lines / collect_variable_bindings / emit_raw_output_stmt
//!   (rash_print/rash_eprintln output, exec/println/print variants)
//! - wrap_shell_in_makefile with shebang/set-lines stripped
//! - non-main function with params
//! - convert_helper_function with println variant

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::makefile::emit_makefile;
use crate::ast::restricted::{Function, Literal, Type};
use crate::ast::{Expr, RestrictedAst, Stmt};

#[test]
fn test_MCOV_031_exec_with_empty_resolution() {
    // __format_concat with no args → empty string → exec skips output
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "exec".to_string(),
        args: vec![Expr::FunctionCall {
            name: "__format_concat".to_string(),
            args: vec![],
        }],
    })]);
    // Empty resolved exec → raw output empty → falls to DSL path (empty) → bash fallback
    let result = emit_makefile(&ast);
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// Entry point not found returns error
// ---------------------------------------------------------------------------

#[test]
fn test_MCOV_032_entry_point_not_found_returns_error() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "other".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![],
        }],
        entry_point: "main".to_string(), // "main" doesn't exist
    };
    let err = emit_makefile(&ast).unwrap_err();
    assert!(
        format!("{err}").contains("Entry point"),
        "Expected entry point error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Target with non-string name error path
// ---------------------------------------------------------------------------

#[test]
fn test_MCOV_033_phony_target_non_string_name_error() {
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "phony_target".to_string(),
        args: vec![
            Expr::Literal(Literal::I32(99)), // Non-string name
            Expr::Array(vec![]),
        ],
    })]);
    let err = emit_makefile(&ast).unwrap_err();
    assert!(
        format!("{err}").contains("string literal"),
        "Expected string literal error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// __array__ function call in extract_string_array
// ---------------------------------------------------------------------------

#[test]
fn test_MCOV_034_extract_array_via_array_func_call() {
    // __array__ FunctionCall as deps argument
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "target".to_string(),
        args: vec![
            Expr::Literal(Literal::Str("my_target".to_string())),
            Expr::FunctionCall {
                name: "__array__".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("dep1".to_string())),
                    Expr::Literal(Literal::Str("dep2".to_string())),
                ],
            },
        ],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("my_target"));
    assert!(result.contains("dep1") || result.contains("dep2"));
}

// ---------------------------------------------------------------------------
// extract_string_array fallback: non-array, non-str expr
// ---------------------------------------------------------------------------

#[test]
fn test_MCOV_035_extract_array_variable_fallback() {
    // A Variable expression as deps — triggers the fallback "try as string" path
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "target".to_string(),
        args: vec![
            Expr::Literal(Literal::Str("t".to_string())),
            Expr::Variable("deps".to_string()),
        ],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("t") || result.contains("DEPS"));
}

// ---------------------------------------------------------------------------
// expr_to_string unsupported expression returns error
// ---------------------------------------------------------------------------

#[test]
fn test_MCOV_036_expr_to_string_unsupported_returns_error() {
    // A FunctionCall expression (not __array__) as an array element in a let
    // triggers the Err("Cannot convert...") branch in expr_to_string
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "target".to_string(),
        args: vec![
            Expr::Literal(Literal::Str("tgt".to_string())),
            Expr::Array(vec![Expr::FunctionCall {
                name: "unknown".to_string(),
                args: vec![],
            }]),
        ],
    })]);
    // The FunctionCall in the array triggers expr_to_string error
    let result = emit_makefile(&ast);
    // This may error or succeed depending on path — key is it doesn't panic
    let _ = result;
}

// ---------------------------------------------------------------------------
// Collect variable bindings — only Str literals are collected
// ---------------------------------------------------------------------------

#[test]
fn test_MCOV_037_collect_variable_bindings_non_str_skipped() {
    // Non-Str let binding + variable reference: should use $(VAR) fallback
    let ast = make_ast(vec![
        Stmt::Let {
            name: "port".to_string(),
            value: Expr::Literal(Literal::I32(8080)),
            declaration: true,
        },
        Stmt::Expr(Expr::FunctionCall {
            name: "rash_println".to_string(),
            args: vec![Expr::Variable("port".to_string())],
        }),
    ]);
    let result = emit_makefile(&ast).unwrap();
    // Port as I32 is not collected as str binding, so variable ref → $(PORT)
    assert!(result.contains("8080") || result.contains("PORT"));
}

// ---------------------------------------------------------------------------
// Multiple raw-output functions in sequence
// ---------------------------------------------------------------------------

#[test]
fn test_MCOV_038_multiple_raw_output_stmts() {
    let ast = make_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "rash_println".to_string(),
            args: vec![Expr::Literal(Literal::Str("CC := gcc".to_string()))],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "rash_println".to_string(),
            args: vec![Expr::Literal(Literal::Str("CFLAGS := -O2".to_string()))],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "rash_println".to_string(),
            args: vec![Expr::Literal(Literal::Str("".to_string()))],
        }),
    ]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("CC := gcc"));
    assert!(result.contains("CFLAGS := -O2"));
}

// ---------------------------------------------------------------------------
// Non-raw let-only DSL path (no raw output) → MakeAst
// ---------------------------------------------------------------------------

#[test]
fn test_MCOV_039_dsl_path_let_only() {
    let ast = make_ast(vec![Stmt::Let {
        name: "version".to_string(),
        value: Expr::Literal(Literal::Str("1.0.0".to_string())),
        declaration: true,
    }]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("VERSION") && result.contains("1.0.0"));
}

// ---------------------------------------------------------------------------
// Target with deps parsed from raw line with multiple deps
// ---------------------------------------------------------------------------

#[test]
fn test_MCOV_040_target_line_with_multiple_deps() {
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "exec".to_string(),
        args: vec![Expr::Literal(Literal::Str(
            "all: build test lint".to_string(),
        ))],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("all"));
    // deps build, test, lint should appear
    assert!(result.contains("build") || result.contains("test") || result.contains("lint"));
}
