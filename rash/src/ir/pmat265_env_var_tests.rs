//! PMAT-265: `std::env::var` is an environment read, and its `Result`
//! methods have exact POSIX spellings.
//!
//! Found by validating the v7.4.0 corpus candidates: `std::env::var("X")`
//! alone lowered to the command substitution `$(std::env::var X)` — a
//! command that does not exist — and `std::env::var("X").unwrap_or(d)`, the
//! commonest Rust spelling of an env default, failed the transpile because
//! the 7.3.0 `unwrap_or` lowering accepts only a bare variable receiver. The
//! same validation showed the 7.3.0 rendering of the default (through
//! `escape_shell_string`) prints literal single quotes under bash and is a
//! syntax error under dash for any default that is not a plain word, and
//! that the exec-string validator refuses `$((…))` as command substitution.
//!
//! The spellings, all POSIX and agreed by bash, dash and busybox:
//!
//! | Rust | shell |
//! |---|---|
//! | `std::env::var("X")` | `"${X}"` |
//! | `.unwrap_or(d)`, `.unwrap_or_else(\|_\| d)` | `"${X-d}"` (unset only) |
//! | `.unwrap_or_default()` | `"${X}"` |
//! | `.unwrap()` | `"${X?}"` (aborts when unset, like the panic) |
//! | `.expect("m")` | `"${X?m}"` |
//!
//! A default that is not a plain word is spelled as a nested double-quoted
//! word with backslash escapes: `"${X-"{brace}"}"`.
#![allow(clippy::unwrap_used)]

use crate::{transpile, Config};

fn transpile_ok(src: &str) -> String {
    transpile(src, &Config::default()).unwrap_or_else(|e| panic!("transpile failed: {e}"))
}

fn transpile_err(src: &str) -> String {
    transpile(src, &Config::default())
        .expect_err("transpile should have failed")
        .to_string()
}

/// Run `script` as a file under `shell` with `env` applied on top of a
/// cleared environment (so the test's own `X` never leaks in), returning
/// (stdout, stderr, exit code).
fn run_with_env(shell: &str, script: &str, env: &[(&str, &str)]) -> (String, String, i32) {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("script.sh");
    std::fs::write(&path, script).unwrap();
    let mut cmd = std::process::Command::new(shell);
    cmd.arg(&path).env_clear().env("PATH", "/usr/bin:/bin");
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd
        .output()
        .unwrap_or_else(|e| panic!("failed to run {shell}: {e}"));
    (
        String::from_utf8_lossy(&out.stdout).trim_end().to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code().unwrap_or(-1),
    )
}

fn stdout_of(shell: &str, script: &str, env: &[(&str, &str)]) -> String {
    let (stdout, stderr, code) = run_with_env(shell, script, env);
    assert_eq!(
        code, 0,
        "script failed under {shell} (stderr: {stderr}):\n{script}"
    );
    stdout
}

// ===== F-TCORE-025: a bare read =====

#[test]
fn test_PMAT265_bare_env_var_is_a_parameter_expansion_not_a_command() {
    let script = transpile_ok(r#"fn main() { let v = std::env::var("X"); println!("[{}]", v); }"#);
    assert!(
        !script.contains("std::env::var"),
        "an env read must not become a command named std::env::var:\n{script}"
    );
    assert!(
        script.contains("${X}"),
        "expected the parameter expansion of X:\n{script}"
    );
    assert_eq!(stdout_of("dash", &script, &[("X", "hello")]), "[hello]");
    assert_eq!(stdout_of("dash", &script, &[]), "[]");
}

// ===== F-TCORE-026: unwrap_or / unwrap_or_else on the env name itself =====

#[test]
fn test_PMAT265_env_var_unwrap_or_is_unset_only_default_on_the_env_name() {
    let sources = [
        r#"fn main() { let v = std::env::var("X").unwrap_or("def"); println!("[{}]", v); }"#,
        r#"fn main() { let v = std::env::var("X").unwrap_or("def".to_string()); println!("[{}]", v); }"#,
        r#"fn main() { let v = std::env::var("X").unwrap_or_else(|_| "def".to_string()); println!("[{}]", v); }"#,
        r#"fn main() { let v = env("X").unwrap_or("def"); println!("[{}]", v); }"#,
    ];
    for src in sources {
        let script = transpile_ok(src);
        assert!(
            script.contains("${X-def}"),
            "expected the unset-only default on X itself:\n{script}"
        );
        assert!(
            !script.contains("${X:-"),
            "must not use the set-or-empty form:\n{script}"
        );
        assert_eq!(
            stdout_of("dash", &script, &[]),
            "[def]",
            "unset X takes the default"
        );
        assert_eq!(
            stdout_of("dash", &script, &[("X", "")]),
            "[]",
            "set-but-empty X keeps its value"
        );
        assert_eq!(stdout_of("dash", &script, &[("X", "set")]), "[set]");
        assert_eq!(stdout_of("bash", &script, &[]), "[def]");
    }
}

// ===== F-TCORE-027: unwrap_or_default, unwrap, expect =====

#[test]
fn test_PMAT265_env_var_unwrap_expect_and_unwrap_or_default_have_exact_spellings() {
    let defaulted = transpile_ok(
        r#"fn main() { let v = std::env::var("X").unwrap_or_default(); println!("[{}]", v); }"#,
    );
    assert!(defaulted.contains("${X}"), "{defaulted}");
    assert_eq!(stdout_of("dash", &defaulted, &[]), "[]");
    assert_eq!(stdout_of("dash", &defaulted, &[("X", "v")]), "[v]");

    let unwrapped =
        transpile_ok(r#"fn main() { let v = std::env::var("X").unwrap(); println!("[{}]", v); }"#);
    assert!(
        unwrapped.contains("${X?}"),
        "unwrap is the abort-when-unset expansion:\n{unwrapped}"
    );
    assert_eq!(stdout_of("dash", &unwrapped, &[("X", "v")]), "[v]");
    let (stdout, _stderr, code) = run_with_env("dash", &unwrapped, &[]);
    assert_ne!(
        code, 0,
        "unwrap of an unset variable must abort, as the panic would:\n{unwrapped}"
    );
    assert!(
        !stdout.contains("[]"),
        "the script must not continue past the abort:\n{unwrapped}"
    );

    let expected = transpile_ok(
        r#"fn main() { let v = std::env::var("X").expect("X must be set"); println!("[{}]", v); }"#,
    );
    assert!(expected.contains("${X?X must be set}"), "{expected}");
    let (_stdout, stderr, code) = run_with_env("dash", &expected, &[]);
    assert_ne!(code, 0);
    assert!(
        stderr.contains("X must be set"),
        "the message reaches stderr: {stderr}"
    );
    assert_eq!(stdout_of("dash", &expected, &[("X", "v")]), "[v]");
}

// ===== F-TCORE-028: the default is spelled for the double-quoted context =====

#[test]
fn test_PMAT265_unwrap_or_default_text_survives_braces_spaces_dollars_and_quotes() {
    // Each default, as Rust source text and as the bytes the script must print.
    let cases: [(&str, &str); 6] = [
        (r#""{brace}""#, "{brace}"),
        (r#""a  b""#, "a  b"),
        (r#""$HOME""#, "$HOME"),
        (r#""it's""#, "it's"),
        (r#""say \"hi\"""#, "say \"hi\""),
        (r#""back\\slash""#, "back\\slash"),
    ];
    for (rust_default, printed) in cases {
        // The env receiver and the bare-variable receiver share one renderer.
        let env_src = format!(
            r#"fn main() {{ let v = std::env::var("X").unwrap_or({rust_default}); println!("[{{}}]", v); }}"#
        );
        let var_src = format!(
            r#"fn main() {{ let x = env("X"); let v = x.unwrap_or({rust_default}); println!("[{{}}]", v); }}"#
        );
        let env_script = transpile_ok(&env_src);
        let want = format!("[{printed}]");
        for shell in ["dash", "bash"] {
            assert_eq!(
                stdout_of(shell, &env_script, &[]),
                want,
                "default {rust_default} under {shell}:\n{env_script}"
            );
        }
        assert_eq!(stdout_of("dash", &env_script, &[("X", "v")]), "[v]");
        // The bare-variable path: `x` is always set here (it holds "${X}"),
        // so the default never applies; what matters is that the script is
        // valid under dash, which the 7.3.0 rendering was not.
        let var_script = transpile_ok(&var_src);
        assert_eq!(
            stdout_of("dash", &var_script, &[("X", "v")]),
            "[v]",
            "{var_script}"
        );
        assert_eq!(
            stdout_of("bash", &var_script, &[("X", "v")]),
            "[v]",
            "{var_script}"
        );
    }
}

// ===== F-TCORE-029: the name must be a shell identifier =====

#[test]
fn test_PMAT265_env_var_non_identifier_name_is_refused() {
    let err = transpile_err(
        r#"fn main() { let v = std::env::var("my-var").unwrap_or("d"); println!("{}", v); }"#,
    );
    assert!(
        err.contains("my-var"),
        "the error must name the variable: {err}"
    );
    let err = transpile_err(r#"fn main() { let v = std::env::var("my-var"); println!("{}", v); }"#);
    assert!(err.contains("my-var"), "{err}");
}

// ===== F-TCORE-030: arithmetic in a capture() string =====

#[test]
fn test_PMAT265_arithmetic_in_capture_string_is_not_command_substitution() {
    let script = transpile_ok(
        r#"fn main() { let out = capture("echo $((1 + 2))"); println!("[{}]", out); }"#,
    );
    assert_eq!(stdout_of("dash", &script, &[]), "[3]", "{script}");

    for src in [
        r#"fn main() { let out = capture("echo $(date)"); println!("{}", out); }"#,
        r#"fn main() { let out = capture("echo $(( $(date) ))"); println!("{}", out); }"#,
        r#"fn main() { let out = capture("echo `date`"); println!("{}", out); }"#,
    ] {
        let err = transpile_err(src);
        assert!(
            err.contains("substitution"),
            "a real command substitution is still refused ({src}): {err}"
        );
    }
}
