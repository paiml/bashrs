//! GH-336: SEC010 must key on the taint of the path variable, not on which
//! command follows the `cd`. `cargo` used to fire only because the name
//! heuristic matched `ARG` inside the word "cargo".

use super::*;
use crate::linter::Severity;

fn script_with(x: &str) -> String {
    format!(
        "#!/usr/bin/env bash\nREPO=\"\"\nwhile [ $# -gt 0 ]; do case \"$1\" in --repo) REPO=\"$2\"; shift;; esac; shift; done\n( cd \"$REPO\" && {x} ) > out.log 2>&1\n"
    )
}

#[test]
fn test_GH336_verdict_independent_of_command_after_cd() {
    let fires = |x: &str| {
        check(&script_with(x))
            .diagnostics
            .iter()
            .any(|d| d.code == "SEC010" && d.severity == Severity::Error)
    };
    for x in [
        "cargo build",
        "cargo test --no-run",
        "ls",
        "timeout 60 ls",
        "make",
        "rm -rf out",
        "./run.sh",
    ] {
        assert!(fires(x), "SEC010 must fire for `cd \"$REPO\" && {x}`");
    }
}

#[test]
fn test_GH336_cargo_in_a_literal_path_line_does_not_fire() {
    let s = "#!/usr/bin/env bash\ncd /opt/build && cargo build\n";
    assert!(check(s).diagnostics.is_empty());
}

#[test]
fn test_GH336_validated_repo_stays_clear() {
    let s = "#!/usr/bin/env bash\nREPO=\"$2\"\ncase \"$REPO\" in /*) ;; *) exit 2;; esac\ncase \"$REPO\" in *..*) exit 2;; esac\n( cd \"$REPO\" && rm -rf out )\n";
    assert!(check(s).diagnostics.iter().all(|d| d.severity != Severity::Error));
}

fn sec010_errors(s: &str) -> usize {
    check(s)
        .diagnostics
        .iter()
        .filter(|d| d.code == "SEC010" && d.severity == Severity::Error)
        .count()
}

fn fires_on(body: &str) -> bool {
    sec010_errors(&format!("#!/usr/bin/env bash\n{body}\n")) >= 1
}

#[test]
fn test_GH336_eval_body_runs_in_this_shell_fires() {
    assert!(fires_on("eval 'cd \"$1\" && rm -rf out'"));
}

#[test]
fn test_GH336_trap_body_runs_in_this_shell_fires() {
    assert!(fires_on("W=\"$1\"\ntrap 'cd /; rm -rf \"$W\"' EXIT"));
}

#[test]
fn test_GH336_escaped_quote_outside_quotes_fires() {
    assert!(fires_on("echo can\\'t; cd \"$1\" && rm -rf out"));
}

#[test]
fn test_GH336_ansi_c_quote_with_escaped_quote_fires() {
    assert!(fires_on("rm -rf $'a\\'b' \"$1\""));
}

#[test]
fn test_GH336_quote_inside_nested_command_substitution_fires() {
    assert!(fires_on("rm -rf \"$(echo \"it's\")\" \"$1\""));
}

#[test]
fn test_GH336_quote_after_escaped_double_quote_fires() {
    assert!(fires_on("echo \"\\\"it's\"; rm -rf \"$1\""));
}

#[test]
fn test_GH336_apostrophe_inside_double_quotes_fires() {
    assert!(fires_on("rm -rf \"it's $1\""));
}

#[test]
fn test_GH336_sh_c_with_tainted_outer_argument_fires() {
    assert!(fires_on("sh -c 'cd \"$1\" && rm -rf out' _ \"$1\""));
}

#[test]
fn test_GH336_other_child_shells_are_masked() {
    for sh in ["bash", "dash", "zsh", "ksh"] {
        let s = format!("#!/usr/bin/env bash\nd=/srv/data\n{sh} -c 'cd \"$1\" && rm -f .probe' _ \"$d\"\n");
        assert_eq!(sec010_errors(&s), 0, "{sh}");
    }
}

// Pins behaviour that already holds on main (not red-then-green).
#[test]
fn test_GH336_pin_positional_cd_then_rm_fires() {
    let s = "#!/usr/bin/env bash\ncd \"$1\" && rm -rf out\n";
    assert_eq!(sec010_errors(s), 1);
}

#[test]
fn test_GH336_inner_sh_c_positional_is_not_outer_taint() {
    // `$1` inside the single-quoted `sh -c` body is the inner shell's own
    // argument; the literal removal in the body is not the path under test.
    let s = "#!/usr/bin/env bash\nd=/srv/data\nrunuser -u noah -- sh -c 'cd \"$1\" && rm -f .probe' _ \"$d\"\n";
    assert_eq!(sec010_errors(s), 0);
}
