//! IR optimization passes and utility functions.
//!
//! Contains: `adjust_range_end`, `eval_arithmetic_op`, `try_fold_constant_arithmetic`,
//! `constant_fold`, `fold_arithmetic_value`, `eliminate_dead_code`,
//! `is_string_value`, `transform_ir`.

use super::shell_ir;
use super::{ShellIR, ShellValue};

/// Adjust range end value for exclusive ranges (0..n → 0..=n-1).
/// For literal integers, directly subtract 1. For variables and expressions,
/// wrap in Arithmetic { Sub, end_val, 1 } so shell emits $((n - 1)).
pub(crate) fn adjust_range_end(end_val: ShellValue, inclusive: bool) -> ShellValue {
    if inclusive {
        return end_val;
    }
    // Exclusive range: subtract 1 from end
    match &end_val {
        ShellValue::String(s) => {
            if let Ok(n) = s.parse::<i32>() {
                ShellValue::String((n - 1).to_string())
            } else {
                // Non-numeric string — wrap in arithmetic
                ShellValue::Arithmetic {
                    op: shell_ir::ArithmeticOp::Sub,
                    left: Box::new(end_val),
                    right: Box::new(ShellValue::String("1".to_string())),
                }
            }
        }
        // Variable or expression — wrap in arithmetic subtraction
        _ => ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Sub,
            left: Box::new(end_val),
            right: Box::new(ShellValue::String("1".to_string())),
        },
    }
}

/// Evaluate an arithmetic operation on two constant integers.
/// Returns None for division/modulo by zero.
fn eval_arithmetic_op(op: &shell_ir::ArithmeticOp, left: i64, right: i64) -> Option<i64> {
    match op {
        shell_ir::ArithmeticOp::Add => Some(left + right),
        shell_ir::ArithmeticOp::Sub => Some(left - right),
        shell_ir::ArithmeticOp::Mul => Some(left * right),
        shell_ir::ArithmeticOp::Div if right != 0 => Some(left / right),
        shell_ir::ArithmeticOp::Mod if right != 0 => Some(left % right),
        shell_ir::ArithmeticOp::BitAnd => Some(left & right),
        shell_ir::ArithmeticOp::BitOr => Some(left | right),
        shell_ir::ArithmeticOp::BitXor => Some(left ^ right),
        shell_ir::ArithmeticOp::Shl => Some(left << right),
        shell_ir::ArithmeticOp::Shr => Some(left >> right),
        _ => None,
    }
}

/// Try to fold two ShellValues as constant integer arithmetic.
/// Returns the folded result string, or None if folding is not possible.
fn try_fold_constant_arithmetic(
    op: &shell_ir::ArithmeticOp,
    left: &ShellValue,
    right: &ShellValue,
) -> Option<String> {
    if let (ShellValue::String(l), ShellValue::String(r)) = (left, right) {
        if let (Ok(ln), Ok(rn)) = (l.parse::<i64>(), r.parse::<i64>()) {
            return eval_arithmetic_op(op, ln, rn).map(|v| v.to_string());
        }
    }
    None
}

pub(crate) fn constant_fold(ir: ShellIR) -> ShellIR {
    let mut transform_fn = |node| match node {
        ShellIR::Let {
            name,
            value: ShellValue::Concat(parts),
            effects,
        } => {
            if parts.iter().all(|p| matches!(p, ShellValue::String(_))) {
                let folded = parts
                    .iter()
                    .filter_map(|p| match p {
                        ShellValue::String(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .collect::<String>();
                ShellIR::Let {
                    name,
                    value: ShellValue::String(folded),
                    effects,
                }
            } else {
                ShellIR::Let {
                    name,
                    value: ShellValue::Concat(parts),
                    effects,
                }
            }
        }
        ShellIR::Let {
            name,
            value: ShellValue::Arithmetic { op, left, right },
            effects,
        } => {
            let folded_left = fold_arithmetic_value(*left);
            let folded_right = fold_arithmetic_value(*right);

            let value = match try_fold_constant_arithmetic(&op, &folded_left, &folded_right) {
                Some(result) => ShellValue::String(result),
                None => ShellValue::Arithmetic {
                    op,
                    left: Box::new(folded_left),
                    right: Box::new(folded_right),
                },
            };
            ShellIR::Let {
                name,
                value,
                effects,
            }
        }
        _ => node,
    };
    transform_ir(ir, &mut transform_fn)
}

/// Recursively fold arithmetic values (for nested expressions like 10 * 1024 * 1024)
fn fold_arithmetic_value(value: ShellValue) -> ShellValue {
    match value {
        ShellValue::Arithmetic { op, left, right } => {
            let folded_left = fold_arithmetic_value(*left);
            let folded_right = fold_arithmetic_value(*right);

            match try_fold_constant_arithmetic(&op, &folded_left, &folded_right) {
                Some(result) => ShellValue::String(result),
                None => ShellValue::Arithmetic {
                    op,
                    left: Box::new(folded_left),
                    right: Box::new(folded_right),
                },
            }
        }
        other => other,
    }
}

pub(crate) fn eliminate_dead_code(ir: ShellIR) -> ShellIR {
    // Simple dead code elimination
    ir // Placeholder - would implement actual DCE
}

/// Check if a ShellValue represents a string type (not a number)
pub(crate) fn is_string_value(value: &ShellValue) -> bool {
    match value {
        ShellValue::String(s) => {
            // Check if it's a string literal (not parseable as number)
            s.parse::<i64>().is_err() && s.parse::<f64>().is_err()
        }
        ShellValue::Bool(_) => false, // Bools are not strings for comparison
        ShellValue::Variable(_) => false, // Can't determine at compile time
        ShellValue::EnvVar { .. } => false, // Can't determine at compile time
        ShellValue::Concat(_) => true, // String concatenation
        ShellValue::CommandSubst(_) => false, // Could be numeric
        ShellValue::Comparison { .. } => false,
        ShellValue::Arithmetic { .. } => false,
        ShellValue::LogicalAnd { .. }
        | ShellValue::LogicalOr { .. }
        | ShellValue::LogicalNot { .. } => false,
        // Sprint 27b: Command-line arguments are not determinable at compile time
        ShellValue::Arg { .. } | ShellValue::ArgCount => false,
        // P0-POSITIONAL-PARAMETERS: Arguments with defaults are not determinable at compile time
        ShellValue::ArgWithDefault { .. } => false,
        // Sprint 27c: Exit code handling - GREEN PHASE (exit codes are numeric, not string)
        ShellValue::ExitCode => false,
        // Dynamic array access returns numeric values
        ShellValue::DynamicArrayAccess { .. } => false,
    }
}

fn transform_ir<F>(ir: ShellIR, transform: &mut F) -> ShellIR
where
    F: FnMut(ShellIR) -> ShellIR,
{
    let transformed = match ir {
        ShellIR::Sequence(stmts) => {
            let new_stmts = stmts
                .into_iter()
                .map(|stmt| transform_ir(stmt, transform))
                .collect();
            ShellIR::Sequence(new_stmts)
        }
        ShellIR::If {
            test,
            then_branch,
            else_branch,
        } => {
            let new_then = Box::new(transform_ir(*then_branch, transform));
            let new_else = else_branch.map(|eb| Box::new(transform_ir(*eb, transform)));
            ShellIR::If {
                test,
                then_branch: new_then,
                else_branch: new_else,
            }
        }
        ShellIR::Function { name, params, body } => {
            let new_body = Box::new(transform_ir(*body, transform));
            ShellIR::Function {
                name,
                params,
                body: new_body,
            }
        }
        other => other,
    };

    transform(transformed)
}
