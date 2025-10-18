//! # Makefile Parser and Purification
//!
//! This module provides parsing, AST representation, and purification for GNU Makefiles.
//!
//! ## Features
//!
//! - Parse GNU Makefiles to AST
//! - Purify Makefiles for determinism and idempotency
//! - Generate safe, reproducible Makefiles
//! - Property-based testing for all transformations
//!
//! ## Usage
//!
//! ```rust,ignore
//! use bashrs::make_parser::{parse_makefile, generate_purified_makefile};
//!
//! let makefile = r#"
//! .PHONY: test
//! test:
//!     cargo test
//! "#;
//!
//! let ast = parse_makefile(makefile).unwrap();
//! let purified = generate_purified_makefile(&ast);
//! ```

pub mod ast;
pub mod generators;
pub mod lexer;
pub mod parser;
pub mod purify;
pub mod semantic;

#[cfg(test)]
pub mod tests;

pub use ast::{MakeAst, MakeItem, MakeMetadata};
pub use parser::parse_makefile;
pub use generators::generate_purified_makefile;
pub use purify::{purify_makefile, PurificationResult};
