#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

// test_bash_parser_test_expressions.rs - EXTREME TDD for test expression parsing
// RED-GREEN-REFACTOR for v6.16.0

use bashrs::bash_parser::ast::TestExpr;
use bashrs::bash_parser::{BashExpr, BashParser, BashStmt};

// RED TEST 1: Parse [ -n "$VAR" ] (string non-empty test)
#[ignore = "RED phase - needs GREEN phase implementation"]
#[test]
fn test_parse_test_string_non_empty() {
    let bash = r#"[ -n "$VAR" ]"#;

    let mut parser = BashParser::new(bash).expect("Failed to create parser");
    let ast = parser.parse().expect("Failed to parse");

    assert_eq!(ast.statements.len(), 1);

    // Should parse as a test expression statement
    if let BashStmt::Command { name, args: _, .. } = &ast.statements[0] {
        assert_eq!(name, "[");
        // In bash, [ is actually a command, so this might be represented differently
    } else {
        panic!("Expected command statement");
    }
}

// RED TEST 2: Parse [ -z "$VAR" ] (string empty test)
#[ignore = "RED phase - needs GREEN phase implementation"]
#[test]
fn test_parse_test_string_empty() {
    let bash = r#"[ -z "$VAR" ]"#;

    let mut parser = BashParser::new(bash).expect("Failed to create parser");
    let ast = parser.parse().expect("Failed to parse");

    assert_eq!(ast.statements.len(), 1);
}

// RED TEST 3: Parse [ -f "$FILE" ] (file exists test)
#[ignore = "RED phase - needs GREEN phase implementation"]
#[test]
fn test_parse_test_file_exists() {
    let bash = r#"[ -f "$FILE" ]"#;

    let mut parser = BashParser::new(bash).expect("Failed to create parser");
    let ast = parser.parse().expect("Failed to parse");

    assert_eq!(ast.statements.len(), 1);
}

// RED TEST 4: Parse if with test expression
#[ignore = "RED phase - needs GREEN phase implementation"]
#[test]
fn test_parse_if_with_test_expression() {
    let bash = r#"
if [ -n "$VAR" ]; then
  echo "VAR is set"
fi
"#;

    let mut parser = BashParser::new(bash).expect("Failed to create parser");
    let ast = parser.parse().expect("Failed to parse");

    assert_eq!(ast.statements.len(), 1);

    // Should be an If statement
    if let BashStmt::If {
        condition,
        then_block,
        ..
    } = &ast.statements[0]
    {
        // Condition should be a test expression
        if let BashExpr::Test(test_expr) = condition {
            // Test expression should be StringNonEmpty
            assert!(matches!(**test_expr, TestExpr::StringNonEmpty(_)));
        } else {
            panic!("Expected test expression, got {:?}", condition);
        }

        assert!(!then_block.is_empty());
    } else {
        panic!("Expected if statement");
    }
}

// RED TEST 5: Parse [[ ]] test (bash extended test)
#[ignore = "RED phase - needs GREEN phase implementation"]
#[test]
fn test_parse_double_bracket_test() {
    let bash = r#"[[ -n "$VAR" ]]"#;

    let mut parser = BashParser::new(bash).expect("Failed to create parser");
    let ast = parser.parse().expect("Failed to parse");

    assert_eq!(ast.statements.len(), 1);
}

// RED TEST 6: Parse test with file operators
#[ignore = "RED phase - needs GREEN phase implementation"]
#[test]
fn test_parse_file_operators() {
    let test_cases = vec![
        ("[  -f /tmp/file ]", "file exists"),
        ("[ -d /tmp/dir ]", "directory exists"),
        ("[ -r /tmp/file ]", "file readable"),
        ("[ -w /tmp/file ]", "file writable"),
        ("[ -x /tmp/file ]", "file executable"),
        ("[ -e /tmp/file ]", "path exists"),
    ];

    for (bash, desc) in test_cases {
        let result = BashParser::new(bash).and_then(|mut p| p.parse());
        assert!(
            result.is_ok(),
            "Failed to parse {}: {:?}",
            desc,
            result.err()
        );
    }
}

// RED TEST 7: Parse test with string operators
#[ignore = "RED phase - needs GREEN phase implementation"]
#[test]
fn test_parse_string_operators() {
    let test_cases = vec![
        (r#"[ "$A" = "$B" ]"#, "string equal"),
        (r#"[ "$A" != "$B" ]"#, "string not equal"),
        (r#"[ -n "$A" ]"#, "string non-empty"),
        (r#"[ -z "$A" ]"#, "string empty"),
    ];

    for (bash, desc) in test_cases {
        let result = BashParser::new(bash).and_then(|mut p| p.parse());
        assert!(
            result.is_ok(),
            "Failed to parse {}: {:?}",
            desc,
            result.err()
        );
    }
}

// RED TEST 8: Parse test with integer operators
#[ignore = "RED phase - needs GREEN phase implementation"]
#[test]
fn test_parse_integer_operators() {
    let test_cases = vec![
        ("[ $A -eq $B ]", "equal"),
        ("[ $A -ne $B ]", "not equal"),
        ("[ $A -lt $B ]", "less than"),
        ("[ $A -le $B ]", "less or equal"),
        ("[ $A -gt $B ]", "greater than"),
        ("[ $A -ge $B ]", "greater or equal"),
    ];

    for (bash, desc) in test_cases {
        let result = BashParser::new(bash).and_then(|mut p| p.parse());
        assert!(
            result.is_ok(),
            "Failed to parse {}: {:?}",
            desc,
            result.err()
        );
    }
}

// RED TEST 9: Format test expression
#[ignore = "RED phase - needs GREEN phase implementation"]
#[test]
fn test_format_test_expression() {
    use bashrs::bash_quality::Formatter;

    let bash = r#"
if [ -n "$VAR" ]; then
  echo "set"
fi
"#;

    let mut formatter = Formatter::new();
    let result = formatter.format_source(bash);

    assert!(result.is_ok(), "Failed to format: {:?}", result.err());

    let formatted = result.unwrap();
    assert!(formatted.contains("[ -n"));
    assert!(formatted.contains("then"));
}

// RED TEST 10: Parse logical operators in tests
#[ignore = "RED phase - needs GREEN phase implementation"]
#[test]
fn test_parse_logical_operators() {
    let test_cases = vec![
        ("[ -n \"$A\" -a -n \"$B\" ]", "AND"),
        ("[ -n \"$A\" -o -n \"$B\" ]", "OR"),
        ("[ ! -f /tmp/file ]", "NOT"),
    ];

    for (bash, desc) in test_cases {
        let result = BashParser::new(bash).and_then(|mut p| p.parse());
        assert!(
            result.is_ok(),
            "Failed to parse {}: {:?}",
            desc,
            result.err()
        );
    }
}
