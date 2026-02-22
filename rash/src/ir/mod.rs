//! Intermediate Representation (IR) module
//!
//! ## Safety Note
//! IR operations use fallible indexing with proper error handling.
//! Production code MUST NOT use unwrap() (Cloudflare-class defect prevention).

pub mod dockerfile_ir;
pub mod effects;
pub mod shell_ir;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod shell_ir_tests;

#[cfg(test)]
mod control_flow_tests;

pub use effects::{Effect, EffectSet};
pub use shell_ir::{Command, ShellExpression, ShellIR, ShellValue};

use crate::ast::RestrictedAst;
use crate::models::{Config, Error, Result};

/// Convert AST to Shell IR
pub fn from_ast(ast: &RestrictedAst) -> Result<ShellIR> {
    let converter = IrConverter::new();
    converter.convert(ast)
}

/// Optimize Shell IR based on configuration
pub fn optimize(ir: ShellIR, config: &Config) -> Result<ShellIR> {
    if !config.optimize {
        return Ok(ir);
    }

    let mut optimized = ir;

    // Apply constant folding
    optimized = constant_fold(optimized);

    // Apply dead code elimination
    optimized = eliminate_dead_code(optimized);

    Ok(optimized)
}

struct IrConverter {
    /// Track array variables: name → element count
    /// Used to expand `for x in arr` into `for x in "$arr_0" "$arr_1" ...`
    arrays: std::cell::RefCell<std::collections::HashMap<String, usize>>,
    /// Track declared variables for shadow detection in loop bodies.
    /// When a `let x = ...` (declaration=true) appears inside a loop body
    /// and `x` is already in this set, it's a shadow that needs renaming.
    declared_vars: std::cell::RefCell<std::collections::HashSet<String>>,
}

impl IrConverter {
    fn new() -> Self {
        Self {
            arrays: std::cell::RefCell::new(std::collections::HashMap::new()),
            declared_vars: std::cell::RefCell::new(std::collections::HashSet::new()),
        }
    }

    fn convert(&self, ast: &RestrictedAst) -> Result<ShellIR> {
        let mut all_ir = Vec::new();

        // Convert all user-defined functions (except main) to shell functions
        for function in &ast.functions {
            if function.name != ast.entry_point {
                let params: Vec<String> = function.params.iter().map(|p| p.name.clone()).collect();
                let mut body_stmts = Vec::new();

                // Convert function body statements
                let has_return_type =
                    !matches!(function.return_type, crate::ast::restricted::Type::Void);
                for (i, stmt) in function.body.iter().enumerate() {
                    let is_last = i == function.body.len() - 1;
                    // Pass should_echo=true for last stmt in non-void functions;
                    // convert_stmt_in_function also handles explicit `return` at any position
                    body_stmts
                        .push(self.convert_stmt_in_function(stmt, is_last && has_return_type)?);
                }

                // Generate function with body (empty functions get Noop via emit_sequence)
                all_ir.push(ShellIR::Function {
                    name: function.name.clone(),
                    params,
                    body: Box::new(ShellIR::Sequence(body_stmts)),
                });
            }
        }

        // Find and convert the entry point function
        let entry_function = ast
            .functions
            .iter()
            .find(|f| f.name == ast.entry_point)
            .ok_or_else(|| Error::IrGeneration("Entry point not found".to_string()))?;

        // Convert the main function body
        for stmt in &entry_function.body {
            all_ir.push(self.convert_stmt(stmt)?);
        }

        Ok(ShellIR::Sequence(all_ir))
    }

    /// Convert a statement in a function context (handles return values).
    /// When `should_echo` is true, the statement is in tail position of a
    /// non-void function and its value must be echoed for capture via $().
    /// Convert a statement in a non-void function context.
    /// `should_echo`: true when this is the tail expression that should be echoed.
    /// Explicit `return expr` is always converted to `echo + return` (not exit).
    /// If statements in tail position propagate should_echo into branches.
    fn convert_stmt_in_function(
        &self,
        stmt: &crate::ast::Stmt,
        should_echo: bool,
    ) -> Result<ShellIR> {
        use crate::ast::Stmt;

        match stmt {
            // Tail expression → echo it
            Stmt::Expr(expr) if should_echo => {
                let value = self.convert_expr_to_value(expr)?;
                Ok(ShellIR::Echo { value })
            }

            // If in tail position → propagate should_echo into branches
            Stmt::If {
                condition,
                then_block,
                else_block,
            } if should_echo => {
                let test_expr = self.convert_expr_to_value(condition)?;
                let then_ir = self.convert_stmts_in_function(then_block, true)?;
                let else_ir = if let Some(else_stmts) = else_block {
                    Some(Box::new(self.convert_stmts_in_function(else_stmts, true)?))
                } else {
                    None
                };
                Ok(ShellIR::If {
                    test: test_expr,
                    then_branch: Box::new(then_ir),
                    else_branch: else_ir,
                })
            }

            // If NOT in tail position but contains returns → still need function context
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                let test_expr = self.convert_expr_to_value(condition)?;
                let then_ir = self.convert_stmts_in_function(then_block, false)?;
                let else_ir = if let Some(else_stmts) = else_block {
                    Some(Box::new(self.convert_stmts_in_function(else_stmts, false)?))
                } else {
                    None
                };
                Ok(ShellIR::If {
                    test: test_expr,
                    then_branch: Box::new(then_ir),
                    else_branch: else_ir,
                })
            }

            // Explicit return expr → echo value + return (not exit!)
            Stmt::Return(Some(expr)) => {
                // Check if the return expression is __if_expr — lower to If with Return in each branch
                if let crate::ast::Expr::FunctionCall { name, args } = expr {
                    if name == "__if_expr" && args.len() == 3 {
                        return self.lower_return_if_expr(args);
                    }
                }
                let value = self.convert_expr_to_value(expr)?;
                Ok(ShellIR::Return { value: Some(value) })
            }

            // Explicit return without value
            Stmt::Return(None) => Ok(ShellIR::Return { value: None }),

            // While loop in function context: propagate function-aware conversion
            // so that `return expr` inside the loop body becomes ShellIR::Return
            // instead of ShellIR::Exit (which would emit debug format)
            Stmt::While {
                condition, body, ..
            } => {
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

            // For loop in function context: propagate function-aware conversion
            Stmt::For {
                pattern,
                iter,
                body,
                ..
            } => {
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

                let loop_ir = match iter {
                    crate::ast::Expr::Range {
                        start,
                        end,
                        inclusive,
                    } => {
                        let start_val = self.convert_expr_to_value(start)?;
                        let end_val = self.convert_expr_to_value(end)?;
                        let adjusted_end = adjust_range_end(end_val, *inclusive);
                        ShellIR::For {
                            var,
                            start: start_val,
                            end: adjusted_end,
                            body: Box::new(body_ir),
                        }
                    }
                    crate::ast::Expr::Array(elements) => {
                        let items: Vec<ShellValue> = elements
                            .iter()
                            .map(|e| self.convert_expr_to_value(e))
                            .collect::<Result<_>>()?;
                        ShellIR::ForIn {
                            var,
                            items,
                            body: Box::new(body_ir),
                        }
                    }
                    crate::ast::Expr::Variable(name) => {
                        // Check if variable is a known array — expand to elements
                        let array_len = self.arrays.borrow().get(name).copied();
                        if let Some(len) = array_len {
                            let items: Vec<ShellValue> = (0..len)
                                .map(|i| ShellValue::Variable(format!("{}_{}", name, i)))
                                .collect();
                            ShellIR::ForIn {
                                var,
                                items,
                                body: Box::new(body_ir),
                            }
                        } else {
                            ShellIR::ForIn {
                                var,
                                items: vec![ShellValue::Variable(name.clone())],
                                body: Box::new(body_ir),
                            }
                        }
                    }
                    other => {
                        let val = self.convert_expr_to_value(other)?;
                        ShellIR::ForIn {
                            var,
                            items: vec![val],
                            body: Box::new(body_ir),
                        }
                    }
                };
                self.wrap_with_shadow_save_restore(loop_ir, &shadow_vars)
            }

            // Match in function context: propagate function-aware conversion
            // Propagate should_echo into arm bodies so match-as-last-expression echoes
            Stmt::Match { scrutinee, arms } => {
                let scrutinee_value = self.convert_expr_to_value(scrutinee)?;

                // Range patterns require if-elif-else chain (case can't do ranges)
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
                    case_arms.push(shell_ir::CaseArm {
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

            // Everything else: regular statement conversion
            _ => self.convert_stmt(stmt),
        }
    }

    /// Convert a block of statements in function context, with the last
    /// statement receiving `should_echo` treatment for implicit returns.
    fn convert_stmts_in_function(
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

    fn convert_stmt(&self, stmt: &crate::ast::Stmt) -> Result<ShellIR> {
        use crate::ast::Stmt;

        match stmt {
            Stmt::Let {
                name,
                value,
                declaration,
            } => {
                // Handle __if_expr: let x = if cond { a } else { b }
                // Lower to: if cond; then x=a; else x=b; fi
                // Supports nested else-if chains: __if_expr(c1, v1, __if_expr(c2, v2, v3))
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
                // Lower to: arr_0=a; arr_1=b; arr_2=c
                if let crate::ast::Expr::Array(elems) = value {
                    // Track array size for for-in expansion
                    self.arrays.borrow_mut().insert(name.clone(), elems.len());
                    let mut stmts = Vec::new();
                    for (i, elem) in elems.iter().enumerate() {
                        let elem_val = self.convert_expr_to_value(elem)?;
                        stmts.push(ShellIR::Let {
                            name: format!("{}_{}", name, i),
                            value: elem_val,
                            effects: EffectSet::pure(),
                        });
                    }
                    return Ok(ShellIR::Sequence(stmts));
                }

                // Handle let x = match y { arms } (match expression in let binding)
                // Lower to: case $y in pattern) x=arm_value ;; esac
                if let crate::ast::Expr::Block(block_stmts) = value {
                    if block_stmts.len() == 1 {
                        if let crate::ast::Stmt::Match { scrutinee, arms } = &block_stmts[0] {
                            return self.lower_let_match(name, scrutinee, arms);
                        }
                        // Handle let x = if cond { ... } else { ... }
                        if let crate::ast::Stmt::If {
                            condition,
                            then_block,
                            else_block,
                        } = &block_stmts[0]
                        {
                            return self.lower_let_if(
                                name,
                                condition,
                                then_block,
                                else_block.as_deref(),
                            );
                        }
                    }
                    // Handle let x = { stmt1; stmt2; ...; last_expr }
                    // where last is if/match/expr — use convert_match_arm_for_let
                    if block_stmts.len() > 1 {
                        return self.convert_match_arm_for_let(name, block_stmts);
                    }
                }

                let shell_value = self.convert_expr_to_value(value)?;
                // Track variable declarations for shadow detection
                if *declaration {
                    self.declared_vars.borrow_mut().insert(name.clone());
                }
                Ok(ShellIR::Let {
                    name: name.clone(),
                    value: shell_value,
                    effects: EffectSet::pure(),
                })
            }
            Stmt::Expr(expr) => self.convert_expr(expr),
            Stmt::Return(Some(expr)) => {
                let value = self.convert_expr_to_value(expr)?;
                Ok(ShellIR::Exit {
                    code: 0,
                    message: Some(format!("{value:?}")), // Simplified
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
            } => {
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
            Stmt::For {
                pattern,
                iter,
                body,
                ..
            } => {
                // Extract variable name from pattern
                let var = match pattern {
                    crate::ast::restricted::Pattern::Variable(name) => name.clone(),
                    _ => {
                        return Err(crate::models::Error::Validation(
                            "Only simple variable patterns supported in for loops".to_string(),
                        ))
                    }
                };

                // Convert range expression to start/end values
                let (start, end) = match iter {
                    crate::ast::Expr::Range {
                        start,
                        end,
                        inclusive,
                    } => {
                        let start_val = self.convert_expr_to_value(start)?;
                        let end_val = self.convert_expr_to_value(end)?;
                        let adjusted_end = adjust_range_end(end_val, *inclusive);

                        (start_val, adjusted_end)
                    }
                    // Non-range iterables: arrays, variables, etc.
                    // Convert to for-in loop over word list
                    other => {
                        let shadow_vars = self.detect_shadows(body);
                        let body_ir = self.convert_loop_body_with_shadows(body, &shadow_vars)?;
                        // Convert iterable to list of ShellValues
                        let loop_ir = match other {
                            crate::ast::Expr::Array(elements) => {
                                let items: Vec<ShellValue> = elements
                                    .iter()
                                    .map(|e| self.convert_expr_to_value(e))
                                    .collect::<Result<_>>()?;
                                ShellIR::ForIn {
                                    var,
                                    items,
                                    body: Box::new(body_ir),
                                }
                            }
                            crate::ast::Expr::Variable(name) => {
                                // Check if variable is a known array — expand to elements
                                let array_len = self.arrays.borrow().get(name).copied();
                                if let Some(len) = array_len {
                                    let items: Vec<ShellValue> = (0..len)
                                        .map(|i| ShellValue::Variable(format!("{}_{}", name, i)))
                                        .collect();
                                    ShellIR::ForIn {
                                        var,
                                        items,
                                        body: Box::new(body_ir),
                                    }
                                } else {
                                    // Non-array variable: for item in $var
                                    ShellIR::ForIn {
                                        var,
                                        items: vec![ShellValue::Variable(name.clone())],
                                        body: Box::new(body_ir),
                                    }
                                }
                            }
                            _ => {
                                // Fallback: try to convert to a single value
                                let val = self.convert_expr_to_value(other)?;
                                ShellIR::ForIn {
                                    var,
                                    items: vec![val],
                                    body: Box::new(body_ir),
                                }
                            }
                        };
                        return self.wrap_with_shadow_save_restore(loop_ir, &shadow_vars);
                    }
                };

                // Range-based for: detect shadows and convert with awareness
                let shadow_vars = self.detect_shadows(body);
                let body_ir = self.convert_loop_body_with_shadows(body, &shadow_vars)?;

                let for_ir = ShellIR::For {
                    var,
                    start,
                    end,
                    body: Box::new(body_ir),
                };
                self.wrap_with_shadow_save_restore(for_ir, &shadow_vars)
            }
            Stmt::Match { scrutinee, arms } => {
                let scrutinee_value = self.convert_expr_to_value(scrutinee)?;

                // Range patterns require if-elif-else chain (case can't do ranges)
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

                    case_arms.push(shell_ir::CaseArm {
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
            Stmt::While {
                condition, body, ..
            } => {
                // Convert condition to shell value
                let condition_value = self.convert_expr_to_value(condition)?;

                // Detect shadow variables: `let x = ...` (declaration=true) where x is already declared
                let shadow_vars = self.detect_shadows(body);

                // Convert body with shadow-awareness
                let body_ir = self.convert_loop_body_with_shadows(body, &shadow_vars)?;

                let while_ir = ShellIR::While {
                    condition: condition_value,
                    body: Box::new(body_ir),
                };

                // Wrap with save/restore if there are shadows
                self.wrap_with_shadow_save_restore(while_ir, &shadow_vars)
            }
            Stmt::Break => Ok(ShellIR::Break),
            Stmt::Continue => Ok(ShellIR::Continue),
        }
    }

    fn convert_stmts(&self, stmts: &[crate::ast::Stmt]) -> Result<ShellIR> {
        let mut ir_stmts = Vec::new();
        for stmt in stmts {
            ir_stmts.push(self.convert_stmt(stmt)?);
        }
        Ok(ShellIR::Sequence(ir_stmts))
    }

    /// Detect variables that are shadowed inside a loop body.
    /// Returns variable names that have `Stmt::Let { declaration: true }` in the body
    /// AND are already in `declared_vars` (i.e., exist in the outer scope).
    fn detect_shadows(&self, body: &[crate::ast::Stmt]) -> Vec<String> {
        let declared = self.declared_vars.borrow();
        let mut shadows = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for stmt in body {
            if let crate::ast::Stmt::Let {
                name,
                declaration: true,
                ..
            } = stmt
            {
                if declared.contains(name) && seen.insert(name.clone()) {
                    shadows.push(name.clone());
                }
            }
        }
        shadows
    }

    /// Convert a loop body with shadow-aware variable renaming.
    /// For each shadow variable `x`, the FIRST `let x = <rhs>` (declaration=true)
    /// is transformed so that the RHS references to `x` use the saved outer value
    /// `__shadow_x_save` instead.
    fn convert_loop_body_with_shadows(
        &self,
        body: &[crate::ast::Stmt],
        shadow_vars: &[String],
    ) -> Result<ShellIR> {
        if shadow_vars.is_empty() {
            return self.convert_stmts(body);
        }

        // Build set of shadow variable names for quick lookup
        let shadow_set: std::collections::HashSet<&str> =
            shadow_vars.iter().map(|s| s.as_str()).collect();
        // Track which shadows we've already processed (first occurrence only)
        let mut processed_shadows: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        let mut ir_stmts = Vec::new();
        for stmt in body {
            if let crate::ast::Stmt::Let {
                name,
                value,
                declaration: true,
            } = stmt
            {
                if shadow_set.contains(name.as_str()) && !processed_shadows.contains(name) {
                    // This is the first shadow of `name` in the loop body.
                    // Convert the value expression, then replace references to `name`
                    // in the result with the saved outer value `__shadow_{name}_save`.
                    let shell_value = self.convert_expr_to_value(value)?;
                    let save_name = format!("__shadow_{}_save", name);
                    let replaced_value =
                        Self::replace_var_refs_in_value(&shell_value, name, &save_name);
                    self.declared_vars.borrow_mut().insert(name.clone());
                    ir_stmts.push(ShellIR::Let {
                        name: name.clone(),
                        value: replaced_value,
                        effects: EffectSet::pure(),
                    });
                    processed_shadows.insert(name.clone());
                    continue;
                }
            }
            ir_stmts.push(self.convert_stmt(stmt)?);
        }
        Ok(ShellIR::Sequence(ir_stmts))
    }

    /// Wrap a loop IR with save/restore statements for shadow variables.
    /// Before: `__shadow_x_save=$x`
    /// After: `x=$__shadow_x_save`
    fn wrap_with_shadow_save_restore(
        &self,
        loop_ir: ShellIR,
        shadow_vars: &[String],
    ) -> Result<ShellIR> {
        if shadow_vars.is_empty() {
            return Ok(loop_ir);
        }

        let mut sequence = Vec::new();

        // Save each shadow variable's outer value before the loop
        for var in shadow_vars {
            let save_name = format!("__shadow_{}_save", var);
            sequence.push(ShellIR::Let {
                name: save_name,
                value: ShellValue::Variable(var.clone()),
                effects: EffectSet::pure(),
            });
        }

        // The loop itself
        sequence.push(loop_ir);

        // Restore each shadow variable's outer value after the loop
        for var in shadow_vars {
            let save_name = format!("__shadow_{}_save", var);
            sequence.push(ShellIR::Let {
                name: var.clone(),
                value: ShellValue::Variable(save_name),
                effects: EffectSet::pure(),
            });
        }

        Ok(ShellIR::Sequence(sequence))
    }

    /// Replace all references to `old_name` with `new_name` in a ShellValue.
    fn replace_var_refs_in_value(value: &ShellValue, old_name: &str, new_name: &str) -> ShellValue {
        match value {
            ShellValue::Variable(name) if name == old_name => {
                ShellValue::Variable(new_name.to_string())
            }
            ShellValue::Arithmetic { op, left, right } => {
                let new_left = Self::replace_var_refs_in_value(left, old_name, new_name);
                let new_right = Self::replace_var_refs_in_value(right, old_name, new_name);
                ShellValue::Arithmetic {
                    op: op.clone(),
                    left: Box::new(new_left),
                    right: Box::new(new_right),
                }
            }
            ShellValue::Concat(parts) => {
                let new_parts: Vec<ShellValue> = parts
                    .iter()
                    .map(|p| Self::replace_var_refs_in_value(p, old_name, new_name))
                    .collect();
                ShellValue::Concat(new_parts)
            }
            ShellValue::Comparison { op, left, right } => {
                let new_left = Self::replace_var_refs_in_value(left, old_name, new_name);
                let new_right = Self::replace_var_refs_in_value(right, old_name, new_name);
                ShellValue::Comparison {
                    op: op.clone(),
                    left: Box::new(new_left),
                    right: Box::new(new_right),
                }
            }
            ShellValue::LogicalNot { operand } => ShellValue::LogicalNot {
                operand: Box::new(Self::replace_var_refs_in_value(operand, old_name, new_name)),
            },
            ShellValue::LogicalAnd { left, right } => {
                let new_left = Self::replace_var_refs_in_value(left, old_name, new_name);
                let new_right = Self::replace_var_refs_in_value(right, old_name, new_name);
                ShellValue::LogicalAnd {
                    left: Box::new(new_left),
                    right: Box::new(new_right),
                }
            }
            ShellValue::LogicalOr { left, right } => {
                let new_left = Self::replace_var_refs_in_value(left, old_name, new_name);
                let new_right = Self::replace_var_refs_in_value(right, old_name, new_name);
                ShellValue::LogicalOr {
                    left: Box::new(new_left),
                    right: Box::new(new_right),
                }
            }
            ShellValue::CommandSubst(cmd) => {
                // Don't recurse into command substitutions for simplicity
                ShellValue::CommandSubst(cmd.clone())
            }
            ShellValue::DynamicArrayAccess { array, index } => {
                let new_index = Self::replace_var_refs_in_value(index, old_name, new_name);
                if array == old_name {
                    ShellValue::DynamicArrayAccess {
                        array: new_name.to_string(),
                        index: Box::new(new_index),
                    }
                } else {
                    ShellValue::DynamicArrayAccess {
                        array: array.clone(),
                        index: Box::new(new_index),
                    }
                }
            }
            // String, TestFlag, FunctionCall, etc. — no variable references to replace
            other => other.clone(),
        }
    }

    fn convert_expr(&self, expr: &crate::ast::Expr) -> Result<ShellIR> {
        use crate::ast::Expr;

        match expr {
            Expr::FunctionCall { name, args } => {
                // Handle __format_concat at expression level (shouldn't happen often, but be safe)
                if name == "__format_concat" {
                    let _value = self.convert_expr_to_value(expr)?;
                    return Ok(ShellIR::Noop);
                }

                // Issue #95: exec() is a DSL built-in that runs a shell command string
                // It should use 'eval' to properly evaluate pipes and operators
                if name == "exec" {
                    // Convert exec("cmd1 | cmd2") to eval 'cmd1 | cmd2'
                    let mut cmd_args = Vec::new();
                    for arg in args {
                        cmd_args.push(self.convert_expr_to_value(arg)?);
                    }
                    return Ok(ShellIR::Exec {
                        cmd: Command {
                            program: "eval".to_string(),
                            args: cmd_args,
                        },
                        effects: self.analyze_command_effects(name),
                    });
                }

                // Convert function calls to shell commands
                let mut cmd_args = Vec::new();
                for arg in args {
                    cmd_args.push(self.convert_expr_to_value(arg)?);
                }

                // Check if this is a stdlib function - if so, use the shell function name
                let program = if crate::stdlib::is_stdlib_function(name) {
                    crate::stdlib::get_shell_function_name(name)
                } else {
                    name.clone()
                };

                Ok(ShellIR::Exec {
                    cmd: Command {
                        program,
                        args: cmd_args,
                    },
                    effects: self.analyze_command_effects(name),
                })
            }
            _ => {
                // For other expressions, convert to values and wrap in a noop
                let _value = self.convert_expr_to_value(expr)?;
                Ok(ShellIR::Noop)
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn convert_expr_to_value(&self, expr: &crate::ast::Expr) -> Result<ShellValue> {
        use crate::ast::{restricted::Literal, Expr};

        match expr {
            Expr::Literal(literal) => match literal {
                Literal::Bool(b) => Ok(ShellValue::Bool(*b)),
                Literal::U16(n) => Ok(ShellValue::String(n.to_string())),
                Literal::U32(n) => Ok(ShellValue::String(n.to_string())),
                Literal::I32(n) => Ok(ShellValue::String(n.to_string())),
                Literal::Str(s) => Ok(ShellValue::String(s.clone())),
            },
            Expr::Variable(name) => Ok(ShellValue::Variable(name.clone())),
            Expr::FunctionCall { name, args } => self.convert_fn_call_to_value(name, args),
            Expr::Unary { op, operand } => self.convert_unary_to_value(op, operand),
            Expr::Binary { op, left, right } => self.convert_binary_to_value(op, left, right),
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => self.convert_method_call_to_value(receiver, method, args),
            Expr::PositionalArgs => Ok(ShellValue::Arg { position: None }),
            Expr::Index { object, index } => self.convert_index_to_value(object, index),
            Expr::Array(elems) => {
                if let Some(first) = elems.first() {
                    self.convert_expr_to_value(first)
                } else {
                    Ok(ShellValue::String("".to_string()))
                }
            }
            _ => Ok(ShellValue::String("unknown".to_string())),
        }
    }

    fn convert_fn_call_to_value(
        &self,
        name: &str,
        args: &[crate::ast::Expr],
    ) -> Result<ShellValue> {
        use crate::ast::{restricted::Literal, Expr};

        if name == "env" || name == "env_var_or" {
            return self.convert_env_call_to_value(name, args);
        }

        if name == "arg" {
            let first_arg = args.first().ok_or_else(|| {
                crate::models::Error::Validation(
                    "arg() requires at least one argument".to_string(),
                )
            })?;
            let position = match first_arg {
                Expr::Literal(Literal::U32(n)) => *n as usize,
                Expr::Literal(Literal::I32(n)) => *n as usize,
                _ => {
                    return Err(crate::models::Error::Validation(
                        "arg() requires integer literal for position".to_string(),
                    ))
                }
            };
            if position == 0 {
                return Err(crate::models::Error::Validation(
                    "arg() position must be >= 1 (use arg(1) for first argument)".to_string(),
                ));
            }
            return Ok(ShellValue::Arg {
                position: Some(position),
            });
        }

        if name == "args" {
            return Ok(ShellValue::Arg { position: None });
        }
        if name == "arg_count" {
            return Ok(ShellValue::ArgCount);
        }
        if name == "exit_code" {
            return Ok(ShellValue::ExitCode);
        }

        if name == "__format_concat" {
            let mut parts = Vec::new();
            for arg in args {
                parts.push(self.convert_expr_to_value(arg)?);
            }
            return Ok(ShellValue::Concat(parts));
        }

        if name == "__if_expr" && args.len() == 3 {
            return self.convert_expr_to_value(&args[1]);
        }

        let mut cmd_args = Vec::new();
        for arg in args {
            cmd_args.push(self.convert_expr_to_value(arg)?);
        }

        let program = if crate::stdlib::is_stdlib_function(name) {
            crate::stdlib::get_shell_function_name(name)
        } else {
            name.to_string()
        };

        Ok(ShellValue::CommandSubst(Command {
            program,
            args: cmd_args,
        }))
    }

    fn convert_env_call_to_value(
        &self,
        name: &str,
        args: &[crate::ast::Expr],
    ) -> Result<ShellValue> {
        use crate::ast::{restricted::Literal, Expr};

        let first_arg = args.first().ok_or_else(|| {
            crate::models::Error::Validation(format!(
                "{}() requires at least one argument",
                name
            ))
        })?;
        let var_name = match first_arg {
            Expr::Literal(Literal::Str(s)) => s.clone(),
            _ => {
                return Err(crate::models::Error::Validation(format!(
                    "{}() requires string literal for variable name",
                    name
                )))
            }
        };

        if !var_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(crate::models::Error::Validation(format!(
                "Invalid environment variable name: '{}'",
                var_name
            )));
        }

        let default = if name == "env_var_or" {
            match &args.get(1) {
                Some(Expr::Literal(Literal::Str(s))) => Some(s.clone()),
                _ => {
                    return Err(crate::models::Error::Validation(
                        "env_var_or() requires string literal for default value".to_string(),
                    ))
                }
            }
        } else {
            None
        };

        Ok(ShellValue::EnvVar {
            name: var_name,
            default,
        })
    }

    fn convert_unary_to_value(
        &self,
        op: &crate::ast::restricted::UnaryOp,
        operand: &crate::ast::Expr,
    ) -> Result<ShellValue> {
        use crate::ast::restricted::UnaryOp;
        let operand_val = self.convert_expr_to_value(operand)?;

        match op {
            UnaryOp::Not => Ok(ShellValue::LogicalNot {
                operand: Box::new(operand_val),
            }),
            UnaryOp::Neg => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Sub,
                left: Box::new(ShellValue::String("0".to_string())),
                right: Box::new(operand_val),
            }),
        }
    }

    fn convert_binary_to_value(
        &self,
        op: &crate::ast::restricted::BinaryOp,
        left: &crate::ast::Expr,
        right: &crate::ast::Expr,
    ) -> Result<ShellValue> {
        use crate::ast::restricted::BinaryOp;
        let left_val = self.convert_expr_to_value(left)?;
        let right_val = self.convert_expr_to_value(right)?;

        match op {
            BinaryOp::Eq => {
                let is_string = is_string_value(&left_val) || is_string_value(&right_val);
                Ok(ShellValue::Comparison {
                    op: if is_string {
                        shell_ir::ComparisonOp::StrEq
                    } else {
                        shell_ir::ComparisonOp::NumEq
                    },
                    left: Box::new(left_val),
                    right: Box::new(right_val),
                })
            }
            BinaryOp::Ne => {
                let is_string = is_string_value(&left_val) || is_string_value(&right_val);
                Ok(ShellValue::Comparison {
                    op: if is_string {
                        shell_ir::ComparisonOp::StrNe
                    } else {
                        shell_ir::ComparisonOp::NumNe
                    },
                    left: Box::new(left_val),
                    right: Box::new(right_val),
                })
            }
            BinaryOp::Gt => Ok(ShellValue::Comparison {
                op: shell_ir::ComparisonOp::Gt,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Ge => Ok(ShellValue::Comparison {
                op: shell_ir::ComparisonOp::Ge,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Lt => Ok(ShellValue::Comparison {
                op: shell_ir::ComparisonOp::Lt,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Le => Ok(ShellValue::Comparison {
                op: shell_ir::ComparisonOp::Le,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Add => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Add,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Sub => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Sub,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Mul => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Mul,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Div => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Div,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Rem => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Mod,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::And => Ok(ShellValue::LogicalAnd {
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Or => Ok(ShellValue::LogicalOr {
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::BitAnd => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::BitAnd,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::BitOr => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::BitOr,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::BitXor => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::BitXor,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Shl => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Shl,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
            BinaryOp::Shr => Ok(ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Shr,
                left: Box::new(left_val),
                right: Box::new(right_val),
            }),
        }
    }

    fn convert_method_call_to_value(
        &self,
        receiver: &crate::ast::Expr,
        method: &str,
        args: &[crate::ast::Expr],
    ) -> Result<ShellValue> {
        use crate::ast::{restricted::Literal, Expr};

        // PARAM-SPEC-005: Detect std::env::args().nth(N).unwrap() pattern
        if method == "unwrap" && args.is_empty() {
            if let Expr::MethodCall {
                receiver: inner_receiver,
                method: inner_method,
                args: inner_args,
            } = receiver
            {
                if inner_method == "nth" && inner_args.len() == 1 {
                    if let Expr::FunctionCall {
                        name,
                        args: fn_args,
                    } = &**inner_receiver
                    {
                        if name == "std::env::args" && fn_args.is_empty() {
                            if let Some(Expr::Literal(Literal::U32(n))) = inner_args.first() {
                                return Ok(ShellValue::Arg {
                                    position: Some(*n as usize),
                                });
                            }
                        }
                    }
                }
            }
        }

        // P0-POSITIONAL-PARAMETERS: Detect args.get(N).unwrap_or(default) pattern
        if method == "unwrap_or" && args.len() == 1 {
            if let Expr::MethodCall {
                receiver: inner_receiver,
                method: inner_method,
                args: inner_args,
            } = receiver
            {
                if inner_method == "get" && inner_args.len() == 1 {
                    if let Some(Expr::Literal(Literal::U32(n))) = inner_args.first() {
                        if let Some(Expr::Literal(Literal::Str(default_val))) = args.first() {
                            return Ok(ShellValue::ArgWithDefault {
                                position: *n as usize,
                                default: default_val.clone(),
                            });
                        }
                    }
                }

                // PARAM-SPEC-005: Detect std::env::args().nth(N).unwrap_or(default) pattern
                if inner_method == "nth" && inner_args.len() == 1 {
                    if let Expr::FunctionCall {
                        name,
                        args: fn_args,
                    } = &**inner_receiver
                    {
                        if name == "std::env::args" && fn_args.is_empty() {
                            if let Some(Expr::Literal(Literal::U32(n))) = inner_args.first() {
                                if let Some(Expr::Literal(Literal::Str(default_val))) = args.first()
                                {
                                    return Ok(ShellValue::ArgWithDefault {
                                        position: *n as usize,
                                        default: default_val.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(ShellValue::String("unknown".to_string()))
    }

    fn convert_index_to_value(
        &self,
        object: &crate::ast::Expr,
        index: &crate::ast::Expr,
    ) -> Result<ShellValue> {
        use crate::ast::Expr;

        if let Expr::Variable(name) = object {
            let idx_val = self.convert_expr_to_value(index)?;
            match idx_val {
                ShellValue::String(s) => Ok(ShellValue::Variable(format!("{}_{}", name, s))),
                ShellValue::Variable(_) | ShellValue::Arithmetic { .. } => {
                    Ok(ShellValue::DynamicArrayAccess {
                        array: name.clone(),
                        index: Box::new(idx_val),
                    })
                }
                _ => Ok(ShellValue::Variable(format!("{}_0", name))),
            }
        } else {
            Ok(ShellValue::String("unknown".to_string()))
        }
    }

    fn analyze_command_effects(&self, command: &str) -> EffectSet {
        // Simple effect analysis based on command name
        let mut effects = EffectSet::pure();

        match command {
            "curl" | "wget" => {
                effects.add(Effect::NetworkAccess);
            }
            "echo" | "printf" => {
                effects.add(Effect::FileWrite);
            }
            _ => {}
        }

        effects
    }

    fn convert_match_pattern(
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
    fn lower_let_match(
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
    fn convert_match_arm_for_let(
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
    fn lower_let_if(
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
    fn has_range_patterns(arms: &[crate::ast::restricted::MatchArm]) -> bool {
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
    fn convert_range_match(
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
    fn convert_range_match_fn(
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
    fn lower_let_if_expr(&self, target: &str, args: &[crate::ast::Expr]) -> Result<ShellIR> {
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
    fn lower_return_if_expr(&self, args: &[crate::ast::Expr]) -> Result<ShellIR> {
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

/// Adjust range end value for exclusive ranges (0..n → 0..=n-1).
/// For literal integers, directly subtract 1. For variables and expressions,
/// wrap in Arithmetic { Sub, end_val, 1 } so shell emits $((n - 1)).
fn adjust_range_end(end_val: ShellValue, inclusive: bool) -> ShellValue {
    if inclusive {
        return end_val;
    }
    // Exclusive range: subtract 1 from end
    match &end_val {
        ShellValue::String(s) => {
            if let Ok(n) = s.parse::<i32>() {
                ShellValue::String((n - 1).to_string())
            } else {
                // Non-numeric string — wrap in arithmetic
                ShellValue::Arithmetic {
                    op: shell_ir::ArithmeticOp::Sub,
                    left: Box::new(end_val),
                    right: Box::new(ShellValue::String("1".to_string())),
                }
            }
        }
        // Variable or expression — wrap in arithmetic subtraction
        _ => ShellValue::Arithmetic {
            op: shell_ir::ArithmeticOp::Sub,
            left: Box::new(end_val),
            right: Box::new(ShellValue::String("1".to_string())),
        },
    }
}

/// Evaluate an arithmetic operation on two constant integers.
/// Returns None for division/modulo by zero.
fn eval_arithmetic_op(op: &shell_ir::ArithmeticOp, left: i64, right: i64) -> Option<i64> {
    match op {
        shell_ir::ArithmeticOp::Add => Some(left + right),
        shell_ir::ArithmeticOp::Sub => Some(left - right),
        shell_ir::ArithmeticOp::Mul => Some(left * right),
        shell_ir::ArithmeticOp::Div if right != 0 => Some(left / right),
        shell_ir::ArithmeticOp::Mod if right != 0 => Some(left % right),
        shell_ir::ArithmeticOp::BitAnd => Some(left & right),
        shell_ir::ArithmeticOp::BitOr => Some(left | right),
        shell_ir::ArithmeticOp::BitXor => Some(left ^ right),
        shell_ir::ArithmeticOp::Shl => Some(left << right),
        shell_ir::ArithmeticOp::Shr => Some(left >> right),
        _ => None,
    }
}

/// Try to fold two ShellValues as constant integer arithmetic.
/// Returns the folded result string, or None if folding is not possible.
fn try_fold_constant_arithmetic(
    op: &shell_ir::ArithmeticOp,
    left: &ShellValue,
    right: &ShellValue,
) -> Option<String> {
    if let (ShellValue::String(l), ShellValue::String(r)) = (left, right) {
        if let (Ok(ln), Ok(rn)) = (l.parse::<i64>(), r.parse::<i64>()) {
            return eval_arithmetic_op(op, ln, rn).map(|v| v.to_string());
        }
    }
    None
}

fn constant_fold(ir: ShellIR) -> ShellIR {
    let mut transform_fn = |node| match node {
        ShellIR::Let {
            name,
            value: ShellValue::Concat(parts),
            effects,
        } => {
            if parts.iter().all(|p| matches!(p, ShellValue::String(_))) {
                let folded = parts
                    .iter()
                    .filter_map(|p| match p {
                        ShellValue::String(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .collect::<String>();
                ShellIR::Let {
                    name,
                    value: ShellValue::String(folded),
                    effects,
                }
            } else {
                ShellIR::Let {
                    name,
                    value: ShellValue::Concat(parts),
                    effects,
                }
            }
        }
        ShellIR::Let {
            name,
            value: ShellValue::Arithmetic { op, left, right },
            effects,
        } => {
            let folded_left = fold_arithmetic_value(*left);
            let folded_right = fold_arithmetic_value(*right);

            let value = match try_fold_constant_arithmetic(&op, &folded_left, &folded_right) {
                Some(result) => ShellValue::String(result),
                None => ShellValue::Arithmetic {
                    op,
                    left: Box::new(folded_left),
                    right: Box::new(folded_right),
                },
            };
            ShellIR::Let {
                name,
                value,
                effects,
            }
        }
        _ => node,
    };
    transform_ir(ir, &mut transform_fn)
}

/// Recursively fold arithmetic values (for nested expressions like 10 * 1024 * 1024)
fn fold_arithmetic_value(value: ShellValue) -> ShellValue {
    match value {
        ShellValue::Arithmetic { op, left, right } => {
            let folded_left = fold_arithmetic_value(*left);
            let folded_right = fold_arithmetic_value(*right);

            match try_fold_constant_arithmetic(&op, &folded_left, &folded_right) {
                Some(result) => ShellValue::String(result),
                None => ShellValue::Arithmetic {
                    op,
                    left: Box::new(folded_left),
                    right: Box::new(folded_right),
                },
            }
        }
        other => other,
    }
}

fn eliminate_dead_code(ir: ShellIR) -> ShellIR {
    // Simple dead code elimination
    ir // Placeholder - would implement actual DCE
}

/// Check if a ShellValue represents a string type (not a number)
fn is_string_value(value: &ShellValue) -> bool {
    match value {
        ShellValue::String(s) => {
            // Check if it's a string literal (not parseable as number)
            s.parse::<i64>().is_err() && s.parse::<f64>().is_err()
        }
        ShellValue::Bool(_) => false, // Bools are not strings for comparison
        ShellValue::Variable(_) => false, // Can't determine at compile time
        ShellValue::EnvVar { .. } => false, // Can't determine at compile time
        ShellValue::Concat(_) => true, // String concatenation
        ShellValue::CommandSubst(_) => false, // Could be numeric
        ShellValue::Comparison { .. } => false,
        ShellValue::Arithmetic { .. } => false,
        ShellValue::LogicalAnd { .. }
        | ShellValue::LogicalOr { .. }
        | ShellValue::LogicalNot { .. } => false,
        // Sprint 27b: Command-line arguments are not determinable at compile time
        ShellValue::Arg { .. } | ShellValue::ArgCount => false,
        // P0-POSITIONAL-PARAMETERS: Arguments with defaults are not determinable at compile time
        ShellValue::ArgWithDefault { .. } => false,
        // Sprint 27c: Exit code handling - GREEN PHASE (exit codes are numeric, not string)
        ShellValue::ExitCode => false,
        // Dynamic array access returns numeric values
        ShellValue::DynamicArrayAccess { .. } => false,
    }
}

fn transform_ir<F>(ir: ShellIR, transform: &mut F) -> ShellIR
where
    F: FnMut(ShellIR) -> ShellIR,
{
    let transformed = match ir {
        ShellIR::Sequence(stmts) => {
            let new_stmts = stmts
                .into_iter()
                .map(|stmt| transform_ir(stmt, transform))
                .collect();
            ShellIR::Sequence(new_stmts)
        }
        ShellIR::If {
            test,
            then_branch,
            else_branch,
        } => {
            let new_then = Box::new(transform_ir(*then_branch, transform));
            let new_else = else_branch.map(|eb| Box::new(transform_ir(*eb, transform)));
            ShellIR::If {
                test,
                then_branch: new_then,
                else_branch: new_else,
            }
        }
        ShellIR::Function { name, params, body } => {
            let new_body = Box::new(transform_ir(*body, transform));
            ShellIR::Function {
                name,
                params,
                body: new_body,
            }
        }
        other => other,
    };

    transform(transformed)
}

#[cfg(test)]
mod convert_expr_tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::ast::restricted::{BinaryOp, Literal, UnaryOp};
    use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};

    /// Helper: wrap a single let statement in a main function and convert to IR
    fn convert_let_stmt(name: &str, value: Expr) -> ShellIR {
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Let {
                    name: name.to_string(),
                    value,
                    declaration: true,
                }],
            }],
            entry_point: "main".to_string(),
        };
        from_ast(&ast).expect("IR conversion should succeed")
    }

    /// Helper: wrap a single let statement and expect conversion to fail
    fn convert_let_stmt_err(name: &str, value: Expr) -> crate::models::Error {
        let ast = RestrictedAst {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Let {
                    name: name.to_string(),
                    value,
                    declaration: true,
                }],
            }],
            entry_point: "main".to_string(),
        };
        from_ast(&ast).expect_err("IR conversion should fail")
    }

    /// Helper: extract the ShellValue from a single Let in a Sequence
    fn extract_let_value(ir: &ShellIR) -> &ShellValue {
        match ir {
            ShellIR::Sequence(stmts) => match &stmts[0] {
                ShellIR::Let { value, .. } => value,
                other => panic!("Expected Let, got {:?}", other),
            },
            other => panic!("Expected Sequence, got {:?}", other),
        }
    }

    // ===== Literal branches =====

    #[test]
    fn test_EXPR_VAL_001_literal_bool_true() {
        let ir = convert_let_stmt("flag", Expr::Literal(Literal::Bool(true)));
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::Bool(true)));
    }

    #[test]
    fn test_EXPR_VAL_002_literal_bool_false() {
        let ir = convert_let_stmt("flag", Expr::Literal(Literal::Bool(false)));
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::Bool(false)));
    }

    #[test]
    fn test_EXPR_VAL_003_literal_u16() {
        let ir = convert_let_stmt("port", Expr::Literal(Literal::U16(443)));
        let val = extract_let_value(&ir);
        match val {
            ShellValue::String(s) => assert_eq!(s, "443"),
            other => panic!("Expected String(\"443\"), got {:?}", other),
        }
    }

    #[test]
    fn test_EXPR_VAL_004_literal_u32() {
        let ir = convert_let_stmt("count", Expr::Literal(Literal::U32(100)));
        let val = extract_let_value(&ir);
        match val {
            ShellValue::String(s) => assert_eq!(s, "100"),
            other => panic!("Expected String(\"100\"), got {:?}", other),
        }
    }

    #[test]
    fn test_EXPR_VAL_005_literal_i32() {
        let ir = convert_let_stmt("offset", Expr::Literal(Literal::I32(-99)));
        let val = extract_let_value(&ir);
        match val {
            ShellValue::String(s) => assert_eq!(s, "-99"),
            other => panic!("Expected String(\"-99\"), got {:?}", other),
        }
    }

    #[test]
    fn test_EXPR_VAL_006_literal_str() {
        let ir = convert_let_stmt(
            "msg",
            Expr::Literal(Literal::Str("hello world".to_string())),
        );
        let val = extract_let_value(&ir);
        match val {
            ShellValue::String(s) => assert_eq!(s, "hello world"),
            other => panic!("Expected String(\"hello world\"), got {:?}", other),
        }
    }

    // ===== Variable =====

    #[test]
    fn test_EXPR_VAL_007_variable() {
        let ir = convert_let_stmt("alias", Expr::Variable("original".to_string()));
        let val = extract_let_value(&ir);
        match val {
            ShellValue::Variable(name) => assert_eq!(name, "original"),
            other => panic!("Expected Variable(\"original\"), got {:?}", other),
        }
    }

    // ===== FunctionCall: stdlib functions =====

    #[test]
    fn test_EXPR_VAL_008_func_env() {
        let ir = convert_let_stmt(
            "home",
            Expr::FunctionCall {
                name: "env".to_string(),
                args: vec![Expr::Literal(Literal::Str("HOME".to_string()))],
            },
        );
        let val = extract_let_value(&ir);
        match val {
            ShellValue::EnvVar { name, default } => {
                assert_eq!(name, "HOME");
                assert!(default.is_none());
            }
            other => panic!("Expected EnvVar {{ HOME, None }}, got {:?}", other),
        }
    }

    #[test]
    fn test_EXPR_VAL_009_func_env_var_or() {
        let ir = convert_let_stmt(
            "editor",
            Expr::FunctionCall {
                name: "env_var_or".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("EDITOR".to_string())),
                    Expr::Literal(Literal::Str("nano".to_string())),
                ],
            },
        );
        let val = extract_let_value(&ir);
        match val {
            ShellValue::EnvVar { name, default } => {
                assert_eq!(name, "EDITOR");
                assert_eq!(default.as_deref(), Some("nano"));
            }
            other => panic!("Expected EnvVar {{ EDITOR, Some(nano) }}, got {:?}", other),
        }
    }

    #[test]
    fn test_EXPR_VAL_010_func_arg() {
        let ir = convert_let_stmt(
            "first",
            Expr::FunctionCall {
                name: "arg".to_string(),
                args: vec![Expr::Literal(Literal::U32(1))],
            },
        );
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::Arg { position: Some(1) }));
    }

    #[test]
    fn test_EXPR_VAL_011_func_args() {
        let ir = convert_let_stmt(
            "all",
            Expr::FunctionCall {
                name: "args".to_string(),
                args: vec![],
            },
        );
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::Arg { position: None }));
    }

    #[test]
    fn test_EXPR_VAL_012_func_arg_count() {
        let ir = convert_let_stmt(
            "n",
            Expr::FunctionCall {
                name: "arg_count".to_string(),
                args: vec![],
            },
        );
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::ArgCount));
    }

    #[test]
    fn test_EXPR_VAL_013_func_exit_code() {
        let ir = convert_let_stmt(
            "rc",
            Expr::FunctionCall {
                name: "exit_code".to_string(),
                args: vec![],
            },
        );
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::ExitCode));
    }

    // ===== FunctionCall: validation errors =====

    #[test]
    fn test_EXPR_VAL_014_env_no_args_error() {
        let err = convert_let_stmt_err(
            "bad",
            Expr::FunctionCall {
                name: "env".to_string(),
                args: vec![],
            },
        );
        let msg = err.to_string();
        assert!(
            msg.contains("requires at least one argument"),
            "Expected 'requires at least one argument', got: {}",
            msg
        );
    }

    #[test]
    fn test_EXPR_VAL_015_env_non_string_arg_error() {
        let err = convert_let_stmt_err(
            "bad",
            Expr::FunctionCall {
                name: "env".to_string(),
                args: vec![Expr::Literal(Literal::U32(42))],
            },
        );
        let msg = err.to_string();
        assert!(
            msg.contains("string literal"),
            "Expected error about string literal, got: {}",
            msg
        );
    }

    #[test]
    fn test_EXPR_VAL_016_env_invalid_var_name_special_chars() {
        let err = convert_let_stmt_err(
            "bad",
            Expr::FunctionCall {
                name: "env".to_string(),
                args: vec![Expr::Literal(Literal::Str("BAD-NAME".to_string()))],
            },
        );
        let msg = err.to_string();
        assert!(
            msg.contains("Invalid environment variable name"),
            "Expected invalid var name error, got: {}",
            msg
        );
    }

    #[test]
    fn test_EXPR_VAL_017_env_var_or_non_string_default_error() {
        let err = convert_let_stmt_err(
            "bad",
            Expr::FunctionCall {
                name: "env_var_or".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("PATH".to_string())),
                    Expr::Literal(Literal::U32(99)),
                ],
            },
        );
        let msg = err.to_string();
        assert!(
            msg.contains("string literal for default value"),
            "Expected default value error, got: {}",
            msg
        );
    }

    #[test]
    fn test_EXPR_VAL_018_arg_zero_position_error() {
        let err = convert_let_stmt_err(
            "bad",
            Expr::FunctionCall {
                name: "arg".to_string(),
                args: vec![Expr::Literal(Literal::U32(0))],
            },
        );
        let msg = err.to_string();
        assert!(
            msg.contains("position must be >= 1"),
            "Expected position >= 1 error, got: {}",
            msg
        );
    }

    // ===== FunctionCall: regular (non-stdlib) =====

    #[test]
    fn test_EXPR_VAL_019_func_call_regular_becomes_command_subst() {
        let ir = convert_let_stmt(
            "output",
            Expr::FunctionCall {
                name: "whoami".to_string(),
                args: vec![],
            },
        );
        let val = extract_let_value(&ir);
        match val {
            ShellValue::CommandSubst(cmd) => {
                assert_eq!(cmd.program, "whoami");
                assert!(cmd.args.is_empty());
            }
            other => panic!("Expected CommandSubst(whoami), got {:?}", other),
        }
    }

    // ===== Unary: Not, Neg =====

    #[test]
    fn test_EXPR_VAL_020_unary_not() {
        let ir = convert_let_stmt(
            "negated",
            Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(Expr::Literal(Literal::Bool(false))),
            },
        );
        let val = extract_let_value(&ir);
        match val {
            ShellValue::LogicalNot { operand } => {
                assert!(matches!(**operand, ShellValue::Bool(false)));
            }
            other => panic!("Expected LogicalNot, got {:?}", other),
        }
    }

    #[test]
    fn test_EXPR_VAL_021_unary_neg() {
        let ir = convert_let_stmt(
            "neg",
            Expr::Unary {
                op: UnaryOp::Neg,
                operand: Box::new(Expr::Literal(Literal::U32(7))),
            },
        );
        let val = extract_let_value(&ir);
        match val {
            ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Sub,
                left,
                right,
            } => {
                // Negation is 0 - operand
                assert!(matches!(**left, ShellValue::String(ref s) if s == "0"));
                assert!(matches!(**right, ShellValue::String(ref s) if s == "7"));
            }
            other => panic!("Expected Arithmetic(Sub, 0, 7), got {:?}", other),
        }
    }

    // ===== Binary: comparison ops =====

    #[test]
    fn test_EXPR_VAL_022_binary_eq_string_vs_numeric() {
        // String operands -> StrEq
        let ir_str = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::Eq,
                left: Box::new(Expr::Literal(Literal::Str("abc".to_string()))),
                right: Box::new(Expr::Literal(Literal::Str("def".to_string()))),
            },
        );
        let val_str = extract_let_value(&ir_str);
        assert!(matches!(
            val_str,
            ShellValue::Comparison {
                op: shell_ir::ComparisonOp::StrEq,
                ..
            }
        ));

        // Numeric operands -> NumEq
        let ir_num = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::Eq,
                left: Box::new(Expr::Literal(Literal::U32(5))),
                right: Box::new(Expr::Literal(Literal::U32(5))),
            },
        );
        let val_num = extract_let_value(&ir_num);
        assert!(matches!(
            val_num,
            ShellValue::Comparison {
                op: shell_ir::ComparisonOp::NumEq,
                ..
            }
        ));
    }

    #[test]
    fn test_EXPR_VAL_023_binary_ne() {
        let ir = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::Ne,
                left: Box::new(Expr::Literal(Literal::U32(1))),
                right: Box::new(Expr::Literal(Literal::U32(2))),
            },
        );
        let val = extract_let_value(&ir);
        assert!(matches!(
            val,
            ShellValue::Comparison {
                op: shell_ir::ComparisonOp::NumNe,
                ..
            }
        ));
    }

    #[test]
    fn test_EXPR_VAL_024_binary_all_comparison_ops() {
        // Gt
        let ir = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::Gt,
                left: Box::new(Expr::Literal(Literal::U32(10))),
                right: Box::new(Expr::Literal(Literal::U32(5))),
            },
        );
        assert!(matches!(
            extract_let_value(&ir),
            ShellValue::Comparison {
                op: shell_ir::ComparisonOp::Gt,
                ..
            }
        ));

        // Ge
        let ir = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::Ge,
                left: Box::new(Expr::Literal(Literal::U32(5))),
                right: Box::new(Expr::Literal(Literal::U32(5))),
            },
        );
        assert!(matches!(
            extract_let_value(&ir),
            ShellValue::Comparison {
                op: shell_ir::ComparisonOp::Ge,
                ..
            }
        ));

        // Lt
        let ir = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::Lt,
                left: Box::new(Expr::Literal(Literal::U32(3))),
                right: Box::new(Expr::Literal(Literal::U32(5))),
            },
        );
        assert!(matches!(
            extract_let_value(&ir),
            ShellValue::Comparison {
                op: shell_ir::ComparisonOp::Lt,
                ..
            }
        ));

        // Le
        let ir = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::Le,
                left: Box::new(Expr::Literal(Literal::U32(3))),
                right: Box::new(Expr::Literal(Literal::U32(3))),
            },
        );
        assert!(matches!(
            extract_let_value(&ir),
            ShellValue::Comparison {
                op: shell_ir::ComparisonOp::Le,
                ..
            }
        ));
    }

    // ===== Binary: arithmetic ops =====

    #[test]
    fn test_EXPR_VAL_025_binary_arithmetic_ops() {
        // Add
        let ir = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Literal(Literal::U32(2))),
                right: Box::new(Expr::Literal(Literal::U32(3))),
            },
        );
        assert!(matches!(
            extract_let_value(&ir),
            ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Add,
                ..
            }
        ));

        // Mul
        let ir = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::Mul,
                left: Box::new(Expr::Literal(Literal::U32(4))),
                right: Box::new(Expr::Literal(Literal::U32(5))),
            },
        );
        assert!(matches!(
            extract_let_value(&ir),
            ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Mul,
                ..
            }
        ));

        // Div
        let ir = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::Div,
                left: Box::new(Expr::Literal(Literal::U32(10))),
                right: Box::new(Expr::Literal(Literal::U32(2))),
            },
        );
        assert!(matches!(
            extract_let_value(&ir),
            ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Div,
                ..
            }
        ));

        // Rem
        let ir = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::Rem,
                left: Box::new(Expr::Literal(Literal::U32(10))),
                right: Box::new(Expr::Literal(Literal::U32(3))),
            },
        );
        assert!(matches!(
            extract_let_value(&ir),
            ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Mod,
                ..
            }
        ));
    }

    // ===== Binary: logical ops =====

    #[test]
    fn test_EXPR_VAL_026_binary_logical_and() {
        let ir = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::And,
                left: Box::new(Expr::Literal(Literal::Bool(true))),
                right: Box::new(Expr::Literal(Literal::Bool(false))),
            },
        );
        let val = extract_let_value(&ir);
        match val {
            ShellValue::LogicalAnd { left, right } => {
                assert!(matches!(**left, ShellValue::Bool(true)));
                assert!(matches!(**right, ShellValue::Bool(false)));
            }
            other => panic!("Expected LogicalAnd, got {:?}", other),
        }
    }

    #[test]
    fn test_EXPR_VAL_027_binary_logical_or() {
        let ir = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::Or,
                left: Box::new(Expr::Literal(Literal::Bool(false))),
                right: Box::new(Expr::Literal(Literal::Bool(true))),
            },
        );
        let val = extract_let_value(&ir);
        match val {
            ShellValue::LogicalOr { left, right } => {
                assert!(matches!(**left, ShellValue::Bool(false)));
                assert!(matches!(**right, ShellValue::Bool(true)));
            }
            other => panic!("Expected LogicalOr, got {:?}", other),
        }
    }

    // ===== MethodCall: std::env::args().nth(N).unwrap() =====

    #[test]
    fn test_EXPR_VAL_028_method_call_env_args_nth_unwrap() {
        // Pattern: std::env::args().nth(1).unwrap() -> $1
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::FunctionCall {
                    name: "std::env::args".to_string(),
                    args: vec![],
                }),
                method: "nth".to_string(),
                args: vec![Expr::Literal(Literal::U32(1))],
            }),
            method: "unwrap".to_string(),
            args: vec![],
        };
        let ir = convert_let_stmt("first_arg", expr);
        let val = extract_let_value(&ir);
        match val {
            ShellValue::Arg { position } => {
                assert_eq!(*position, Some(1));
            }
            other => panic!("Expected Arg {{ position: Some(1) }}, got {:?}", other),
        }
    }

    // ===== MethodCall: args.get(N).unwrap_or(default) =====

    #[test]
    fn test_EXPR_VAL_029_method_call_args_get_unwrap_or() {
        // Pattern: args.get(2).unwrap_or("default") -> ${2:-default}
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::Variable("args".to_string())),
                method: "get".to_string(),
                args: vec![Expr::Literal(Literal::U32(2))],
            }),
            method: "unwrap_or".to_string(),
            args: vec![Expr::Literal(Literal::Str("fallback".to_string()))],
        };
        let ir = convert_let_stmt("param", expr);
        let val = extract_let_value(&ir);
        match val {
            ShellValue::ArgWithDefault { position, default } => {
                assert_eq!(*position, 2);
                assert_eq!(default, "fallback");
            }
            other => panic!("Expected ArgWithDefault {{ 2, fallback }}, got {:?}", other),
        }
    }

    // ===== MethodCall: std::env::args().nth(N).unwrap_or(default) =====

    #[test]
    fn test_EXPR_VAL_030_method_call_env_args_nth_unwrap_or() {
        // Pattern: std::env::args().nth(0).unwrap_or("script") -> ${0:-script}
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::FunctionCall {
                    name: "std::env::args".to_string(),
                    args: vec![],
                }),
                method: "nth".to_string(),
                args: vec![Expr::Literal(Literal::U32(0))],
            }),
            method: "unwrap_or".to_string(),
            args: vec![Expr::Literal(Literal::Str("default_script".to_string()))],
        };
        let ir = convert_let_stmt("script_name", expr);
        let val = extract_let_value(&ir);
        match val {
            ShellValue::ArgWithDefault { position, default } => {
                assert_eq!(*position, 0);
                assert_eq!(default, "default_script");
            }
            other => panic!(
                "Expected ArgWithDefault {{ 0, default_script }}, got {:?}",
                other
            ),
        }
    }

    // ===== MethodCall: unrecognized pattern =====

    #[test]
    fn test_EXPR_VAL_031_method_call_unrecognized_falls_to_unknown() {
        // A method call that doesn't match any recognized pattern
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::Variable("vec".to_string())),
            method: "len".to_string(),
            args: vec![],
        };
        let ir = convert_let_stmt("length", expr);
        let val = extract_let_value(&ir);
        match val {
            ShellValue::String(s) => assert_eq!(s, "unknown"),
            other => panic!("Expected String(\"unknown\"), got {:?}", other),
        }
    }

    // ===== PositionalArgs =====

    #[test]
    fn test_EXPR_VAL_032_positional_args() {
        let ir = convert_let_stmt("all_args", Expr::PositionalArgs);
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::Arg { position: None }));
    }

    // ===== Fallback (_) branch =====

    #[test]
    fn test_EXPR_VAL_033_array_expr_expands_to_indexed_lets() {
        // Array in let context: let arr = [1, 2] → arr_0=1; arr_1=2
        let expr = Expr::Array(vec![
            Expr::Literal(Literal::U32(1)),
            Expr::Literal(Literal::U32(2)),
        ]);
        let ir = convert_let_stmt("arr", expr);
        // Should produce Sequence([Sequence([Let arr_0=1, Let arr_1=2]])
        match &ir {
            ShellIR::Sequence(stmts) => {
                // The outer sequence wraps the inner array expansion
                match &stmts[0] {
                    ShellIR::Sequence(inner) => {
                        assert_eq!(inner.len(), 2);
                        match &inner[0] {
                            ShellIR::Let { name, value, .. } => {
                                assert_eq!(name, "arr_0");
                                assert!(matches!(value, ShellValue::String(s) if s == "1"));
                            }
                            other => panic!("Expected Let arr_0, got {:?}", other),
                        }
                        match &inner[1] {
                            ShellIR::Let { name, value, .. } => {
                                assert_eq!(name, "arr_1");
                                assert!(matches!(value, ShellValue::String(s) if s == "2"));
                            }
                            other => panic!("Expected Let arr_1, got {:?}", other),
                        }
                    }
                    other => panic!("Expected inner Sequence, got {:?}", other),
                }
            }
            other => panic!("Expected Sequence, got {:?}", other),
        }
    }

    #[test]
    fn test_EXPR_VAL_034_index_expr_becomes_variable() {
        // arr[0] → $arr_0
        let expr = Expr::Index {
            object: Box::new(Expr::Variable("arr".to_string())),
            index: Box::new(Expr::Literal(Literal::U32(0))),
        };
        let ir = convert_let_stmt("elem", expr);
        let val = extract_let_value(&ir);
        match val {
            ShellValue::Variable(name) => assert_eq!(name, "arr_0"),
            other => panic!("Expected Variable(\"arr_0\"), got {:?}", other),
        }
    }

    #[test]
    fn test_EXPR_VAL_035_fallback_range_expr() {
        // Range expressions hit the fallback _ branch
        let expr = Expr::Range {
            start: Box::new(Expr::Literal(Literal::U32(0))),
            end: Box::new(Expr::Literal(Literal::U32(10))),
            inclusive: false,
        };
        let ir = convert_let_stmt("rng", expr);
        let val = extract_let_value(&ir);
        match val {
            ShellValue::String(s) => assert_eq!(s, "unknown"),
            other => panic!("Expected String(\"unknown\") fallback, got {:?}", other),
        }
    }

    // ===== Edge cases: MethodCall partial matches that fall through =====

    #[test]
    fn test_EXPR_VAL_036_method_unwrap_non_nth_receiver() {
        // .unwrap() on something that is NOT .nth() -> falls to "unknown"
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::Variable("x".to_string())),
                method: "first".to_string(), // not "nth"
                args: vec![Expr::Literal(Literal::U32(0))],
            }),
            method: "unwrap".to_string(),
            args: vec![],
        };
        let ir = convert_let_stmt("val", expr);
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
    }

    #[test]
    fn test_EXPR_VAL_037_method_unwrap_or_non_get_non_nth() {
        // .unwrap_or(default) on something that is NOT .get() or .nth() -> falls to "unknown"
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::Variable("x".to_string())),
                method: "find".to_string(), // not "get" or "nth"
                args: vec![Expr::Literal(Literal::U32(0))],
            }),
            method: "unwrap_or".to_string(),
            args: vec![Expr::Literal(Literal::Str("default".to_string()))],
        };
        let ir = convert_let_stmt("val", expr);
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
    }

    #[test]
    fn test_EXPR_VAL_038_method_unwrap_with_args_not_recognized() {
        // .unwrap() with non-empty args -> not the recognized pattern -> falls through
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::Variable("x".to_string())),
            method: "unwrap".to_string(),
            args: vec![Expr::Literal(Literal::U32(42))], // unwrap doesn't take args normally
        };
        let ir = convert_let_stmt("val", expr);
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
    }

    // ===== FunctionCall: arg with i32 =====

    #[test]
    fn test_EXPR_VAL_039_func_arg_with_i32_position() {
        let ir = convert_let_stmt(
            "second",
            Expr::FunctionCall {
                name: "arg".to_string(),
                args: vec![Expr::Literal(Literal::I32(2))],
            },
        );
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::Arg { position: Some(2) }));
    }

    // ===== FunctionCall: arg() with no args (error) =====

    #[test]
    fn test_EXPR_VAL_040_func_arg_no_args_error() {
        let err = convert_let_stmt_err(
            "bad",
            Expr::FunctionCall {
                name: "arg".to_string(),
                args: vec![],
            },
        );
        let msg = err.to_string();
        assert!(
            msg.contains("requires at least one argument"),
            "Expected arg() requires at least one argument, got: {}",
            msg
        );
    }

    // ===== FunctionCall: arg() with string arg (error) =====

    #[test]
    fn test_EXPR_VAL_041_func_arg_string_position_error() {
        let err = convert_let_stmt_err(
            "bad",
            Expr::FunctionCall {
                name: "arg".to_string(),
                args: vec![Expr::Literal(Literal::Str("one".to_string()))],
            },
        );
        let msg = err.to_string();
        assert!(
            msg.contains("integer literal"),
            "Expected integer literal error, got: {}",
            msg
        );
    }

    // ===== FunctionCall: regular function with args becomes CommandSubst =====

    #[test]
    fn test_EXPR_VAL_042_func_call_with_args_becomes_command_subst() {
        let ir = convert_let_stmt(
            "output",
            Expr::FunctionCall {
                name: "date".to_string(),
                args: vec![Expr::Literal(Literal::Str("+%Y".to_string()))],
            },
        );
        let val = extract_let_value(&ir);
        match val {
            ShellValue::CommandSubst(cmd) => {
                assert_eq!(cmd.program, "date");
                assert_eq!(cmd.args.len(), 1);
            }
            other => panic!("Expected CommandSubst(date), got {:?}", other),
        }
    }

    // ===== MethodCall: env::args().nth() with non-U32 falls through =====

    #[test]
    fn test_EXPR_VAL_043_method_env_args_nth_non_u32_falls_through() {
        // std::env::args().nth("abc").unwrap() - nth arg is not U32
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::FunctionCall {
                    name: "std::env::args".to_string(),
                    args: vec![],
                }),
                method: "nth".to_string(),
                args: vec![Expr::Literal(Literal::Str("abc".to_string()))],
            }),
            method: "unwrap".to_string(),
            args: vec![],
        };
        let ir = convert_let_stmt("val", expr);
        let val = extract_let_value(&ir);
        // Non-U32 arg to nth doesn't match -> falls through to "unknown"
        assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
    }

    // ===== MethodCall: unwrap_or with non-string default falls through =====

    #[test]
    fn test_EXPR_VAL_044_method_get_unwrap_or_non_string_default() {
        // args.get(1).unwrap_or(42) - default is not Str
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::Variable("args".to_string())),
                method: "get".to_string(),
                args: vec![Expr::Literal(Literal::U32(1))],
            }),
            method: "unwrap_or".to_string(),
            args: vec![Expr::Literal(Literal::U32(42))],
        };
        let ir = convert_let_stmt("val", expr);
        let val = extract_let_value(&ir);
        // Non-Str default doesn't match -> falls through to "unknown"
        assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
    }

    // ===== MethodCall: unwrap on non-MethodCall receiver =====

    #[test]
    fn test_EXPR_VAL_045_method_unwrap_on_non_method_receiver() {
        // variable.unwrap() - receiver is Variable, not MethodCall
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::Variable("option_val".to_string())),
            method: "unwrap".to_string(),
            args: vec![],
        };
        let ir = convert_let_stmt("val", expr);
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
    }

    // ===== Binary::Sub (missed in EXPR_VAL_025) =====

    #[test]
    fn test_EXPR_VAL_046_binary_sub() {
        let ir = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::Sub,
                left: Box::new(Expr::Literal(Literal::U32(10))),
                right: Box::new(Expr::Literal(Literal::U32(3))),
            },
        );
        assert!(matches!(
            extract_let_value(&ir),
            ShellValue::Arithmetic {
                op: shell_ir::ArithmeticOp::Sub,
                ..
            }
        ));
    }

    // ===== Binary::Ne with string operands → StrNe =====

    #[test]
    fn test_EXPR_VAL_047_binary_ne_string() {
        let ir = convert_let_stmt(
            "r",
            Expr::Binary {
                op: BinaryOp::Ne,
                left: Box::new(Expr::Literal(Literal::Str("hello".to_string()))),
                right: Box::new(Expr::Literal(Literal::Str("world".to_string()))),
            },
        );
        assert!(matches!(
            extract_let_value(&ir),
            ShellValue::Comparison {
                op: shell_ir::ComparisonOp::StrNe,
                ..
            }
        ));
    }

    // ===== FunctionCall: stdlib function as value → rash_<name> =====

    #[test]
    fn test_EXPR_VAL_048_func_stdlib_becomes_rash_prefixed_command_subst() {
        let ir = convert_let_stmt(
            "trimmed",
            Expr::FunctionCall {
                name: "string_trim".to_string(),
                args: vec![Expr::Literal(Literal::Str("  hello  ".to_string()))],
            },
        );
        let val = extract_let_value(&ir);
        match val {
            ShellValue::CommandSubst(cmd) => {
                assert_eq!(cmd.program, "rash_string_trim");
                assert_eq!(cmd.args.len(), 1);
            }
            other => panic!("Expected CommandSubst(rash_string_trim), got {:?}", other),
        }
    }

    // ===== MethodCall: unwrap_or on non-MethodCall receiver =====

    #[test]
    fn test_EXPR_VAL_049_method_unwrap_or_on_variable_receiver() {
        // variable.unwrap_or("default") - receiver is Variable, not MethodCall
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::Variable("maybe_val".to_string())),
            method: "unwrap_or".to_string(),
            args: vec![Expr::Literal(Literal::Str("fallback".to_string()))],
        };
        let ir = convert_let_stmt("val", expr);
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
    }

    // ===== MethodCall: unwrap_or + args.get(N) where N is not U32 =====

    #[test]
    fn test_EXPR_VAL_050_method_get_unwrap_or_non_u32_position() {
        // args.get("abc").unwrap_or("default") - position is Str, not U32
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::Variable("args".to_string())),
                method: "get".to_string(),
                args: vec![Expr::Literal(Literal::Str("abc".to_string()))],
            }),
            method: "unwrap_or".to_string(),
            args: vec![Expr::Literal(Literal::Str("default".to_string()))],
        };
        let ir = convert_let_stmt("val", expr);
        let val = extract_let_value(&ir);
        // Non-U32 position in get() doesn't match -> falls through
        assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
    }

    // ===== MethodCall: env::args().nth(N).unwrap_or() where N is not U32 =====

    #[test]
    fn test_EXPR_VAL_051_method_env_args_nth_unwrap_or_non_u32_position() {
        // std::env::args().nth("x").unwrap_or("default") - nth arg is Str
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::FunctionCall {
                    name: "std::env::args".to_string(),
                    args: vec![],
                }),
                method: "nth".to_string(),
                args: vec![Expr::Literal(Literal::Str("x".to_string()))],
            }),
            method: "unwrap_or".to_string(),
            args: vec![Expr::Literal(Literal::Str("default".to_string()))],
        };
        let ir = convert_let_stmt("val", expr);
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
    }

    // ===== MethodCall: env::args().nth(N).unwrap_or() where default is not Str =====

    #[test]
    fn test_EXPR_VAL_052_method_env_args_nth_unwrap_or_non_str_default() {
        // std::env::args().nth(0).unwrap_or(42) - default is U32, not Str
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::FunctionCall {
                    name: "std::env::args".to_string(),
                    args: vec![],
                }),
                method: "nth".to_string(),
                args: vec![Expr::Literal(Literal::U32(0))],
            }),
            method: "unwrap_or".to_string(),
            args: vec![Expr::Literal(Literal::U32(42))],
        };
        let ir = convert_let_stmt("val", expr);
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
    }

    // ===== MethodCall: non-env::args().nth().unwrap_or() =====

    #[test]
    fn test_EXPR_VAL_053_method_nth_unwrap_or_non_env_args_receiver() {
        // other_func().nth(0).unwrap_or("default") - receiver is not std::env::args
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::FunctionCall {
                    name: "some_other_func".to_string(),
                    args: vec![],
                }),
                method: "nth".to_string(),
                args: vec![Expr::Literal(Literal::U32(0))],
            }),
            method: "unwrap_or".to_string(),
            args: vec![Expr::Literal(Literal::Str("default".to_string()))],
        };
        let ir = convert_let_stmt("val", expr);
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
    }

    // ===== MethodCall: env::args().nth().unwrap() where env::args has args =====

    #[test]
    fn test_EXPR_VAL_054_method_env_args_with_args_nth_unwrap() {
        // std::env::args(42).nth(0).unwrap() - fn_args is not empty
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::FunctionCall {
                    name: "std::env::args".to_string(),
                    args: vec![Expr::Literal(Literal::U32(42))],
                }),
                method: "nth".to_string(),
                args: vec![Expr::Literal(Literal::U32(0))],
            }),
            method: "unwrap".to_string(),
            args: vec![],
        };
        let ir = convert_let_stmt("val", expr);
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
    }

    // ===== MethodCall: nth().unwrap() where receiver is not FunctionCall =====

    #[test]
    fn test_EXPR_VAL_055_method_nth_unwrap_variable_receiver() {
        // iter.nth(0).unwrap() - inner receiver of nth is Variable, not FunctionCall
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::Variable("iter".to_string())),
                method: "nth".to_string(),
                args: vec![Expr::Literal(Literal::U32(0))],
            }),
            method: "unwrap".to_string(),
            args: vec![],
        };
        let ir = convert_let_stmt("val", expr);
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
    }

    // ===== MethodCall: nth().unwrap_or() where inner receiver is Variable =====

    #[test]
    fn test_EXPR_VAL_056_method_nth_unwrap_or_variable_inner_receiver() {
        // iter.nth(0).unwrap_or("default") - inner receiver of nth is Variable
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::MethodCall {
                receiver: Box::new(Expr::Variable("iter".to_string())),
                method: "nth".to_string(),
                args: vec![Expr::Literal(Literal::U32(0))],
            }),
            method: "unwrap_or".to_string(),
            args: vec![Expr::Literal(Literal::Str("default".to_string()))],
        };
        let ir = convert_let_stmt("val", expr);
        let val = extract_let_value(&ir);
        assert!(matches!(val, ShellValue::String(s) if s == "unknown"));
    }
}
