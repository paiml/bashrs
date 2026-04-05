impl IrConverter {

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
