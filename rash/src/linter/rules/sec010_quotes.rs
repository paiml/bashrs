//! GH-336: quote scanner for SEC010.
//!
//! Only one text is hidden from the taint lookup: the single-quoted script
//! argument of a child shell, `sh -c '...$1...' _ "$d"`. There `$1` belongs to
//! the child shell. Text single-quoted after `eval`, `trap` or anything else
//! runs in THIS shell (or is unknown), so it stays visible.
//!
//! The scanner fails towards reporting: when it cannot classify the line with
//! certainty (unterminated span, backtick, unbalanced `$(`), nothing is masked.

const SHELLS: &[&str] = &["sh", "bash", "dash", "zsh", "ksh"];

#[derive(Clone, Copy, PartialEq)]
enum Ctx {
    Double,
    Paren,
}

/// Byte ranges (start of `'`, end after closing `'`) of top-level plain
/// single-quoted spans, or `None` if the line cannot be classified.
fn top_level_single_spans(line: &str) -> Option<Vec<(usize, usize)>> {
    let b = line.as_bytes();
    let mut stack: Vec<Ctx> = Vec::new();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let top = stack.last().copied();
        match (top, c) {
            (_, b'`') => return None,
            (_, b'\\') => i += 1,
            (Some(Ctx::Double), b'"') => {
                stack.pop();
            }
            (Some(Ctx::Double), b'$') if b.get(i + 1) == Some(&b'(') => {
                stack.push(Ctx::Paren);
                i += 1;
            }
            (Some(Ctx::Double), _) => {}
            (_, b'#') if i == 0 || b[i - 1].is_ascii_whitespace() => break,
            (_, b'"') => stack.push(Ctx::Double),
            (_, b'$') if b.get(i + 1) == Some(&b'(') => {
                stack.push(Ctx::Paren);
                i += 1;
            }
            (_, b'$') if b.get(i + 1) == Some(&b'\'') => {
                i = skip_ansi_c(b, i + 2)?;
            }
            (Some(Ctx::Paren), b'(') => stack.push(Ctx::Paren),
            (Some(Ctx::Paren), b')') => {
                stack.pop();
            }
            (_, b'\'') => {
                let end = b[i + 1..].iter().position(|&x| x == b'\'')? + i + 2;
                if stack.is_empty() {
                    spans.push((i, end));
                }
                i = end - 1;
            }
            _ => {}
        }
        i += 1;
    }
    if stack.is_empty() {
        Some(spans)
    } else {
        None
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

/// `before` ends with `<shell> [-flags...] -c` (the `-c` word may be `-ec`...).
fn follows_child_shell_c(before: &str) -> bool {
    let mut words = before.split_whitespace().rev();
    let is_c_flag = |w: &str| {
        w.len() >= 2
            && w.starts_with('-')
            && !w.starts_with("--")
            && w[1..].bytes().all(|x| x.is_ascii_alphabetic())
            && w.ends_with('c')
    };
    if !words.next().is_some_and(is_c_flag) {
        return false;
    }
    let shell = words.find(|w| !w.starts_with('-'));
    shell.is_some_and(|w| SHELLS.contains(&w.rsplit('/').next().unwrap_or(w)))
}

/// Blank `$` inside the single-quoted script of child-shell `-c` calls.
/// Byte columns are preserved.
pub(super) fn mask_child_shell_scripts(line: &str) -> String {
    let Some(spans) = top_level_single_spans(line) else {
        return line.to_string();
    };
    let b = line.as_bytes();
    let mut out = b.to_vec();
    for (start, end) in spans {
        let word_start = start == 0 || b[start - 1].is_ascii_whitespace();
        let word_end = end >= b.len() || matches!(b[end], b' ' | b'\t' | b';' | b'&' | b'|' | b')');
        if word_start && word_end && follows_child_shell_c(&line[..start]) {
            for x in &mut out[start..end] {
                if *x == b'$' {
                    *x = b' ';
                }
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| line.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_sh_c_body() {
        let m = mask_child_shell_scripts("sh -c 'cd \"$1\"' _ \"$d\"");
        assert_eq!(m, "sh -c 'cd \"\u{20}1\"' _ \"$d\"");
    }

    #[test]
    fn test_mask_other_shells_and_flags() {
        for s in ["bash", "dash", "zsh", "ksh", "/bin/sh"] {
            let l = format!("{s} -c 'x $1' _");
            assert!(!mask_child_shell_scripts(&l).contains('$'), "{l}");
        }
        assert!(!mask_child_shell_scripts("sh -e -c 'x $1' _").contains('$'));
        assert!(!mask_child_shell_scripts("bash -lc 'x $1' _").contains('$'));
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
            assert_eq!(mask_child_shell_scripts(l), l, "{l}");
        }
    }

    #[test]
    fn test_no_mask_when_line_uncertain() {
        for l in [
            "sh -c 'x $1' _ 'unterminated",
            "sh -c 'x $1' _ `echo '",
            "sh -c 'x $1' _ \"$(echo \"it's\")",
        ] {
            assert_eq!(mask_child_shell_scripts(l), l, "{l}");
        }
    }

    #[test]
    fn test_scanner_quote_cases() {
        let spans = |l: &str| top_level_single_spans(l);
        assert_eq!(spans("echo can\\'t; cd x"), Some(vec![]));
        assert_eq!(spans("rm $'a\\'b' x"), Some(vec![]));
        assert_eq!(spans("rm \"$(echo \"it's\")\" x"), Some(vec![]));
        assert_eq!(spans("echo \"\\\"it's\"; rm x"), Some(vec![]));
        assert_eq!(spans("a 'b' # it's"), Some(vec![(2, 5)]));
        assert_eq!(spans("a 'b"), None);
    }
}
