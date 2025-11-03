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
//! ## Safety Note
//! Makefile parser uses unwrap() and indexing on checked invariants (validated syntax, regex captures).
//! Positions and splits are validated before access for performance in hot paths.
//! Some parser functions are placeholders during development.
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]
#![allow(dead_code)]
#![allow(unused_variables)]
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
pub mod error;
pub mod generators;
pub mod lexer;
pub mod parser;
pub mod purify;
pub mod semantic;
pub mod test_generator;

#[cfg(test)]
pub mod tests;

pub use ast::{MakeAst, MakeItem, MakeMetadata};
pub use error::{MakeParseError, SourceLocation};
pub use generators::generate_purified_makefile;
pub use parser::{extract_function_calls, parse_makefile};
pub use purify::{purify_makefile, PurificationResult};
pub use test_generator::{MakefileTestGenerator, MakefileTestGeneratorOptions};
