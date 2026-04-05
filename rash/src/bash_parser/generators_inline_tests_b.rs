//! Tests extracted from generators.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::bash_parser::generators::*;
use proptest::strategy::ValueTree;

#[test]
fn test_generate_test_expr_file_directory() {
    let expr = TestExpr::FileDirectory(BashExpr::Literal("/tmp".to_string()));
    let output = generate_test_expr(&expr);
    assert!(output.contains("-d /tmp"));
}

#[test]
fn test_generate_test_expr_string_empty() {
    let expr = TestExpr::StringEmpty(BashExpr::Variable("x".to_string()));
    let output = generate_test_expr(&expr);
    assert!(output.contains("-z"));
}

#[test]
fn test_generate_test_expr_string_non_empty() {
    let expr = TestExpr::StringNonEmpty(BashExpr::Variable("x".to_string()));
    let output = generate_test_expr(&expr);
    assert!(output.contains("-n"));
}

#[test]
fn test_generate_test_expr_and() {
    let expr = TestExpr::And(
        Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
        Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
    );
    let output = generate_test_expr(&expr);
    assert!(output.contains("&&"));
}

#[test]
fn test_generate_test_expr_or() {
    let expr = TestExpr::Or(
        Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
        Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
    );
    let output = generate_test_expr(&expr);
    assert!(output.contains("||"));
}

#[test]
fn test_generate_test_expr_not() {
    let expr = TestExpr::Not(Box::new(TestExpr::FileExists(BashExpr::Literal(
        "a".to_string(),
    ))));
    let output = generate_test_expr(&expr);
    assert!(output.contains("!"));
}

// ============== negate_condition tests ==============

#[test]
fn test_negate_condition_test() {
    let expr = BashExpr::Test(Box::new(TestExpr::IntGt(
        BashExpr::Variable("x".to_string()),
        BashExpr::Literal("5".to_string()),
    )));
    let output = negate_condition(&expr);
    assert!(output.contains("!"));
}

#[test]
fn test_negate_condition_other() {
    let expr = BashExpr::Variable("x".to_string());
    let output = negate_condition(&expr);
    assert!(output.starts_with("!"));
}

// ============== generate_test_condition tests ==============

#[test]
fn test_generate_test_condition_all_types() {
    // Test all test condition variants
    let tests = vec![
        (
            TestExpr::StringEq(
                BashExpr::Variable("a".to_string()),
                BashExpr::Literal("b".to_string()),
            ),
            "=",
        ),
        (
            TestExpr::StringNe(
                BashExpr::Variable("a".to_string()),
                BashExpr::Literal("b".to_string()),
            ),
            "!=",
        ),
        (
            TestExpr::IntEq(
                BashExpr::Variable("a".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            "-eq",
        ),
        (
            TestExpr::IntNe(
                BashExpr::Variable("a".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            "-ne",
        ),
        (
            TestExpr::IntLt(
                BashExpr::Variable("a".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            "-lt",
        ),
        (
            TestExpr::IntLe(
                BashExpr::Variable("a".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            "-le",
        ),
        (
            TestExpr::IntGt(
                BashExpr::Variable("a".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            "-gt",
        ),
        (
            TestExpr::IntGe(
                BashExpr::Variable("a".to_string()),
                BashExpr::Literal("1".to_string()),
            ),
            "-ge",
        ),
        (
            TestExpr::FileExists(BashExpr::Literal("f".to_string())),
            "-e",
        ),
        (
            TestExpr::FileReadable(BashExpr::Literal("f".to_string())),
            "-r",
        ),
        (
            TestExpr::FileWritable(BashExpr::Literal("f".to_string())),
            "-w",
        ),
        (
            TestExpr::FileExecutable(BashExpr::Literal("f".to_string())),
            "-x",
        ),
        (
            TestExpr::FileDirectory(BashExpr::Literal("f".to_string())),
            "-d",
        ),
        (
            TestExpr::StringEmpty(BashExpr::Variable("x".to_string())),
            "-z",
        ),
        (
            TestExpr::StringNonEmpty(BashExpr::Variable("x".to_string())),
            "-n",
        ),
    ];

    for (expr, expected) in tests {
        let output = generate_test_condition(&expr);
        assert!(
            output.contains(expected),
            "Expected '{}' in output: {}",
            expected,
            output
        );
    }
}

#[test]
fn test_generate_test_condition_and_or_not() {
    let and_expr = TestExpr::And(
        Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
        Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
    );
    let and_output = generate_test_condition(&and_expr);
    assert!(and_output.contains("&&"));

    let or_expr = TestExpr::Or(
        Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
        Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
    );
    let or_output = generate_test_condition(&or_expr);
    assert!(or_output.contains("||"));

    let not_expr = TestExpr::Not(Box::new(TestExpr::FileExists(BashExpr::Literal(
        "a".to_string(),
    ))));
    let not_output = generate_test_condition(&not_expr);
    assert!(not_output.contains("!"));
}

// ============== generate_condition tests ==============

#[test]
fn test_generate_condition_with_test() {
    let expr = BashExpr::Test(Box::new(TestExpr::FileExists(BashExpr::Literal(
        "/tmp".to_string(),
    ))));
    let output = generate_condition(&expr);
    assert!(output.contains("-e /tmp"));
}

#[test]
fn test_generate_condition_with_other() {
    let expr = BashExpr::Variable("x".to_string());
    let output = generate_condition(&expr);
    assert_eq!(output, "\"$x\"");
}

// ============== BASH_KEYWORDS tests ==============

#[test]
fn test_bash_keywords_contains_expected() {
    assert!(BASH_KEYWORDS.contains(&"if"));
    assert!(BASH_KEYWORDS.contains(&"then"));
    assert!(BASH_KEYWORDS.contains(&"else"));
    assert!(BASH_KEYWORDS.contains(&"fi"));
    assert!(BASH_KEYWORDS.contains(&"for"));
    assert!(BASH_KEYWORDS.contains(&"while"));
    assert!(BASH_KEYWORDS.contains(&"do"));
    assert!(BASH_KEYWORDS.contains(&"done"));
    assert!(BASH_KEYWORDS.contains(&"case"));
    assert!(BASH_KEYWORDS.contains(&"esac"));
}

// ============== Strategy function type tests ==============

#[test]
fn test_bash_string_generates_valid_output() {
    use proptest::test_runner::TestRunner;
    let strategy = bash_string();
    let mut runner = TestRunner::default();

    // Generate a few values to verify the strategy works
    for _ in 0..5 {
        let value = strategy.new_tree(&mut runner).unwrap().current();
        assert!(value.len() <= 20);
        // Valid characters only
        assert!(value
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == ' '));
    }
}

#[test]
fn test_bash_integer_generates_valid_range() {
    use proptest::test_runner::TestRunner;
    let strategy = bash_integer();
    let mut runner = TestRunner::default();

    for _ in 0..10 {
        let value = strategy.new_tree(&mut runner).unwrap().current();
        assert!(value >= -1000);
        assert!(value < 1000);
    }
}

#[test]
fn test_bash_variable_name_generates_valid() {
    use proptest::test_runner::TestRunner;
    let strategy = bash_variable_name();
    let mut runner = TestRunner::default();

    for _ in 0..5 {
        let value = strategy.new_tree(&mut runner).unwrap().current();
        assert!(!value.is_empty());
        // Should be one of the known variable names
        let valid_names = vec![
            "FOO", "BAR", "PATH", "HOME", "USER", "x", "y", "status", "result",
        ];
        assert!(valid_names.contains(&value.as_str()));
    }
}

#[test]
fn test_bash_test_expr_generates_valid() {
    use proptest::test_runner::TestRunner;
    let strategy = bash_test_expr();
    let mut runner = TestRunner::default();

    // Just verify it generates without panic
    for _ in 0..5 {
        let _value = strategy.new_tree(&mut runner).unwrap().current();
    }
}
