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
    let s = "#!/usr/bin/env bash\nd=/srv/data\nsh -c 'cd \"$1\" && rm -f .probe' _ \"$d\"\n";
    assert_eq!(sec010_errors(s), 0);
}

#[test]
fn test_GH336_tainted_outer_arg_on_continuation_line_fires() {
    assert!(fires_on("sh -c 'cd \"$1\" && rm -rf out' \\\n  _ \"$1\""));
}

#[test]
fn test_GH336_tainted_outer_arg_alone_on_continuation_line_fires() {
    assert!(fires_on("sh -c 'cd \"$1\" && rm -rf out' _ \\\n  \"$1\""));
}

#[test]
fn test_GH336_exported_tainted_variable_read_by_child_shell_fires() {
    assert!(fires_on("export D=\"$1\"\nsh -c 'cd \"$D\" && rm -rf out'"));
}

#[test]
fn test_GH336_exported_tainted_variable_with_outer_arg_fires() {
    assert!(fires_on("export D=\"$1\"\nsh -c 'cd \"$D\" && rm -rf out' _ /x"));
}

// Allow-list round: a `sh -c` script is masked only for a plain simple command
// at top level. Each input below was an Error on main and must stay one.
const CHILD: &str = "sh -c 'cd \"$1\" && rm -rf out' _ /x";

#[test]
fn test_GH336_unquoted_heredoc_body_piped_to_sh_fires() {
    assert!(fires_on(&format!("cat <<EOF | sh\n{CHILD}\nEOF")));
}

#[test]
fn test_GH336_unquoted_heredoc_body_ssh_fires() {
    assert!(fires_on(&format!("ssh host bash <<EOF\n{CHILD}\nEOF")));
}

#[test]
fn test_GH336_unquoted_heredoc_body_written_to_file_fires() {
    assert!(fires_on(&format!("cat > run.sh <<EOF\n{CHILD}\nEOF")));
}

#[test]
fn test_GH336_line_inside_multiline_double_quote_then_eval_fires() {
    assert!(fires_on(&format!("cmd=\"\n{CHILD}\n\"\neval \"$cmd\"")));
}

#[test]
fn test_GH336_line_inside_multiline_bash_c_double_quote_fires() {
    assert!(fires_on(&format!("bash -c \"\n{CHILD}\n\"")));
}

#[test]
fn test_GH336_eval_sh_c_fires() {
    assert!(fires_on(&format!("eval {CHILD}")));
}

#[test]
fn test_GH336_echo_sh_c_fires() {
    assert!(fires_on(&format!("echo {CHILD}")));
}

#[test]
fn test_GH336_ssh_host_sh_c_fires() {
    assert!(fires_on(&format!("ssh host {CHILD}")));
}

#[test]
fn test_GH336_sh_c_with_dollar_zero_outer_argument_fires() {
    assert!(fires_on("sh -c 'cd \"$1\" && rm -rf out' \"$0\" /x"));
}

#[test]
fn test_GH336_sh_c_inside_spaced_command_substitution_fires() {
    assert!(fires_on("x=$( sh -c 'cd \"$1\" && rm -rf out' _ /x )"));
}

#[test]
fn test_GH336_plain_sh_c_with_fixed_path_is_silent() {
    for body in [
        CHILD.to_string(),
        format!("true; {CHILD}"),
        format!("true && {CHILD}"),
        format!("true || {CHILD}"),
        format!("true | {CHILD}"),
        format!("( {CHILD} )"),
        format!("if true; then {CHILD}; fi"),
        "D=/srv sh -c 'cd \"$1\" && rm -rf out' _ /x".to_string(),
    ] {
        assert_eq!(sec010_errors(&format!("#!/usr/bin/env bash\n{body}\n")), 0, "{body}");
    }
}

// Round 4: the ten inputs of the fourth review. Each is an Error on main; the
// parent shell expands the `$1` in CHILD in every one of them.
#[test]
fn test_GH336_round4_ten_violating_inputs_fire() {
    let inputs: [(&str, String); 10] = [
        ("n27", format!("x='a\nb'; cat <<EOF | sh\n{CHILD}\nEOF")),
        ("n28", format!("x=\"a\nb\"; cat <<EOF | sh\n{CHILD}\nEOF")),
        ("n47", format!("echo \"x\n\"; ssh host bash <<EOF\n{CHILD}\nEOF")),
        ("n26", format!("true;# it's\ncat <<EOF | sh\nit's here\n{CHILD}\nEOF")),
        ("n45", format!("false||# don't\ncat <<EOF | sh\ndon't here\n{CHILD}\nEOF")),
        ("n46", format!("(# it's\ncat <<EOF | sh\nit's here\n{CHILD}\nEOF")),
        ("n52", format!("sleep 1 &# it's\ncat <<EOF | sh\nit's here\n{CHILD}\nEOF")),
        ("n51", format!("true;# it's\nx=\"'\n{CHILD}\n\"\neval \"$x\"")),
        ("n29", format!("x=`date`\"\n{CHILD}\n\"\neval \"$x\"")),
        ("n23", format!("cat <<EOF | sh\ntrue\nEOF\r\n{CHILD}\nEOF")),
    ];
    let silent: Vec<&str> = inputs.iter().filter(|(_, b)| !fires_on(b)).map(|(n, _)| *n).collect();
    assert!(silent.is_empty(), "silenced real findings: {silent:?}");
}

// Mask-off rule: the first thing the tracker does not understand turns the
// mask off for the rest of the file. CHILD on the last line must then report.
#[test]
fn test_GH336_first_unparsed_construct_turns_mask_off_for_the_rest() {
    let mut silent: Vec<&str> = Vec::new();
    for first in [
        "x=`date`",
        "echo `date",
        "x=$(\ndate\n)",
        "x=\"$(date\"",
        "cat <<EOF\nbody\nEOF",
        "cat <<NEVER_CLOSED\nbody",
        "cat <<'A' | sh\nx\nA",
        "echo $'a\\'b'",
        "echo \"open\nclosed\"",
        "echo 'open\nclosed'",
        "true # it's",
        "true # a <<B",
        "echo hi # \"q\"",
        "v=\"${y#'}\"",
        "echo ok\r",
        "echo 'unterminated",
    ] {
        silent.extend((!fires_on(&format!("{first}\n{CHILD}"))).then_some(first));
    }
    assert!(silent.is_empty(), "mask not turned off after: {silent:?}");
}

#[test]
fn test_GH336_plain_lines_before_the_first_unparsed_construct_stay_masked() {
    let s = format!("#!/usr/bin/env bash\ntrue # fine\n{CHILD}\ncat <<EOF\nx\nEOF\n");
    assert_eq!(sec010_errors(&s), 0);
}

// Pins words.rs: a backslash before the command is never tokenised, and a `)`
// ends a case pattern so the next word starts a command.
#[test]
fn test_GH336_backslash_assignment_word_is_not_a_child_shell_fires() {
    assert!(fires_on("D=a\\ sh -c 'cd \"$1\" && rm -rf out' _ /x"));
}

#[test]
fn test_GH336_case_pattern_paren_starts_a_command_is_masked() {
    let s = "#!/usr/bin/env bash\ncase x in a) sh -c 'cd \"$1\" && rm -rf out' _ /x;; esac\n";
    assert_eq!(sec010_errors(s), 0);
}
