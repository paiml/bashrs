//! Function call and method call expression converters.
//!
//! Contains: `convert_fn_call_to_value`, `convert_env_call_to_value`,
//! `convert_method_call_to_value`.
//!
//! Extracted from `expr.rs` to reduce per-file complexity.

use super::shell_ir;
use super::{IrConverter, ShellValue};
use crate::models::Result;

/// Is this command string anything more than a bare argv?
///
/// bashrs#268. `capture`/`exec` used to ask only whether the string contained
/// `|`, `&&`, `||` or `;`. Everything else was split on whitespace and each
/// token re-quoted as a literal, which destroys every other piece of shell
/// syntax:
///
///   capture("grep -c 'runs-on' ci.yml")  ->  grep '-c' ''"'"'runs-on'"'"'' ci.yml
///
/// The quotes became part of the argument, so grep matched nothing and returned
/// 0 instead of 3 — silently the wrong command, not a failure.
///
/// So the question is not "does this contain an operator" but "does this contain
/// anything a shell would interpret". If it does, it goes to `sh -c` intact. A
/// bare argv keeps the fast path and does not gain a subshell.
fn needs_shell_interpretation(command: &str) -> bool {
    command.contains(|c: char| {
        matches!(
            c,
            '\'' | '"'
                | '$'
                | '`'
                | '*'
                | '?'
                | '['
                | ']'
                | '<'
                | '>'
                | '|'
                | '&'
                | ';'
                | '('
                | ')'
                | '\\'
                | '\n'
                | '~'
                | '{'
                | '}'
                | '!'
                | '#'
        )
    })
}

impl IrConverter {
    pub(super) fn convert_fn_call_to_value(
        &self,
        name: &str,
        args: &[crate::ast::Expr],
    ) -> Result<ShellValue> {
        // A dispatch table, not a chain of early returns. Same behaviour, and it
        // reads as the one-of-N choice it actually is — the `if name == …` chain
        // it replaced scored cognitive 29 against this repo's own limit of 25.
        match name {
            "env" | "env_var_or" => self.convert_env_call_to_value(name, args),
            "arg" => Self::convert_arg_call(args),
            "args" => Ok(ShellValue::Arg { position: None }),
            "arg_count" => Ok(ShellValue::ArgCount),
            "exit_code" => Ok(ShellValue::ExitCode),
            // GH-148: capture("cmd arg1 arg2") → $(cmd arg1 arg2);
            // capture("cmd | filter") → $(sh -c 'cmd | filter')
            "capture" => self.convert_capture_call(name, args),
            // GH-148: glob("*.txt") → an unquoted glob, so shell expansion works
            // in for-in loops.
            "glob" => self.convert_glob_call(args),
            "__format_concat" => self.convert_format_concat(args),
            "__if_expr" if args.len() == 3 => self.convert_expr_to_value(&args[1]),
            _ => self.convert_regular_fn_call(name, args),
        }
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

impl super::IrConverter {
    /// GH-148 / bashrs#268: `capture("cmd")` → a command substitution.
    ///
    /// Extracted from `convert_fn_call_to_value`, which was already at cognitive
    /// 76 on main — over this repo's own threshold — before the #268 fix touched
    /// it. The pre-commit gate refused the commit and was right to.
    fn convert_capture_call(&self, name: &str, args: &[crate::ast::Expr]) -> Result<ShellValue> {
        let Some(arg) = args.first() else {
            return self.convert_regular_fn_call(name, args);
        };
        // An interpolated string is not a literal command — regular handling
        // builds it at runtime.
        let ShellValue::String(command) = self.convert_expr_to_value(arg)? else {
            return self.convert_regular_fn_call(name, args);
        };
        Ok(ShellValue::CommandSubst(lower_command(&command)))
    }
}

/// Lower a command STRING to a `Command`, preserving shell syntax.
fn lower_command(command: &str) -> shell_ir::Command {
    if needs_shell_interpretation(command) {
        return shell_ir::Command {
            program: "sh".to_string(),
            args: vec![
                ShellValue::String("-c".to_string()),
                ShellValue::String(command.to_string()),
            ],
        };
    }
    // A bare argv: split into program + args, so an ordinary command keeps the
    // fast path and does not gain a subshell.
    let mut parts = command.split_whitespace();
    shell_ir::Command {
        program: parts.next().unwrap_or("").to_string(),
        args: parts.map(|p| ShellValue::String(p.to_string())).collect(),
    }
}

impl super::IrConverter {
    /// GH-148: `glob("*.txt")` → an unquoted glob, so shell expansion works in
    /// for-in loops.
    fn convert_glob_call(&self, args: &[crate::ast::Expr]) -> Result<ShellValue> {
        if let Some(arg) = args.first() {
            if let ShellValue::String(pattern) = self.convert_expr_to_value(arg)? {
                return Ok(ShellValue::Glob(pattern));
            }
        }
        Err(crate::models::Error::Validation(
            "glob() requires a string literal pattern argument".to_string(),
        ))
    }
}
