use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Restricted AST for transpilable Rust subset
///
/// Represents a validated Rust program that can be safely transpiled to shell script.
/// Only supports a restricted subset of Rust features for shell compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestrictedAst {
    /// List of functions in the program
    pub functions: Vec<Function>,
    /// Name of the entry point function (typically "main")
    pub entry_point: String,
}

impl RestrictedAst {
    pub fn validate(&self) -> Result<(), String> {
        // Check for entry point
        if !self.functions.iter().any(|f| f.name == self.entry_point) {
            return Err(format!(
                "Entry point function '{}' not found",
                self.entry_point
            ));
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
                return Err(format!(
                    "Recursion detected involving function '{}'",
                    function.name
                ));
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

/// Function declaration in restricted AST
///
/// Represents a function with parameters, return type, and body statements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    /// Function name
    pub name: String,
    /// Function parameters
    pub params: Vec<Parameter>,
    /// Return type
    pub return_type: Type,
    /// Function body statements
    pub body: Vec<Stmt>,
}

impl Function {
    pub fn validate(&self) -> Result<(), String> {
        // Validate function name
        Self::validate_identifier(&self.name)?;

        // Validate parameter names and check for duplicates
        let mut param_names = std::collections::HashSet::new();
        for param in &self.params {
            Self::validate_identifier(&param.name)?;
            if !param_names.insert(&param.name) {
                return Err(format!("Duplicate parameter name: {}", param.name));
            }
        }

        // Empty body is OK for functions

        // Validate all statements
        for stmt in &self.body {
            stmt.validate()?;
        }

        Ok(())
    }

    fn validate_identifier(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Identifiers cannot be empty".to_string());
        }
        if name.contains('\0') {
            return Err("Null characters not allowed in identifiers".to_string());
        }
        // Check for shell-unsafe characters
        if name.contains('$') || name.contains('`') || name.contains('\\') {
            return Err(format!("Unsafe characters in identifier: {}", name));
        }
        Ok(())
    }

    pub fn collect_function_calls(&self, calls: &mut Vec<String>) {
        for stmt in &self.body {
            stmt.collect_function_calls(calls);
        }
    }
}

/// Function parameter in restricted AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: Type,
}

/// Type system for restricted AST
///
/// Supports basic types that can be mapped to shell script equivalents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Type {
    /// Void type (no return value)
    Void,
    /// Boolean type
    Bool,
    /// 32-bit unsigned integer
    U32,
    /// String type
    Str,
    /// Result type with ok and error variants
    Result {
        /// Ok variant type
        ok_type: Box<Type>,
        /// Error variant type
        err_type: Box<Type>,
    },
    /// Optional type
    Option {
        /// Inner type
        inner_type: Box<Type>,
    },
}

impl Type {
    pub fn is_allowed(&self) -> bool {
        match self {
            Type::Void | Type::Bool | Type::U32 | Type::Str => true,
            Type::Result { ok_type, err_type } => ok_type.is_allowed() && err_type.is_allowed(),
            Type::Option { inner_type } => inner_type.is_allowed(),
        }
    }
}

/// Statement types in restricted AST
///
/// Represents the allowed statement types that can be transpiled to shell script.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stmt {
    /// Let binding (variable declaration)
    Let {
        /// Variable name
        name: String,
        /// Initial value
        value: Expr,
    },
    /// Expression statement
    Expr(Expr),
    /// Return statement with optional value
    Return(Option<Expr>),
    /// If/else conditional
    If {
        /// Condition expression
        condition: Expr,
        /// Statements in then branch
        then_block: Vec<Stmt>,
        /// Optional statements in else branch
        else_block: Option<Vec<Stmt>>,
    },
    /// Match expression (pattern matching)
    Match {
        /// Expression being matched
        scrutinee: Expr,
        /// Match arms
        arms: Vec<MatchArm>,
    },
    /// For loop
    For {
        /// Loop variable pattern
        pattern: Pattern,
        /// Iterator expression
        iter: Expr,
        /// Loop body statements
        body: Vec<Stmt>,
        /// Optional maximum iterations (for safety)
        max_iterations: Option<u32>,
    },
    /// While loop
    While {
        /// Loop condition
        condition: Expr,
        /// Loop body statements
        body: Vec<Stmt>,
        /// Optional maximum iterations (for safety)
        max_iterations: Option<u32>,
    },
    /// Break statement
    Break,
    /// Continue statement
    Continue,
}

impl Stmt {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Stmt::Let { name, value } => {
                Self::validate_identifier(name)?;
                value.validate()
            }
            Stmt::Expr(expr) => expr.validate(),
            Stmt::Return(Some(expr)) => expr.validate(),
            Stmt::Return(None) => Ok(()),
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => self.validate_if_stmt(condition, then_block, else_block.as_ref()),
            Stmt::Match { scrutinee, arms } => self.validate_match_stmt(scrutinee, arms),
            Stmt::For {
                pattern,
                iter,
                body,
                max_iterations,
            } => self.validate_for_stmt(pattern, iter, body, *max_iterations),
            Stmt::While {
                condition,
                body,
                max_iterations,
            } => self.validate_while_stmt(condition, body, *max_iterations),
            Stmt::Break | Stmt::Continue => Ok(()),
        }
    }

    fn validate_identifier(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Identifiers cannot be empty".to_string());
        }
        if name.contains('\0') {
            return Err("Null characters not allowed in identifiers".to_string());
        }
        // Check for shell-unsafe characters
        if name.contains('$') || name.contains('`') || name.contains('\\') {
            return Err(format!("Unsafe characters in identifier: {}", name));
        }
        Ok(())
    }

    fn validate_if_stmt(
        &self,
        condition: &Expr,
        then_block: &[Stmt],
        else_block: Option<&Vec<Stmt>>,
    ) -> Result<(), String> {
        condition.validate()?;
        self.validate_stmt_block(then_block)?;
        if let Some(else_stmts) = else_block {
            self.validate_stmt_block(else_stmts)?
        }
        Ok(())
    }

    fn validate_match_stmt(&self, scrutinee: &Expr, arms: &[MatchArm]) -> Result<(), String> {
        scrutinee.validate()?;
        for arm in arms {
            arm.pattern.validate()?;
            if let Some(guard) = &arm.guard {
                guard.validate()?;
            }
            self.validate_stmt_block(&arm.body)?;
        }
        Ok(())
    }

    fn validate_for_stmt(
        &self,
        pattern: &Pattern,
        iter: &Expr,
        body: &[Stmt],
        max_iterations: Option<u32>,
    ) -> Result<(), String> {
        self.validate_bounded_iteration(max_iterations, "For")?;
        pattern.validate()?;
        iter.validate()?;
        self.validate_stmt_block(body)
    }

    fn validate_while_stmt(
        &self,
        condition: &Expr,
        body: &[Stmt],
        max_iterations: Option<u32>,
    ) -> Result<(), String> {
        self.validate_bounded_iteration(max_iterations, "While")?;
        condition.validate()?;
        self.validate_stmt_block(body)
    }

    fn validate_bounded_iteration(
        &self,
        max_iterations: Option<u32>,
        loop_type: &str,
    ) -> Result<(), String> {
        if max_iterations.is_none() {
            return Err(format!(
                "{loop_type} loops must have bounded iterations for verification"
            ));
        }
        Ok(())
    }

    fn validate_stmt_block(&self, stmts: &[Stmt]) -> Result<(), String> {
        for stmt in stmts {
            stmt.validate()?;
        }
        Ok(())
    }

    pub fn collect_function_calls(&self, calls: &mut Vec<String>) {
        match self {
            Stmt::Let { value, .. } => value.collect_function_calls(calls),
            Stmt::Expr(expr) => expr.collect_function_calls(calls),
            Stmt::Return(Some(expr)) => expr.collect_function_calls(calls),
            Stmt::Return(None) => {}
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
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
            Stmt::While {
                condition, body, ..
            } => {
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
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    Array(Vec<Expr>),
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Try {
        expr: Box<Expr>,
    },
    Block(Vec<Stmt>),
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
    /// Special marker for positional arguments from std::env::args()
    /// This represents the entire args array, not individual parameters
    PositionalArgs,
}

impl Expr {
    pub fn validate(&self) -> Result<(), String> {
        // Check nesting depth
        let depth = self.nesting_depth();
        if depth > 30 {
            return Err(format!(
                "Expression nesting too deep: {depth} levels (max 30)"
            ));
        }

        match self {
            Expr::Literal(Literal::Str(s)) => {
                if s.contains('\0') {
                    return Err("Null characters not allowed in strings".to_string());
                }
                Ok(())
            }
            Expr::Literal(_) => Ok(()),
            Expr::Variable(name) => Self::validate_identifier(name),
            Expr::FunctionCall { name, args } => {
                Self::validate_identifier(name)?;
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
            Expr::MethodCall {
                receiver,
                method,
                args,
            } => {
                receiver.validate()?;
                Self::validate_identifier(method)?;
                for arg in args {
                    arg.validate()?;
                }
                Ok(())
            }
            Expr::Range { start, end, .. } => {
                start.validate()?;
                end.validate()
            }
            // Placeholder for new expression types - TODO: implement properly
            _ => Ok(()), // Array, Index, Try, Block
        }
    }

    fn validate_identifier(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Identifiers cannot be empty".to_string());
        }
        if name.contains('\0') {
            return Err("Null characters not allowed in identifiers".to_string());
        }
        // Check for shell-unsafe characters
        if name.contains('$') || name.contains('`') || name.contains('\\') {
            return Err(format!("Unsafe characters in identifier: {}", name));
        }
        Ok(())
    }

    pub fn nesting_depth(&self) -> usize {
        match self {
            Expr::Binary { left, right, .. } => 1 + left.nesting_depth().max(right.nesting_depth()),
            Expr::Unary { operand, .. } => 1 + operand.nesting_depth(),
            Expr::FunctionCall { args, .. } => {
                1 + args.iter().map(|a| a.nesting_depth()).max().unwrap_or(0)
            }
            Expr::MethodCall { receiver, args, .. } => {
                let receiver_depth = receiver.nesting_depth();
                let args_depth = args.iter().map(|a| a.nesting_depth()).max().unwrap_or(0);
                1 + receiver_depth.max(args_depth)
            }
            Expr::Range { start, end, .. } => 1 + start.nesting_depth().max(end.nesting_depth()),
            _ => 0,
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
            Expr::Range { start, end, .. } => {
                start.collect_function_calls(calls);
                end.collect_function_calls(calls);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Literal {
    Bool(bool),
    U32(u32),
    I32(i32), // Support negative integers
    Str(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem, // Modulo operator (%)
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
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
    },
}

impl Pattern {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Pattern::Literal(Literal::Str(s)) => {
                if s.contains('\0') {
                    return Err(
                        "Null characters not allowed in pattern string literals".to_string()
                    );
                }
                Ok(())
            }
            Pattern::Literal(_) => Ok(()),
            Pattern::Variable(name) => Self::validate_identifier(name),
            Pattern::Wildcard => Ok(()),
            Pattern::Tuple(patterns) => {
                if patterns.is_empty() {
                    return Err("Empty tuple patterns not allowed".to_string());
                }
                for pattern in patterns {
                    pattern.validate()?;
                }
                Ok(())
            }
            Pattern::Struct { name, fields } => {
                Self::validate_identifier(name)?;
                if fields.is_empty() {
                    return Err("Empty struct patterns not allowed".to_string());
                }
                for (field_name, pattern) in fields {
                    Self::validate_identifier(field_name)?;
                    pattern.validate()?;
                }
                Ok(())
            }
        }
    }

    fn validate_identifier(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Identifiers cannot be empty".to_string());
        }
        if name.contains('\0') {
            return Err("Null characters not allowed in identifiers".to_string());
        }
        // Check for shell-unsafe characters
        if name.contains('$') || name.contains('`') || name.contains('\\') {
            return Err(format!("Unsafe characters in identifier: {}", name));
        }
        Ok(())
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
