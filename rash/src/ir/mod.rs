//! Intermediate Representation (IR) module
//!
//! ## Safety Note
//! IR uses unwrap() on validated AST operations and checked invariants.
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

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
    // Converter state (currently stateless)
}

impl IrConverter {
    fn new() -> Self {
        Self {}
    }

    fn convert(&self, ast: &RestrictedAst) -> Result<ShellIR> {
        let mut all_ir = Vec::new();

        // Convert all user-defined functions (except main) to shell functions
        for function in &ast.functions {
            if function.name != ast.entry_point {
                let params: Vec<String> = function.params.iter().map(|p| p.name.clone()).collect();
                let mut body_stmts = Vec::new();

                // Convert function body statements
                for (i, stmt) in function.body.iter().enumerate() {
                    let is_last = i == function.body.len() - 1;
                    let has_return_type =
                        !matches!(function.return_type, crate::ast::restricted::Type::Void);

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

    /// Convert a statement in a function context (handles return values)
    fn convert_stmt_in_function(
        &self,
        stmt: &crate::ast::Stmt,
        should_echo: bool,
    ) -> Result<ShellIR> {
        use crate::ast::Stmt;

        match stmt {
            Stmt::Expr(expr) if should_echo => {
                // Last expression in function with return type - emit as echo
                let value = self.convert_expr_to_value(expr)?;
                Ok(ShellIR::Echo { value })
            }
            _ => self.convert_stmt(stmt),
        }
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
                        let mut end_val = self.convert_expr_to_value(end)?;

                        // For exclusive range (0..3), adjust end to be inclusive (0..=2)
                        if !inclusive {
                            // Subtract 1 from end value
                            if let ShellValue::String(s) = &end_val {
                                if let Ok(n) = s.parse::<i32>() {
                                    end_val = ShellValue::String((n - 1).to_string());
                                }
                            }
                        }

                        (start_val, end_val)
                    }
                    _ => {
                        return Err(crate::models::Error::Validation(
                            "For loops only support range expressions (e.g., 0..10)".to_string(),
                        ))
                    }
                };

                // Convert body
                let body_ir = self.convert_stmts(body)?;

                Ok(ShellIR::For {
                    var,
                    start,
                    end,
                    body: Box::new(body_ir),
                })
            }
            Stmt::Match { scrutinee, arms } => {
                // Convert the scrutinee to a shell value
                let scrutinee_value = self.convert_expr_to_value(scrutinee)?;

                // Convert each match arm to a case arm
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

                // Convert body
                let body_ir = self.convert_stmts(body)?;

                Ok(ShellIR::While {
                    condition: condition_value,
                    body: Box::new(body_ir),
                })
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

    fn convert_expr(&self, expr: &crate::ast::Expr) -> Result<ShellIR> {
        use crate::ast::Expr;

        match expr {
            Expr::FunctionCall { name, args } => {
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
                Literal::U32(n) => Ok(ShellValue::String(n.to_string())),
                Literal::I32(n) => Ok(ShellValue::String(n.to_string())),
                Literal::Str(s) => Ok(ShellValue::String(s.clone())),
            },
            Expr::Variable(name) => Ok(ShellValue::Variable(name.clone())),
            Expr::FunctionCall { name, args } => {
                // Sprint 27a: Handle env() and env_var_or() specially
                if name == "env" || name == "env_var_or" {
                    // Extract variable name from first argument
                    let var_name = match &args[0] {
                        Expr::Literal(Literal::Str(s)) => s.clone(),
                        _ => {
                            return Err(crate::models::Error::Validation(format!(
                                "{}() requires string literal for variable name",
                                name
                            )))
                        }
                    };

                    // Validate var name (security)
                    if !var_name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
                    {
                        return Err(crate::models::Error::Validation(format!(
                            "Invalid environment variable name: '{}'",
                            var_name
                        )));
                    }

                    // Extract default value for env_var_or()
                    let default = if name == "env_var_or" {
                        match &args.get(1) {
                            Some(Expr::Literal(Literal::Str(s))) => Some(s.clone()),
                            _ => {
                                return Err(crate::models::Error::Validation(
                                    "env_var_or() requires string literal for default value"
                                        .to_string(),
                                ))
                            }
                        }
                    } else {
                        None
                    };

                    return Ok(ShellValue::EnvVar {
                        name: var_name,
                        default,
                    });
                }

                // Sprint 27b: Handle arg(), args(), and arg_count() specially
                if name == "arg" {
                    // Extract position from first argument
                    let position = match &args[0] {
                        Expr::Literal(Literal::U32(n)) => *n as usize,
                        Expr::Literal(Literal::I32(n)) => *n as usize,
                        _ => {
                            return Err(crate::models::Error::Validation(
                                "arg() requires integer literal for position".to_string(),
                            ))
                        }
                    };

                    // Validate position (must be >= 1)
                    if position == 0 {
                        return Err(crate::models::Error::Validation(
                            "arg() position must be >= 1 (use arg(1) for first argument)"
                                .to_string(),
                        ));
                    }

                    return Ok(ShellValue::Arg {
                        position: Some(position),
                    });
                }

                if name == "args" {
                    return Ok(ShellValue::Arg { position: None }); // None = $@
                }

                if name == "arg_count" {
                    return Ok(ShellValue::ArgCount);
                }

                // Sprint 27c: Handle exit_code() specially
                if name == "exit_code" {
                    return Ok(ShellValue::ExitCode);
                }

                // Function call used as value - capture output with command substitution
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

                Ok(ShellValue::CommandSubst(Command {
                    program,
                    args: cmd_args,
                }))
            }
            Expr::Unary { op, operand } => {
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
            Expr::Binary { op, left, right } => {
                use crate::ast::restricted::BinaryOp;
                let left_val = self.convert_expr_to_value(left)?;
                let right_val = self.convert_expr_to_value(right)?;

                // Convert comparison and arithmetic operators to proper variants
                match op {
                    // Comparison operators - detect string vs numeric
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
                    // Arithmetic operators
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
                    // Logical operators
                    BinaryOp::And => Ok(ShellValue::LogicalAnd {
                        left: Box::new(left_val),
                        right: Box::new(right_val),
                    }),
                    BinaryOp::Or => Ok(ShellValue::LogicalOr {
                        left: Box::new(left_val),
                        right: Box::new(right_val),
                    }),
                }
            }
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => {
                // PARAM-SPEC-005: Detect std::env::args().nth(N).unwrap() pattern (without default)
                // This becomes $N in shell (e.g., $0 for script name, $1 for first arg)
                if method == "unwrap" && args.is_empty() {
                    if let Expr::MethodCall {
                        receiver: inner_receiver,
                        method: inner_method,
                        args: inner_args,
                    } = &**receiver
                    {
                        if inner_method == "nth" && inner_args.len() == 1 {
                            // Check if inner receiver is std::env::args()
                            if let Expr::FunctionCall {
                                name,
                                args: fn_args,
                            } = &**inner_receiver
                            {
                                if name == "std::env::args" && fn_args.is_empty() {
                                    // Extract the position number
                                    if let Expr::Literal(Literal::U32(n)) = &inner_args[0] {
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
                // This becomes ${N:-default} in shell
                if method == "unwrap_or" && args.len() == 1 {
                    // Check if receiver is args.get(N)
                    if let Expr::MethodCall {
                        receiver: inner_receiver,
                        method: inner_method,
                        args: inner_args,
                    } = &**receiver
                    {
                        if inner_method == "get" && inner_args.len() == 1 {
                            // Check if inner receiver is a variable holding positional args
                            // For now, we'll check if it's assigned from PositionalArgs
                            // Extract the position number
                            if let Expr::Literal(Literal::U32(n)) = &inner_args[0] {
                                // Extract the default value
                                if let Expr::Literal(Literal::Str(default_val)) = &args[0] {
                                    return Ok(ShellValue::ArgWithDefault {
                                        position: *n as usize,
                                        default: default_val.clone(),
                                    });
                                }
                            }
                        }

                        // PARAM-SPEC-005: Detect std::env::args().nth(N).unwrap_or(default) pattern
                        // This becomes ${N:-default} in shell (e.g., ${0:-default} for script name)
                        if inner_method == "nth" && inner_args.len() == 1 {
                            // Check if inner receiver is std::env::args()
                            if let Expr::FunctionCall {
                                name,
                                args: fn_args,
                            } = &**inner_receiver
                            {
                                if name == "std::env::args" && fn_args.is_empty() {
                                    // Extract the position number
                                    if let Expr::Literal(Literal::U32(n)) = &inner_args[0] {
                                        // Extract the default value
                                        if let Expr::Literal(Literal::Str(default_val)) = &args[0] {
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

                // Not a recognized pattern - fall through to unknown
                Ok(ShellValue::String("unknown".to_string()))
            }
            Expr::PositionalArgs => {
                // std::env::args().collect() → $@ (all positional parameters)
                Ok(ShellValue::Arg { position: None })
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
            Pattern::Tuple(_) | Pattern::Struct { .. } => Err(crate::models::Error::Validation(
                "Tuple and struct patterns not yet supported in match expressions".to_string(),
            )),
        }
    }
}

fn constant_fold(ir: ShellIR) -> ShellIR {
    // Constant folding pass - fold string concat and arithmetic
    let mut transform_fn = |node| match node {
        ShellIR::Let {
            name,
            value: ShellValue::Concat(parts),
            effects,
        } => {
            // Fold string concatenation
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
            // Recursively fold nested arithmetic first
            let folded_left = fold_arithmetic_value(*left);
            let folded_right = fold_arithmetic_value(*right);

            // Try to fold arithmetic if both operands are constant strings containing integers
            if let (ShellValue::String(left_str), ShellValue::String(right_str)) =
                (&folded_left, &folded_right)
            {
                if let (Ok(left_num), Ok(right_num)) =
                    (left_str.parse::<i64>(), right_str.parse::<i64>())
                {
                    let result = match op {
                        shell_ir::ArithmeticOp::Add => left_num + right_num,
                        shell_ir::ArithmeticOp::Sub => left_num - right_num,
                        shell_ir::ArithmeticOp::Mul => left_num * right_num,
                        shell_ir::ArithmeticOp::Div => {
                            if right_num != 0 {
                                left_num / right_num
                            } else {
                                // Division by zero - don't fold
                                return ShellIR::Let {
                                    name,
                                    value: ShellValue::Arithmetic {
                                        op,
                                        left: Box::new(folded_left),
                                        right: Box::new(folded_right),
                                    },
                                    effects,
                                };
                            }
                        }
                        shell_ir::ArithmeticOp::Mod => {
                            if right_num != 0 {
                                left_num % right_num
                            } else {
                                // Modulo by zero - don't fold
                                return ShellIR::Let {
                                    name,
                                    value: ShellValue::Arithmetic {
                                        op,
                                        left: Box::new(folded_left),
                                        right: Box::new(folded_right),
                                    },
                                    effects,
                                };
                            }
                        }
                    };

                    return ShellIR::Let {
                        name,
                        value: ShellValue::String(result.to_string()),
                        effects,
                    };
                }
            }

            // Cannot fold - return with recursively folded operands
            ShellIR::Let {
                name,
                value: ShellValue::Arithmetic {
                    op,
                    left: Box::new(folded_left),
                    right: Box::new(folded_right),
                },
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

            // Try to fold if both are constant integers
            if let (ShellValue::String(left_str), ShellValue::String(right_str)) =
                (&folded_left, &folded_right)
            {
                if let (Ok(left_num), Ok(right_num)) =
                    (left_str.parse::<i64>(), right_str.parse::<i64>())
                {
                    let result = match op {
                        shell_ir::ArithmeticOp::Add => left_num + right_num,
                        shell_ir::ArithmeticOp::Sub => left_num - right_num,
                        shell_ir::ArithmeticOp::Mul => left_num * right_num,
                        shell_ir::ArithmeticOp::Div => {
                            if right_num != 0 {
                                left_num / right_num
                            } else {
                                // Division by zero - don't fold
                                return ShellValue::Arithmetic {
                                    op,
                                    left: Box::new(folded_left),
                                    right: Box::new(folded_right),
                                };
                            }
                        }
                        shell_ir::ArithmeticOp::Mod => {
                            if right_num != 0 {
                                left_num % right_num
                            } else {
                                // Modulo by zero - don't fold
                                return ShellValue::Arithmetic {
                                    op,
                                    left: Box::new(folded_left),
                                    right: Box::new(folded_right),
                                };
                            }
                        }
                    };

                    return ShellValue::String(result.to_string());
                }
            }

            // Cannot fold
            ShellValue::Arithmetic {
                op,
                left: Box::new(folded_left),
                right: Box::new(folded_right),
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
