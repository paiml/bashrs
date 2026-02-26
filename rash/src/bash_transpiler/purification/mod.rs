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

mod commands;
mod control_flow;
mod expressions;
mod test_exprs;

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests;

#[cfg(test)]
#[path = "golden_tests.rs"]
mod golden_tests;

#[cfg(test)]
#[path = "control_flow_coverage_tests.rs"]
mod control_flow_coverage_tests;

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
    pub(super) options: PurificationOptions,
    pub(super) report: PurificationReport,
    pub(super) non_deterministic_vars: HashSet<String>,
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

    pub(super) fn purify_statement(&mut self, stmt: &BashStmt) -> PurificationResult<BashStmt> {
        match stmt {
            BashStmt::Assignment {
                name,
                index,
                value,
                exported,
                span,
            } => {
                let purified_value = self.purify_expression(value)?;

                Ok(BashStmt::Assignment {
                    name: name.clone(),
                    index: index.clone(),
                    value: purified_value,
                    exported: *exported,
                    span: *span,
                })
            }

            BashStmt::Command { .. }
            | BashStmt::Pipeline { .. }
            | BashStmt::AndList { .. }
            | BashStmt::OrList { .. }
            | BashStmt::BraceGroup { .. }
            | BashStmt::Coproc { .. } => self.purify_command_stmt(stmt),

            BashStmt::Function { name, body, span } => {
                let purified_body = self.purify_body(body)?;

                Ok(BashStmt::Function {
                    name: name.clone(),
                    body: purified_body,
                    span: *span,
                })
            }

            BashStmt::If { .. }
            | BashStmt::While { .. }
            | BashStmt::Until { .. }
            | BashStmt::For { .. }
            | BashStmt::ForCStyle { .. }
            | BashStmt::Case { .. }
            | BashStmt::Select { .. } => self.purify_control_flow(stmt),

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

            BashStmt::Negated { command, span } => {
                let purified_cmd = self.purify_statement(command)?;
                Ok(BashStmt::Negated {
                    command: Box::new(purified_cmd),
                    span: *span,
                })
            }
        }
    }

    /// Purify a list of statements (shared helper for body blocks)
    pub(super) fn purify_body(&mut self, stmts: &[BashStmt]) -> PurificationResult<Vec<BashStmt>> {
        let mut purified = Vec::new();
        for stmt in stmts {
            purified.push(self.purify_statement(stmt)?);
        }
        Ok(purified)
    }
}
