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
            } => {
                let inferred_type = self.infer_type(value);

                scope.variables.insert(
                    name.clone(),
                    VarInfo {
                        name: name.clone(),
                        exported: *exported,
                        assigned: true,
                        used: false,
                        inferred_type,
                    },
                );

                if *exported {
                    self.effects.env_modifications.insert(name.clone());
                }

                self.analyze_expression(value, scope)?;
            }

            BashStmt::Command { name, args, .. } => {
                self.track_command_effects(name);
                self.effects.process_spawns.insert(name.clone());

                for arg in args {
                    self.analyze_expression(arg, scope)?;
                }
            }

            BashStmt::Function { name, body, .. } => {
                if scope.functions.contains_key(name) {
                    return Err(SemanticError::FunctionRedefinition(name.clone()));
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
                    name.clone(),
                    FunctionInfo {
                        name: name.clone(),
                        parameter_count: 0, // TODO: detect from $1, $2, etc.
                        calls_detected: calls,
                    },
                );
            }

            BashStmt::If {
                condition,
                then_block,
                elif_blocks,
                else_block,
                ..
            } => {
                self.analyze_expression(condition, scope)?;

                for stmt in then_block {
                    self.analyze_statement(stmt, scope)?;
                }

                for (elif_cond, elif_body) in elif_blocks {
                    self.analyze_expression(elif_cond, scope)?;
                    for stmt in elif_body {
                        self.analyze_statement(stmt, scope)?;
                    }
                }

                if let Some(else_body) = else_block {
                    for stmt in else_body {
                        self.analyze_statement(stmt, scope)?;
                    }
                }
            }

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
                for stmt in body {
                    self.analyze_statement(stmt, scope)?;
                }
            }

            BashStmt::Return { code, .. } => {
                if let Some(expr) = code {
                    self.analyze_expression(expr, scope)?;
                }
            }

            BashStmt::Comment { .. } => {
                // Comments don't affect semantics
            }
        }

        Ok(())
    }

    fn analyze_expression(&mut self, expr: &BashExpr, scope: &mut ScopeInfo) -> SemanticResult<()> {
        match expr {
            BashExpr::Variable(name) => {
                // Mark variable as used
                if let Some(var) = scope.variables.get_mut(name) {
                    var.used = true;
                }
                // Note: We don't error on undefined variables in bash
                // since they can come from environment
            }

            BashExpr::CommandSubst(cmd) => {
                self.analyze_statement(cmd, scope)?;
            }

            BashExpr::Array(items) | BashExpr::Concat(items) => {
                for item in items {
                    self.analyze_expression(item, scope)?;
                }
            }

            BashExpr::Test(test_expr) => {
                self.analyze_test_expr(test_expr, scope)?;
            }

            BashExpr::Literal(_) | BashExpr::Glob(_) => {
                // Literals have no semantic effects
            }

            BashExpr::Arithmetic(arith) => {
                self.analyze_arithmetic(arith, scope)?;
            }

            BashExpr::DefaultValue { variable, default } => {
                // Mark variable as used (even if it might be unset)
                if let Some(var) = scope.variables.get_mut(variable) {
                    var.used = true;
                }
                // Analyze the default value expression
                self.analyze_expression(default, scope)?;
            }

            BashExpr::AssignDefault { variable, default } => {
                // Mark variable as both used and assigned
                // ${VAR:=default} assigns to VAR if unset
                if let Some(var) = scope.variables.get_mut(variable) {
                    var.used = true;
                    var.assigned = true;
                } else {
                    // Variable doesn't exist yet, will be assigned
                    scope.variables.insert(
                        variable.clone(),
                        VarInfo {
                            name: variable.clone(),
                            exported: false,
                            assigned: true,
                            used: true,
                            inferred_type: InferredType::Unknown,
                        },
                    );
                }
                // Analyze the default value expression
                self.analyze_expression(default, scope)?;
            }

            BashExpr::ErrorIfUnset { variable, message } => {
                // Mark variable as used
                // ${VAR:?message} exits if VAR is unset
                if let Some(var) = scope.variables.get_mut(variable) {
                    var.used = true;
                }
                // Analyze the error message expression
                self.analyze_expression(message, scope)?;
            }

            BashExpr::AlternativeValue { variable, alternative } => {
                // Mark variable as used
                // ${VAR:+alt_value} uses alt_value if VAR is set
                if let Some(var) = scope.variables.get_mut(variable) {
                    var.used = true;
                }
                // Analyze the alternative value expression
                self.analyze_expression(alternative, scope)?;
            }

            BashExpr::StringLength { variable } => {
                // Mark variable as used
                // ${#VAR} gets the length of variable's value
                if let Some(var) = scope.variables.get_mut(variable) {
                    var.used = true;
                }
            }

            BashExpr::RemoveSuffix { variable, pattern } => {
                // Mark variable as used
                // ${VAR%pattern} removes shortest matching suffix
                if let Some(var) = scope.variables.get_mut(variable) {
                    var.used = true;
                }
                // Analyze the pattern expression
                self.analyze_expression(pattern, scope)?;
            }

            BashExpr::RemovePrefix { variable, pattern } => {
                // Mark variable as used
                // ${VAR#pattern} removes shortest matching prefix
                if let Some(var) = scope.variables.get_mut(variable) {
                    var.used = true;
                }
                // Analyze the pattern expression
                self.analyze_expression(pattern, scope)?;
            }

            BashExpr::RemoveLongestPrefix { variable, pattern } => {
                // Mark variable as used
                // ${VAR##pattern} removes longest matching prefix (greedy)
                if let Some(var) = scope.variables.get_mut(variable) {
                    var.used = true;
                }
                // Analyze the pattern expression
                self.analyze_expression(pattern, scope)?;
            }

            BashExpr::RemoveLongestSuffix { variable, pattern } => {
                // Mark variable as used
                // ${VAR%%pattern} removes longest matching suffix (greedy)
                if let Some(var) = scope.variables.get_mut(variable) {
                    var.used = true;
                }
                // Analyze the pattern expression
                self.analyze_expression(pattern, scope)?;
            }
        }

        Ok(())
    }

    fn analyze_test_expr(&mut self, test: &TestExpr, scope: &mut ScopeInfo) -> SemanticResult<()> {
        match test {
            TestExpr::StringEq(a, b)
            | TestExpr::StringNe(a, b)
            | TestExpr::IntEq(a, b)
            | TestExpr::IntNe(a, b)
            | TestExpr::IntLt(a, b)
            | TestExpr::IntLe(a, b)
            | TestExpr::IntGt(a, b)
            | TestExpr::IntGe(a, b) => {
                self.analyze_expression(a, scope)?;
                self.analyze_expression(b, scope)?;
            }

            TestExpr::FileExists(path)
            | TestExpr::FileReadable(path)
            | TestExpr::FileWritable(path)
            | TestExpr::FileExecutable(path)
            | TestExpr::FileDirectory(path) => {
                self.analyze_expression(path, scope)?;
                // File tests imply file reads
                if let BashExpr::Literal(p) = path {
                    self.effects.file_reads.insert(p.clone());
                }
            }

            TestExpr::StringEmpty(s) | TestExpr::StringNonEmpty(s) => {
                self.analyze_expression(s, scope)?;
            }

            TestExpr::And(a, b) | TestExpr::Or(a, b) => {
                self.analyze_test_expr(a, scope)?;
                self.analyze_test_expr(b, scope)?;
            }

            TestExpr::Not(t) => {
                self.analyze_test_expr(t, scope)?;
            }
        }

        Ok(())
    }

    fn analyze_arithmetic(
        &mut self,
        arith: &ArithExpr,
        scope: &mut ScopeInfo,
    ) -> SemanticResult<()> {
        match arith {
            ArithExpr::Variable(name) => {
                if let Some(var) = scope.variables.get_mut(name) {
                    var.used = true;
                }
            }
            ArithExpr::Add(a, b)
            | ArithExpr::Sub(a, b)
            | ArithExpr::Mul(a, b)
            | ArithExpr::Div(a, b)
            | ArithExpr::Mod(a, b) => {
                self.analyze_arithmetic(a, scope)?;
                self.analyze_arithmetic(b, scope)?;
            }
            ArithExpr::Number(_) => {}
        }
        Ok(())
    }

    fn infer_type(&self, expr: &BashExpr) -> InferredType {
        match expr {
            BashExpr::Literal(s) => {
                if s.parse::<i64>().is_ok() {
                    InferredType::Integer
                } else {
                    InferredType::String
                }
            }
            BashExpr::Array(_) => InferredType::Array,
            BashExpr::Arithmetic(_) => InferredType::Integer,
            _ => InferredType::Unknown,
        }
    }

    fn track_command_effects(&mut self, command: &str) {
        // Track known commands with side effects
        match command {
            "curl" | "wget" | "nc" | "telnet" | "ssh" => {
                self.effects.network_access = true;
            }
            "rm" | "mv" | "cp" | "touch" | "mkdir" | "rmdir" => {
                // File modification commands
                self.effects.file_writes.insert(command.to_string());
            }
            "cat" | "less" | "more" | "head" | "tail" | "grep" => {
                // File reading commands
                self.effects.file_reads.insert(command.to_string());
            }
            _ => {}
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub scope_info: ScopeInfo,
    pub effects: EffectTracker,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_tracking() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "FOO".to_string(),
                value: BashExpr::Literal("bar".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.variables.contains_key("FOO"));
    }

    #[test]
    fn test_effect_tracking() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "curl".to_string(),
                args: vec![BashExpr::Literal("http://example.com".to_string())],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.network_access);
    }
}
