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

            BashStmt::AndList { left, right, .. }
            | BashStmt::OrList { left, right, .. } => {
                self.analyze_statement(left, scope)?;
                self.analyze_statement(right, scope)
            }

            BashStmt::BraceGroup { body, .. }
            | BashStmt::Coproc { body, .. } => self.analyze_body(body, scope),

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

    /// Analyze a case statement with pattern arms.
    fn analyze_case_stmt(
        &mut self,
        word: &BashExpr,
        arms: &[CaseArm],
        scope: &mut ScopeInfo,
    ) -> SemanticResult<()> {
        self.analyze_expression(word, scope)?;

        for arm in arms {
            self.analyze_body(&arm.body, scope)?;
        }
        Ok(())
    }

    /// Analyze a select statement, registering the iteration variable.
    fn analyze_select_stmt(
        &mut self,
        variable: &str,
        body: &[BashStmt],
        scope: &mut ScopeInfo,
    ) -> SemanticResult<()> {
        // F017: Analyze select statement - variable is assigned in each iteration
        scope.variables.insert(
            variable.to_string(),
            VarInfo {
                name: variable.to_string(),
                exported: false,
                assigned: true,
                used: false,
                inferred_type: InferredType::String, // User selection is string
            },
        );
        self.analyze_body(body, scope)
    }

    /// Analyze a sequence of statements (loop body, block, etc.).
    fn analyze_body(&mut self, body: &[BashStmt], scope: &mut ScopeInfo) -> SemanticResult<()> {
        for stmt in body {
            self.analyze_statement(stmt, scope)?;
        }
        Ok(())
    }

    fn analyze_expression(&mut self, expr: &BashExpr, scope: &mut ScopeInfo) -> SemanticResult<()> {
        match expr {
            BashExpr::Variable(name) => {
                // Mark variable as used
                // Note: We don't error on undefined variables in bash
                // since they can come from environment
                Self::mark_var_used(scope, name);
            }

            BashExpr::CommandSubst(cmd)
            | BashExpr::CommandCondition(cmd) => {
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

            BashExpr::Literal(_) | BashExpr::Glob(_) => {}

            BashExpr::Arithmetic(arith) => {
                self.analyze_arithmetic(arith, scope)?;
            }

            BashExpr::DefaultValue { variable, default }
            | BashExpr::ErrorIfUnset { variable, message: default } => {
                Self::mark_var_used(scope, variable);
                self.analyze_expression(default, scope)?;
            }

            BashExpr::AlternativeValue {
                variable,
                alternative,
            } => {
                Self::mark_var_used(scope, variable);
                self.analyze_expression(alternative, scope)?;
            }

            BashExpr::AssignDefault { variable, default } => {
                self.analyze_assign_default(variable, default, scope)?;
            }

            BashExpr::StringLength { variable } => {
                Self::mark_var_used(scope, variable);
            }

            BashExpr::RemoveSuffix { variable, pattern }
            | BashExpr::RemovePrefix { variable, pattern }
            | BashExpr::RemoveLongestPrefix { variable, pattern }
            | BashExpr::RemoveLongestSuffix { variable, pattern } => {
                Self::mark_var_used(scope, variable);
                self.analyze_expression(pattern, scope)?;
            }
        }

        Ok(())
    }

    /// Mark a variable as used in the given scope.
    fn mark_var_used(scope: &mut ScopeInfo, name: &str) {
        if let Some(var) = scope.variables.get_mut(name) {
            var.used = true;
        }
    }

    /// Analyze ${VAR:=default} — assigns to VAR if unset.
    fn analyze_assign_default(
        &mut self,
        variable: &str,
        default: &BashExpr,
        scope: &mut ScopeInfo,
    ) -> SemanticResult<()> {
        if let Some(var) = scope.variables.get_mut(variable) {
            var.used = true;
            var.assigned = true;
        } else {
            scope.variables.insert(
                variable.to_string(),
                VarInfo {
                    name: variable.to_string(),
                    exported: false,
                    assigned: true,
                    used: true,
                    inferred_type: InferredType::Unknown,
                },
            );
        }
        self.analyze_expression(default, scope)
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

    fn make_ast(statements: Vec<BashStmt>) -> BashAst {
        BashAst {
            statements,
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        }
    }

    #[test]
    fn test_variable_tracking() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Assignment {
            name: "FOO".to_string(),
            index: None,
            value: BashExpr::Literal("bar".to_string()),
            exported: false,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.variables.contains_key("FOO"));
    }

    #[test]
    fn test_exported_variable_tracking() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Assignment {
            name: "PATH".to_string(),
            index: None,
            value: BashExpr::Literal("/usr/bin".to_string()),
            exported: true,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.variables.get("PATH").unwrap().exported);
        assert!(report.effects.env_modifications.contains("PATH"));
    }

    #[test]
    fn test_effect_tracking() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Command {
            name: "curl".to_string(),
            args: vec![BashExpr::Literal("http://example.com".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.network_access);
    }

    #[test]
    fn test_effect_tracker_is_pure() {
        let tracker = EffectTracker::new();
        assert!(tracker.is_pure());

        let mut impure = EffectTracker::new();
        impure.network_access = true;
        assert!(!impure.is_pure());
    }

    #[test]
    fn test_effect_tracker_default() {
        let tracker = EffectTracker::default();
        assert!(tracker.is_pure());
    }

    #[test]
    fn test_file_read_commands() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Command {
            name: "cat".to_string(),
            args: vec![BashExpr::Literal("file.txt".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_reads.contains("cat"));
    }

    #[test]
    fn test_file_write_commands() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Command {
            name: "rm".to_string(),
            args: vec![BashExpr::Literal("file.txt".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_writes.contains("rm"));
    }

    #[test]
    fn test_network_commands() {
        for cmd in &["wget", "nc", "telnet", "ssh"] {
            let mut analyzer = SemanticAnalyzer::new();
            let ast = make_ast(vec![BashStmt::Command {
                name: cmd.to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }]);

            let report = analyzer.analyze(&ast).unwrap();
            assert!(
                report.effects.network_access,
                "Command {} should enable network_access",
                cmd
            );
        }
    }

    #[test]
    fn test_if_statement() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::StringNonEmpty(BashExpr::Variable(
                "VAR".to_string(),
            )))),
            then_block: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("yes".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            elif_blocks: vec![(
                BashExpr::Test(Box::new(TestExpr::StringEmpty(BashExpr::Literal(
                    "".to_string(),
                )))),
                vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("elif".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
            )],
            else_block: Some(vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("no".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }]),
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("echo"));
    }

    #[test]
    fn test_while_loop() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::While {
            condition: BashExpr::Literal("true".to_string()),
            body: vec![BashStmt::Command {
                name: "sleep".to_string(),
                args: vec![BashExpr::Literal("1".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("sleep"));
    }

    #[test]
    fn test_until_loop() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Until {
            condition: BashExpr::Literal("false".to_string()),
            body: vec![BashStmt::Command {
                name: "wait".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("wait"));
    }

    #[test]
    fn test_for_loop() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::For {
            variable: "i".to_string(),
            items: BashExpr::Array(vec![
                BashExpr::Literal("1".to_string()),
                BashExpr::Literal("2".to_string()),
            ]),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("echo"));
    }

    #[test]
    fn test_for_cstyle() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::ForCStyle {
            init: "i=0".to_string(),
            condition: "i<10".to_string(),
            increment: "i++".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("loop".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("echo"));
    }

    #[test]
    fn test_case_statement() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Case {
            word: BashExpr::Variable("opt".to_string()),
            arms: vec![CaseArm {
                patterns: vec!["a".to_string()],
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("option a".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("echo"));
    }

    #[test]
    fn test_pipeline() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Pipeline {
            commands: vec![
                BashStmt::Command {
                    name: "cat".to_string(),
                    args: vec![BashExpr::Literal("file".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
                BashStmt::Command {
                    name: "grep".to_string(),
                    args: vec![BashExpr::Literal("pattern".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                },
            ],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_reads.contains("cat"));
        assert!(report.effects.file_reads.contains("grep"));
    }

    #[test]
    fn test_and_list() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::AndList {
            left: Box::new(BashStmt::Command {
                name: "test".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }),
            right: Box::new(BashStmt::Command {
                name: "echo".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("test"));
        assert!(report.effects.process_spawns.contains("echo"));
    }

    #[test]
    fn test_or_list() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::OrList {
            left: Box::new(BashStmt::Command {
                name: "false".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }),
            right: Box::new(BashStmt::Command {
                name: "true".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }),
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("false"));
        assert!(report.effects.process_spawns.contains("true"));
    }

    #[test]
    fn test_brace_group() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::BraceGroup {
            body: vec![BashStmt::Command {
                name: "pwd".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            subshell: false,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("pwd"));
    }

    #[test]
    fn test_coproc() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Coproc {
            name: Some("mycoproc".to_string()),
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_reads.contains("cat"));
    }

    #[test]
    fn test_function_definition() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Function {
            name: "myfunc".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("hello".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.functions.contains_key("myfunc"));
        let func = report.scope_info.functions.get("myfunc").unwrap();
        assert!(func.calls_detected.contains("echo"));
    }

    #[test]
    fn test_function_redefinition_error() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Function {
                name: "myfunc".to_string(),
                body: vec![],
                span: Span::dummy(),
            },
            BashStmt::Function {
                name: "myfunc".to_string(),
                body: vec![],
                span: Span::dummy(),
            },
        ]);

        let result = analyzer.analyze(&ast);
        assert!(matches!(
            result,
            Err(SemanticError::FunctionRedefinition(_))
        ));
    }

    #[test]
    fn test_return_statement() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Return {
            code: Some(BashExpr::Literal("0".to_string())),
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.is_pure());
    }

    #[test]
    fn test_return_without_code() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Return {
            code: None,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.is_pure());
    }

    #[test]
    fn test_comment() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Comment {
            text: "# This is a comment".to_string(),
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.is_pure());
    }

    #[test]
    fn test_command_substitution() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Assignment {
            name: "OUT".to_string(),
            index: None,
            value: BashExpr::CommandSubst(Box::new(BashStmt::Command {
                name: "date".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            })),
            exported: false,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("date"));
    }

    #[test]
    fn test_concat_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Assignment {
            name: "X".to_string(),
            index: None,
            value: BashExpr::Concat(vec![
                BashExpr::Literal("a".to_string()),
                BashExpr::Variable("B".to_string()),
            ]),
            exported: false,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.variables.contains_key("X"));
    }

    #[test]
    fn test_default_value_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("set".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::DefaultValue {
                    variable: "VAR".to_string(),
                    default: Box::new(BashExpr::Literal("default".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let var = report.scope_info.variables.get("VAR").unwrap();
        assert!(var.used);
    }

    #[test]
    fn test_assign_default_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::AssignDefault {
                variable: "NEWVAR".to_string(),
                default: Box::new(BashExpr::Literal("value".to_string())),
            }],
            redirects: vec![],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.variables.contains_key("NEWVAR"));
        let var = report.scope_info.variables.get("NEWVAR").unwrap();
        assert!(var.assigned);
        assert!(var.used);
    }

    #[test]
    fn test_assign_default_existing_var() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("original".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::AssignDefault {
                    variable: "VAR".to_string(),
                    default: Box::new(BashExpr::Literal("new".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let var = report.scope_info.variables.get("VAR").unwrap();
        assert!(var.used);
    }

    #[test]
    fn test_error_if_unset_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("set".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::ErrorIfUnset {
                    variable: "VAR".to_string(),
                    message: Box::new(BashExpr::Literal("VAR is unset".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let var = report.scope_info.variables.get("VAR").unwrap();
        assert!(var.used);
    }

    #[test]
    fn test_alternative_value_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("set".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::AlternativeValue {
                    variable: "VAR".to_string(),
                    alternative: Box::new(BashExpr::Literal("alt".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let var = report.scope_info.variables.get("VAR").unwrap();
        assert!(var.used);
    }

    #[test]
    fn test_string_length_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "STR".to_string(),
                index: None,
                value: BashExpr::Literal("hello".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::StringLength {
                    variable: "STR".to_string(),
                }],
                redirects: vec![],
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let var = report.scope_info.variables.get("STR").unwrap();
        assert!(var.used);
    }

    #[test]
    fn test_remove_suffix_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "FILE".to_string(),
                index: None,
                value: BashExpr::Literal("test.txt".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemoveSuffix {
                    variable: "FILE".to_string(),
                    pattern: Box::new(BashExpr::Literal(".txt".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let var = report.scope_info.variables.get("FILE").unwrap();
        assert!(var.used);
    }

    #[test]
    fn test_remove_prefix_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "PATH".to_string(),
                index: None,
                value: BashExpr::Literal("/usr/local/bin".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemovePrefix {
                    variable: "PATH".to_string(),
                    pattern: Box::new(BashExpr::Literal("/usr/".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let var = report.scope_info.variables.get("PATH").unwrap();
        assert!(var.used);
    }

    #[test]
    fn test_remove_longest_prefix() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("aaa/bbb/ccc".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemoveLongestPrefix {
                    variable: "VAR".to_string(),
                    pattern: Box::new(BashExpr::Literal("*/".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let var = report.scope_info.variables.get("VAR").unwrap();
        assert!(var.used);
    }

    #[test]
    fn test_remove_longest_suffix() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "VAR".to_string(),
                index: None,
                value: BashExpr::Literal("aaa/bbb/ccc".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::RemoveLongestSuffix {
                    variable: "VAR".to_string(),
                    pattern: Box::new(BashExpr::Literal("/*".to_string())),
                }],
                redirects: vec![],
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let var = report.scope_info.variables.get("VAR").unwrap();
        assert!(var.used);
    }

    #[test]
    fn test_command_condition() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::If {
            condition: BashExpr::CommandCondition(Box::new(BashStmt::Command {
                name: "grep".to_string(),
                args: vec![
                    BashExpr::Literal("-q".to_string()),
                    BashExpr::Literal("pattern".to_string()),
                    BashExpr::Literal("file".to_string()),
                ],
                redirects: vec![],
                span: Span::dummy(),
            })),
            then_block: vec![],
            elif_blocks: vec![],
            else_block: None,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_reads.contains("grep"));
    }

    #[test]
    fn test_glob_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Command {
            name: "ls".to_string(),
            args: vec![BashExpr::Glob("*.txt".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.process_spawns.contains("ls"));
    }

    #[test]
    fn test_arithmetic_expression() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![
            BashStmt::Assignment {
                name: "X".to_string(),
                index: None,
                value: BashExpr::Literal("5".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Assignment {
                name: "Y".to_string(),
                index: None,
                value: BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                    Box::new(ArithExpr::Variable("X".to_string())),
                    Box::new(ArithExpr::Number(10)),
                ))),
                exported: false,
                span: Span::dummy(),
            },
        ]);

        let report = analyzer.analyze(&ast).unwrap();
        let x = report.scope_info.variables.get("X").unwrap();
        assert!(x.used);
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::Assignment {
            name: "RESULT".to_string(),
            index: None,
            value: BashExpr::Arithmetic(Box::new(ArithExpr::Mod(
                Box::new(ArithExpr::Div(
                    Box::new(ArithExpr::Mul(
                        Box::new(ArithExpr::Sub(
                            Box::new(ArithExpr::Number(10)),
                            Box::new(ArithExpr::Number(2)),
                        )),
                        Box::new(ArithExpr::Number(3)),
                    )),
                    Box::new(ArithExpr::Number(4)),
                )),
                Box::new(ArithExpr::Number(5)),
            ))),
            exported: false,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.scope_info.variables.contains_key("RESULT"));
    }

    #[test]
    fn test_test_expressions_comparison() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::And(
                Box::new(TestExpr::StringEq(
                    BashExpr::Literal("a".to_string()),
                    BashExpr::Literal("a".to_string()),
                )),
                Box::new(TestExpr::Or(
                    Box::new(TestExpr::StringNe(
                        BashExpr::Literal("x".to_string()),
                        BashExpr::Literal("y".to_string()),
                    )),
                    Box::new(TestExpr::Not(Box::new(TestExpr::StringEmpty(
                        BashExpr::Literal("test".to_string()),
                    )))),
                )),
            ))),
            then_block: vec![],
            elif_blocks: vec![],
            else_block: None,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.is_pure());
    }

    #[test]
    fn test_test_expressions_integer() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::And(
                Box::new(TestExpr::IntEq(
                    BashExpr::Literal("1".to_string()),
                    BashExpr::Literal("1".to_string()),
                )),
                Box::new(TestExpr::And(
                    Box::new(TestExpr::IntNe(
                        BashExpr::Literal("1".to_string()),
                        BashExpr::Literal("2".to_string()),
                    )),
                    Box::new(TestExpr::And(
                        Box::new(TestExpr::IntLt(
                            BashExpr::Literal("1".to_string()),
                            BashExpr::Literal("2".to_string()),
                        )),
                        Box::new(TestExpr::And(
                            Box::new(TestExpr::IntLe(
                                BashExpr::Literal("1".to_string()),
                                BashExpr::Literal("1".to_string()),
                            )),
                            Box::new(TestExpr::And(
                                Box::new(TestExpr::IntGt(
                                    BashExpr::Literal("2".to_string()),
                                    BashExpr::Literal("1".to_string()),
                                )),
                                Box::new(TestExpr::IntGe(
                                    BashExpr::Literal("2".to_string()),
                                    BashExpr::Literal("2".to_string()),
                                )),
                            )),
                        )),
                    )),
                )),
            ))),
            then_block: vec![],
            elif_blocks: vec![],
            else_block: None,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.is_pure());
    }

    #[test]
    fn test_test_expressions_file() {
        let mut analyzer = SemanticAnalyzer::new();
        let ast = make_ast(vec![BashStmt::If {
            condition: BashExpr::Test(Box::new(TestExpr::And(
                Box::new(TestExpr::FileExists(BashExpr::Literal("/tmp".to_string()))),
                Box::new(TestExpr::And(
                    Box::new(TestExpr::FileReadable(BashExpr::Literal(
                        "/tmp".to_string(),
                    ))),
                    Box::new(TestExpr::And(
                        Box::new(TestExpr::FileWritable(BashExpr::Literal(
                            "/tmp".to_string(),
                        ))),
                        Box::new(TestExpr::And(
                            Box::new(TestExpr::FileExecutable(BashExpr::Literal(
                                "/tmp".to_string(),
                            ))),
                            Box::new(TestExpr::FileDirectory(BashExpr::Literal(
                                "/tmp".to_string(),
                            ))),
                        )),
                    )),
                )),
            ))),
            then_block: vec![],
            elif_blocks: vec![],
            else_block: None,
            span: Span::dummy(),
        }]);

        let report = analyzer.analyze(&ast).unwrap();
        assert!(report.effects.file_reads.contains("/tmp"));
    }

    #[test]
    fn test_infer_type_integer() {
        let analyzer = SemanticAnalyzer::new();
        let expr = BashExpr::Literal("42".to_string());
        assert_eq!(analyzer.infer_type(&expr), InferredType::Integer);
    }

    #[test]
    fn test_infer_type_string() {
        let analyzer = SemanticAnalyzer::new();
        let expr = BashExpr::Literal("hello".to_string());
        assert_eq!(analyzer.infer_type(&expr), InferredType::String);
    }

    #[test]
    fn test_infer_type_array() {
        let analyzer = SemanticAnalyzer::new();
        let expr = BashExpr::Array(vec![BashExpr::Literal("a".to_string())]);
        assert_eq!(analyzer.infer_type(&expr), InferredType::Array);
    }

    #[test]
    fn test_infer_type_arithmetic() {
        let analyzer = SemanticAnalyzer::new();
        let expr = BashExpr::Arithmetic(Box::new(ArithExpr::Number(5)));
        assert_eq!(analyzer.infer_type(&expr), InferredType::Integer);
    }

    #[test]
    fn test_infer_type_unknown() {
        let analyzer = SemanticAnalyzer::new();
        let expr = BashExpr::Variable("X".to_string());
        assert_eq!(analyzer.infer_type(&expr), InferredType::Unknown);
    }

    #[test]
    fn test_semantic_analyzer_default() {
        let analyzer = SemanticAnalyzer::default();
        assert!(analyzer.global_scope.variables.is_empty());
    }

    #[test]
    fn test_var_info_fields() {
        let var = VarInfo {
            name: "TEST".to_string(),
            exported: true,
            assigned: true,
            used: false,
            inferred_type: InferredType::String,
        };
        assert_eq!(var.name, "TEST");
        assert!(var.exported);
        assert!(var.assigned);
        assert!(!var.used);
        assert_eq!(var.inferred_type, InferredType::String);
    }

    #[test]
    fn test_function_info_fields() {
        let mut calls = HashSet::new();
        calls.insert("echo".to_string());
        let func = FunctionInfo {
            name: "myfunc".to_string(),
            parameter_count: 2,
            calls_detected: calls,
        };
        assert_eq!(func.name, "myfunc");
        assert_eq!(func.parameter_count, 2);
        assert!(func.calls_detected.contains("echo"));
    }

    #[test]
    fn test_scope_info_with_parent() {
        let parent = ScopeInfo {
            variables: HashMap::new(),
            functions: HashMap::new(),
            parent: None,
        };
        let child = ScopeInfo {
            variables: HashMap::new(),
            functions: HashMap::new(),
            parent: Some(Box::new(parent)),
        };
        assert!(child.parent.is_some());
    }
}
