//! IR statement emitters. Extracted from posix.rs.
use super::escape::{escape_shell_string, escape_variable_name};
use super::posix::{classify_if_structure, get_indent, unwrap_single_if};
use crate::ir::shell_ir::Command;
use crate::ir::{ShellIR, ShellValue};
use crate::models::Result;
use std::fmt::Write;
impl super::posix::PosixEmitter {
    pub(crate) fn emit_ir(&self, output: &mut String, ir: &ShellIR, indent: usize) -> Result<()> {
        let ir_label = match ir {
            ShellIR::Let { .. } => "Let",
            ShellIR::Exec { .. } => "Exec",
            ShellIR::If { .. } => "If",
            ShellIR::Exit { .. } => "Exit",
            ShellIR::Sequence(_) => "Sequence",
            ShellIR::Noop => "Noop",
            ShellIR::Function { .. } => "Function",
            ShellIR::Echo { .. } => "Echo",
            ShellIR::For { .. } => "For",
            ShellIR::While { .. } => "While",
            ShellIR::Case { .. } => "Case",
            ShellIR::ForIn { .. } => "ForIn",
            ShellIR::Break => "Break",
            ShellIR::Continue => "Continue",
            ShellIR::Return { .. } => "Return",
        };
        self.record_decision("ir_dispatch", ir_label, ir_label);
        match ir {
            ShellIR::Let { name, value, .. } => {
                self.emit_let_statement(output, name, value, indent)
            }
            ShellIR::Exec { cmd, .. } => self.emit_exec_statement(output, cmd, indent),
            ShellIR::If {
                test,
                then_branch,
                else_branch,
            } => self.emit_if_statement(output, test, then_branch, else_branch.as_deref(), indent),
            ShellIR::Exit { code, message } => {
                self.emit_exit_statement(output, (*code).into(), message.as_ref(), indent)
            }
            ShellIR::Sequence(items) => self.emit_sequence(output, items, indent),
            ShellIR::Noop => self.emit_noop(output, indent),
            ShellIR::Function { name, params, body } => {
                self.emit_function(output, name, params, body, indent)
            }
            ShellIR::Echo { value } => self.emit_echo_statement(output, value, indent),
            ShellIR::For {
                var,
                start,
                end,
                body,
            } => self.emit_for_statement(output, var, start, end, body, indent),
            ShellIR::While { condition, body } => {
                self.emit_while_statement(output, condition, body, indent)
            }
            ShellIR::Case { scrutinee, arms } => {
                self.emit_case_statement(output, scrutinee, arms, indent)
            }
            ShellIR::ForIn { var, items, body } => {
                self.emit_for_in_statement(output, var, items, body, indent)
            }
            ShellIR::Break => {
                let indent_str = get_indent(indent + 1);
                writeln!(output, "{indent_str}break")?;
                Ok(())
            }
            ShellIR::Continue => {
                let indent_str = get_indent(indent + 1);
                writeln!(output, "{indent_str}continue")?;
                Ok(())
            }
            ShellIR::Return { value } => {
                let indent_str = get_indent(indent + 1);
                if let Some(val) = value {
                    let value_str = self.emit_shell_value(val)?;
                    writeln!(output, "{indent_str}echo {value_str}")?;
                }
                writeln!(output, "{indent_str}return")?;
                Ok(())
            }
        }
    }
    pub(crate) fn emit_let_statement(
        &self,
        output: &mut String,
        name: &str,
        value: &ShellValue,
        indent: usize,
    ) -> Result<()> {
        let choice = match value {
            ShellValue::String(_) => "single_quote",
            ShellValue::Variable(_) => "variable_ref",
            ShellValue::Concat(_) => "concat",
            ShellValue::CommandSubst(_) => "cmd_subst",
            ShellValue::Arithmetic { .. } => "arithmetic",
            ShellValue::Bool(_) => "bool_literal",
            _ => "other",
        };
        self.record_decision("assignment_value", choice, "Let");
        let indent_str = get_indent(indent + 1);
        let var_name = escape_variable_name(name);
        let var_value = self.emit_assignment_value(value)?;
        writeln!(output, "{indent_str}{var_name}={var_value}")?;
        Ok(())
    }
    /// Emit a value for use on the RHS of a variable assignment.
    /// String literals are always single-quoted for safety and POSIX compliance.
    pub(crate) fn emit_assignment_value(&self, value: &ShellValue) -> Result<String> {
        match value {
            ShellValue::String(s) => {
                if s.is_empty() {
                    Ok("''".to_string())
                } else if !s.contains('\'') {
                    Ok(format!("'{s}'"))
                } else {
                    Ok(escape_shell_string(s))
                }
            }
            _ => self.emit_shell_value(value),
        }
    }
    pub(crate) fn emit_exec_statement(
        &self,
        output: &mut String,
        cmd: &Command,
        indent: usize,
    ) -> Result<()> {
        let choice = if cmd.program == "echo" || cmd.program == "printf" {
            "builtin_echo"
        } else if cmd.program.starts_with("rash_") {
            "runtime_call"
        } else {
            "external_cmd"
        };
        self.record_decision("exec_command", choice, "Exec");
        let indent_str = get_indent(indent + 1);
        let command_str = self.emit_command(cmd)?;
        writeln!(output, "{indent_str}{command_str}")?;
        Ok(())
    }
    pub(crate) fn emit_if_statement(
        &self,
        output: &mut String,
        test: &ShellValue,
        then_branch: &ShellIR,
        else_branch: Option<&ShellIR>,
        indent: usize,
    ) -> Result<()> {
        self.record_decision("if_structure", classify_if_structure(else_branch), "If");
        let indent_str = get_indent(indent + 1);
        let test_expr = self.emit_test_expression(test)?;
        writeln!(output, "{indent_str}if {test_expr}; then")?;
        self.emit_ir(output, then_branch, indent + 1)?;
        if let Some(else_ir) = else_branch {
            self.emit_else_branch(output, else_ir, indent)?;
        }
        writeln!(output, "{indent_str}fi")?;
        Ok(())
    }
    /// Emit the else portion of an if statement, collapsing chained if-else-if into elif.
    pub(crate) fn emit_else_branch(
        &self,
        output: &mut String,
        else_ir: &ShellIR,
        indent: usize,
    ) -> Result<()> {
        let indent_str = get_indent(indent + 1);
        let unwrapped = unwrap_single_if(else_ir);
        if let ShellIR::If {
            test: elif_test,
            then_branch: elif_then,
            else_branch: elif_else,
        } = unwrapped
        {
            let elif_expr = self.emit_test_expression(elif_test)?;
            writeln!(output, "{indent_str}elif {elif_expr}; then")?;
            self.emit_ir(output, elif_then, indent + 1)?;
            if let Some(final_else) = elif_else {
                self.emit_elif_chain(output, final_else, indent)?;
            }
        } else {
            writeln!(output, "{indent_str}else")?;
            self.emit_ir(output, else_ir, indent + 1)?;
        }
        Ok(())
    }
    pub(crate) fn emit_elif_chain(
        &self,
        output: &mut String,
        else_ir: &ShellIR,
        indent: usize,
    ) -> Result<()> {
        let indent_str = get_indent(indent + 1);
        let unwrapped = unwrap_single_if(else_ir);
        if let ShellIR::If {
            test: elif_test,
            then_branch: elif_then,
            else_branch: elif_else,
        } = unwrapped
        {
            let elif_expr = self.emit_test_expression(elif_test)?;
            writeln!(output, "{indent_str}elif {elif_expr}; then")?;
            self.emit_ir(output, elif_then, indent + 1)?;
            if let Some(final_else) = elif_else {
                self.emit_elif_chain(output, final_else, indent)?;
            }
        } else {
            writeln!(output, "{indent_str}else")?;
            self.emit_ir(output, else_ir, indent + 1)?;
        }
        Ok(())
    }
    pub(crate) fn emit_exit_statement(
        &self,
        output: &mut String,
        code: i32,
        message: Option<&String>,
        indent: usize,
    ) -> Result<()> {
        let indent_str = get_indent(indent + 1);
        if let Some(msg) = message {
            let escaped_msg = escape_shell_string(msg);
            writeln!(output, "{indent_str}echo {escaped_msg} >&2")?;
        }
        writeln!(output, "{indent_str}exit {code}")?;
        Ok(())
    }
    pub(crate) fn emit_sequence(
        &self,
        output: &mut String,
        items: &[ShellIR],
        indent: usize,
    ) -> Result<()> {
        if items.is_empty() {
            // Empty sequence needs at least ':' for valid POSIX syntax
            self.emit_noop(output, indent)?;
        } else {
            for item in items {
                self.emit_ir(output, item, indent)?;
            }
        }
        Ok(())
    }
    pub(crate) fn emit_noop(&self, output: &mut String, indent: usize) -> Result<()> {
        let indent_str = get_indent(indent + 1);
        // Use ':' (true command) instead of comment for valid POSIX syntax
        writeln!(output, "{indent_str}:")?;
        Ok(())
    }
    pub(crate) fn emit_echo_statement(
        &self,
        output: &mut String,
        value: &ShellValue,
        indent: usize,
    ) -> Result<()> {
        let indent_str = get_indent(indent + 1);
        let value_str = self.emit_shell_value(value)?;
        // Use echo to return value from function
        writeln!(output, "{indent_str}echo {value_str}")?;
        Ok(())
    }
    pub(crate) fn emit_for_statement(
        &self,
        output: &mut String,
        var: &str,
        start: &ShellValue,
        end: &ShellValue,
        body: &ShellIR,
        indent: usize,
    ) -> Result<()> {
        self.record_decision("for_construct", "seq_range", "For");
        let indent_str = get_indent(indent + 1);
        let var_name = escape_variable_name(var);
        // Emit shell values for start and end
        let start_str = self.emit_shell_value(start)?;
        let end_str = self.emit_shell_value(end)?;
        // Generate POSIX for loop using seq
        // for i in $(seq 0 2); do
        writeln!(
            output,
            "{indent_str}for {var_name} in $(seq {start_str} {end_str}); do"
        )?;
        // Emit body
        self.emit_ir(output, body, indent + 1)?;
        // Close loop
        writeln!(output, "{indent_str}done")?;
        Ok(())
    }
    pub(crate) fn emit_for_in_statement(
        &self,
        output: &mut String,
        var: &str,
        items: &[ShellValue],
        body: &ShellIR,
        indent: usize,
    ) -> Result<()> {
        self.record_decision("for_construct", "for_in_list", "ForIn");
        let indent_str = get_indent(indent + 1);
        let var_name = escape_variable_name(var);

        // Emit: for var in item1 item2 item3; do
        let items_str: Vec<String> = items
            .iter()
            .map(|item| self.emit_shell_value(item))
            .collect::<Result<Vec<_>>>()?;
        let items_joined = items_str.join(" ");

        writeln!(output, "{indent_str}for {var_name} in {items_joined}; do")?;

        // Emit body
        self.emit_ir(output, body, indent + 1)?;

        // Close loop
        writeln!(output, "{indent_str}done")?;
        Ok(())
    }

    pub(crate) fn emit_while_statement(
        &self,
        output: &mut String,
        condition: &ShellValue,
        body: &ShellIR,
        indent: usize,
    ) -> Result<()> {
        self.record_decision("while_construct", "while_test", "While");

        let indent_str = get_indent(indent + 1);

        // Handle special cases for condition
        let condition_test = match condition {
            ShellValue::Bool(true) => {
                // while true - infinite loop
                "true".to_string()
            }
            ShellValue::Comparison { .. } => {
                // Comparison expression - use emit_shell_value which handles it
                self.emit_shell_value(condition)?
            }
            ShellValue::LogicalAnd { left, right } => {
                // Compound AND: emit as separate test commands chained with &&
                // POSIX requires: [ cond1 ] && [ cond2 ], NOT [ cond1 && cond2 ]
                let left_cond = self.emit_while_condition(left)?;
                let right_cond = self.emit_while_condition(right)?;
                format!("{left_cond} && {right_cond}")
            }
            ShellValue::LogicalOr { left, right } => {
                // Compound OR: emit as separate test commands chained with ||
                let left_cond = self.emit_while_condition(left)?;
                let right_cond = self.emit_while_condition(right)?;
                format!("{left_cond} || {right_cond}")
            }
            ShellValue::LogicalNot { operand } => {
                // Negation: emit as ! [ cond ]
                let inner = self.emit_while_condition(operand)?;
                format!("! {inner}")
            }
            _ => {
                // General expression - treat as test
                let cond_str = self.emit_shell_value(condition)?;
                format!("[ {cond_str} ]")
            }
        };

        // Emit while loop
        writeln!(output, "{indent_str}while {condition_test}; do")?;

        // Emit body
        self.emit_ir(output, body, indent + 1)?;

        // Close loop
        writeln!(output, "{indent_str}done")?;
        Ok(())
    }

    /// Emit a single while-loop condition operand, wrapping in [ ] for test expressions.
    /// Recursively handles nested LogicalAnd/Or/Not for compound conditions.
    pub(crate) fn emit_while_condition(&self, value: &ShellValue) -> Result<String> {
        match value {
            ShellValue::Bool(true) => Ok("true".to_string()),
            ShellValue::Bool(false) => Ok("false".to_string()),
            ShellValue::Comparison { .. } => self.emit_shell_value(value),
            ShellValue::LogicalAnd { left, right } => {
                let l = self.emit_while_condition(left)?;
                let r = self.emit_while_condition(right)?;
                Ok(format!("{l} && {r}"))
            }
            ShellValue::LogicalOr { left, right } => {
                let l = self.emit_while_condition(left)?;
                let r = self.emit_while_condition(right)?;
                Ok(format!("{l} || {r}"))
            }
            ShellValue::LogicalNot { operand } => {
                let inner = self.emit_while_condition(operand)?;
                Ok(format!("! {inner}"))
            }
            _ => {
                let cond_str = self.emit_shell_value(value)?;
                Ok(format!("[ {cond_str} ]"))
            }
        }
    }

    pub(crate) fn emit_case_statement(
        &self,
        output: &mut String,
        scrutinee: &ShellValue,
        arms: &[crate::ir::shell_ir::CaseArm],
        indent: usize,
    ) -> Result<()> {
        use crate::ir::shell_ir::CasePattern;

        self.record_decision("case_dispatch", "case_arms", "Case");

        let indent_str = get_indent(indent + 1);
        let scrutinee_str = self.emit_shell_value(scrutinee)?;

        // case "$x" in
        writeln!(output, "{indent_str}case {scrutinee_str} in")?;

        // Emit each case arm
        for arm in arms {
            let pattern_str = match &arm.pattern {
                CasePattern::Literal(lit) => lit.clone(),
                CasePattern::Wildcard => "*".to_string(),
            };

            // pattern)
            writeln!(output, "{}    {})", indent_str, pattern_str)?;

            // If guard is present, wrap body in an if statement
            if let Some(guard) = &arm.guard {
                let guard_str = self.emit_shell_value(guard)?;
                writeln!(output, "{}        if {guard_str}; then", indent_str)?;
                self.emit_ir(output, &arm.body, indent + 2)?;
                writeln!(output, "{}        fi", indent_str)?;
            } else {
                // Emit body with additional indentation
                self.emit_ir(output, &arm.body, indent + 1)?;
            }

            // ;;
            writeln!(output, "{}    ;;", indent_str)?;
        }

        // esac
        writeln!(output, "{indent_str}esac")?;
        Ok(())
    }

    pub(crate) fn emit_function(
        &self,
        output: &mut String,
        name: &str,
        params: &[String],
        body: &ShellIR,
        indent: usize,
    ) -> Result<()> {
        // Skip emitting function definitions for known builtins/external commands with empty bodies
        // This prevents user-defined empty stub functions from shadowing shell builtins
        let is_empty_body = matches!(body, ShellIR::Noop)
            || matches!(body, ShellIR::Sequence(items) if items.is_empty());

        if is_empty_body && self.is_known_command(name) {
            // Don't emit the function definition - calls will use the builtin/external command directly
            return Ok(());
        }

        let indent_str = get_indent(indent);
        let body_indent_str = get_indent(indent + 1);

        // Shell function definition
        writeln!(output, "{indent_str}{name}() {{")?;

        // Bind positional parameters to named variables
        for (i, param) in params.iter().enumerate() {
            let pos = i + 1;
            let param_name = escape_variable_name(param);
            // TODO: Restore readonly once proper variable shadowing is implemented
            writeln!(output, "{body_indent_str}{param_name}=\"${pos}\"")?;
        }

        if !params.is_empty() {
            writeln!(output)?;
        }

        // Emit function body
        self.emit_ir(output, body, indent + 1)?;

        writeln!(output, "{indent_str}}}")?;
        writeln!(output)?;

        Ok(())
    }

    /// Check if a name is a known shell builtin or common external command
    pub(crate) fn is_known_command(&self, name: &str) -> bool {
        // POSIX shell builtins
        const BUILTINS: &[&str] = &[
            "echo", "cd", "pwd", "test", "export", "readonly", "shift", "set", "unset", "read",
            "printf", "return", "exit", "trap", "true", "false", ":", ".", "source", "eval",
            "exec", "wait",
        ];

        // Common external commands
        const EXTERNAL_COMMANDS: &[&str] = &[
            "cat", "grep", "sed", "awk", "cut", "sort", "uniq", "wc", "ls", "cp", "mv", "rm",
            "mkdir", "rmdir", "touch", "chmod", "chown", "find", "xargs", "tar", "gzip", "curl",
            "wget", "git", "make", "docker", "ssh", "scp", "rsync",
        ];

        BUILTINS.contains(&name) || EXTERNAL_COMMANDS.contains(&name)
    }
}
