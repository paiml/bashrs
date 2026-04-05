//! Tests for `parse_variable_expansion` and `parse_test_condition` in `parser_expr.rs`.
//!
//! Covers variable expansion patterns (simple, braced, default, assign-default,
//! alternate, error-if-unset, string-length, prefix/suffix removal, substitution,
//! special variables) and test condition patterns (file tests, string tests,
//! numeric comparisons, negation, double-bracket, combined conditions).

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::ast::{BashExpr, BashStmt, TestExpr};
use super::parser::BashParser;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Call `parse_variable_expansion` directly with the given content string.
/// The content is what would appear inside `${ }` (without the braces) or
/// what comes after a bare `$`.
#[test]
fn test_parse_test_double_bracket_file_test() {
    let expr = parse_condition("[[ -d /var/log ]]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::FileDirectory(_)),
        "expected FileDirectory from [[ ]], got: {test:?}"
    );
}

// ===========================================================================
// parse_test_condition: combined conditions with -a and -o
// ===========================================================================

#[test]
fn test_parse_test_combined_and() {
    let expr = parse_condition("[ -f /etc/passwd -a -r /etc/passwd ]");
    let test = unwrap_test(expr);
    match test {
        TestExpr::And(left, right) => {
            assert!(
                matches!(*left, TestExpr::FileExists(_)),
                "expected left FileExists, got: {left:?}"
            );
            assert!(
                matches!(*right, TestExpr::FileReadable(_)),
                "expected right FileReadable, got: {right:?}"
            );
        }
        other => panic!("expected And, got: {other:?}"),
    }
}

#[test]
fn test_parse_test_combined_or() {
    let expr = parse_condition("[ -f /a -o -f /b ]");
    let test = unwrap_test(expr);
    match test {
        TestExpr::Or(left, right) => {
            assert!(
                matches!(*left, TestExpr::FileExists(_)),
                "expected left FileExists, got: {left:?}"
            );
            assert!(
                matches!(*right, TestExpr::FileExists(_)),
                "expected right FileExists, got: {right:?}"
            );
        }
        other => panic!("expected Or, got: {other:?}"),
    }
}

// ===========================================================================
// parse_test_condition: double-bracket && and || inside [[ ]]
// ===========================================================================

#[test]
fn test_parse_test_double_bracket_and() {
    let expr = parse_condition("[[ -f /a && -d /b ]]");
    let test = unwrap_test(expr);
    match test {
        TestExpr::And(left, right) => {
            assert!(matches!(*left, TestExpr::FileExists(_)));
            assert!(matches!(*right, TestExpr::FileDirectory(_)));
        }
        other => panic!("expected And from [[ && ]], got: {other:?}"),
    }
}

#[test]
fn test_parse_test_double_bracket_or() {
    let expr = parse_condition("[[ -z \"$a\" || -z \"$b\" ]]");
    let test = unwrap_test(expr);
    match test {
        TestExpr::Or(left, right) => {
            assert!(matches!(*left, TestExpr::StringEmpty(_)));
            assert!(matches!(*right, TestExpr::StringEmpty(_)));
        }
        other => panic!("expected Or from [[ || ]], got: {other:?}"),
    }
}

// ===========================================================================
// parse_test_condition: compound tests across brackets ([ ] && [ ])
// ===========================================================================

#[test]
fn test_parse_test_compound_and_across_brackets() {
    let expr = parse_condition("[ -f /a ] && [ -f /b ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::And(_, _)),
        "expected And from compound, got: {test:?}"
    );
}

#[test]
fn test_parse_test_compound_or_across_brackets() {
    let expr = parse_condition("[ -f /a ] || [ -f /b ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::Or(_, _)),
        "expected Or from compound, got: {test:?}"
    );
}

// ===========================================================================
// parse_variable_expansion: edge cases
// ===========================================================================

#[test]
fn test_parse_var_expansion_empty_default() {
    // ${var:=} — assign empty default
    let result = expand("var:=");
    assert_eq!(
        result,
        BashExpr::AssignDefault {
            variable: "var".to_string(),
            default: Box::new(BashExpr::Literal(String::new())),
        }
    );
}

#[test]
fn test_parse_var_expansion_underscore_variable() {
    assert_eq!(
        expand("_my_var_123"),
        BashExpr::Variable("_my_var_123".to_string()),
    );
}

#[test]
fn test_parse_var_expansion_error_if_unset_empty_message() {
    let result = expand("var:?");
    assert_eq!(
        result,
        BashExpr::ErrorIfUnset {
            variable: "var".to_string(),
            message: Box::new(BashExpr::Literal(String::new())),
        }
    );
}

#[test]
fn test_parse_var_expansion_alternate_empty() {
    let result = expand("var:+");
    assert_eq!(
        result,
        BashExpr::AlternativeValue {
            variable: "var".to_string(),
            alternative: Box::new(BashExpr::Literal(String::new())),
        }
    );
}

// ===========================================================================
// parse_test_condition: edge cases — bare string as condition
// ===========================================================================

#[test]
fn test_parse_test_bare_string_becomes_string_nonempty() {
    // [ somestring ] — no operator, just a value → StringNonEmpty
    let expr = parse_condition("[ someword ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::StringNonEmpty(_)),
        "expected StringNonEmpty for bare word, got: {test:?}"
    );
}
