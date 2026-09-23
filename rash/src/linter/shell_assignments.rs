//! Variable assignments located by word POSITION (GH-370).
//!
//! Rules used the line regex `\w+\s*=\s*…` to find an assignment, which let `=`
//! and the value sit in DIFFERENT shell words — so the empty `ps` format `pid=`
//! followed by the long option `--ppid` read as `pid=--ppid` (SC2210), and
//! `cmd --out=\`pwd\`` read as an assignment to `out` (SC2225). Whether a word
//! is an assignment depends on where it sits in its simple command, which is
//! exactly what [`crate::linter::shell_words`] already computes.

use crate::linter::shell_words::{simple_commands, ShellWord, SimpleCommand, WordRole};

/// Builtins whose `NAME=value` operands are assignments, not argument text.
const DECLARATION_BUILTINS: &[&str] = &["declare", "local", "export", "readonly", "typeset"];

/// One variable assignment found on a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    /// 1-indexed byte column of the `NAME`.
    pub col: usize,
    /// The variable name.
    pub name: String,
    /// The value exactly as written (quotes and all).
    pub value: String,
}

/// Every assignment on `line`. A word is an assignment when it is:
///
/// 1. a `NAME=value` prefix before the command name (`x=1 cmd`, `x=1`);
/// 2. a `NAME=value` operand of `declare`/`local`/`export`/`readonly`/`typeset`.
///
/// Two spaced *attempts* at an assignment are also returned, because the rules
/// built on this exist to catch them and fired on them before:
///
/// 3. `NAME = value` — the command is `NAME`, its first argument a bare `=`;
/// 4. `NAME= value` — an empty prefix, and `value` is then run as the command.
///
/// Command substitutions are analysed as their own commands, as in
/// [`simple_commands`].
///
/// ```
/// use bashrs::linter::shell_assignments::assignments;
///
/// let found = assignments(r#"c="$(ps -o pid= --ppid "$p")""#);
/// assert!(found.iter().all(|a| a.name == "c"));
/// assert_eq!(assignments("local n=--total")[0].value, "--total");
/// ```
pub fn assignments(line: &str) -> Vec<Assignment> {
    let mut out = Vec::new();
    for cmd in simple_commands(line) {
        collect(&cmd, &mut out);
    }
    out
}

fn collect(cmd: &SimpleCommand, out: &mut Vec<Assignment>) {
    let declares = cmd
        .name
        .as_deref()
        .is_some_and(|n| DECLARATION_BUILTINS.contains(&n));
    for (i, w) in cmd.words.iter().enumerate() {
        let next = cmd.words.get(i + 1);
        match w.role {
            WordRole::AssignPrefix => push_word_assignment(w, next, out),
            WordRole::Argument if declares => push_word_assignment(w, None, out),
            WordRole::CommandName => push_spaced_assignment(w, next, cmd.words.get(i + 2), out),
            _ => {}
        }
    }
}

fn is_name(s: &str) -> bool {
    let mut bytes = s.bytes();
    bytes
        .next()
        .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Shapes 1, 2 and 4: `NAME=value`, or `NAME=` followed by the command word.
fn push_word_assignment(w: &ShellWord, next: Option<&ShellWord>, out: &mut Vec<Assignment>) {
    let Some((name, value)) = w.raw.split_once('=') else {
        return;
    };
    if !is_name(name) {
        return;
    }
    let value = match next {
        Some(n) if value.is_empty() && n.role == WordRole::CommandName => n.raw.as_str(),
        _ => value,
    };
    out.push(Assignment {
        col: w.col,
        name: name.to_string(),
        value: value.to_string(),
    });
}

/// Shape 3: `NAME = value`.
fn push_spaced_assignment(
    w: &ShellWord,
    eq: Option<&ShellWord>,
    value: Option<&ShellWord>,
    out: &mut Vec<Assignment>,
) {
    if let (true, Some(eq), Some(value)) = (is_name(&w.raw), eq, value) {
        if eq.raw == "=" {
            out.push(Assignment {
                col: w.col,
                name: w.raw.clone(),
                value: value.raw.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(line: &str) -> Vec<(String, String)> {
        assignments(line)
            .into_iter()
            .map(|a| (a.name, a.value))
            .collect()
    }

    fn pair(n: &str, v: &str) -> (String, String) {
        (n.to_string(), v.to_string())
    }

    #[test]
    fn test_gh370_argument_words_are_not_assignments() {
        assert_eq!(values("ps -o pid= --ppid 1"), vec![]);
        assert_eq!(values("git log --format=--x"), vec![]);
        assert_eq!(values(r#"echo "x=--y""#), vec![]);
    }

    #[test]
    fn test_gh370_every_assignment_shape() {
        assert_eq!(values("x=1 y=2 cmd"), vec![pair("x", "1"), pair("y", "2")]);
        assert_eq!(values("local -r n=--t"), vec![pair("n", "--t")]);
        assert_eq!(values("x = ++y"), vec![pair("x", "++y")]);
        assert_eq!(values("x= --y"), vec![pair("x", "--y")]);
        assert_eq!(
            values(r#"o="$(z=3)""#),
            vec![pair("o", r#""$(z=3)""#), pair("z", "3")]
        );
    }
}
