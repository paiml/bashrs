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

fn emitter() -> PosixEmitter {
    PosixEmitter::new(Config::default())
}

// ---------------------------------------------------------------------------
// EnvVar values
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_001_env_var_no_default() {
    let ir = ShellIR::Let {
        name: "path".to_string(),
        value: ShellValue::EnvVar {
            name: "PATH".to_string(),
            default: None,
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("\"${PATH}\""));
}

#[test]
fn test_COV_POSIX_002_env_var_with_default() {
    let ir = ShellIR::Let {
        name: "shell".to_string(),
        value: ShellValue::EnvVar {
            name: "SHELL".to_string(),
            default: Some("/bin/sh".to_string()),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("${SHELL:-/bin/sh}"));
}

// ---------------------------------------------------------------------------
// Arg, ArgWithDefault, ArgCount, ExitCode
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_003_arg_with_position() {
    let ir = ShellIR::Echo {
        value: ShellValue::Arg {
            position: Some(1),
        },
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("\"$1\""));
}

#[test]
fn test_COV_POSIX_004_arg_all() {
    let ir = ShellIR::Echo {
        value: ShellValue::Arg { position: None },
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("\"$@\""));
}

#[test]
fn test_COV_POSIX_005_arg_with_default() {
    let ir = ShellIR::Echo {
        value: ShellValue::ArgWithDefault {
            position: 2,
            default: "fallback".to_string(),
        },
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("${2:-"));
    assert!(result.contains("fallback"));
}

#[test]
fn test_COV_POSIX_006_arg_count() {
    let ir = ShellIR::Echo {
        value: ShellValue::ArgCount,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("\"$#\""));
}

#[test]
fn test_COV_POSIX_007_exit_code() {
    let ir = ShellIR::Echo {
        value: ShellValue::ExitCode,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("\"$?\""));
}

// ---------------------------------------------------------------------------
// DynamicArrayAccess
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_008_dynamic_array_access_variable_index() {
    let ir = ShellIR::Echo {
        value: ShellValue::DynamicArrayAccess {
            array: "items".to_string(),
            index: Box::new(ShellValue::Variable("i".to_string())),
        },
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("items"));
    assert!(result.contains("eval"));
}

#[test]
fn test_COV_POSIX_009_dynamic_array_access_arithmetic_index() {
    let ir = ShellIR::Echo {
        value: ShellValue::DynamicArrayAccess {
            array: "data".to_string(),
            index: Box::new(ShellValue::Arithmetic {
                op: ArithmeticOp::Add,
                left: Box::new(ShellValue::Variable("i".to_string())),
                right: Box::new(ShellValue::String("1".to_string())),
            }),
        },
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("data"));
    assert!(result.contains("eval"));
}

#[test]
fn test_COV_POSIX_010_dynamic_array_access_other_index() {
    // Non-variable, non-arithmetic index falls to "0"
    let ir = ShellIR::Echo {
        value: ShellValue::DynamicArrayAccess {
            array: "arr".to_string(),
            index: Box::new(ShellValue::String("3".to_string())),
        },
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("arr"));
}

// ---------------------------------------------------------------------------
// Logical operators at runtime (not constant-foldable)
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_011_logical_and_runtime_values() {
    // Uses variables so constant-folding cannot eliminate
    let ir = ShellIR::Echo {
        value: ShellValue::LogicalAnd {
            left: Box::new(ShellValue::Variable("a".to_string())),
            right: Box::new(ShellValue::Variable("b".to_string())),
        },
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("&&"));
}

#[test]
fn test_COV_POSIX_012_logical_or_runtime_values() {
    let ir = ShellIR::Echo {
        value: ShellValue::LogicalOr {
            left: Box::new(ShellValue::Variable("a".to_string())),
            right: Box::new(ShellValue::Variable("b".to_string())),
        },
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("||"));
}

#[test]
fn test_COV_POSIX_013_logical_not_runtime_variable() {
    let ir = ShellIR::Echo {
        value: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Variable("flag".to_string())),
        },
    };
    let result = emitter().emit(&ir).unwrap();
    // Should contain negation operator
    assert!(result.contains("!"));
}

#[test]
fn test_COV_POSIX_014_logical_and_constant_fold_true() {
    let ir = ShellIR::Echo {
        value: ShellValue::LogicalAnd {
            left: Box::new(ShellValue::Bool(true)),
            right: Box::new(ShellValue::Bool(true)),
        },
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("true"));
}

#[test]
fn test_COV_POSIX_015_logical_or_constant_fold_false() {
    let ir = ShellIR::Echo {
        value: ShellValue::LogicalOr {
            left: Box::new(ShellValue::Bool(false)),
            right: Box::new(ShellValue::Bool(false)),
        },
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("false"));
}

#[test]
fn test_COV_POSIX_016_logical_not_constant_fold_true() {
    let ir = ShellIR::Echo {
        value: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Bool(false)),
        },
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("true"));
}

// ---------------------------------------------------------------------------
// Concatenation — Bool, EnvVar, Arg, ArgWithDefault, ArgCount, ExitCode
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_017_concat_with_bool() {
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("flag=".to_string()),
            ShellValue::Bool(true),
        ]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("flag="));
    assert!(result.contains("true"));
}

#[test]
fn test_COV_POSIX_018_concat_with_env_var_no_default() {
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("home=".to_string()),
            ShellValue::EnvVar {
                name: "HOME".to_string(),
                default: None,
            },
        ]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("${HOME}"));
}

#[test]
fn test_COV_POSIX_019_concat_with_env_var_with_default() {
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::EnvVar {
                name: "EDITOR".to_string(),
                default: Some("vi".to_string()),
            },
        ]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("${EDITOR:-vi}"));
}

#[test]
fn test_COV_POSIX_020_concat_with_arg_position() {
    let ir = ShellIR::Let {
        name: "val".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("arg=".to_string()),
            ShellValue::Arg {
                position: Some(1),
            },
        ]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("$1"));
}

#[test]
fn test_COV_POSIX_021_concat_with_arg_all() {
    let ir = ShellIR::Let {
        name: "all".to_string(),
        value: ShellValue::Concat(vec![ShellValue::Arg { position: None }]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("$@"));
}

#[test]
fn test_COV_POSIX_022_concat_with_arg_with_default() {
    let ir = ShellIR::Let {
        name: "val".to_string(),
        value: ShellValue::Concat(vec![ShellValue::ArgWithDefault {
            position: 1,
            default: "default".to_string(),
        }]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("${1:-default}"));
}

#[test]
fn test_COV_POSIX_023_concat_with_arg_count() {
    let ir = ShellIR::Let {
        name: "cnt".to_string(),
        value: ShellValue::Concat(vec![ShellValue::ArgCount]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("$#"));
}

#[test]
fn test_COV_POSIX_024_concat_with_exit_code() {
    let ir = ShellIR::Let {
        name: "ret".to_string(),
        value: ShellValue::Concat(vec![ShellValue::ExitCode]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("$?"));
}

#[test]
fn test_COV_POSIX_025_concat_with_dynamic_array() {
    let ir = ShellIR::Let {
        name: "val".to_string(),
        value: ShellValue::Concat(vec![ShellValue::DynamicArrayAccess {
            array: "arr".to_string(),
            index: Box::new(ShellValue::Variable("i".to_string())),
        }]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("arr"));
}

#[test]
fn test_COV_POSIX_026_concat_with_nested_concat() {
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("outer".to_string()),
            ShellValue::Concat(vec![
                ShellValue::String("inner1".to_string()),
                ShellValue::String("inner2".to_string()),
            ]),
        ]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("outer"));
    assert!(result.contains("inner1"));
    assert!(result.contains("inner2"));
}

#[test]
fn test_COV_POSIX_027_concat_comparison_returns_error() {
    let e = emitter();
    let value = ShellValue::Concat(vec![ShellValue::Comparison {
        op: ComparisonOp::NumEq,
        left: Box::new(ShellValue::Variable("x".to_string())),
        right: Box::new(ShellValue::String("1".to_string())),
    }]);
    let ir = ShellIR::Let {
        name: "v".to_string(),
        value,
        effects: EffectSet::pure(),
    };
    // Comparison in concat is an error
    assert!(e.emit(&ir).is_err());
}

#[test]
fn test_COV_POSIX_028_concat_logical_and_returns_error() {
    let e = emitter();
    let value = ShellValue::Concat(vec![ShellValue::LogicalAnd {
        left: Box::new(ShellValue::Bool(true)),
        right: Box::new(ShellValue::Bool(false)),
    }]);
    // LogicalAnd in concat is an error (even for constant values the concat
    // path hits the error arm before constant-folding)
    let ir = ShellIR::Let {
        name: "v".to_string(),
        value,
        effects: EffectSet::pure(),
    };
    assert!(e.emit(&ir).is_err());
}

// ---------------------------------------------------------------------------
// Arithmetic operand — DynamicArrayAccess and CommandSubst
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_029_arithmetic_with_cmd_subst_operand() {
    let ir = ShellIR::Let {
        name: "n".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Add,
            left: Box::new(ShellValue::CommandSubst(Command {
                program: "wc".to_string(),
                args: vec![ShellValue::String("-l".to_string())],
            })),
            right: Box::new(ShellValue::String("0".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("$(wc"));
    assert!(result.contains("+"));
}

#[test]
fn test_COV_POSIX_030_arithmetic_with_dynamic_array_operand() {
    let ir = ShellIR::Let {
        name: "sum".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Add,
            left: Box::new(ShellValue::DynamicArrayAccess {
                array: "nums".to_string(),
                index: Box::new(ShellValue::Variable("i".to_string())),
            }),
            right: Box::new(ShellValue::String("0".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("nums"));
    assert!(result.contains("+"));
}

#[test]
fn test_COV_POSIX_031_arithmetic_unsupported_operand_returns_error() {
    let e = emitter();
    let ir = ShellIR::Let {
        name: "n".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Add,
            left: Box::new(ShellValue::Bool(true)), // Bool unsupported in arithmetic
            right: Box::new(ShellValue::String("1".to_string())),
        },
        effects: EffectSet::pure(),
    };
    assert!(e.emit(&ir).is_err());
}

// ---------------------------------------------------------------------------
// Arithmetic operators — all variants
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_032_arithmetic_sub() {
    let ir = ShellIR::Let {
        name: "diff".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Sub,
            left: Box::new(ShellValue::String("10".to_string())),
            right: Box::new(ShellValue::String("3".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("10 - 3"));
}

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
        test: ShellValue::Arg {
            position: Some(1),
        },
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

#[test]
fn test_COV_POSIX_066_runtime_rash_string_to_upper() {
    let ir = make_runtime_call_ir("rash_string_to_upper");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_string_to_upper()"));
}

#[test]
fn test_COV_POSIX_067_runtime_rash_string_to_lower() {
    let ir = make_runtime_call_ir("rash_string_to_lower");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_string_to_lower()"));
}

#[test]
fn test_COV_POSIX_068_runtime_rash_fs_exists() {
    let ir = make_runtime_call_ir("rash_fs_exists");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_fs_exists()"));
}

#[test]
fn test_COV_POSIX_069_runtime_rash_fs_read_file() {
    let ir = make_runtime_call_ir("rash_fs_read_file");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_fs_read_file()"));
}

#[test]
fn test_COV_POSIX_070_runtime_rash_fs_write_file() {
    let ir = make_runtime_call_ir("rash_fs_write_file");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_fs_write_file()"));
}

#[test]
fn test_COV_POSIX_071_runtime_rash_fs_copy() {
    let ir = make_runtime_call_ir("rash_fs_copy");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_fs_copy()"));
}

#[test]
fn test_COV_POSIX_072_runtime_rash_fs_remove() {
    let ir = make_runtime_call_ir("rash_fs_remove");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_fs_remove()"));
}

#[test]
fn test_COV_POSIX_073_runtime_rash_fs_is_file() {
    let ir = make_runtime_call_ir("rash_fs_is_file");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_fs_is_file()"));
}

#[test]
fn test_COV_POSIX_074_runtime_rash_fs_is_dir() {
    let ir = make_runtime_call_ir("rash_fs_is_dir");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_fs_is_dir()"));
}

#[test]
fn test_COV_POSIX_075_runtime_rash_string_split() {
    let ir = make_runtime_call_ir("rash_string_split");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_string_split()"));
}

#[test]
fn test_COV_POSIX_076_runtime_rash_array_len() {
    let ir = make_runtime_call_ir("rash_array_len");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_array_len()"));
}

#[test]
fn test_COV_POSIX_077_runtime_rash_array_join() {
    let ir = make_runtime_call_ir("rash_array_join");
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_array_join()"));
}

// ---------------------------------------------------------------------------
// separate_functions with non-Sequence IR
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_078_separate_functions_non_sequence() {
    // Non-Sequence IR is treated as the whole main body
    let ir = ShellIR::Echo {
        value: ShellValue::String("standalone".to_string()),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("echo"));
    assert!(result.contains("standalone"));
}

// ---------------------------------------------------------------------------
// emit_function — known command with empty body is NOT emitted
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_079_function_known_command_empty_body_not_emitted() {
    let ir = ShellIR::Function {
        name: "echo".to_string(),
        params: vec![],
        body: Box::new(ShellIR::Noop),
    };
    let result = emitter().emit(&ir).unwrap();
    // echo() function def should NOT appear since echo is a known command
    assert!(!result.contains("echo() {"));
}

#[test]
fn test_COV_POSIX_080_function_known_command_empty_sequence_not_emitted() {
    let ir = ShellIR::Function {
        name: "grep".to_string(),
        params: vec![],
        body: Box::new(ShellIR::Sequence(vec![])),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(!result.contains("grep() {"));
}

#[test]
fn test_COV_POSIX_081_function_user_defined_with_params() {
    let ir = ShellIR::Sequence(vec![ShellIR::Function {
        name: "my_func".to_string(),
        params: vec!["arg1".to_string(), "arg2".to_string()],
        body: Box::new(ShellIR::Noop),
    }]);
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("my_func()"));
    assert!(result.contains("arg1=\"$1\""));
    assert!(result.contains("arg2=\"$2\""));
}

// ---------------------------------------------------------------------------
// Return statement
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_082_return_with_value() {
    let ir = ShellIR::Return {
        value: Some(ShellValue::Variable("result".to_string())),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("echo"));
    assert!(result.contains("result"));
    assert!(result.contains("return"));
}

#[test]
fn test_COV_POSIX_083_return_without_value() {
    let ir = ShellIR::Return { value: None };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("return"));
}

// ---------------------------------------------------------------------------
// ForIn emission
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_084_for_in_multi_items() {
    let ir = ShellIR::ForIn {
        var: "item".to_string(),
        items: vec![
            ShellValue::String("a".to_string()),
            ShellValue::String("b".to_string()),
            ShellValue::String("c".to_string()),
        ],
        body: Box::new(ShellIR::Echo {
            value: ShellValue::Variable("item".to_string()),
        }),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("for item in"));
    assert!(result.contains("do"));
    assert!(result.contains("done"));
}

// ---------------------------------------------------------------------------
// Empty sequence uses noop
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_085_empty_sequence_emits_noop() {
    let ir = ShellIR::Sequence(vec![]);
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains(':'));
}

// ---------------------------------------------------------------------------
// Case arm with guard
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_086_case_arm_with_guard() {
    let ir = ShellIR::Case {
        scrutinee: ShellValue::Variable("x".to_string()),
        arms: vec![CaseArm {
            pattern: CasePattern::Literal("1".to_string()),
            guard: Some(ShellValue::Comparison {
                op: ComparisonOp::Gt,
                left: Box::new(ShellValue::Variable("y".to_string())),
                right: Box::new(ShellValue::String("0".to_string())),
            }),
            body: Box::new(ShellIR::Echo {
                value: ShellValue::String("guarded".to_string()),
            }),
        }],
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("case"));
    assert!(result.contains("if"));
    assert!(result.contains("-gt"));
    assert!(result.contains("guarded"));
    assert!(result.contains("fi"));
}

// ---------------------------------------------------------------------------
// emit_with_trace
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_087_emit_with_trace_returns_trace() {
    use crate::emitter::emit_with_trace;
    let ir = ShellIR::Echo {
        value: ShellValue::String("traced".to_string()),
    };
    let config = Config::default();
    let (output, trace) = emit_with_trace(&ir, &config).unwrap();
    assert!(output.contains("echo"));
    assert!(!trace.is_empty());
}

// ---------------------------------------------------------------------------
// String with single-quote in assignment
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_088_assignment_string_with_single_quote() {
    // String containing a single quote — triggers escape_shell_string path
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::String("it's fine".to_string()),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("msg="));
    // The output should contain the text somehow (escaped)
    assert!(result.contains("it") && result.contains("fine"));
}

// ---------------------------------------------------------------------------
// Exec with runtime and builtin detection
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_089_exec_runtime_rash_prefix() {
    let ir = ShellIR::Exec {
        cmd: Command {
            program: "rash_println".to_string(),
            args: vec![ShellValue::String("hello".to_string())],
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_println"));
}

#[test]
fn test_COV_POSIX_090_exec_external_command() {
    let ir = ShellIR::Exec {
        cmd: Command {
            program: "git".to_string(),
            args: vec![
                ShellValue::String("status".to_string()),
                ShellValue::String("--short".to_string()),
            ],
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("git"));
    assert!(result.contains("--short"));
}

// ---------------------------------------------------------------------------
// StrEq and StrNe comparisons
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_091_comparison_str_eq() {
    let ir = ShellIR::If {
        test: ShellValue::Comparison {
            op: ComparisonOp::StrEq,
            left: Box::new(ShellValue::Variable("name".to_string())),
            right: Box::new(ShellValue::String("alice".to_string())),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains(" = "));
}

#[test]
fn test_COV_POSIX_092_comparison_str_ne() {
    let ir = ShellIR::If {
        test: ShellValue::Comparison {
            op: ComparisonOp::StrNe,
            left: Box::new(ShellValue::Variable("name".to_string())),
            right: Box::new(ShellValue::String("bob".to_string())),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("!="));
}

// ---------------------------------------------------------------------------
// elif chain
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_093_elif_chain() {
    let ir = ShellIR::If {
        test: ShellValue::Comparison {
            op: ComparisonOp::NumEq,
            left: Box::new(ShellValue::Variable("x".to_string())),
            right: Box::new(ShellValue::String("1".to_string())),
        },
        then_branch: Box::new(ShellIR::Echo {
            value: ShellValue::String("one".to_string()),
        }),
        else_branch: Some(Box::new(ShellIR::If {
            test: ShellValue::Comparison {
                op: ComparisonOp::NumEq,
                left: Box::new(ShellValue::Variable("x".to_string())),
                right: Box::new(ShellValue::String("2".to_string())),
            },
            then_branch: Box::new(ShellIR::Echo {
                value: ShellValue::String("two".to_string()),
            }),
            else_branch: Some(Box::new(ShellIR::Echo {
                value: ShellValue::String("other".to_string()),
            })),
        })),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("elif"));
    assert!(result.contains("else"));
    assert!(result.contains("one"));
    assert!(result.contains("two"));
    assert!(result.contains("other"));
}

// ---------------------------------------------------------------------------
// Assignment value — empty string
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_094_assignment_empty_string() {
    let ir = ShellIR::Let {
        name: "empty".to_string(),
        value: ShellValue::String(String::new()),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("empty=''"));
}

// ---------------------------------------------------------------------------
// emit_while_condition — Bool false
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_095_while_condition_bool_false() {
    let ir = ShellIR::While {
        condition: ShellValue::Bool(false),
        body: Box::new(ShellIR::Break),
    };
    let result = emitter().emit(&ir).unwrap();
    // Bool(false) falls to the general case: [ false ]
    assert!(result.contains("while") && result.contains("false"));
}
