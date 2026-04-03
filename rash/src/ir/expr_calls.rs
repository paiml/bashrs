//! Function call and method call expression converters.
//!
//! Contains: `convert_fn_call_to_value`, `convert_env_call_to_value`,
//! `convert_method_call_to_value`.
//!
//! Extracted from `expr.rs` to reduce per-file complexity.

use super::shell_ir;
use super::{IrConverter, ShellValue};
use crate::models::Result;

impl IrConverter {
    pub(super) fn convert_fn_call_to_value(
        &self,
        name: &str,
        args: &[crate::ast::Expr],
    ) -> Result<ShellValue> {
        if name == "env" || name == "env_var_or" {
            return self.convert_env_call_to_value(name, args);
        }
        if name == "arg" {
            return Self::convert_arg_call(args);
        }
        if name == "args" {
            return Ok(ShellValue::Arg { position: None });
        }
        if name == "arg_count" {
            return Ok(ShellValue::ArgCount);
        }
        if name == "exit_code" {
            return Ok(ShellValue::ExitCode);
        }
        // GH-148: capture("cmd arg1 arg2") → $(cmd arg1 arg2)
        // capture("cmd | filter") → $(sh -c 'cmd | filter')  (pipe-safe)
        if name == "capture" {
            if let Some(arg) = args.first() {
                let cmd_value = self.convert_expr_to_value(arg)?;
                match &cmd_value {
                    ShellValue::String(s) => {
                        // If the command contains shell operators (pipes, &&, ||, ;),
                        // wrap in sh -c to preserve operator semantics
                        let has_shell_operators = s.contains(" | ")
                            || s.contains(" && ")
                            || s.contains(" || ")
                            || s.contains(';');
                        if has_shell_operators {
                            return Ok(ShellValue::CommandSubst(shell_ir::Command {
                                program: "sh".to_string(),
                                args: vec![
                                    ShellValue::String("-c".to_string()),
                                    ShellValue::String(s.clone()),
                                ],
                            }));
                        }
                        // Simple command: split into program + args
                        let mut parts = s.split_whitespace();
                        let program = parts.next().unwrap_or("").to_string();
                        let cmd_args: Vec<ShellValue> =
                            parts.map(|p| ShellValue::String(p.to_string())).collect();
                        return Ok(ShellValue::CommandSubst(shell_ir::Command {
                            program,
                            args: cmd_args,
                        }));
                    }
                    ShellValue::Concat(_) => {
                        // For interpolated strings, fall through to regular handling
                        return self.convert_regular_fn_call(name, args);
                    }
                    _ => return self.convert_regular_fn_call(name, args),
                }
            }
            return self.convert_regular_fn_call(name, args);
        }
        // GH-148: glob("*.txt") → ShellValue::Glob("*.txt")
        // Emitted unquoted so shell expansion works in for-in loops
        if name == "glob" {
            if let Some(arg) = args.first() {
                let val = self.convert_expr_to_value(arg)?;
                if let ShellValue::String(pattern) = val {
                    return Ok(ShellValue::Glob(pattern));
                }
            }
            return Err(crate::models::Error::Validation(
                "glob() requires a string literal pattern argument".to_string(),
            ));
        }
        if name == "__format_concat" {
            return self.convert_format_concat(args);
        }
        if name == "__if_expr" && args.len() == 3 {
            return self.convert_expr_to_value(&args[1]);
        }

        self.convert_regular_fn_call(name, args)
    }

    /// Convert `arg(N)` → positional parameter
    fn convert_arg_call(args: &[crate::ast::Expr]) -> Result<ShellValue> {
        use crate::ast::{restricted::Literal, Expr};

        let first_arg = args.first().ok_or_else(|| {
            crate::models::Error::Validation("arg() requires at least one argument".to_string())
        })?;
        let position = match first_arg {
            Expr::Literal(Literal::U32(n)) => *n as usize,
            Expr::Literal(Literal::I32(n)) => *n as usize,
            _ => {
                return Err(crate::models::Error::Validation(
                    "arg() requires integer literal for position".to_string(),
                ))
            }
        };
        if position == 0 {
            return Err(crate::models::Error::Validation(
                "arg() position must be >= 1 (use arg(1) for first argument)".to_string(),
            ));
        }
        Ok(ShellValue::Arg {
            position: Some(position),
        })
    }

    /// Convert `__format_concat(parts...)` → Concat
    fn convert_format_concat(&self, args: &[crate::ast::Expr]) -> Result<ShellValue> {
        let mut parts = Vec::new();
        for arg in args {
            parts.push(self.convert_expr_to_value(arg)?);
        }
        Ok(ShellValue::Concat(parts))
    }

    /// Convert a regular (non-stdlib-special) function call → CommandSubst
    fn convert_regular_fn_call(&self, name: &str, args: &[crate::ast::Expr]) -> Result<ShellValue> {
        let mut cmd_args = Vec::new();
        for arg in args {
            cmd_args.push(self.convert_expr_to_value(arg)?);
        }

        let program = if crate::stdlib::is_stdlib_function(name) {
            crate::stdlib::get_shell_function_name(name)
        } else {
            name.to_string()
        };

        Ok(ShellValue::CommandSubst(shell_ir::Command {
            program,
            args: cmd_args,
        }))
    }

    fn convert_env_call_to_value(
        &self,
        name: &str,
        args: &[crate::ast::Expr],
    ) -> Result<ShellValue> {
        use crate::ast::{restricted::Literal, Expr};

        let first_arg = args.first().ok_or_else(|| {
            crate::models::Error::Validation(format!("{}() requires at least one argument", name))
        })?;
        let var_name = match first_arg {
            Expr::Literal(Literal::Str(s)) => s.clone(),
            _ => {
                return Err(crate::models::Error::Validation(format!(
                    "{}() requires string literal for variable name",
                    name
                )))
            }
        };

        if !var_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(crate::models::Error::Validation(format!(
                "Invalid environment variable name: '{}'",
                var_name
            )));
        }

        let default = if name == "env_var_or" {
            match &args.get(1) {
                Some(Expr::Literal(Literal::Str(s))) => Some(s.clone()),
                _ => {
                    return Err(crate::models::Error::Validation(
                        "env_var_or() requires string literal for default value".to_string(),
                    ))
                }
            }
        } else {
            None
        };

        Ok(ShellValue::EnvVar {
            name: var_name,
            default,
        })
    }

    pub(super) fn convert_method_call_to_value(
        &self,
        receiver: &crate::ast::Expr,
        method: &str,
        args: &[crate::ast::Expr],
    ) -> Result<ShellValue> {
        if method == "unwrap" && args.is_empty() {
            if let Some(val) = Self::try_unwrap_env_args_nth(receiver) {
                return Ok(val);
            }
        }

        if method == "unwrap_or" && args.len() == 1 {
            if let Some(val) = Self::try_unwrap_or_pattern(receiver, args) {
                return Ok(val);
            }
        }

        Ok(ShellValue::String("unknown".to_string()))
    }

    /// Match `std::env::args().nth(N).unwrap()` → `Arg { position: Some(N) }`
    fn try_unwrap_env_args_nth(receiver: &crate::ast::Expr) -> Option<ShellValue> {
        use crate::ast::{restricted::Literal, Expr};

        let Expr::MethodCall {
            receiver: inner_receiver,
            method: inner_method,
            args: inner_args,
        } = receiver
        else {
            return None;
        };
        if inner_method != "nth" || inner_args.len() != 1 {
            return None;
        }
        let Expr::FunctionCall {
            name,
            args: fn_args,
        } = &**inner_receiver
        else {
            return None;
        };
        if name != "std::env::args" || !fn_args.is_empty() {
            return None;
        }
        if let Some(Expr::Literal(Literal::U32(n))) = inner_args.first() {
            return Some(ShellValue::Arg {
                position: Some(*n as usize),
            });
        }
        None
    }

    /// Match `args.get(N).unwrap_or(default)` or `std::env::args().nth(N).unwrap_or(default)`
    fn try_unwrap_or_pattern(
        receiver: &crate::ast::Expr,
        args: &[crate::ast::Expr],
    ) -> Option<ShellValue> {
        use crate::ast::{restricted::Literal, Expr};

        let Expr::MethodCall {
            receiver: inner_receiver,
            method: inner_method,
            args: inner_args,
        } = receiver
        else {
            return None;
        };

        if inner_method == "get" && inner_args.len() == 1 {
            if let Some(Expr::Literal(Literal::U32(n))) = inner_args.first() {
                if let Some(Expr::Literal(Literal::Str(default_val))) = args.first() {
                    return Some(ShellValue::ArgWithDefault {
                        position: *n as usize,
                        default: default_val.clone(),
                    });
                }
            }
        }

        if inner_method == "nth" && inner_args.len() == 1 {
            return Self::try_env_args_nth_unwrap_or(inner_receiver, inner_args, args);
        }

        None
    }

    /// Match `std::env::args().nth(N).unwrap_or(default)` → `ArgWithDefault`
    fn try_env_args_nth_unwrap_or(
        inner_receiver: &crate::ast::Expr,
        inner_args: &[crate::ast::Expr],
        args: &[crate::ast::Expr],
    ) -> Option<ShellValue> {
        use crate::ast::{restricted::Literal, Expr};

        let Expr::FunctionCall {
            name,
            args: fn_args,
        } = inner_receiver
        else {
            return None;
        };
        if name != "std::env::args" || !fn_args.is_empty() {
            return None;
        }
        if let Some(Expr::Literal(Literal::U32(n))) = inner_args.first() {
            if let Some(Expr::Literal(Literal::Str(default_val))) = args.first() {
                return Some(ShellValue::ArgWithDefault {
                    position: *n as usize,
                    default: default_val.clone(),
                });
            }
        }
        None
    }
}
