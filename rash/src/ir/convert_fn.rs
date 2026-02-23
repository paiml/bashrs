//! Function-context statement conversion (`convert_stmt_in_function`,
//! `convert_stmts_in_function`).
//!
//! Extracted from `convert.rs` to reduce per-file complexity.

use super::optimizations::adjust_range_end;
use super::{IrConverter, ShellIR, ShellValue};
use crate::models::Result;

impl IrConverter {
    /// Convert a statement in a non-void function context.
    /// `should_echo`: true when this is the tail expression that should be echoed.
    /// Explicit `return expr` is always converted to `echo + return` (not exit).
    /// If statements in tail position propagate should_echo into branches.
    pub(crate) fn convert_stmt_in_function(
        &self,
        stmt: &crate::ast::Stmt,
        should_echo: bool,
    ) -> Result<ShellIR> {
        use crate::ast::Stmt;

        match stmt {
            Stmt::Expr(expr) if should_echo => {
                let value = self.convert_expr_to_value(expr)?;
                Ok(ShellIR::Echo { value })
            }
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => self.convert_if_in_fn(condition, then_block, else_block.as_deref(), should_echo),
            Stmt::Return(Some(expr)) => self.convert_return_expr_in_fn(expr),
            Stmt::Return(None) => Ok(ShellIR::Return { value: None }),
            Stmt::While {
                condition, body, ..
            } => self.convert_while_in_fn(condition, body),
            Stmt::For {
                pattern, iter, body, ..
            } => self.convert_for_in_fn(pattern, iter, body),
            Stmt::Match { scrutinee, arms } => self.convert_match_in_fn(scrutinee, arms, should_echo),
            _ => self.convert_stmt(stmt),
        }
    }

    fn convert_if_in_fn(
        &self,
        condition: &crate::ast::Expr,
        then_block: &[crate::ast::Stmt],
        else_block: Option<&[crate::ast::Stmt]>,
        should_echo: bool,
    ) -> Result<ShellIR> {
        let test_expr = self.convert_expr_to_value(condition)?;
        let then_ir = self.convert_stmts_in_function(then_block, should_echo)?;
        let else_ir = if let Some(else_stmts) = else_block {
            Some(Box::new(self.convert_stmts_in_function(else_stmts, should_echo)?))
        } else {
            None
        };
        Ok(ShellIR::If {
            test: test_expr,
            then_branch: Box::new(then_ir),
            else_branch: else_ir,
        })
    }

    fn convert_return_expr_in_fn(&self, expr: &crate::ast::Expr) -> Result<ShellIR> {
        if let crate::ast::Expr::FunctionCall { name, args } = expr {
            if name == "__if_expr" && args.len() == 3 {
                return self.lower_return_if_expr(args);
            }
        }
        let value = self.convert_expr_to_value(expr)?;
        Ok(ShellIR::Return { value: Some(value) })
    }

    fn convert_while_in_fn(
        &self,
        condition: &crate::ast::Expr,
        body: &[crate::ast::Stmt],
    ) -> Result<ShellIR> {
        let condition_value = self.convert_expr_to_value(condition)?;
        let shadow_vars = self.detect_shadows(body);
        let body_ir = if shadow_vars.is_empty() {
            self.convert_stmts_in_function(body, false)?
        } else {
            self.convert_loop_body_with_shadows(body, &shadow_vars)?
        };
        let while_ir = ShellIR::While {
            condition: condition_value,
            body: Box::new(body_ir),
        };
        self.wrap_with_shadow_save_restore(while_ir, &shadow_vars)
    }

    fn convert_for_in_fn(
        &self,
        pattern: &crate::ast::restricted::Pattern,
        iter: &crate::ast::Expr,
        body: &[crate::ast::Stmt],
    ) -> Result<ShellIR> {
        let var = match pattern {
            crate::ast::restricted::Pattern::Variable(name) => name.clone(),
            _ => {
                return Err(crate::models::Error::Validation(
                    "Only simple variable patterns supported in for loops".to_string(),
                ))
            }
        };

        let shadow_vars = self.detect_shadows(body);
        let body_ir = if shadow_vars.is_empty() {
            self.convert_stmts_in_function(body, false)?
        } else {
            self.convert_loop_body_with_shadows(body, &shadow_vars)?
        };

        let loop_ir = self.build_fn_loop_ir(&var, iter, body_ir)?;
        self.wrap_with_shadow_save_restore(loop_ir, &shadow_vars)
    }

    fn build_fn_loop_ir(
        &self,
        var: &str,
        iter: &crate::ast::Expr,
        body_ir: ShellIR,
    ) -> Result<ShellIR> {
        match iter {
            crate::ast::Expr::Range {
                start, end, inclusive,
            } => {
                let start_val = self.convert_expr_to_value(start)?;
                let end_val = self.convert_expr_to_value(end)?;
                let adjusted_end = adjust_range_end(end_val, *inclusive);
                Ok(ShellIR::For {
                    var: var.to_string(),
                    start: start_val,
                    end: adjusted_end,
                    body: Box::new(body_ir),
                })
            }
            crate::ast::Expr::Array(elements) => {
                let items: Vec<ShellValue> = elements
                    .iter()
                    .map(|e| self.convert_expr_to_value(e))
                    .collect::<Result<_>>()?;
                Ok(ShellIR::ForIn { var: var.to_string(), items, body: Box::new(body_ir) })
            }
            crate::ast::Expr::Variable(name) => {
                let array_len = self.arrays.borrow().get(name).copied();
                if let Some(len) = array_len {
                    let items: Vec<ShellValue> = (0..len)
                        .map(|i| ShellValue::Variable(format!("{}_{}", name, i)))
                        .collect();
                    Ok(ShellIR::ForIn { var: var.to_string(), items, body: Box::new(body_ir) })
                } else {
                    Ok(ShellIR::ForIn {
                        var: var.to_string(),
                        items: vec![ShellValue::Variable(name.clone())],
                        body: Box::new(body_ir),
                    })
                }
            }
            other => {
                let val = self.convert_expr_to_value(other)?;
                Ok(ShellIR::ForIn { var: var.to_string(), items: vec![val], body: Box::new(body_ir) })
            }
        }
    }

    fn convert_match_in_fn(
        &self,
        scrutinee: &crate::ast::Expr,
        arms: &[crate::ast::restricted::MatchArm],
        should_echo: bool,
    ) -> Result<ShellIR> {
        let scrutinee_value = self.convert_expr_to_value(scrutinee)?;
        if Self::has_range_patterns(arms) {
            return self.convert_range_match_fn(scrutinee_value, arms, should_echo);
        }
        let mut case_arms = Vec::new();
        for arm in arms {
            let pattern = self.convert_match_pattern(&arm.pattern)?;
            let guard = if let Some(guard_expr) = &arm.guard {
                Some(self.convert_expr_to_value(guard_expr)?)
            } else {
                None
            };
            let body = self.convert_stmts_in_function(&arm.body, should_echo)?;
            case_arms.push(super::shell_ir::CaseArm {
                pattern,
                guard,
                body: Box::new(body),
            });
        }
        Ok(ShellIR::Case {
            scrutinee: scrutinee_value,
            arms: case_arms,
        })
    }

    /// Convert a block of statements in function context, with the last
    /// statement receiving `should_echo` treatment for implicit returns.
    pub(crate) fn convert_stmts_in_function(
        &self,
        stmts: &[crate::ast::Stmt],
        should_echo: bool,
    ) -> Result<ShellIR> {
        let mut ir_stmts = Vec::new();
        for (i, stmt) in stmts.iter().enumerate() {
            let is_last = i == stmts.len() - 1;
            ir_stmts.push(self.convert_stmt_in_function(stmt, is_last && should_echo)?);
        }
        Ok(ShellIR::Sequence(ir_stmts))
    }
}
