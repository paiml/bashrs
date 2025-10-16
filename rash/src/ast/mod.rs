//! # Abstract Syntax Tree (AST) Module
//!
//! This module provides the Abstract Syntax Tree representation for Rash,
//! a Rust-to-Shell transpiler. The AST is designed to represent a safe
//! subset of Rust that can be reliably transpiled to POSIX shell scripts.
//!
//! ## Overview
//!
//! The AST module consists of:
//! - **Restricted AST**: A limited subset of Rust syntax that ensures safe shell generation
//! - **Validation**: Compile-time checks for safety and correctness
//! - **Visitor Pattern**: For AST traversal and transformation
//!
//! ## Examples
//!
//! ### Creating a Simple AST
//!
//! ```rust
//! use bashrs::ast::{RestrictedAst, Function, Stmt, Expr, Type};
//! use bashrs::ast::restricted::Literal;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a simple "Hello, World!" function
//! let main_fn = Function {
//!     name: "main".to_string(),
//!     params: vec![],
//!     body: vec![
//!         Stmt::Expr(Expr::FunctionCall {
//!             name: "println".to_string(),
//!             args: vec![Expr::Literal(Literal::Str("Hello, World!".to_string()))],
//!         })
//!     ],
//!     return_type: Type::Void,
//! };
//!
//! let ast = RestrictedAst {
//!     functions: vec![main_fn],
//!     entry_point: "main".to_string(),
//! };
//!
//! // Validate the AST
//! assert!(ast.validate().is_ok());
//! # Ok(())
//! # }
//! ```
//!
//! ### Variable Usage
//!
//! ```rust
//! use bashrs::ast::{RestrictedAst, Function, Stmt, Expr, Type};
//! use bashrs::ast::restricted::Literal;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let main_fn = Function {
//!     name: "main".to_string(),
//!     params: vec![],
//!     body: vec![
//!         // Create a variable
//!         Stmt::Let {
//!             name: "user".to_string(),
//!             value: Expr::FunctionCall {
//!                 name: "env".to_string(),
//!                 args: vec![Expr::Literal(Literal::Str("USER".to_string()))],
//!             },
//!         },
//!         // Use the variable
//!         Stmt::Expr(Expr::FunctionCall {
//!             name: "echo".to_string(),
//!             args: vec![
//!                 Expr::Variable("user".to_string())
//!             ],
//!         })
//!     ],
//!     return_type: Type::Void,
//! };
//!
//! let ast = RestrictedAst {
//!     functions: vec![main_fn],
//!     entry_point: "main".to_string(),
//! };
//!
//! // Variables must be valid identifiers
//! assert!(ast.validate().is_ok());
//! # Ok(())
//! # }
//! ```
//!
//! ### Invalid AST Example
//!
//! ```rust
//! use bashrs::ast::{RestrictedAst, Function, Stmt, Expr, Type};
//! use bashrs::ast::restricted::{Literal, BinaryOp};
//!
//! # fn main() {
//! // Functions cannot have empty names
//! let invalid_fn = Function {
//!     name: "".to_string(), // Empty function name!
//!     params: vec![],
//!     body: vec![
//!         Stmt::Expr(Expr::Literal(Literal::Str("test".to_string())))
//!     ],
//!     return_type: Type::Void,
//! };
//!
//! let ast = RestrictedAst {
//!     functions: vec![invalid_fn],
//!     entry_point: "main".to_string(),
//! };
//!
//! // This will fail validation
//! assert!(ast.validate().is_err());
//! # }
//! ```

pub mod restricted;
pub mod visitor;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod visitor_tests;

#[cfg(test)]
mod restricted_validation_test;

#[cfg(test)]
mod restricted_test;

pub use restricted::{Expr, Function, RestrictedAst, Stmt, Type};

use crate::models::{Error, Result};

/// Validate that an AST conforms to Rash restrictions
///
/// # Examples
///
/// ```rust
/// use bashrs::ast::{validate, RestrictedAst, Function, Type};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let ast = RestrictedAst {
///     functions: vec![
///         Function {
///             name: "main".to_string(),
///             params: vec![],
///             body: vec![],
///             return_type: Type::Void,
///         }
///     ],
///     entry_point: "main".to_string(),
/// };
///
/// // Validate the AST
/// validate(&ast)?;
/// # Ok(())
/// # }
/// ```
pub fn validate(ast: &RestrictedAst) -> Result<()> {
    ast.validate().map_err(Error::Validation)
}
