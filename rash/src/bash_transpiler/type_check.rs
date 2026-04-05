//! Gradual Type System for Shell Purification
//!
//! Provides optional type checking for bash scripts during purification.
//! Like TypeScript for JavaScript: untyped scripts pass through unchanged,
//! annotated scripts get type checking and optional runtime guards.
//!
//! ## Type Annotations
//!
//! Variables can be annotated via comments or `declare`:
//! ```bash
//! # @type port: int
//! port=8080
//!
//! declare -i count=0
//! declare -a items=(a b c)
//! ```
//!
//! Functions can have parameter and return type annotations:
//! ```bash
//! # @param name: str
//! # @param port: int
//! # @returns: int
//! start_server() { ... }
//! ```

use crate::bash_parser::ast::{ArithExpr, BashAst, BashExpr, BashStmt, Span, TestExpr};
use crate::formatter::types::ShellType;
use std::collections::HashMap;
use std::fmt;

/// Severity level for type diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Type error — likely runtime failure
    Error,
    /// Type warning — suspicious but may work
    Warning,
    /// Informational — implicit coercion noted
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

/// Kind of type diagnostic
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// Type mismatch between expected and actual
    TypeMismatch {
        expected: ShellType,
        actual: ShellType,
    },
    /// Variable used without declaration (informational in gradual mode)
    UndeclaredVariable { name: String },
    /// Implicit coercion between types
    ImplicitCoercion { from: ShellType, to: ShellType },
    /// String used in arithmetic context
    StringInArithmetic { variable: String },
}

impl fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticKind::TypeMismatch { expected, actual } => {
                write!(
                    f,
                    "type mismatch: expected {}, found {}",
                    expected.display(),
                    actual.display()
                )
            }
            DiagnosticKind::UndeclaredVariable { name } => {
                write!(f, "undeclared variable: {name}")
            }
            DiagnosticKind::ImplicitCoercion { from, to } => {
                write!(
                    f,
                    "implicit coercion from {} to {}",
                    from.display(),
                    to.display()
                )
            }
            DiagnosticKind::StringInArithmetic { variable } => {
                write!(
                    f,
                    "variable '{variable}' used in arithmetic context but typed as string"
                )
            }
        }
    }
}

/// A type diagnostic with location and severity
#[derive(Debug, Clone)]
pub struct TypeDiagnostic {
    pub span: Span,
    pub kind: DiagnosticKind,
    pub severity: Severity,
    pub message: String,
}

impl fmt::Display for TypeDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}: {}",
            self.span.start_line, self.span.start_col, self.severity, self.kind, self.message,
        )
    }
}

/// Function type signature
#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub params: Vec<(String, ShellType)>,
    pub return_type: Option<ShellType>,
}

/// Scoped variable type environment
#[derive(Debug)]
pub struct TypeContext {
    /// Stack of scopes (innermost last)
    scopes: Vec<HashMap<String, ShellType>>,
    /// Function signatures
    functions: HashMap<String, FunctionSig>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
        }
    }

    /// Push a new scope (entering function/block)
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the current scope (leaving function/block)
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Set a variable's type in the current scope
    pub fn set_type(&mut self, name: &str, ty: ShellType) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    /// Look up a variable's type, searching from innermost scope outward
    pub fn lookup(&self, name: &str) -> Option<&ShellType> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    /// Register a function signature
    pub fn set_function_sig(&mut self, name: &str, sig: FunctionSig) {
        self.functions.insert(name.to_string(), sig);
    }

    /// Look up a function signature
    pub fn lookup_function(&self, name: &str) -> Option<&FunctionSig> {
        self.functions.get(name)
    }

    /// Get the number of active scopes (for testing)
    pub fn scope_depth(&self) -> usize {
        self.scopes.len()
    }
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Gradual type checker for bash ASTs
///
/// Walks the AST and performs type inference and checking.
/// Untyped variables produce no errors (gradual typing).
/// Type annotations come from comments (`# @type`) and `declare` statements.
pub struct TypeChecker {
    ctx: TypeContext,
    diagnostics: Vec<TypeDiagnostic>,
    /// Pending type annotations from comment parsing
    pending_annotations: Vec<TypeAnnotation>,
    /// Original annotation type names (for guard generation, e.g., "path" vs "str")
    annotation_hints: HashMap<String, String>,
}

/// A parsed type annotation from a comment
#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    /// The variable or parameter name
    pub name: String,
    /// The annotated type
    pub shell_type: ShellType,
    /// Original type name string (e.g., "path", "int", "str")
    pub type_hint: String,
    /// Whether this is a return type annotation
    pub is_return: bool,
    /// Whether this is a parameter annotation
    pub is_param: bool,
}

include!("type_check_typechecker.rs");
