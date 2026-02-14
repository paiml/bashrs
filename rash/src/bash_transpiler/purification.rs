//! Purification Transforms for Bash Scripts
//!
//! Transforms bash scripts to ensure:
//! - Idempotency: Running multiple times produces same result
//! - Determinism: No random or time-based values
//! - Side-effect isolation: Clear tracking of mutations

use crate::bash_parser::ast::*;
use crate::bash_transpiler::type_check::{TypeChecker, TypeDiagnostic};
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

    /// Enable gradual type checking during purification
    pub type_check: bool,

    /// Emit runtime type guards in purified output
    pub emit_guards: bool,

    /// Treat type warnings as errors
    pub type_strict: bool,
}

impl Default for PurificationOptions {
    fn default() -> Self {
        Self {
            strict_idempotency: true,
            remove_non_deterministic: true,
            track_side_effects: true,
            type_check: false,
            emit_guards: false,
            type_strict: false,
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
    /// Type diagnostics collected during type checking
    pub type_diagnostics: Vec<TypeDiagnostic>,
}

impl PurificationReport {
    fn new() -> Self {
        Self {
            idempotency_fixes: Vec::new(),
            determinism_fixes: Vec::new(),
            side_effects_isolated: Vec::new(),
            warnings: Vec::new(),
            type_diagnostics: Vec::new(),
        }
    }
}

/// Purifies bash AST to ensure idempotency and determinism
pub struct Purifier {
    options: PurificationOptions,
    report: PurificationReport,
    non_deterministic_vars: HashSet<String>,
    /// Retained type checker for guard generation (avoids double-checking)
    type_checker: Option<TypeChecker>,
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
            type_checker: None,
        }
    }

    pub fn purify(&mut self, ast: &BashAst) -> PurificationResult<BashAst> {
        let mut purified_statements = Vec::new();

        for stmt in &ast.statements {
            let purified = self.purify_statement(stmt)?;
            purified_statements.push(purified);
        }

        let purified_ast = BashAst {
            statements: purified_statements,
            metadata: ast.metadata.clone(),
        };

        // Optional type checking phase
        if self.options.type_check || self.options.emit_guards {
            let mut checker = TypeChecker::new();
            let diagnostics = checker.check_ast(&purified_ast);
            self.report.type_diagnostics = diagnostics;
            self.type_checker = Some(checker);
        }

        Ok(purified_ast)
    }

    pub fn report(&self) -> &PurificationReport {
        &self.report
    }

    /// Get the type checker (if type checking was enabled)
    pub fn type_checker(&self) -> Option<&TypeChecker> {
        self.type_checker.as_ref()
    }

    fn purify_statement(&mut self, stmt: &BashStmt) -> PurificationResult<BashStmt> {
        match stmt {
            BashStmt::Assignment {
                name,
                index,
                value,
                exported,
                span,
            } => {
                // Check if value contains non-deterministic elements
                let purified_value = self.purify_expression(value)?;

                Ok(BashStmt::Assignment {
                    name: name.clone(),
                    index: index.clone(),
                    value: purified_value,
                    exported: *exported,
                    span: *span,
                })
            }

            BashStmt::Command {
                name,
                args,
                redirects,
                span,
            } => {
                // Detect and transform non-idempotent operations
                // Issue #72: Pass redirects through to preserve them
                let (purified_cmds, idempotent_wrapper) =
                    self.make_command_idempotent(name, args, redirects, *span)?;

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

            BashStmt::BraceGroup {
                body,
                subshell,
                span,
            } => {
                // Purify all statements in the brace group/subshell
                let mut purified_body = Vec::new();
                for stmt in body {
                    purified_body.push(self.purify_statement(stmt)?);
                }

                Ok(BashStmt::BraceGroup {
                    body: purified_body,
                    subshell: *subshell,
                    span: *span,
                })
            }

            BashStmt::Coproc { name, body, span } => {
                // Purify all statements in the coproc body
                let mut purified_body = Vec::new();
                for stmt in body {
                    purified_body.push(self.purify_statement(stmt)?);
                }

                Ok(BashStmt::Coproc {
                    name: name.clone(),
                    body: purified_body,
                    span: *span,
                })
            }
            BashStmt::Select {
                variable,
                items,
                body,
                span,
            } => {
                // F017: Purify select statement
                let purified_items = self.purify_expression(items)?;
                let mut purified_body = Vec::new();
                for stmt in body {
                    purified_body.push(self.purify_statement(stmt)?);
                }

                Ok(BashStmt::Select {
                    variable: variable.clone(),
                    items: purified_items,
                    body: purified_body,
                    span: *span,
                })
            }

            BashStmt::Negated { command, span } => {
                // Issue #133: Purify negated command
                let purified_cmd = self.purify_statement(command)?;
                Ok(BashStmt::Negated {
                    command: Box::new(purified_cmd),
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

            BashExpr::CommandCondition(cmd) => {
                // Issue #93: Purify command condition (command used as condition in if/while)
                let purified_cmd = self.purify_statement(cmd)?;
                Ok(BashExpr::CommandCondition(Box::new(purified_cmd)))
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

    fn make_command_idempotent(
        &mut self,
        name: &str,
        args: &[BashExpr],
        redirects: &[Redirect],
        span: Span,
    ) -> PurificationResult<(Vec<BashStmt>, Option<String>)> {
        // Detect non-idempotent operations and suggest idempotent alternatives
        let fix_msg = match name {
            "echo" | "cat" | "ls" | "grep" => {
                // Read-only commands are already idempotent
                None
            }

            "mkdir" => {
                // mkdir should use -p flag for idempotency
                let purified_args: Result<Vec<_>, _> =
                    args.iter().map(|arg| self.purify_expression(arg)).collect();
                let purified_args = purified_args?;

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

                return Ok((
                    vec![BashStmt::Command {
                        name: name.to_string(),
                        args: mkdir_args,
                        redirects: redirects.to_vec(),
                        span,
                    }],
                    Some("Added -p flag to mkdir for idempotency".to_string()),
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
                            redirects: redirects.to_vec(), // Issue #72: Preserve redirects
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
                redirects: redirects.to_vec(), // Issue #72: Preserve redirects
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
                index: None,
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
                    index: None,
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
    fn test_PHASE2_001_mkdir_gets_p_flag() {
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
        let purified = purifier.purify(&ast).expect("purification should succeed");

        // Should produce a single mkdir -p command
        assert_eq!(purified.statements.len(), 1);
        match &purified.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "mkdir");
                let has_p_flag = args
                    .iter()
                    .any(|arg| matches!(arg, BashExpr::Literal(s) if s == "-p"));
                assert!(has_p_flag, "mkdir should have -p flag: {args:?}");
            }
            other => panic!("Expected Command, got: {other:?}"),
        }

        assert!(
            !purifier.report().idempotency_fixes.is_empty(),
            "Should report idempotency fix"
        );
    }

    #[test]
    fn test_PHASE2_002_mkdir_p_integration() {
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
        let purified = purifier.purify(&ast).expect("purification should succeed");
        let generated_code = generate_purified_bash(&purified);

        // Generated code should have mkdir -p
        assert!(
            generated_code.contains("mkdir") && generated_code.contains("-p"),
            "Generated code should have mkdir -p: {}",
            generated_code
        );
    }

    // ============== PurificationOptions tests ==============

    #[test]
    fn test_purification_options_default() {
        let opts = PurificationOptions::default();
        assert!(opts.strict_idempotency);
        assert!(opts.remove_non_deterministic);
        assert!(opts.track_side_effects);
    }

    #[test]
    fn test_purification_options_clone() {
        let opts = PurificationOptions {
            strict_idempotency: false,
            remove_non_deterministic: true,
            track_side_effects: false,
            type_check: false,
            emit_guards: false,
            type_strict: false,
        };
        let cloned = opts.clone();
        assert!(!cloned.strict_idempotency);
        assert!(cloned.remove_non_deterministic);
        assert!(!cloned.track_side_effects);
    }

    #[test]
    fn test_purification_options_debug() {
        let opts = PurificationOptions::default();
        let debug_str = format!("{:?}", opts);
        assert!(debug_str.contains("strict_idempotency"));
        assert!(debug_str.contains("remove_non_deterministic"));
    }

    // ============== PurificationReport tests ==============

    #[test]
    fn test_purification_report_new() {
        let report = PurificationReport::new();
        assert!(report.idempotency_fixes.is_empty());
        assert!(report.determinism_fixes.is_empty());
        assert!(report.side_effects_isolated.is_empty());
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn test_purification_report_clone() {
        let mut report = PurificationReport::new();
        report.idempotency_fixes.push("fix1".to_string());
        report.warnings.push("warn1".to_string());
        let cloned = report.clone();
        assert_eq!(cloned.idempotency_fixes.len(), 1);
        assert_eq!(cloned.warnings.len(), 1);
    }

    #[test]
    fn test_purification_report_debug() {
        let report = PurificationReport::new();
        let debug_str = format!("{:?}", report);
        assert!(debug_str.contains("idempotency_fixes"));
    }

    // ============== PurificationError tests ==============

    #[test]
    fn test_purification_error_non_deterministic() {
        let err = PurificationError::NonDeterministicConstruct("$RANDOM".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("non-deterministic"));
        assert!(msg.contains("$RANDOM"));
    }

    #[test]
    fn test_purification_error_non_idempotent() {
        let err = PurificationError::NonIdempotentSideEffect("mkdir /tmp".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("idempotent"));
    }

    #[test]
    fn test_purification_error_debug() {
        let err = PurificationError::NonDeterministicConstruct("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NonDeterministicConstruct"));
    }

    // ============== Purifier non-deterministic variable tests ==============

    #[test]
    fn test_purify_removes_seconds_variable() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "time".to_string(),
                index: None,
                value: BashExpr::Variable("SECONDS".to_string()),
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

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => {
                assert!(matches!(value, BashExpr::Literal(s) if s == "0"));
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_purify_removes_bashpid_variable() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "pid".to_string(),
                index: None,
                value: BashExpr::Variable("BASHPID".to_string()),
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

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => {
                assert!(matches!(value, BashExpr::Literal(_)));
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_purify_removes_ppid_variable() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "parent".to_string(),
                index: None,
                value: BashExpr::Variable("PPID".to_string()),
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

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => {
                assert!(matches!(value, BashExpr::Literal(_)));
            }
            _ => panic!("Expected assignment"),
        }
    }

    // ============== Purifier strict mode tests ==============

    #[test]
    fn test_purify_strict_mode_rejects_random() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
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

        let opts = PurificationOptions {
            strict_idempotency: true,
            remove_non_deterministic: false,
            track_side_effects: false,
            type_check: false,
            emit_guards: false,
            type_strict: false,
        };

        let mut purifier = Purifier::new(opts);
        let result = purifier.purify(&ast);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            PurificationError::NonDeterministicConstruct(_)
        ));
    }

    // ============== Command purification tests ==============

    #[test]
    fn test_purify_rm_adds_force_flag() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "rm".to_string(),
                args: vec![BashExpr::Literal("/tmp/file".to_string())],
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

        match &purified.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "rm");
                assert!(args
                    .iter()
                    .any(|a| matches!(a, BashExpr::Literal(s) if s == "-f")));
            }
            _ => panic!("Expected command"),
        }

        assert!(!purifier.report().idempotency_fixes.is_empty());
    }

    #[test]
    fn test_purify_rm_keeps_existing_force_flag() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "rm".to_string(),
                args: vec![
                    BashExpr::Literal("-f".to_string()),
                    BashExpr::Literal("/tmp/file".to_string()),
                ],
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

        match &purified.statements[0] {
            BashStmt::Command { args, .. } => {
                // Should not have duplicate -f flags
                let f_count = args
                    .iter()
                    .filter(|a| matches!(a, BashExpr::Literal(s) if s == "-f"))
                    .count();
                assert_eq!(f_count, 1);
            }
            _ => panic!("Expected command"),
        }
    }

    #[test]
    fn test_purify_echo_unchanged() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("hello".to_string())],
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

        match &purified.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "echo");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected command"),
        }
    }

    #[test]
    fn test_purify_cp_generates_warning() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "cp".to_string(),
                args: vec![
                    BashExpr::Literal("src".to_string()),
                    BashExpr::Literal("dst".to_string()),
                ],
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

        assert!(!purifier.report().warnings.is_empty());
        assert!(purifier.report().warnings[0].contains("cp"));
    }

    #[test]
    fn test_purify_mv_generates_warning() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "mv".to_string(),
                args: vec![
                    BashExpr::Literal("src".to_string()),
                    BashExpr::Literal("dst".to_string()),
                ],
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

        assert!(!purifier.report().warnings.is_empty());
        assert!(purifier.report().warnings[0].contains("mv"));
    }

    #[test]
    fn test_purify_unknown_command_tracks_side_effect() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "custom_cmd".to_string(),
                args: vec![BashExpr::Literal("arg1".to_string())],
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

        assert!(!purifier.report().side_effects_isolated.is_empty());
    }

    // ============== Function purification tests ==============

    #[test]
    fn test_purify_function() {
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "my_func".to_string(),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("hello".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
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

        match &purified.statements[0] {
            BashStmt::Function { name, body, .. } => {
                assert_eq!(name, "my_func");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected function"),
        }
    }

    // ============== If statement purification tests ==============

    #[test]
    fn test_purify_if_statement() {
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Test(Box::new(TestExpr::StringNonEmpty(BashExpr::Variable(
                    "x".to_string(),
                )))),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("yes".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                elif_blocks: vec![],
                else_block: None,
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

        assert!(matches!(&purified.statements[0], BashStmt::If { .. }));
    }

    #[test]
    fn test_purify_if_with_elif_and_else() {
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Test(Box::new(TestExpr::IntEq(
                    BashExpr::Variable("x".to_string()),
                    BashExpr::Literal("1".to_string()),
                ))),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("one".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                elif_blocks: vec![(
                    BashExpr::Test(Box::new(TestExpr::IntEq(
                        BashExpr::Variable("x".to_string()),
                        BashExpr::Literal("2".to_string()),
                    ))),
                    vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("two".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    }],
                )],
                else_block: Some(vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("other".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }]),
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

        match &purified.statements[0] {
            BashStmt::If {
                elif_blocks,
                else_block,
                ..
            } => {
                assert_eq!(elif_blocks.len(), 1);
                assert!(else_block.is_some());
            }
            _ => panic!("Expected if statement"),
        }
    }

    // ============== Loop purification tests ==============

    #[test]
    fn test_purify_while_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::While {
                condition: BashExpr::Test(Box::new(TestExpr::IntLt(
                    BashExpr::Variable("i".to_string()),
                    BashExpr::Literal("10".to_string()),
                ))),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("i".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
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

        assert!(matches!(&purified.statements[0], BashStmt::While { .. }));
    }

    #[test]
    fn test_purify_until_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::Until {
                condition: BashExpr::Test(Box::new(TestExpr::IntGe(
                    BashExpr::Variable("i".to_string()),
                    BashExpr::Literal("10".to_string()),
                ))),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("i".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
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

        assert!(matches!(&purified.statements[0], BashStmt::Until { .. }));
    }

    #[test]
    fn test_purify_for_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::For {
                variable: "item".to_string(),
                items: BashExpr::Array(vec![
                    BashExpr::Literal("a".to_string()),
                    BashExpr::Literal("b".to_string()),
                ]),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("item".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
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

        assert!(matches!(&purified.statements[0], BashStmt::For { .. }));
    }

    #[test]
    fn test_purify_for_c_style_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::ForCStyle {
                init: "i=0".to_string(),
                condition: "i<10".to_string(),
                increment: "i++".to_string(),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("i".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
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

        assert!(matches!(
            &purified.statements[0],
            BashStmt::ForCStyle { .. }
        ));
    }

    // ============== Case statement purification tests ==============

    #[test]
    fn test_purify_case_statement() {
        let ast = BashAst {
            statements: vec![BashStmt::Case {
                word: BashExpr::Variable("x".to_string()),
                arms: vec![
                    CaseArm {
                        patterns: vec!["a".to_string()],
                        body: vec![BashStmt::Command {
                            name: "echo".to_string(),
                            args: vec![BashExpr::Literal("A".to_string())],
                            redirects: vec![],
                            span: Span::dummy(),
                        }],
                    },
                    CaseArm {
                        patterns: vec!["*".to_string()],
                        body: vec![BashStmt::Command {
                            name: "echo".to_string(),
                            args: vec![BashExpr::Literal("default".to_string())],
                            redirects: vec![],
                            span: Span::dummy(),
                        }],
                    },
                ],
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

        match &purified.statements[0] {
            BashStmt::Case { arms, .. } => {
                assert_eq!(arms.len(), 2);
            }
            _ => panic!("Expected case statement"),
        }
    }

    // ============== Return statement purification tests ==============

    #[test]
    fn test_purify_return_with_code() {
        let ast = BashAst {
            statements: vec![BashStmt::Return {
                code: Some(BashExpr::Literal("0".to_string())),
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

        match &purified.statements[0] {
            BashStmt::Return { code, .. } => {
                assert!(code.is_some());
            }
            _ => panic!("Expected return statement"),
        }
    }

    #[test]
    fn test_purify_return_without_code() {
        let ast = BashAst {
            statements: vec![BashStmt::Return {
                code: None,
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

        match &purified.statements[0] {
            BashStmt::Return { code, .. } => {
                assert!(code.is_none());
            }
            _ => panic!("Expected return statement"),
        }
    }

    // ============== Comment purification tests ==============

    #[test]
    fn test_purify_comment_unchanged() {
        let ast = BashAst {
            statements: vec![BashStmt::Comment {
                text: "This is a comment".to_string(),
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

        match &purified.statements[0] {
            BashStmt::Comment { text, .. } => {
                assert_eq!(text, "This is a comment");
            }
            _ => panic!("Expected comment"),
        }
    }

    // ============== Pipeline purification tests ==============

    #[test]
    fn test_purify_pipeline() {
        let ast = BashAst {
            statements: vec![BashStmt::Pipeline {
                commands: vec![
                    BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("hello".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                    BashStmt::Command {
                        name: "grep".to_string(),
                        args: vec![BashExpr::Literal("h".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                ],
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

        match &purified.statements[0] {
            BashStmt::Pipeline { commands, .. } => {
                assert_eq!(commands.len(), 2);
            }
            _ => panic!("Expected pipeline"),
        }
    }

    // ============== AndList/OrList purification tests ==============

    #[test]
    fn test_purify_and_list() {
        let ast = BashAst {
            statements: vec![BashStmt::AndList {
                left: Box::new(BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![
                        BashExpr::Literal("-f".to_string()),
                        BashExpr::Literal("file".to_string()),
                    ],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("exists".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
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

        assert!(matches!(&purified.statements[0], BashStmt::AndList { .. }));
    }

    #[test]
    fn test_purify_or_list() {
        let ast = BashAst {
            statements: vec![BashStmt::OrList {
                left: Box::new(BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![
                        BashExpr::Literal("-f".to_string()),
                        BashExpr::Literal("file".to_string()),
                    ],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("not found".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
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

        assert!(matches!(&purified.statements[0], BashStmt::OrList { .. }));
    }

    // ============== BraceGroup purification tests ==============

    #[test]
    fn test_purify_brace_group() {
        let ast = BashAst {
            statements: vec![BashStmt::BraceGroup {
                body: vec![
                    BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("one".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                    BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("two".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                ],
                subshell: false,
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

        match &purified.statements[0] {
            BashStmt::BraceGroup { body, .. } => {
                assert_eq!(body.len(), 2);
            }
            _ => panic!("Expected brace group"),
        }
    }

    // ============== Coproc purification tests ==============

    #[test]
    fn test_purify_coproc() {
        let ast = BashAst {
            statements: vec![BashStmt::Coproc {
                name: Some("mycoproc".to_string()),
                body: vec![BashStmt::Command {
                    name: "cat".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
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

        match &purified.statements[0] {
            BashStmt::Coproc { name, body, .. } => {
                assert_eq!(name.as_deref(), Some("mycoproc"));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected coproc"),
        }
    }

    // ============== Expression purification tests ==============

    #[test]
    fn test_purify_command_substitution() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "output".to_string(),
                index: None,
                value: BashExpr::CommandSubst(Box::new(BashStmt::Command {
                    name: "date".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                })),
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

        // Should generate a warning about command substitution
        assert!(!purifier.report().warnings.is_empty());
        assert!(purifier.report().warnings[0].contains("Command substitution"));
    }

    #[test]
    fn test_purify_array() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "arr".to_string(),
                index: None,
                value: BashExpr::Array(vec![
                    BashExpr::Literal("a".to_string()),
                    BashExpr::Variable("RANDOM".to_string()),
                ]),
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

        // RANDOM should be replaced
        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => match value {
                BashExpr::Array(items) => {
                    assert_eq!(items.len(), 2);
                    assert!(matches!(&items[1], BashExpr::Literal(_)));
                }
                _ => panic!("Expected array"),
            },
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_purify_concat() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::Concat(vec![
                    BashExpr::Literal("prefix_".to_string()),
                    BashExpr::Variable("RANDOM".to_string()),
                ]),
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

        // RANDOM in concat should be replaced
        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_literal_unchanged() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::Literal("hello".to_string()),
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

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => {
                assert!(matches!(value, BashExpr::Literal(s) if s == "hello"));
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_purify_glob_unchanged() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "files".to_string(),
                index: None,
                value: BashExpr::Glob("*.txt".to_string()),
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

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => {
                assert!(matches!(value, BashExpr::Glob(s) if s == "*.txt"));
            }
            _ => panic!("Expected assignment"),
        }
    }

    // ============== Default value expression tests ==============

    #[test]
    fn test_purify_default_value() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::DefaultValue {
                    variable: "FOO".to_string(),
                    default: Box::new(BashExpr::Literal("default".to_string())),
                },
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

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => {
                assert!(matches!(value, BashExpr::DefaultValue { .. }));
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_purify_default_value_with_non_deterministic_var() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::DefaultValue {
                    variable: "RANDOM".to_string(),
                    default: Box::new(BashExpr::Literal("0".to_string())),
                },
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
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_assign_default() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::AssignDefault {
                    variable: "RANDOM".to_string(),
                    default: Box::new(BashExpr::Literal("0".to_string())),
                },
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
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_error_if_unset() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::ErrorIfUnset {
                    variable: "RANDOM".to_string(),
                    message: Box::new(BashExpr::Literal("error".to_string())),
                },
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
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_alternative_value() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::AlternativeValue {
                    variable: "RANDOM".to_string(),
                    alternative: Box::new(BashExpr::Literal("alt".to_string())),
                },
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
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_string_length() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "len".to_string(),
                index: None,
                value: BashExpr::StringLength {
                    variable: "RANDOM".to_string(),
                },
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
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_remove_suffix() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::RemoveSuffix {
                    variable: "RANDOM".to_string(),
                    pattern: Box::new(BashExpr::Literal("*".to_string())),
                },
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
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_remove_prefix() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::RemovePrefix {
                    variable: "RANDOM".to_string(),
                    pattern: Box::new(BashExpr::Literal("*".to_string())),
                },
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
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_remove_longest_prefix() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::RemoveLongestPrefix {
                    variable: "RANDOM".to_string(),
                    pattern: Box::new(BashExpr::Literal("*".to_string())),
                },
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
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_remove_longest_suffix() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::RemoveLongestSuffix {
                    variable: "RANDOM".to_string(),
                    pattern: Box::new(BashExpr::Literal("*".to_string())),
                },
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
        let _ = purifier.purify(&ast).unwrap();

        assert!(!purifier.report().determinism_fixes.is_empty());
    }

    #[test]
    fn test_purify_command_condition() {
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::CommandCondition(Box::new(BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![
                        BashExpr::Literal("-f".to_string()),
                        BashExpr::Literal("file".to_string()),
                    ],
                    redirects: vec![],
                    span: Span::dummy(),
                })),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("ok".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                elif_blocks: vec![],
                else_block: None,
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

        assert!(matches!(&purified.statements[0], BashStmt::If { .. }));
    }

    // ============== Test expression purification tests ==============

    #[test]
    fn test_purify_test_all_comparison_types() {
        let tests = vec![
            TestExpr::StringEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("y".to_string()),
            ),
            TestExpr::StringNe(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("y".to_string()),
            ),
            TestExpr::IntEq(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            TestExpr::IntNe(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            TestExpr::IntLt(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            TestExpr::IntLe(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            TestExpr::IntGt(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            TestExpr::IntGe(
                BashExpr::Variable("x".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
        ];

        for test in tests {
            let ast = BashAst {
                statements: vec![BashStmt::If {
                    condition: BashExpr::Test(Box::new(test)),
                    then_block: vec![],
                    elif_blocks: vec![],
                    else_block: None,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let mut purifier = Purifier::new(PurificationOptions::default());
            let result = purifier.purify(&ast);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_purify_test_file_tests() {
        let tests = vec![
            TestExpr::FileExists(BashExpr::Literal("/tmp".to_string())),
            TestExpr::FileReadable(BashExpr::Literal("/tmp".to_string())),
            TestExpr::FileWritable(BashExpr::Literal("/tmp".to_string())),
            TestExpr::FileExecutable(BashExpr::Literal("/tmp".to_string())),
            TestExpr::FileDirectory(BashExpr::Literal("/tmp".to_string())),
        ];

        for test in tests {
            let ast = BashAst {
                statements: vec![BashStmt::If {
                    condition: BashExpr::Test(Box::new(test)),
                    then_block: vec![],
                    elif_blocks: vec![],
                    else_block: None,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let mut purifier = Purifier::new(PurificationOptions::default());
            let result = purifier.purify(&ast);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_purify_test_string_tests() {
        let tests = vec![
            TestExpr::StringEmpty(BashExpr::Variable("x".to_string())),
            TestExpr::StringNonEmpty(BashExpr::Variable("x".to_string())),
        ];

        for test in tests {
            let ast = BashAst {
                statements: vec![BashStmt::If {
                    condition: BashExpr::Test(Box::new(test)),
                    then_block: vec![],
                    elif_blocks: vec![],
                    else_block: None,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let mut purifier = Purifier::new(PurificationOptions::default());
            let result = purifier.purify(&ast);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_purify_test_logical_operators() {
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Test(Box::new(TestExpr::And(
                    Box::new(TestExpr::StringNonEmpty(BashExpr::Variable(
                        "x".to_string(),
                    ))),
                    Box::new(TestExpr::Or(
                        Box::new(TestExpr::IntGt(
                            BashExpr::Variable("y".to_string()),
                            BashExpr::Literal("0".to_string()),
                        )),
                        Box::new(TestExpr::Not(Box::new(TestExpr::FileExists(
                            BashExpr::Literal("/tmp".to_string()),
                        )))),
                    )),
                ))),
                then_block: vec![],
                elif_blocks: vec![],
                else_block: None,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut purifier = Purifier::new(PurificationOptions::default());
        let result = purifier.purify(&ast);
        assert!(result.is_ok());
    }

    // ============== Arithmetic purification tests ==============

    #[test]
    fn test_purify_arithmetic_all_operators() {
        let ops = vec![
            ArithExpr::Add(
                Box::new(ArithExpr::Number(1)),
                Box::new(ArithExpr::Number(2)),
            ),
            ArithExpr::Sub(
                Box::new(ArithExpr::Number(5)),
                Box::new(ArithExpr::Number(3)),
            ),
            ArithExpr::Mul(
                Box::new(ArithExpr::Number(2)),
                Box::new(ArithExpr::Number(3)),
            ),
            ArithExpr::Div(
                Box::new(ArithExpr::Number(6)),
                Box::new(ArithExpr::Number(2)),
            ),
            ArithExpr::Mod(
                Box::new(ArithExpr::Number(7)),
                Box::new(ArithExpr::Number(3)),
            ),
        ];

        for op in ops {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
                    index: None,
                    value: BashExpr::Arithmetic(Box::new(op)),
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
            let result = purifier.purify(&ast);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_purify_arithmetic_with_random_variable() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "result".to_string(),
                index: None,
                value: BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                    Box::new(ArithExpr::Variable("RANDOM".to_string())),
                    Box::new(ArithExpr::Number(1)),
                ))),
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

        // RANDOM should be replaced with 0
        assert!(!purifier.report().determinism_fixes.is_empty());

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => match value {
                BashExpr::Arithmetic(arith) => match arith.as_ref() {
                    ArithExpr::Add(left, _) => {
                        assert!(matches!(left.as_ref(), ArithExpr::Number(0)));
                    }
                    _ => panic!("Expected Add"),
                },
                _ => panic!("Expected Arithmetic"),
            },
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_purify_arithmetic_number_unchanged() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
                value: BashExpr::Arithmetic(Box::new(ArithExpr::Number(42))),
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

        match &purified.statements[0] {
            BashStmt::Assignment { value, .. } => match value {
                BashExpr::Arithmetic(arith) => {
                    assert!(matches!(arith.as_ref(), ArithExpr::Number(42)));
                }
                _ => panic!("Expected Arithmetic"),
            },
            _ => panic!("Expected assignment"),
        }
    }

    // ============== Report accessor test ==============

    #[test]
    fn test_purifier_report_accessor() {
        let mut purifier = Purifier::new(PurificationOptions::default());

        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "x".to_string(),
                index: None,
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

        let _ = purifier.purify(&ast).unwrap();

        let report = purifier.report();
        assert!(!report.determinism_fixes.is_empty());
    }

    // ============== Exported assignment test ==============

    #[test]
    fn test_purify_exported_assignment() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "PATH".to_string(),
                index: None,
                value: BashExpr::Literal("/usr/bin".to_string()),
                exported: true,
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

        match &purified.statements[0] {
            BashStmt::Assignment { exported, .. } => {
                assert!(*exported);
            }
            _ => panic!("Expected assignment"),
        }
    }

}
