//! Unit tests for the assignment scanner (GH-272).

use super::*;

fn a(line: &str) -> Vec<String> {
    line_assignments(line)
}

// ===== assigned_name: what shape is an assignment word =====

#[test]
fn assigned_name_accepts_the_real_shapes() {
    assert_eq!(assigned_name("x=1"), Some("x"));
    assert_eq!(assigned_name("_x=1"), Some("_x"));
    assert_eq!(assigned_name("x9=1"), Some("x9"));
    assert_eq!(assigned_name("x="), Some("x"));
    assert_eq!(assigned_name("p+=2"), Some("p"));
    assert_eq!(assigned_name("q[0]=1"), Some("q"));
    assert_eq!(assigned_name("map[$k]=v"), Some("map"));
    assert_eq!(assigned_name("arr+=(a)"), Some("arr"));
}

#[test]
fn assigned_name_rejects_everything_that_is_not_one() {
    assert_eq!(assigned_name("--color=auto"), None);
    assert_eq!(assigned_name("-o=x"), None);
    assert_eq!(assigned_name("=1"), None);
    assert_eq!(assigned_name("2x=1"), None);
    assert_eq!(assigned_name("x"), None);
    assert_eq!(assigned_name("x.y=1"), None);
    assert_eq!(assigned_name("q[0=1"), None);
    assert_eq!(assigned_name(""), None);
}

// ===== command position =====

#[test]
fn only_the_command_position_counts() {
    assert_eq!(a("x=1"), ["x"]);
    assert!(a("echo x=1").is_empty());
    assert!(a("grep --color=auto q").is_empty());
    assert!(a("[ \"$z\" = 1 ]").is_empty());
}

#[test]
fn separators_reopen_the_command_position() {
    assert_eq!(a("a=1; b=2"), ["a", "b"]);
    assert_eq!(a("a=1;b=2;c=3"), ["a", "b", "c"]);
    assert_eq!(a("a=1 && b=2"), ["a", "b"]);
    assert_eq!(a("a=1 || b=2"), ["a", "b"]);
    assert_eq!(a("a=1 & b=2"), ["a", "b"]);
    assert_eq!(a("a=1; ( b=2 )"), ["a", "b"]);
    assert_eq!(a("a=1; { b=2; }"), ["a", "b"]);
}

#[test]
fn a_command_word_closes_the_position_until_the_next_separator() {
    // `b=2` is an argument to echo; `c=3` starts a fresh command.
    assert_eq!(a("a=1; echo b=2; c=3"), ["a", "c"]);
}

#[test]
fn assignment_prefixes_stack() {
    assert_eq!(a("A=1 B=2 env"), ["A", "B"]);
    assert!(a("A=1 env B=2").len() == 1);
}

#[test]
fn declaration_builtins_take_several_names() {
    assert_eq!(
        a("local name=$1 expect=$2 body=$3"),
        ["name", "expect", "body"]
    );
    assert_eq!(a("export A=1 B=2"), ["A", "B"]);
    assert_eq!(a("declare -i n=1 m=2"), ["n", "m"]);
    assert_eq!(a("readonly -r t=1"), ["t"]);
}

#[test]
fn control_keywords_do_not_close_the_command_position() {
    assert_eq!(a("if true; then c=1; fi"), ["c"]);
    assert_eq!(a("for i in 1 2; do d=1; done"), ["d"]);
    assert_eq!(a("if false; then :; else e=1; fi"), ["e"]);
    assert_eq!(a("case x in x) g=1 ;; esac"), ["g"]);
}

#[test]
fn a_one_line_function_body_is_reached() {
    assert_eq!(a("f() { local x=$1 y=$2; echo \"$x$y\"; }"), ["x", "y"]);
}

// ===== quoting and expansion =====

#[test]
fn quoted_text_is_never_an_assignment() {
    assert!(a("echo \"x=1\"").is_empty());
    assert!(a("echo 'y=1'").is_empty());
    assert!(a("printf '%s' \"a=1;b=2\"").is_empty());
}

#[test]
fn an_expansion_does_not_split_the_word() {
    // `${1:-}` closes with `}`; that brace must not read as a command separator
    // or `course` and `delete` go missing. This is the rmedia line verbatim.
    assert_eq!(
        a("repo=${1:-}; course=${2:-}; delete=${3:-}"),
        ["repo", "course", "delete"]
    );
    assert_eq!(a("dir=$(mktemp -d); marker=\"$dir/n\""), ["dir", "marker"]);
    assert_eq!(a("n=$((1 + 2)); m=3"), ["n", "m"]);
}

#[test]
fn a_trailing_comment_is_not_scanned() {
    assert_eq!(a("a=1  # b=2"), ["a"]);
    assert!(a("curl http://x#y=1").is_empty());
}

#[test]
fn brace_expansion_does_not_split_a_word() {
    assert!(a("cp file{,.bak}").is_empty());
}

#[test]
fn redirections_end_a_word_without_opening_a_command() {
    assert_eq!(a("a=1 >out"), ["a"]);
    assert!(a("echo hi >out b=2").is_empty());
}

#[test]
fn unterminated_quote_or_expansion_does_not_panic() {
    let _ = a("a=\"unterminated");
    let _ = a("a=${unterminated");
    let _ = a("a=$(unterminated");
    let _ = a("a='");
    let _ = a("\\");
}

// ===== mapfile / readarray =====

#[test]
fn mapfile_names_its_array() {
    assert_eq!(a("mapfile -t names < <(ls)"), ["names"]);
    assert_eq!(a("readarray -t rows < <(ls)"), ["rows"]);
    assert_eq!(a("mapfile arr < f"), ["arr"]);
}

#[test]
fn mapfile_option_arguments_are_not_the_array() {
    assert_eq!(
        a("mapfile -d '' -n 5 -O 1 -s 2 -u 3 -C cb -c 4 -t arr < f"),
        ["arr"]
    );
}

#[test]
fn mapfile_without_a_name_assigns_nothing_explicit() {
    // The implicit target is MAPFILE, already a known builtin.
    assert!(a("mapfile -t < <(ls)").is_empty());
}

#[test]
fn mapfile_only_counts_as_the_command_word() {
    assert!(a("echo mapfile -t out").is_empty());
    assert!(a("echo 'use mapfile -t out'").is_empty());
}
