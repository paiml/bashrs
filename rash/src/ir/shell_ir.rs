use super::effects::EffectSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseArm {
    pub pattern: CasePattern,
    pub guard: Option<ShellValue>,
    pub body: Box<ShellIR>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CasePattern {
    Literal(String), // Literal value to match
    Wildcard,        // * pattern
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellIR {
    /// Variable assignment: readonly NAME=VALUE
    Let {
        name: String,
        value: ShellValue,
        effects: EffectSet,
    },

    /// Command execution
    Exec { cmd: Command, effects: EffectSet },

    /// Conditional execution
    If {
        test: ShellValue,
        then_branch: Box<ShellIR>,
        else_branch: Option<Box<ShellIR>>,
    },

    /// Exit with code
    Exit { code: u8, message: Option<String> },

    /// Sequence of operations
    Sequence(Vec<ShellIR>),

    /// No-op
    Noop,

    /// Function definition
    Function {
        name: String,
        params: Vec<String>,
        body: Box<ShellIR>,
    },

    /// Echo a value (for function returns)
    Echo { value: ShellValue },

    /// For loop with range
    For {
        var: String,
        start: ShellValue,
        end: ShellValue,
        body: Box<ShellIR>,
    },

    /// Case statement (for match expressions)
    Case {
        scrutinee: ShellValue,
        arms: Vec<CaseArm>,
    },

    /// While loop
    While {
        condition: ShellValue,
        body: Box<ShellIR>,
    },

    /// Break statement
    Break,

    /// Continue statement
    Continue,
}

impl ShellIR {
    /// Get all effects from this IR node and its children
    pub fn effects(&self) -> EffectSet {
        match self {
            ShellIR::Let { effects, .. } | ShellIR::Exec { effects, .. } => effects.clone(),
            ShellIR::If {
                then_branch,
                else_branch,
                ..
            } => {
                let mut combined = then_branch.effects();
                if let Some(else_ir) = else_branch {
                    combined = combined.union(&else_ir.effects());
                }
                combined
            }
            ShellIR::Sequence(items) => items
                .iter()
                .fold(EffectSet::pure(), |acc, item| acc.union(&item.effects())),
            ShellIR::Exit { .. } | ShellIR::Noop | ShellIR::Echo { .. } => EffectSet::pure(),
            ShellIR::Function { body, .. } => body.effects(),
            ShellIR::For { body, .. } => body.effects(),
            ShellIR::While { body, .. } => body.effects(),
            ShellIR::Case { arms, .. } => arms
                .iter()
                .fold(EffectSet::pure(), |acc, arm| acc.union(&arm.body.effects())),
            ShellIR::Break | ShellIR::Continue => EffectSet::pure(),
        }
    }

    /// Check if this IR node is pure (has no side effects)
    pub fn is_pure(&self) -> bool {
        self.effects().is_pure()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub program: String,
    pub args: Vec<ShellValue>,
}

impl Command {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: ShellValue) -> Self {
        self.args.push(arg);
        self
    }

    pub fn args(mut self, args: Vec<ShellValue>) -> Self {
        self.args.extend(args);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellValue {
    /// String literal
    String(String),

    /// Boolean value (converted to "true"/"false")
    Bool(bool),

    /// Variable reference
    Variable(String),

    /// Concatenated values
    Concat(Vec<ShellValue>),

    /// Command substitution
    CommandSubst(Command),

    /// Comparison operation (for test conditions)
    Comparison {
        op: ComparisonOp,
        left: Box<ShellValue>,
        right: Box<ShellValue>,
    },

    /// Arithmetic operation (for $((expr)))
    Arithmetic {
        op: ArithmeticOp,
        left: Box<ShellValue>,
        right: Box<ShellValue>,
    },

    /// Logical AND (&&) operation
    LogicalAnd {
        left: Box<ShellValue>,
        right: Box<ShellValue>,
    },

    /// Logical OR (||) operation
    LogicalOr {
        left: Box<ShellValue>,
        right: Box<ShellValue>,
    },

    /// Logical NOT (!) operation
    LogicalNot { operand: Box<ShellValue> },

    /// Environment variable expansion: ${VAR} or ${VAR:-default}
    /// Sprint 27a: Environment Variables Support
    EnvVar {
        name: String,
        default: Option<String>,
    },

    /// Command-line argument access: $1, $2, $@, etc.
    /// Sprint 27b: Command-Line Arguments Support
    Arg {
        position: Option<usize>, // None = all args ($@)
    },

    /// Command-line argument with default value: ${1:-default}
    /// P0-POSITIONAL-PARAMETERS: Support for args.get(N).unwrap_or(default)
    ArgWithDefault { position: usize, default: String },

    /// Argument count: $#
    /// Sprint 27b: Command-Line Arguments Support
    ArgCount,

    /// Exit code of last command: $?
    /// Sprint 27c: Exit Code Handling
    ExitCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    /// -eq: numeric equality
    NumEq,
    /// -ne: numeric inequality
    NumNe,
    /// -gt: numeric greater than
    Gt,
    /// -ge: numeric greater than or equal
    Ge,
    /// -lt: numeric less than
    Lt,
    /// -le: numeric less than or equal
    Le,
    /// =: string equality
    StrEq,
    /// !=: string inequality
    StrNe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArithmeticOp {
    /// + : addition
    Add,
    /// - : subtraction
    Sub,
    /// * : multiplication
    Mul,
    /// / : division
    Div,
    /// % : modulo
    Mod,
}

impl ShellValue {
    /// Check if this value is a constant (doesn't depend on variables or commands)
    pub fn is_constant(&self) -> bool {
        match self {
            ShellValue::String(_) | ShellValue::Bool(_) => true,
            ShellValue::Variable(_)
            | ShellValue::CommandSubst(_)
            | ShellValue::EnvVar { .. }
            | ShellValue::Arg { .. }
            | ShellValue::ArgWithDefault { .. }
            | ShellValue::ArgCount
            | ShellValue::ExitCode => false,
            ShellValue::Concat(parts) => parts.iter().all(|p| p.is_constant()),
            ShellValue::Comparison { left, right, .. }
            | ShellValue::Arithmetic { left, right, .. }
            | ShellValue::LogicalAnd { left, right }
            | ShellValue::LogicalOr { left, right } => left.is_constant() && right.is_constant(),
            ShellValue::LogicalNot { operand } => operand.is_constant(),
        }
    }

    /// Get the string representation for constant values
    pub fn as_constant_string(&self) -> Option<String> {
        match self {
            ShellValue::String(s) => Some(s.clone()),
            ShellValue::Bool(b) => Some(if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }),
            ShellValue::Concat(parts) => {
                if parts.iter().all(|p| p.is_constant()) {
                    let mut result = String::new();
                    for part in parts {
                        if let Some(s) = part.as_constant_string() {
                            result.push_str(&s);
                        } else {
                            return None;
                        }
                    }
                    Some(result)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellExpression {
    String(String),
    Variable(String, bool), // (name, is_quoted)
    Command(String),
    Arithmetic(String),
}

impl ShellExpression {
    pub fn is_quoted(&self) -> bool {
        match self {
            ShellExpression::String(s) => s.starts_with('"') && s.ends_with('"'),
            ShellExpression::Variable(_, quoted) => *quoted,
            ShellExpression::Command(_) => false,
            ShellExpression::Arithmetic(_) => true,
        }
    }
}
