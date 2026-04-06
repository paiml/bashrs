//! Bash-to-Rash Parser Module
//!
//! This module implements Phase 1 of the bash-to-rash transpilation workflow:
//! - Formal parsing of bash scripts into AST
//! - Semantic analysis (variable scopes, command effects)
//! - Support for POSIX shell subset
//!
//! ## Design Principles
//! - Jidoka: Complete error handling for all parsing paths
//! - Genchi Genbutsu: Test against real shell scripts
//! - Hansei: Fix broken functionality before adding features
//! - Kaizen: Incremental verification with property tests
//!
//! ## Safety Note
//! Parser uses unwrap() and indexing on checked invariants (lookahead tokens, validated positions).
//! This is safe because positions are validated before access.
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::indexing_slicing)]

pub mod ast;
pub mod codegen; // Bash code generation (needed for purify command)
pub mod lexer;
pub mod parser;
pub mod parser_arith;
pub mod parser_cmd;
pub mod parser_control;
pub mod parser_decl;
pub mod parser_error_display;
pub mod parser_expr;
pub mod parser_pipeline;
pub mod semantic;

pub use ast::{BashAst, BashExpr, BashNode, BashStmt};
pub use parser::{BashParser, ParseError, ParseResult};
pub use semantic::{EffectTracker, ScopeInfo, SemanticAnalyzer};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod instrumentation_tests;

#[cfg(test)]
mod codegen_tests; // Comprehensive codegen coverage tests (26.5% → >90%)

#[cfg(test)]
pub mod generators; // Property-based test generators

#[cfg(test)]
#[path = "control_coverage_tests.rs"]
mod control_coverage_tests;

#[cfg(test)]
#[path = "expr_coverage_tests.rs"]
mod expr_coverage_tests;

#[cfg(test)]
#[path = "parser_coverage_tests.rs"]
mod parser_coverage_tests;

#[cfg(test)]
// FIXME(PMAT-238): #[path = "parser_arith_tests.rs"]
// FIXME(PMAT-238): mod parser_arith_tests;
#[cfg(test)]
#[path = "parser_decl_tests.rs"]
mod parser_decl_tests;

#[cfg(test)]
#[path = "parser_expr_tests.rs"]
mod parser_expr_tests;

#[cfg(test)]
#[path = "lexer_operator_tests.rs"]
mod lexer_operator_tests;

#[cfg(test)]
#[path = "parser_control_cov_tests.rs"]
mod parser_control_cov_tests;

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "parser_main_tests.rs"]
// FIXME(PMAT-238): mod parser_main_tests;
// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "parser_main_tests_p10.rs"]
// FIXME(PMAT-238): mod parser_main_tests_p10;
// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "parser_main_tests_p2.rs"]
// FIXME(PMAT-238): mod parser_main_tests_p2;
// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "parser_main_tests_p3.rs"]
// FIXME(PMAT-238): mod parser_main_tests_p3;
#[cfg(test)]
#[path = "parser_main_tests_p4.rs"]
mod parser_main_tests_p4;
#[cfg(test)]
#[path = "parser_main_tests_p5.rs"]
mod parser_main_tests_p5;
#[cfg(test)]
#[path = "parser_main_tests_p6.rs"]
mod parser_main_tests_p6;
// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "parser_main_tests_p7.rs"]
// FIXME(PMAT-238): mod parser_main_tests_p7;
// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "parser_main_tests_p8.rs"]
// FIXME(PMAT-238): mod parser_main_tests_p8;
// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "parser_main_tests_p9.rs"]
// FIXME(PMAT-238): mod parser_main_tests_p9;
