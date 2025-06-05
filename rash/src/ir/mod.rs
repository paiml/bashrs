pub mod effects;
pub mod shell_ir;

#[cfg(test)]
mod tests;

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
    // Converter state
}

impl IrConverter {
    fn new() -> Self {
        Self {}
    }

    fn convert(&self, ast: &RestrictedAst) -> Result<ShellIR> {
        // Find the entry point function
        let entry_function = ast
            .functions
            .iter()
            .find(|f| f.name == ast.entry_point)
            .ok_or_else(|| Error::IrGeneration("Entry point not found".to_string()))?;

        // Convert the main function to IR
        let mut statements = Vec::new();

        for stmt in &entry_function.body {
            statements.push(self.convert_stmt(stmt)?);
        }

        Ok(ShellIR::Sequence(statements))
    }

    fn convert_stmt(&self, stmt: &crate::ast::Stmt) -> Result<ShellIR> {
        use crate::ast::Stmt;

        match stmt {
            Stmt::Let { name, value } => {
                let shell_value = self.convert_expr_to_value(value)?;
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
            // Placeholder for new AST nodes - TODO: implement properly
            _ => Ok(ShellIR::Noop), // Match, For, While, Break, Continue
        }
    }

    fn convert_stmts(&self, stmts: &[crate::ast::Stmt]) -> Result<ShellIR> {
        let mut ir_stmts = Vec::new();
        for stmt in stmts {
            ir_stmts.push(self.convert_stmt(stmt)?);
        }
        Ok(ShellIR::Sequence(ir_stmts))
    }

    fn convert_expr(&self, expr: &crate::ast::Expr) -> Result<ShellIR> {
        use crate::ast::Expr;

        match expr {
            Expr::FunctionCall { name, args } => {
                // Convert function calls to shell commands
                let mut cmd_args = Vec::new();
                for arg in args {
                    cmd_args.push(self.convert_expr_to_value(arg)?);
                }

                Ok(ShellIR::Exec {
                    cmd: Command {
                        program: name.clone(),
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
                Literal::U32(n) => Ok(ShellValue::String(n.to_string())),
                Literal::Str(s) => Ok(ShellValue::String(s.clone())),
            },
            Expr::Variable(name) => Ok(ShellValue::Variable(name.clone())),
            Expr::Binary { op: _, left, right } => {
                let left_val = self.convert_expr_to_value(left)?;
                let right_val = self.convert_expr_to_value(right)?;
                Ok(ShellValue::Concat(vec![left_val, right_val])) // Simplified
            }
            _ => Ok(ShellValue::String("unknown".to_string())), // Fallback
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
}

fn constant_fold(ir: ShellIR) -> ShellIR {
    // Simple constant folding pass
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
        _ => node,
    };
    transform_ir(ir, &mut transform_fn)
}

fn eliminate_dead_code(ir: ShellIR) -> ShellIR {
    // Simple dead code elimination
    ir // Placeholder - would implement actual DCE
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
        other => other,
    };

    transform(transformed)
}
