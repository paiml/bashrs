//! PMAT-255 / GH-293: array_join and array_len must receive the elements of
//! a local array literal, not the never-assigned "$name" scalar variable.
//!
//! These transpile a small program and then actually run the emitted script
//! under `dash` and `sh` to prove the fix is behavioral, not just "compiles".
#![allow(clippy::unwrap_used)]

use crate::{transpile, Config};
use std::process::Command;

fn run_shell(shell: &str, script: &str) -> (String, i32) {
    let output = Command::new(shell)
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

fn transpile_ok(src: &str) -> String {
    transpile(src, &Config::default()).unwrap_or_else(|e| panic!("transpile failed: {e}"))
}

#[test]
fn test_PMAT255_gh293_array_join_runs_under_dash() {
    let src = r#"
        fn main() {
            let items = ["a", "b"];
            let joined = array_join(items, ",");
            println!("{}", joined);
        }
    "#;
    let script = transpile_ok(src);
    let (stdout, code) = run_shell("dash", &script);
    assert_eq!(code, 0, "script aborted under dash:\n{script}");
    assert_eq!(stdout, "a,b");
}

#[test]
fn test_PMAT255_gh293_array_join_runs_under_sh() {
    let src = r#"
        fn main() {
            let items = ["a", "b"];
            let joined = array_join(items, ",");
            println!("{}", joined);
        }
    "#;
    let script = transpile_ok(src);
    let (stdout, code) = run_shell("sh", &script);
    assert_eq!(code, 0, "script aborted under sh:\n{script}");
    assert_eq!(stdout, "a,b");
}

#[test]
fn test_PMAT255_gh293_array_len_runs_under_dash() {
    let src = r#"
        fn main() {
            let items = ["a", "b"];
            let n = array_len(items);
            println!("{}", n);
        }
    "#;
    let script = transpile_ok(src);
    let (stdout, code) = run_shell("dash", &script);
    assert_eq!(code, 0, "script aborted under dash:\n{script}");
    assert_eq!(stdout, "2");
}

#[test]
fn test_PMAT255_gh293_for_in_still_prints_a_b() {
    // Guard: the already-working for-in lowering must not regress.
    let src = r#"
        fn main() {
            let items = ["a", "b"];
            for x in items {
                println!("{}", x);
            }
        }
    "#;
    let script = transpile_ok(src);
    let (stdout, code) = run_shell("dash", &script);
    assert_eq!(code, 0, "script aborted under dash:\n{script}");
    assert_eq!(stdout, "a\nb");
}
