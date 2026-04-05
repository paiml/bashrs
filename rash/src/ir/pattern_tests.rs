//! Coverage tests for `rash/src/ir/pattern.rs`.
//!
//! Tests `convert_match_pattern`, `lower_let_match`, `convert_match_arm_for_let`,
//! `lower_let_if`, `has_range_patterns`, `convert_range_match`, `lower_let_if_expr`,
//! `lower_return_if_expr`, and related helpers.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]


#[cfg(test)]
#[path = "pattern_tests_tests_extracted.rs"]
mod tests_extracted;
