//! Expression conversion methods for `IrConverter`.
//!
//! Contains: `convert_expr`, `convert_expr_to_value`, `convert_unary_to_value`,
//! `convert_binary_to_value`, `convert_index_to_value`, `analyze_command_effects`.
//!
//! Call-related converters are in `expr_calls.rs`:
//! `convert_fn_call_to_value`, `convert_env_call_to_value`, `convert_method_call_to_value`.

use super::optimizations::is_string_value;
use super::shell_ir;
use super::{Effect, EffectSet, IrConverter, ShellIR, ShellValue};
use crate::models::Result;

impl IrConverter {
    pub(crate) fn convert_expr(&self, expr: &crate::ast::Expr) -> Result<ShellIR> {
        use crate::ast::Expr;

        match expr {
            Expr::FunctionCall { name, args } => {
                // Handle __format_concat at expression level (shouldn't happen often, but be safe)
                if name == "__format_concat" {
                    let _value = self.convert_expr_to_value(expr)?;
                    return Ok(ShellIR::Noop);
                }

                // Issue #95: exec() is a DSL built-in that runs a shell command string
                // It should use 'eval' to properly evaluate pipes and operators
                if name == "exec" {
                    // Convert exec("cmd1 | cmd2") to eval 'cmd1 | cmd2'
                    let mut cmd_args = Vec::new();
                    for arg in args {
                        cmd_args.push(self.convert_expr_to_value(arg)?);
                    }
                    return Ok(ShellIR::Exec {
                        cmd: shell_ir::Command {
                            program: "eval".to_string(),
                            args: cmd_args,
                        },
                        effects: self.analyze_command_effects(name),
                    });
                }

                // Convert function calls to shell commands
                let mut cmd_args = Vec::new();
                for arg in args {
                    cmd_args.push(self.convert_expr_to_value(arg)?);
                }

                // Check if this is a stdlib function - if so, use the shell function name
                let program = if crate::stdlib::is_stdlib_function(name) {
                    crate::stdlib::get_shell_function_name(name)
                } else {
                    name.clone()
                };

                Ok(ShellIR::Exec {
                    cmd: shell_ir::Command {
                        program,
                        args: cmd_args,
                    },
                    effects: self.analyze_command_effects(name),
                })
            }
            _ => {
                // For other expressions, convert to values and wrap in a noop
                let _value = self.convert_expr_to_value(expr)?;
                Ok(ShellIR::Noop)
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    pub(crate) fn convert_expr_to_value(&self, expr: &crate::ast::Expr) -> Result<ShellValue> {
        use crate::ast::{restricted::Literal, Expr};

        match expr {
            Expr::Literal(literal) => match literal {
                Literal::Bool(b) => Ok(ShellValue::Bool(*b)),
                Literal::U16(n) => Ok(ShellValue::String(n.to_string())),
                Literal::U32(n) => Ok(ShellValue::String(n.to_string())),
                Literal::I32(n) => Ok(ShellValue::String(n.to_string())),
                Literal::Str(s) => Ok(ShellValue::String(s.clone())),
            },
            Expr::Variable(name) => Ok(ShellValue::Variable(name.clone())),
            Expr::FunctionCall { name, args } => self.convert_fn_call_to_value(name, args),
            Expr::Unary { op, operand } => self.convert_unary_to_value(op, operand),
            Expr::Binary { op, left, right } => self.convert_binary_to_value(op, left, right),
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => self.convert_method_call_to_value(receiver, method, args),
            Expr::PositionalArgs => Ok(ShellValue::Arg { position: None }),
            Expr::Index { object, index } => self.convert_index_to_value(object, index),
            Expr::Array(elems) => {
                if let Some(first) = elems.first() {
                    self.convert_expr_to_value(first)
                } else {
                    Ok(ShellValue::String("".to_string()))
                }
            }
            _ => Ok(ShellValue::String("unknown".to_string())),
        }
    }

    fn convert_unary_to_value(
        &self,
        op: &crate::ast::restricted::UnaryOp,
        operand: &crate::ast::Expr,
    ) -> Result<ShellValue> {
        use crate::ast::restricted::UnaryOp;
        let operand_val = self.convert_expr_to_value(operand)?;

        match op {
            UnaryOp::Not => Ok(ShellValue::LogicalNot {
                operand: Box::new(operand_val),
            }),
            UnaryOp::Neg => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Sub,
                left: Box::new(ShellValue::String("0".to_string())),
                right: Box::new(operand_val),
            }),
        }
    }

    fn convert_binary_to_value(
        &self,
        op: &crate::ast::restricted::BinaryOp,
        left: &crate::ast::Expr,
        right: &crate::ast::Expr,
    ) -> Result<ShellValue> {
        use crate::ast::restricted::BinaryOp;
        let left_val = self.convert_expr_to_value(left)?;
        let right_val = self.convert_expr_to_value(right)?;

        match op {
            BinaryOp::Eq => {
                let is_string = is_string_value(&left_val) || is_string_value(&right_val);
                Ok(ShellValue::Comparison {
                    op: if is_string {
                        shell_ir::ComparisonOp::StrEq
                    } else {
                        shell_ir::ComparisonOp::NumEq
                    },
                    left: Box::new(left_val),
                    right: Box::new(right_val),
                })
            }
            BinaryOp::Ne => {
                let is_string = is_string_value(&left_val) || is_string_value(&right_val);
                Ok(ShellValue::Comparison {
                    op: if is_string {
                        shell_ir::ComparisonOp::StrNe
                    } else {
                        shell_ir::ComparisonOp::NumNe
                    },
                    left: Box::new(left_val),
                    right: Box::new(right_val),
                })
            }
            BinaryOp::Gt => Ok(ShellValue::Comparison {
                op: shell_ir::ComparisonOp::Gt,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Ge => Ok(ShellValue::Comparison {
                op: shell_ir::ComparisonOp::Ge,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Lt => Ok(ShellValue::Comparison {
                op: shell_ir::ComparisonOp::Lt,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Le => Ok(ShellValue::Comparison {
                op: shell_ir::ComparisonOp::Le,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Add => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Add,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Sub => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Sub,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Mul => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Mul,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Div => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Div,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Rem => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Mod,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::And => Ok(ShellValue::LogicalAnd {
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Or => Ok(ShellValue::LogicalOr {
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::BitAnd => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::BitAnd,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::BitOr => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::BitOr,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::BitXor => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::BitXor,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Shl => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Shl,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Shr => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Shr,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
        }
    }

    fn convert_index_to_value(
        &self,
        object: &crate::ast::Expr,
        index: &crate::ast::Expr,
    ) -> Result<ShellValue> {
        use crate::ast::Expr;

        if let Expr::Variable(name) = object {
            let idx_val = self.convert_expr_to_value(index)?;
            match idx_val {
                ShellValue::String(s) => Ok(ShellValue::Variable(format!("{}_{}", name, s))),
                ShellValue::Variable(_) | ShellValue::Arithmetic { .. } => {
                    Ok(ShellValue::DynamicArrayAccess {
                        array: name.clone(),
                        index: Box::new(idx_val),
                    })
                }
                _ => Ok(ShellValue::Variable(format!("{}_0", name))),
            }
        } else {
            Ok(ShellValue::String("unknown".to_string()))
        }
    }

    pub(crate) fn analyze_command_effects(&self, command: &str) -> EffectSet {
        // Simple effect analysis based on command name
        let mut effects = EffectSet::pure();

        match command {
            "curl" | "wget" => {
                effects.add(Effect::NetworkAccess);
            }
            "echo" | "printf" => {
                effects.add(Effect::FileWrite);
            }
            _ => {}
        }

        effects
    }
}
