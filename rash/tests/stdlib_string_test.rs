#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use bashrs::{transpile, Config};

/// Test string_trim() function (stdlib without use statements for now)
#[test]
fn test_stdlib_string_trim_basic() {
    let source = r#"
fn main() {
    let text = "  hello world  ";
    let result = string_trim(text);
    echo(result);
}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

    assert!(
        result.is_ok(),
        "string_trim() should transpile successfully"
    );
    let script = result.unwrap();

    // Should contain the trim runtime function
    assert!(script.contains("rash_string_trim"));
    // Should call the function
    assert!(script.contains("$(rash_string_trim"));
}

#[test]
fn test_stdlib_string_trim_empty() {
    let source = r#"
fn main() {
    let text = "";
    let result = string_trim(text);
    echo(result);
}
"#;

    let config = Config::default();
    let result = transpile(source, &config);
    assert!(
        result.is_ok(),
        "string_trim() with empty string should work"
    );
}

#[test]
fn test_stdlib_string_trim_no_whitespace() {
    let source = r#"
fn main() {
    let text = "hello";
    let result = string_trim(text);
    echo(result);
}
"#;

    let config = Config::default();
    let result = transpile(source, &config);
    assert!(
        result.is_ok(),
        "string_trim() with no whitespace should work"
    );
}
