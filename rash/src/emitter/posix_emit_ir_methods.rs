impl super::posix::PosixEmitter {

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
