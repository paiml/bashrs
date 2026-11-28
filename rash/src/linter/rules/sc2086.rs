// SC2086: Double quote to prevent globbing and word splitting - THIN SHIM
// All logic extracted to sc2086_logic.rs

use super::sc2086_logic::*;
use crate::linter::shell_words::{self, Expansion, ShellWord, WordRole};
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// bashrs#679: an unquoted `${NAME<op>WORD}` whose WORD holds an unquoted command
/// substitution (`${x:-$(date)}`) is word-split with the whole expansion. ShellCheck 0.8.0
/// reports SC2086 on that expansion; the inner `$(date)` is not SC2046 (sc2046 skips it).
/// Assignment words and `case` subjects are not split, so they are skipped.
fn default_cmdsub_expansions(line: &str) -> Vec<Expansion> {
    if line.trim_start().starts_with('#') {
        return Vec::new();
    }
    let mut out = Vec::new();
    for cmd in shell_words::simple_commands(line) {
        for (i, word) in cmd.words.iter().enumerate() {
            if is_split_word(&cmd.words, i) {
                out.extend(
                    word.expansions
                        .iter()
                        .filter(|e| {
                            is_unquoted_default_cmdsub(word, e) && !is_case_pattern(line, e)
                        })
                        .cloned(),
                );
            }
        }
    }
    out
}

/// True when `words[i]` is word-split by the shell: not an assignment prefix or reserved word,
/// and not the subject of `case`.
fn is_split_word(words: &[ShellWord], i: usize) -> bool {
    if matches!(words[i].role, WordRole::AssignPrefix | WordRole::Reserved) {
        return false;
    }
    !(i > 0 && is_case_keyword(&words[i - 1]))
}

fn is_case_keyword(word: &ShellWord) -> bool {
    word.role == WordRole::Reserved && word.literal == "case"
}

/// True when `e` is an unquoted `${NAME<op>WORD}` (a parameter default with a word operator)
/// whose WORD holds an unquoted command substitution.
fn is_unquoted_default_cmdsub(word: &ShellWord, e: &Expansion) -> bool {
    e.braced
        && !e.quoted
        && takes_word_operator(e)
        && word
            .substitutions
            .iter()
            .any(|s| !s.quoted && s.col > e.col && s.end_col <= e.end_col)
}

/// True when the text after `${NAME` starts with a word operator: `:-` `-` `:=` `=` `:+` `+`
/// `:?` `?`. `${#a[$(date)]}` and `${a[$(date)]:-x}` do not match.
fn takes_word_operator(e: &Expansion) -> bool {
    const OPS: [&str; 8] = [":-", "-", ":=", "=", ":+", "+", ":?", "?"];
    e.text
        .strip_prefix("${")
        .and_then(|s| s.strip_prefix(e.name.as_str()))
        .is_some_and(|rest| OPS.iter().any(|op| rest.starts_with(op)))
}

/// True when `e` ends a case pattern: `${y:-$(date)})` at the start of a line or right after
/// `in`. ShellCheck 0.8.0 reports SC2254 on such a pattern, not SC2086.
fn is_case_pattern(line: &str, e: &Expansion) -> bool {
    let after = line.get(e.end_col - 1..).unwrap_or("");
    let before = line.get(..e.col - 1).unwrap_or("").trim_end();
    after.starts_with(')') && (before.is_empty() || before.ends_with(" in"))
}

/// Check for unquoted variable expansions (SC2086)
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let pattern = get_var_pattern();
    // PMAT-654: a variable only ever assigned an integer literal cannot word-split; shellcheck 0.8.0 agrees.
    let mut cstyle_vars = get_cstyle_for_loop_vars(source);
    cstyle_vars.extend(get_integer_literal_vars(source));
    let body = super::sc2086_heredoc::body_command_substitution_bytes(source);
    for (line_num, line) in source.lines().enumerate() {
        let subst = body.get(&(line_num + 1));
        for e in default_cmdsub_expansions(line) {
            if subst.is_some_and(|s| !s.contains(&(e.col - 1))) {
                continue;
            }
            result.add(
                Diagnostic::new(
                    "SC2086",
                    Severity::Warning,
                    format!(
                        "Double quote to prevent globbing and word splitting on {}",
                        e.text
                    ),
                    Span::new(line_num + 1, e.col, line_num + 1, e.end_col),
                )
                .with_fix(Fix::new(format!("\"{}\"", e.text))),
            );
        }
        for uv in find_unquoted_vars(line, &pattern, &cstyle_vars) {
            if subst.is_some_and(|s| !s.contains(&(uv.col - 1))) {
                continue;
            }
            let span = Span::new(line_num + 1, uv.col, line_num + 1, uv.end_col);
            let var_text = format_var_text(&uv.var_name, uv.is_braced);
            result.add(
                Diagnostic::new(
                    "SC2086",
                    Severity::Warning,
                    format!(
                        "Double quote to prevent globbing and word splitting on {}",
                        var_text
                    ),
                    span,
                )
                .with_fix(Fix::new(format_quoted_var(&uv.var_name, uv.is_braced))),
            );
        }
    }
    result
}

#[cfg(test)]
#[path = "sc2086_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "sc2086_tests_heredoc.rs"]
mod tests_heredoc;

#[cfg(test)]
mod integer_literal_tests_654 {
    use super::check;

    // PMAT-654 / bashrs#654: shellcheck 0.8.0 does not flag $n when n is assigned an integer literal.
    #[test]
    fn echo_of_integer_literal_var_is_not_flagged() {
        let r = check("n=1\necho $n\n");
        assert!(
            r.diagnostics.is_empty(),
            "SC2086 fired on integer-literal var: {:?}",
            r.diagnostics
                .iter()
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_bracket_of_integer_literal_var_is_not_flagged() {
        let r = check("n=1\nif [ $n -le 1 ]; then\n  echo ok\nfi\n");
        assert!(
            r.diagnostics.is_empty(),
            "SC2086 fired in [ ] on integer-literal var"
        );
    }

    #[test]
    fn non_literal_var_is_still_flagged() {
        let r = check("n=$(date)\necho $n\n");
        assert!(!r.diagnostics.is_empty());
    }

    // Shellcheck 0.8.0 flags an unquoted positional parameter ($1..$9) with SC2086.
    #[test]
    fn unquoted_positional_param_is_flagged_sc2086() {
        let r = check("#!/bin/bash\necho $1 $2\n");
        let codes: Vec<String> = r.diagnostics.iter().map(|d| d.code.clone()).collect();
        assert_eq!(
            codes.iter().filter(|c| *c == "SC2086").count(),
            2,
            "SC2086 must fire on $1 and $2, got {:?}",
            r.diagnostics
                .iter()
                .map(|d| (d.code.clone(), d.message.clone()))
                .collect::<Vec<_>>()
        );
    }

    // Positional-parameter parity with shellcheck 0.8.0 (measured, `shellcheck -s bash -f gcc`):
    //   echo $10        -> SC2086 at col 6 (one note; SC1037 error)
    //   echo $12 $99    -> SC2086 at col 6 and col 10
    //   echo $1$foo     -> SC2086 at col 6 and col 8
    //   echo "$1" / echo $# -> none
    fn sc2086_cols(src: &str) -> Vec<usize> {
        check(src)
            .diagnostics
            .iter()
            .filter(|d| d.code == "SC2086")
            .map(|d| d.span.start_col)
            .collect()
    }

    #[test]
    fn adjacent_positional_then_var_reports_both() {
        // Regression: the `$1` match used to consume the `1` that `$foo` needs as its `pre` char.
        assert_eq!(sc2086_cols("#!/bin/bash\necho $1$foo\n"), vec![6, 8]);
    }

    #[test]
    fn adjacent_var_then_var_reports_both() {
        assert_eq!(sc2086_cols("#!/bin/bash\necho $foo$bar\n"), vec![6, 10]);
    }

    #[test]
    fn multi_digit_positional_matches_shellcheck_at_dollar() {
        // shellcheck 0.8.0 reports SC2086 at the `$` of `$10`, and at each of `$12` and `$99`.
        assert_eq!(sc2086_cols("#!/bin/bash\necho $10\n"), vec![6]);
        assert_eq!(sc2086_cols("#!/bin/bash\necho $12 $99\n"), vec![6, 10]);
    }

    #[test]
    fn quoted_positional_stays_silent() {
        assert!(sc2086_cols("#!/bin/bash\necho \"$1\"\n").is_empty());
        assert!(sc2086_cols("#!/bin/bash\necho '$1'\n").is_empty());
    }

    #[test]
    fn special_params_stay_silent() {
        assert!(sc2086_cols("#!/bin/bash\necho $#\n").is_empty());
    }
}

#[cfg(test)]
mod command_name_tests_671 {
    use super::check;

    // bashrs#671: a variable in command-name position (first word of a simple command) is not
    // word-split into arguments by the shell's command lookup; shellcheck 0.8.0 is silent on it.
    fn sc2086_count(src: &str) -> usize {
        check(src)
            .diagnostics
            .iter()
            .filter(|d| d.code == "SC2086")
            .count()
    }

    #[test]
    fn unquoted_command_name_var_is_not_flagged() {
        assert_eq!(sc2086_count("#!/bin/bash\ncmd=ls\n$cmd --flag\n"), 0);
    }

    #[test]
    fn indented_and_chained_command_name_var_is_not_flagged() {
        assert_eq!(
            sc2086_count("#!/bin/bash\nif true; then\n  $cmd --flag\nfi\n"),
            0
        );
        assert_eq!(sc2086_count("#!/bin/bash\ntrue && $cmd --flag\n"), 0);
    }

    #[test]
    fn variable_argument_is_still_flagged() {
        assert_eq!(sc2086_count("#!/bin/bash\necho $cmd\n"), 1);
        assert_eq!(sc2086_count("#!/bin/bash\ndir=/tmp\nls $dir\n"), 1);
    }
}

#[cfg(test)]
mod default_cmdsub_tests_679 {
    use super::check;

    // bashrs#679: shellcheck 0.8.0 reports SC2086 on the `${x:-$(date)}` expansion.
    #[test]
    fn default_value_cmdsub_expansion_is_sc2086() {
        let r = check("x=\"\"; echo ${x:-$(date)}");
        let n = r.diagnostics.iter().filter(|d| d.code == "SC2086").count();
        assert_eq!(
            n, 1,
            "SC2086 must fire once on ${{x:-$(date)}}, got {:?}",
            r.diagnostics
        );
    }
}

#[cfg(test)]
mod default_cmdsub_oracle_tests_679 {
    use super::check;

    // Every expectation below was measured with local shellcheck 0.8.0 (`-s bash -f gcc`).
    fn sc2086_cols(src: &str) -> Vec<usize> {
        check(src)
            .diagnostics
            .iter()
            .filter(|d| d.code == "SC2086")
            .map(|d| d.span.start_col)
            .collect()
    }

    #[test]
    fn quoted_default_cmdsub_is_silent() {
        // shellcheck 0.8.0: no finding on `echo "${x:-$(date)}"`.
        assert!(sc2086_cols("#!/bin/bash\nx=\"\"; echo \"${x:-$(date)}\"\n").is_empty());
    }

    #[test]
    fn array_length_subscript_cmdsub_is_silent() {
        // shellcheck 0.8.0: no finding on `${#a[$(date)]}`, which is not a parameter default.
        assert!(sc2086_cols("#!/bin/bash\na=(1); echo ${#a[$(date)]}\n").is_empty());
    }

    #[test]
    fn case_pattern_default_cmdsub_is_not_sc2086() {
        // shellcheck 0.8.0 reports SC2254 (not SC2086) on a case pattern, on the same line and on its own line.
        assert!(sc2086_cols(
            "#!/bin/bash\nx=\"\"; y=\"\"; case $x in ${y:-$(date)}) echo hi;; esac\n"
        )
        .is_empty());
        assert!(sc2086_cols(
            "#!/bin/bash\nx=\"\"; y=\"\"\ncase $x in\n  ${y:-$(date)}) echo hi;;\nesac\n"
        )
        .is_empty());
    }
}
