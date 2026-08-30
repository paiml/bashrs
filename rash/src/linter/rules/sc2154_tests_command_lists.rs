//! GH-275: an assignment is not always the first thing on its line.
//!
//! `collect_variable_info` found assignments with a regex anchored to `^\s*`,
//! so `captures_iter` could only ever match once, at the start of the line.
//! Every assignment after the first separator on a line was invisible, and the
//! variable it defined was then reported as "referenced but not assigned":
//!
//! ```sh
//! repo=${1:-}; course=${2:-}; delete=${3:-}   # only `repo` registered
//! ```
//!
//! 143 of the 150 SC2154 findings on the rmedia corpus were this. `shellcheck`
//! reports none of them; it is the reference implementation for the SCxxxx
//! namespace and every case below was checked against it.
//!
//! The must-not-regress direction matters more than the count: widening the
//! search must not start reading `--color=auto` or a quoted `"x=1"` as an
//! assignment, because that would trade these false positives for SC2154 going
//! blind on a genuinely unassigned variable.

use super::sc2154::check;

fn flagged(src: &str) -> Vec<String> {
    check(src)
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC2154")
        .map(|d| d.message.split('\'').nth(1).unwrap_or_default().to_string())
        .collect()
}

fn assert_clean(src: &str) {
    let got = flagged(src);
    assert!(got.is_empty(), "SC2154 false positive on {src:?}: {got:?}");
}

// ===== the separators =====

#[test]
fn semicolon_separated_assignments_are_all_tracked() {
    // rmedia scripts/archive-delivered-course.sh:36
    assert_clean("repo=${1:-}; course=${2:-}; delete=${3:-}\necho \"$repo$course$delete\"");
}

#[test]
fn semicolon_without_spaces() {
    assert_clean("a=1;b=2;c=3\necho \"$a$b$c\"");
}

#[test]
fn andand_and_oror_separated_assignments() {
    assert_clean("a=1 && b=2\necho \"$a$b\"");
    assert_clean("a=1 || b=2\necho \"$a$b\"");
    assert_clean("a=1 && b=2 || c=3\necho \"$a$b$c\"");
}

#[test]
fn newline_separated_assignments_still_work() {
    assert_clean("a=1\nb=2\necho \"$a$b\"");
}

#[test]
fn assignments_inside_a_brace_group_or_subshell() {
    assert_clean("a=1; { b=2; }\necho \"$a$b\"");
    assert_clean("a=1; ( b=2 )\necho \"$a$b\"");
}

#[test]
fn assignment_after_a_control_keyword_on_one_line() {
    assert_clean("if true; then c=1; fi\necho \"$c\"");
    assert_clean("for i in 1 2; do d=1; done\necho \"$d\"");
    assert_clean("if false; then :; else e=1; fi\necho \"$e\"");
}

#[test]
fn assignment_in_a_case_branch_on_one_line() {
    assert_clean("case x in x) g=1 ;; esac\necho \"$g\"");
}

// ===== the declaration builtins =====

#[test]
fn local_declares_several_variables_at_once() {
    // rmedia scripts/falsify-ci-retry-classifier.sh:19
    assert_clean("f() {\n  local name=$1 expect=$2 body=$3\n  echo \"$name$expect$body\"\n}");
}

#[test]
fn declaration_builtins_on_a_single_line_function_body() {
    // The `^\s*` anchor also lost the FIRST assignment whenever the line began
    // with anything else — here, the function header.
    assert_clean("f() { local x=$1 y=$2; echo \"$x$y\"; }");
}

#[test]
fn local_then_assign_across_a_semicolon() {
    // rmedia scripts/falsify-ci-retry-classifier.sh:20
    assert_clean(
        "f() {\n  local dir; dir=$(mktemp -d); local marker=\"$dir/n\"\n  echo \"$dir$marker\"\n}",
    );
}

#[test]
fn export_readonly_declare_typeset_with_flags() {
    assert_clean("export A=1 B=2\necho \"$A$B\"");
    assert_clean("readonly r=1 s=2\necho \"$r$s\"");
    assert_clean("declare -i n=1 m=2\necho \"$n$m\"");
    assert_clean("typeset -r t=1 u=2\necho \"$t$u\"");
}

#[test]
fn assignment_prefixes_on_one_command() {
    // `FOO=1 BAR=2 cmd` — both are assignments, cmd is not.
    assert_clean("A=1 B=2 env >/dev/null\necho \"$A$B\"");
}

#[test]
fn append_and_array_subscript_assignment() {
    assert_clean("p=1; p+=2\necho \"$p\"");
    assert_clean("q[0]=1; q[1]=2\necho \"${q[0]}\"");
}

// ===== MUST STILL FIRE =====
//
// Each of these is a variable `shellcheck` also reports. If widening the
// assignment search silences any of them, the fix has traded a false positive
// for a false negative and SC2154 is no longer trustworthy when it is red.

#[test]
fn must_still_fire_on_a_plainly_unassigned_variable() {
    assert_eq!(flagged("echo \"$never_assigned\""), ["never_assigned"]);
}

#[test]
fn must_still_fire_when_the_equals_is_inside_an_option_argument() {
    // `--color=auto` is an argument to grep, NOT an assignment to `color`.
    // This is the discriminating case for the whole fix: a naive unanchored
    // `([A-Za-z_]\w*)=` regex passes every test above and fails this one.
    assert_eq!(
        flagged("grep --color=auto q /dev/null\necho \"$color\""),
        ["color"]
    );
}

#[test]
fn must_still_fire_when_the_equals_is_inside_a_string_literal() {
    assert_eq!(flagged("echo \"x=1\"\necho \"$x\""), ["x"]);
    assert_eq!(flagged("echo 'y=1'\necho \"$y\""), ["y"]);
}

#[test]
fn must_still_fire_when_the_equals_is_an_argument_word() {
    // `echo a=1` prints the text "a=1"; it assigns nothing.
    assert_eq!(flagged("echo a=1\necho \"$a\""), ["a"]);
}

#[test]
fn must_still_fire_on_a_test_comparison() {
    // `[ "$z" = 1 ]` is a comparison. `z` is still unassigned.
    let got = flagged("[ \"$z\" = 1 ] && echo hi");
    assert_eq!(got, ["z"]);
}

#[test]
fn must_still_fire_for_a_variable_assigned_only_in_a_comment() {
    assert_eq!(flagged("# c=1\necho \"$c\""), ["c"]);
}

// ===== `mapfile` / `readarray` assign too =====
//
// Same shape as the separator bug: an assignment form `collect_variable_info`
// could not see. `read` was already handled; its array siblings were not, so
// every `mapfile -t names < <(…)` left `names` looking unassigned. 9 of the 11
// SC2154 findings left on the rmedia corpus were this. `shellcheck` reports
// none of them.

#[test]
fn mapfile_and_readarray_assign_their_array() {
    assert_clean("mapfile -t names < <(ls)\necho \"${names[0]}\"");
    assert_clean("readarray -t rows < <(ls)\necho \"${rows[0]}\"");
    // rmedia scripts/lint-contract-validity.sh:17
    assert_clean("mapfile -t allow < <(grep -v '^#' f)\necho \"${allow[@]}\"");
}

#[test]
fn mapfile_flags_that_take_an_argument_are_skipped() {
    assert_clean("mapfile -d '' -n 5 -O 1 -s 2 -u 3 -C cb -c 4 -t arr < <(ls)\necho \"${arr[0]}\"");
}

#[test]
fn mapfile_with_no_array_name_assigns_MAPFILE() {
    assert_clean("mapfile -t < <(ls)\necho \"${MAPFILE[0]}\"");
}

#[test]
fn must_still_fire_alongside_a_mapfile() {
    // Widening to mapfile must not make the rule blind to its neighbours.
    assert_eq!(
        flagged("mapfile -t names < <(ls)\necho \"${names[0]}$ghost\""),
        ["ghost"]
    );
}

#[test]
fn must_still_fire_when_mapfile_is_only_a_word_in_a_message() {
    assert_eq!(flagged("echo 'use mapfile -t out'\necho \"$out\""), ["out"]);
}
