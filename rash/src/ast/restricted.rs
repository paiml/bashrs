use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestrictedAst {
    pub functions: Vec<Function>,
    pub entry_point: String,
}

impl RestrictedAst {
    pub fn validate(&self) -> Result<(), String> {
        // Check for entry point
        if !self.functions.iter().any(|f| f.name == self.entry_point) {
            return Err(format!("Entry point function '{}' not found", self.entry_point));
        }
        
        // Validate each function
        for function in &self.functions {
            function.validate()?;
        }
        
        // Check for recursion
        self.check_no_recursion()?;
        
        Ok(())
    }
    
    fn check_no_recursion(&self) -> Result<(), String> {
        let mut call_graph: HashMap<String, Vec<String>> = HashMap::new();
        
        // Build call graph
        for function in &self.functions {
            let mut calls = Vec::new();
            function.collect_function_calls(&mut calls);
            call_graph.insert(function.name.clone(), calls);
        }
        
        // Detect cycles using DFS
        for function in &self.functions {
            let mut visited = std::collections::HashSet::new();
            let mut rec_stack = std::collections::HashSet::new();
            
            if self.has_cycle(&call_graph, &function.name, &mut visited, &mut rec_stack) {
                return Err(format!("Recursion detected involving function '{}'", function.name));
            }
        }
        
        Ok(())
    }
    
    #[allow(clippy::only_used_in_recursion)]
    fn has_cycle(
        &self,
        graph: &HashMap<String, Vec<String>>,
        node: &str,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        if rec_stack.contains(node) {
            return true;
        }
        
        if visited.contains(node) {
            return false;
        }
        
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        
        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if self.has_cycle(graph, neighbor, visited, rec_stack) {
                    return true;
                }
            }
        }
        
        rec_stack.remove(node);
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
}

impl Function {
    pub fn validate(&self) -> Result<(), String> {
        // Check that body is not empty
        if self.body.is_empty() {
            return Err(format!("Function '{}' has empty body", self.name));
        }
        
        // Validate all statements
        for stmt in &self.body {
            stmt.validate()?;
        }
        
        Ok(())
    }
    
    pub fn collect_function_calls(&self, calls: &mut Vec<String>) {
        for stmt in &self.body {
            stmt.collect_function_calls(calls);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    Bool,
    U32,
    Str,
    Result { ok_type: Box<Type>, err_type: Box<Type> },
    Option { inner_type: Box<Type> },
}

impl Type {
    pub fn is_allowed(&self) -> bool {
        match self {
            Type::Bool | Type::U32 | Type::Str => true,
            Type::Result { ok_type, err_type } => {
                ok_type.is_allowed() && err_type.is_allowed()
            }
            Type::Option { inner_type } => inner_type.is_allowed(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stmt {
    Let { name: String, value: Expr },
    Expr(Expr),
    Return(Option<Expr>),
    If { condition: Expr, then_block: Vec<Stmt>, else_block: Option<Vec<Stmt>> },
    Match { scrutinee: Expr, arms: Vec<MatchArm> },
    For { pattern: Pattern, iter: Expr, body: Vec<Stmt>, max_iterations: Option<u32> },
    While { condition: Expr, body: Vec<Stmt>, max_iterations: Option<u32> },
    Break,
    Continue,
}

impl Stmt {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Stmt::Let { value, .. } => value.validate(),
            Stmt::Expr(expr) => expr.validate(),
            Stmt::Return(Some(expr)) => expr.validate(),
            Stmt::Return(None) => Ok(()),
            Stmt::If { condition, then_block, else_block } => {
                condition.validate()?;
                for stmt in then_block {
                    stmt.validate()?;
                }
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        stmt.validate()?;
                    }
                }
                Ok(())
            }
            Stmt::Match { scrutinee, arms } => {
                scrutinee.validate()?;
                for arm in arms {
                    arm.pattern.validate()?;
                    if let Some(guard) = &arm.guard {
                        guard.validate()?;
                    }
                    for stmt in &arm.body {
                        stmt.validate()?;
                    }
                }
                Ok(())
            }
            Stmt::For { pattern, iter, body, max_iterations } => {
                // Enforce bounded iteration for verification
                if max_iterations.is_none() {
                    return Err("For loops must have bounded iterations for verification".to_string());
                }
                pattern.validate()?;
                iter.validate()?;
                for stmt in body {
                    stmt.validate()?;
                }
                Ok(())
            }
            Stmt::While { condition, body, max_iterations } => {
                // Enforce bounded iteration for verification
                if max_iterations.is_none() {
                    return Err("While loops must have bounded iterations for verification".to_string());
                }
                condition.validate()?;
                for stmt in body {
                    stmt.validate()?;
                }
                Ok(())
            }
            Stmt::Break | Stmt::Continue => Ok(()),
        }
    }
    
    pub fn collect_function_calls(&self, calls: &mut Vec<String>) {
        match self {
            Stmt::Let { value, .. } => value.collect_function_calls(calls),
            Stmt::Expr(expr) => expr.collect_function_calls(calls),
            Stmt::Return(Some(expr)) => expr.collect_function_calls(calls),
            Stmt::Return(None) => {}
            Stmt::If { condition, then_block, else_block } => {
                condition.collect_function_calls(calls);
                for stmt in then_block {
                    stmt.collect_function_calls(calls);
                }
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        stmt.collect_function_calls(calls);
                    }
                }
            }
            Stmt::Match { scrutinee, arms } => {
                scrutinee.collect_function_calls(calls);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        guard.collect_function_calls(calls);
                    }
                    for stmt in &arm.body {
                        stmt.collect_function_calls(calls);
                    }
                }
            }
            Stmt::For { iter, body, .. } => {
                iter.collect_function_calls(calls);
                for stmt in body {
                    stmt.collect_function_calls(calls);
                }
            }
            Stmt::While { condition, body, .. } => {
                condition.collect_function_calls(calls);
                for stmt in body {
                    stmt.collect_function_calls(calls);
                }
            }
            Stmt::Break | Stmt::Continue => {}
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Literal(Literal),
    Variable(String),
    FunctionCall { name: String, args: Vec<Expr> },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Unary { op: UnaryOp, operand: Box<Expr> },
    MethodCall { receiver: Box<Expr>, method: String, args: Vec<Expr> },
    Array(Vec<Expr>),
    Index { object: Box<Expr>, index: Box<Expr> },
    Try { expr: Box<Expr> },
    Block(Vec<Stmt>),
}

impl Expr {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Expr::Literal(_) | Expr::Variable(_) => Ok(()),
            Expr::FunctionCall { args, .. } => {
                for arg in args {
                    arg.validate()?;
                }
                Ok(())
            }
            Expr::Binary { left, right, .. } => {
                left.validate()?;
                right.validate()
            }
            Expr::Unary { operand, .. } => operand.validate(),
            Expr::MethodCall { receiver, args, .. } => {
                receiver.validate()?;
                for arg in args {
                    arg.validate()?;
                }
                Ok(())
            }
            // Placeholder for new expression types - TODO: implement properly
            _ => Ok(()), // Array, Index, Try, Block
        }
    }
    
    pub fn collect_function_calls(&self, calls: &mut Vec<String>) {
        match self {
            Expr::FunctionCall { name, args } => {
                calls.push(name.clone());
                for arg in args {
                    arg.collect_function_calls(calls);
                }
            }
            Expr::Binary { left, right, .. } => {
                left.collect_function_calls(calls);
                right.collect_function_calls(calls);
            }
            Expr::Unary { operand, .. } => {
                operand.collect_function_calls(calls);
            }
            Expr::MethodCall { receiver, args, .. } => {
                receiver.collect_function_calls(calls);
                for arg in args {
                    arg.collect_function_calls(calls);
                }
            }
            Expr::Array(elements) => {
                for element in elements {
                    element.collect_function_calls(calls);
                }
            }
            Expr::Index { object, index } => {
                object.collect_function_calls(calls);
                index.collect_function_calls(calls);
            }
            Expr::Try { expr } => {
                expr.collect_function_calls(calls);
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    stmt.collect_function_calls(calls);
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Literal {
    Bool(bool),
    U32(u32),
    Str(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    Literal(Literal),
    Variable(String),
    Wildcard,
    Tuple(Vec<Pattern>),
    Struct { name: String, fields: Vec<(String, Pattern)> },
}

impl Pattern {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Pattern::Literal(_) | Pattern::Variable(_) | Pattern::Wildcard => Ok(()),
            Pattern::Tuple(patterns) => {
                for pattern in patterns {
                    pattern.validate()?;
                }
                Ok(())
            }
            Pattern::Struct { fields, .. } => {
                for (_, pattern) in fields {
                    pattern.validate()?;
                }
                Ok(())
            }
        }
    }
    
    pub fn binds_variable(&self, name: &str) -> bool {
        match self {
            Pattern::Variable(var_name) => var_name == name,
            Pattern::Tuple(patterns) => patterns.iter().any(|p| p.binds_variable(name)),
            Pattern::Struct { fields, .. } => fields.iter().any(|(_, p)| p.binds_variable(name)),
            _ => false,
        }
    }
}