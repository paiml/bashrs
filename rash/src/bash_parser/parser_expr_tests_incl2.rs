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
