use super::effects::EffectSet;
use serde::{Deserialize, Serialize};

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
            ShellIR::Exit { .. } | ShellIR::Noop => EffectSet::pure(),
            ShellIR::Function { body, .. } => body.effects(),
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    /// -eq: numeric equality
    Eq,
    /// -ne: numeric inequality
    Ne,
    /// -gt: greater than
    Gt,
    /// -ge: greater than or equal
    Ge,
    /// -lt: less than
    Lt,
    /// -le: less than or equal
    Le,
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
            ShellValue::Variable(_) | ShellValue::CommandSubst(_) => false,
            ShellValue::Concat(parts) => parts.iter().all(|p| p.is_constant()),
            ShellValue::Comparison { left, right, .. } | ShellValue::Arithmetic { left, right, .. } => {
                left.is_constant() && right.is_constant()
            }
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
