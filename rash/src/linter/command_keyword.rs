//! Shell keywords and builtins (`local`, `return`, `exit`) found in COMMAND position (GH-438).
//!
//! Rules that matched `\blocal\s+` on a raw line fired on prose: Make recipe comments
//! (`@# touches the local registry`), arguments of `:` or `echo`, text after a quote. This module
//! asks the question those rules skipped, using [`shell_words::simple_commands`]: is the word the
//! NAME of a simple command? Make recipe prefixes (`@`, `-`, `+`) are blanked first, and
//! backslash-continued lines are read as one logical line.

use crate::linter::shell_words::{simple_commands, WordRole};

/// One command-position occurrence of a keyword.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordHit {
    /// 1-indexed physical line of the keyword.
    pub line: usize,
    /// 1-indexed byte column of the keyword on that line.
    pub col: usize,
    /// The first argument, if any: its literal text and the 1-indexed (line, column) just past it.
    pub first_arg: Option<(String, usize, usize)>,
}

/// A logical line and where each of its pieces came from.
struct Logical {
    text: String,
    /// `(offset in text, physical line index)`; every piece starts at column 1 of its line.
    pieces: Vec<(usize, usize)>,
}

impl Logical {
    fn locate(&self, off: usize) -> (usize, usize) {
        let (start, line) = self
            .pieces
            .iter()
            .rev()
            .find(|p| p.0 <= off)
            .copied()
            .unwrap_or((0, 0));
        (line + 1, off - start + 1)
    }
}

/// Blank the leading Make recipe prefixes (`@`, `-`, `+`), keeping every offset.
fn blank_recipe_prefix(text: &mut String) {
    let lead = text.len() - text.trim_start().len();
    let run = text[lead..]
        .bytes()
        .take_while(|b| matches!(b, b'@' | b'-' | b'+'))
        .count();
    text.replace_range(lead..lead + run, &" ".repeat(run));
}

/// Is this preprocessed Make recipe text? `preprocess_for_linting` blanks every non-recipe line, so
/// recipe text has no non-blank line at column 1 (a continued line may be indented with spaces).
fn is_make_recipe_text(source: &str) -> bool {
    let mut after_cont = false;
    let mut any_tab = false;
    for raw in source.lines() {
        let cont = matches!(raw.strip_suffix('\\'), Some(b) if !b.ends_with('\\'));
        if !after_cont && !raw.trim().is_empty() {
            if !raw.starts_with('\t') {
                return false;
            }
            any_tab = true;
        }
        after_cont = cont;
    }
    any_tab
}

/// Does this line, after its Make recipe prefixes, begin a comment?
fn is_comment_line(raw: &str) -> bool {
    raw.trim_start()
        .trim_start_matches(['@', '-', '+'])
        .trim_start()
        .starts_with('#')
}

/// Join backslash-continued physical lines (the `\` and the newline are dropped, as the shell does).
/// A shell comment never continues, so `# note \` leaves the next line alone; in a Make recipe the
/// backslash continues the comment too.
fn logical_lines(source: &str) -> Vec<Logical> {
    let mut out = Vec::new();
    let mut cur: Option<Logical> = None;
    let make = is_make_recipe_text(source);
    for (idx, raw) in source.lines().enumerate() {
        let (body, mut continued) = match raw.strip_suffix('\\') {
            Some(b) if !b.ends_with('\\') => (b, true),
            _ => (raw, false),
        };
        if continued && !make && cur.is_none() && is_comment_line(raw) {
            continued = false;
        }
        let l = cur.get_or_insert_with(|| Logical {
            text: String::new(),
            pieces: Vec::new(),
        });
        l.pieces.push((l.text.len(), idx));
        l.text.push_str(body);
        if !continued {
            if let Some(mut done) = cur.take() {
                if make {
                    blank_recipe_prefix(&mut done.text);
                }
                out.push(done);
            }
        }
    }
    if let Some(mut done) = cur {
        if make {
            blank_recipe_prefix(&mut done.text);
        }
        out.push(done);
    }
    out
}

/// Every place `keyword` is the name of a simple command in `source`.
pub fn command_keyword_hits(source: &str, keyword: &str) -> Vec<KeywordHit> {
    let mut hits = Vec::new();
    for logical in logical_lines(source) {
        for cmd in simple_commands(&logical.text) {
            // Reserved words (`then`, `{`) and assignment prefixes come before the name.
            let mut words = cmd
                .words
                .iter()
                .skip_while(|w| w.role != WordRole::CommandName);
            let Some(name) = words.next() else { continue };
            if name.literal != keyword {
                continue;
            }
            let (line, col) = logical.locate(name.col - 1);
            let first_arg = words.next().map(|a| {
                let (end_line, last) = logical.locate(a.col - 1 + a.raw.len() - 1);
                (a.literal.clone(), end_line, last + 1)
            });
            hits.push(KeywordHit {
                line,
                col,
                first_arg,
            });
        }
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(src: &str, kw: &str) -> Vec<usize> {
        command_keyword_hits(src, kw)
            .iter()
            .map(|h| h.line)
            .collect()
    }

    #[test]
    fn test_gh438_prose_is_not_command_position() {
        for src in [
            "\t@# touches the local registry",
            "# local x",
            "\t-# local a",
            "\t+# local a",
            "\t@-# local b",
            "\t: touches the local docker registry, which; \\\n\t  more",
            "\techo local x",
            "echo \"a\" local x",
            "echo 'a' \"b\" local x",
        ] {
            assert_eq!(lines(src, "local"), Vec::<usize>::new(), "{src:?}");
        }
    }

    #[test]
    fn test_gh438_real_command_position_still_found() {
        assert_eq!(lines("\tlocal x=1", "local"), vec![1]);
        assert_eq!(lines("\t@local x=1", "local"), vec![1]);
        assert_eq!(lines("true; local x", "local"), vec![1]);
        assert_eq!(lines("if true; then local x; fi", "local"), vec![1]);
        assert_eq!(lines("x=1 local y", "local"), vec![1]);
        assert_eq!(lines("echo $(local y)", "local"), vec![1]);
    }

    #[test]
    fn test_gh438_continued_line_is_one_logical_line() {
        let src = "echo a \\\n  local x\nlocal y \\\n  z";
        assert_eq!(lines(src, "local"), vec![3]);
        let hits = command_keyword_hits("true; \\\n  local x", "local");
        assert_eq!((hits[0].line, hits[0].col), (2, 3));
    }

    #[test]
    fn test_gh438_columns_and_first_arg() {
        let h = command_keyword_hits("\t@return 256", "return");
        assert_eq!((h[0].line, h[0].col), (1, 3));
        assert_eq!(h[0].first_arg, Some(("256".to_string(), 1, 13)));
        assert_eq!(command_keyword_hits("exit", "exit")[0].first_arg, None);
    }

    #[test]
    fn test_gh438_shell_comment_does_not_continue() {
        assert_eq!(lines("# note \\\nlocal x=1", "local"), vec![2]);
        assert_eq!(lines("  # note \\\nlocal x=1", "local"), vec![2]);
        assert_eq!(lines("echo a\n# note \\\nlocal x=1", "local"), vec![3]);
    }

    #[test]
    fn test_gh438_make_recipe_comment_continues() {
        // preprocessed Make text: every non-blank line starts with a tab (or continues one)
        assert_eq!(
            lines("\n\t@# note \\\n  local x=1\n", "local"),
            Vec::<usize>::new()
        );
        assert_eq!(
            lines("\n\t# note \\\n\tlocal x=1\n", "local"),
            Vec::<usize>::new()
        );
        // the comment ends with the continuation; the next recipe line is a command again
        assert_eq!(
            lines("\n\t@# note \\\n  more\n\tlocal x=1\n", "local"),
            vec![4]
        );
    }

    #[test]
    fn test_gh438_recipe_prefix_blanked_only_in_make() {
        // plain shell: `-local` / `+local` are not the command `local`
        assert_eq!(lines("-local x", "local"), Vec::<usize>::new());
        assert_eq!(lines("+local x", "local"), Vec::<usize>::new());
        assert_eq!(lines("true\n@local x", "local"), Vec::<usize>::new());
        // Make recipe text: the prefixes are Make's, the shell sees `local`
        assert_eq!(lines("\n\t-local x\n", "local"), vec![2]);
        assert_eq!(lines("\n\t@local x\n", "local"), vec![2]);
    }

    #[test]
    fn test_gh438_first_arg_end_is_on_its_own_line() {
        let h = command_keyword_hits("return \\\n  256", "return");
        assert_eq!((h[0].line, h[0].col), (1, 1));
        assert_eq!(h[0].first_arg, Some(("256".to_string(), 2, 6)));
    }
}
