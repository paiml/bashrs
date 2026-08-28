//! Every stdlib function must produce a script that RUNS.
//!
//! # Why this file exists (bashrs#266)
//!
//! `let out = exec("echo hi")` transpiled cleanly — `bashrs check` said
//! "✓ compatible with Rash", `bashrs build` said "Successfully transpiled" and
//! exited 0 — and the artifact died with `rash_exec: not found`, exit 127.
//! `mkdir`, `mv`, `chmod` and `sleep` were the same in expression position.
//!
//! The five-whys landed here: `stdlib::is_stdlib_function()` (the whitelist) and
//! `posix_runtime::write_selective_runtime()` (the emitter dispatch) are two
//! hand-maintained lists with nothing tying them together, and **the test suite
//! asserted that transpilation SUCCEEDS, never that the ARTIFACT EXECUTES.**
//! GH-148 added seven functions to the first list and none to the second, and
//! every test still passed.
//!
//! So these tests transpile *and then run the script*, asserting on its stdout
//! and exit status. A helper that is never emitted cannot pass here, and neither
//! can a lowering that is syntactically valid but semantically wrong.

use std::fs;
use std::process::Command;

/// Transpile `source`, execute the result with /bin/sh, return (stdout, exit code).
fn transpile_and_run(name: &str, source: &str) -> (String, i32) {
    let dir = std::env::temp_dir().join(format!("rash_roundtrip_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");

    let script = bashrs::transpile(source, &bashrs::Config::default())
        .unwrap_or_else(|e| panic!("{name}: transpile failed: {e}"));

    let path = dir.join("script.sh");
    fs::write(&path, &script).expect("write script");

    let out = Command::new("/bin/sh")
        .arg(&path)
        .current_dir(&dir)
        .output()
        .expect("run script");

    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let code = out.status.code().unwrap_or(-1);

    // 127 is "command not found" — the exact signature of a helper that was
    // called but never defined. Name it explicitly so a regression is obvious
    // from the failure message rather than from a mystery exit code.
    if code == 127 {
        panic!(
            "{name}: script exited 127 (command not found) — a helper was called \
             but never defined. This is bashrs#266.\nstderr: {}\nscript:\n{script}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let _ = fs::remove_dir_all(&dir);
    (stdout, code)
}

#[test]
fn bashrs_266_exec_in_expression_position_runs() {
    let (stdout, code) = transpile_and_run(
        "exec_expr",
        r#"fn main() { let out = exec("echo hi"); println!("{}", out); }"#,
    );
    assert_eq!(code, 0, "stdout was {stdout:?}");
    assert_eq!(stdout, "hi");
}

#[test]
fn exec_in_statement_position_runs() {
    let (stdout, code) =
        transpile_and_run("exec_stmt", r#"fn main() { exec("echo from-statement"); }"#);
    assert_eq!(code, 0);
    assert_eq!(stdout, "from-statement");
}

#[test]
fn capture_runs_and_captures() {
    // capture() was NOT broken — recorded here so a future "fix" cannot silently
    // regress it. It lowers to a command substitution by design.
    let (stdout, code) = transpile_and_run(
        "capture",
        r#"fn main() { let v = capture("echo captured"); println!("{}", v); }"#,
    );
    assert_eq!(code, 0);
    assert_eq!(stdout, "captured");
}

#[test]
fn capture_with_a_pipe_runs() {
    let (stdout, code) = transpile_and_run(
        "capture_pipe",
        r#"fn main() { let v = capture("printf 'a\nb\n' | wc -l"); println!("{}", v); }"#,
    );
    assert_eq!(code, 0);
    assert_eq!(stdout.trim(), "2");
}

#[test]
fn mkdir_in_statement_position_runs() {
    let (_stdout, code) = transpile_and_run("mkdir_stmt", r#"fn main() { mkdir("sub"); }"#);
    assert_eq!(code, 0);
}

#[test]
fn sleep_in_statement_position_runs() {
    let (_stdout, code) = transpile_and_run("sleep_stmt", r#"fn main() { sleep(0); }"#);
    assert_eq!(code, 0);
}

#[test]
fn calling_an_external_command_still_runs() {
    // Intentional Rash behaviour: an undeclared call lowers to the bare command.
    // Pinned so the bashrs#266 invariant cannot be over-tightened into rejecting
    // it — that would break a documented idiom, and the existing integration
    // tests (`fn echo(msg: &str) {}` stubs) depend on this shape.
    let (stdout, code) = transpile_and_run("external", r#"fn main() { echo("external-ok"); }"#);
    assert_eq!(code, 0);
    assert_eq!(stdout, "external-ok");
}

/// A void stdlib function in expression position must FAIL THE BUILD, not
/// produce a script that dies at 127.
///
/// Assigning the result of `mkdir` has no meaning in shell. Before bashrs#266
/// it transpiled happily and exploded at runtime; now it is a compile error, and
/// the message names the function and tells you to call it as a statement.
#[test]
fn bashrs_266_void_stdlib_in_expression_position_is_a_build_error() {
    let err = bashrs::transpile(
        r#"fn main() { let v = mkdir("d"); println!("{}", v); }"#,
        &bashrs::Config::default(),
    )
    .expect_err("assigning mkdir() must not transpile");

    let text = format!("{err}");
    assert!(text.contains("rash_mkdir"), "message must name it: {text}");
    assert!(
        text.contains("STATEMENT") || text.contains("statement"),
        "message must say how to fix it: {text}"
    );
}

/// The structural guarantee, stated as a test.
///
/// Whatever a program does, the emitted script must never CALL a `rash_*` helper
/// it does not DEFINE. This is the invariant that closes the class rather than
/// the five individual symptoms — a future stdlib addition with no writer fails
/// here instead of at 3am.
#[test]
fn bashrs_266_emitted_scripts_never_call_an_undefined_helper() {
    let programs = [
        r#"fn main() { let out = exec("echo a"); println!("{}", out); }"#,
        r#"fn main() { exec("echo b"); }"#,
        r#"fn main() { let v = capture("echo c"); println!("{}", v); }"#,
        r#"fn main() { mkdir("x"); }"#,
        r#"fn main() { sleep(0); }"#,
        r#"fn main() { println!("plain"); }"#,
        r#"fn main() { let s = "hi"; println!("{}", s); }"#,
    ];

    for source in programs {
        let Ok(script) = bashrs::transpile(source, &bashrs::Config::default()) else {
            // A rejected program is fine; a broken artifact is not.
            continue;
        };
        let defined: Vec<String> = script
            .lines()
            .filter_map(|l| l.trim_start().strip_suffix("() {").map(str::to_string))
            .filter(|n| n.starts_with("rash_"))
            .collect();
        for line in script.lines() {
            for token in line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
                if token.starts_with("rash_") && token.len() > 5 {
                    assert!(
                        defined.iter().any(|d| d == token),
                        "script calls {token} but never defines it.\nsource: {source}\nscript:\n{script}"
                    );
                }
            }
        }
    }
}
