//! Bash Abstract Syntax Tree
//!
//! Represents parsed bash scripts in a type-safe AST structure.
//! Designed to capture semantics needed for transpilation to Rash.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Root AST node representing a complete bash script
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BashAst {
    pub statements: Vec<BashStmt>,
    pub metadata: AstMetadata,
}

/// Metadata about the parsed script
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstMetadata {
    pub source_file: Option<String>,
    pub line_count: usize,
    pub parse_time_ms: u64,
}

/// Statement-level AST node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BashStmt {
    /// Variable assignment: VAR=value
    Assignment {
        name: String,
        value: BashExpr,
        exported: bool,
        span: Span,
    },

    /// Command execution: echo "hello"
    Command {
        name: String,
        args: Vec<BashExpr>,
        span: Span,
    },

    /// Function definition
    Function {
        name: String,
        body: Vec<BashStmt>,
        span: Span,
    },

    /// If statement
    If {
        condition: BashExpr,
        then_block: Vec<BashStmt>,
        elif_blocks: Vec<(BashExpr, Vec<BashStmt>)>,
        else_block: Option<Vec<BashStmt>>,
        span: Span,
    },

    /// While loop
    While {
        condition: BashExpr,
        body: Vec<BashStmt>,
        span: Span,
    },

    /// Until loop: until CONDITION; do BODY; done
    /// Note: Purified to `while ! CONDITION` for POSIX compatibility
    Until {
        condition: BashExpr,
        body: Vec<BashStmt>,
        span: Span,
    },

    /// For loop: for VAR in LIST; do BODY; done
    For {
        variable: String,
        items: BashExpr,
        body: Vec<BashStmt>,
        span: Span,
    },

    /// Return statement
    Return { code: Option<BashExpr>, span: Span },

    /// Case statement: case WORD in PATTERN) BODY;; esac
    Case {
        word: BashExpr,
        arms: Vec<CaseArm>,
        span: Span,
    },

    /// Comment (preserved for documentation)
    Comment { text: String, span: Span },
}

/// Case statement arm
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseArm {
    pub patterns: Vec<String>,
    pub body: Vec<BashStmt>,
}

/// Expression-level AST node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BashExpr {
    /// String literal
    Literal(String),

    /// Variable reference: $VAR or ${VAR}
    Variable(String),

    /// Command substitution: $(cmd) or `cmd`
    CommandSubst(Box<BashStmt>),

    /// Arithmetic expansion: $((expr))
    Arithmetic(Box<ArithExpr>),

    /// Array/list: (item1 item2 item3)
    Array(Vec<BashExpr>),

    /// String concatenation
    Concat(Vec<BashExpr>),

    /// Test expression: [ expr ]
    Test(Box<TestExpr>),

    /// Glob pattern: *.txt
    Glob(String),

    /// Default value expansion: ${VAR:-default}
    /// If variable is unset or null, use default value
    DefaultValue {
        variable: String,
        default: Box<BashExpr>,
    },

    /// Assign default value expansion: ${VAR:=default}
    /// If variable is unset or null, assign default value to variable and use it
    AssignDefault {
        variable: String,
        default: Box<BashExpr>,
    },

    /// Error if unset expansion: ${VAR:?message}
    /// If variable is unset or null, exit with error message
    ErrorIfUnset {
        variable: String,
        message: Box<BashExpr>,
    },

    /// Alternative value expansion: ${VAR:+alt_value}
    /// If variable is set and non-null, use alt_value, otherwise empty string
    AlternativeValue {
        variable: String,
        alternative: Box<BashExpr>,
    },

    /// String length expansion: ${#VAR}
    /// Get the length of the string value of variable
    StringLength { variable: String },

    /// Remove suffix expansion: ${VAR%pattern}
    /// Remove shortest matching suffix pattern from variable
    RemoveSuffix {
        variable: String,
        pattern: Box<BashExpr>,
    },

    /// Remove prefix expansion: ${VAR#pattern}
    /// Remove shortest matching prefix pattern from variable
    RemovePrefix {
        variable: String,
        pattern: Box<BashExpr>,
    },

    /// Remove longest prefix expansion: ${VAR##pattern}
    /// Remove longest matching prefix pattern from variable (greedy)
    RemoveLongestPrefix {
        variable: String,
        pattern: Box<BashExpr>,
    },

    /// Remove longest suffix expansion: ${VAR%%pattern}
    /// Remove longest matching suffix pattern from variable (greedy)
    RemoveLongestSuffix {
        variable: String,
        pattern: Box<BashExpr>,
    },
}

/// Arithmetic expression
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArithExpr {
    Number(i64),
    Variable(String),
    Add(Box<ArithExpr>, Box<ArithExpr>),
    Sub(Box<ArithExpr>, Box<ArithExpr>),
    Mul(Box<ArithExpr>, Box<ArithExpr>),
    Div(Box<ArithExpr>, Box<ArithExpr>),
    Mod(Box<ArithExpr>, Box<ArithExpr>),
}

/// Test expression (conditional)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestExpr {
    /// String comparison
    StringEq(BashExpr, BashExpr),
    StringNe(BashExpr, BashExpr),

    /// Integer comparison
    IntEq(BashExpr, BashExpr),
    IntNe(BashExpr, BashExpr),
    IntLt(BashExpr, BashExpr),
    IntLe(BashExpr, BashExpr),
    IntGt(BashExpr, BashExpr),
    IntGe(BashExpr, BashExpr),

    /// File tests
    FileExists(BashExpr),
    FileReadable(BashExpr),
    FileWritable(BashExpr),
    FileExecutable(BashExpr),
    FileDirectory(BashExpr),

    /// String tests
    StringEmpty(BashExpr),
    StringNonEmpty(BashExpr),

    /// Logical operations
    And(Box<TestExpr>, Box<TestExpr>),
    Or(Box<TestExpr>, Box<TestExpr>),
    Not(Box<TestExpr>),
}

/// Source code span for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Span {
    pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    pub fn dummy() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

impl fmt::Display for BashStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BashStmt::Assignment { name, .. } => write!(f, "Assignment({})", name),
            BashStmt::Command { name, .. } => write!(f, "Command({})", name),
            BashStmt::Function { name, .. } => write!(f, "Function({})", name),
            BashStmt::If { .. } => write!(f, "If"),
            BashStmt::While { .. } => write!(f, "While"),
            BashStmt::Until { .. } => write!(f, "Until"),
            BashStmt::For { variable, .. } => write!(f, "For({})", variable),
            BashStmt::Case { .. } => write!(f, "Case"),
            BashStmt::Return { .. } => write!(f, "Return"),
            BashStmt::Comment { .. } => write!(f, "Comment"),
        }
    }
}

/// Wrapper type for AST nodes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BashNode<T> {
    pub node: T,
    pub span: Span,
}

impl<T> BashNode<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_construction() {
        let stmt = BashStmt::Assignment {
            name: "FOO".to_string(),
            value: BashExpr::Literal("bar".to_string()),
            exported: false,
            span: Span::dummy(),
        };

        assert!(matches!(stmt, BashStmt::Assignment { .. }));
    }

    #[test]
    fn test_span_creation() {
        let span = Span::new(1, 5, 1, 10);
        assert_eq!(span.start_line, 1);
        assert_eq!(span.start_col, 5);
        assert_eq!(span.end_line, 1);
        assert_eq!(span.end_col, 10);
    }
}
