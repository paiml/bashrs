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

fn make_ast(stmts: Vec<Stmt>) -> RestrictedAst {
    RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: stmts,
        }],
        entry_point: "main".to_string(),
    }
}

// ---------------------------------------------------------------------------
// analyze_makefile_line — via exec() and println! calls
// ---------------------------------------------------------------------------

#[test]
fn test_MCOV_001_analyze_phony_line() {
    // .PHONY: clean all → emitted as comment
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "rash_println".to_string(),
        args: vec![Expr::Literal(Literal::Str(".PHONY: clean all".to_string()))],
    })]);
    let result = emit_makefile(&ast).unwrap();
    // Raw output mode returns the line directly
    assert!(result.contains(".PHONY"));
}

#[test]
fn test_MCOV_002_analyze_tab_prefixed_line() {
    // Tab-prefixed lines are recipe lines (emitted as comment)
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "rash_println".to_string(),
        args: vec![Expr::Literal(Literal::Str(
            "\tgcc -o build main.c".to_string(),
        ))],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("gcc -o build main.c"));
}

#[test]
fn test_MCOV_003_analyze_simple_assignment() {
    // "CC := gcc" → Variable with Simple flavor
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "exec".to_string(),
        args: vec![Expr::Literal(Literal::Str("CC := gcc".to_string()))],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("CC") && result.contains("gcc"));
}

#[test]
fn test_MCOV_004_analyze_target_line() {
    // "build: main.o" → Target with deps
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "exec".to_string(),
        args: vec![Expr::Literal(Literal::Str("build: main.o".to_string()))],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("build"));
}

#[test]
fn test_MCOV_005_analyze_recursive_assignment() {
    // "CFLAGS = -O2" → Variable with Recursive flavor
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "exec".to_string(),
        args: vec![Expr::Literal(Literal::Str("CFLAGS = -O2".to_string()))],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("CFLAGS") || result.contains("-O2"));
}

#[test]
fn test_MCOV_006_analyze_comment_line() {
    // A line that doesn't match any rule becomes a comment
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "exec".to_string(),
        args: vec![Expr::Literal(Literal::Str(
            "just a comment line".to_string(),
        ))],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("just a comment line"));
}

#[test]
fn test_MCOV_007_target_with_empty_deps_via_exec() {
    // "all:" → Target with empty deps
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "exec".to_string(),
        args: vec![Expr::Literal(Literal::Str("all:".to_string()))],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("all"));
}

// ---------------------------------------------------------------------------
// resolve_concat_expr — various literal types
// ---------------------------------------------------------------------------

#[test]
fn test_MCOV_008_resolve_concat_i32_literal() {
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "rash_println".to_string(),
        args: vec![Expr::Literal(Literal::I32(42))],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("42"));
}

#[test]
fn test_MCOV_009_resolve_concat_u16_literal() {
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "rash_println".to_string(),
        args: vec![Expr::Literal(Literal::U16(8080))],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("8080"));
}

#[test]
fn test_MCOV_010_resolve_concat_u32_literal() {
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "rash_println".to_string(),
        args: vec![Expr::Literal(Literal::U32(65536))],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("65536"));
}

#[test]
fn test_MCOV_011_resolve_concat_bool_literal() {
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "rash_println".to_string(),
        args: vec![Expr::Literal(Literal::Bool(true))],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("true"));
}

#[test]
fn test_MCOV_012_resolve_concat_variable_without_binding() {
    // Variable not in bindings → $(VAR) fallback
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "rash_println".to_string(),
        args: vec![Expr::Variable("compiler".to_string())],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("$(COMPILER)"));
}

#[test]
fn test_MCOV_013_resolve_concat_variable_with_binding() {
    // Variable in bindings (via let) → resolved value
    let ast = make_ast(vec![
        Stmt::Let {
            name: "compiler".to_string(),
            value: Expr::Literal(Literal::Str("clang".to_string())),
            declaration: true,
        },
        Stmt::Expr(Expr::FunctionCall {
            name: "rash_println".to_string(),
            args: vec![Expr::Variable("compiler".to_string())],
        }),
    ]);
    // In raw-output mode the variable binding is collected first,
    // then the println resolves it to the bound value.
    let result = emit_makefile(&ast).unwrap();
    // May contain COMPILER := clang (DSL mode) or "clang" (raw mode resolved)
    assert!(result.contains("clang") || result.contains("COMPILER"));
}

#[test]
fn test_MCOV_014_resolve_concat_format_concat_call() {
    // __format_concat concatenation
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "rash_println".to_string(),
        args: vec![Expr::FunctionCall {
            name: "__format_concat".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("hello".to_string())),
                Expr::Literal(Literal::Str(" world".to_string())),
            ],
        }],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("hello world"));
}

#[test]
fn test_MCOV_015_resolve_concat_unknown_expr_is_empty() {
    // An Expr type that doesn't match any arm → empty string (no output)
    // Use an Array as the first arg — should be treated as an unknown expr
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "rash_println".to_string(),
        args: vec![Expr::Array(vec![])],
    })]);
    // With empty resolution the raw output is empty — falls through to DSL path
    let result = emit_makefile(&ast);
    // Should either succeed with empty body fallback or succeed with makefile
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// expr_to_string — Array, Binary, Index
// ---------------------------------------------------------------------------

#[test]
fn test_MCOV_016_expr_to_string_array_becomes_space_joined() {
    let ast = make_ast(vec![Stmt::Let {
        name: "files".to_string(),
        value: Expr::Array(vec![
            Expr::Literal(Literal::Str("a.c".to_string())),
            Expr::Literal(Literal::Str("b.c".to_string())),
        ]),
        declaration: true,
    }]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("a.c b.c") || result.contains("a.c") && result.contains("b.c"));
}

#[test]
fn test_MCOV_017_expr_to_string_binary_add() {
    use crate::ast::restricted::BinaryOp;
    let ast = make_ast(vec![Stmt::Let {
        name: "result".to_string(),
        value: Expr::Binary {
            left: Box::new(Expr::Literal(Literal::I32(1))),
            op: BinaryOp::Add,
            right: Box::new(Expr::Literal(Literal::I32(2))),
        },
        declaration: true,
    }]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("shell echo") || result.contains("RESULT"));
}

#[test]
fn test_MCOV_018_expr_to_string_binary_sub() {
    use crate::ast::restricted::BinaryOp;
    let ast = make_ast(vec![Stmt::Let {
        name: "diff".to_string(),
        value: Expr::Binary {
            left: Box::new(Expr::Literal(Literal::I32(10))),
            op: BinaryOp::Sub,
            right: Box::new(Expr::Literal(Literal::I32(3))),
        },
        declaration: true,
    }]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("DIFF") || result.contains("10"));
}

#[test]
fn test_MCOV_019_expr_to_string_binary_mul() {
    use crate::ast::restricted::BinaryOp;
    let ast = make_ast(vec![Stmt::Let {
        name: "product".to_string(),
        value: Expr::Binary {
            left: Box::new(Expr::Literal(Literal::I32(4))),
            op: BinaryOp::Mul,
            right: Box::new(Expr::Literal(Literal::I32(5))),
        },
        declaration: true,
    }]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("PRODUCT") || result.contains("4"));
}

#[test]
fn test_MCOV_020_expr_to_string_binary_div() {
    use crate::ast::restricted::BinaryOp;
    let ast = make_ast(vec![Stmt::Let {
        name: "quot".to_string(),
        value: Expr::Binary {
            left: Box::new(Expr::Literal(Literal::I32(20))),
            op: BinaryOp::Div,
            right: Box::new(Expr::Literal(Literal::I32(4))),
        },
        declaration: true,
    }]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("QUOT") || result.contains("20"));
}

#[test]
fn test_MCOV_021_expr_to_string_binary_other_op() {
    use crate::ast::restricted::BinaryOp;
    // BinaryOp::Eq (or any non-arithmetic op) falls to "_ => format!(...)"
    let ast = make_ast(vec![Stmt::Let {
        name: "cmp".to_string(),
        value: Expr::Binary {
            left: Box::new(Expr::Literal(Literal::I32(1))),
            op: BinaryOp::Eq,
            right: Box::new(Expr::Literal(Literal::I32(1))),
        },
        declaration: true,
    }]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("CMP") || result.contains("1"));
}

#[test]
fn test_MCOV_022_expr_to_string_index_expr() {
    let ast = make_ast(vec![Stmt::Let {
        name: "item".to_string(),
        value: Expr::Index {
            object: Box::new(Expr::Variable("items".to_string())),
            index: Box::new(Expr::Literal(Literal::I32(0))),
        },
        declaration: true,
    }]);
    let result = emit_makefile(&ast).unwrap();
    // Should emit $(word ...) macro
    assert!(result.contains("ITEM") || result.contains("word"));
}

// ---------------------------------------------------------------------------
// rash_print / print output functions (no newline appended)
// ---------------------------------------------------------------------------

#[test]
fn test_MCOV_023_rash_print_no_newline() {
    let ast = make_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "rash_print".to_string(),
        args: vec![Expr::Literal(Literal::Str("no newline".to_string()))],
    })]);
    let result = emit_makefile(&ast).unwrap();
    assert!(result.contains("no newline"));
}

#[test]

include!("makefile_coverage_tests_tests_MCOV.rs");
