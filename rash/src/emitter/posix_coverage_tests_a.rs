//! Coverage tests for posix.rs — targeting uncovered branches
//!
//! Focuses on:
//! - emit_shell_value branches (EnvVar, Arg, ArgWithDefault, ArgCount, ExitCode,
//!   DynamicArrayAccess, LogicalAnd/Or/Not runtime, LogicalNot constant-fold)
//! - emit_concatenation branches (Bool, EnvVar, Arg, ArgWithDefault, ArgCount,
//!   ExitCode, DynamicArrayAccess, Comparison error, Logical error, nested Concat)
//! - emit_arithmetic_operand (DynamicArrayAccess, CommandSubst, unsupported)
//! - emit_while_statement (LogicalAnd, LogicalOr, LogicalNot, general condition)
//! - emit_test_expression (String "0", CommandSubst predicate vs non-predicate)
//! - selective runtime emission (all rash_ functions)
//! - separate_functions with non-Sequence IR
//! - emit_function with empty body known command (no emit)
//! - emit_function with params, no-params
//! - Return with and without value
//! - ForIn emission
//! - empty Sequence noop path
//! - Case arm with guard
//! - classify helpers: classify_if_structure, classify_test_expression

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::posix::PosixEmitter;
use crate::ir::shell_ir::{ArithmeticOp, CaseArm, CasePattern, ComparisonOp};
use crate::ir::{Command, EffectSet, ShellIR, ShellValue};
use crate::models::Config;

#[test]
fn test_COV_POSIX_033_arithmetic_mul() {
    let ir = ShellIR::Let {
        name: "prod".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Mul,
            left: Box::new(ShellValue::String("4".to_string())),
            right: Box::new(ShellValue::String("5".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("4 * 5"));
}

#[test]
fn test_COV_POSIX_034_arithmetic_div() {
    let ir = ShellIR::Let {
        name: "quot".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Div,
            left: Box::new(ShellValue::String("20".to_string())),
            right: Box::new(ShellValue::String("4".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("20 / 4"));
}

#[test]
fn test_COV_POSIX_035_arithmetic_mod() {
    let ir = ShellIR::Let {
        name: "rem".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Mod,
            left: Box::new(ShellValue::String("7".to_string())),
            right: Box::new(ShellValue::String("3".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("7 % 3"));
}

#[test]
fn test_COV_POSIX_036_arithmetic_bitand() {
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::BitAnd,
            left: Box::new(ShellValue::String("15".to_string())),
            right: Box::new(ShellValue::String("9".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("15 & 9"));
}

#[test]
fn test_COV_POSIX_037_arithmetic_bitor() {
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::BitOr,
            left: Box::new(ShellValue::String("6".to_string())),
            right: Box::new(ShellValue::String("3".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("6 | 3"));
}

#[test]
fn test_COV_POSIX_038_arithmetic_bitxor() {
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::BitXor,
            left: Box::new(ShellValue::String("5".to_string())),
            right: Box::new(ShellValue::String("3".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("5 ^ 3"));
}

#[test]
fn test_COV_POSIX_039_arithmetic_shl() {
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Shl,
            left: Box::new(ShellValue::String("1".to_string())),
            right: Box::new(ShellValue::String("4".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("1 << 4"));
}

#[test]
fn test_COV_POSIX_040_arithmetic_shr() {
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Shr,
            left: Box::new(ShellValue::String("64".to_string())),
            right: Box::new(ShellValue::String("2".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("64 >> 2"));
}

// ---------------------------------------------------------------------------
// Nested arithmetic with precedence-based parenthesization
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_041_nested_arithmetic_parens() {
    // (1 + 2) * 3 — the Add sub-expression must be wrapped in parens
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Mul,
            left: Box::new(ShellValue::Arithmetic {
                op: ArithmeticOp::Add,
                left: Box::new(ShellValue::String("1".to_string())),
                right: Box::new(ShellValue::String("2".to_string())),
            }),
            right: Box::new(ShellValue::String("3".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("(1 + 2)"));
    assert!(result.contains("* 3"));
}

// ---------------------------------------------------------------------------
// While loop — compound conditions
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_042_while_logical_and_condition() {
    let ir = ShellIR::While {
        condition: ShellValue::LogicalAnd {
            left: Box::new(ShellValue::Comparison {
                op: ComparisonOp::Lt,
                left: Box::new(ShellValue::Variable("i".to_string())),
                right: Box::new(ShellValue::String("10".to_string())),
            }),
            right: Box::new(ShellValue::Comparison {
                op: ComparisonOp::Gt,
                left: Box::new(ShellValue::Variable("j".to_string())),
                right: Box::new(ShellValue::String("0".to_string())),
            }),
        },
        body: Box::new(ShellIR::Break),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("while"));
    assert!(result.contains("&&"));
    assert!(result.contains("-lt") || result.contains("-gt"));
}

#[test]
fn test_COV_POSIX_043_while_logical_or_condition() {
    let ir = ShellIR::While {
        condition: ShellValue::LogicalOr {
            left: Box::new(ShellValue::Comparison {
                op: ComparisonOp::NumEq,
                left: Box::new(ShellValue::Variable("x".to_string())),
                right: Box::new(ShellValue::String("0".to_string())),
            }),
            right: Box::new(ShellValue::Comparison {
                op: ComparisonOp::NumEq,
                left: Box::new(ShellValue::Variable("y".to_string())),
                right: Box::new(ShellValue::String("0".to_string())),
            }),
        },
        body: Box::new(ShellIR::Break),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("||"));
}

#[test]
fn test_COV_POSIX_044_while_logical_not_condition() {
    let ir = ShellIR::While {
        condition: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Comparison {
                op: ComparisonOp::NumEq,
                left: Box::new(ShellValue::Variable("done_flag".to_string())),
                right: Box::new(ShellValue::String("1".to_string())),
            }),
        },
        body: Box::new(ShellIR::Break),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("!"));
    assert!(result.contains("-eq"));
}

#[test]
fn test_COV_POSIX_045_while_general_condition() {
    // Falls into the general "[ cond ]" path
    let ir = ShellIR::While {
        condition: ShellValue::Variable("running".to_string()),
        body: Box::new(ShellIR::Break),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("while"));
    assert!(result.contains("running"));
}

// ---------------------------------------------------------------------------
// emit_test_expression — string "0" → true, CommandSubst predicate
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_046_test_string_true_value() {
    let ir = ShellIR::If {
        test: ShellValue::String("true".to_string()),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("if true"));
}

#[test]
fn test_COV_POSIX_047_test_string_zero_value() {
    let ir = ShellIR::If {
        test: ShellValue::String("0".to_string()),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("if true"));
}

#[test]
fn test_COV_POSIX_048_test_string_other_value() {
    let ir = ShellIR::If {
        test: ShellValue::String("foo".to_string()),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("if false"));
}

#[test]
fn test_COV_POSIX_049_test_command_subst_predicate_function() {
    let ir = ShellIR::If {
        test: ShellValue::CommandSubst(Command {
            program: "rash_string_contains".to_string(),
            args: vec![
                ShellValue::Variable("s".to_string()),
                ShellValue::String("x".to_string()),
            ],
        }),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    // predicate functions are emitted directly (no "test -n")
    assert!(result.contains("rash_string_contains"));
}

#[test]
fn test_COV_POSIX_050_test_command_subst_value_function() {
    let ir = ShellIR::If {
        test: ShellValue::CommandSubst(Command {
            program: "compute_value".to_string(),
            args: vec![],
        }),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("test -n"));
}

#[test]
fn test_COV_POSIX_051_test_other_value_uses_test_n() {
    // An Arg value in test context falls to the "other" branch
    let ir = ShellIR::If {
        test: ShellValue::Arg { position: Some(1) },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("test -n"));
}

// ---------------------------------------------------------------------------
// Test expression — LogicalNot constant fold
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_052_test_logical_not_constant_fold() {
    let ir = ShellIR::If {
        test: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Bool(true)),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("if false"));
}

#[test]
fn test_COV_POSIX_053_test_logical_not_variable() {
    let ir = ShellIR::If {
        test: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Variable("flag".to_string())),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("! \"$flag\""));
}

#[test]
fn test_COV_POSIX_054_test_logical_not_other_folded() {
    // Non-bool, non-variable falls through to emit_test_expression inner
    let ir = ShellIR::If {
        test: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Comparison {
                op: ComparisonOp::NumEq,
                left: Box::new(ShellValue::Variable("x".to_string())),
                right: Box::new(ShellValue::String("0".to_string())),
            }),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("!"));
    assert!(result.contains("-eq"));
}

// ---------------------------------------------------------------------------
// Test logical and/or constant folding in test context
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_055_test_logical_and_constant_fold() {
    let ir = ShellIR::If {
        test: ShellValue::LogicalAnd {
            left: Box::new(ShellValue::Bool(true)),
            right: Box::new(ShellValue::Bool(false)),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("if false"));
}

#[test]
fn test_COV_POSIX_056_test_logical_or_constant_fold() {
    let ir = ShellIR::If {
        test: ShellValue::LogicalOr {
            left: Box::new(ShellValue::Bool(false)),
            right: Box::new(ShellValue::Bool(true)),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("if true"));
}

#[test]
fn test_COV_POSIX_057_test_logical_and_runtime() {
    let ir = ShellIR::If {
        test: ShellValue::LogicalAnd {
            left: Box::new(ShellValue::Variable("a".to_string())),
            right: Box::new(ShellValue::Variable("b".to_string())),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("&&"));
}

// ---------------------------------------------------------------------------
// Selective runtime emission
// ---------------------------------------------------------------------------

fn make_runtime_call_ir(func: &str) -> ShellIR {
    ShellIR::Exec {
        cmd: Command {
            program: func.to_string(),
            args: vec![ShellValue::Variable("x".to_string())],
        },
        effects: EffectSet::pure(),
    }
}

#[test]
fn test_COV_POSIX_058_runtime_rash_print() {
    let ir = make_runtime_call_ir("rash_print");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_print()"));
}

#[test]
fn test_COV_POSIX_059_runtime_rash_eprintln() {
    let ir = make_runtime_call_ir("rash_eprintln");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_eprintln()"));
}

#[test]
fn test_COV_POSIX_060_runtime_rash_require() {
    let ir = make_runtime_call_ir("rash_require");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_require()"));
}

#[test]
fn test_COV_POSIX_061_runtime_rash_download_verified() {
    let ir = make_runtime_call_ir("rash_download_verified");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_download_verified()"));
}

#[test]
fn test_COV_POSIX_062_runtime_rash_string_trim() {
    let ir = make_runtime_call_ir("rash_string_trim");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_string_trim()"));
}

#[test]
fn test_COV_POSIX_063_runtime_rash_string_contains() {
    let ir = make_runtime_call_ir("rash_string_contains");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_string_contains()"));
}

#[test]
fn test_COV_POSIX_064_runtime_rash_string_len() {
    let ir = make_runtime_call_ir("rash_string_len");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_string_len()"));
}

#[test]
fn test_COV_POSIX_065_runtime_rash_string_replace() {
    let ir = make_runtime_call_ir("rash_string_replace");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_string_replace()"));
}

