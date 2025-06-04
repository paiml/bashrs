pub mod restricted;
pub mod visitor;

#[cfg(test)]
mod tests;

pub use restricted::{RestrictedAst, Function, Type, Expr, Stmt};

use crate::models::{Error, Result};

/// Validate that an AST conforms to Rash restrictions
pub fn validate(ast: &RestrictedAst) -> Result<()> {
    ast.validate().map_err(Error::Validation)
}