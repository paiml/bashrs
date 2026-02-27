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
fn expand(content: &str) -> BashExpr {
    let parser = BashParser::new("echo x").unwrap();
    parser.parse_variable_expansion(content).unwrap()
}

/// Parse a complete bash script and return its statements.
fn parse_script(script: &str) -> Vec<BashStmt> {
    let mut parser = BashParser::new(script).unwrap();
    parser.parse().unwrap().statements
}

/// Parse an if-statement script and extract the condition expression.
fn parse_condition(test_clause: &str) -> BashExpr {
    let script = format!("if {test_clause}; then\n  echo ok\nfi");
    let stmts = parse_script(&script);
    match &stmts[0] {
        BashStmt::If { condition, .. } => condition.clone(),
        other => panic!("expected If statement, got: {other:?}"),
    }
}

/// Extract the inner `TestExpr` from a `BashExpr::Test`.
fn unwrap_test(expr: BashExpr) -> TestExpr {
    match expr {
        BashExpr::Test(inner) => *inner,
        other => panic!("expected BashExpr::Test, got: {other:?}"),
    }
}

// ===========================================================================
// parse_variable_expansion: simple variable
// ===========================================================================

#[test]
fn test_parse_var_expansion_simple_variable() {
    assert_eq!(expand("myvar"), BashExpr::Variable("myvar".to_string()));
}

#[test]
fn test_parse_var_expansion_braced_variable() {
    // ${var} — after lexer strips braces, content is just "var"
    assert_eq!(expand("var"), BashExpr::Variable("var".to_string()));
}

// ===========================================================================
// parse_variable_expansion: default value ${var:-default}
// ===========================================================================

#[test]
fn test_parse_var_expansion_default_value() {
    let result = expand("var:-default");
    assert_eq!(
        result,
        BashExpr::DefaultValue {
            variable: "var".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        }
    );
}

#[test]
fn test_parse_var_expansion_default_value_empty() {
    let result = expand("var:-");
    assert_eq!(
        result,
        BashExpr::DefaultValue {
            variable: "var".to_string(),
            default: Box::new(BashExpr::Literal(String::new())),
        }
    );
}

// ===========================================================================
// parse_variable_expansion: assign default ${var:=default}
// ===========================================================================

#[test]
fn test_parse_var_expansion_assign_default() {
    let result = expand("var:=fallback");
    assert_eq!(
        result,
        BashExpr::AssignDefault {
            variable: "var".to_string(),
            default: Box::new(BashExpr::Literal("fallback".to_string())),
        }
    );
}

// ===========================================================================
// parse_variable_expansion: alternate value ${var:+alternate}
// ===========================================================================

#[test]
fn test_parse_var_expansion_alternate_value() {
    let result = expand("var:+alt");
    assert_eq!(
        result,
        BashExpr::AlternativeValue {
            variable: "var".to_string(),
            alternative: Box::new(BashExpr::Literal("alt".to_string())),
        }
    );
}

// ===========================================================================
// parse_variable_expansion: error if unset ${var:?error}
// ===========================================================================

#[test]
fn test_parse_var_expansion_error_if_unset() {
    let result = expand("var:?variable not set");
    assert_eq!(
        result,
        BashExpr::ErrorIfUnset {
            variable: "var".to_string(),
            message: Box::new(BashExpr::Literal("variable not set".to_string())),
        }
    );
}

// ===========================================================================
// parse_variable_expansion: string length ${#var}
// ===========================================================================

#[test]
fn test_parse_var_expansion_string_length() {
    let result = expand("#var");
    assert_eq!(
        result,
        BashExpr::StringLength {
            variable: "var".to_string(),
        }
    );
}

#[test]
fn test_parse_var_expansion_string_length_multichar() {
    let result = expand("#MY_LONG_VAR");
    assert_eq!(
        result,
        BashExpr::StringLength {
            variable: "MY_LONG_VAR".to_string(),
        }
    );
}

// ===========================================================================
// parse_variable_expansion: suffix removal ${var%pattern}, ${var%%pattern}
// ===========================================================================

#[test]
fn test_parse_var_expansion_remove_suffix() {
    let result = expand("filename%.txt");
    assert_eq!(
        result,
        BashExpr::RemoveSuffix {
            variable: "filename".to_string(),
            pattern: Box::new(BashExpr::Literal(".txt".to_string())),
        }
    );
}

#[test]
fn test_parse_var_expansion_remove_longest_suffix() {
    let result = expand("path%%/*");
    assert_eq!(
        result,
        BashExpr::RemoveLongestSuffix {
            variable: "path".to_string(),
            pattern: Box::new(BashExpr::Literal("/*".to_string())),
        }
    );
}

// ===========================================================================
// parse_variable_expansion: prefix removal ${var#pattern}, ${var##pattern}
// ===========================================================================

#[test]
fn test_parse_var_expansion_remove_prefix() {
    let result = expand("path#*/");
    assert_eq!(
        result,
        BashExpr::RemovePrefix {
            variable: "path".to_string(),
            pattern: Box::new(BashExpr::Literal("*/".to_string())),
        }
    );
}

#[test]
fn test_parse_var_expansion_remove_longest_prefix() {
    let result = expand("path##*/");
    assert_eq!(
        result,
        BashExpr::RemoveLongestPrefix {
            variable: "path".to_string(),
            pattern: Box::new(BashExpr::Literal("*/".to_string())),
        }
    );
}

// ===========================================================================
// parse_variable_expansion: substitution ${var/pattern/replacement}
// Note: The parser currently falls through to simple Variable for /
// patterns since there's no explicit handler for substitution in the code.
// These tests document the actual behavior.
// ===========================================================================

#[test]
fn test_parse_var_expansion_substitution_single() {
    // ${var/pattern/replacement} — parser doesn't have a dedicated handler,
    // so it falls through. The content contains / which isn't a recognized
    // expansion operator, so it becomes a plain Variable.
    let result = expand("var/old/new");
    // No substitution handler — treated as variable name
    assert_eq!(
        result,
        BashExpr::Variable("var/old/new".to_string()),
    );
}

#[test]
fn test_parse_var_expansion_global_substitution() {
    // ${var//pattern/replacement} — same as above, no handler
    let result = expand("var//old/new");
    assert_eq!(
        result,
        BashExpr::Variable("var//old/new".to_string()),
    );
}

// ===========================================================================
// parse_variable_expansion: special variables ($1, $@, $*, $#, $?, $$)
// These are produced by the lexer as Token::Variable("1"), etc.
// ===========================================================================

#[test]
fn test_parse_var_expansion_positional_param() {
    // $1 — lexer yields Variable("1"), parse_variable_expansion("1") → Variable
    assert_eq!(expand("1"), BashExpr::Variable("1".to_string()));
}

#[test]
fn test_parse_var_expansion_all_params_at() {
    // $@ — lexer yields Variable("@")
    assert_eq!(expand("@"), BashExpr::Variable("@".to_string()));
}

#[test]
fn test_parse_var_expansion_all_params_star() {
    // $* — lexer yields Variable("*"), but since * is not alphanumeric the lexer
    // may handle it differently. Test the expansion function directly.
    assert_eq!(expand("*"), BashExpr::Variable("*".to_string()));
}

#[test]
fn test_parse_var_expansion_param_count() {
    // $# — lexer yields Variable("#"), but parse_variable_expansion("#") sees
    // starts_with('#') with len==1, so it falls through to simple variable.
    assert_eq!(expand("#"), BashExpr::Variable("#".to_string()));
}

#[test]
fn test_parse_var_expansion_exit_status() {
    // $? — lexer yields Variable("?")
    assert_eq!(expand("?"), BashExpr::Variable("?".to_string()));
}

#[test]
fn test_parse_var_expansion_process_id() {
    // $$ — lexer yields Variable("$")
    assert_eq!(expand("$"), BashExpr::Variable("$".to_string()));
}

// ===========================================================================
// parse_variable_expansion: full-script integration (lexer + parser)
// ===========================================================================

#[test]
fn test_parse_var_expansion_in_echo_default() {
    let stmts = parse_script("echo ${NAME:-World}");
    match &stmts[0] {
        BashStmt::Command { args, .. } => {
            assert_eq!(
                args[0],
                BashExpr::DefaultValue {
                    variable: "NAME".to_string(),
                    default: Box::new(BashExpr::Literal("World".to_string())),
                }
            );
        }
        other => panic!("expected Command, got: {other:?}"),
    }
}

#[test]
fn test_parse_var_expansion_in_assignment_string_length() {
    let stmts = parse_script("LEN=${#STR}");
    match &stmts[0] {
        BashStmt::Assignment { value, .. } => {
            assert_eq!(
                *value,
                BashExpr::StringLength {
                    variable: "STR".to_string(),
                }
            );
        }
        other => panic!("expected Assignment, got: {other:?}"),
    }
}

// ===========================================================================
// parse_test_condition: file tests
// ===========================================================================

#[test]
fn test_parse_test_file_exists() {
    let expr = parse_condition("[ -f /tmp/file ]");
    let test = unwrap_test(expr);
    match test {
        TestExpr::FileExists(BashExpr::Literal(path)) => {
            assert!(path.contains("tmp") || path.contains("file"));
        }
        other => panic!("expected FileExists, got: {other:?}"),
    }
}

#[test]
fn test_parse_test_directory() {
    let expr = parse_condition("[ -d /tmp ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::FileDirectory(_)),
        "expected FileDirectory, got: {test:?}"
    );
}

#[test]
fn test_parse_test_readable() {
    let expr = parse_condition("[ -r /etc/passwd ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::FileReadable(_)),
        "expected FileReadable, got: {test:?}"
    );
}

#[test]
fn test_parse_test_writable() {
    let expr = parse_condition("[ -w /tmp/out ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::FileWritable(_)),
        "expected FileWritable, got: {test:?}"
    );
}

#[test]
fn test_parse_test_executable() {
    let expr = parse_condition("[ -x /usr/bin/env ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::FileExecutable(_)),
        "expected FileExecutable, got: {test:?}"
    );
}

// ===========================================================================
// parse_test_condition: string tests
// ===========================================================================

#[test]
fn test_parse_test_string_empty() {
    let expr = parse_condition("[ -z \"$var\" ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::StringEmpty(_)),
        "expected StringEmpty, got: {test:?}"
    );
}

#[test]
fn test_parse_test_string_nonempty() {
    let expr = parse_condition("[ -n \"$var\" ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::StringNonEmpty(_)),
        "expected StringNonEmpty, got: {test:?}"
    );
}

// ===========================================================================
// parse_test_condition: string comparison
// ===========================================================================

#[test]
fn test_parse_test_string_equality() {
    let expr = parse_condition("[ \"$a\" = \"$b\" ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::StringEq(_, _)),
        "expected StringEq, got: {test:?}"
    );
}

#[test]
fn test_parse_test_string_inequality() {
    let expr = parse_condition("[ \"$a\" != \"$b\" ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::StringNe(_, _)),
        "expected StringNe, got: {test:?}"
    );
}

// ===========================================================================
// parse_test_condition: integer comparison
// ===========================================================================

#[test]
fn test_parse_test_int_eq() {
    let expr = parse_condition("[ \"$x\" -eq \"$y\" ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::IntEq(_, _)),
        "expected IntEq, got: {test:?}"
    );
}

#[test]
fn test_parse_test_int_ne() {
    let expr = parse_condition("[ \"$a\" -ne 0 ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::IntNe(_, _)),
        "expected IntNe, got: {test:?}"
    );
}

#[test]
fn test_parse_test_int_gt() {
    let expr = parse_condition("[ \"$count\" -gt 10 ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::IntGt(_, _)),
        "expected IntGt, got: {test:?}"
    );
}

#[test]
fn test_parse_test_int_lt() {
    let expr = parse_condition("[ \"$x\" -lt 5 ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::IntLt(_, _)),
        "expected IntLt, got: {test:?}"
    );
}

#[test]
fn test_parse_test_int_ge() {
    let expr = parse_condition("[ \"$x\" -ge 1 ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::IntGe(_, _)),
        "expected IntGe, got: {test:?}"
    );
}

#[test]
fn test_parse_test_int_le() {
    let expr = parse_condition("[ \"$x\" -le 100 ]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::IntLe(_, _)),
        "expected IntLe, got: {test:?}"
    );
}

// ===========================================================================
// parse_test_condition: negation
// ===========================================================================

#[test]
fn test_parse_test_negation() {
    let expr = parse_condition("[ ! -f /tmp/lock ]");
    let test = unwrap_test(expr);
    match test {
        TestExpr::Not(inner) => {
            assert!(
                matches!(*inner, TestExpr::FileExists(_)),
                "expected Not(FileExists), got inner: {inner:?}"
            );
        }
        other => panic!("expected Not, got: {other:?}"),
    }
}

// ===========================================================================
// parse_test_condition: double-bracket [[ ]]
// ===========================================================================

#[test]
fn test_parse_test_double_bracket_string_eq() {
    let expr = parse_condition("[[ \"$a\" = \"$b\" ]]");
    let test = unwrap_test(expr);
    assert!(
        matches!(test, TestExpr::StringEq(_, _)),
        "expected StringEq from [[ ]], got: {test:?}"
    );
}

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
