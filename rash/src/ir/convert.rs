//! Core AST-to-IR conversion: `new`, `convert`, and shadow/loop helpers.
//!
//! Statement conversion is split across submodules:
//! - `convert_stmt`: Top-level `convert_stmt` / `convert_stmts`
//! - `convert_fn`: Function-context `convert_stmt_in_function` / `convert_stmts_in_function`

use super::{EffectSet, IrConverter, ShellIR, ShellValue};
use crate::ast::RestrictedAst;
use crate::models::{Error, Result};

impl IrConverter {
    pub(crate) fn new() -> Self {
        Self {
            arrays: std::cell::RefCell::new(std::collections::HashMap::new()),
            declared_vars: std::cell::RefCell::new(std::collections::HashSet::new()),
        }
    }

    pub(crate) fn convert(&self, ast: &RestrictedAst) -> Result<ShellIR> {
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

    /// Detect variables that are shadowed inside a loop body.
    /// Returns variable names that have `Stmt::Let { declaration: true }` in the body
    /// AND are already in `declared_vars` (i.e., exist in the outer scope).
    pub(crate) fn detect_shadows(&self, body: &[crate::ast::Stmt]) -> Vec<String> {
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
    pub(crate) fn convert_loop_body_with_shadows(
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
    pub(crate) fn wrap_with_shadow_save_restore(
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
    pub(crate) fn replace_var_refs_in_value(
        value: &ShellValue,
        old_name: &str,
        new_name: &str,
    ) -> ShellValue {
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
}
