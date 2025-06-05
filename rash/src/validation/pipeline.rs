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

    fn validate_expr(&self, _expr: &crate::ast::Expr) -> RashResult<()> {
        // TODO: Implement expression validation
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
    fn validate_expression(&self, expr: &crate::ir::ShellExpression) -> RashResult<()> {
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
            _ => {}
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
