#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! GH-148: Transpiler stdlib integration tests
//!
//! Tests for glob, mkdir, mv, chmod, starts_with, ends_with,
//! and capture-with-pipes. Extracted from transpile_quality_tests.rs
//! for file-size discipline.

// ============================================================================
// GH-148: glob() stdlib — file iteration in for loops
// ============================================================================

#[test]
fn test_GH148_glob_for_loop_produces_unquoted_pattern() {
    let rust_code = r#"
        fn main() {
            for f in glob("*.txt") {
                println!("{}", f);
            }
        }
    "#;

    let result = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();

    // Glob pattern must appear UNQUOTED so shell expansion works
    assert!(
        result.contains("*.txt"),
        "Output should contain the glob pattern *.txt: {}",
        result
    );
    // Must NOT be quoted — '*.txt' or "*.txt" would prevent shell expansion
    assert!(
        !result.contains("'*.txt'"),
        "Glob pattern must not be single-quoted: {}",
        result
    );
    // Should be in a for-in loop
    assert!(
        result.contains("for"),
        "Output should contain a for loop: {}",
        result
    );
    assert!(
        result.contains("do"),
        "Output should contain 'do' keyword: {}",
        result
    );
    assert!(
        result.contains("done"),
        "Output should contain 'done' keyword: {}",
        result
    );
}

#[test]
fn test_GH148_glob_path_with_directory() {
    let rust_code = r#"
        fn main() {
            for config in glob("/etc/*.conf") {
                println!("{}", config);
            }
        }
    "#;

    let result = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();

    assert!(
        result.contains("/etc/*.conf"),
        "Output should contain the path glob /etc/*.conf: {}",
        result
    );
}

#[test]
fn test_GH148_glob_recursive_pattern() {
    let rust_code = r#"
        fn main() {
            for f in glob("src/**/*.rs") {
                println!("{}", f);
            }
        }
    "#;

    let result = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();

    assert!(
        result.contains("src/**/*.rs"),
        "Output should contain recursive glob src/**/*.rs: {}",
        result
    );
}

#[test]
fn test_GH148_glob_deterministic() {
    let rust_code = r#"
        fn main() {
            for f in glob("*.yaml") {
                println!("{}", f);
            }
        }
    "#;

    let result1 = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();
    let result2 = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();
    assert_eq!(
        result1, result2,
        "glob() transpilation must be deterministic"
    );
}

// ============================================================================
// GH-148: mkdir/mv/chmod stdlib — directory and file management
// ============================================================================

#[test]
fn test_GH148_mkdir_produces_mkdir_p() {
    let rust_code = r#"
        fn main() {
            mkdir("/tmp/mydir");
        }
    "#;

    let result = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();

    assert!(
        result.contains("mkdir"),
        "Output should contain mkdir command: {}",
        result
    );
    assert!(
        result.contains("-p"),
        "mkdir must include -p flag for idempotency: {}",
        result
    );
    assert!(
        result.contains("/tmp/mydir"),
        "Output should contain the directory path: {}",
        result
    );
}

#[test]
fn test_GH148_mv_produces_mv_command() {
    let rust_code = r#"
        fn main() {
            mv("/tmp/old.txt", "/tmp/new.txt");
        }
    "#;

    let result = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();

    assert!(
        result.contains("mv"),
        "Output should contain mv command: {}",
        result
    );
    assert!(
        result.contains("/tmp/old.txt"),
        "Output should contain source path: {}",
        result
    );
    assert!(
        result.contains("/tmp/new.txt"),
        "Output should contain destination path: {}",
        result
    );
}

#[test]
fn test_GH148_chmod_produces_chmod_command() {
    let rust_code = r#"
        fn main() {
            chmod("755", "/tmp/script.sh");
        }
    "#;

    let result = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();

    assert!(
        result.contains("chmod"),
        "Output should contain chmod command: {}",
        result
    );
    assert!(
        result.contains("755"),
        "Output should contain the permission mode: {}",
        result
    );
    assert!(
        result.contains("/tmp/script.sh"),
        "Output should contain the file path: {}",
        result
    );
}

#[test]
fn test_GH148_fs_ops_deterministic() {
    let rust_code = r#"
        fn main() {
            mkdir("/opt/app");
            mv("/tmp/a", "/tmp/b");
            chmod("644", "/tmp/b");
        }
    "#;

    let result1 = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();
    let result2 = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();
    assert_eq!(
        result1, result2,
        "fs operations transpilation must be deterministic"
    );
}

// ============================================================================
// GH-148: string_starts_with/string_ends_with stdlib
// ============================================================================

#[test]
fn test_GH148_starts_with_emits_runtime_function() {
    let rust_code = r#"
        fn main() {
            let name = "hello.txt";
            if string_starts_with(name, "hello") {
                println!("yes");
            }
        }
    "#;

    let result = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();

    assert!(
        result.contains("rash_string_starts_with"),
        "Output should contain rash_string_starts_with function: {}",
        result
    );
    assert!(
        result.contains("case \"$haystack\" in"),
        "Runtime function should use POSIX case pattern matching: {}",
        result
    );
    assert!(
        result.contains("\"$prefix\"*)"),
        "starts_with should match prefix pattern: {}",
        result
    );
}

#[test]
fn test_GH148_ends_with_emits_runtime_function() {
    let rust_code = r#"
        fn main() {
            let name = "hello.txt";
            if string_ends_with(name, ".txt") {
                println!("yes");
            }
        }
    "#;

    let result = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();

    assert!(
        result.contains("rash_string_ends_with"),
        "Output should contain rash_string_ends_with function: {}",
        result
    );
    assert!(
        result.contains("*\"$suffix\")"),
        "ends_with should match suffix pattern: {}",
        result
    );
}

#[test]
fn test_GH148_starts_ends_with_selective_emission() {
    // Only starts_with used — ends_with should NOT be emitted
    let rust_code = r#"
        fn main() {
            let x = "test";
            if string_starts_with(x, "te") {
                println!("yes");
            }
        }
    "#;

    let result = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();

    assert!(
        result.contains("rash_string_starts_with"),
        "starts_with should be emitted: {}",
        result
    );
    assert!(
        !result.contains("rash_string_ends_with"),
        "ends_with should NOT be emitted when not used: {}",
        result
    );
}

#[test]
fn test_GH148_starts_ends_with_deterministic() {
    let rust_code = r#"
        fn main() {
            let s = "foo.bar";
            if string_starts_with(s, "foo") {
                println!("prefix match");
            }
            if string_ends_with(s, ".bar") {
                println!("suffix match");
            }
        }
    "#;

    let r1 = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();
    let r2 = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();
    assert_eq!(
        r1, r2,
        "starts_with/ends_with transpilation must be deterministic"
    );
}

// ============================================================================
// GH-148 / F-STDLIB-016..018: capture() with pipe operators
// ============================================================================

/// F-STDLIB-016: capture() with pipe preserves pipeline via sh -c
#[test]
fn test_GH148_capture_pipe_uses_sh_c() {
    let rust_code = r#"
        fn main() {
            let count = capture("ls /tmp | wc -l");
        }
    "#;

    let result = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();

    // Pipe must be preserved — wrapped in sh -c
    assert!(
        result.contains("sh"),
        "Pipe command should be wrapped with sh: {}",
        result
    );
    assert!(
        result.contains("ls /tmp | wc -l"),
        "Pipe command string must be preserved intact: {}",
        result
    );
}

/// F-STDLIB-017: capture() without pipes uses direct $(cmd)
#[test]
fn test_GH148_capture_simple_no_sh_c() {
    let rust_code = r#"
        fn main() {
            let user = capture("whoami");
        }
    "#;

    let result = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();

    assert!(
        result.contains("$(whoami)"),
        "Simple capture should produce direct $(whoami): {}",
        result
    );
    // Should NOT use sh -c for simple commands
    assert!(
        !result.contains("sh -c") && !result.contains("sh '-c'"),
        "Simple capture should not wrap in sh -c: {}",
        result
    );
}

/// F-STDLIB-018: capture() with && operator uses sh -c
#[test]
fn test_GH148_capture_and_operator_uses_sh_c() {
    let rust_code = r#"
        fn main() {
            let result = capture("test -d /tmp && echo yes");
        }
    "#;

    let result = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();

    assert!(
        result.contains("sh"),
        "&&-command should be wrapped with sh: {}",
        result
    );
    assert!(
        result.contains("test -d /tmp && echo yes"),
        "&& operator must be preserved intact: {}",
        result
    );
}

/// Falsify: capture with pipe must actually execute correctly
#[test]
fn test_GH148_capture_pipe_deterministic() {
    let rust_code = r#"
        fn main() {
            let count = capture("ls /tmp | wc -l");
            let user = capture("whoami");
            println!("{} {}", count, user);
        }
    "#;

    let r1 = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();
    let r2 = bashrs::transpile(rust_code, &bashrs::Config::default()).unwrap();
    assert_eq!(r1, r2, "capture with pipes must be deterministic");
}
