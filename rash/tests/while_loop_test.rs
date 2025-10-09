// While loop tests (TICKET-6001)
// RED phase: These tests should fail initially

use bashrs::{transpile, Config};

#[test]
fn test_while_loop_basic() {
    let source = r#"
fn main() {
    let i = 0;
    while i < 5 {
        let x = i + 1;
    }
}
"#;
    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok(), "Basic while loop should transpile");

    let script = result.unwrap();

    // Should generate POSIX while loop
    assert!(script.contains("while"), "Should contain 'while' keyword");
    assert!(script.contains("do"), "While loop should have 'do'");
    assert!(script.contains("done"), "While loop should end with 'done'");

    // Should have condition check
    assert!(
        script.contains("$i") && script.contains("5"),
        "Should contain loop variable and limit"
    );

    // Should NOT contain unsupported
    assert!(
        !script.to_lowercase().contains("unsupported"),
        "While loops should be supported"
    );
}

#[test]
fn test_while_loop_with_break() {
    let source = r#"
fn main() {
    let count = 0;
    while count < 10 {
        if count == 5 {
            break;
        }
        let x = count + 1;
    }
}
"#;
    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok(), "While loop with break should transpile");

    let script = result.unwrap();

    assert!(script.contains("while"), "Should have while loop");
    assert!(script.contains("break"), "Should have break statement");
}

#[test]
fn test_while_true_infinite_loop() {
    let source = r#"
fn main() {
    while true {
        println!("Running...");
        break;
    }
}
"#;
    let config = Config::default();
    let result = transpile(source, config);

    assert!(result.is_ok(), "Infinite while loop should transpile");

    let script = result.unwrap();

    // Should generate while true or while : syntax
    assert!(
        script.contains("while true") || script.contains("while :"),
        "Should have infinite loop construct"
    );
}
