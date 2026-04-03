#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]
//! Provable Contract Tests: transpiler-core-v1.yaml
//!
//! Each test attempts to FALSIFY a contract claim from
//! provable-contracts/contracts/transpiler-core-v1.yaml.
//! A passing test means the claim survived falsification.
//!
//! Reference: GH-183 (KZ-11: Missing provable contracts)

// ============================================================================
// F-CORE-001 / F-CORE-002: Determinism
// ============================================================================

#[test]
fn falsify_CORE_001_determinism_simple() {
    let input = r#"
        fn main() {
            let x = 42;
            println!("{}", x);
        }
    "#;

    let r1 = bashrs::transpile(input, &bashrs::Config::default()).unwrap();
    let r2 = bashrs::transpile(input, &bashrs::Config::default()).unwrap();
    assert_eq!(r1, r2, "F-CORE-001: transpile(input) must equal transpile(input)");
}

#[test]
fn falsify_CORE_002_determinism_complex_stdlib() {
    let input = r#"
        fn main() {
            let user = capture("whoami");
            mkdir("/tmp/test");
            for f in glob("*.txt") {
                println!("{}", f);
            }
            if string_starts_with(user, "root") {
                exit(1);
            }
        }
    "#;

    let r1 = bashrs::transpile(input, &bashrs::Config::default()).unwrap();
    let r2 = bashrs::transpile(input, &bashrs::Config::default()).unwrap();
    assert_eq!(
        r1, r2,
        "F-CORE-002: stdlib calls must not introduce non-determinism"
    );
}

// ============================================================================
// F-CORE-003 / F-CORE-004 / F-CORE-005: POSIX Structure
// ============================================================================

#[test]
fn falsify_CORE_003_shebang() {
    let input = r#"fn main() { let x = 1; }"#;
    let output = bashrs::transpile(input, &bashrs::Config::default()).unwrap();
    assert!(
        output.starts_with("#!/bin/sh"),
        "F-CORE-003: output must start with #!/bin/sh shebang, got: {}",
        output.lines().next().unwrap_or("")
    );
}

#[test]
fn falsify_CORE_004_set_euf() {
    let input = r#"fn main() { let x = 1; }"#;
    let output = bashrs::transpile(input, &bashrs::Config::default()).unwrap();
    assert!(
        output.contains("set -euf"),
        "F-CORE-004: output must contain 'set -euf' safety flags"
    );
}

#[test]
fn falsify_CORE_005_main_wrapper() {
    let input = r#"fn main() { let x = 1; }"#;
    let output = bashrs::transpile(input, &bashrs::Config::default()).unwrap();
    assert!(
        output.contains("main()"),
        "F-CORE-005: output must wrap code in main() function"
    );
    assert!(
        output.contains("main \"$@\""),
        "F-CORE-005: output must invoke main with \"$@\""
    );
}

// ============================================================================
// F-CORE-006: Variable Safety
// ============================================================================

#[test]
fn falsify_CORE_006_variable_quoting() {
    let input = r#"
        fn main() {
            let name = "hello world";
            println!("{}", name);
        }
    "#;
    let output = bashrs::transpile(input, &bashrs::Config::default()).unwrap();

    // Variable must be quoted: "$name" not bare $name
    assert!(
        output.contains("\"$name\"") || output.contains("\"${name}\""),
        "F-CORE-006: variable expansion must be double-quoted, got:\n{}",
        output
    );
}

// ============================================================================
// F-CORE-007 / F-CORE-008: Selective Stdlib Emission
// ============================================================================

#[test]
fn falsify_CORE_007_unused_stdlib_not_emitted() {
    let input = r#"
        fn main() {
            let x = 42;
            println!("{}", x);
        }
    "#;
    let output = bashrs::transpile(input, &bashrs::Config::default()).unwrap();

    assert!(
        !output.contains("rash_string_"),
        "F-CORE-007: unused string stdlib must not be emitted"
    );
    assert!(
        !output.contains("rash_fs_"),
        "F-CORE-007: unused fs stdlib must not be emitted"
    );
}

#[test]
fn falsify_CORE_008_used_stdlib_is_emitted() {
    let input = r#"
        fn main() {
            if string_contains("hello world", "hello") {
                println!("found");
            }
        }
    "#;
    let output = bashrs::transpile(input, &bashrs::Config::default()).unwrap();

    assert!(
        output.contains("rash_string_contains()"),
        "F-CORE-008: used stdlib function must be emitted as shell function definition"
    );
}

// ============================================================================
// F-CORE-009: Idempotent Operations
// ============================================================================

#[test]
fn falsify_CORE_009_mkdir_uses_p_flag() {
    let input = r#"
        fn main() {
            mkdir("/tmp/test");
        }
    "#;
    let output = bashrs::transpile(input, &bashrs::Config::default()).unwrap();

    assert!(
        output.contains("mkdir") && output.contains("-p"),
        "F-CORE-009: mkdir must always use -p flag for idempotency"
    );
}

// ============================================================================
// F-CORE-010 / F-CORE-011: Pipe Safety
// ============================================================================

#[test]
fn falsify_CORE_010_capture_pipe_preserves_pipeline() {
    let input = r#"
        fn main() {
            let c = capture("ls | wc -l");
        }
    "#;
    let output = bashrs::transpile(input, &bashrs::Config::default()).unwrap();

    assert!(
        output.contains("ls | wc -l"),
        "F-CORE-010: pipe operator must be preserved, not escaped as literal"
    );
}

#[test]
fn falsify_CORE_011_capture_simple_direct_subst() {
    let input = r#"
        fn main() {
            let u = capture("whoami");
        }
    "#;
    let output = bashrs::transpile(input, &bashrs::Config::default()).unwrap();

    assert!(
        output.contains("$(whoami)"),
        "F-CORE-011: simple capture must use direct $(cmd) form, got:\n{}",
        output
    );
}
