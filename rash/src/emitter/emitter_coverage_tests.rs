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

fn e() -> PosixEmitter {
    PosixEmitter::new()
}
fn s(v: &str) -> ShellValue {
    ShellValue::String(v.to_string())
}
fn var(v: &str) -> ShellValue {
    ShellValue::Variable(v.to_string())
}
fn cmp(op: ComparisonOp, l: ShellValue, r: ShellValue) -> ShellValue {
    ShellValue::Comparison {
        op,
        left: Box::new(l),
        right: Box::new(r),
    }
}
fn echo(v: ShellValue) -> ShellIR {
    ShellIR::Echo { value: v }
}
fn if_ir(test: ShellValue, then: ShellIR, el: Option<ShellIR>) -> ShellIR {
    ShellIR::If {
        test,
        then_branch: Box::new(then),
        else_branch: el.map(Box::new),
    }
}

// --- emit_logical_operand: nested Or, Not, Arithmetic, Comparison, Bool, String ---

#[test]
fn test_ECOV_001_logical_operand_nested_or() {
    let ir = echo(ShellValue::LogicalAnd {
        left: Box::new(ShellValue::LogicalOr {
            left: Box::new(var("a")),
            right: Box::new(var("b")),
        }),
        right: Box::new(var("c")),
    });
    let r = e().emit(&ir).unwrap();
    assert!(
        r.contains("||") && r.contains("&&"),
        "Nested OR in AND: {r}"
    );
}

#[test]
fn test_ECOV_002_logical_operand_nested_not() {
    let ir = echo(ShellValue::LogicalAnd {
        left: Box::new(ShellValue::LogicalNot {
            operand: Box::new(var("flag")),
        }),
        right: Box::new(var("ok")),
    });
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("!flag"), "Negated in logical operand: {r}");
}

#[test]
fn test_ECOV_003_logical_operand_with_arithmetic() {
    let ir = echo(ShellValue::LogicalAnd {
        left: Box::new(ShellValue::Arithmetic {
            op: ArithmeticOp::Add,
            left: Box::new(var("x")),
            right: Box::new(s("1")),
        }),
        right: Box::new(ShellValue::Bool(true)),
    });
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("x + 1"), "Arithmetic in logical: {r}");
}

#[test]
fn test_ECOV_004_logical_operand_with_comparison() {
    let ir = echo(ShellValue::LogicalOr {
        left: Box::new(cmp(ComparisonOp::Gt, var("x"), s("0"))),
        right: Box::new(ShellValue::Bool(false)),
    });
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("-gt"), "Comparison in logical: {r}");
}

#[test]
fn test_ECOV_005_logical_operand_bool_values() {
    let ir = echo(ShellValue::LogicalAnd {
        left: Box::new(ShellValue::Bool(false)),
        right: Box::new(var("v")),
    });
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("0"), "Bool(false) emits 0: {r}");
}

#[test]
fn test_ECOV_033_logical_operand_string_fallback() {
    let ir = echo(ShellValue::LogicalAnd {
        left: Box::new(s("1")),
        right: Box::new(s("0")),
    });
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("&&"), "Logical AND with strings: {r}");
}

// --- append_concat_part: Arithmetic, CommandSubst, LogicalOr/Not error paths ---

#[test]
fn test_ECOV_006_concat_with_arithmetic() {
    let ir = ShellIR::Let {
        name: "msg".into(),
        value: ShellValue::Concat(vec![
            s("count="),
            ShellValue::Arithmetic {
                op: ArithmeticOp::Add,
                left: Box::new(var("n")),
                right: Box::new(s("1")),
            },
        ]),
        effects: EffectSet::pure(),
    };
    let r = e().emit(&ir).unwrap();
    assert!(
        r.contains("count=") && r.contains("$(("),
        "Arith in concat: {r}"
    );
}

#[test]
fn test_ECOV_007_concat_with_command_subst() {
    let ir = ShellIR::Let {
        name: "info".into(),
        value: ShellValue::Concat(vec![
            s("user="),
            ShellValue::CommandSubst(Command {
                program: "whoami".into(),
                args: vec![],
            }),
        ]),
        effects: EffectSet::pure(),
    };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("$(whoami)"), "Command subst in concat: {r}");
}

#[test]
fn test_ECOV_029_concat_logical_or_returns_error() {
    let ir = ShellIR::Let {
        name: "v".into(),
        value: ShellValue::Concat(vec![ShellValue::LogicalOr {
            left: Box::new(ShellValue::Bool(true)),
            right: Box::new(ShellValue::Bool(false)),
        }]),
        effects: EffectSet::pure(),
    };
    assert!(e().emit(&ir).is_err(), "LogicalOr in concat is error");
}

#[test]
fn test_ECOV_030_concat_logical_not_returns_error() {
    let ir = ShellIR::Let {
        name: "v".into(),
        value: ShellValue::Concat(vec![ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Bool(true)),
        }]),
        effects: EffectSet::pure(),
    };
    assert!(e().emit(&ir).is_err(), "LogicalNot in concat is error");
}

// --- emit_while_condition: nested compound, Bool(false), String, Comparison ---

#[test]
fn test_ECOV_008_while_condition_nested_and_in_or() {
    let ir = ShellIR::While {
        condition: ShellValue::LogicalOr {
            left: Box::new(ShellValue::LogicalAnd {
                left: Box::new(cmp(ComparisonOp::Lt, var("i"), s("10"))),
                right: Box::new(cmp(ComparisonOp::Gt, var("j"), s("0"))),
            }),
            right: Box::new(ShellValue::Bool(true)),
        },
        body: Box::new(ShellIR::Break),
    };
    let r = e().emit(&ir).unwrap();
    assert!(
        r.contains("&&") && r.contains("||"),
        "Nested AND in OR: {r}"
    );
}

#[test]
fn test_ECOV_009_while_condition_not_wrapping_and() {
    let ir = ShellIR::While {
        condition: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::LogicalAnd {
                left: Box::new(cmp(ComparisonOp::NumEq, var("x"), s("1"))),
                right: Box::new(cmp(ComparisonOp::NumEq, var("y"), s("1"))),
            }),
        },
        body: Box::new(ShellIR::Break),
    };
    let r = e().emit(&ir).unwrap();
    assert!(
        r.contains("! ") && r.contains("&&"),
        "NOT wrapping AND: {r}"
    );
}

#[test]
fn test_ECOV_028_while_condition_string_value() {
    let ir = ShellIR::While {
        condition: s("1"),
        body: Box::new(ShellIR::Break),
    };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("while [ 1 ]"), "String as condition: {r}");
}

#[test]
fn test_ECOV_032_while_condition_bool_false_nested() {
    let ir = ShellIR::While {
        condition: ShellValue::LogicalOr {
            left: Box::new(ShellValue::Bool(false)),
            right: Box::new(cmp(ComparisonOp::Lt, var("i"), s("5"))),
        },
        body: Box::new(ShellIR::Break),
    };
    let r = e().emit(&ir).unwrap();
    assert!(
        r.contains("false") && r.contains("||"),
        "Bool false in nested: {r}"
    );
}

#[test]
fn test_ECOV_038_while_comparison_condition() {
    let ir = ShellIR::While {
        condition: cmp(ComparisonOp::Lt, var("i"), s("10")),
        body: Box::new(ShellIR::Noop),
    };
    let r = e().emit(&ir).unwrap();
    assert!(
        r.contains("while [ ") && r.contains("-lt"),
        "Comparison in while: {r}"
    );
}

// --- Triple elif, Sequence-wrapped elif ---

#[test]
fn test_ECOV_010_triple_elif_chain() {
    let ir = if_ir(
        cmp(ComparisonOp::NumEq, var("x"), s("1")),
        echo(s("one")),
        Some(if_ir(
            cmp(ComparisonOp::NumEq, var("x"), s("2")),
            echo(s("two")),
            Some(if_ir(
                cmp(ComparisonOp::NumEq, var("x"), s("3")),
                echo(s("three")),
                Some(echo(s("other"))),
            )),
        )),
    );
    let r = e().emit(&ir).unwrap();
    assert_eq!(r.matches("elif").count(), 2, "Two elif: {r}");
    assert!(r.contains("else"), "Final else: {r}");
}

#[test]
fn test_ECOV_011_elif_via_sequence_wrapped_if() {
    let ir = if_ir(
        ShellValue::Bool(true),
        echo(s("first")),
        Some(ShellIR::Sequence(vec![if_ir(
            ShellValue::Bool(false),
            echo(s("second")),
            None,
        )])),
    );
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("elif"), "Sequence-wrapped If becomes elif: {r}");
}

// --- Functions ---

#[test]
fn test_ECOV_012_function_user_defined_with_body() {
    let ir = ShellIR::Sequence(vec![ShellIR::Function {
        name: "greet".into(),
        params: vec!["name".into()],
        body: Box::new(echo(var("name"))),
    }]);
    let r = e().emit(&ir).unwrap();
    assert!(
        r.contains("greet()") && r.contains("name=\"$1\"") && r.contains("echo"),
        "{r}"
    );
}

#[test]
fn test_ECOV_013_function_unknown_command_empty_body() {
    let ir = ShellIR::Function {
        name: "my_custom_cmd".into(),
        params: vec![],
        body: Box::new(ShellIR::Noop),
    };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("my_custom_cmd()"), "Custom func emitted: {r}");
}

// --- Test expressions: LogicalOr runtime, Variable, Bool(false) ---

#[test]
fn test_ECOV_014_test_logical_or_runtime() {
    let r = e()
        .emit(&if_ir(
            ShellValue::LogicalOr {
                left: Box::new(var("a")),
                right: Box::new(var("b")),
            },
            ShellIR::Noop,
            None,
        ))
        .unwrap();
    assert!(r.contains("||"), "Runtime LogicalOr in test: {r}");
}

#[test]
fn test_ECOV_015_test_variable() {
    let r = e().emit(&if_ir(var("flag"), ShellIR::Noop, None)).unwrap();
    assert!(
        r.contains("test -n") && r.contains("$flag"),
        "Variable test: {r}"
    );
}

#[test]

include!("emitter_coverage_tests_incl2.rs");
