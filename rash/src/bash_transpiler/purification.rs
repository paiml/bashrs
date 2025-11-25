//! Purification Transforms for Bash Scripts
//!
//! Transforms bash scripts to ensure:
//! - Idempotency: Running multiple times produces same result
//! - Determinism: No random or time-based values
//! - Side-effect isolation: Clear tracking of mutations

use crate::bash_parser::ast::*;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PurificationError {
    #[error("Cannot purify non-deterministic construct: {0}")]
    NonDeterministicConstruct(String),

    #[error("Side effect cannot be made idempotent: {0}")]
    NonIdempotentSideEffect(String),
}

pub type PurificationResult<T> = Result<T, PurificationError>;

/// Configuration for purification
#[derive(Debug, Clone)]
pub struct PurificationOptions {
    /// Enforce strict idempotency (fail on non-idempotent operations)
    pub strict_idempotency: bool,

    /// Remove all non-deterministic elements
    pub remove_non_deterministic: bool,

    /// Track all side effects
    pub track_side_effects: bool,
}

impl Default for PurificationOptions {
    fn default() -> Self {
        Self {
            strict_idempotency: true,
            remove_non_deterministic: true,
            track_side_effects: true,
        }
    }
}

/// Report of purification transformations applied
#[derive(Debug, Clone)]
pub struct PurificationReport {
    pub idempotency_fixes: Vec<String>,
    pub determinism_fixes: Vec<String>,
    pub side_effects_isolated: Vec<String>,
    pub warnings: Vec<String>,
}

impl PurificationReport {
    fn new() -> Self {
        Self {
            idempotency_fixes: Vec::new(),
            determinism_fixes: Vec::new(),
            side_effects_isolated: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Purifies bash AST to ensure idempotency and determinism
pub struct Purifier {
    options: PurificationOptions,
    report: PurificationReport,
    non_deterministic_vars: HashSet<String>,
}

impl Purifier {
    pub fn new(options: PurificationOptions) -> Self {
        let mut non_deterministic_vars = HashSet::new();
        // Common non-deterministic bash variables
        non_deterministic_vars.insert("RANDOM".to_string());
        non_deterministic_vars.insert("SECONDS".to_string());
        non_deterministic_vars.insert("BASHPID".to_string());
        non_deterministic_vars.insert("PPID".to_string());

        Self {
            options,
            report: PurificationReport::new(),
            non_deterministic_vars,
        }
    }

    pub fn purify(&mut self, ast: &BashAst) -> PurificationResult<BashAst> {
        let mut purified_statements = Vec::new();

        for stmt in &ast.statements {
            let purified = self.purify_statement(stmt)?;
            purified_statements.push(purified);
        }

        Ok(BashAst {
            statements: purified_statements,
            metadata: ast.metadata.clone(),
        })
    }

    pub fn report(&self) -> &PurificationReport {
        &self.report
    }

    fn purify_statement(&mut self, stmt: &BashStmt) -> PurificationResult<BashStmt> {
        match stmt {
            BashStmt::Assignment {
                name,
                value,
                exported,
                span,
            } => {
                // Check if value contains non-deterministic elements
                let purified_value = self.purify_expression(value)?;

                Ok(BashStmt::Assignment {
                    name: name.clone(),
                    value: purified_value,
                    exported: *exported,
                    span: *span,
                })
            }

            BashStmt::Command {
                name,
                args,
                redirects: _,
                span,
            } => {
                // Detect and transform non-idempotent operations
                let (purified_cmds, idempotent_wrapper) =
                    self.make_command_idempotent(name, args, *span)?;

                if let Some(wrapper) = idempotent_wrapper {
                    self.report.idempotency_fixes.push(wrapper);
                }

                // If multiple statements were generated (e.g., permission check + command),
                // we need to handle this specially
                if purified_cmds.len() == 1 {
                    // SAFETY: We verified length is 1, so next() will return Some
                    Ok(purified_cmds.into_iter().next().unwrap_or_else(|| {
                        // This should never happen given len check above
                        BashStmt::Comment {
                            text: "ERROR: empty purified_cmds".to_string(),
                            span: *span,
                        }
                    }))
                } else {
                    // For now, we'll return a Pipeline to group multiple statements
                    // This ensures they're executed together
                    Ok(BashStmt::Pipeline {
                        commands: purified_cmds,
                        span: *span,
                    })
                }
            }

            BashStmt::Function { name, body, span } => {
                let mut purified_body = Vec::new();
                for stmt in body {
                    purified_body.push(self.purify_statement(stmt)?);
                }

                Ok(BashStmt::Function {
                    name: name.clone(),
                    body: purified_body,
                    span: *span,
                })
            }

            BashStmt::If {
                condition,
                then_block,
                elif_blocks,
                else_block,
                span,
            } => {
                let purified_condition = self.purify_expression(condition)?;

                let mut purified_then = Vec::new();
                for stmt in then_block {
                    purified_then.push(self.purify_statement(stmt)?);
                }

                let mut purified_elif = Vec::new();
                for (cond, body) in elif_blocks {
                    let p_cond = self.purify_expression(cond)?;
                    let mut p_body = Vec::new();
                    for stmt in body {
                        p_body.push(self.purify_statement(stmt)?);
                    }
                    purified_elif.push((p_cond, p_body));
                }

                let purified_else = if let Some(else_body) = else_block {
                    let mut p_else = Vec::new();
                    for stmt in else_body {
                        p_else.push(self.purify_statement(stmt)?);
                    }
                    Some(p_else)
                } else {
                    None
                };

                Ok(BashStmt::If {
                    condition: purified_condition,
                    then_block: purified_then,
                    elif_blocks: purified_elif,
                    else_block: purified_else,
                    span: *span,
                })
            }

            BashStmt::While {
                condition,
                body,
                span,
            } => {
                let purified_condition = self.purify_expression(condition)?;
                let mut purified_body = Vec::new();
                for stmt in body {
                    purified_body.push(self.purify_statement(stmt)?);
                }

                Ok(BashStmt::While {
                    condition: purified_condition,
                    body: purified_body,
                    span: *span,
                })
            }

            BashStmt::Until {
                condition,
                body,
                span,
            } => {
                let purified_condition = self.purify_expression(condition)?;
                let mut purified_body = Vec::new();
                for stmt in body {
                    purified_body.push(self.purify_statement(stmt)?);
                }

                Ok(BashStmt::Until {
                    condition: purified_condition,
                    body: purified_body,
                    span: *span,
                })
            }

            BashStmt::For {
                variable,
                items,
                body,
                span,
            } => {
                let purified_items = self.purify_expression(items)?;
                let mut purified_body = Vec::new();
                for stmt in body {
                    purified_body.push(self.purify_statement(stmt)?);
                }

                Ok(BashStmt::For {
                    variable: variable.clone(),
                    items: purified_items,
                    body: purified_body,
                    span: *span,
                })
            }

            // Issue #68: Purify C-style for loop (already handled by codegen)
            BashStmt::ForCStyle {
                init,
                condition,
                increment,
                body,
                span,
            } => {
                // Purify the body statements
                let mut purified_body = Vec::new();
                for stmt in body {
                    purified_body.push(self.purify_statement(stmt)?);
                }

                // Return the purified C-style for loop as-is
                // The codegen will convert it to POSIX while loop
                Ok(BashStmt::ForCStyle {
                    init: init.clone(),
                    condition: condition.clone(),
                    increment: increment.clone(),
                    body: purified_body,
                    span: *span,
                })
            }

            BashStmt::Return { code, span } => {
                let purified_code = if let Some(expr) = code {
                    Some(self.purify_expression(expr)?)
                } else {
                    None
                };

                Ok(BashStmt::Return {
                    code: purified_code,
                    span: *span,
                })
            }

            BashStmt::Comment { .. } => Ok(stmt.clone()),

            BashStmt::Case { word, arms, span } => {
                let purified_word = self.purify_expression(word)?;

                let mut purified_arms = Vec::new();
                for arm in arms {
                    let mut purified_body = Vec::new();
                    for stmt in &arm.body {
                        purified_body.push(self.purify_statement(stmt)?);
                    }
                    purified_arms.push(crate::bash_parser::ast::CaseArm {
                        patterns: arm.patterns.clone(),
                        body: purified_body,
                    });
                }

                Ok(BashStmt::Case {
                    word: purified_word,
                    arms: purified_arms,
                    span: *span,
                })
            }

            BashStmt::Pipeline { commands, span } => {
                // Check if this is already a "permission check + mkdir" pipeline
                // If so, don't re-purify to avoid duplication (idempotency bug fix)
                if self.is_permission_check_mkdir_pipeline(commands) {
                    // Already purified - return as-is to maintain idempotency
                    return Ok(BashStmt::Pipeline {
                        commands: commands.clone(),
                        span: *span,
                    });
                }

                // Purify each command in the pipeline
                let mut purified_commands = Vec::new();
                for cmd in commands {
                    purified_commands.push(self.purify_statement(cmd)?);
                }

                Ok(BashStmt::Pipeline {
                    commands: purified_commands,
                    span: *span,
                })
            }

            BashStmt::AndList { left, right, span } => {
                // Purify both sides of the AND list
                let purified_left = self.purify_statement(left)?;
                let purified_right = self.purify_statement(right)?;

                Ok(BashStmt::AndList {
                    left: Box::new(purified_left),
                    right: Box::new(purified_right),
                    span: *span,
                })
            }

            BashStmt::OrList { left, right, span } => {
                // Purify both sides of the OR list
                let purified_left = self.purify_statement(left)?;
                let purified_right = self.purify_statement(right)?;

                Ok(BashStmt::OrList {
                    left: Box::new(purified_left),
                    right: Box::new(purified_right),
                    span: *span,
                })
            }

            BashStmt::BraceGroup { body, span } => {
                // Purify all statements in the brace group
                let mut purified_body = Vec::new();
                for stmt in body {
                    purified_body.push(self.purify_statement(stmt)?);
                }

                Ok(BashStmt::BraceGroup {
                    body: purified_body,
                    span: *span,
                })
            }
        }
    }

    fn purify_expression(&mut self, expr: &BashExpr) -> PurificationResult<BashExpr> {
        match expr {
            BashExpr::Variable(name) => {
                // Check for non-deterministic variables
                if self.non_deterministic_vars.contains(name) {
                    if self.options.remove_non_deterministic {
                        self.report
                            .determinism_fixes
                            .push(format!("Removed non-deterministic variable: ${}", name));
                        // Replace with a deterministic default
                        return Ok(BashExpr::Literal("0".to_string()));
                    } else if self.options.strict_idempotency {
                        return Err(PurificationError::NonDeterministicConstruct(format!(
                            "Variable ${} is non-deterministic",
                            name
                        )));
                    }
                }
                Ok(expr.clone())
            }

            BashExpr::CommandSubst(cmd) => {
                // Command substitutions can be non-deterministic
                self.report
                    .warnings
                    .push("Command substitution detected - may affect determinism".to_string());
                let purified_cmd = self.purify_statement(cmd)?;
                Ok(BashExpr::CommandSubst(Box::new(purified_cmd)))
            }

            BashExpr::Array(items) => {
                let mut purified_items = Vec::new();
                for item in items {
                    purified_items.push(self.purify_expression(item)?);
                }
                Ok(BashExpr::Array(purified_items))
            }

            BashExpr::Concat(parts) => {
                let mut purified_parts = Vec::new();
                for part in parts {
                    purified_parts.push(self.purify_expression(part)?);
                }
                Ok(BashExpr::Concat(purified_parts))
            }

            BashExpr::Test(test_expr) => {
                let purified_test = self.purify_test_expr(test_expr)?;
                Ok(BashExpr::Test(Box::new(purified_test)))
            }

            BashExpr::Arithmetic(arith) => {
                let purified_arith = self.purify_arithmetic(arith)?;
                Ok(BashExpr::Arithmetic(Box::new(purified_arith)))
            }

            // Literals and globs are deterministic
            BashExpr::Literal(_) | BashExpr::Glob(_) => Ok(expr.clone()),

            BashExpr::DefaultValue { variable, default } => {
                // Check variable for non-determinism
                if self.non_deterministic_vars.contains(variable) {
                    self.report.determinism_fixes.push(format!(
                        "Default value expansion uses non-deterministic variable: ${}",
                        variable
                    ));
                }
                // Purify the default value expression
                let purified_default = self.purify_expression(default)?;
                Ok(BashExpr::DefaultValue {
                    variable: variable.clone(),
                    default: Box::new(purified_default),
                })
            }

            BashExpr::AssignDefault { variable, default } => {
                // Check variable for non-determinism
                if self.non_deterministic_vars.contains(variable) {
                    self.report.determinism_fixes.push(format!(
                        "Assign default expansion uses non-deterministic variable: ${}",
                        variable
                    ));
                }
                // Purify the default value expression
                let purified_default = self.purify_expression(default)?;
                Ok(BashExpr::AssignDefault {
                    variable: variable.clone(),
                    default: Box::new(purified_default),
                })
            }

            BashExpr::ErrorIfUnset { variable, message } => {
                // Check variable for non-determinism
                if self.non_deterministic_vars.contains(variable) {
                    self.report.determinism_fixes.push(format!(
                        "Error-if-unset expansion uses non-deterministic variable: ${}",
                        variable
                    ));
                }
                // Purify the error message expression
                let purified_message = self.purify_expression(message)?;
                Ok(BashExpr::ErrorIfUnset {
                    variable: variable.clone(),
                    message: Box::new(purified_message),
                })
            }

            BashExpr::AlternativeValue {
                variable,
                alternative,
            } => {
                // Check variable for non-determinism
                if self.non_deterministic_vars.contains(variable) {
                    self.report.determinism_fixes.push(format!(
                        "Alternative value expansion uses non-deterministic variable: ${}",
                        variable
                    ));
                }
                // Purify the alternative value expression
                let purified_alternative = self.purify_expression(alternative)?;
                Ok(BashExpr::AlternativeValue {
                    variable: variable.clone(),
                    alternative: Box::new(purified_alternative),
                })
            }

            BashExpr::StringLength { variable } => {
                // Check variable for non-determinism
                // ${#VAR} gets the length of variable's value
                if self.non_deterministic_vars.contains(variable) {
                    self.report.determinism_fixes.push(format!(
                        "String length expansion uses non-deterministic variable: ${}",
                        variable
                    ));
                }
                Ok(BashExpr::StringLength {
                    variable: variable.clone(),
                })
            }

            BashExpr::RemoveSuffix { variable, pattern } => {
                // Check variable for non-determinism
                // ${VAR%pattern} removes shortest matching suffix
                if self.non_deterministic_vars.contains(variable) {
                    self.report.determinism_fixes.push(format!(
                        "Remove suffix expansion uses non-deterministic variable: ${}",
                        variable
                    ));
                }
                // Purify the pattern expression
                let purified_pattern = Box::new(self.purify_expression(pattern)?);
                Ok(BashExpr::RemoveSuffix {
                    variable: variable.clone(),
                    pattern: purified_pattern,
                })
            }

            BashExpr::RemovePrefix { variable, pattern } => {
                // Check variable for non-determinism
                // ${VAR#pattern} removes shortest matching prefix
                if self.non_deterministic_vars.contains(variable) {
                    self.report.determinism_fixes.push(format!(
                        "Remove prefix expansion uses non-deterministic variable: ${}",
                        variable
                    ));
                }
                // Purify the pattern expression
                let purified_pattern = Box::new(self.purify_expression(pattern)?);
                Ok(BashExpr::RemovePrefix {
                    variable: variable.clone(),
                    pattern: purified_pattern,
                })
            }

            BashExpr::RemoveLongestPrefix { variable, pattern } => {
                // Check variable for non-determinism
                // ${VAR##pattern} removes longest matching prefix (greedy)
                if self.non_deterministic_vars.contains(variable) {
                    self.report.determinism_fixes.push(format!(
                        "Remove longest prefix expansion uses non-deterministic variable: ${}",
                        variable
                    ));
                }
                // Purify the pattern expression
                let purified_pattern = Box::new(self.purify_expression(pattern)?);
                Ok(BashExpr::RemoveLongestPrefix {
                    variable: variable.clone(),
                    pattern: purified_pattern,
                })
            }

            BashExpr::RemoveLongestSuffix { variable, pattern } => {
                // Check variable for non-determinism
                // ${VAR%%pattern} removes longest matching suffix (greedy)
                if self.non_deterministic_vars.contains(variable) {
                    self.report.determinism_fixes.push(format!(
                        "Remove longest suffix expansion uses non-deterministic variable: ${}",
                        variable
                    ));
                }
                // Purify the pattern expression
                let purified_pattern = Box::new(self.purify_expression(pattern)?);
                Ok(BashExpr::RemoveLongestSuffix {
                    variable: variable.clone(),
                    pattern: purified_pattern,
                })
            }
        }
    }

    fn purify_test_expr(&mut self, test: &TestExpr) -> PurificationResult<TestExpr> {
        match test {
            TestExpr::StringEq(a, b)
            | TestExpr::StringNe(a, b)
            | TestExpr::IntEq(a, b)
            | TestExpr::IntNe(a, b)
            | TestExpr::IntLt(a, b)
            | TestExpr::IntLe(a, b)
            | TestExpr::IntGt(a, b)
            | TestExpr::IntGe(a, b) => {
                let purified_a = self.purify_expression(a)?;
                let purified_b = self.purify_expression(b)?;

                Ok(match test {
                    TestExpr::StringEq(_, _) => TestExpr::StringEq(purified_a, purified_b),
                    TestExpr::StringNe(_, _) => TestExpr::StringNe(purified_a, purified_b),
                    TestExpr::IntEq(_, _) => TestExpr::IntEq(purified_a, purified_b),
                    TestExpr::IntNe(_, _) => TestExpr::IntNe(purified_a, purified_b),
                    TestExpr::IntLt(_, _) => TestExpr::IntLt(purified_a, purified_b),
                    TestExpr::IntLe(_, _) => TestExpr::IntLe(purified_a, purified_b),
                    TestExpr::IntGt(_, _) => TestExpr::IntGt(purified_a, purified_b),
                    TestExpr::IntGe(_, _) => TestExpr::IntGe(purified_a, purified_b),
                    _ => unreachable!(),
                })
            }

            TestExpr::FileExists(p)
            | TestExpr::FileReadable(p)
            | TestExpr::FileWritable(p)
            | TestExpr::FileExecutable(p)
            | TestExpr::FileDirectory(p) => {
                let purified_p = self.purify_expression(p)?;

                Ok(match test {
                    TestExpr::FileExists(_) => TestExpr::FileExists(purified_p),
                    TestExpr::FileReadable(_) => TestExpr::FileReadable(purified_p),
                    TestExpr::FileWritable(_) => TestExpr::FileWritable(purified_p),
                    TestExpr::FileExecutable(_) => TestExpr::FileExecutable(purified_p),
                    TestExpr::FileDirectory(_) => TestExpr::FileDirectory(purified_p),
                    _ => unreachable!(),
                })
            }

            TestExpr::StringEmpty(s) | TestExpr::StringNonEmpty(s) => {
                let purified_s = self.purify_expression(s)?;

                Ok(match test {
                    TestExpr::StringEmpty(_) => TestExpr::StringEmpty(purified_s),
                    TestExpr::StringNonEmpty(_) => TestExpr::StringNonEmpty(purified_s),
                    _ => unreachable!(),
                })
            }

            TestExpr::And(a, b) | TestExpr::Or(a, b) => {
                let purified_a = self.purify_test_expr(a)?;
                let purified_b = self.purify_test_expr(b)?;

                Ok(match test {
                    TestExpr::And(_, _) => {
                        TestExpr::And(Box::new(purified_a), Box::new(purified_b))
                    }
                    TestExpr::Or(_, _) => TestExpr::Or(Box::new(purified_a), Box::new(purified_b)),
                    _ => unreachable!(),
                })
            }

            TestExpr::Not(t) => {
                let purified_t = self.purify_test_expr(t)?;
                Ok(TestExpr::Not(Box::new(purified_t)))
            }
        }
    }

    fn purify_arithmetic(&mut self, arith: &ArithExpr) -> PurificationResult<ArithExpr> {
        match arith {
            ArithExpr::Variable(name) => {
                if self.non_deterministic_vars.contains(name)
                    && self.options.remove_non_deterministic
                {
                    self.report.determinism_fixes.push(format!(
                        "Removed non-deterministic variable in arithmetic: {}",
                        name
                    ));
                    return Ok(ArithExpr::Number(0));
                }
                Ok(arith.clone())
            }

            ArithExpr::Add(a, b)
            | ArithExpr::Sub(a, b)
            | ArithExpr::Mul(a, b)
            | ArithExpr::Div(a, b)
            | ArithExpr::Mod(a, b) => {
                let purified_a = self.purify_arithmetic(a)?;
                let purified_b = self.purify_arithmetic(b)?;

                Ok(match arith {
                    ArithExpr::Add(_, _) => {
                        ArithExpr::Add(Box::new(purified_a), Box::new(purified_b))
                    }
                    ArithExpr::Sub(_, _) => {
                        ArithExpr::Sub(Box::new(purified_a), Box::new(purified_b))
                    }
                    ArithExpr::Mul(_, _) => {
                        ArithExpr::Mul(Box::new(purified_a), Box::new(purified_b))
                    }
                    ArithExpr::Div(_, _) => {
                        ArithExpr::Div(Box::new(purified_a), Box::new(purified_b))
                    }
                    ArithExpr::Mod(_, _) => {
                        ArithExpr::Mod(Box::new(purified_a), Box::new(purified_b))
                    }
                    _ => unreachable!(),
                })
            }

            ArithExpr::Number(_) => Ok(arith.clone()),
        }
    }

    /// Check if a pipeline is already a "permission check + mkdir" pattern
    ///
    /// Returns true if the pipeline contains:
    /// 1. An If statement with permission check (contains "Permission denied")
    /// 2. Followed by a mkdir -p command
    ///
    /// This prevents re-purification from adding duplicate permission checks.
    fn is_permission_check_mkdir_pipeline(&self, commands: &[BashStmt]) -> bool {
        if commands.len() != 2 {
            return false;
        }

        // First command should be If statement with permission check
        let has_permission_check = commands.first().is_some_and(|cmd| {
            matches!(cmd, BashStmt::If { else_block, .. } if {
                // Check if else block contains "Permission denied" error message
                else_block.as_ref().is_some_and(|stmts| {
                    stmts.iter().any(|stmt| {
                        matches!(stmt, BashStmt::Command { name, args, .. }
                            if name == "echo" && args.iter().any(|arg| {
                                matches!(arg, BashExpr::Literal(s) if s.contains("Permission denied"))
                            }))
                    })
                })
            })
        });

        // Second command should be mkdir -p
        let has_mkdir_p = commands.get(1).is_some_and(|cmd| {
            matches!(cmd, BashStmt::Command { name, args, .. }
            if name == "mkdir" && args.iter().any(|arg| {
                matches!(arg, BashExpr::Literal(s) if s.contains("-p"))
            }))
        });

        has_permission_check && has_mkdir_p
    }

    /// Generate a permission check for file operations
    ///
    /// Creates an If statement that checks if a parent directory is writable
    /// before attempting to create a subdirectory. If not writable, exits with error.
    fn generate_permission_check(
        &self,
        target_dir: &BashExpr,
        operation: &str,
        span: Span,
    ) -> BashStmt {
        let error_message = format!(
            "{}: Permission denied: {}",
            operation,
            match target_dir {
                BashExpr::Literal(s) => s.clone(),
                _ => "target".to_string(),
            }
        );

        BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::FileWritable(BashExpr::CommandSubst(
                Box::new(BashStmt::Command {
                    name: "dirname".to_string(),
                    args: vec![target_dir.clone()],
                    redirects: vec![],
                    span,
                }),
            )))),
            then_block: vec![], // Empty - continue if writable
            elif_blocks: vec![],
            else_block: Some(vec![
                BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal(error_message)],
                    redirects: vec![],
                    span,
                },
                BashStmt::Command {
                    name: "exit".to_string(),
                    args: vec![BashExpr::Literal("1".to_string())],
                    redirects: vec![],
                    span,
                },
            ]),
            span,
        }
    }

    fn make_command_idempotent(
        &mut self,
        name: &str,
        args: &[BashExpr],
        span: Span,
    ) -> PurificationResult<(Vec<BashStmt>, Option<String>)> {
        // Detect non-idempotent operations and suggest idempotent alternatives
        let fix_msg = match name {
            "echo" | "cat" | "ls" | "grep" => {
                // Read-only commands are already idempotent
                None
            }

            "mkdir" => {
                // mkdir should use -p flag for idempotency AND check permissions
                let purified_args: Result<Vec<_>, _> =
                    args.iter().map(|arg| self.purify_expression(arg)).collect();
                let purified_args = purified_args?;

                // Extract target directory (last argument)
                let target_dir = purified_args.last().ok_or_else(|| {
                    PurificationError::NonIdempotentSideEffect(
                        "mkdir requires a target directory".to_string(),
                    )
                })?;

                // Generate permission check: [ -w "$(dirname "$TARGET")" ] || { echo "error" >&2; exit 1; }
                let permission_check = self.generate_permission_check(target_dir, "mkdir", span);

                // Build mkdir -p command
                let mut mkdir_args = if !purified_args
                    .iter()
                    .any(|arg| matches!(arg, BashExpr::Literal(s) if s.contains("-p")))
                {
                    vec![BashExpr::Literal("-p".to_string())]
                } else {
                    vec![]
                };
                mkdir_args.extend(purified_args);

                let mkdir_cmd = BashStmt::Command {
                    name: name.to_string(),
                    args: mkdir_args,
                    redirects: vec![],
                    span,
                };

                return Ok((
                    vec![permission_check, mkdir_cmd],
                    Some(
                        "Added permission check and -p flag to mkdir for safety and idempotency"
                            .to_string(),
                    ),
                ));
            }

            "rm" => {
                // rm should use -f flag for idempotency
                if !args
                    .iter()
                    .any(|arg| matches!(arg, BashExpr::Literal(s) if s.contains("-f")))
                {
                    // Add -f flag for idempotency (like mkdir -p)
                    let purified_args: Result<Vec<_>, _> =
                        args.iter().map(|arg| self.purify_expression(arg)).collect();
                    let mut new_args = vec![BashExpr::Literal("-f".to_string())];
                    new_args.extend(purified_args?);

                    return Ok((
                        vec![BashStmt::Command {
                            name: name.to_string(),
                            args: new_args,
                            redirects: vec![],
                            span,
                        }],
                        Some("Added -f flag to rm for idempotency".to_string()),
                    ));
                } else {
                    None
                }
            }

            "cp" | "mv" => {
                // Copy/move operations may not be idempotent
                self.report.warnings.push(format!(
                    "Command '{}' may not be idempotent - consider checking if destination exists",
                    name
                ));
                None
            }

            _ => {
                // Track unknown commands as potential side effects
                if self.options.track_side_effects {
                    self.report
                        .side_effects_isolated
                        .push(format!("Side effect detected: command '{}'", name));
                }
                None
            }
        };

        let purified_args: Result<Vec<_>, _> =
            args.iter().map(|arg| self.purify_expression(arg)).collect();

        Ok((
            vec![BashStmt::Command {
                name: name.to_string(),
                args: purified_args?,
                redirects: vec![],
                span,
            }],
            fix_msg,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purify_removes_random_variable() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "value".to_string(),
                value: BashExpr::Variable("RANDOM".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        // RANDOM should be replaced with deterministic value
        assert_eq!(purified.statements.len(), 1);
        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => {
                assert!(matches!(value, BashExpr::Literal(_)));
            }
            _ => panic!("Expected assignment"),
        }

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_mkdir_idempotency_warning() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "mkdir".to_string(),
                args: vec![BashExpr::Literal("/tmp/test".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().idempotency_fixes.is_empty());
    }

    #[test]
    fn test_purify_preserves_deterministic_code() {
        let ast = BashAst {
            statements: vec![
                BashStmt::Assignment {
                    name: "FOO".to_string(),
                    value: BashExpr::Literal("bar".to_string()),
                    exported: false,
                    span: Span::dummy(),
                },
                BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("FOO".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
            ],
            metadata: AstMetadata {
                source_file: None,
                line_count: 2,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        // Deterministic code should be unchanged
        assert_eq!(purified.statements.len(), ast.statements.len());
        assert!(purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_PHASE2_001_permission_aware_mkdir() {
        // RED PHASE: This test should FAIL initially
        // Testing permission-aware purification (Toyota Way review §6.2)

        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "mkdir".to_string(),
                args: vec![BashExpr::Literal("/app/releases".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();

        // Should generate 1 Pipeline containing 2 statements:
        // 1. If statement with permission check
        // 2. mkdir -p command
        assert_eq!(
            purified.statements.len(),
            1,
            "Expected single Pipeline statement wrapping permission check + mkdir"
        );

        // The statement should be a Pipeline
        match &purified.statements[0] {
            BashStmt::Pipeline { commands, .. } => {
                assert_eq!(
                    commands.len(),
                    2,
                    "Expected 2 commands in pipeline: permission check + mkdir"
                );

                // First command should be If statement with permission check
                match &commands[0] {
                    BashStmt::If {
                        condition,
                        else_block,
                        ..
                    } => {
                        // Condition should test file writability
                        let condition_str = format!("{:?}", condition);
                        assert!(
                            condition_str.contains("FileWritable") || condition_str.contains("-w"),
                            "Expected FileWritable permission check, got: {}",
                            condition_str
                        );

                        // Should have else block with error handling
                        assert!(
                            else_block.is_some(),
                            "Expected else block with error handling"
                        );
                    }
                    other => panic!(
                        "Expected If statement for permission check, got: {:?}",
                        other
                    ),
                }

                // Second command should be mkdir -p
                match &commands[1] {
                    BashStmt::Command { name, args, .. } => {
                        assert_eq!(name, "mkdir", "Expected mkdir command");

                        // Should have -p flag
                        let has_p_flag = args
                            .iter()
                            .any(|arg| matches!(arg, BashExpr::Literal(s) if s.contains("-p")));
                        assert!(has_p_flag, "mkdir should have -p flag for idempotency");
                    }
                    other => panic!("Expected mkdir command, got: {:?}", other),
                }
            }
            other => panic!("Expected Pipeline statement, got: {:?}", other),
        }

        // Report should mention permission check injection
        assert!(
            !purifier.report().idempotency_fixes.is_empty(),
            "Should report idempotency fix"
        );
    }

    #[test]
    fn test_PHASE2_002_permission_aware_mkdir_integration() {
        // Integration test: Verify generated code is valid shell
        use crate::bash_parser::codegen::generate_purified_bash;

        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "mkdir".to_string(),
                args: vec![BashExpr::Literal("/opt/app".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();
        let generated_code = generate_purified_bash(&purified);

        // Generated code should contain permission check
        assert!(
            generated_code.contains("-w") || generated_code.contains("writable"),
            "Generated code should check write permission: {}",
            generated_code
        );

        // Generated code should contain error message for permission denied
        assert!(
            generated_code.contains("Permission denied")
                || generated_code.contains("permission denied"),
            "Generated code should have permission denied error: {}",
            generated_code
        );

        // Generated code should have mkdir -p
        assert!(
            generated_code.contains("mkdir") && generated_code.contains("-p"),
            "Generated code should have mkdir -p: {}",
            generated_code
        );
    }
}
