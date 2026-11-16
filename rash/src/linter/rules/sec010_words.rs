//! GH-336: is a `-c` word the head of a simple command?
//!
//! Allow-list for the SEC010 child-shell mask: the shell word must be the FIRST
//! word of its simple command (start of line, or after `;` `&` `&&` `||` `|`
//! `(` `)` `{` `!` `then` `do` `else`), optionally after plain `VAR=value`
//! words. Anything else (`eval`, `echo`, `ssh`, `sudo`, `env`, ...) is not
//! masked. When the text before the span cannot be tokenised with certainty,
//! the answer is `false`.

const SHELLS: &[&str] = &["sh", "bash", "dash", "zsh", "ksh"];
const KEYWORDS: &[&str] = &["then", "do", "else", "!", "{"];

/// Words of the last simple command in `before`, or `None` when uncertain.
fn last_command_words(before: &str) -> Option<Vec<String>> {
    let b = before.as_bytes();
    let mut words: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut in_word = false;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' | b'`' => return None,
            b'$' if b.get(i + 1) == Some(&b'(') => return None,
            b' ' | b'\t' => flush(&mut words, &mut cur, &mut in_word),
            b';' | b'&' | b'|' | b'(' | b')' => {
                flush(&mut words, &mut cur, &mut in_word);
                words.clear();
            }
            q @ (b'\'' | b'"') => {
                let rel = b[i + 1..].iter().position(|&x| x == q)?;
                let body = &before[i + 1..i + 1 + rel];
                if q == b'"' && (body.contains(['\\', '`']) || body.contains("$(")) {
                    return None;
                }
                cur.push_str(body);
                in_word = true;
                i += rel + 1;
            }
            c => {
                cur.push(c as char);
                in_word = true;
            }
        }
        i += 1;
    }
    flush(&mut words, &mut cur, &mut in_word);
    Some(words)
}

fn flush(words: &mut Vec<String>, cur: &mut String, in_word: &mut bool) {
    if *in_word {
        words.push(std::mem::take(cur));
        *in_word = false;
    }
}

fn is_assignment(w: &str) -> bool {
    match w.split_once('=') {
        Some((name, _)) => {
            !name.is_empty()
                && !name.starts_with(|c: char| c.is_ascii_digit())
                && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
        }
        None => false,
    }
}

fn is_c_flag(w: &str) -> bool {
    w.len() >= 2
        && w.starts_with('-')
        && !w.starts_with("--")
        && w[1..].bytes().all(|x| x.is_ascii_alphabetic())
        && w.ends_with('c')
}

/// `before` is the text in front of a single-quoted span. True when it is
/// `[VAR=value ...] <shell> [-flags ...] -c` forming a whole simple command.
pub(super) fn starts_child_shell_c(before: &str) -> bool {
    let Some(words) = last_command_words(before) else {
        return false;
    };
    let mut it = words.iter().map(String::as_str).peekable();
    while it.next_if(|w| KEYWORDS.contains(w)).is_some() {}
    while it.next_if(|w| is_assignment(w)).is_some() {}
    let Some(shell) = it.next() else {
        return false;
    };
    if !SHELLS.contains(&shell.rsplit('/').next().unwrap_or(shell)) {
        return false;
    }
    let flags: Vec<&str> = it.collect();
    flags.iter().all(|w| w.starts_with('-')) && flags.last().is_some_and(|w| is_c_flag(w))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_heads() {
        for b in [
            "sh -c ",
            "/bin/sh -c ",
            "bash -lc ",
            "bash --norc -c ",
            "sh -e -c ",
            "true; sh -c ",
            "a && sh -c ",
            "a || sh -c ",
            "a | sh -c ",
            "( sh -c ",
            "{ sh -c ",
            "! sh -c ",
            "if a; then sh -c ",
            "for x in y; do sh -c ",
            "D=1 sh -c ",
            "D=\"a b\" E=2 sh -c ",
        ] {
            assert!(starts_child_shell_c(b), "{b}");
        }
    }

    #[test]
    fn test_refused_heads() {
        for b in [
            "eval sh -c ",
            "echo sh -c ",
            "printf sh -c ",
            "ssh host sh -c ",
            "sudo sh -c ",
            "env sh -c ",
            "xargs sh -c ",
            "exec sh -c ",
            "command sh -c ",
            "time sh -c ",
            "nohup sh -c ",
            "runuser -u x -- sh -c ",
            "sh -o errexit -c ",
            "sh ",
            "perl -c ",
            "x=$(sh -c ",
            "x=`sh -c ",
            "sh \\\n -c ",
            "echo \"a;b\" sh -c ",
            "'unterminated sh -c ",
        ] {
            assert!(!starts_child_shell_c(b), "{b}");
        }
    }
}
