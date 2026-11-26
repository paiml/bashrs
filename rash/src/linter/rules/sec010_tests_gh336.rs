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
    assert!(check(s)
        .diagnostics
        .iter()
        .all(|d| d.severity != Severity::Error));
}

#[test]
fn test_GH528_positional_inside_child_shell_script_does_not_fire() {
    for line in [
        "bash -c 'cd \"$1\" && ls' _ /opt/build",
        "sh -c 'rm -rf \"$1\"' sh /tmp/x",
        "bash -lc 'cat \"$1\"' _ /etc/hostname",
    ] {
        let s = format!("#!/usr/bin/env bash\n{line}\n");
        assert!(
            check(&s).diagnostics.iter().all(|d| d.code != "SEC010"),
            "{line}"
        );
    }
}

#[test]
fn test_GH528_parent_shell_positional_still_fires() {
    let s = "#!/usr/bin/env bash\nREPO=\"$1\"\ncd \"$REPO\" && ls\n";
    assert!(check(s).diagnostics.iter().any(|d| d.code == "SEC010"));
}

#[test]
fn test_GH528_child_shell_script_with_inner_command_after_space() {
    for line in [
        "bash -c 'set -e; cd \"$1\" && make' _ /fixed",
        "sh -c 'echo hi; cat \"$1\"' _ /etc/hosts",
        "bash -c 'test -d \"$1\" || mkdir -p \"$1\"' _ out",
    ] {
        let s = format!("#!/usr/bin/env bash\n{line}\n");
        assert!(
            check(&s).diagnostics.iter().all(|d| d.code != "SEC010"),
            "{line}"
        );
    }
}

#[test]
fn test_GH528_arg_after_child_shell_script_still_checked() {
    let s = "#!/usr/bin/env bash\nREPO=\"$1\"\nbash -c 'echo x' _ && cd \"$REPO\"\n";
    assert!(check(s).diagnostics.iter().any(|d| d.code == "SEC010"));
}

#[test]
fn test_GH336_direct_positional_cd_fires() {
    let s = "#!/usr/bin/env bash\ncd \"$1\" && rm -rf out\n";
    assert!(
        check(s)
            .diagnostics
            .iter()
            .any(|d| d.code == "SEC010" && d.severity == Severity::Error),
        "cd \"$1\" is external input"
    );
}

#[test]
fn test_GH336_read_r_validated_stays_clear() {
    let s = "#!/usr/bin/env bash\nset -euo pipefail\ntd=$(mktemp -d)\nread -r r < \"$td/f\"\nif [[ \"$r\" == *..* ]] || [[ \"$r\" != /* ]]; then exit 2; fi\nout=$(cd \"$r\" && cargo build 2>&1); echo \"$out\"\n";
    assert!(check(s).diagnostics.iter().all(|d| d.code != "SEC010"));
}

fn sec010_fires(s: &str) -> bool {
    check(s).diagnostics.iter().any(|d| d.code == "SEC010")
}

#[test]
fn test_GH336_stdin_cat_substitution_is_external() {
    for src in [
        "r=$(cat)",
        "r=$(cat -)",
        "r=$(cat /dev/stdin)",
        "r=$(cat <&0)",
        "r=`cat`",
        "r=$(cat 2>/dev/null)",
    ] {
        for sink in ["cd \"$r\"", "rm -rf \"$r\""] {
            let s = format!("#!/usr/bin/env bash\n{src}\n{sink}\n");
            assert!(
                sec010_fires(&s),
                "SEC010 must fire for `{src}` then `{sink}`"
            );
        }
    }
}

#[test]
fn test_GH336_file_cat_substitution_stays_clean() {
    for src in [
        "r=$(cat \"$ROOT/VERSION\")",
        "r=$(cat VERSION)",
        "r=$(cat < VERSION)",
        "r=$(cat <<EOF)",
    ] {
        let s = format!("#!/usr/bin/env bash\nROOT=/opt/app\n{src}\ncd \"$r\"\n");
        assert!(!sec010_fires(&s), "SEC010 must stay silent for `{src}`");
    }
}

#[test]
fn test_GH336_cat_args_end_at_control_operators() {
    for src in [
        "r=$(cat && echo ok)",
        "r=$(cat; echo ok)",
        "r=$(cat | tr a b)",
        "r=`cat && echo ok`",
        "r=$(cat || echo ok)",
    ] {
        let s = format!("#!/usr/bin/env bash\n{src}\ncd \"$r\"\n");
        assert!(sec010_fires(&s), "SEC010 must fire for `{src}`");
    }
}

#[test]
fn test_GH336_cat_redirect_target_is_not_a_file_operand() {
    for src in [
        "r=$(cat 2> /dev/null)",
        "r=$(cat > /dev/null)",
        "r=$(cat 2>/dev/null)",
        "r=$(cat >/dev/null)",
        "r=$(cat >> log.txt)",
        "r=$(cat 2>&1)",
        "r=$(cat &>/dev/null)",
    ] {
        let s = format!("#!/usr/bin/env bash\n{src}\nrm -rf \"$r\"\n");
        assert!(sec010_fires(&s), "SEC010 must fire for `{src}`");
    }
}

#[test]
fn test_GH336_cat_unspaced_and_quoted_stdin_forms() {
    for src in [
        "r=$(cat </dev/stdin)",
        "r=$(cat 0</dev/stdin)",
        "r=$(cat \"/dev/stdin\")",
        "r=$(cat < \"/dev/stdin\")",
        "r=$(cat <&0)",
        "r=$(cat 0<&0)",
    ] {
        let s = format!("#!/usr/bin/env bash\n{src}\ncd \"$r\"\n");
        assert!(sec010_fires(&s), "SEC010 must fire for `{src}`");
    }
}

#[test]
fn test_GH336_cat_outside_command_position_is_not_stdin() {
    for src in [
        "r=$(dirname /opt/cat)",
        "r=$(basename cat)",
        "r=$(cat \"$ROOT/VERSION\" && echo ok)",
        "r=$(cat 2>/dev/null VERSION)",
        "r=$(cat <VERSION)",
    ] {
        let s = format!("#!/usr/bin/env bash\nROOT=/opt/app\n{src}\ncd \"$r\"\n");
        assert!(!sec010_fires(&s), "SEC010 must stay silent for `{src}`");
    }
}

#[test]
fn test_GH336_cat_nonzero_fd_input_redirect_still_reads_stdin() {
    for src in [
        "r=$(cat 3</dev/null)",
        "r=$(cat 3<file.txt)",
        "r=$(cat 3< file.txt)",
        "r=$(cat 3<&0)",
    ] {
        let s = format!("#!/usr/bin/env bash\n{src}\ncd \"$r\"\n");
        assert!(sec010_fires(&s), "SEC010 must fire for `{src}`");
    }
}

#[test]
fn test_GH336_cat_stdin_in_extended_command_positions() {
    for src in [
        "r=$(! cat)",
        "r=$(if true; then cat; fi)",
        "r=$(if false; then echo x; else cat; fi)",
        "r=$(if false; then echo x; elif true; then cat; fi)",
        "r=$(while true; do cat; done)",
        "r=$(until false; do cat; done)",
        "r=$(IFS= cat)",
        "r=$(A=1 B=2 cat)",
    ] {
        let s = format!("#!/usr/bin/env bash\n{src}\ncd \"$r\"\n");
        assert!(sec010_fires(&s), "SEC010 must fire for `{src}`");
    }
}

#[test]
fn test_GH336_cat_after_non_command_words_stays_silent() {
    for src in [
        "r=$(echo then cat)",
        "r=$(echo FOO=1 cat)",
        "r=$(echo ! cat)",
    ] {
        let s = format!("#!/usr/bin/env bash\n{src}\ncd \"$r\"\n");
        assert!(!sec010_fires(&s), "SEC010 must stay silent for `{src}`");
    }
}

// GH-336 (bashrs#336, multi-line): an unvalidated `cd "$VAR"` moves the working
// directory for every later line. A later literal `rm -rf` on its own line
// must be flagged; the old line-keyed check said nothing about it.
fn sec010_lines(src: &str) -> Vec<usize> {
    check(src)
        .diagnostics
        .iter()
        .filter(|d| d.code == "SEC010")
        .map(|d| d.span.start_line)
        .collect()
}

#[test]
fn test_GH336_rm_after_tainted_cd_on_later_line_fires() {
    let lines = sec010_lines("#!/bin/sh\ncd \"$1\" || exit 1\nrm -rf build\n");
    assert!(
        lines.contains(&3),
        "rm -rf on line 3 not flagged after tainted cd: {lines:?}"
    );
}

#[test]
fn test_GH336_cargo_after_tainted_cd_on_later_line_is_not_flagged() {
    let lines = sec010_lines("#!/bin/sh\ncd \"$1\" || exit 1\ncargo build\n");
    assert!(
        !lines.contains(&3),
        "cargo on line 3 is a false positive: {lines:?}"
    );
}

#[test]
fn test_GH336_literal_absolute_cd_resets_carried_taint() {
    let lines =
        sec010_lines("#!/bin/sh\ncd \"$1\" || exit 1\ncd /opt/app || exit 1\nrm -rf build\n");
    assert!(
        !lines.contains(&4),
        "rm after literal cd still flagged: {lines:?}"
    );
}

#[test]
fn test_GH336_quoted_absolute_rm_after_tainted_cd_is_not_relative() {
    let lines = sec010_lines("#!/bin/sh\ncd \"$1\" || exit 1\nrm -rf \"/tmp/build\"\n");
    assert!(
        !lines.contains(&3),
        "quoted absolute rm misread as relative: {lines:?}"
    );
}
