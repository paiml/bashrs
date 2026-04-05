//! Parser arithmetic inline tests — tokenizer, c-style for, and arithmetic expressions.
//!
//! Extracted from parser_core_tests.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::bash_parser::ast::*;
use crate::bash_parser::parser::*;
use crate::bash_parser::parser_arith::ArithToken;

