//! GH-528: the child-shell mask applies only to a plain file. Every input in
//! `stay()` must keep reporting; the silent table must not report.

use super::*;

const C: &str = "sh -c 'cd \"$1\" && rm -rf out' _ /x";

fn reported_on_c_line(src: &str) -> bool {
    let idx = src
        .lines()
        .position(|l| l.contains("sh -c"))
        .unwrap_or(usize::MAX);
    check(src)
        .diagnostics
        .iter()
        .any(|d| d.code == "SEC010" && d.span.start_line == idx + 1)
}

fn stay() -> Vec<String> {
    let mut v: Vec<String> = vec![
        format!("x='a\nb'; cat <<EOF | sh\n{C}\nEOF\n"),
        format!("x=\"a\nb\"; cat <<EOF | sh\n{C}\nEOF\n"),
        format!("echo \"x\n\"; ssh host bash <<EOF\n{C}\nEOF\n"),
        format!("true;# it's\ncat <<EOF | sh\nit's here\n{C}\nEOF\n"),
        format!("false||# don't\ncat <<EOF | sh\nit's here\n{C}\nEOF\n"),
        format!("(# it's\ncat <<EOF | sh\nit's here\n{C}\nEOF\n"),
        format!("sleep 1 &# it's\ncat <<EOF | sh\nit's here\n{C}\nEOF\n"),
        "true;# it's\nx=\"'\nsh -c 'cd $1 && rm -rf out' _ x\n\"\neval \"$x\"\n".into(),
        "x=`date`\"\nsh -c 'cd $1 && rm -rf out' _ x\n\"\neval \"$x\"\n".into(),
        format!("cat <<EOF | sh\ntrue\nEOF\r\n{C}\nEOF\n"),
        format!("x=\"$(case a in a) echo \"\n{C}\n\" ;; esac)\"\neval \"$x\"\n"),
        format!("x=\"$(case a in a) echo \"\n{C}\n\";; esac)\"\n"),
        "v=\"${y:-x;'}\"\nsh -c ';cd \"$1\" && rm -rf out' _ /x\n'}\"\neval \"$v\"\n".into(),
        format!("v=\"${{y:='}}\"\n{C}\n'}}\"\n"),
        format!("v=\"${{y%'}}\"\n{C}\n'}}\"\n"),
        format!("v=\"${{y/'}}\"\n{C}\n'}}\"\n"),
    ];
    for p in [
        "echo `date`",
        "cat <<EOF >/dev/null\nEOF",
        "x=$(date)",
        "x=${y}",
        "x=$'a'",
        "x=$\"a\"",
        "echo a\\b",
        "echo a\r",
        "echo \u{e9}",
        "echo it's",
        "echo \"a",
        "echo \"a'b'c\"",
        "echo a #b",
        "echo $#",
    ] {
        v.push(format!("{p}\n{C}\n"));
    }
    v
}

#[test]
fn gh528_not_plain_files_report_as_before() {
    for (i, s) in stay().iter().enumerate() {
        assert!(reported_on_c_line(s), "case {i} must keep reporting: {s:?}");
    }
}

#[test]
fn gh528_plain_file_child_shell_positional_is_silent() {
    let silent = [
        "d=/srv/data\nsh -c 'cd \"$1\" && rm -rf out' _ \"$d\"\n",
        "bash -c 'cd \"$1\" && rm -rf out' _ /x\n",
        "#!/bin/sh\n# note\nsh -c 'mkdir -p \"$1\"/x && cp \"$2\" /dst' _ /a /b\n",
        "sh -c 'cd \"$@\" && rm -rf out' _ /x\n",
        "runuser -u svc -- sh -c 'cd \"$1\" && rm -rf out' _ /x\n",
    ];
    let noisy: Vec<_> = silent
        .iter()
        .filter(|s| !check(s).diagnostics.is_empty())
        .collect();
    assert!(noisy.is_empty(), "must be silent: {noisy:?}");
}

#[test]
fn gh528_plain_file_parent_positional_still_reports() {
    let s = "sh -c 'echo hi'; cd \"$1\" && rm -rf out\n";
    assert!(check(s).diagnostics.iter().any(|d| d.code == "SEC010"));
}
