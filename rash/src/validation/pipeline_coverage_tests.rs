//! Coverage tests for `validate_expr_in_exec_context` exercised via the public API.
//!
//! Since `validate_expr_in_exec_context` is private, we exercise it by calling
//! `validate_expr` on a `FunctionCall { name: "exec", args: [...] }` expression,
//! which internally dispatches to `validate_expr_in_exec_context` for each arg.
//! We also test via `validate_ast` and `validate_ir`.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

#[cfg(test)]
#[path = "pipeline_coverage_tests_tests_strict_pipel.rs"]
mod tests_extracted;
