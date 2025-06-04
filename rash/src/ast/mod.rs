pub mod restricted;
pub mod visitor;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod visitor_tests;

pub use restricted::{Expr, Function, RestrictedAst, Stmt, Type};

use crate::models::{Error, Result};

/// Validate that an AST conforms to Rash restrictions
pub fn validate(ast: &RestrictedAst) -> Result<()> {
    ast.validate().map_err(Error::Validation)
}
