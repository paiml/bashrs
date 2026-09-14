//! bashrs#335: `bashrs fix` on the issue's file must leave a program that runs
//! the way it did before. This executes the artifact rather than trusting a
//! count of applied fixes -- the 7.4.0 output passed `bash -n` and was wrong.

#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use std::fs;
use std::process::Command as Proc;

const ISSUE_335: &str = r#"#!/usr/bin/env bash
set -euo pipefail
TD=$(mktemp -d)
trap 'rm -rf -- "${TD:?}"' EXIT
assert_row() { local label="$1" want="$2" file="$3" needle="$4"; grep -q "$needle" "$file" && echo "ok $label" || echo "FAIL $label ($want)"; }
printf 'added=1\n' > "$TD/append.yaml"
assert_row 'append one entry' PASS "$TD/append.yaml" 'added=1'
"#;

fn run_bash(path: &std::path::Path) -> (Option<i32>, String) {
    let out = Proc::new("bash").arg(path).output().unwrap();
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn test_cli_fix_335_program_still_runs_the_same() {
    let dir = std::env::temp_dir().join(format!("bashrs-335-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let script = dir.join("fixme.sh");
    fs::write(&script, ISSUE_335).unwrap();

    let before = run_bash(&script);
    assert_eq!(
        before,
        (Some(0), "ok append one entry\n".to_string()),
        "fixture sanity"
    );

    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("fix")
        .arg(&script)
        .assert()
        .success();

    let after_src = fs::read_to_string(&script).unwrap();
    let after = run_bash(&script);
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(
        after, before,
        "bashrs fix changed what the program does:\n{after_src}"
    );
}
