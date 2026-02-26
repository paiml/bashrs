//! Top-level statement conversion (`convert_stmt`, `convert_stmts`).
//!
//! Extracted from `convert.rs` to reduce per-file complexity.

use super::optimizations::adjust_range_end;
use super::{EffectSet, IrConverter, ShellIR, ShellValue};
use crate::models::Result;

impl IrConverter {
    pub(crate) fn convert_stmt(&self, stmt: &crate::ast::Stmt) -> Result<ShellIR> {
        use crate::ast::Stmt;

        match stmt {
            Stmt::Let {
                name,
                value,
                declaration,
            } => self.convert_let_stmt(name, value, *declaration),
            Stmt::Expr(expr) => self.convert_expr(expr),
            Stmt::Return(Some(expr)) => {
                let value = self.convert_expr_to_value(expr)?;
                Ok(ShellIR::Exit {
                    code: 0,
                    message: Some(format!("{value:?}")),
                })
            }
            Stmt::Return(None) => Ok(ShellIR::Exit {
                code: 0,
                message: None,
            }),
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => self.convert_if_stmt(condition, then_block, else_block.as_deref()),
            Stmt::For {
                pattern,
                iter,
                body,
                ..
            } => self.convert_for_stmt(pattern, iter, body),
            Stmt::Match { scrutinee, arms } => self.convert_match_stmt(scrutinee, arms),
            Stmt::While {
                condition, body, ..
            } => self.convert_while_stmt(condition, body),
            Stmt::Break => Ok(ShellIR::Break),
            Stmt::Continue => Ok(ShellIR::Continue),
        }
    }

    fn convert_let_stmt(
        &self,
        name: &str,
        value: &crate::ast::Expr,
        declaration: bool,
    ) -> Result<ShellIR> {
        // Handle __if_expr: let x = if cond { a } else { b }
        if let crate::ast::Expr::FunctionCall {
            name: fn_name,
            args,
        } = value
        {
            if fn_name == "__if_expr" && args.len() == 3 {
                return self.lower_let_if_expr(name, args);
            }
        }

        // Handle array initialization: let arr = [a, b, c]
        if let crate::ast::Expr::Array(elems) = value {
            return self.convert_let_array(name, elems);
        }

        // Handle let x = match/if/block expressions
        if let crate::ast::Expr::Block(block_stmts) = value {
            return self.convert_let_block(name, block_stmts);
        }

        let shell_value = self.convert_expr_to_value(value)?;
        if declaration {
            self.declared_vars.borrow_mut().insert(name.to_string());
        }
        Ok(ShellIR::Let {
            name: name.to_string(),
            value: shell_value,
            effects: EffectSet::pure(),
        })
    }

    fn convert_let_array(&self, name: &str, elems: &[crate::ast::Expr]) -> Result<ShellIR> {
        self.arrays
            .borrow_mut()
            .insert(name.to_string(), elems.len());
        let mut stmts = Vec::new();
        for (i, elem) in elems.iter().enumerate() {
            let elem_val = self.convert_expr_to_value(elem)?;
            stmts.push(ShellIR::Let {
                name: format!("{}_{}", name, i),
                value: elem_val,
                effects: EffectSet::pure(),
            });
        }
        Ok(ShellIR::Sequence(stmts))
    }

    fn convert_let_block(&self, name: &str, block_stmts: &[crate::ast::Stmt]) -> Result<ShellIR> {
        if block_stmts.len() == 1 {
            if let crate::ast::Stmt::Match { scrutinee, arms } = &block_stmts[0] {
                return self.lower_let_match(name, scrutinee, arms);
            }
            if let crate::ast::Stmt::If {
                condition,
                then_block,
                else_block,
            } = &block_stmts[0]
            {
                return self.lower_let_if(name, condition, then_block, else_block.as_deref());
            }
        }
        if block_stmts.len() > 1 {
            return self.convert_match_arm_for_let(name, block_stmts);
        }
        // Single non-match/non-if block statement — shouldn't normally happen
        let shell_value =
            self.convert_expr_to_value(&crate::ast::Expr::Block(block_stmts.to_vec()))?;
        Ok(ShellIR::Let {
            name: name.to_string(),
            value: shell_value,
            effects: EffectSet::pure(),
        })
    }

    fn convert_if_stmt(
        &self,
        condition: &crate::ast::Expr,
        then_block: &[crate::ast::Stmt],
        else_block: Option<&[crate::ast::Stmt]>,
    ) -> Result<ShellIR> {
        let test_expr = self.convert_expr_to_value(condition)?;
        let then_ir = self.convert_stmts(then_block)?;
        let else_ir = if let Some(else_stmts) = else_block {
            Some(Box::new(self.convert_stmts(else_stmts)?))
        } else {
            None
        };
        Ok(ShellIR::If {
            test: test_expr,
            then_branch: Box::new(then_ir),
            else_branch: else_ir,
        })
    }

    fn convert_for_stmt(
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

        match iter {
            crate::ast::Expr::Range {
                start,
                end,
                inclusive,
            } => self.convert_for_range(&var, start, end, *inclusive, body),
            other => self.convert_for_iterable(&var, other, body),
        }
    }

    fn convert_for_range(
        &self,
        var: &str,
        start: &crate::ast::Expr,
        end: &crate::ast::Expr,
        inclusive: bool,
        body: &[crate::ast::Stmt],
    ) -> Result<ShellIR> {
        let start_val = self.convert_expr_to_value(start)?;
        let end_val = self.convert_expr_to_value(end)?;
        let adjusted_end = adjust_range_end(end_val, inclusive);
        let shadow_vars = self.detect_shadows(body);
        let body_ir = self.convert_loop_body_with_shadows(body, &shadow_vars)?;
        let for_ir = ShellIR::For {
            var: var.to_string(),
            start: start_val,
            end: adjusted_end,
            body: Box::new(body_ir),
        };
        self.wrap_with_shadow_save_restore(for_ir, &shadow_vars)
    }

    fn convert_for_iterable(
        &self,
        var: &str,
        iter: &crate::ast::Expr,
        body: &[crate::ast::Stmt],
    ) -> Result<ShellIR> {
        let shadow_vars = self.detect_shadows(body);
        let body_ir = self.convert_loop_body_with_shadows(body, &shadow_vars)?;
        let loop_ir = match iter {
            crate::ast::Expr::Array(elements) => {
                let items: Vec<ShellValue> = elements
                    .iter()
                    .map(|e| self.convert_expr_to_value(e))
                    .collect::<Result<_>>()?;
                ShellIR::ForIn {
                    var: var.to_string(),
                    items,
                    body: Box::new(body_ir),
                }
            }
            crate::ast::Expr::Variable(name) => {
                let array_len = self.arrays.borrow().get(name).copied();
                if let Some(len) = array_len {
                    let items: Vec<ShellValue> = (0..len)
                        .map(|i| ShellValue::Variable(format!("{}_{}", name, i)))
                        .collect();
                    ShellIR::ForIn {
                        var: var.to_string(),
                        items,
                        body: Box::new(body_ir),
                    }
                } else {
                    ShellIR::ForIn {
                        var: var.to_string(),
                        items: vec![ShellValue::Variable(name.clone())],
                        body: Box::new(body_ir),
                    }
                }
            }
            _ => {
                let val = self.convert_expr_to_value(iter)?;
                ShellIR::ForIn {
                    var: var.to_string(),
                    items: vec![val],
                    body: Box::new(body_ir),
                }
            }
        };
        self.wrap_with_shadow_save_restore(loop_ir, &shadow_vars)
    }

    fn convert_match_stmt(
        &self,
        scrutinee: &crate::ast::Expr,
        arms: &[crate::ast::restricted::MatchArm],
    ) -> Result<ShellIR> {
        let scrutinee_value = self.convert_expr_to_value(scrutinee)?;
        if Self::has_range_patterns(arms) {
            return self.convert_range_match(scrutinee_value, arms);
        }
        let mut case_arms = Vec::new();
        for arm in arms {
            let pattern = self.convert_match_pattern(&arm.pattern)?;
            let guard = if let Some(guard_expr) = &arm.guard {
                Some(self.convert_expr_to_value(guard_expr)?)
            } else {
                None
            };
            let body = self.convert_stmts(&arm.body)?;
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

    fn convert_while_stmt(
        &self,
        condition: &crate::ast::Expr,
        body: &[crate::ast::Stmt],
    ) -> Result<ShellIR> {
        let condition_value = self.convert_expr_to_value(condition)?;
        let shadow_vars = self.detect_shadows(body);
        let body_ir = self.convert_loop_body_with_shadows(body, &shadow_vars)?;
        let while_ir = ShellIR::While {
            condition: condition_value,
            body: Box::new(body_ir),
        };
        self.wrap_with_shadow_save_restore(while_ir, &shadow_vars)
    }

    pub(crate) fn convert_stmts(&self, stmts: &[crate::ast::Stmt]) -> Result<ShellIR> {
        let mut ir_stmts = Vec::new();
        for stmt in stmts {
            ir_stmts.push(self.convert_stmt(stmt)?);
        }
        Ok(ShellIR::Sequence(ir_stmts))
    }
}
