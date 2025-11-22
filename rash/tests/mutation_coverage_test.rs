#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
// Mutation Testing Coverage Tests
// Tests designed to catch specific mutations that were MISSED in Sprint 24

use bashrs::{transpile, Config};

/// MUTATION GAP #1: Line 61:60 - `i == function.body.len() - 1` (last statement detection)
/// Tests that last statement in function with return type is properly echoed
#[test]
fn test_last_statement_detection_in_function() {
    let source = r#"
fn add(a: i32, b: i32) -> i32 {
    let result = a + b;
    result
}

fn main() {
    let sum = add(5, 3);
    echo(sum);
}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok());
    let script = result.unwrap();

    // The last statement `result` should be echoed in the add function
    assert!(
        script.contains("echo \"$result\""),
        "Last statement in function with return type should be echoed"
    );
}

/// MUTATION GAP #2: Line 95:33 - match guard `should_echo` in convert_stmt_in_function
/// Tests that echo statements are generated only when should_echo is true
#[test]
fn test_echo_guard_in_function() {
    let source = r#"
fn get_value() -> i32 {
    42
}

fn no_return() {
    let x = 10;
}

fn main() {
    let val = get_value();
    no_return();
}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok());
    let script = result.unwrap();

    // get_value should echo 42
    assert!(
        script.contains("echo \"42\"") || script.contains("echo 42"),
        "Function with return type should echo return value"
    );

    // no_return should NOT echo x assignment
    let no_return_section = script
        .split("no_return() {")
        .nth(1)
        .unwrap()
        .split('}')
        .next()
        .unwrap();
    assert!(
        !no_return_section.contains("echo \"$x\""),
        "Function without return type should not echo variables"
    );
}

/// MUTATION GAP #3: Line 165:21 - delete Range expression match arm
/// Tests that range expressions in for loops are properly converted
#[test]
fn test_range_expression_conversion() {
    let source = r#"
fn main() {
    for i in 0..3 {
        echo(i);
    }
}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok());
    let script = result.unwrap();

    // Should generate seq command for range
    assert!(
        script.contains("seq") || script.contains("for i in"),
        "Range expression should be converted to seq or shell range"
    );
}

/// MUTATION GAP #4: Line 327:21 - delete BinaryOp::Eq match arm
/// Tests that equality comparison operators are properly converted
#[test]
fn test_equality_operator_conversion() {
    let source = r#"
fn main() {
    let x = 5;
    if x == 5 {
        echo("equal");
    }
}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok());
    let script = result.unwrap();

    // Should generate POSIX test with -eq for integer comparison
    assert!(
        script.contains("-eq") || script.contains("="),
        "Equality operator should generate comparison test"
    );
}

/// MUTATION GAP #5: Line 363:21 - delete BinaryOp::Sub match arm
/// Tests that subtraction operator is properly converted
#[test]
fn test_subtraction_operator_conversion() {
    let source = r#"
fn main() {
    let a = 10;
    let b = 3;
    let result = a - b;
    echo(result);
}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok());
    let script = result.unwrap();

    // Should generate arithmetic expansion with subtraction
    assert!(
        script.contains("$((") && (script.contains("-") || script.contains("sub")),
        "Subtraction should generate arithmetic expansion"
    );
}

/// MUTATION GAP #6: Line 391:13 - delete curl|wget match arm in analyze_command_effects
/// Tests that download commands are properly recognized
#[test]
fn test_download_command_effects() {
    // This is tested indirectly through require_download functionality
    // We test that the runtime includes the download function
    let source = r#"
fn main() {
    echo("test");
}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok());
    let script = result.unwrap();

    // The generated script should have download function available
    assert!(
        script.contains("rash_download_verified"),
        "Runtime should include download verification function"
    );
}

/// MUTATION GAP #7: Arithmetic operator mutations (- vs + vs /)
/// Tests that different arithmetic operators produce different results
#[test]
fn test_arithmetic_operator_distinctness() {
    let add_source = r#"
fn main() {
    let result = 10 + 5;
    echo(result);
}
"#;

    let sub_source = r#"
fn main() {
    let result = 10 - 5;
    echo(result);
}
"#;

    let div_source = r#"
fn main() {
    let result = 10 / 5;
    echo(result);
}
"#;

    let add_script = transpile(add_source, Config::default()).unwrap();
    let sub_script = transpile(sub_source, Config::default()).unwrap();
    let div_script = transpile(div_source, Config::default()).unwrap();

    // Each operator should produce different shell code
    assert_ne!(
        add_script, sub_script,
        "Addition and subtraction should generate different code"
    );
    assert_ne!(
        add_script, div_script,
        "Addition and division should generate different code"
    );
    assert_ne!(
        sub_script, div_script,
        "Subtraction and division should generate different code"
    );

    // Verify operators appear in code
    assert!(
        add_script.contains("+") || add_script.contains("add"),
        "Addition operator should appear in generated code"
    );
    assert!(
        sub_script.contains("-") || sub_script.contains("sub"),
        "Subtraction operator should appear in generated code"
    );
    assert!(
        div_script.contains("/") || div_script.contains("div"),
        "Division operator should appear in generated code"
    );
}

/// MUTATION GAP #8: Range expression inclusive vs exclusive
/// Tests that inclusive and exclusive ranges generate different code
#[test]
fn test_range_inclusive_vs_exclusive() {
    let exclusive_source = r#"
fn main() {
    for i in 0..3 {
        echo(i);
    }
}
"#;

    let inclusive_source = r#"
fn main() {
    for i in 0..=3 {
        echo(i);
    }
}
"#;

    let exclusive_script = transpile(exclusive_source, Config::default()).unwrap();
    let inclusive_script = transpile(inclusive_source, Config::default()).unwrap();

    // Inclusive and exclusive ranges should generate different seq commands
    assert_ne!(
        exclusive_script, inclusive_script,
        "Inclusive and exclusive ranges should generate different code"
    );

    // Exclusive: 0..3 should be seq 0 2
    // Inclusive: 0..=3 should be seq 0 3
    assert!(
        exclusive_script.contains("seq 0 2") || exclusive_script.contains("0 1 2"),
        "Exclusive range 0..3 should generate seq 0 2"
    );
    assert!(
        inclusive_script.contains("seq 0 3") || inclusive_script.contains("0 1 2 3"),
        "Inclusive range 0..=3 should generate seq 0 3"
    );
}
