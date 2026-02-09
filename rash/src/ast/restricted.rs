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

        // Recursion is allowed — shell supports recursive functions
        // (previously rejected, but recursive patterns like factorial are valid)

        Ok(())
    }

    #[allow(dead_code)]
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

    #[allow(dead_code, clippy::only_used_in_recursion)]
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
    /// 16-bit unsigned integer
    U16,
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
            Type::Void | Type::Bool | Type::U16 | Type::U32 | Type::Str => true,
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
    U16(u16),
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
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
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
    Range {
        start: Literal,
        end: Literal,
        inclusive: bool,
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
            Pattern::Range { .. } => Ok(()),
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

#[cfg(test)]
mod tests {
    use super::*;

    // ===== RestrictedAst validation tests =====

    fn create_valid_ast() -> RestrictedAst {
        RestrictedAst {
            entry_point: "main".to_string(),
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            }],
        }
    }

    #[test]
    fn test_valid_ast_validates() {
        let ast = create_valid_ast();
        assert!(ast.validate().is_ok());
    }

    #[test]
    fn test_missing_entry_point() {
        let ast = RestrictedAst {
            entry_point: "nonexistent".to_string(),
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            }],
        };
        let result = ast.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Entry point function"));
    }

    #[test]
    fn test_recursion_allowed_direct() {
        // Recursive functions are allowed — shell supports them
        let ast = RestrictedAst {
            entry_point: "a".to_string(),
            functions: vec![Function {
                name: "a".to_string(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "a".to_string(),
                    args: vec![],
                })],
            }],
        };
        let result = ast.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_recursion_allowed_indirect() {
        // Indirect recursion is also allowed
        let ast = RestrictedAst {
            entry_point: "a".to_string(),
            functions: vec![
                Function {
                    name: "a".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "b".to_string(),
                        args: vec![],
                    })],
                },
                Function {
                    name: "b".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "a".to_string(),
                        args: vec![],
                    })],
                },
            ],
        };
        let result = ast.validate();
        assert!(result.is_ok());
    }

    // ===== Function validation tests =====

    #[test]
    fn test_function_empty_name() {
        let func = Function {
            name: "".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![],
        };
        let result = func.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_function_null_char_in_name() {
        let func = Function {
            name: "func\0name".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![],
        };
        let result = func.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Null"));
    }

    #[test]
    fn test_function_unsafe_chars_in_name() {
        for c in ["$", "`", "\\"] {
            let func = Function {
                name: format!("func{}name", c),
                params: vec![],
                return_type: Type::Void,
                body: vec![],
            };
            let result = func.validate();
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("Unsafe"));
        }
    }

    #[test]
    fn test_function_duplicate_params() {
        let func = Function {
            name: "test".to_string(),
            params: vec![
                Parameter {
                    name: "x".to_string(),
                    param_type: Type::U32,
                },
                Parameter {
                    name: "x".to_string(),
                    param_type: Type::U32,
                },
            ],
            return_type: Type::Void,
            body: vec![],
        };
        let result = func.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate parameter"));
    }

    #[test]
    fn test_function_collect_calls() {
        let func = Function {
            name: "test".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![
                Stmt::Expr(Expr::FunctionCall {
                    name: "foo".to_string(),
                    args: vec![],
                }),
                Stmt::Expr(Expr::FunctionCall {
                    name: "bar".to_string(),
                    args: vec![],
                }),
            ],
        };
        let mut calls = vec![];
        func.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["foo", "bar"]);
    }

    // ===== Type tests =====

    #[test]
    fn test_type_is_allowed_basic() {
        assert!(Type::Void.is_allowed());
        assert!(Type::Bool.is_allowed());
        assert!(Type::U32.is_allowed());
        assert!(Type::Str.is_allowed());
    }

    #[test]
    fn test_type_is_allowed_result() {
        let result_type = Type::Result {
            ok_type: Box::new(Type::U32),
            err_type: Box::new(Type::Str),
        };
        assert!(result_type.is_allowed());
    }

    #[test]
    fn test_type_is_allowed_option() {
        let option_type = Type::Option {
            inner_type: Box::new(Type::Bool),
        };
        assert!(option_type.is_allowed());
    }

    // ===== Statement validation tests =====

    #[test]
    fn test_stmt_let_empty_name() {
        let stmt = Stmt::Let {
            name: "".to_string(),
            value: Expr::Literal(Literal::U32(1)),
        };
        let result = stmt.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_stmt_for_without_max_iterations() {
        let stmt = Stmt::For {
            pattern: Pattern::Variable("i".to_string()),
            iter: Expr::Range {
                start: Box::new(Expr::Literal(Literal::U32(0))),
                end: Box::new(Expr::Literal(Literal::U32(10))),
                inclusive: false,
            },
            body: vec![],
            max_iterations: None,
        };
        let result = stmt.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("bounded iterations"));
    }

    #[test]
    fn test_stmt_while_without_max_iterations() {
        let stmt = Stmt::While {
            condition: Expr::Literal(Literal::Bool(true)),
            body: vec![],
            max_iterations: None,
        };
        let result = stmt.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("bounded iterations"));
    }

    #[test]
    fn test_stmt_break_continue_validate() {
        assert!(Stmt::Break.validate().is_ok());
        assert!(Stmt::Continue.validate().is_ok());
    }

    #[test]
    fn test_stmt_return_none_validates() {
        assert!(Stmt::Return(None).validate().is_ok());
    }

    #[test]
    fn test_stmt_if_validation() {
        let stmt = Stmt::If {
            condition: Expr::Variable("x".to_string()),
            then_block: vec![Stmt::Return(None)],
            else_block: Some(vec![Stmt::Break]),
        };
        assert!(stmt.validate().is_ok());
    }

    #[test]
    fn test_stmt_match_validation() {
        let stmt = Stmt::Match {
            scrutinee: Expr::Variable("x".to_string()),
            arms: vec![MatchArm {
                pattern: Pattern::Wildcard,
                guard: Some(Expr::Literal(Literal::Bool(true))),
                body: vec![Stmt::Return(None)],
            }],
        };
        assert!(stmt.validate().is_ok());
    }

    #[test]
    fn test_stmt_collect_calls_if() {
        let stmt = Stmt::If {
            condition: Expr::FunctionCall {
                name: "cond".to_string(),
                args: vec![],
            },
            then_block: vec![Stmt::Expr(Expr::FunctionCall {
                name: "then_fn".to_string(),
                args: vec![],
            })],
            else_block: Some(vec![Stmt::Expr(Expr::FunctionCall {
                name: "else_fn".to_string(),
                args: vec![],
            })]),
        };
        let mut calls = vec![];
        stmt.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["cond", "then_fn", "else_fn"]);
    }

    #[test]
    fn test_stmt_collect_calls_match() {
        let stmt = Stmt::Match {
            scrutinee: Expr::FunctionCall {
                name: "scrut".to_string(),
                args: vec![],
            },
            arms: vec![MatchArm {
                pattern: Pattern::Wildcard,
                guard: Some(Expr::FunctionCall {
                    name: "guard".to_string(),
                    args: vec![],
                }),
                body: vec![Stmt::Expr(Expr::FunctionCall {
                    name: "body".to_string(),
                    args: vec![],
                })],
            }],
        };
        let mut calls = vec![];
        stmt.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["scrut", "guard", "body"]);
    }

    #[test]
    fn test_stmt_collect_calls_for_while() {
        let for_stmt = Stmt::For {
            pattern: Pattern::Variable("i".to_string()),
            iter: Expr::FunctionCall {
                name: "iter".to_string(),
                args: vec![],
            },
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "loop_fn".to_string(),
                args: vec![],
            })],
            max_iterations: Some(10),
        };
        let mut calls = vec![];
        for_stmt.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["iter", "loop_fn"]);

        let while_stmt = Stmt::While {
            condition: Expr::FunctionCall {
                name: "cond".to_string(),
                args: vec![],
            },
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "body".to_string(),
                args: vec![],
            })],
            max_iterations: Some(10),
        };
        let mut calls = vec![];
        while_stmt.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["cond", "body"]);
    }

    // ===== Expression validation tests =====

    #[test]
    fn test_expr_literal_null_string() {
        let expr = Expr::Literal(Literal::Str("hello\0world".to_string()));
        let result = expr.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Null"));
    }

    #[test]
    fn test_expr_variable_empty_name() {
        let expr = Expr::Variable("".to_string());
        let result = expr.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_expr_function_call_empty_name() {
        let expr = Expr::FunctionCall {
            name: "".to_string(),
            args: vec![],
        };
        let result = expr.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_expr_method_call_empty_method() {
        let expr = Expr::MethodCall {
            receiver: Box::new(Expr::Variable("obj".to_string())),
            method: "".to_string(),
            args: vec![],
        };
        let result = expr.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_expr_nesting_depth() {
        // Create deeply nested expression
        let mut expr = Expr::Literal(Literal::U32(1));
        for _ in 0..35 {
            expr = Expr::Unary {
                op: UnaryOp::Neg,
                operand: Box::new(expr),
            };
        }
        let result = expr.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nesting too deep"));
    }

    #[test]
    fn test_expr_nesting_depth_binary() {
        let leaf = Expr::Literal(Literal::U32(1));
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(leaf.clone()),
            right: Box::new(leaf),
        };
        assert_eq!(expr.nesting_depth(), 1);
    }

    #[test]
    fn test_expr_collect_calls_nested() {
        let expr = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::FunctionCall {
                name: "left".to_string(),
                args: vec![],
            }),
            right: Box::new(Expr::FunctionCall {
                name: "right".to_string(),
                args: vec![],
            }),
        };
        let mut calls = vec![];
        expr.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["left", "right"]);
    }

    #[test]
    fn test_expr_collect_calls_array() {
        let expr = Expr::Array(vec![
            Expr::FunctionCall {
                name: "a".to_string(),
                args: vec![],
            },
            Expr::FunctionCall {
                name: "b".to_string(),
                args: vec![],
            },
        ]);
        let mut calls = vec![];
        expr.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["a", "b"]);
    }

    #[test]
    fn test_expr_collect_calls_index() {
        let expr = Expr::Index {
            object: Box::new(Expr::FunctionCall {
                name: "arr".to_string(),
                args: vec![],
            }),
            index: Box::new(Expr::FunctionCall {
                name: "idx".to_string(),
                args: vec![],
            }),
        };
        let mut calls = vec![];
        expr.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["arr", "idx"]);
    }

    #[test]
    fn test_expr_collect_calls_try() {
        let expr = Expr::Try {
            expr: Box::new(Expr::FunctionCall {
                name: "fallible".to_string(),
                args: vec![],
            }),
        };
        let mut calls = vec![];
        expr.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["fallible"]);
    }

    #[test]
    fn test_expr_collect_calls_block() {
        let expr = Expr::Block(vec![Stmt::Expr(Expr::FunctionCall {
            name: "inner".to_string(),
            args: vec![],
        })]);
        let mut calls = vec![];
        expr.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["inner"]);
    }

    #[test]
    fn test_expr_collect_calls_range() {
        let expr = Expr::Range {
            start: Box::new(Expr::FunctionCall {
                name: "start".to_string(),
                args: vec![],
            }),
            end: Box::new(Expr::FunctionCall {
                name: "end".to_string(),
                args: vec![],
            }),
            inclusive: false,
        };
        let mut calls = vec![];
        expr.collect_function_calls(&mut calls);
        assert_eq!(calls, vec!["start", "end"]);
    }

    // ===== Pattern validation tests =====

    #[test]
    fn test_pattern_literal_null_string() {
        let pattern = Pattern::Literal(Literal::Str("hello\0world".to_string()));
        let result = pattern.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Null"));
    }

    #[test]
    fn test_pattern_variable_empty() {
        let pattern = Pattern::Variable("".to_string());
        let result = pattern.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_wildcard_validates() {
        assert!(Pattern::Wildcard.validate().is_ok());
    }

    #[test]
    fn test_pattern_tuple_empty() {
        let pattern = Pattern::Tuple(vec![]);
        let result = pattern.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Empty tuple"));
    }

    #[test]
    fn test_pattern_tuple_valid() {
        let pattern = Pattern::Tuple(vec![Pattern::Variable("a".to_string()), Pattern::Wildcard]);
        assert!(pattern.validate().is_ok());
    }

    #[test]
    fn test_pattern_struct_empty() {
        let pattern = Pattern::Struct {
            name: "MyStruct".to_string(),
            fields: vec![],
        };
        let result = pattern.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Empty struct"));
    }

    #[test]
    fn test_pattern_struct_invalid_name() {
        let pattern = Pattern::Struct {
            name: "".to_string(),
            fields: vec![("x".to_string(), Pattern::Wildcard)],
        };
        let result = pattern.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_struct_invalid_field_name() {
        let pattern = Pattern::Struct {
            name: "MyStruct".to_string(),
            fields: vec![("".to_string(), Pattern::Wildcard)],
        };
        let result = pattern.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_binds_variable() {
        let pattern = Pattern::Variable("x".to_string());
        assert!(pattern.binds_variable("x"));
        assert!(!pattern.binds_variable("y"));
    }

    #[test]
    fn test_pattern_binds_variable_tuple() {
        let pattern = Pattern::Tuple(vec![
            Pattern::Variable("a".to_string()),
            Pattern::Variable("b".to_string()),
        ]);
        assert!(pattern.binds_variable("a"));
        assert!(pattern.binds_variable("b"));
        assert!(!pattern.binds_variable("c"));
    }

    #[test]
    fn test_pattern_binds_variable_struct() {
        let pattern = Pattern::Struct {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Pattern::Variable("px".to_string())),
                ("y".to_string(), Pattern::Variable("py".to_string())),
            ],
        };
        assert!(pattern.binds_variable("px"));
        assert!(pattern.binds_variable("py"));
        assert!(!pattern.binds_variable("x"));
    }

    #[test]
    fn test_pattern_binds_variable_wildcard() {
        assert!(!Pattern::Wildcard.binds_variable("x"));
    }

    #[test]
    fn test_pattern_binds_variable_literal() {
        let pattern = Pattern::Literal(Literal::U32(42));
        assert!(!pattern.binds_variable("x"));
    }

    // ===== Literal tests =====

    #[test]
    fn test_literal_eq() {
        assert_eq!(Literal::Bool(true), Literal::Bool(true));
        assert_ne!(Literal::Bool(true), Literal::Bool(false));
        assert_eq!(Literal::U32(42), Literal::U32(42));
        assert_eq!(Literal::I32(-5), Literal::I32(-5));
        assert_eq!(
            Literal::Str("hello".to_string()),
            Literal::Str("hello".to_string())
        );
    }

    // ===== No recursion with multiple functions =====

    #[test]
    fn test_no_recursion_chain() {
        let ast = RestrictedAst {
            entry_point: "a".to_string(),
            functions: vec![
                Function {
                    name: "a".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "b".to_string(),
                        args: vec![],
                    })],
                },
                Function {
                    name: "b".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![Stmt::Expr(Expr::FunctionCall {
                        name: "c".to_string(),
                        args: vec![],
                    })],
                },
                Function {
                    name: "c".to_string(),
                    params: vec![],
                    return_type: Type::Void,
                    body: vec![],
                },
            ],
        };
        assert!(ast.validate().is_ok());
    }
}
