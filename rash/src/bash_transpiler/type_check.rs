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
}

/// A parsed type annotation from a comment
#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    /// The variable or parameter name
    pub name: String,
    /// The annotated type
    pub shell_type: ShellType,
    /// Whether this is a return type annotation
    pub is_return: bool,
    /// Whether this is a parameter annotation
    pub is_param: bool,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            ctx: TypeContext::new(),
            diagnostics: Vec::new(),
            pending_annotations: Vec::new(),
        }
    }

    /// Type-check a complete AST, returning diagnostics
    pub fn check_ast(&mut self, ast: &BashAst) -> Vec<TypeDiagnostic> {
        for stmt in &ast.statements {
            self.check_statement(stmt);
        }
        self.diagnostics.clone()
    }

    /// Check a single statement
    pub fn check_statement(&mut self, stmt: &BashStmt) {
        match stmt {
            BashStmt::Comment { text, .. } => {
                // Parse type annotations from comments
                if let Some(annotation) = parse_type_annotation(text) {
                    self.pending_annotations.push(annotation);
                }
            }

            BashStmt::Assignment {
                name, value, span, ..
            } => {
                // Check if there's a pending type annotation for this variable
                let annotated_type = self.consume_annotation(name);

                // Infer the type of the value
                let inferred = self.infer_expr(value);

                if let Some(ref ann_ty) = annotated_type {
                    // Register the annotated type
                    self.ctx.set_type(name, ann_ty.clone());

                    // Check compatibility if both sides are typed
                    if let Some(ref inf_ty) = inferred {
                        if !ann_ty.is_compatible(inf_ty) && !is_gradual_compatible(ann_ty, inf_ty) {
                            self.diagnostics.push(TypeDiagnostic {
                                span: *span,
                                kind: DiagnosticKind::TypeMismatch {
                                    expected: ann_ty.clone(),
                                    actual: inf_ty.clone(),
                                },
                                severity: Severity::Warning,
                                message: format!(
                                    "variable '{}' annotated as {} but assigned {}",
                                    name,
                                    ann_ty.display(),
                                    inf_ty.display()
                                ),
                            });
                        }
                    }
                } else if let Some(inf_ty) = inferred {
                    // No annotation — use inferred type
                    self.ctx.set_type(name, inf_ty);
                }
            }

            BashStmt::Command {
                name, args, span, ..
            } => {
                // Check declare commands for type info
                if name == "declare" || name == "typeset" || name == "local" {
                    self.check_declare(args, *span);
                }

                // Check arguments
                for arg in args {
                    self.infer_expr(arg);
                }
            }

            BashStmt::Function { name, body, .. } => {
                // Collect pending param/return annotations into a signature
                let sig = self.collect_function_sig();
                if sig.is_some() {
                    self.ctx
                        .set_function_sig(name, sig.as_ref().cloned().unwrap_or(FunctionSig {
                            params: Vec::new(),
                            return_type: None,
                        }));
                }

                // Enter function scope
                self.ctx.push_scope();

                // Register parameter types from the signature
                if let Some(ref sig) = sig {
                    for (param_name, param_type) in &sig.params {
                        self.ctx.set_type(param_name, param_type.clone());
                    }
                }

                // Check body
                for stmt in body {
                    self.check_statement(stmt);
                }

                // Leave function scope
                self.ctx.pop_scope();
            }

            BashStmt::If {
                condition,
                then_block,
                elif_blocks,
                else_block,
                ..
            } => {
                self.infer_expr(condition);
                for stmt in then_block {
                    self.check_statement(stmt);
                }
                for (cond, block) in elif_blocks {
                    self.infer_expr(cond);
                    for stmt in block {
                        self.check_statement(stmt);
                    }
                }
                if let Some(else_body) = else_block {
                    for stmt in else_body {
                        self.check_statement(stmt);
                    }
                }
            }

            BashStmt::While { condition, body, .. }
            | BashStmt::Until { condition, body, .. } => {
                self.infer_expr(condition);
                for stmt in body {
                    self.check_statement(stmt);
                }
            }

            BashStmt::For { body, items, .. } => {
                self.infer_expr(items);
                for stmt in body {
                    self.check_statement(stmt);
                }
            }

            BashStmt::ForCStyle { body, .. } => {
                for stmt in body {
                    self.check_statement(stmt);
                }
            }

            BashStmt::Case { word, arms, .. } => {
                self.infer_expr(word);
                for arm in arms {
                    for stmt in &arm.body {
                        self.check_statement(stmt);
                    }
                }
            }

            BashStmt::Pipeline { commands, .. } => {
                for cmd in commands {
                    self.check_statement(cmd);
                }
            }

            BashStmt::AndList { left, right, .. } | BashStmt::OrList { left, right, .. } => {
                self.check_statement(left);
                self.check_statement(right);
            }

            BashStmt::BraceGroup { body, .. } | BashStmt::Coproc { body, .. } => {
                for stmt in body {
                    self.check_statement(stmt);
                }
            }

            BashStmt::Select { body, items, .. } => {
                self.infer_expr(items);
                for stmt in body {
                    self.check_statement(stmt);
                }
            }

            BashStmt::Negated { command, .. } => {
                self.check_statement(command);
            }

            BashStmt::Return { code, .. } => {
                if let Some(expr) = code {
                    self.infer_expr(expr);
                }
            }
        }
    }

    /// Infer the type of an expression
    pub fn infer_expr(&mut self, expr: &BashExpr) -> Option<ShellType> {
        match expr {
            BashExpr::Literal(s) => {
                // Try to detect integer literals
                if s.chars().all(|c| c.is_ascii_digit() || c == '-')
                    && !s.is_empty()
                    && s != "-"
                {
                    Some(ShellType::Integer)
                } else {
                    Some(ShellType::String)
                }
            }

            BashExpr::Variable(name) => self.ctx.lookup(name).cloned(),

            BashExpr::CommandSubst(_) => {
                // Command substitution always returns a string
                Some(ShellType::String)
            }

            BashExpr::Arithmetic(_arith) => {
                // Arithmetic expressions always produce integers
                Some(ShellType::Integer)
            }

            BashExpr::Array(elements) => {
                // Infer element types
                for elem in elements {
                    self.infer_expr(elem);
                }
                Some(ShellType::Array(Box::new(ShellType::String)))
            }

            BashExpr::Concat(parts) => {
                // String concatenation always produces a string
                for part in parts {
                    self.infer_expr(part);
                }
                Some(ShellType::String)
            }

            BashExpr::Test(_) => Some(ShellType::Boolean),

            BashExpr::Glob(_) => Some(ShellType::String),

            BashExpr::CommandCondition(_) => Some(ShellType::ExitCode),

            BashExpr::DefaultValue { variable, default } => {
                self.infer_expr(default);
                // Type is the variable's type if known, else default's type
                self.ctx.lookup(variable).cloned()
            }

            BashExpr::AssignDefault { variable, default } => {
                self.infer_expr(default);
                self.ctx.lookup(variable).cloned()
            }

            BashExpr::ErrorIfUnset { variable, message } => {
                self.infer_expr(message);
                self.ctx.lookup(variable).cloned()
            }

            BashExpr::AlternativeValue {
                variable,
                alternative,
            } => {
                self.infer_expr(alternative);
                self.ctx.lookup(variable).cloned()
            }

            BashExpr::StringLength { .. } => Some(ShellType::Integer),

            BashExpr::RemoveSuffix { pattern, .. }
            | BashExpr::RemovePrefix { pattern, .. }
            | BashExpr::RemoveLongestPrefix { pattern, .. }
            | BashExpr::RemoveLongestSuffix { pattern, .. } => {
                self.infer_expr(pattern);
                Some(ShellType::String)
            }
        }
    }

    /// Infer the type of an arithmetic expression (always Integer)
    pub fn infer_arithmetic(&self, _arith: &ArithExpr) -> ShellType {
        ShellType::Integer
    }

    /// Infer the type of a test expression (always Boolean)
    pub fn infer_test(&self, _test: &TestExpr) -> ShellType {
        ShellType::Boolean
    }

    /// Get collected diagnostics
    pub fn diagnostics(&self) -> &[TypeDiagnostic] {
        &self.diagnostics
    }

    /// Get the type context (for inspection/testing)
    pub fn context(&self) -> &TypeContext {
        &self.ctx
    }

    /// Consume a pending type annotation matching the given variable name
    fn consume_annotation(&mut self, name: &str) -> Option<ShellType> {
        let pos = self
            .pending_annotations
            .iter()
            .position(|a| a.name == name && !a.is_return && !a.is_param)?;
        Some(self.pending_annotations.remove(pos).shell_type)
    }

    /// Collect pending param/return annotations into a function signature
    fn collect_function_sig(&mut self) -> Option<FunctionSig> {
        let params: Vec<_> = self
            .pending_annotations
            .iter()
            .filter(|a| a.is_param)
            .map(|a| (a.name.clone(), a.shell_type.clone()))
            .collect();

        let return_type = self
            .pending_annotations
            .iter()
            .find(|a| a.is_return)
            .map(|a| a.shell_type.clone());

        if params.is_empty() && return_type.is_none() {
            return None;
        }

        // Remove consumed annotations
        self.pending_annotations
            .retain(|a| !a.is_param && !a.is_return);

        Some(FunctionSig {
            params,
            return_type,
        })
    }

    /// Handle declare/typeset/local with type flags
    fn check_declare(&mut self, args: &[BashExpr], _span: Span) {
        let mut current_type: Option<ShellType> = None;

        for arg in args {
            if let BashExpr::Literal(s) = arg {
                match s.as_str() {
                    "-i" => current_type = Some(ShellType::Integer),
                    "-a" => current_type = Some(ShellType::Array(Box::new(ShellType::String))),
                    "-A" => current_type = Some(ShellType::AssocArray {
                        key: Box::new(ShellType::String),
                        value: Box::new(ShellType::String),
                    }),
                    _ => {
                        // Check for name=value patterns
                        if let Some(eq_pos) = s.find('=') {
                            let var_name = &s[..eq_pos];
                            if let Some(ref ty) = current_type {
                                self.ctx.set_type(var_name, ty.clone());
                            }
                        } else if !s.starts_with('-') {
                            // Just a variable name without assignment
                            if let Some(ref ty) = current_type {
                                self.ctx.set_type(s, ty.clone());
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a type annotation from a comment string
///
/// Supported formats:
/// - `@type varname: int`
/// - `@type varname: str`
/// - `@type varname: path`
/// - `@type varname: bool`
/// - `@type varname: array`
/// - `@param name: int`
/// - `@returns: int`
pub fn parse_type_annotation(comment: &str) -> Option<TypeAnnotation> {
    let trimmed = comment.trim();

    // @type varname: type
    if let Some(rest) = trimmed.strip_prefix("@type ") {
        let (name, ty) = parse_name_type(rest)?;
        return Some(TypeAnnotation {
            name,
            shell_type: ty,
            is_return: false,
            is_param: false,
        });
    }

    // @param name: type
    if let Some(rest) = trimmed.strip_prefix("@param ") {
        let (name, ty) = parse_name_type(rest)?;
        return Some(TypeAnnotation {
            name,
            shell_type: ty,
            is_return: false,
            is_param: true,
        });
    }

    // @returns: type
    if let Some(rest) = trimmed.strip_prefix("@returns: ") {
        let ty = parse_type_name(rest.trim())?;
        return Some(TypeAnnotation {
            name: String::new(),
            shell_type: ty,
            is_return: true,
            is_param: false,
        });
    }

    None
}

/// Parse "name: type" from annotation text
fn parse_name_type(text: &str) -> Option<(String, ShellType)> {
    let parts: Vec<&str> = text.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }
    let name = parts[0].trim().to_string();
    let ty = parse_type_name(parts[1].trim())?;
    Some((name, ty))
}

/// Parse a type name string into a ShellType
pub fn parse_type_name(name: &str) -> Option<ShellType> {
    match name {
        "int" | "integer" => Some(ShellType::Integer),
        "str" | "string" => Some(ShellType::String),
        "bool" | "boolean" => Some(ShellType::Boolean),
        "path" => Some(ShellType::String), // Path is a string subtype for now
        "array" => Some(ShellType::Array(Box::new(ShellType::String))),
        "fd" => Some(ShellType::FileDescriptor),
        "exit_code" => Some(ShellType::ExitCode),
        _ => None,
    }
}

/// Check gradual compatibility — untyped is compatible with everything
fn is_gradual_compatible(expected: &ShellType, actual: &ShellType) -> bool {
    // Integer is compatible with String context (integers are valid strings)
    // But NOT the reverse — String→Integer should warn (not every string is a number)
    matches!(
        (expected, actual),
        (ShellType::String, ShellType::Integer)
    )
}

/// Generate a POSIX sh runtime guard for an integer-typed variable
pub fn generate_integer_guard(var_name: &str) -> String {
    format!(
        r#"case "${var}" in
    *[!0-9]*) echo "type error: {var} must be integer" >&2; exit 1 ;;
esac"#,
        var = var_name
    )
}

/// Generate a POSIX sh runtime guard for a path-typed variable
pub fn generate_path_guard(var_name: &str) -> String {
    format!(
        r#"case "${var}" in
    /*|./*|../*) ;;
    *) echo "type error: {var} must be a path" >&2; exit 1 ;;
esac"#,
        var = var_name
    )
}

/// Generate a POSIX sh runtime guard for a non-empty string
pub fn generate_nonempty_guard(var_name: &str) -> String {
    format!(
        r#"if [ -z "${var}" ]; then
    echo "type error: {var} must be non-empty string" >&2; exit 1
fi"#,
        var = var_name
    )
}

/// Mapping of annotation type names to guard generators
pub fn generate_guard_for_type(var_name: &str, ty: &ShellType) -> Option<String> {
    match ty {
        ShellType::Integer => Some(generate_integer_guard(var_name)),
        ShellType::String => Some(generate_nonempty_guard(var_name)),
        ShellType::Boolean => None, // No specific guard for booleans
        ShellType::Array(_) => None,
        ShellType::AssocArray { .. } => None,
        ShellType::FileDescriptor => None,
        ShellType::ExitCode => None,
        ShellType::Signal => None,
        ShellType::TypeVar(_) => None,
        ShellType::Union(_) => None,
    }
}

#[cfg(test)]
#[path = "type_check_tests.rs"]
mod tests;
