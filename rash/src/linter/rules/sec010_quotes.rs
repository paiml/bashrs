//! GH-336: quote scanner for SEC010.
//!
//! ALLOW-LIST. Only one text is hidden from the taint lookup: the
//! single-quoted script argument of a child shell, `sh -c '...$1...' _ /x`,
//! and only when ALL hold (otherwise nothing is masked, as before):
//! - the line is outside every here-doc body and starts at top level, not
//!   inside a quoted string opened on an earlier line (see `sec010_ctx`);
//! - no `\` continuation before or on the command;
//! - the `-c` word belongs to the FIRST word of a simple command, after only
//!   plain `VAR=value` words (see `sec010_words`): not `eval`, `echo`, `ssh`,
//!   `sudo`, `env`, `xargs`, `exec`, `command`, `time`, `nohup`, ...;
//! - the outer arguments do not mention `$0`;
//! - the span is top level on its line (not inside `$( )` or `"..."`).
//!
//! The scanner fails towards reporting: when it cannot classify the line with
//! certainty nothing is masked.

use super::sec010_words::starts_child_shell_c;

/// Quote state carried from one line to the next.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Carry {
    Top,
    Single,
    Double,
    Unknown,
}

/// Result of scanning one line from top-level state.
#[derive(PartialEq, Debug)]
pub(super) enum Scan {
    /// Byte ranges (start of `'`, end after closing `'`) of top-level plain
    /// single-quoted spans; the line ends at top level.
    Spans(Vec<(usize, usize)>),
    /// This line cannot be classified, the carried state is unaffected.
    Uncertain,
    /// The line ends inside a quote opened on it.
    Open(Carry),
}

#[derive(Clone, Copy, PartialEq)]
enum Ctx {
    Double,
    Paren,
}

pub(super) fn top_level_single_spans(line: &str) -> Scan {
    if let Some(scan) = backtick_scan(line) {
        return scan;
    }
    let b = line.as_bytes();
    let mut stack: Vec<Ctx> = Vec::new();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match step(b, i, &mut stack, &mut spans) {
            Ok(Some(next)) => i = next + 1,
            Ok(None) => break,
            Err(open) => return Scan::Open(open),
        }
    }
    end_scan(&stack, spans)
}

/// Consume the token at `i`. `Ok(Some(j))`: continue after `j`; `Ok(None)`:
/// the rest of the line is a comment; `Err`: unterminated quote.
fn step(
    b: &[u8],
    i: usize,
    stack: &mut Vec<Ctx>,
    spans: &mut Vec<(usize, usize)>,
) -> Result<Option<usize>, Carry> {
    if stack.last() == Some(&Ctx::Double) {
        return Ok(Some(in_double(b, i, stack)));
    }
    let next = b.get(i + 1).copied();
    match b[i] {
        b'\\' => return Ok(Some(i + 1)),
        b'#' if i == 0 || b[i - 1].is_ascii_whitespace() => return Ok(None),
        b'"' => stack.push(Ctx::Double),
        b'$' if next == Some(b'(') => {
            stack.push(Ctx::Paren);
            return Ok(Some(i + 1));
        }
        b'$' if next == Some(b'\'') => return skip_ansi_c(b, i + 2).map(Some).ok_or(Carry::Unknown),
        b'(' if stack.last() == Some(&Ctx::Paren) => stack.push(Ctx::Paren),
        b')' if stack.last() == Some(&Ctx::Paren) => {
            stack.pop();
        }
        b'\'' => {
            let (end, span) = single_span(b, i, stack.is_empty())?;
            spans.extend(span);
            return Ok(Some(end - 1));
        }
        _ => {}
    }
    Ok(Some(i))
}

/// One step inside a double-quoted string; returns the index consumed.
fn in_double(b: &[u8], i: usize, stack: &mut Vec<Ctx>) -> usize {
    match b[i] {
        b'\\' => return i + 1,
        b'"' => {
            stack.pop();
        }
        b'$' if b.get(i + 1) == Some(&b'(') => {
            stack.push(Ctx::Paren);
            return i + 1;
        }
        _ => {}
    }
    i
}

/// A backtick makes the line uncertain; an odd count poisons later lines.
fn backtick_scan(line: &str) -> Option<Scan> {
    if !line.contains('`') {
        return None;
    }
    let odd = line.bytes().filter(|&x| x == b'`').count() % 2 == 1;
    Some(if odd { Scan::Open(Carry::Unknown) } else { Scan::Uncertain })
}

/// Plain single-quoted span starting at `i`: (end, span if top level), or the
/// carried state when it is unterminated.
fn single_span(b: &[u8], i: usize, top: bool) -> Result<(usize, Option<(usize, usize)>), Carry> {
    let Some(rel) = b[i + 1..].iter().position(|&x| x == b'\'') else {
        return Err(if top { Carry::Single } else { Carry::Unknown });
    };
    let end = rel + i + 2;
    Ok((end, top.then_some((i, end))))
}

fn end_scan(stack: &[Ctx], spans: Vec<(usize, usize)>) -> Scan {
    match stack {
        [] => Scan::Spans(spans),
        [Ctx::Double] => Scan::Open(Carry::Double),
        _ => Scan::Open(Carry::Unknown),
    }
}

/// Index of the closing `'` of an ANSI-C quote whose body starts at `i`.
fn skip_ansi_c(b: &[u8], mut i: usize) -> Option<usize> {
    while i < b.len() {
        match b[i] {
            b'\\' => i += 1,
            b'\'' => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Blank the `$` of positional references (`$0`-`$9`, `${N`, `$@`, `$*`,
/// `$#`) in `out[start..end]`. Named variables stay visible: a child shell reads
/// exported ones, so a tainted one must still be reported.
fn blank_positionals(out: &mut [u8], start: usize, end: usize) {
    for i in start..end {
        if out[i] != b'$' || i + 1 >= end {
            continue;
        }
        let n = out[i + 1];
        let braced_digit = n == b'{' && out.get(i + 2).is_some_and(|d| d.is_ascii_digit());
        if n.is_ascii_digit() || matches!(n, b'@' | b'*' | b'#') || braced_digit {
            out[i] = b' ';
        }
    }
}

/// Blank positional `$` inside the single-quoted script of child-shell `-c`
/// calls. Byte columns are preserved. `blocked` (a `\` continuation, a here-doc
/// body, a line that starts inside a multi-line quote) means: never mask.
pub(super) fn mask_child_shell_scripts(line: &str, blocked: bool) -> String {
    if blocked {
        return line.to_string();
    }
    let Scan::Spans(spans) = top_level_single_spans(line) else {
        return line.to_string();
    };
    let b = line.as_bytes();
    let mut out = b.to_vec();
    for (start, end) in spans {
        let word_start = start == 0 || b[start - 1].is_ascii_whitespace();
        let word_end = end >= b.len() || matches!(b[end], b' ' | b'\t' | b';' | b'&' | b'|' | b')');
        let outer_zero = line[end..].contains("$0") || line[end..].contains("${0");
        if word_start && word_end && !outer_zero && starts_child_shell_c(&line[..start]) {
            blank_positionals(&mut out, start, end);
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| line.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_sh_c_body() {
        let m = mask_child_shell_scripts("sh -c 'cd \"$1\"' _ \"$d\"", false);
        assert_eq!(m, "sh -c 'cd \"\u{20}1\"' _ \"$d\"");
    }

    #[test]
    fn test_mask_other_shells_and_flags() {
        for s in ["bash", "dash", "zsh", "ksh", "/bin/sh"] {
            let l = format!("{s} -c 'x $1' _");
            assert!(!mask_child_shell_scripts(&l, false).contains('$'), "{l}");
        }
        assert!(!mask_child_shell_scripts("sh -e -c 'x $1' _", false).contains('$'));
        assert!(!mask_child_shell_scripts("bash -lc 'x $1' _", false).contains('$'));
    }

    #[test]
    fn test_no_mask_eval_trap_echo() {
        for l in [
            "eval 'cd \"$1\"'",
            "trap 'rm -rf \"$W\"' EXIT",
            "echo 'cd $1'",
            "sh 'x $1'",
            "perl -c 'x $1'",
        ] {
            assert_eq!(mask_child_shell_scripts(l, false), l, "{l}");
        }
    }

    #[test]
    fn test_no_mask_when_line_uncertain() {
        for l in [
            "sh -c 'x $1' _ 'unterminated",
            "sh -c 'x $1' _ `echo '",
            "sh -c 'x $1' _ \"$(echo \"it's\")",
        ] {
            assert_eq!(mask_child_shell_scripts(l, false), l, "{l}");
        }
    }

    #[test]
    fn test_scanner_quote_cases() {
        let spans = |l: &str| match top_level_single_spans(l) {
            Scan::Spans(s) => Some(s),
            _ => None,
        };
        assert_eq!(spans("echo can\\'t; cd x"), Some(vec![]));
        assert_eq!(spans("rm $'a\\'b' x"), Some(vec![]));
        assert_eq!(spans("rm \"$(echo \"it's\")\" x"), Some(vec![]));
        assert_eq!(spans("echo \"\\\"it's\"; rm x"), Some(vec![]));
        assert_eq!(spans("a 'b' # it's"), Some(vec![(2, 5)]));
        assert_eq!(spans("a 'b"), None);
    }

    #[test]
    fn test_mask_blanks_only_positionals() {
        let m = mask_child_shell_scripts(
            "sh -c 'a $0 $9 $@ $* $# ${1} ${2:-x} $D ${D} ${#D} $$' _",
            false,
        );
        assert_eq!(m, "sh -c 'a  0  9  @  *  #  {1}  {2:-x} $D ${D} ${#D} $$' _");
    }

    #[test]
    fn test_no_mask_when_command_continues() {
        let l = "sh -c 'cd \"$1\"' \\";
        assert_eq!(mask_child_shell_scripts(l, true), l);
    }

    #[test]
    fn test_no_mask_child_shell_inside_command_substitution() {
        let l = "x=\"$( sh -c 'cd \"$1\"' _ )\"";
        assert_eq!(mask_child_shell_scripts(l, false), l);
    }

    #[test]
    fn test_no_mask_non_c_flag() {
        let l = "sh -e 'x $1' _";
        assert_eq!(mask_child_shell_scripts(l, false), l);
    }

    #[test]
    fn test_no_mask_span_glued_to_neighbouring_word() {
        for l in ["sh -c'x $1' _", "sh -c 'x $1'y _"] {
            assert_eq!(mask_child_shell_scripts(l, false), l, "{l}");
        }
    }
}
