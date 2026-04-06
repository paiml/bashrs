/// Collect function calls from a block of statements
fn collect_calls_from_block(stmts: &[Stmt], calls: &mut Vec<String>) {
    for stmt in stmts {
        stmt.collect_function_calls(calls);
    }
}

/// Collect function calls from match arms
fn collect_calls_from_match_arms(arms: &[MatchArm], calls: &mut Vec<String>) {
    for arm in arms {
        if let Some(guard) = &arm.guard {
            guard.collect_function_calls(calls);
        }
        for stmt in &arm.body {
            stmt.collect_function_calls(calls);
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
#[path = "restricted_tests_create_valid.rs"]
mod tests_extracted;
