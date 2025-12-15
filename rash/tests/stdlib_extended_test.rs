#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
#![allow(clippy::expect_used)]
// Integration tests for extended stdlib functions (Sprint 25)
use bashrs::{transpile, Config};

// =============== String Functions Tests ===============

#[test]
fn test_string_replace_transpiles() {
    let source = r#"
fn main() {
    let text = "hello world";
    let result = string_replace(text, "world", "rust");
    echo(result);
}
"#;

    let config = Config::default();
    let result = transpile(source, config);
    assert!(result.is_ok());
    let script = result.unwrap();
    assert!(script.contains("rash_string_replace"));
}

#[test]
fn test_string_to_upper_transpiles() {
    let source = r#"
fn main() {
    let text = "hello";
    let result = string_to_upper(text);
    echo(result);
}
"#;

    let config = Config::default();
    let result = transpile(source, config);
    assert!(result.is_ok());
    let script = result.unwrap();
    assert!(script.contains("rash_string_to_upper"));
}

#[test]
fn test_string_to_lower_transpiles() {
    let source = r#"
fn main() {
    let text = "HELLO";
    let result = string_to_lower(text);
    echo(result);
}
"#;

    let config = Config::default();
    let result = transpile(source, config);
    assert!(result.is_ok());
    let script = result.unwrap();
    assert!(script.contains("rash_string_to_lower"));
}

// =============== File System Functions Tests ===============

#[test]
fn test_fs_is_file_transpiles() {
    let source = r#"
fn main() {
    let is_file = fs_is_file("/etc/passwd");
    if is_file {
        echo("yes");
    }
}
"#;

    let config = Config::default();
    let result = transpile(source, config);
    assert!(result.is_ok());
    let script = result.unwrap();
    assert!(script.contains("rash_fs_is_file"));
}

#[test]
fn test_fs_is_dir_transpiles() {
    let source = r#"
fn main() {
    let is_dir = fs_is_dir("/etc");
    if is_dir {
        echo("yes");
    }
}
"#;

    let config = Config::default();
    let result = transpile(source, config);
    assert!(result.is_ok());
    let script = result.unwrap();
    assert!(script.contains("rash_fs_is_dir"));
}

#[test]
fn test_fs_copy_transpiles() {
    let source = r#"
fn main() {
    let result = fs_copy("/tmp/src.txt", "/tmp/dst.txt");
    if result {
        echo("copied");
    }
}
"#;

    let config = Config::default();
    let result = transpile(source, config);
    assert!(result.is_ok());
    let script = result.unwrap();
    assert!(script.contains("rash_fs_copy"));
}

#[test]
fn test_fs_remove_transpiles() {
    let source = r#"
fn main() {
    let result = fs_remove("/tmp/test.txt");
    if result {
        echo("removed");
    }
}
"#;

    let config = Config::default();
    let result = transpile(source, config);
    assert!(result.is_ok());
    let script = result.unwrap();
    assert!(script.contains("rash_fs_remove"));
}

// =============== Combined Tests ===============

#[test]
fn test_multiple_new_stdlib_functions() {
    let source = r#"
fn main() {
    let text = "HELLO WORLD";
    let lower = string_to_lower(text);
    let replaced = string_replace(lower, "world", "rust");

    if fs_is_dir("/tmp") {
        fs_write_file("/tmp/test.txt", replaced);
        if fs_is_file("/tmp/test.txt") {
            fs_remove("/tmp/test.txt");
        }
    }
}
"#;

    let config = Config::default();
    let result = transpile(source, config);
    assert!(result.is_ok());
    let script = result.unwrap();

    // All new stdlib functions should be included
    assert!(script.contains("rash_string_to_lower"));
    assert!(script.contains("rash_string_replace"));
    assert!(script.contains("rash_fs_is_dir"));
    assert!(script.contains("rash_fs_write_file"));
    assert!(script.contains("rash_fs_is_file"));
    assert!(script.contains("rash_fs_remove"));
}
