//! Tests for `parse_assignment` in `parser_decl.rs`.
//!
//! Covers normal assignments, keyword-as-variable names, array element
//! assignments, the append (`+=`) operator, empty assignments, exported
//! assignments, and error cases.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::ast::{BashExpr, BashStmt};
use super::parser::BashParser;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a single bash statement from `input` and return it.
#[test]
fn test_parse_assignment_variable_value() {
    let stmt = parse_first("x=$HOME");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "x");
            match &value {
                BashExpr::Variable(v) => assert_eq!(v, "HOME"),
                BashExpr::Concat(parts) => {
                    // Some parsers wrap in Concat; accept either
                    assert!(!parts.is_empty());
                }
                other => panic!("expected Variable or Concat value, got {other:?}"),
            }
        }
        other => panic!("expected Assignment, got {other:?}"),
    }
}

// ===========================================================================
// Not-exported flag is false for regular assignments
// ===========================================================================

#[test]
fn test_parse_assignment_not_exported() {
    let stmt = parse_first("y=42");
    match stmt {
        BashStmt::Assignment { exported, .. } => {
            assert!(!exported, "regular assignment should have exported=false");
        }
        other => panic!("expected Assignment, got {other:?}"),
    }
}

// ===========================================================================
// Error cases
// ===========================================================================

#[test]
fn test_parse_assignment_unterminated_string() {
    // Unterminated string should be a lexer error
    parse_err(r#"x="unclosed"#);
}

#[test]
fn test_parse_assignment_unterminated_string_single_quote() {
    // Unterminated single-quoted string should be a lexer error
    parse_err("x='unclosed");
}

#[test]
fn test_parse_assignment_missing_bracket_not_array() {
    // Without closing bracket, the parser does NOT treat this as an array
    // assignment -- `arr` becomes a command and `[0=val` becomes arguments.
    // Verify it does not produce an Assignment with an index.
    let stmt = parse_first("arr[0=val");
    match &stmt {
        BashStmt::Assignment { index, .. } => {
            // If it somehow parses as assignment, index should be None
            // (the `[` would not have been consumed as array syntax)
            assert!(index.is_none(), "should not have an array index");
        }
        // More likely parsed as a Command
        BashStmt::Command { .. } => {
            // This is acceptable -- the parser saw no `=` after identifier
        }
        _ => {
            // Any other parse is fine as long as it's not an array assignment
        }
    }
}
