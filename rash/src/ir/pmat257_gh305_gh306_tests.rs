//! PMAT-257 / GH-305 / GH-306: an unlowerable method call must fail the
//! transpile with an error naming the method, not silently emit a no-op
//! (`:`) or the placeholder string `"unknown"`.
//!
//! GH-305: `m.exec()` (or any method call `convert_method_call_to_value`
//! has no lowering for) used to fall through to
//! `ShellValue::String("unknown")`, which `convert_expr` (statement
//! position) then discarded entirely, emitting `ShellIR::Noop` -> `:`.
//! The program silently did nothing.
//!
//! GH-306: `items.len()` on a local array literal has an exact element
//! count known at transpile time -- the same literal `known_array_items`
//! already extracts for `array_len(items)` (GH-293, PMAT-255). Route
//! `.len()` through that helper instead of falling through to `"unknown"`.
#![allow(clippy::unwrap_used)]

use crate::{transpile, Config};

fn transpile_ok(src: &str) -> String {
    transpile(src, &Config::default()).unwrap_or_else(|e| panic!("transpile failed: {e}"))
}

fn transpile_err(src: &str) -> crate::models::Error {
    transpile(src, &Config::default()).expect_err("transpile should have failed")
}

#[test]
fn test_PMAT257_gh305_unlowerable_method_call_is_an_error() {
    // `exec()` has a real lowering as a top-level *function* call, but no
    // lowering exists for it as a *method* call on an arbitrary receiver --
    // this must fail loudly, not lower to the no-op `:`.
    let src = r#"
        fn main() {
            let m = "echo pwned";
            m.exec();
        }
    "#;
    let err = transpile_err(src);
    let msg = err.to_string();
    assert!(
        msg.contains("exec"),
        "error should name the unlowerable method `exec`, got: {msg}"
    );
}

#[test]
fn test_PMAT257_gh305_method_call_with_lowering_still_transpiles() {
    // Guard: `std::env::args().nth(N).unwrap()` has a real lowering
    // (try_unwrap_env_args_nth) and must keep working after the fallback
    // for *unlowerable* calls becomes an error.
    let src = r#"
        fn main() {
            let first = std::env::args().nth(1).unwrap();
            println!("{}", first);
        }
    "#;
    let script = transpile_ok(src);
    assert!(
        script.contains("$1"),
        "expected the known args().nth(1).unwrap() lowering to positional $1:\n{script}"
    );
}

#[test]
fn test_PMAT257_gh306_len_on_local_array_literal_is_the_count() {
    let src = r#"
        fn main() {
            let items = ["a", "b", "c"];
            let n = items.len();
            println!("{}", n);
        }
    "#;
    let script = transpile_ok(src);
    let (stdout, code) = run_shell("dash", &script);
    assert_eq!(code, 0, "script aborted under dash:\n{script}");
    assert_eq!(stdout, "3", "script:\n{script}");
    assert!(
        !script.contains("unknown"),
        "local array .len() must not fall back to \"unknown\":\n{script}"
    );
}

#[test]
fn test_PMAT257_gh306_len_on_non_literal_array_is_an_error() {
    // `items` here is a scalar assigned from a stdlib call, never inserted
    // into the known-array-literal table -- its length is not known at
    // transpile time, so `.len()` must fail rather than silently emit the
    // string "unknown".
    let src = r#"
        fn main() {
            let items = env("PATH");
            let n = items.len();
        }
    "#;
    let err = transpile_err(src);
    let msg = err.to_string();
    assert!(
        msg.contains("len"),
        "error should name the unlowerable method `len`, got: {msg}"
    );
    assert!(
        !msg.contains("unknown"),
        "error must not just restate the old \"unknown\" placeholder: {msg}"
    );
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
