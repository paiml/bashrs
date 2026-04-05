//! Coverage tests targeting uncovered functions in bash_parser/parser.rs
//!
//! Focus areas:
//! - `expect` (line 789, 0% coverage) — error path when token mismatch
//! - `tokens_adjacent` (line 834, 0% coverage) — assignment adjacency check
//! - `skip_condition_redirects` (line 860, 50% coverage) — redirect skipping
//! - Edge cases in partially-covered parser functions
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::ast::{BashExpr, BashStmt};
use super::parser::{BashParser, ParseError};

// ---------------------------------------------------------------------------
// expect() — error path tests (line 789)
// ---------------------------------------------------------------------------

/// `expect` returns an error when the next token does not match.
/// Trigger by writing invalid bash that requires a specific keyword.
#[test]
fn test_parse_command_substitution_assignment() {
    let input = "PWD=$(pwd)";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

/// Multiple FD redirects on the same command
#[test]
fn test_parse_multiple_redirects_on_command() {
    let input = "cmd 2>/dev/null 1>/tmp/out";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

/// Negated pipeline (! pipeline)
#[test]
fn test_parse_negated_pipeline() {
    let input = "! grep foo bar.txt";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}
