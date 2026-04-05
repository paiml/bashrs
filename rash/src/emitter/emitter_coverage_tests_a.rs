//! Additional emitter coverage tests for posix.rs and makefile.rs uncovered branches.
//!
//! Targets branches NOT already tested in posix_coverage_tests.rs / makefile_coverage_tests.rs:
//! emit_logical_operand (nested Or/Not/Arithmetic/Comparison/Bool/String), concat with
//! Arithmetic/CommandSubst/LogicalOr/LogicalNot error paths, emit_while_condition nested
//! compound, triple elif, Sequence-wrapped elif, function user-defined, test LogicalOr/Variable,
//! all comparison operators, ForIn, Case wildcard, Exit, Continue/Break, Noop, Makefile targets,
//! emit_shell_value flattened concat, Bash dialect fallback, Bool(false) in test.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::emitter::posix::PosixEmitter;
use crate::ir::shell_ir::{ArithmeticOp, CaseArm, CasePattern, ComparisonOp};
use crate::ir::{Command, EffectSet, ShellIR, ShellValue};
use crate::models::Config;

#[test]
fn test_ECOV_027_noop_standalone() {
    let r = e().emit(&ShellIR::Noop).unwrap();
    assert!(r.contains(':'), "Noop emits colon: {r}");
}

// --- Arithmetic right-side parens ---

#[test]
fn test_ECOV_031_arithmetic_right_side_parens() {
    let ir = ShellIR::Let {
        name: "r".into(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Div,
            left: Box::new(s("10")),
            right: Box::new(ShellValue::Arithmetic {
                op: ArithmeticOp::Sub,
                left: Box::new(s("3")),
                right: Box::new(s("1")),
            }),
        },
        effects: EffectSet::pure(),
    };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("(3 - 1)"), "Right side parenthesized: {r}");
}

// --- Flattened concat, Bash dialect ---

#[test]
fn test_ECOV_037_flattened_content_no_quotes() {
    let val = ShellValue::Concat(vec![s("a"), ShellValue::Concat(vec![var("x")])]);
    let r = e().emit_shell_value(&val).unwrap();
    assert!(r.contains("a") && r.contains("${x}"), "Flattened: {r}");
}

#[test]
fn test_ECOV_039_emit_bash_dialect() {
    use crate::emitter::emit;
    let r = emit(&echo(s("hello"))).unwrap();
    assert!(r.contains("echo"), "Bash dialect emits: {r}");
}

// --- Makefile additional branch coverage ---

#[test]
fn test_ECOV_035_makefile_target_with_recipes() {
    use crate::ast::restricted::{Function, Literal, Type};
    use crate::ast::{Expr, RestrictedAst, Stmt};
    use crate::emitter::makefile::emit_makefile;
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".into(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "target".into(),
                args: vec![
                    Expr::Literal(Literal::Str("build".into())),
                    Expr::Array(vec![Expr::Literal(Literal::Str("main.c".into()))]),
                    Expr::Array(vec![Expr::Literal(Literal::Str(
                        "gcc -o build main.c".into(),
                    ))]),
                ],
            })],
        }],
        entry_point: "main".into(),
    };
    let r = emit_makefile(&ast).unwrap();
    assert!(r.contains("build"), "Target present: {r}");
}

#[test]
fn test_ECOV_036_makefile_non_main_function_no_echo() {
    use crate::ast::restricted::{Function, Literal, Type};
    use crate::ast::{Expr, RestrictedAst, Stmt};
    use crate::emitter::makefile::emit_makefile;
    let ast = RestrictedAst {
        functions: vec![
            Function {
                name: "main".into(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Let {
                    name: "x".into(),
                    value: Expr::Literal(Literal::Str("val".into())),
                    declaration: true,
                }],
            },
            Function {
                name: "helper".into(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Let {
                    name: "y".into(),
                    value: Expr::Literal(Literal::Str("inner".into())),
                    declaration: true,
                }],
            },
        ],
        entry_point: "main".into(),
    };
    let r = emit_makefile(&ast).unwrap();
    assert!(!r.contains("helper:"), "No helper target: {r}");
}
