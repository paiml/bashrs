//! SC2086 in heredoc bodies (bashrs#524).
//!
//! Plain text in an unquoted heredoc body is not word-split, so `$name` there
//! is not an SC2086 finding. Inside a `$( )` or backtick command substitution
//! in that body, splitting does apply (the substitution may span lines).
//! Arithmetic `$(( ))` and escaped openers stay quiet.

use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq)]
enum Ctx {
    Cmd,
    Arith,
    Paren,
}

/// Byte offsets (0-indexed, per 1-indexed line) that lie inside a command
/// substitution, for the unquoted heredoc body lines of `source`.
pub fn body_command_substitution_bytes(source: &str) -> HashMap<usize, HashSet<usize>> {
    use crate::linter::quoting::{heredoc_body_lines, quoted_heredoc_lines};
    let any = heredoc_body_lines(source);
    let quoted = quoted_heredoc_lines(source);
    let mut out = HashMap::new();
    let mut stack: Vec<Ctx> = Vec::new();
    let mut backtick = false;
    for (idx, line) in source.lines().enumerate() {
        let n = idx + 1;
        if !any.contains(&n) {
            stack.clear();
            backtick = false;
            continue;
        }
        if quoted.contains(&n) {
            out.insert(n, HashSet::new());
            continue;
        }
        let mut inside = HashSet::new();
        scan_line(line.as_bytes(), &mut stack, &mut backtick, &mut inside);
        out.insert(n, inside);
    }
    out
}

fn in_cmd(stack: &[Ctx], backtick: bool) -> bool {
    stack.iter().rev().find(|c| **c != Ctx::Paren) == Some(&Ctx::Cmd)
        || (backtick && !stack.contains(&Ctx::Arith))
}

fn scan_line(b: &[u8], stack: &mut Vec<Ctx>, backtick: &mut bool, inside: &mut HashSet<usize>) {
    let mut i = 0;
    while i < b.len() {
        if in_cmd(stack, *backtick) {
            inside.insert(i);
        }
        i += step(b, i, stack, backtick);
    }
}

/// Handle the token at `i`; returns how many bytes it consumed.
fn step(b: &[u8], i: usize, stack: &mut Vec<Ctx>, backtick: &mut bool) -> usize {
    match b[i] {
        b'\\' => 2,
        b'`' => {
            if !stack.contains(&Ctx::Arith) {
                *backtick = !*backtick;
            }
            1
        }
        b'$' => open_dollar(b, i, stack),
        b'(' => {
            if !stack.is_empty() {
                stack.push(Ctx::Paren);
            }
            1
        }
        b')' => close_paren(b, i, stack),
        _ => 1,
    }
}

fn open_dollar(b: &[u8], i: usize, stack: &mut Vec<Ctx>) -> usize {
    if b.get(i + 1) != Some(&b'(') {
        return 1;
    }
    if b.get(i + 2) == Some(&b'(') {
        stack.push(Ctx::Arith);
        return 3;
    }
    stack.push(Ctx::Cmd);
    2
}

fn close_paren(b: &[u8], i: usize, stack: &mut Vec<Ctx>) -> usize {
    let closes_arith = stack.pop() == Some(Ctx::Arith) && b.get(i + 1) == Some(&b')');
    if closes_arith {
        2
    } else {
        1
    }
}
