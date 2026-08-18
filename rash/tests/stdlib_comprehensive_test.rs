#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use bashrs::{transpile, Config};

// =============== String Module Tests ===============

#[test]
fn test_string_contains_found() {
    let source = r#"
fn main() {
    if string_contains("hello world", "world") {
        echo("found");
    }
}
"#;

    let config = Config::default();
    let result = transpile(source, &config);
    assert!(result.is_ok());
    let script = result.unwrap();
    assert!(script.contains("rash_string_contains"));
}

#[test]
fn test_string_contains_not_found() {
    let source = r#"
fn main() {
    if string_contains("hello", "xyz") {
        echo("found");
    }
}
"#;

    let config = Config::default();
    let result = transpile(source, &config);
    assert!(result.is_ok());
}

#[test]
fn test_string_len() {
    let source = r#"
fn main() {
    let text = "hello";
    let length = string_len(text);
    echo(length);
}
"#;

    let config = Config::default();
    let result = transpile(source, &config);
    assert!(result.is_ok());
    let script = result.unwrap();
    assert!(script.contains("rash_string_len"));
}

// =============== File System Module Tests ===============

#[test]
fn test_fs_exists() {
    let source = r#"
fn main() {
    if fs_exists("/etc/passwd") {
        echo("exists");
    }
}
"#;

    let config = Config::default();
    let result = transpile(source, &config);
    assert!(result.is_ok());
    let script = result.unwrap();
    assert!(script.contains("rash_fs_exists"));
}

#[test]
fn test_fs_read_file() {
    let source = r#"
fn main() {
    let content = fs_read_file("/tmp/test.txt");
    echo(content);
}
"#;

    let config = Config::default();
    let result = transpile(source, &config);
    assert!(result.is_ok());
    let script = result.unwrap();
    assert!(script.contains("rash_fs_read_file"));
}

#[test]
fn test_fs_write_file() {
    let source = r#"
fn main() {
    fs_write_file("/tmp/output.txt", "Hello, World!");
}
"#;

    let config = Config::default();
    let result = transpile(source, &config);
    assert!(result.is_ok());
    let script = result.unwrap();
    assert!(script.contains("rash_fs_write_file"));
}

// =============== Integration Tests ===============

#[test]
fn test_multiple_stdlib_functions() {
    let source = r#"
fn main() {
    let text = "  hello  ";
    let trimmed = string_trim(text);
    let length = string_len(trimmed);

    if fs_exists("/tmp") {
        fs_write_file("/tmp/test.txt", trimmed);
    }
}
"#;

    let config = Config::default();
    let result = transpile(source, &config);
    assert!(result.is_ok());
    let script = result.unwrap();

    // All stdlib functions should be included
    assert!(script.contains("rash_string_trim"));
    assert!(script.contains("rash_string_len"));
    assert!(script.contains("rash_fs_exists"));
    assert!(script.contains("rash_fs_write_file"));
}
