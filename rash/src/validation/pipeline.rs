use super::{ValidationError, ValidationLevel};
use crate::ast::RestrictedAst;
use crate::ir::ShellIR;
use crate::models::config::Config;
use crate::models::error::{RashError, RashResult};

pub struct ValidationPipeline {
    pub(crate) level: ValidationLevel,
    pub(crate) strict_mode: bool,
}

impl ValidationPipeline {
    pub fn new(config: &Config) -> Self {
        Self {
            level: config.validation_level.unwrap_or_default(),
            strict_mode: config.strict_mode,
        }
    }

    pub fn validate_ast(&self, ast: &RestrictedAst) -> RashResult<()> {
        if self.level == ValidationLevel::None {
            return Ok(());
        }

        // Validate all functions
        for function in &ast.functions {
            // Validate function name is not a shell builtin
            self.validate_function_name(&function.name)?;

            // Validate function body
            for stmt in &function.body {
                self.validate_stmt(stmt)?;
            }
        }
        Ok(())
    }

    pub fn validate_ir(&self, ir: &ShellIR) -> RashResult<()> {
        if self.level == ValidationLevel::None {
            return Ok(());
        }

        self.validate_ir_recursive(ir)
    }

    pub fn validate_output(&self, _shell_script: &str) -> RashResult<()> {
        if self.level == ValidationLevel::None {
            return Ok(());
        }

        #[cfg(debug_assertions)]
        self.verify_with_embedded_rules(_shell_script)?;

        Ok(())
    }

    fn validate_stmt(&self, stmt: &crate::ast::Stmt) -> RashResult<()> {
        use crate::ast::Stmt;

        match stmt {
            Stmt::Let { value, .. } => {
                self.validate_expr(value)?;
            }
            Stmt::Expr(expr) => {
                self.validate_expr(expr)?;
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                self.validate_expr(condition)?;
                for stmt in then_block {
                    self.validate_stmt(stmt)?;
                }
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.validate_stmt(stmt)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_expr(&self, expr: &crate::ast::Expr) -> RashResult<()> {
        use crate::ast::Expr;

        match expr {
            Expr::Literal(lit) => self.validate_literal(lit),
            Expr::Variable(name) => self.validate_variable_name(name),
            Expr::Binary { left, right, .. } => self.validate_binary_expr(left, right),
            Expr::Unary { operand, .. } => self.validate_expr(operand),
            Expr::FunctionCall { name, args } => self.validate_function_call(name, args),
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => self.validate_method_call(receiver, method, args),
            Expr::Array(items) => self.validate_array_items(items),
            Expr::Index { object, index } => self.validate_index_expr(object, index),
            Expr::Try { expr } => self.validate_expr(expr),
            Expr::Block(stmts) => self.validate_block_statements(stmts),
            Expr::Range { start, end, .. } => {
                self.validate_expr(start)?;
                self.validate_expr(end)
            }
            Expr::PositionalArgs => {
                // Positional arguments are valid - will be handled by emitter
                Ok(())
            }
        }
    }

    // Helper methods to reduce complexity
    fn validate_literal(&self, lit: &crate::ast::restricted::Literal) -> RashResult<()> {
        use crate::ast::restricted::Literal;

        match lit {
            Literal::Str(s) => self.validate_string_literal(s),
            Literal::Bool(_) | Literal::U32(_) | Literal::I32(_) => Ok(()),
        }
    }

    fn validate_string_literal(&self, s: &str) -> RashResult<()> {
        if self.level == ValidationLevel::None {
            return Ok(());
        }

        // Special handling for shellshock-style attacks
        if s.contains("() { :; }") {
            return Err(RashError::ValidationError(
                "Shellshock-style function definition detected in string literal".to_string(),
            ));
        }

        // Check for command injection patterns
        let dangerous_patterns = [
            (
                "; ",
                "Semicolon command separator detected in string literal",
            ),
            ("| ", "Pipe operator detected in string literal"),
            ("$(", "Command substitution detected in string literal"),
            (
                "`",
                "Backtick command substitution detected in string literal (SC2006)",
            ),
            ("&& ", "AND operator detected in string literal"),
            ("|| ", "OR operator detected in string literal"),
            (
                "'; ",
                "Quote escape with semicolon detected in string literal",
            ),
            (
                "\"; ",
                "Quote escape with semicolon detected in string literal",
            ),
            ("<<", "Here-doc syntax detected in string literal"),
            ("eval ", "eval command detected in string literal"),
            ("exec ", "exec command detected in string literal"),
        ];

        for (pattern, message) in &dangerous_patterns {
            if s.contains(pattern) {
                return Err(RashError::ValidationError(format!(
                    "{}: '{}'",
                    message,
                    s.chars().take(50).collect::<String>()
                )));
            }
        }

        // Check for newlines/carriage returns followed by dangerous commands
        if s.contains('\n') || s.contains('\r') {
            let lines: Vec<&str> = s.split(&['\n', '\r'][..]).collect();
            for line in lines {
                let trimmed = line.trim();
                // Check if any line starts with a shell command that could be dangerous
                let dangerous_starts = ["rm ", "curl ", "wget ", "eval ", "exec ", "bash ", "sh "];
                for start in &dangerous_starts {
                    if trimmed.starts_with(start) {
                        return Err(RashError::ValidationError(format!(
                            "Newline followed by dangerous command detected: '{}'",
                            trimmed.chars().take(50).collect::<String>()
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_function_name(&self, name: &str) -> RashResult<()> {
        if name.is_empty() {
            return Err(RashError::ValidationError(
                "Empty function name".to_string(),
            ));
        }

        // Check for reserved shell builtins that cannot be redefined
        // These are POSIX special builtins and common builtins that cause errors
        let reserved_builtins = [
            "break", "continue", "exit", "return", "shift", "trap", "unset", "export", "readonly",
            "set", "times", "exec", "eval", ".", ":", "true", "false", "test", "[",
        ];

        if reserved_builtins.contains(&name) {
            return Err(RashError::ValidationError(format!(
                "Function name '{}' is a reserved shell builtin and cannot be redefined",
                name
            )));
        }

        Ok(())
    }

    fn validate_variable_name(&self, name: &str) -> RashResult<()> {
        if name.is_empty() {
            return Err(RashError::ValidationError(
                "Empty variable name".to_string(),
            ));
        }
        if name.contains(char::is_whitespace) {
            return Err(RashError::ValidationError(format!(
                "Variable name '{name}' contains whitespace"
            )));
        }
        Ok(())
    }

    fn validate_binary_expr(
        &self,
        left: &crate::ast::Expr,
        right: &crate::ast::Expr,
    ) -> RashResult<()> {
        self.validate_expr(left)?;
        self.validate_expr(right)?;
        Ok(())
    }

    fn validate_function_call(&self, name: &str, args: &[crate::ast::Expr]) -> RashResult<()> {
        if name.is_empty() {
            return Err(RashError::ValidationError(
                "Empty function name".to_string(),
            ));
        }
        for arg in args {
            self.validate_expr(arg)?;
        }
        Ok(())
    }

    fn validate_method_call(
        &self,
        receiver: &crate::ast::Expr,
        method: &str,
        args: &[crate::ast::Expr],
    ) -> RashResult<()> {
        if method.is_empty() {
            return Err(RashError::ValidationError("Empty method name".to_string()));
        }
        self.validate_expr(receiver)?;
        for arg in args {
            self.validate_expr(arg)?;
        }
        Ok(())
    }

    fn validate_array_items(&self, items: &[crate::ast::Expr]) -> RashResult<()> {
        for item in items {
            self.validate_expr(item)?;
        }
        Ok(())
    }

    fn validate_index_expr(
        &self,
        object: &crate::ast::Expr,
        index: &crate::ast::Expr,
    ) -> RashResult<()> {
        self.validate_expr(object)?;
        self.validate_expr(index)?;
        Ok(())
    }

    fn validate_block_statements(&self, stmts: &[crate::ast::Stmt]) -> RashResult<()> {
        for stmt in stmts {
            self.validate_stmt(stmt)?;
        }
        Ok(())
    }

    fn validate_ir_recursive(&self, ir: &ShellIR) -> RashResult<()> {
        match ir {
            ShellIR::Let { value, .. } => {
                self.validate_shell_value(value)?;
            }
            ShellIR::Exec { cmd, .. } => {
                for arg in &cmd.args {
                    self.validate_shell_value(arg)?;
                }
            }
            ShellIR::If {
                test,
                then_branch,
                else_branch,
            } => {
                self.validate_shell_value(test)?;
                self.validate_ir_recursive(then_branch)?;
                if let Some(else_b) = else_branch {
                    self.validate_ir_recursive(else_b)?;
                }
            }
            ShellIR::Sequence(irs) => {
                for ir in irs {
                    self.validate_ir_recursive(ir)?;
                }
            }
            ShellIR::Function { body, .. } => {
                self.validate_ir_recursive(body)?;
            }
            ShellIR::Exit { .. } | ShellIR::Noop => {}
            ShellIR::Echo { value } => {
                self.validate_shell_value(value)?;
            }
            ShellIR::For {
                var,
                start,
                end,
                body,
            } => {
                // Validate variable name
                if var.is_empty() {
                    return Err(RashError::ValidationError(
                        "For loop variable name cannot be empty".to_string(),
                    ));
                }
                // Validate range values
                self.validate_shell_value(start)?;
                self.validate_shell_value(end)?;
                // Validate body
                self.validate_ir_recursive(body)?;
            }
            ShellIR::While { condition, body } => {
                // Validate condition
                self.validate_shell_value(condition)?;
                // Validate body
                self.validate_ir_recursive(body)?;
            }
            ShellIR::Case { scrutinee, arms } => {
                // Validate scrutinee
                self.validate_shell_value(scrutinee)?;

                // Validate each arm
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.validate_shell_value(guard)?;
                    }
                    self.validate_ir_recursive(&arm.body)?;
                }

                // Check for at least one arm
                if arms.is_empty() {
                    return Err(RashError::ValidationError(
                        "Match expression must have at least one arm".to_string(),
                    ));
                }
            }
            ShellIR::Break | ShellIR::Continue => {
                // Break and continue are always valid in IR
                // Context validation (must be inside loop) happens during AST validation
            }
        }
        Ok(())
    }

    pub(crate) fn validate_shell_value(&self, value: &crate::ir::ShellValue) -> RashResult<()> {
        use crate::ir::ShellValue;

        match value {
            ShellValue::Variable(_name) => {
                if self.level >= ValidationLevel::Minimal {
                    // Variables should generally be quoted in shell
                    // This is a simplified check - real implementation would check context
                }
            }
            ShellValue::CommandSubst(cmd) => {
                if cmd.program.contains('`') && self.level >= ValidationLevel::Minimal {
                    return Err(RashError::ValidationError(
                        "Use $(...) instead of backticks (SC2006)".to_string(),
                    ));
                }
            }
            ShellValue::Concat(parts) => {
                for part in parts {
                    self.validate_shell_value(part)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn validate_expression(&self, expr: &crate::ir::ShellExpression) -> RashResult<()> {
        use crate::ir::ShellExpression;

        match expr {
            ShellExpression::Variable(name, quoted) => {
                if !quoted && self.level >= ValidationLevel::Minimal {
                    return Err(RashError::ValidationError(format!(
                        "Unquoted variable ${name} (SC2086)"
                    )));
                }
            }
            ShellExpression::Command(cmd) => {
                if cmd.contains('`') && self.level >= ValidationLevel::Minimal {
                    return Err(RashError::ValidationError(
                        "Use $(...) instead of backticks (SC2006)".to_string(),
                    ));
                }
            }
            ShellExpression::String(_) => {}
            ShellExpression::Arithmetic(_) => {}
        }
        Ok(())
    }

    #[cfg(debug_assertions)]
    fn verify_with_embedded_rules(&self, script: &str) -> RashResult<()> {
        super::rules::validate_all(script)
    }

    pub fn report_error(&self, error: &ValidationError) -> String {
        if self.strict_mode && error.severity == super::Severity::Error {
            format!("ERROR: {error}")
        } else {
            format!("{error}")
        }
    }

    pub fn should_fail(&self, errors: &[ValidationError]) -> bool {
        if self.strict_mode {
            !errors.is_empty()
        } else {
            errors.iter().any(|e| e.severity == super::Severity::Error)
        }
    }
}
