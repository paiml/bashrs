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
