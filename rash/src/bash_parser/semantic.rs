//! Semantic Analysis for Bash AST
//!
//! Performs semantic analysis including:
//! - Variable scope resolution
//! - Command effect tracking
//! - Type inference (basic)

use super::ast::*;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SemanticError {
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Variable {0} used before assignment")]
    UseBeforeAssignment(String),

    #[error("Redefinition of function: {0}")]
    FunctionRedefinition(String),
}

pub type SemanticResult<T> = Result<T, SemanticError>;

/// Tracks variable scopes and their metadata
#[derive(Debug, Clone)]
pub struct ScopeInfo {
    pub variables: HashMap<String, VarInfo>,
    pub functions: HashMap<String, FunctionInfo>,
    pub parent: Option<Box<ScopeInfo>>,
}

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub name: String,
    pub exported: bool,
    pub assigned: bool,
    pub used: bool,
    pub inferred_type: InferredType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InferredType {
    String,
    Integer,
    Array,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub parameter_count: usize,
    pub calls_detected: HashSet<String>, // External commands called
}

/// Tracks side effects of commands
#[derive(Debug, Clone)]
pub struct EffectTracker {
    pub file_reads: HashSet<String>,
    pub file_writes: HashSet<String>,
    pub network_access: bool,
    pub process_spawns: HashSet<String>,
    pub env_modifications: HashSet<String>,
}

impl EffectTracker {
    pub fn new() -> Self {
        Self {
            file_reads: HashSet::new(),
            file_writes: HashSet::new(),
            network_access: false,
            process_spawns: HashSet::new(),
            env_modifications: HashSet::new(),
        }
    }

    pub fn is_pure(&self) -> bool {
        self.file_reads.is_empty()
            && self.file_writes.is_empty()
            && !self.network_access
            && self.process_spawns.is_empty()
            && self.env_modifications.is_empty()
    }
}

impl Default for EffectTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SemanticAnalyzer {
    global_scope: ScopeInfo,
    effects: EffectTracker,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            global_scope: ScopeInfo {
                variables: HashMap::new(),
                functions: HashMap::new(),
                parent: None,
            },
            effects: EffectTracker::new(),
        }
    }

    pub fn analyze(&mut self, ast: &BashAst) -> SemanticResult<AnalysisReport> {
        // Contract: parser-soundness-v1.yaml precondition (pv codegen)
        contract_pre_semantic_analyze!(ast);
        // Take ownership temporarily to avoid borrow checker issues
        let mut scope = std::mem::replace(
            &mut self.global_scope,
            ScopeInfo {
                variables: HashMap::new(),
                functions: HashMap::new(),
                parent: None,
            },
        );

        for stmt in &ast.statements {
            self.analyze_statement(stmt, &mut scope)?;
        }

        // Put the scope back
        self.global_scope = scope;

        Ok(AnalysisReport {
            scope_info: self.global_scope.clone(),
            effects: self.effects.clone(),
            warnings: vec![],
        })
    }

    fn analyze_statement(&mut self, stmt: &BashStmt, scope: &mut ScopeInfo) -> SemanticResult<()> {
        match stmt {
            BashStmt::Assignment {
                name,
                value,
                exported,
                ..
            } => self.analyze_assignment_stmt(name, value, *exported, scope),

            BashStmt::Command { name, args, .. } => self.analyze_command_stmt(name, args, scope),

            BashStmt::Function { name, body, .. } => self.analyze_function_def(name, body, scope),

            BashStmt::If {
                condition,
                then_block,
                elif_blocks,
                else_block,
                ..
            } => self.analyze_if_stmt(condition, then_block, elif_blocks, else_block, scope),

            BashStmt::While {
                condition, body, ..
            }
            | BashStmt::Until {
                condition, body, ..
            }
            | BashStmt::For {
                items: condition,
                body,
                ..
            } => {
                self.analyze_expression(condition, scope)?;
                self.analyze_body(body, scope)
            }

            // Issue #68: C-style for loop
            BashStmt::ForCStyle { body, .. } => self.analyze_body(body, scope),

            BashStmt::Return { code, .. } => {
                if let Some(expr) = code {
                    self.analyze_expression(expr, scope)?;
                }
                Ok(())
            }

            BashStmt::Comment { .. } => Ok(()),

            BashStmt::Case { word, arms, .. } => self.analyze_case_stmt(word, arms, scope),

            BashStmt::Pipeline { commands, .. } => self.analyze_body(commands, scope),

            BashStmt::AndList { left, right, .. } | BashStmt::OrList { left, right, .. } => {
                self.analyze_statement(left, scope)?;
                self.analyze_statement(right, scope)
            }

            BashStmt::BraceGroup { body, .. } | BashStmt::Coproc { body, .. } => {
                self.analyze_body(body, scope)
            }

            BashStmt::Select { variable, body, .. } => {
                self.analyze_select_stmt(variable, body, scope)
            }

            BashStmt::Negated { command, .. } => self.analyze_statement(command, scope),
        }
    }

    /// Analyze a variable assignment statement.
    fn analyze_assignment_stmt(
        &mut self,
        name: &str,
        value: &BashExpr,
        exported: bool,
        scope: &mut ScopeInfo,
    ) -> SemanticResult<()> {
        let inferred_type = self.infer_type(value);

        scope.variables.insert(
            name.to_string(),
            VarInfo {
                name: name.to_string(),
                exported,
                assigned: true,
                used: false,
                inferred_type,
            },
        );

        if exported {
            self.effects.env_modifications.insert(name.to_string());
        }

        self.analyze_expression(value, scope)
    }

    /// Analyze a command statement, tracking effects and spawns.
    fn analyze_command_stmt(
        &mut self,
        name: &str,
        args: &[BashExpr],
        scope: &mut ScopeInfo,
    ) -> SemanticResult<()> {
        self.track_command_effects(name);
        self.effects.process_spawns.insert(name.to_string());

        for arg in args {
            self.analyze_expression(arg, scope)?;
        }
        Ok(())
    }

    /// Analyze a function definition, creating a child scope.
    fn analyze_function_def(
        &mut self,
        name: &str,
        body: &[BashStmt],
        scope: &mut ScopeInfo,
    ) -> SemanticResult<()> {
        if scope.functions.contains_key(name) {
            return Err(SemanticError::FunctionRedefinition(name.to_string()));
        }

        let mut func_scope = ScopeInfo {
            variables: HashMap::new(),
            functions: HashMap::new(),
            parent: Some(Box::new(scope.clone())),
        };

        let mut calls = HashSet::new();
        for stmt in body {
            if let BashStmt::Command { name, .. } = stmt {
                calls.insert(name.clone());
            }
            self.analyze_statement(stmt, &mut func_scope)?;
        }

        scope.functions.insert(
            name.to_string(),
            FunctionInfo {
                name: name.to_string(),
                parameter_count: 0, // TODO: detect from $1, $2, etc.
                calls_detected: calls,
            },
        );
        Ok(())
    }

    /// Analyze an if/elif/else statement.
    fn analyze_if_stmt(
        &mut self,
        condition: &BashExpr,
        then_block: &[BashStmt],
        elif_blocks: &[(BashExpr, Vec<BashStmt>)],
        else_block: &Option<Vec<BashStmt>>,
        scope: &mut ScopeInfo,
    ) -> SemanticResult<()> {
        self.analyze_expression(condition, scope)?;
        self.analyze_body(then_block, scope)?;

        for (elif_cond, elif_body) in elif_blocks {
            self.analyze_expression(elif_cond, scope)?;
            self.analyze_body(elif_body, scope)?;
        }

        if let Some(else_body) = else_block {
            self.analyze_body(else_body, scope)?;
        }
        Ok(())
    }
}

include!("semantic_analyze_case_st.rs");
