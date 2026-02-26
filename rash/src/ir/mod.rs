//! Intermediate Representation (IR) module
//!
//! ## Safety Note
//! IR operations use fallible indexing with proper error handling.
//! Production code MUST NOT use unwrap() (Cloudflare-class defect prevention).
//!
//! ## Module structure
//! - `convert`: Core AST-to-IR entry point, shadow/loop helpers
//! - `convert_stmt`: Top-level statement conversion (`convert_stmt`, `convert_stmts`)
//! - `convert_fn`: Function-context statement conversion (`convert_stmt_in_function`)
//! - `expr`: Expression-to-ShellValue conversion (operators, literals, index)
//! - `expr_calls`: Function/method call expression converters
//! - `pattern`: Match/pattern and range handling
//! - `optimizations`: Constant folding, dead code elimination, IR transforms

pub mod dockerfile_ir;
pub mod effects;
pub mod shell_ir;

mod convert;
mod convert_fn;
mod convert_stmt;
mod expr;
mod expr_calls;
mod optimizations;
mod pattern;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod shell_ir_tests;

#[cfg(test)]
mod control_flow_tests;

#[cfg(test)]
mod convert_expr_tests;

#[cfg(test)]
mod convert_expr_tests2;

#[cfg(test)]
mod convert_expr_tests3;

#[cfg(test)]
#[path = "convert_coverage_tests.rs"]
mod convert_coverage_tests;

#[cfg(test)]
#[path = "binary_ops_coverage_tests.rs"]
mod binary_ops_coverage_tests;

pub use effects::{Effect, EffectSet};
pub use shell_ir::{Command, ShellExpression, ShellIR, ShellValue};

use crate::ast::RestrictedAst;
use crate::models::{Config, Result};

/// Convert AST to Shell IR
pub fn from_ast(ast: &RestrictedAst) -> Result<ShellIR> {
    let converter = IrConverter::new();
    converter.convert(ast)
}

/// Optimize Shell IR based on configuration
pub fn optimize(ir: ShellIR, config: &Config) -> Result<ShellIR> {
    if !config.optimize {
        return Ok(ir);
    }

    let mut optimized = ir;

    // Apply constant folding
    optimized = optimizations::constant_fold(optimized);

    // Apply dead code elimination
    optimized = optimizations::eliminate_dead_code(optimized);

    Ok(optimized)
}

struct IrConverter {
    /// Track array variables: name → element count
    /// Used to expand `for x in arr` into `for x in "$arr_0" "$arr_1" ...`
    arrays: std::cell::RefCell<std::collections::HashMap<String, usize>>,
    /// Track declared variables for shadow detection in loop bodies.
    /// When a `let x = ...` (declaration=true) appears inside a loop body
    /// and `x` is already in this set, it's a shadow that needs renaming.
    declared_vars: std::cell::RefCell<std::collections::HashSet<String>>,
}
