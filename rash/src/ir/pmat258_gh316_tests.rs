//! PMAT-258 / GH-316: honest lowerings for stdlib methods that v7.2.0's
//! GH-305 fix (see `pmat257_gh305_gh306_tests.rs`) turned into transpile
//! errors instead of the old silent `"unknown"` placeholder.
//!
//! v7.2.0 exposed a real gap: 63 corpus entries used stdlib methods with no
//! lowering. This ticket implements the ones with an honest POSIX shell
//! spelling (`len()` on a string, `to_string()`, and -- second half, GH-316,
//! decided 3-0 by blind quorum -- `unwrap_or()`/`unwrap_or_else()` on a bare
//! variable, lowered to the *unset-only* `${x-d}`) and confirms the ones
//! that stay unlowerable (`push_str()`, `push()`, `insert()`, `rev()`,
//! `unwrap_or()`/`unwrap_or_else()` with a default that has no honest
//! no-side-effect spelling) still fail the transpile, naming the method,
//! rather than emitting a no-op or a wrong value.
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

/// Like `run_shell`, but writes `script` to a file inside a `TempDir` first
/// and runs `<shell> <path>` -- exercising the script as a real file on
/// disk, per PMAT-258's brief, rather than as inline `-c` text.
fn run_shell_file(shell: &str, script: &str) -> (String, i32) {
    let dir = tempfile::TempDir::new().unwrap_or_else(|e| panic!("failed to create tempdir: {e}"));
    let script_path = dir.path().join("script.sh");
    std::fs::write(&script_path, script).unwrap_or_else(|e| panic!("failed to write script: {e}"));
    let output = std::process::Command::new(shell)
        .arg(&script_path)
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

// ===== unwrap_or / unwrap_or_else on a bare variable: `${x-d}`, not `${x:-d}` =====
//
// PMAT-258 / GH-316 second half, decided 3-0 by blind quorum: POSIX shell
// has no Option type -- a variable is unset, set-and-empty, or set. Rust's
// `unwrap_or`/`unwrap_or_else` substitute the default only for `None`;
// `Some("")` is a present value. The faithful lowering is therefore the
// *unset-only* `${x-d}`, never the set-or-empty `${x:-d}`.

#[test]
fn test_PMAT258_unwrap_or_on_unset_variable_yields_default() {
    // `missing` is never declared anywhere -- the restricted AST does not
    // distinguish "declared but unset at runtime" from "never declared" (a
    // shell variable can be neither, since Rust requires every binding to
    // be initialized), so an undeclared identifier is how this suite
    // simulates a genuinely unset shell variable.
    let src = r#"
        fn main() {
            let y = missing.unwrap_or("fallback");
            println!("{}", y);
        }
    "#;
    let script = transpile_ok(src);
    assert!(
        !script.contains("unknown"),
        "unwrap_or must not fall back to \"unknown\":\n{script}"
    );

    let (stdout, code) = run_shell_file("dash", &script);
    assert_eq!(code, 0, "script aborted under dash:\n{script}");
    assert_eq!(
        stdout, "fallback",
        "an unset variable must take the unwrap_or default:\n{script}"
    );
}

#[test]
fn test_PMAT258_unwrap_or_on_set_but_empty_variable_keeps_empty_value() {
    // THE distinguishing test: `x` is declared and set to the empty string
    // -- a *present* value in Rust's Option model. `${x-fallback}` (unset
    // only) must keep the empty value; `${x:-fallback}` (unset OR empty)
    // would wrongly replace it. This must fail if the lowering is ever
    // switched from `-` to `:-`.
    let src = r#"
        fn main() {
            let x = "";
            let y = x.unwrap_or("fallback");
            println!("[{}]", y);
        }
    "#;
    let script = transpile_ok(src);
    assert!(
        script.contains("${x-"),
        "expected the unset-only `${{x-...}}` expansion, not `${{x:-...}}`:\n{script}"
    );
    assert!(
        !script.contains("${x:-"),
        "must not use the set-or-empty `${{x:-...}}` form -- Some(\"\") is a present value:\n{script}"
    );

    let (stdout, code) = run_shell_file("dash", &script);
    assert_eq!(code, 0, "script aborted under dash:\n{script}");
    assert_eq!(
        stdout, "[]",
        "a set-but-empty variable must keep its empty value, not take the default:\n{script}"
    );
}

#[test]
fn test_PMAT258_unwrap_or_else_with_simple_closure_body_lowers() {
    let src = r#"
        fn main() {
            let y = missing.unwrap_or_else(|| "fallback");
            println!("{}", y);
        }
    "#;
    let script = transpile_ok(src);
    assert!(
        !script.contains("unknown"),
        "unwrap_or_else with a simple closure body must not fall back to \"unknown\":\n{script}"
    );

    let (stdout, code) = run_shell_file("dash", &script);
    assert_eq!(code, 0, "script aborted under dash:\n{script}");
    assert_eq!(
        stdout, "fallback",
        "an unset variable must take the unwrap_or_else default:\n{script}"
    );
}

#[test]
fn test_PMAT258_unwrap_or_else_with_function_call_body_is_an_error() {
    // A closure that CALLS a function has a side effect (or at least an
    // effect this converter cannot inline) and must not be silently turned
    // into a value -- keep erroring, naming the method.
    let src = r#"
        fn main() {
            let s = "hello";
            let r = s.unwrap_or_else(|| compute_fallback());
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
fn test_PMAT258_unwrap_on_arbitrary_receiver_still_errors() {
    // A companion for an unrelated, still-unlowerable method: this ticket
    // only implements `unwrap_or`/`unwrap_or_else` -- plain `.unwrap()` on a
    // receiver that isn't the `std::env::args().nth(N)` special case must
    // keep failing loudly rather than emitting a wrong value.
    let src = r#"
        fn main() {
            let s = "hello";
            let r = s.unwrap();
            println!("{}", r);
        }
    "#;
    let err = transpile_err(src);
    let msg = err.to_string();
    assert!(
        msg.contains("unwrap"),
        "error should name the unlowerable method `unwrap`, got: {msg}"
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
