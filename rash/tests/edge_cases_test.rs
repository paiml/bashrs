// Edge case tests discovered during book development
// These tests document known issues that need fixing

use bashrs::{transpile, Config};

/// TICKET-5007: Function return values should use command substitution
#[test]
fn test_edge_case_08_function_return_values() {
    let source = r#"
fn main() {
    let x = add(1, 2);
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
    let config = Config::default();
    let result = transpile(source, config).unwrap();

    // Function should emit echo for return value
    assert!(
        result.contains("echo $((a + b))") || result.contains("echo \"$((a + b))\""),
        "Function with return type should echo result"
    );

    // Call site should capture with command substitution
    assert!(
        result.contains("x=\"$(add 1 2)\"") || result.contains("x=$(add 1 2)"),
        "Function call result should be captured with $(...)"
    );

    // Should NOT contain "unknown"
    assert!(!result.contains("x=unknown"), "Should not assign unknown");
}

/// TICKET-5006: Arithmetic expressions should generate $((expr))
#[test]
fn test_edge_case_09_arithmetic_expressions() {
    let source = r#"
fn main() {
    let x = 1 + 2;
    let y = 10 - 3;
    let z = 4 * 5;
    let w = 20 / 4;
}
"#;
    let config = Config::default();
    let result = transpile(source, config).unwrap();

    // Verify arithmetic expansion syntax
    assert!(
        result.contains("$((1 + 2))"),
        "Should use arithmetic expansion for addition"
    );
    assert!(
        result.contains("$((10 - 3))"),
        "Should use arithmetic expansion for subtraction"
    );
    assert!(
        result.contains("$((4 * 5))"),
        "Should use arithmetic expansion for multiplication"
    );
    assert!(
        result.contains("$((20 / 4))"),
        "Should use arithmetic expansion for division"
    );

    // Should NOT contain string concatenation patterns
    assert!(
        !result.contains("\"${x}${y}\""),
        "Should not use string concatenation for arithmetic"
    );
}

#[test]
fn test_edge_case_03_negative_integers() {
    let source = r#"
fn main() {
    let x = -1;
    let y = -42;
    let z = 0;
}
"#;

    let config = Config::default();
    let result = transpile(source, config).unwrap();

    // Should NOT contain "unknown"
    assert!(
        !result.contains("unknown"),
        "Negative integers should not transpile to 'unknown'"
    );

    // Should contain actual negative numbers
    assert!(
        result.contains("x=-1") || result.contains("x='-1'"),
        "Should assign x=-1"
    );
    assert!(
        result.contains("y=-42") || result.contains("y='-42'"),
        "Should assign y=-42"
    );
    assert!(result.contains("z=0"), "Should assign z=0");
}

#[test]
fn test_edge_case_02_println_macro() {
    let source = r#"
fn main() {
    println!("Hello, World!");
}
"#;

    let config = Config::default();
    let result = transpile(source, config);

    // Should succeed (not return error)
    assert!(result.is_ok(), "println! should be supported");

    let script = result.unwrap();

    // Should generate printf or echo
    assert!(
        script.contains("printf") || script.contains("echo"),
        "println! should generate output command"
    );
    assert!(
        script.contains("Hello, World!"),
        "Should preserve the string"
    );
}

#[test]
fn test_edge_case_01_empty_function_bodies() {
    let source = r#"
fn main() {
    echo("test");
}

fn echo(msg: &str) {
    // Empty function - should generate : no-op
}
"#;

    let config = Config::default();
    let result = transpile(source, config).unwrap();

    // UPDATED (v6.17.1): Empty functions for known builtins/commands are NOT emitted
    // This allows the shell builtin to be used directly, which is the intended behavior
    // See: Issue fixing e2e test failures - empty stub functions should not shadow builtins
    assert!(
        !result.contains("echo() {"),
        "Empty builtin function should NOT generate function definition (uses builtin directly)"
    );

    // Should call the builtin echo command directly
    assert!(
        result.contains("echo ") || result.contains("echo test"),
        "Should call the builtin echo command"
    );
}

#[test]
fn test_edge_case_04_comparison_operators() {
    let source = r#"
fn main() {
    let x = 1;
    if x > 0 {
        let y = 2;
    }
}
"#;

    let config = Config::default();
    let result = transpile(source, config).unwrap();

    // Should NOT contain wrong string concatenation test
    assert!(
        !result.contains("test -n \"${x}0\""),
        "Comparison should not use string concatenation test"
    );

    // Should contain proper integer comparison
    assert!(
        result.contains("-gt")
            || result.contains("test") && result.contains("$x") && result.contains("0"),
        "Should use POSIX integer comparison (-gt, -lt, -eq, etc.)"
    );
}

/// TICKET-5008: For loops with range syntax (P2)
#[test]
fn test_edge_case_06_for_loops() {
    let source = r#"
fn main() {
    for i in 0..3 {
        let x = i;
    }
}
"#;
    let config = Config::default();
    let result = transpile(source, config).unwrap();

    // Should generate POSIX for loop with seq
    assert!(result.contains("for i in"), "Should have for loop");
    assert!(
        result.contains("seq") || result.contains("$(seq"),
        "Should use seq for range iteration"
    );
    assert!(
        result.contains("0") && result.contains("2"),
        "Range 0..3 should be 0 to 2 inclusive"
    );

    // Should NOT contain "unsupported"
    assert!(
        !result.to_lowercase().contains("unsupported"),
        "For loops should be supported"
    );
}

/// TICKET-5009: Match expressions (P2)
#[test]
fn test_edge_case_07_match_expressions() {
    let source = r#"
fn main() {
    let x = 2;
    match x {
        1 => {
            let y = 10;
        }
        2 => {
            let y = 20;
        }
        _ => {
            let y = 0;
        }
    }
}
"#;
    let config = Config::default();
    let result = transpile(source, config).unwrap();

    // Should generate POSIX case statement
    assert!(result.contains("case "), "Should have case statement");
    assert!(result.contains("esac"), "Should close with esac");

    // Should contain pattern matches
    assert!(result.contains("1)"), "Should match literal 1");
    assert!(result.contains("2)"), "Should match literal 2");
    assert!(result.contains("*)"), "Should have wildcard pattern");

    // Each case should end with ;;
    assert!(
        result.matches(";;").count() >= 3,
        "Each case should end with ;;"
    );

    // Should NOT contain "unsupported"
    assert!(
        !result.to_lowercase().contains("unsupported"),
        "Match expressions should be supported"
    );
}

/// TICKET-5010: Empty main() function (P3)
#[test]
fn test_edge_case_10_empty_main() {
    let source = r#"
fn main() {
}
"#;
    let config = Config::default();
    let result = transpile(source, config);

    // Should successfully transpile
    assert!(result.is_ok(), "Empty main() should transpile successfully");

    let script = result.unwrap();

    // Should be valid shell script (starts with shebang or has main)
    assert!(
        script.starts_with("#!/") || script.contains("main()"),
        "Should be a valid shell script"
    );

    // Should NOT contain errors or warnings in the main function
    // (stdlib functions may contain "ERROR" in their error handling code)
    let main_section = script.split("# Main script begins").last().unwrap_or("");
    assert!(
        !main_section.to_lowercase().contains("error:"),
        "Main script should not contain error messages"
    );
}

/// TICKET-5011: Integer overflow handling (P3)
#[test]
fn test_edge_case_11_integer_overflow() {
    let source = r#"
fn main() {
    let x = 2147483647;  // i32::MAX
    let y = -2147483648; // i32::MIN
}
"#;
    let config = Config::default();
    let result = transpile(source, config);

    // Should successfully transpile
    assert!(result.is_ok(), "Boundary integers should transpile");

    let script = result.unwrap();

    // Should contain the actual values (not "unknown")
    assert!(script.contains("2147483647"), "Should handle i32::MAX");
    assert!(
        script.contains("-2147483648") || script.contains("2147483648"),
        "Should handle i32::MIN"
    );

    // Should NOT contain "unknown"
    assert!(
        !script.contains("unknown"),
        "Should not transpile to unknown for boundary values"
    );
}
