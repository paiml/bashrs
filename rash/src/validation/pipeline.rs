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

        // Validate the main function
        for function in &ast.functions {
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
            Expr::Literal(_) => Ok(()),
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
        }
    }

    // Helper methods to reduce complexity
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
            ShellIR::Exit { .. } | ShellIR::Noop => {}
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
