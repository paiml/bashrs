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
            // GH-293: array_len/array_join operate on the *elements* of a
            // local array literal, not on a scalar "$name" that is never
            // assigned for an array.
            "array_len" | "array_join" => self.convert_array_stdlib_call(name, args),
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

    /// GH-293: `array_join`/`array_len`'s first argument names an array, but a
    /// bare `Expr::Variable(name)` lowers to `ShellValue::Variable(name)`
    /// unconditionally — the same `"$name"` a scalar would use. No such
    /// variable is ever assigned for an array (the literal lowers to
    /// `name_0`, `name_1`, ... — see `convert_for_iterable`), so the stdlib
    /// call read an unset variable and the script aborted under `set -u`.
    ///
    /// Fix: reconstruct the newline-joined element list the runtime helpers
    /// (`rash_array_len`/`rash_array_join`) already expect in `$1`, via
    /// `$(printf '%s\n' "$name_0" "$name_1" ...)`.
    fn convert_array_stdlib_call(
        &self,
        name: &str,
        args: &[crate::ast::Expr],
    ) -> Result<ShellValue> {
        // GH-293: for a local array literal the element variables and the
        // length are known here, so the call lowers exactly — no `$( )`,
        // whose trailing-newline stripping would drop empty trailing elements.
        if let Some(items) = self.known_array_items(args.first())? {
            return match name {
                "array_len" => Ok(ShellValue::String(items.len().to_string())),
                _ => self.join_items(items, args.get(1)),
            };
        }
        let cmd_args = args
            .iter()
            .map(|arg| self.convert_expr_to_value(arg))
            .collect::<Result<Vec<_>>>()?;
        let program = crate::stdlib::get_shell_function_name(name);
        Ok(ShellValue::CommandSubst(shell_ir::Command {
            program,
            args: cmd_args,
        }))
    }

    /// The element values of a local array literal (by name or inline), or
    /// `None` when the argument is anything else.
    fn known_array_items(&self, arg: Option<&crate::ast::Expr>) -> Result<Option<Vec<ShellValue>>> {
        use crate::ast::Expr;
        match arg {
            Some(Expr::Variable(name)) => Ok(self.arrays.borrow().get(name).copied().map(|len| {
                (0..len)
                    .map(|i| ShellValue::Variable(format!("{name}_{i}")))
                    .collect()
            })),
            Some(Expr::Array(elements)) => elements
                .iter()
                .map(|e| self.convert_expr_to_value(e))
                .collect::<Result<Vec<_>>>()
                .map(Some),
            _ => Ok(None),
        }
    }

    /// `item0 sep item1 sep …` as one concatenated value.
    fn join_items(
        &self,
        items: Vec<ShellValue>,
        sep: Option<&crate::ast::Expr>,
    ) -> Result<ShellValue> {
        let sep = match sep {
            Some(expr) => self.convert_expr_to_value(expr)?,
            None => ShellValue::String(String::new()),
        };
        let mut parts = Vec::with_capacity(items.len() * 2);
        for (i, item) in items.into_iter().enumerate() {
            if i > 0 {
                parts.push(sep.clone());
            }
            parts.push(item);
        }
        Ok(match parts.len() {
            0 => ShellValue::String(String::new()),
            1 => parts.remove(0),
            _ => ShellValue::Concat(parts),
        })
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

    /// GH-305: an unlowerable method call used to fall through to the
    /// placeholder `ShellValue::String("unknown")`. In statement position
    /// (`convert_expr`'s catch-all) that value is then discarded entirely
    /// and the whole call lowers to `ShellIR::Noop` -> `:` — the program
    /// silently does nothing instead of failing to transpile. Every method
    /// call must either have a real lowering or fail loudly, naming the
    /// method, so callers see a transpile error instead of a shell script
    /// that runs and does the wrong thing.
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

        // PMAT-258 / GH-316 (second half, #316 blind quorum, 3-0):
        // `x.unwrap_or(d)` / `x.unwrap_or_else(|| d)` on a bare variable `x`
        // -- the general case, distinct from the `args.get(N)` /
        // `std::env::args().nth(N)` patterns just above (those keep their
        // existing `${N:-d}` lowering: a missing positional argument can
        // never be "set and empty" the way an arbitrary variable can).
        if (method == "unwrap_or" || method == "unwrap_or_else") && args.len() == 1 {
            if let Some(val) = self.try_unwrap_or_dash(receiver, &args[0]) {
                return Ok(val);
            }
        }

        // GH-306 / PMAT-258 GH-316: `.len()` and `.to_string()` each have an
        // honest lowering for some receivers (array literal, string) —
        // extracted so this dispatcher's own branching stays under the
        // per-function cognitive-complexity limit.
        if let Some(value) = self.try_len_or_to_string(receiver, method, args)? {
            return Ok(value);
        }

        // PMAT-258 / GH-316: `push_str`/`push` (mutate a local string),
        // `insert` and `rev` do not have a lowering here on purpose. Making
        // `push_str`/`push` observable would require the *statement*-level
        // caller (`convert_expr` in expr.rs) to emit an assignment instead
        // of discarding this value as `ShellIR::Noop` — out of scope for
        // this ticket (PMAT-258 scope is `expr_calls.rs` only). `insert` has
        // no generally-correct POSIX spelling, and `rev()` on a string has
        // no POSIX builtin at all. All four fall through to the error below,
        // naming the method, rather than silently emitting a wrong value.
        Err(crate::models::Error::Validation(format!(
            "cannot transpile `.{method}()`: no lowering exists for this method call. \
             See bashrs#305."
        )))
    }

    /// PMAT-258 / GH-316: `.len()` (array literal or string) and
    /// `.to_string()` (identity), or `None` when `method`/`args` don't match
    /// either — in which case the caller falls through to the error.
    fn try_len_or_to_string(
        &self,
        receiver: &crate::ast::Expr,
        method: &str,
        args: &[crate::ast::Expr],
    ) -> Result<Option<ShellValue>> {
        // GH-306: `items.len()` on a local array literal has an exact
        // element count known here — the same literal `known_array_items`
        // already extracts for `array_len(items)` (GH-293). A receiver that
        // is not a known array literal (or whose length is otherwise not
        // statically known) falls through to the error below rather than
        // the old "unknown" placeholder.
        if method == "len" && args.is_empty() {
            if let Some(items) = self.known_array_items(Some(receiver))? {
                return Ok(Some(ShellValue::String(items.len().to_string())));
            }
            // PMAT-258 / GH-316: `.len()` on a *string* has an honest POSIX
            // spelling that `known_array_items` cannot give us, because a
            // string never gets the `name_0`, `name_1`, ... element table an
            // array literal does. Route it through `string_len_value`
            // instead of falling through to the error below.
            if let Some(value) = self.string_len_value(receiver) {
                return Ok(Some(value));
            }
            return Ok(None);
        }

        // PMAT-258 / GH-316: `.to_string()` is the identity in shell — every
        // value is already a string, so there is nothing to lower beyond
        // whatever the receiver itself evaluates to.
        if method == "to_string" && args.is_empty() {
            return Ok(Some(self.convert_expr_to_value(receiver)?));
        }

        Ok(None)
    }

    /// PMAT-258 / GH-316: the honest POSIX spelling of `.len()` on a string.
    ///
    /// A variable receiver becomes `${#var}` — computed at *runtime*, so it
    /// is correct no matter what the variable holds (there is no way to know
    /// a variable's compile-time value here; `known_array_items` is the only
    /// existing compile-time table and it tracks array literals, not
    /// strings). A string-literal receiver is measured directly.
    ///
    /// Reuses `ShellValue::Glob`, the existing "emit this text unquoted and
    /// unescaped" variant (GH-148), rather than adding a new `ShellValue`
    /// variant — that would require emitter changes outside this ticket's
    /// scope (`rash/src/ir/expr_calls.rs` only).
    fn string_len_value(&self, receiver: &crate::ast::Expr) -> Option<ShellValue> {
        use crate::ast::{restricted::Literal, Expr};
        match receiver {
            Expr::Literal(Literal::Str(s)) => Some(ShellValue::String(s.len().to_string())),
            // PMAT-258: `${#var}` is the STRING length. bashrs flattens an
            // array literal to scalars, so a variable the converter knows to be
            // an array must never take this path: the count path above owns it,
            // and falling through here would report the joined text's length —
            // the wrong-value class GH-305 and GH-306 exist to prevent. A
            // variable the converter knows nothing about is a string as far as
            // this IR can tell; a collection built any other way is outside the
            // supported subset (bashrs#323).
            Expr::Variable(name) if !self.arrays.borrow().contains_key(name) => {
                Some(ShellValue::Glob(format!("${{#{name}}}")))
            }
            _ => None,
        }
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

    /// PMAT-258 / GH-316 (second half): the faithful lowering of
    /// `x.unwrap_or(d)` / `x.unwrap_or_else(|| d)` on a bare variable `x` is
    /// `${x-d}` -- the *unset-only* form -- not `${x:-d}`.
    ///
    /// POSIX shell has no Option type: a variable is unset, set-and-empty,
    /// or set. `${x:-d}` substitutes `d` on either unset OR empty; `${x-d}`
    /// substitutes only when unset. Rust's `unwrap_or`/`unwrap_or_else`
    /// return the default only for `None` -- `Some("")` is a present value
    /// and must not be replaced. Twelve corpus entries removed in v7.2.0
    /// used one of these two methods, which had been silently lowering to a
    /// no-op; a blind quorum decided 3-0 on `${x-d}` as the one honest
    /// spelling (bashrs#316).
    ///
    /// `receiver` must be a bare variable: parameter expansion needs a name,
    /// not an arbitrary subexpression. Anything else (or a `default` with no
    /// honest no-side-effect spelling, see [`Self::unwrap_or_default_text`])
    /// returns `None` and the caller falls through to the loud
    /// "no lowering exists" error, naming the method.
    fn try_unwrap_or_dash(
        &self,
        receiver: &crate::ast::Expr,
        default: &crate::ast::Expr,
    ) -> Option<ShellValue> {
        use crate::ast::Expr;

        let Expr::Variable(name) = receiver else {
            return None;
        };
        let default_text = self.unwrap_or_default_text(default)?;
        // Reuses `ShellValue::Glob`, the existing "emit this text unquoted
        // and unescaped" variant (GH-148) that `string_len_value` already
        // uses for `${#var}` -- adding a dedicated `ShellValue` variant would
        // require emitter changes outside this ticket's scope
        // (`rash/src/ir/expr_calls.rs` only). The double quotes are baked
        // into the text itself so the emitted expansion is still quoted.
        Some(ShellValue::Glob(format!("\"${{{name}-{default_text}}}\"")))
    }

    /// The literal text to splice into `${x-…}` for `default`, or `None`
    /// when `default` has no honest, no-side-effect spelling.
    ///
    /// A string/integer literal is always safe. A variable is safe only if
    /// this converter has actually seen it declared (`self.declared_vars`):
    /// `syn`'s closure sugar is stripped before this AST layer even sees it
    /// (`SynExpr::Closure(c) => convert_expr(&c.body)` in
    /// `parser_convert_2.rs`), so `x.unwrap_or_else(|| literal)` and a bare
    /// `x.unwrap_or_else(some_fn)` (passing a function *reference*, not
    /// calling it) erase to the exact same `Expr::Variable` shape here.
    /// Requiring the name to already be a known local -- not merely any
    /// identifier -- is the only signal left to tell "a value" from "a
    /// function name", and a closure that calls a function must not be
    /// silently turned into a value. An actual function *call*
    /// (`Expr::FunctionCall`) is unambiguous and always rejected.
    fn unwrap_or_default_text(&self, default: &crate::ast::Expr) -> Option<String> {
        use crate::ast::{restricted::Literal, Expr};

        match default {
            Expr::Literal(Literal::Str(s)) => Some(crate::emitter::escape::escape_shell_string(s)),
            Expr::Literal(Literal::U32(n)) => Some(n.to_string()),
            Expr::Literal(Literal::I32(n)) => Some(n.to_string()),
            Expr::Literal(Literal::U16(n)) => Some(n.to_string()),
            Expr::Literal(Literal::Bool(b)) => Some(b.to_string()),
            Expr::Variable(name) if self.declared_vars.borrow().contains(name) => {
                Some(format!("${name}"))
            }
            _ => None,
        }
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
