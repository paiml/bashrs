//! PMAT-258 / GH-316: honest lowerings for stdlib methods that v7.2.0's
//! GH-305 fix (see `pmat257_gh305_gh306_tests.rs`) turned into transpile
//! errors instead of the old silent `"unknown"` placeholder.
//!
//! v7.2.0 exposed a real gap: 63 corpus entries used stdlib methods with no
//! lowering. This ticket implements the ones with an honest POSIX shell
//! spelling (`len()` on a string, `to_string()`) and confirms the ones that
//! stay unlowerable (`push_str()`, `push()`, `insert()`, `rev()`,
//! `unwrap_or()`/`unwrap_or_else()` on an arbitrary receiver) still fail the
//! transpile, naming the method, rather than emitting a no-op or a wrong
//! value.
#![allow(clippy::unwrap_used)]

use crate::{transpile, Config};

fn transpile_ok(src: &str) -> String {
    transpile(src, &Config::default()).unwrap_or_else(|e| panic!("transpile failed: {e}"))
}

fn transpile_err(src: &str) -> crate::models::Error {
    transpile(src, &Config::default()).expect_err("transpile should have failed")
}

fn run_shell(shell: &str, script: &str) -> (String, i32) {
    let output = std::process::Command::new(shell)
        .arg("-c")
        .arg(script)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {shell}: {e}"));
    (
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string(),
        output.status.code().unwrap_or(-1),
    )
}

// ===== len() on a string =====

#[test]
fn test_PMAT258_gh316_len_on_string_variable_prints_length() {
    let src = r#"
        fn main() {
            let s = "hello";
            let n = s.len();
            println!("{}", n);
        }
    "#;
    let script = transpile_ok(src);
    let (stdout, code) = run_shell("dash", &script);
    assert_eq!(code, 0, "script aborted under dash:\n{script}");
    assert_eq!(stdout, "5", "script:\n{script}");
    assert!(
        !script.contains("unknown"),
        "string .len() must not fall back to \"unknown\":\n{script}"
    );
}

#[test]
fn test_PMAT258_gh316_len_on_string_literal_receiver_prints_length() {
    let src = r#"
        fn main() {
            let n = "hello world".len();
            println!("{}", n);
        }
    "#;
    let script = transpile_ok(src);
    let (stdout, code) = run_shell("dash", &script);
    assert_eq!(code, 0, "script aborted under dash:\n{script}");
    assert_eq!(stdout, "11", "script:\n{script}");
}

#[test]
fn test_PMAT258_gh316_len_uses_shell_length_expansion_not_a_command() {
    // The POSIX spelling is `${#var}`, a parameter expansion -- not a
    // subprocess (`wc -c`, `expr length`, ...). Confirm the emitted script
    // actually contains the expansion syntax for the variable receiver case.
    let src = r#"
        fn main() {
            let s = "hi";
            let n = s.len();
            println!("{}", n);
        }
    "#;
    let script = transpile_ok(src);
    assert!(
        script.contains("${#s}"),
        "expected the `${{#s}}` length expansion in the emitted script:\n{script}"
    );
}

// ===== to_string() =====

#[test]
fn test_PMAT258_gh316_to_string_on_string_literal_is_the_value() {
    let src = r#"
        fn main() {
            let s = "hello".to_string();
            println!("{}", s);
        }
    "#;
    let script = transpile_ok(src);
    let (stdout, code) = run_shell("dash", &script);
    assert_eq!(code, 0, "script aborted under dash:\n{script}");
    assert_eq!(stdout, "hello", "script:\n{script}");
    assert!(
        !script.contains("unknown"),
        "to_string() must not fall back to \"unknown\":\n{script}"
    );
}

#[test]
fn test_PMAT258_gh316_to_string_on_variable_is_the_value() {
    let src = r#"
        fn main() {
            let s = "hello";
            let t = s.to_string();
            println!("{}", t);
        }
    "#;
    let script = transpile_ok(src);
    let (stdout, code) = run_shell("dash", &script);
    assert_eq!(code, 0, "script aborted under dash:\n{script}");
    assert_eq!(stdout, "hello", "script:\n{script}");
}

// ===== methods that stay unlowerable (error must name the method) =====

#[test]
fn test_PMAT258_gh316_push_str_has_no_lowering_is_an_error() {
    // push_str mutates its receiver -- making that mutation observable to
    // later statements would require the *statement*-level caller
    // (`convert_expr` in expr.rs) to emit an assignment instead of
    // discarding the value as `ShellIR::Noop`, which is out of scope for
    // this ticket (PMAT-258 scope is `expr_calls.rs` only). It must still
    // fail loudly rather than silently doing nothing.
    let src = r#"
        fn main() {
            let mut s = "hello".to_string();
            s.push_str(" world");
            println!("{}", s);
        }
    "#;
    let err = transpile_err(src);
    let msg = err.to_string();
    assert!(
        msg.contains("push_str"),
        "error should name the unlowerable method `push_str`, got: {msg}"
    );
}

#[test]
fn test_PMAT258_gh316_push_has_no_lowering_is_an_error() {
    let src = r#"
        fn main() {
            let mut s = "hello".to_string();
            s.push("!");
            println!("{}", s);
        }
    "#;
    let err = transpile_err(src);
    let msg = err.to_string();
    assert!(
        msg.contains("push"),
        "error should name the unlowerable method `push`, got: {msg}"
    );
}

#[test]
fn test_PMAT258_gh316_insert_has_no_lowering_is_an_error() {
    let src = r#"
        fn main() {
            let mut s = "hello".to_string();
            s.insert(0, "X");
            println!("{}", s);
        }
    "#;
    let err = transpile_err(src);
    let msg = err.to_string();
    assert!(
        msg.contains("insert"),
        "error should name the unlowerable method `insert`, got: {msg}"
    );
}

#[test]
fn test_PMAT258_gh316_rev_has_no_lowering_is_an_error() {
    // No POSIX builtin reverses a string deterministically and
    // shellcheck-clean; leave it erroring rather than guessing at a
    // spelling.
    let src = r#"
        fn main() {
            let s = "hello";
            let r = s.rev();
            println!("{}", r);
        }
    "#;
    let err = transpile_err(src);
    let msg = err.to_string();
    assert!(
        msg.contains("rev"),
        "error should name the unlowerable method `rev`, got: {msg}"
    );
    assert!(
        !msg.contains("unknown"),
        "error must not just restate the old \"unknown\" placeholder: {msg}"
    );
}

#[test]
fn test_PMAT258_gh316_unwrap_or_else_on_arbitrary_receiver_is_an_error() {
    // unwrap_or()/unwrap_or_else() need a decision about what an absent
    // value IS in shell -- deliberately not implemented here. The one
    // existing lowering (`args.get(N).unwrap_or(default)` /
    // `std::env::args().nth(N).unwrap_or(default)`) is untouched and keeps
    // working; every other receiver must still error.
    let src = r#"
        fn main() {
            let s = "hello";
            let r = s.unwrap_or_else(fallback);
            println!("{}", r);
        }
    "#;
    let err = transpile_err(src);
    let msg = err.to_string();
    assert!(
        msg.contains("unwrap_or_else"),
        "error should name the unlowerable method `unwrap_or_else`, got: {msg}"
    );
}

#[test]
fn test_PMAT258_gh316_unimplemented_method_fails_transpile_not_a_noop() {
    // GH-305 guard, restated for a method this ticket explicitly does not
    // implement: a method with no lowering must fail the transpile, never
    // silently succeed with a `:` no-op.
    let src = r#"
        fn main() {
            let s = "hello";
            s.push_str("!");
        }
    "#;
    let result = transpile(src, &Config::default());
    assert!(
        result.is_err(),
        "unlowerable method call must fail the transpile, not silently succeed: {result:?}"
    );
}
