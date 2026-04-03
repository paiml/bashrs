//! Value and expression emitters. Extracted from posix.rs.
use super::escape::{escape_command_name, escape_shell_string, escape_variable_name};
use super::posix::{
    arithmetic_op_str, classify_test_expression, needs_arithmetic_parens, try_fold_logical,
};
use crate::ir::shell_ir::Command;
use crate::ir::ShellValue;
use crate::models::Result;
impl super::posix::PosixEmitter {
    pub fn emit_shell_value(&self, value: &ShellValue) -> Result<String> {
        let choice = match value {
            ShellValue::String(_) => "literal_string",
            ShellValue::Variable(_) => "variable",
            ShellValue::Bool(_) => "bool",
            ShellValue::CommandSubst(_) => "cmd_subst",
            ShellValue::Concat(_) => "concat",
            ShellValue::Comparison { .. } => "comparison",
            ShellValue::Arithmetic { .. } => "arithmetic",
            ShellValue::LogicalAnd { .. }
            | ShellValue::LogicalOr { .. }
            | ShellValue::LogicalNot { .. } => "logical",
            ShellValue::Arg { .. } | ShellValue::ArgWithDefault { .. } | ShellValue::ArgCount => {
                "arg_access"
            }
            ShellValue::EnvVar { .. } => "env_var",
            ShellValue::ExitCode => "exit_code",
            ShellValue::DynamicArrayAccess { .. } => "dynamic_array",
            ShellValue::Glob(_) => "glob_pattern",
        };
        self.record_decision("value_emit", choice, "Value");

        match value {
            ShellValue::String(s) => Ok(escape_shell_string(s)),
            ShellValue::Bool(b) => Ok(self.emit_bool_value(*b)),
            ShellValue::Variable(name) => Ok(format!("\"${}\"", escape_variable_name(name))),
            // Sprint 27a: Environment variable expansion
            ShellValue::EnvVar { name, default } => match default {
                None => Ok(format!("\"${{{}}}\"", name)),
                Some(def) => Ok(format!("\"${{{}:-{}}}\"", name, def)),
            },
            ShellValue::Concat(parts) => self.emit_concatenation(parts),
            ShellValue::CommandSubst(cmd) => {
                let cmd_str = self.emit_command(cmd)?;
                Ok(format!("\"$({cmd_str})\""))
            }
            ShellValue::Comparison { op, left, right } => self.emit_comparison(op, left, right),
            ShellValue::Arithmetic { op, left, right } => self.emit_arithmetic(op, left, right),
            ShellValue::LogicalAnd { left, right } => {
                // Constant-fold all-literal boolean expressions at compile time
                if let (Some(l), Some(r)) = (try_fold_logical(left), try_fold_logical(right)) {
                    return Ok(self.emit_bool_value(l && r));
                }
                let left_str = self.emit_logical_operand(left)?;
                let right_str = self.emit_logical_operand(right)?;
                Ok(format!("$(({left_str} && {right_str}))"))
            }
            ShellValue::LogicalOr { left, right } => {
                if let (Some(l), Some(r)) = (try_fold_logical(left), try_fold_logical(right)) {
                    return Ok(self.emit_bool_value(l || r));
                }
                let left_str = self.emit_logical_operand(left)?;
                let right_str = self.emit_logical_operand(right)?;
                Ok(format!("$(({left_str} || {right_str}))"))
            }
            ShellValue::LogicalNot { operand } => {
                if let Some(b) = try_fold_logical(operand) {
                    return Ok(self.emit_bool_value(!b));
                }
                let operand_str = self.emit_logical_operand(operand)?;
                Ok(format!("$((!{operand_str}))"))
            }
            // Sprint 27b: Command-line argument access
            ShellValue::Arg { position } => match position {
                Some(n) => Ok(format!("\"${}\"", n)), // "$1", "$2", etc.
                None => Ok("\"$@\"".to_string()),     // All args
            },
            // P0-POSITIONAL-PARAMETERS: Argument with default value
            ShellValue::ArgWithDefault { position, default } => Ok(format!(
                "\"${{{}:-{}}}\"",
                position,
                escape_shell_string(default)
            )),
            ShellValue::ArgCount => Ok("\"$#\"".to_string()), // Argument count
            // Sprint 27c: Exit code access - GREEN PHASE
            ShellValue::ExitCode => Ok("\"$?\"".to_string()), // Exit code of last command
            // Dynamic array access: arr[i] → eval-based POSIX lookup
            ShellValue::DynamicArrayAccess { array, index } => {
                let idx_expr = self.emit_dynamic_index_expr(index)?;
                Ok(format!(
                    "\"$(eval \"printf '%s' \\\"\\${}_{}\\\"\")\"",
                    escape_variable_name(array),
                    idx_expr
                ))
            }
            // GH-148: Glob patterns emitted UNQUOTED for shell expansion
            ShellValue::Glob(pattern) => Ok(pattern.clone()),
        }
    }

    pub(crate) fn emit_comparison(
        &self,
        op: &crate::ir::shell_ir::ComparisonOp,
        left: &ShellValue,
        right: &ShellValue,
    ) -> Result<String> {
        use crate::ir::shell_ir::ComparisonOp;

        let left_val = self.emit_shell_value(left)?;
        let right_val = self.emit_shell_value(right)?;

        let op_str = match op {
            ComparisonOp::NumEq => "-eq",
            ComparisonOp::NumNe => "-ne",
            ComparisonOp::Gt => "-gt",
            ComparisonOp::Ge => "-ge",
            ComparisonOp::Lt => "-lt",
            ComparisonOp::Le => "-le",
            ComparisonOp::StrEq => "=",
            ComparisonOp::StrNe => "!=",
        };

        // Generate POSIX test command: [ "$left" -op "$right" ]
        Ok(format!("[ {left_val} {op_str} {right_val} ]"))
    }

    pub(crate) fn emit_arithmetic(
        &self,
        op: &crate::ir::shell_ir::ArithmeticOp,
        left: &ShellValue,
        right: &ShellValue,
    ) -> Result<String> {
        // For arithmetic, emit raw values (no quotes needed inside $((...)))
        // Pass parent op so children can decide about parentheses
        let left_str = self.emit_arithmetic_operand(left, Some(op), false)?;
        let right_str = self.emit_arithmetic_operand(right, Some(op), true)?;

        let op_str = arithmetic_op_str(op);

        // Generate POSIX arithmetic expansion: $((expr))
        Ok(format!("$(({left_str} {op_str} {right_str}))"))
    }

    pub(crate) fn emit_arithmetic_operand(
        &self,
        value: &ShellValue,
        parent_op: Option<&crate::ir::shell_ir::ArithmeticOp>,
        is_right: bool,
    ) -> Result<String> {
        match value {
            ShellValue::String(s) => Ok(s.clone()),
            ShellValue::Variable(name) => Ok(escape_variable_name(name)),
            ShellValue::Arithmetic { op, left, right } => {
                self.emit_nested_arithmetic(op, left, right, parent_op, is_right)
            }
            ShellValue::CommandSubst(cmd) => self.emit_arithmetic_cmd_subst(cmd),
            ShellValue::DynamicArrayAccess { array, index } => {
                self.emit_arithmetic_dynamic_access(array, index)
            }
            _ => Err(crate::models::Error::Emission(format!(
                "Unsupported value in arithmetic expression: {:?}",
                value
            ))),
        }
    }

    /// Emit a nested arithmetic expression with precedence-aware parenthesization.
    pub(crate) fn emit_nested_arithmetic(
        &self,
        op: &crate::ir::shell_ir::ArithmeticOp,
        left: &ShellValue,
        right: &ShellValue,
        parent_op: Option<&crate::ir::shell_ir::ArithmeticOp>,
        is_right: bool,
    ) -> Result<String> {
        let left_str = self.emit_arithmetic_operand(left, Some(op), false)?;
        let right_str = self.emit_arithmetic_operand(right, Some(op), true)?;
        let op_str = arithmetic_op_str(op);
        let expr = format!("{left_str} {op_str} {right_str}");

        if needs_arithmetic_parens(op, parent_op, is_right) {
            Ok(format!("({expr})"))
        } else {
            Ok(expr)
        }
    }

    /// Emit a command substitution in arithmetic context: $(func arg1 arg2).
    pub(crate) fn emit_arithmetic_cmd_subst(&self, cmd: &Command) -> Result<String> {
        let mut parts = vec![cmd.program.clone()];
        for arg in &cmd.args {
            parts.push(self.emit_shell_value(arg)?);
        }
        Ok(format!("$({})", parts.join(" ")))
    }

    /// Emit dynamic array access in arithmetic context.
    pub(crate) fn emit_arithmetic_dynamic_access(
        &self,
        array: &str,
        index: &ShellValue,
    ) -> Result<String> {
        let idx_expr = self.emit_dynamic_index_expr(index)?;
        Ok(format!(
            "$(eval \"printf '%s' \\\"\\${}_{}\\\"\")",
            escape_variable_name(array),
            idx_expr
        ))
    }

    /// Emit a value for use inside a logical/boolean arithmetic context.
    /// Returns bare variable names and values without quotes (for use inside $(( ))).
    pub(crate) fn emit_logical_operand(&self, value: &ShellValue) -> Result<String> {
        match value {
            ShellValue::String(s) => Ok(s.clone()),
            ShellValue::Variable(name) => Ok(escape_variable_name(name)),
            ShellValue::Bool(b) => Ok(if *b { "1".to_string() } else { "0".to_string() }),
            ShellValue::LogicalAnd { left, right } => {
                let l = self.emit_logical_operand(left)?;
                let r = self.emit_logical_operand(right)?;
                Ok(format!("{l} && {r}"))
            }
            ShellValue::LogicalOr { left, right } => {
                let l = self.emit_logical_operand(left)?;
                let r = self.emit_logical_operand(right)?;
                Ok(format!("({l} || {r})"))
            }
            ShellValue::LogicalNot { operand } => {
                let o = self.emit_logical_operand(operand)?;
                Ok(format!("!{o}"))
            }
            ShellValue::Arithmetic { op, left, right } => {
                let l = self.emit_arithmetic_operand(left, Some(op), false)?;
                let r = self.emit_arithmetic_operand(right, Some(op), true)?;
                let op_str = arithmetic_op_str(op);
                Ok(format!("({l} {op_str} {r})"))
            }
            ShellValue::Comparison { op, left, right } => self.emit_comparison(op, left, right),
            _ => self.emit_shell_value(value),
        }
    }

    pub(crate) fn emit_bool_value(&self, value: bool) -> String {
        if value { "true" } else { "false" }.to_string()
    }

    /// Emit the index expression for dynamic array access.
    /// Returns shell expression like `${i}` for variable or `$((i + 1))` for arithmetic.
    pub(crate) fn emit_dynamic_index_expr(&self, index: &ShellValue) -> Result<String> {
        match index {
            ShellValue::Variable(v) => Ok(format!("${{{}}}", escape_variable_name(v))),
            ShellValue::Arithmetic { op, left, right } => {
                let left_str = self.emit_arithmetic_operand(left, Some(op), false)?;
                let right_str = self.emit_arithmetic_operand(right, Some(op), true)?;
                let op_str = arithmetic_op_str(op);
                Ok(format!("$(({} {} {}))", left_str, op_str, right_str))
            }
            _ => Ok("0".to_string()),
        }
    }

    pub(crate) fn emit_concatenation(&self, parts: &[ShellValue]) -> Result<String> {
        let mut result = String::new();
        result.push('"');

        for part in parts {
            self.append_concat_part(&mut result, part)?;
        }

        result.push('"');
        Ok(result)
    }

    pub(crate) fn append_concat_part(&self, result: &mut String, part: &ShellValue) -> Result<()> {
        match part {
            ShellValue::String(s) => result.push_str(s),
            ShellValue::Bool(b) => result.push_str(&self.emit_bool_value(*b)),
            ShellValue::Variable(name) => {
                result.push_str(&format!("${{{}}}", escape_variable_name(name)));
            }
            // Sprint 27a: Environment variable expansion in concatenation
            ShellValue::EnvVar { name, default } => match default {
                None => result.push_str(&format!("${{{}}}", name)),
                Some(def) => result.push_str(&format!("${{{}:-{}}}", name, def)),
            },
            ShellValue::CommandSubst(cmd) => {
                let cmd_str = self.emit_command(cmd)?;
                result.push_str(&format!("$({cmd_str})"));
            }
            ShellValue::Concat(_) => {
                // Nested concatenation - flatten it
                let nested = self.emit_shell_value(part)?;
                self.append_flattened_content(result, &nested);
            }
            ShellValue::Comparison { .. } => {
                // Comparisons don't make sense in concatenation context
                // This should be caught at validation, but handle gracefully
                return Err(crate::models::Error::IrGeneration(
                    "Comparison expression cannot be used in string concatenation".to_string(),
                ));
            }
            ShellValue::Arithmetic { op, left, right } => {
                // Arithmetic in concat context - emit the $((...)) form
                let arith_str = self.emit_arithmetic(op, left, right)?;
                result.push_str(&arith_str);
            }
            ShellValue::LogicalAnd { .. }
            | ShellValue::LogicalOr { .. }
            | ShellValue::LogicalNot { .. } => {
                // Logical operators don't make sense in concatenation context
                return Err(crate::models::Error::IrGeneration(
                    "Logical expression cannot be used in string concatenation".to_string(),
                ));
            }
            // Sprint 27b: Command-line argument access in concatenation
            ShellValue::Arg { position } => match position {
                Some(n) => result.push_str(&format!("${}", n)),
                None => result.push_str("$@"),
            },
            // P0-POSITIONAL-PARAMETERS: Argument with default value in concatenation
            ShellValue::ArgWithDefault { position, default } => {
                result.push_str(&format!("${{{}:-{}}}", position, default));
            }
            ShellValue::ArgCount => {
                result.push_str("$#");
            }
            // Sprint 27c: Exit code in concatenation - GREEN PHASE
            ShellValue::ExitCode => {
                result.push_str("$?");
            }
            // Dynamic array access in concatenation
            ShellValue::DynamicArrayAccess { array, index } => {
                let idx_expr = self.emit_dynamic_index_expr(index)?;
                result.push_str(&format!(
                    "$(eval \"printf '%s' \\\"\\${}_{}\\\"\")",
                    escape_variable_name(array),
                    idx_expr
                ));
            }
            ShellValue::Glob(pattern) => {
                result.push_str(pattern);
            }
        }
        Ok(())
    }

    pub(crate) fn append_flattened_content(&self, result: &mut String, nested: &str) {
        // Remove quotes from nested value and add content
        if nested.starts_with('"') && nested.ends_with('"') {
            result.push_str(&nested[1..nested.len() - 1]);
        } else {
            result.push_str(nested);
        }
    }

    pub(crate) fn emit_command(&self, cmd: &Command) -> Result<String> {
        let mut result = escape_command_name(&cmd.program);

        for arg in &cmd.args {
            result.push(' ');
            result.push_str(&self.emit_shell_value(arg)?);
        }

        Ok(result)
    }

    pub fn emit_test_expression(&self, test: &ShellValue) -> Result<String> {
        self.record_decision("test_syntax", classify_test_expression(test), "Test");

        match test {
            ShellValue::Bool(true) => Ok("true".to_string()),
            ShellValue::Bool(false) => Ok("false".to_string()),
            ShellValue::Variable(name) => {
                Ok(format!("test -n \"${}\"", escape_variable_name(name)))
            }
            ShellValue::String(s) => Ok(self.emit_test_string_literal(s)),
            ShellValue::Comparison { .. } => self.emit_shell_value(test),
            ShellValue::LogicalNot { operand } => self.emit_test_not(operand),
            ShellValue::LogicalAnd { left, right } => {
                self.emit_test_binary_logical(left, right, true)
            }
            ShellValue::LogicalOr { left, right } => {
                self.emit_test_binary_logical(left, right, false)
            }
            ShellValue::CommandSubst(cmd) => self.emit_test_command_subst(test, cmd),
            other => {
                let value = self.emit_shell_value(other)?;
                Ok(format!("test -n {value}"))
            }
        }
    }

    /// Emit a string literal in test context: "true"/"0" map to true, all else to false.
    pub(crate) fn emit_test_string_literal(&self, s: &str) -> String {
        if s == "true" || s == "0" {
            "true".to_string()
        } else {
            "false".to_string()
        }
    }

    /// Emit a LogicalNot in test context using command negation (! cmd).
    pub(crate) fn emit_test_not(&self, operand: &ShellValue) -> Result<String> {
        if let Some(b) = try_fold_logical(operand) {
            return Ok(self.emit_bool_value(!b));
        }
        // For variable operands, use ! "$var" (treat as boolean command)
        if let ShellValue::Variable(name) = operand {
            return Ok(format!("! \"${}\"", escape_variable_name(name)));
        }
        let inner = self.emit_test_expression(operand)?;
        Ok(format!("! {inner}"))
    }

    /// Emit a LogicalAnd or LogicalOr in test context.
    /// When `is_and` is true, emits `&&`; otherwise emits `||`.
    pub(crate) fn emit_test_binary_logical(
        &self,
        left: &ShellValue,
        right: &ShellValue,
        is_and: bool,
    ) -> Result<String> {
        if let (Some(l), Some(r)) = (try_fold_logical(left), try_fold_logical(right)) {
            let folded = if is_and { l && r } else { l || r };
            return Ok(self.emit_bool_value(folded));
        }
        let l = self.emit_test_expression(left)?;
        let r = self.emit_test_expression(right)?;
        let op = if is_and { "&&" } else { "||" };
        Ok(format!("{l} {op} {r}"))
    }

    /// Emit a CommandSubst in test context, distinguishing predicates from value-producing functions.
    pub(crate) fn emit_test_command_subst(
        &self,
        test: &ShellValue,
        cmd: &Command,
    ) -> Result<String> {
        if self.is_predicate_function(&cmd.program) {
            self.emit_command(cmd)
        } else {
            let value = self.emit_shell_value(test)?;
            Ok(format!("test -n {value}"))
        }
    }

    pub(crate) fn is_predicate_function(&self, name: &str) -> bool {
        // Predicate functions return bool via exit code (0 = true, 1 = false)
        matches!(
            name,
            "rash_string_contains"
                | "rash_string_starts_with"
                | "rash_string_ends_with"
                | "rash_fs_exists"
                | "rash_fs_is_file"
                | "rash_fs_is_dir"
                | "test"
                | "["
        )
    }
}
