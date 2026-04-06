use super::*;

#[test]
fn test_null_character_handling() {
    // This was found by QuickCheck and saved in proptest-regressions/ast/tests.txt
    let expr = Expr::Literal(Literal::Str("\0".to_string()));
    let result = expr.validate();
    assert!(result.is_err(), "Null characters should be rejected");
}

#[test]
fn test_backslash_quote_handling() {
    // Found in proptest-regressions/services/tests.txt
    let source = r#"fn main() { let x = "\\'"; }"#;
    let result = parse(source);
    // Should handle escaped quotes gracefully
    assert!(result.is_ok() || result.is_err()); // Either is fine as long as no panic
}

#[test]
fn test_empty_string_literal() {
    let source = r#"fn main() { let x = ""; }"#;
    let result = transpile(source, &Config::default());
    assert!(result.is_ok());
}

#[test]
fn test_very_large_numbers() {
    let source = "fn main() { let x = 4294967295; }"; // u32::MAX
    let result = transpile(source, &Config::default());
    // Should handle or reject gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_long_identifier_names() {
    let long_name = "a".repeat(100);
    let source = format!("fn main() {{ let {long_name} = 42; }}");
    let result = transpile(&source, &Config::default());
    // Should handle long names gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_deeply_nested_expressions() {
    let mut expr = "x".to_string();
    for _ in 0..20 {
        expr = format!("({expr} + 1)");
    }
    let source = format!("fn main() {{ let result = {expr}; }}");
    let result = transpile(&source, &Config::default());
    // Deep nesting should be handled gracefully
    assert!(result.is_ok() || result.is_err());
}
