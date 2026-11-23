//! Wrapper peeling for `bashrs parse`: `sudo`, `env`, `command`, `exec`,
//! `xargs`, `nohup`, `timeout`, and nested shells (`sh -c`, `bash -c`, `eval`).
//!
//! A wrapper whose option it does not know is an `Unparsed` error: guessing
//! where the wrapped command starts would let a command hide behind an option.

use super::shell_ast::*;
use super::shell_parse::Parser;

pub struct Analysis {
    pub wrappers: Vec<Wrapper>,
    pub exec_index: usize,
    pub embedded: Vec<Embedded>,
}

type R<T> = Result<T, Unparsed>;

#[derive(Default)]
struct Spec {
    with_arg: &'static str,
    opt_arg: &'static str,
    flags: &'static str,
    long_with_arg: &'static [&'static str],
    long_opt: &'static [&'static str],
    long_flags: &'static [&'static str],
}

const SHELLS: [&str; 6] = ["sh", "bash", "dash", "zsh", "ksh", "ash"];

fn bad<T>(line: usize, reason: String) -> R<T> {
    Err(Unparsed { line, reason })
}

fn value_at(argv: &[Word], i: usize) -> Option<String> {
    argv.get(i).and_then(|w| w.value())
}

/// A `--long` option at `argv[i]`; returns the index after it.
fn scan_long(
    argv: &[Word],
    i: usize,
    v: &str,
    spec: &Spec,
    name: &str,
    seen: &mut Vec<String>,
) -> R<usize> {
    let long = &v[2..];
    let (key, has_eq) = match long.split_once('=') {
        Some((k, _)) => (k, true),
        None => (long, false),
    };
    seen.push(v.to_string());
    if spec.long_flags.contains(&key) || spec.long_opt.contains(&key) {
        return Ok(i + 1);
    }
    if !spec.long_with_arg.contains(&key) {
        return bad(
            argv[i].line,
            format!("`{name}` option `--{key}` is not modelled"),
        );
    }
    if has_eq {
        return Ok(i + 1);
    }
    take_arg(argv, i, false, &format!("`{name} --{key}`"), seen)
}

/// The argument of an option that takes one: attached (`-n1`) or the next word.
fn take_arg(
    argv: &[Word],
    i: usize,
    attached: bool,
    what: &str,
    seen: &mut Vec<String>,
) -> R<usize> {
    if attached {
        return Ok(i + 1);
    }
    match argv.get(i + 1) {
        Some(a) => {
            seen.push(a.text.clone());
            Ok(i + 2)
        }
        None => bad(argv[i].line, format!("{what} has no argument")),
    }
}

/// A `-abc` option cluster at `argv[i]`; returns the index after it.
fn scan_short(
    argv: &[Word],
    i: usize,
    v: &str,
    spec: &Spec,
    name: &str,
    seen: &mut Vec<String>,
) -> R<usize> {
    seen.push(v.to_string());
    let chars: Vec<char> = v.chars().skip(1).collect();
    for (k, c) in chars.iter().enumerate() {
        if spec.with_arg.contains(*c) {
            return take_arg(
                argv,
                i,
                k + 1 < chars.len(),
                &format!("`{name} -{c}`"),
                seen,
            );
        }
        if spec.opt_arg.contains(*c) {
            return Ok(i + 1);
        }
        if !spec.flags.contains(*c) {
            return bad(
                argv[i].line,
                format!("`{name}` option `-{c}` is not modelled"),
            );
        }
    }
    Ok(i + 1)
}

/// Scans a wrapper's options starting at `start`; returns the index of the
/// first word that is not an option and the option words consumed.
fn scan(argv: &[Word], start: usize, spec: &Spec, name: &str) -> R<(usize, Vec<String>)> {
    let mut i = start;
    let mut seen = Vec::new();
    while let Some(v) = value_at(argv, i) {
        if v == "--" {
            i += 1;
            break;
        }
        if v.starts_with("--") {
            i = scan_long(argv, i, &v, spec, name, &mut seen)?;
        } else if v.len() > 1 && v.starts_with('-') {
            i = scan_short(argv, i, &v, spec, name, &mut seen)?;
        } else {
            break;
        }
    }
    Ok((i, seen))
}

fn is_assignment(v: &str) -> bool {
    match v.split_once('=') {
        Some((n, _)) => {
            let mut it = n.chars();
            matches!(it.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
                && it.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        None => false,
    }
}

fn skip_assignments(argv: &[Word], mut i: usize) -> (usize, Vec<String>) {
    let mut out = Vec::new();
    while let Some(v) = value_at(argv, i) {
        if !is_assignment(&v) {
            break;
        }
        out.push(argv[i].text.clone());
        i += 1;
    }
    (i, out)
}

fn peel(base: &str, argv: &[Word], idx: usize) -> R<Option<(usize, Wrapper)>> {
    let mk = |args: Vec<String>, assignments: Vec<String>, stdin_args: bool| Wrapper {
        name: base.to_string(),
        args,
        assignments,
        stdin_args,
    };
    let next_is = |s: &str| {
        value_at(argv, idx + 1)
            .map(|v| v.starts_with(s))
            .unwrap_or(false)
    };
    Ok(match base {
        "sudo" => {
            let spec = Spec {
                with_arg: "CDghpRrtTUu",
                flags: "AbEHKknPSvB",
                long_with_arg: &[
                    "user",
                    "group",
                    "host",
                    "prompt",
                    "chdir",
                    "chroot",
                    "role",
                    "type",
                    "close-from",
                    "command-timeout",
                    "other-user",
                ],
                long_opt: &["preserve-env"],
                long_flags: &[
                    "non-interactive",
                    "stdin",
                    "askpass",
                    "background",
                    "preserve-groups",
                    "set-home",
                    "reset-timestamp",
                    "remove-timestamp",
                    "validate",
                    "bell",
                ],
                ..Spec::default()
            };
            let (i, args) = scan(argv, idx + 1, &spec, "sudo")?;
            let (i, asg) = skip_assignments(argv, i);
            Some((i, mk(args, asg, false)))
        }
        "env" => {
            let spec = Spec {
                with_arg: "uC",
                flags: "i0v",
                long_with_arg: &["unset", "chdir"],
                long_flags: &["ignore-environment", "null", "debug"],
                ..Spec::default()
            };
            if next_is("-S") || next_is("--split-string") {
                return bad(argv[idx].line, "`env -S` is not modelled".to_string());
            }
            let (i, args) = scan(argv, idx + 1, &spec, "env")?;
            let (i, asg) = skip_assignments(argv, i);
            Some((i, mk(args, asg, false)))
        }
        "command" => {
            if next_is("-v") || next_is("-V") {
                return Ok(None);
            }
            let spec = Spec {
                flags: "p",
                ..Spec::default()
            };
            let (i, args) = scan(argv, idx + 1, &spec, "command")?;
            Some((i, mk(args, Vec::new(), false)))
        }
        "exec" => {
            let spec = Spec {
                with_arg: "a",
                flags: "cl",
                ..Spec::default()
            };
            let (i, args) = scan(argv, idx + 1, &spec, "exec")?;
            Some((i, mk(args, Vec::new(), false)))
        }
        "xargs" => {
            let spec = Spec {
                with_arg: "InPsEdaL",
                opt_arg: "ile",
                flags: "0rtpxoh",
                long_with_arg: &[
                    "max-args",
                    "max-procs",
                    "max-chars",
                    "delimiter",
                    "arg-file",
                    "process-slot-var",
                ],
                long_opt: &["replace", "eof", "max-lines"],
                long_flags: &[
                    "null",
                    "no-run-if-empty",
                    "verbose",
                    "interactive",
                    "exit",
                    "open-tty",
                    "show-limits",
                ],
            };
            let (i, args) = scan(argv, idx + 1, &spec, "xargs")?;
            Some((i, mk(args, Vec::new(), true)))
        }
        "nohup" => {
            let mut i = idx + 1;
            if value_at(argv, i).as_deref() == Some("--") {
                i += 1;
            }
            Some((i, mk(Vec::new(), Vec::new(), false)))
        }
        "timeout" => {
            let spec = Spec {
                with_arg: "sk",
                flags: "v",
                long_with_arg: &["signal", "kill-after"],
                long_flags: &["foreground", "preserve-status", "verbose"],
                ..Spec::default()
            };
            let (i, mut args) = scan(argv, idx + 1, &spec, "timeout")?;
            if i >= argv.len() {
                return Ok(None);
            }
            args.push(argv[i].text.clone());
            Some((i + 1, mk(args, Vec::new(), false)))
        }
        _ => None,
    })
}

fn parse_embedded(kind: &str, shell: &str, text: &str, line: usize) -> R<Embedded> {
    let mut p = Parser::new(text, line);
    let nodes = p.parse_program().map_err(|e| Unparsed {
        line: e.line,
        reason: format!("in {kind} string: {}", e.reason),
    })?;
    Ok(Embedded {
        kind: kind.to_string(),
        shell: shell.to_string(),
        nodes,
    })
}

struct ShellOpts {
    /// Index of the first word after the options.
    next: usize,
    c_seen: bool,
    /// A non-static word ended the options.
    dynamic: bool,
}

/// One short cluster (`-ec`, `+o`): whether it sets `-c`, and whether it
/// takes the following word (`-o name`).
fn shell_cluster(base: &str, v: &str, line: usize) -> R<(bool, bool)> {
    let (mut c_seen, mut takes_next) = (false, false);
    for c in v.chars().skip(1) {
        if c == 'c' && v.starts_with('-') {
            c_seen = true;
        } else if c == 'o' {
            takes_next = true;
        } else if !c.is_ascii_alphabetic() {
            return bad(line, format!("`{base}` option `{v}` is not modelled"));
        }
    }
    Ok((c_seen, takes_next))
}

/// One option word: `(sets -c, words consumed)`, or `None` when it is not an option.
fn shell_step(base: &str, v: &str, line: usize) -> R<Option<(bool, usize)>> {
    if let Some(long) = v.strip_prefix("--") {
        let n = if matches!(long, "rcfile" | "init-file") {
            2
        } else {
            1
        };
        return Ok(Some((false, n)));
    }
    if v.len() > 1 && (v.starts_with('-') || v.starts_with('+')) {
        let (c, takes_next) = shell_cluster(base, v, line)?;
        return Ok(Some((c, if takes_next { 2 } else { 1 })));
    }
    Ok(None)
}

fn shell_options(base: &str, argv: &[Word], idx: usize) -> R<ShellOpts> {
    let mut o = ShellOpts {
        next: idx + 1,
        c_seen: false,
        dynamic: false,
    };
    while o.next < argv.len() {
        let v = match argv[o.next].value() {
            Some(v) => v,
            None => {
                o.dynamic = true;
                break;
            }
        };
        if v == "--" {
            o.next += 1;
            break;
        }
        match shell_step(base, &v, argv[o.next].line)? {
            Some((c, step)) => {
                o.c_seen |= c;
                o.next += step;
            }
            None => break,
        }
    }
    Ok(o)
}

/// `sh -c SCRIPT`, `bash -lc SCRIPT`, `sh -o pipefail -c SCRIPT`.
fn shell_script(base: &str, argv: &[Word], idx: usize) -> R<Option<Embedded>> {
    let o = shell_options(base, argv, idx)?;
    if !o.c_seen {
        let later_c = o.dynamic
            && argv[o.next..]
                .iter()
                .filter_map(|w| w.value())
                .any(|v| v.starts_with('-') && !v.starts_with("--") && v.contains('c'));
        if later_c {
            return bad(
                argv[o.next].line,
                format!("`{base}` with a dynamic option before `-c` is not modelled"),
            );
        }
        return Ok(None);
    }
    let script = match argv.get(o.next) {
        Some(w) => w,
        None => return bad(argv[idx].line, format!("`{base} -c` has no command string")),
    };
    Ok(Some(parse_embedded(
        "sh-c",
        base,
        &script.template(),
        script.line,
    )?))
}

pub fn analyze(argv: &[Word]) -> R<Analysis> {
    let mut wrappers: Vec<Wrapper> = Vec::new();
    let mut idx = 0;
    while let Some(name) = value_at(argv, idx) {
        let base = name.rsplit('/').next().unwrap_or(&name).to_string();
        match peel(&base, argv, idx)? {
            Some((next, w)) if next < argv.len() => {
                wrappers.push(w);
                idx = next;
            }
            _ => break,
        }
    }
    let mut embedded = Vec::new();
    if let Some(name) = value_at(argv, idx) {
        let base = name.rsplit('/').next().unwrap_or(&name).to_string();
        if SHELLS.contains(&base.as_str()) {
            if let Some(e) = shell_script(&base, argv, idx)? {
                embedded.push(e);
            }
        } else if base == "eval" && idx + 1 < argv.len() {
            let text = argv[idx + 1..]
                .iter()
                .map(|w| w.template())
                .collect::<Vec<_>>()
                .join(" ");
            embedded.push(parse_embedded("eval", "eval", &text, argv[idx].line)?);
        }
    }
    Ok(Analysis {
        wrappers,
        exec_index: idx,
        embedded,
    })
}
