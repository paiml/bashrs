//! PMAT-248: end-to-end regression tests for the lexer-context false-positive
//! class (`contracts/linter-lexer-context-v1.yaml`).
//!
//! Eight open issues describe one defect: a rule reads bytes the shell would
//! never parse as shell, or reads a real construct with the wrong grammar. Each
//! test here runs the issue's own reproducer through the public entry point
//! (`lint_shell` / `lint_makefile`) and asserts two things:
//!
//! - the false-positive code is ABSENT on the reproducer, and
//! - the SAME code still FIRES on a true positive, so a fix that deletes or
//!   blunts the rule is not a fix.
//!
//! Each phase of PMAT-248 adds its test in the commit that makes it RED and
//! turns it green in the next commit, so the tree is never left with a
//! long-lived red test. The two issues already clean on 7.0.2 (#235, #258)
//! were pinned first.

#![allow(clippy::unwrap_used)]

use crate::linter::{lint_makefile, lint_shell, LintResult};

fn codes(result: &LintResult) -> Vec<&str> {
    result.diagnostics.iter().map(|d| d.code.as_str()).collect()
}

fn assert_absent(result: &LintResult, code: &str, src: &str) {
    assert!(
        !codes(result).contains(&code),
        "{code} must not fire on:\n{src}\ngot {:?}",
        result.diagnostics
    );
}

fn assert_fires(result: &LintResult, code: &str, src: &str) {
    assert!(
        codes(result).contains(&code),
        "{code} must still fire on:\n{src}\ngot {:?}",
        result.diagnostics
    );
}

fn shell_absent(src: &str, code: &str) {
    assert_absent(&lint_shell(src), code, src);
}

fn shell_fires(src: &str, code: &str) {
    assert_fires(&lint_shell(src), code, src);
}

// ---------------------------------------------------------------------------
// GH-235 (pinned): an apostrophe inside "..." is text.
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT248_gh235_apostrophe_inside_double_quotes_is_text() {
    let src = "#!/bin/bash\necho \"Make sure you've done this\"\necho \"done\"\n";
    shell_absent(src, "SC1078");
    shell_absent(src, "SC1004");
    // A genuinely unterminated double quote is still reported.
    shell_fires("#!/bin/bash\necho \"oops\n", "SC1078");
}

// ---------------------------------------------------------------------------
// GH-258 (pinned): a trailing # comment after ] is a comment.
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT248_gh258_trailing_comment_after_test_is_a_comment() {
    let src = "#!/bin/bash\n[ -f /etc/passwd ] # check the file\n";
    shell_absent(src, "SC1140");
    // A real stray token after ] is still reported.
    shell_fires("#!/bin/bash\n[ -f /etc/passwd ] echo hi\n", "SC1140");
}
// ---------------------------------------------------------------------------
// GH-237: $(( )) is arithmetic expansion, one word, never word-split.
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT248_gh237_arith_expansion_is_not_command_substitution() {
    let src = "#!/bin/bash\nn=7\ni=2\nif [ $((n % i)) -eq 0 ]; then\n  echo yes\nfi\n";
    shell_absent(src, "SC2046");
    shell_absent(src, "SC1028");
    // A real unquoted command substitution is still reported.
    shell_fires("#!/bin/bash\necho $(date)\n", "SC2046");
}

// ---------------------------------------------------------------------------
// GH-241: a $var inside "$( ... "..." ... )" is quoted.
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT248_gh241_quoted_var_inside_nested_substitution_is_quoted() {
    let src = "#!/bin/sh\ncoverage=90\nif [ \"$(echo \"$coverage >= 80\" | bc -l)\" -eq 1 ]; then echo ok; fi\n";
    shell_absent(src, "SC2047");
    shell_absent(src, "SC2122");
    // An unquoted variable in a [ ] test is still reported.
    shell_fires(
        "#!/bin/sh\nx=1\nif [ $x -eq 1 ]; then echo y; fi\n",
        "SC2047",
    );
}

// ---------------------------------------------------------------------------
// GH-262: SC2046 reports only where the shell would split — not an assignment
// RHS, not inside double quotes, not a `case` word.
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT248_gh262_sc2046_only_where_the_shell_splits() {
    shell_absent("#!/bin/sh\nx=$(date)\necho \"$x\"\n", "SC2046");
    shell_absent("#!/bin/sh\necho \"$(date)\"\n", "SC2046");
    shell_absent(
        "#!/bin/sh\ncase $(uname) in\n  Linux) echo l ;;\n  *) echo o ;;\nesac\n",
        "SC2046",
    );
    // An unquoted command substitution in argument position still splits.
    shell_fires("#!/bin/sh\necho $(date)\n", "SC2046");
}
// ---------------------------------------------------------------------------
// GH-242: a heredoc body is data.
// ---------------------------------------------------------------------------

const GH242_SRC: &str =
    "#!/bin/sh\ncat <<MARKER\n- [ ] a markdown checkbox\n<li>x &lt; 10</li>\nMARKER\n";

#[test]
fn test_PMAT248_gh242_heredoc_body_is_data() {
    shell_absent(GH242_SRC, "SC1109");
    shell_absent(GH242_SRC, "SC2188");
    shell_absent(GH242_SRC, "SC2104");
    // An HTML entity in bare code is still reported.
    shell_fires("#!/bin/sh\necho a &lt; b\n", "SC1109");
}

#[test]
fn test_PMAT248_gh242_cat_heredoc_to_stdout_is_not_useless() {
    shell_absent(GH242_SRC, "SC2276");
    // A cat whose only job is to feed a pipe is still reported.
    shell_fires("#!/bin/sh\ncat <<EOF | grep x\nfoo\nEOF\n", "SC2276");
}

// ---------------------------------------------------------------------------
// GH-252: a backslash-escaped backtick inside "..." is text.
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT248_gh252_escaped_backtick_inside_double_quotes_is_text() {
    let src = "#!/usr/bin/env bash\ngh issue create --body \"Acceptance Criteria:\n- [ ] Parse markdown links: \\`[text](url)\\`\n- [ ] Handle edge cases (nested brackets, special characters)\n\"\n";
    for code in ["SC2006", "SC2046", "SC2099", "SC1028", "SC1078"] {
        shell_absent(src, code);
    }
    // A real backtick substitution is still reported.
    shell_fires("#!/bin/bash\necho `date`\n", "SC2006");
}

// ---------------------------------------------------------------------------
// GH-255: a Makefile target-line comment never reaches a shell rule.
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT248_gh255_makefile_target_comment_is_not_shell() {
    let src = "dev-setup: ## Set up local dev environment\n\t@echo hi\n";
    assert_absent(&lint_makefile(src), "SC2168", src);
    // `local` on a recipe line outside a function is still reported.
    let bare = "build:\n\tlocal x=1\n";
    assert_fires(&lint_makefile(bare), "SC2168", bare);
}
// ---------------------------------------------------------------------------
// GH-261: SC1012 means what shellcheck's SC1012 means - an escape the shell drops.
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT248_gh261_printf_format_escapes_are_interpreted() {
    let src = "#!/bin/bash\nprintf 'hello %s\\n' \"$1\"\n";
    shell_absent(src, "SC1012");
    // Inside single quotes the escape reaches the command intact; echo's case
    // belongs to SC2028/SC2271, not SC1012.
    shell_absent("#!/bin/bash\necho 'a\\nb'\n", "SC1012");
    // Unquoted, the shell drops the backslash: `echo a\tb` prints "atb".
    shell_fires("#!/bin/bash\necho a\\tb\n", "SC1012");
}

// ---------------------------------------------------------------------------
// Phase 7 review findings, each measured against the 7.0.2 binary before the
// fix: two regressions of this branch and two pre-existing wrong reports.
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT248_review_sc2046_command_position_is_still_reported() {
    // 7.0.2 reported both; the word-role rewrite had silenced CommandName.
    shell_fires("#!/bin/sh\n$(get_command)\n", "SC2046");
    shell_fires("#!/bin/sh\n$(get_command) arg\n", "SC2046");
    // The word of `case … in` is still not split and still not reported.
    shell_absent(
        "#!/bin/sh\ncase $(uname) in\n  Linux) echo l ;;\n  *) echo o ;;\nesac\n",
        "SC2046",
    );
}

#[test]
fn test_PMAT248_review_sc2047_considers_only_operands_of_the_test() {
    // `$x` is an operand of echo, not of `[`, and the outer word is quoted;
    // an unquoted variable inside a substitution is SC2086's subject.
    shell_absent(
        "#!/bin/sh\nx=1\nif [ -n \"$(echo $x)\" ]; then :; fi\n",
        "SC2047",
    );
    shell_fires("#!/bin/sh\nx=1\nif [ $x -eq 1 ]; then :; fi\n", "SC2047");
}

#[test]
fn test_PMAT248_review_make_recipe_continuation_reaches_shell_rules() {
    // A backslash-newline continues the recipe line; GNU make does not require
    // a leading tab on the continued line. 7.0.2 reported this line.
    let src = "build:\n\t@true \\\n  local x=1\n";
    assert_fires(&lint_makefile(src), "SC2168", src);
}

#[test]
fn test_PMAT248_review_sc2276_ignores_a_pipe_inside_quotes() {
    shell_absent("#!/bin/sh\ncat <<EOF > \"file|name\"\nfoo\nEOF\n", "SC2276");
    shell_fires("#!/bin/sh\ncat <<EOF | grep x\nfoo\nEOF\n", "SC2276");
}

// ---------------------------------------------------------------------------
// PMAT-250 review: a substitution inside a `${ … }` default keeps SC2046's
// verdict from before shell_words owned substitutions.
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT255_pmat250_sc2046_inside_brace_default_unquoted() {
    shell_fires("#!/bin/sh\necho ${var:-$(date)}\n", "SC2046");
}

#[test]
fn test_PMAT255_pmat250_sc2046_inside_brace_default_quoted() {
    shell_absent("#!/bin/sh\necho \"${var:-$(date)}\"\n", "SC2046");
    shell_absent("#!/bin/sh\necho ${var:-\"$(date)\"}\n", "SC2046");
}

// ---------------------------------------------------------------------------
// PMAT-257 / GH-309: a regression introduced in 7.1.0 — SC2046 fired on a
// command substitution in an assignment RHS when the substitution spanned
// multiple physical lines. An assignment RHS is never word-split, single- or
// multi-line alike (the single-line spelling was already fixed under
// GH-262/#262 and must stay fixed).
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT257_gh309_sc2046_multiline_substitution_in_assignment_rhs() {
    shell_absent("#!/bin/sh\nx=$(\n  cmd\n)\necho \"$x\"\n", "SC2046");
    // A companion unquoted multi-line substitution in argument position must
    // still be reported, so a fix that blunts the rule fails this too.
    shell_fires("#!/bin/sh\necho $(\n  cmd\n)\n", "SC2046");
}

// ---------------------------------------------------------------------------
// PMAT-257 / GH-314: IDEM002 read the letters `rm` inside a quoted sentence
// as an `rm` command. `rm` is only a command when it is in command position,
// not when it is a substring of a word or inside quotes.
// ---------------------------------------------------------------------------

#[test]
fn test_PMAT257_gh314_idem002_rm_inside_a_quoted_word_is_text() {
    shell_absent(
        "#!/bin/sh\necho \"the pipe form fails silently\"\n",
        "IDEM002",
    );
    // A real bare `rm` command is still reported.
    shell_fires("#!/bin/sh\nrm /app/current\n", "IDEM002");
}
