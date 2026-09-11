//! PMAT-255 / GH-294: a string literal whose emitted form is single-quoted
//! must not be refused for holding a backtick, `$(`, or `$VAR` — those bytes
//! are inert inside single quotes in every POSIX shell. `exec()`/`capture()`
//! arguments are executed, so the same bytes there stay refused (true
//! positives, guarded here too).
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
fn test_PMAT255_gh294_backtick_literal_transpiles_and_prints_verbatim() {
    let src = r#"
        fn main() {
            let s = "a `b` c";
            println!("{}", s);
        }
    "#;
    let script = transpile_ok(src);

    let (stdout_dash, code_dash) = run_shell("dash", &script);
    assert_eq!(code_dash, 0, "script aborted under dash:\n{script}");
    assert_eq!(stdout_dash, "a `b` c");

    let (stdout_sh, code_sh) = run_shell("sh", &script);
    assert_eq!(code_sh, 0, "script aborted under sh:\n{script}");
    assert_eq!(stdout_sh, "a `b` c");
}

#[test]
fn test_PMAT255_gh294_dollar_paren_and_dollar_var_literal_prints_verbatim() {
    let src = r#"
        fn main() {
            let s = "cost $(x) and $HOME";
            println!("{}", s);
        }
    "#;
    let script = transpile_ok(src);
    let (stdout, code) = run_shell("dash", &script);
    assert_eq!(code, 0, "script aborted under dash:\n{script}");
    assert_eq!(stdout, "cost $(x) and $HOME");
}

#[test]
fn test_PMAT255_gh294_exec_backtick_still_refused() {
    // True positive: exec() strings are executed, so a backtick here IS a
    // live command substitution — validation must keep rejecting it.
    let src = r#"
        fn main() {
            exec("echo `date`");
        }
    "#;
    let result = transpile(src, &Config::default());
    assert!(
        result.is_err(),
        "exec(\"echo `date`\") must still be refused, got: {result:?}"
    );
}

#[test]
fn test_PMAT255_gh294_exec_dollar_paren_still_refused() {
    // True positive: exec() strings are executed, so $( ) here IS a live
    // command substitution — validation must keep rejecting it.
    let src = r#"
        fn main() {
            exec("echo $(date)");
        }
    "#;
    let result = transpile(src, &Config::default());
    assert!(
        result.is_err(),
        "exec(\"echo $(date)\") must still be refused, got: {result:?}"
    );
}

#[test]
fn test_PMAT255_gh294_validate_backticks_is_quote_aware() {
    use super::rules::validate_backticks;
    assert!(
        validate_backticks("s='a `b` c'").is_ok(),
        "inert inside single quotes"
    );
    assert!(
        validate_backticks("echo \"a \\`b\\` c\"").is_ok(),
        "escaped inside double quotes"
    );
    assert!(
        validate_backticks("echo `date`").is_err(),
        "a live backtick is still reported"
    );
    assert!(
        validate_backticks("echo \"a `b` c\"").is_err(),
        "live inside double quotes"
    );
}

#[test]
fn test_PMAT255_gh294_variable_reaching_exec_still_refused() {
    let src = r#"fn main() { let m = "$(whoami)"; exec(m); }"#;
    let err = transpile(src, &Config::default())
        .expect_err("a literal that reaches eval through a variable must stay refused");
    assert!(
        err.to_string().contains("substitution"),
        "refused for the wrong reason: {err}"
    );
}

#[test]
fn test_PMAT255_gh294_parameter_reaching_exec_still_refused() {
    let src = r#"fn run(s: &str) { exec(s); } fn main() { run("a `whoami` b"); }"#;
    let err = transpile(src, &Config::default())
        .expect_err("a literal that reaches eval through a parameter must stay refused");
    assert!(
        err.to_string().contains("acktick"),
        "refused for the wrong reason: {err}"
    );
}

#[test]
fn test_PMAT255_gh294_static_exec_keeps_printing_relaxed() {
    let src = r#"fn main() { let s = "a `b` c"; println!("{}", s); exec("true"); }"#;
    let script = transpile_ok(src);
    let (out, code) = run_shell("dash", &script);
    assert_eq!(code, 0, "script failed under dash:\n{script}");
    assert_eq!(out, "a `b` c");
}
