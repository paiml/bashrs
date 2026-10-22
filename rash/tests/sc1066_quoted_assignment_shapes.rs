//! #437 / forjar mq-label: SC1066 (`$VAR=` on the left of an assignment) must
//! not fire on `$p=$v` that is TEXT inside a quoted word, one fixture per form
//! variant, and must still fire on a bare `$p=v` at command position.
//!
//! Every error-clean fixture is clean under `bash -n`.

use bashrs::linter::lint_shell;

fn sc1066(src: &str) -> usize {
    lint_shell(src)
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC1066")
        .count()
}

/// The exact line forjar's I8 gate refused on a task (bashrs 6.68.0 inside
/// forjar 1.33.0): infra machines/intel/forjar-ci.yaml, mq-label apply.
#[test]
fn mq_label_accumulator_line_is_clean() {
    assert_eq!(
        sc1066("ids=\"\"\nfor WANT in a b; do\n  id=1\n  ids=\"$ids $WANT=$id\"\ndone\n"),
        0
    );
}

/// #437: an echo argument after a single-quoted word that holds a `"`.
#[test]
fn echo_arg_after_single_quoted_dquote_is_clean() {
    let src = "for p in a b; do\n  v=\"$(busctl get \"$p\")\"\n  [ \"$v\" = 's \"ignore\"' ] || { echo \"logind says $p=$v after HUP\" >&2; exit 1; }\ndone\n";
    assert_eq!(sc1066(src), 0);
}

#[test]
fn assignment_rhs_is_clean() {
    assert_eq!(sc1066("h=x\ns=\"\"\ns=\"$s $h=UP\"\n"), 0);
}

#[test]
fn single_quoted_is_clean() {
    assert_eq!(sc1066("echo '$p=$v'\n"), 0);
}

/// Controls: the real defect must still be reported.
#[test]
fn bare_dollar_assignment_still_fires() {
    assert_eq!(sc1066("$VAR=hello\n"), 1);
    assert_eq!(sc1066("echo hi; $X=5\n"), 1);
}
