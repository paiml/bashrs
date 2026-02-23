//! Match/pattern handling methods for `IrConverter`.
//!
//! Contains: `convert_match_pattern`, `lower_let_match`, `convert_match_arm_for_let`,
//! `lower_let_if`, `convert_block_for_let`, `has_range_patterns`, `literal_to_string`,
//! `pattern_to_condition`, `convert_range_match`, `convert_range_match_fn`,
//! `convert_range_match_for_let`, `lower_let_if_expr`, `lower_return_if_expr`.

use super::shell_ir;
use super::{EffectSet, IrConverter, ShellIR, ShellValue};
use crate::models::Result;

impl IrConverter {
    pub(crate) fn convert_match_pattern(
        &self,
        pattern: &crate::ast::restricted::Pattern,
    ) -> Result<shell_ir::CasePattern> {
        use crate::ast::restricted::{Literal, Pattern};

        match pattern {
            Pattern::Literal(literal) => {
                // Convert literal to string representation for case pattern
                let lit_str = match literal {
                    Literal::Bool(b) => b.to_string(),
                    Literal::U16(n) => n.to_string(),
                    Literal::U32(n) => n.to_string(),
                    Literal::I32(n) => n.to_string(),
                    Literal::Str(s) => s.clone(),
                };
                Ok(shell_ir::CasePattern::Literal(lit_str))
            }
            Pattern::Wildcard => Ok(shell_ir::CasePattern::Wildcard),
            Pattern::Variable(_) => {
                // Variables in patterns are treated as wildcards for now
                // (proper binding would require more complex analysis)
                Ok(shell_ir::CasePattern::Wildcard)
            }
            Pattern::Range { .. } => {
                // Range patterns are handled via guards — emit wildcard here
                Ok(shell_ir::CasePattern::Wildcard)
            }
            Pattern::Tuple(_) | Pattern::Struct { .. } => Err(crate::models::Error::Validation(
                "Tuple and struct patterns not yet supported in match expressions".to_string(),
            )),
        }
    }

    /// Extract the value expression from a match arm body.
    /// Lower `let target = match scrutinee { arms }` into a Case where each arm
    /// assigns to `target`. Handles nested match expressions and block bodies.
    pub(crate) fn lower_let_match(
        &self,
        target: &str,
        scrutinee: &crate::ast::Expr,
        arms: &[crate::ast::restricted::MatchArm],
    ) -> Result<ShellIR> {
        let scrutinee_value = self.convert_expr_to_value(scrutinee)?;

        // Range patterns require if-elif-else chain (case can't do ranges)
        if Self::has_range_patterns(arms) {
            return self.convert_range_match_for_let(target, scrutinee_value, arms);
        }

        let mut case_arms = Vec::new();
        for arm in arms {
            let pattern = self.convert_match_pattern(&arm.pattern)?;
            let guard = if let Some(guard_expr) = &arm.guard {
                Some(self.convert_expr_to_value(guard_expr)?)
            } else {
                None
            };
            let body_ir = self.convert_match_arm_for_let(target, &arm.body)?;
            case_arms.push(shell_ir::CaseArm {
                pattern,
                guard,
                body: Box::new(body_ir),
            });
        }
        Ok(ShellIR::Case {
            scrutinee: scrutinee_value,
            arms: case_arms,
        })
    }

    /// Convert a match arm body into IR that assigns the result to `target`.
    /// Handles: simple expressions, nested match, and block bodies with
    /// multiple statements.
    pub(crate) fn convert_match_arm_for_let(
        &self,
        target: &str,
        body: &[crate::ast::Stmt],
    ) -> Result<ShellIR> {
        if body.is_empty() {
            return Ok(ShellIR::Let {
                name: target.to_string(),
                value: ShellValue::String("0".to_string()),
                effects: EffectSet::pure(),
            });
        }

        // Single statement arm
        if body.len() == 1 {
            match &body[0] {
                // Simple expression → target=value
                crate::ast::Stmt::Expr(expr) => {
                    let val = self.convert_expr_to_value(expr)?;
                    return Ok(ShellIR::Let {
                        name: target.to_string(),
                        value: val,
                        effects: EffectSet::pure(),
                    });
                }
                // return expr → target=value
                crate::ast::Stmt::Return(Some(expr)) => {
                    let val = self.convert_expr_to_value(expr)?;
                    return Ok(ShellIR::Let {
                        name: target.to_string(),
                        value: val,
                        effects: EffectSet::pure(),
                    });
                }
                // Nested match → recursive lower_let_match
                crate::ast::Stmt::Match {
                    scrutinee,
                    arms: inner_arms,
                } => {
                    return self.lower_let_match(target, scrutinee, inner_arms);
                }
                // If-else expression → lower_let_if
                crate::ast::Stmt::If {
                    condition,
                    then_block,
                    else_block,
                } => {
                    return self.lower_let_if(target, condition, then_block, else_block.as_deref());
                }
                _ => {}
            }
        }

        // Multiple statements: convert all but last, then handle last
        let mut ir_stmts = Vec::new();
        for stmt in &body[..body.len() - 1] {
            ir_stmts.push(self.convert_stmt(stmt)?);
        }
        let last = &body[body.len() - 1];
        match last {
            crate::ast::Stmt::Expr(expr) => {
                let val = self.convert_expr_to_value(expr)?;
                ir_stmts.push(ShellIR::Let {
                    name: target.to_string(),
                    value: val,
                    effects: EffectSet::pure(),
                });
            }
            crate::ast::Stmt::Return(Some(expr)) => {
                let val = self.convert_expr_to_value(expr)?;
                ir_stmts.push(ShellIR::Let {
                    name: target.to_string(),
                    value: val,
                    effects: EffectSet::pure(),
                });
            }
            crate::ast::Stmt::Match {
                scrutinee,
                arms: inner_arms,
            } => {
                ir_stmts.push(self.lower_let_match(target, scrutinee, inner_arms)?);
            }
            crate::ast::Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                ir_stmts.push(self.lower_let_if(
                    target,
                    condition,
                    then_block,
                    else_block.as_deref(),
                )?);
            }
            other => {
                ir_stmts.push(self.convert_stmt(other)?);
            }
        }
        Ok(ShellIR::Sequence(ir_stmts))
    }

    /// Lower `let target = if cond { then } else { else }` into an If IR node
    /// where each branch assigns to `target`.
    pub(crate) fn lower_let_if(
        &self,
        target: &str,
        condition: &crate::ast::Expr,
        then_block: &[crate::ast::Stmt],
        else_block: Option<&[crate::ast::Stmt]>,
    ) -> Result<ShellIR> {
        let test_expr = self.convert_expr_to_value(condition)?;
        let then_ir = self.convert_block_for_let(target, then_block)?;
        let else_ir = if let Some(else_stmts) = else_block {
            Some(Box::new(self.convert_block_for_let(target, else_stmts)?))
        } else {
            None
        };
        Ok(ShellIR::If {
            test: test_expr,
            then_branch: Box::new(then_ir),
            else_branch: else_ir,
        })
    }

    /// Convert a block of statements where the last expression assigns to `target`.
    fn convert_block_for_let(&self, target: &str, stmts: &[crate::ast::Stmt]) -> Result<ShellIR> {
        self.convert_match_arm_for_let(target, stmts)
    }

    /// Check if any match arm uses range patterns (requiring if-chain instead of case).
    pub(crate) fn has_range_patterns(arms: &[crate::ast::restricted::MatchArm]) -> bool {
        arms.iter()
            .any(|arm| matches!(&arm.pattern, crate::ast::restricted::Pattern::Range { .. }))
    }

    /// Convert a literal to its string representation for shell comparisons.
    fn literal_to_string(lit: &crate::ast::restricted::Literal) -> String {
        use crate::ast::restricted::Literal;
        match lit {
            Literal::Bool(b) => b.to_string(),
            Literal::U16(n) => n.to_string(),
            Literal::U32(n) => n.to_string(),
            Literal::I32(n) => n.to_string(),
            Literal::Str(s) => s.clone(),
        }
    }

    /// Convert a match pattern to an if-test condition against a scrutinee variable.
    /// Returns None for wildcard/variable patterns (which become the else branch).
    fn pattern_to_condition(
        scrutinee_ref: &ShellValue,
        pattern: &crate::ast::restricted::Pattern,
    ) -> Option<ShellValue> {
        use crate::ast::restricted::Pattern;
        match pattern {
            Pattern::Literal(lit) => {
                let lit_str = Self::literal_to_string(lit);
                Some(ShellValue::Comparison {
                    op: shell_ir::ComparisonOp::NumEq,
                    left: Box::new(scrutinee_ref.clone()),
                    right: Box::new(ShellValue::String(lit_str)),
                })
            }
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let start_str = Self::literal_to_string(start);
                let end_str = Self::literal_to_string(end);
                let end_op = if *inclusive {
                    shell_ir::ComparisonOp::Le
                } else {
                    shell_ir::ComparisonOp::Lt
                };
                Some(ShellValue::LogicalAnd {
                    left: Box::new(ShellValue::Comparison {
                        op: shell_ir::ComparisonOp::Ge,
                        left: Box::new(scrutinee_ref.clone()),
                        right: Box::new(ShellValue::String(start_str)),
                    }),
                    right: Box::new(ShellValue::Comparison {
                        op: end_op,
                        left: Box::new(scrutinee_ref.clone()),
                        right: Box::new(ShellValue::String(end_str)),
                    }),
                })
            }
            Pattern::Wildcard | Pattern::Variable(_) => None,
            _ => None,
        }
    }

    /// Convert a match with range patterns to an if-elif-else chain (non-function context).
    pub(crate) fn convert_range_match(
        &self,
        scrutinee_value: ShellValue,
        arms: &[crate::ast::restricted::MatchArm],
    ) -> Result<ShellIR> {
        let temp_var = "__match_val".to_string();
        let temp_let = ShellIR::Let {
            name: temp_var.clone(),
            value: scrutinee_value,
            effects: EffectSet::pure(),
        };
        let temp_ref = ShellValue::Variable(temp_var);

        let mut result: Option<ShellIR> = None;
        for arm in arms.iter().rev() {
            let body = self.convert_stmts(&arm.body)?;

            if let Some(cond) = Self::pattern_to_condition(&temp_ref, &arm.pattern) {
                result = Some(ShellIR::If {
                    test: cond,
                    then_branch: Box::new(body),
                    else_branch: result.map(Box::new),
                });
            } else {
                result = Some(body);
            }
        }

        Ok(ShellIR::Sequence(vec![
            temp_let,
            result.unwrap_or(ShellIR::Noop),
        ]))
    }

    /// Convert a match with range patterns in function context, propagating should_echo.
    pub(crate) fn convert_range_match_fn(
        &self,
        scrutinee_value: ShellValue,
        arms: &[crate::ast::restricted::MatchArm],
        should_echo: bool,
    ) -> Result<ShellIR> {
        let temp_var = "__match_val".to_string();
        let temp_let = ShellIR::Let {
            name: temp_var.clone(),
            value: scrutinee_value,
            effects: EffectSet::pure(),
        };
        let temp_ref = ShellValue::Variable(temp_var);

        let mut result: Option<ShellIR> = None;
        for arm in arms.iter().rev() {
            let body = self.convert_stmts_in_function(&arm.body, should_echo)?;

            if let Some(cond) = Self::pattern_to_condition(&temp_ref, &arm.pattern) {
                result = Some(ShellIR::If {
                    test: cond,
                    then_branch: Box::new(body),
                    else_branch: result.map(Box::new),
                });
            } else {
                result = Some(body);
            }
        }

        Ok(ShellIR::Sequence(vec![
            temp_let,
            result.unwrap_or(ShellIR::Noop),
        ]))
    }

    /// Convert a let-match with range patterns to an if-elif-else chain
    /// where each branch assigns to `target`.
    fn convert_range_match_for_let(
        &self,
        target: &str,
        scrutinee_value: ShellValue,
        arms: &[crate::ast::restricted::MatchArm],
    ) -> Result<ShellIR> {
        let temp_var = "__match_val".to_string();
        let temp_let = ShellIR::Let {
            name: temp_var.clone(),
            value: scrutinee_value,
            effects: EffectSet::pure(),
        };
        let temp_ref = ShellValue::Variable(temp_var);

        let mut result: Option<ShellIR> = None;
        for arm in arms.iter().rev() {
            let body = self.convert_match_arm_for_let(target, &arm.body)?;

            if let Some(cond) = Self::pattern_to_condition(&temp_ref, &arm.pattern) {
                result = Some(ShellIR::If {
                    test: cond,
                    then_branch: Box::new(body),
                    else_branch: result.map(Box::new),
                });
            } else {
                result = Some(body);
            }
        }

        Ok(ShellIR::Sequence(vec![
            temp_let,
            result.unwrap_or(ShellIR::Noop),
        ]))
    }

    /// Lower `let target = __if_expr(cond, then_val, else_val)` into If IR.
    /// Handles nested else-if chains where else_val is itself `__if_expr(...)`.
    pub(crate) fn lower_let_if_expr(
        &self,
        target: &str,
        args: &[crate::ast::Expr],
    ) -> Result<ShellIR> {
        let cond_val = self.convert_expr_to_value(&args[0])?;

        // Check if then-value is a nested __if_expr (nested conditional in then-branch)
        let then_ir = if let crate::ast::Expr::FunctionCall {
            name: fn_name,
            args: inner_args,
        } = &args[1]
        {
            if fn_name == "__if_expr" && inner_args.len() == 3 {
                self.lower_let_if_expr(target, inner_args)?
            } else {
                let then_val = self.convert_expr_to_value(&args[1])?;
                ShellIR::Let {
                    name: target.to_string(),
                    value: then_val,
                    effects: EffectSet::pure(),
                }
            }
        } else {
            let then_val = self.convert_expr_to_value(&args[1])?;
            ShellIR::Let {
                name: target.to_string(),
                value: then_val,
                effects: EffectSet::pure(),
            }
        };

        // Check if else-value is a nested __if_expr (else-if chain)
        let else_ir = if let crate::ast::Expr::FunctionCall {
            name: fn_name,
            args: inner_args,
        } = &args[2]
        {
            if fn_name == "__if_expr" && inner_args.len() == 3 {
                // Recursive: else-if chain
                self.lower_let_if_expr(target, inner_args)?
            } else {
                let else_val = self.convert_expr_to_value(&args[2])?;
                ShellIR::Let {
                    name: target.to_string(),
                    value: else_val,
                    effects: EffectSet::pure(),
                }
            }
        } else {
            let else_val = self.convert_expr_to_value(&args[2])?;
            ShellIR::Let {
                name: target.to_string(),
                value: else_val,
                effects: EffectSet::pure(),
            }
        };

        Ok(ShellIR::If {
            test: cond_val,
            then_branch: Box::new(then_ir),
            else_branch: Some(Box::new(else_ir)),
        })
    }

    /// Lower `return __if_expr(cond, then_val, else_val)` into If IR with Return in each branch.
    /// Handles nested else-if chains where else_val is itself `__if_expr(...)`.
    pub(crate) fn lower_return_if_expr(&self, args: &[crate::ast::Expr]) -> Result<ShellIR> {
        let cond_val = self.convert_expr_to_value(&args[0])?;

        // Check if then-value is a nested __if_expr
        let then_ir = if let crate::ast::Expr::FunctionCall {
            name: fn_name,
            args: inner_args,
        } = &args[1]
        {
            if fn_name == "__if_expr" && inner_args.len() == 3 {
                self.lower_return_if_expr(inner_args)?
            } else {
                let then_val = self.convert_expr_to_value(&args[1])?;
                ShellIR::Return {
                    value: Some(then_val),
                }
            }
        } else {
            let then_val = self.convert_expr_to_value(&args[1])?;
            ShellIR::Return {
                value: Some(then_val),
            }
        };

        // Check if else-value is a nested __if_expr (else-if chain)
        let else_ir = if let crate::ast::Expr::FunctionCall {
            name: fn_name,
            args: inner_args,
        } = &args[2]
        {
            if fn_name == "__if_expr" && inner_args.len() == 3 {
                self.lower_return_if_expr(inner_args)?
            } else {
                let else_val = self.convert_expr_to_value(&args[2])?;
                ShellIR::Return {
                    value: Some(else_val),
                }
            }
        } else {
            let else_val = self.convert_expr_to_value(&args[2])?;
            ShellIR::Return {
                value: Some(else_val),
            }
        };

        Ok(ShellIR::If {
            test: cond_val,
            then_branch: Box::new(then_ir),
            else_branch: Some(Box::new(else_ir)),
        })
    }
}
