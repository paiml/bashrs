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
    PosixEmitter::new()
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
        value: ShellValue::Arg { position: Some(1) },
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

include!("posix_coverage_tests_COV.rs");
