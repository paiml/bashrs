//! Unit tests for the GH-227 intra-file taint pass.
#![allow(clippy::unwrap_used)]

use super::*;

fn taint_of(source: &str, line_idx: usize, var: &str) -> TaintKind {
    analyze(source).var_taint(line_idx, var)
}

// ---------------------------------------------------------------------------
// intrinsic lattice
// ---------------------------------------------------------------------------

#[test]
fn test_GH227_intrinsic_positional_is_external() {
    assert_eq!(intrinsic_taint("1"), TaintKind::External);
    assert_eq!(intrinsic_taint("12"), TaintKind::External);
    assert_eq!(intrinsic_taint("@"), TaintKind::External);
    assert_eq!(intrinsic_taint("*"), TaintKind::External);
}

#[test]
fn test_GH227_intrinsic_argv_zero_is_clean() {
    assert_eq!(intrinsic_taint("0"), TaintKind::Clean);
}

#[test]
fn test_GH227_intrinsic_shell_meta_is_clean() {
    assert_eq!(intrinsic_taint("#"), TaintKind::Clean);
    assert_eq!(intrinsic_taint("?"), TaintKind::Clean);
    assert_eq!(intrinsic_taint("$"), TaintKind::Clean);
}

#[test]
fn test_GH227_intrinsic_getopts_sinks_are_external() {
    assert_eq!(intrinsic_taint("OPTARG"), TaintKind::External);
    assert_eq!(intrinsic_taint("REPLY"), TaintKind::External);
}

#[test]
fn test_GH227_intrinsic_indirect_expansion_is_external() {
    assert_eq!(intrinsic_taint("!x"), TaintKind::External);
}

#[test]
fn test_GH227_intrinsic_safe_env_vars_are_clean() {
    assert_eq!(intrinsic_taint("HOME"), TaintKind::Clean);
    assert_eq!(intrinsic_taint("PWD"), TaintKind::Clean);
    assert_eq!(intrinsic_taint("XDG_CACHE_HOME"), TaintKind::Clean);
}

#[test]
fn test_GH227_intrinsic_unknown_is_ambient() {
    assert_eq!(intrinsic_taint("SOME_UNSEEN_VAR"), TaintKind::Ambient);
}

#[test]
fn test_GH227_taint_kind_is_ordered() {
    assert!(TaintKind::Clean < TaintKind::Ambient);
    assert!(TaintKind::Ambient < TaintKind::External);
}

// ---------------------------------------------------------------------------
// var_names lexer
// ---------------------------------------------------------------------------

#[test]
fn test_GH227_var_names_plain_and_braced() {
    assert_eq!(var_names("$FOO ${BAR}"), vec!["FOO", "BAR"]);
}

#[test]
fn test_GH227_var_names_skips_command_substitution_marker() {
    // `$(` is not a name; the nested `$1` is still found.
    assert_eq!(var_names(r#"x=$(basename "$1")"#), vec!["1"]);
}

#[test]
fn test_GH227_var_names_strips_array_index_and_modifiers() {
    assert_eq!(var_names("${ARGS[0]}"), vec!["ARGS"]);
    assert_eq!(var_names("${VAR:-fallback}"), vec!["VAR"]);
    assert_eq!(var_names("${#LIST}"), vec!["LIST"]);
}

#[test]
fn test_GH227_var_names_marks_indirect_expansion() {
    assert_eq!(var_names("${!ref}"), vec!["!ref"]);
}

#[test]
fn test_GH227_var_names_handles_arithmetic_and_dangling_dollar() {
    assert_eq!(var_names("$(( 1 + 2 ))"), Vec::<String>::new());
    assert_eq!(var_names("costs 5$"), Vec::<String>::new());
    assert_eq!(var_names("${unterminated"), Vec::<String>::new());
}

// ---------------------------------------------------------------------------
// assignments
// ---------------------------------------------------------------------------

#[test]
fn test_GH227_split_assignment_accepts_modifiers() {
    assert_eq!(
        split_assignment("export X=1").map(|(n, _)| n),
        Some("X".to_string())
    );
    assert_eq!(
        split_assignment("  local X=1").map(|(n, _)| n),
        Some("X".to_string())
    );
    assert_eq!(
        split_assignment("readonly X=1").map(|(n, _)| n),
        Some("X".to_string())
    );
    // case-arm prefix
    assert_eq!(
        split_assignment(r#"    d) OUT_DIR="$OPTARG" ;;"#).map(|(n, _)| n),
        Some("OUT_DIR".to_string())
    );
}

#[test]
fn test_GH227_split_assignment_rejects_non_assignments() {
    assert!(split_assignment("X[0]=1").is_none());
    assert!(split_assignment(r#"if [ "$a" == "b" ]; then"#).is_none());
    assert!(split_assignment(r#"if [ "$a" = "b" ]; then"#).is_none());
    assert!(split_assignment("X+=1").is_none());
    assert!(split_assignment("./configure --prefix=/usr").is_none());
    assert!(split_assignment("awk -F= '{print $1}'").is_none());
    assert!(split_assignment("mkdir -p out").is_none());
}

#[test]
fn test_GH227_literal_assignment_is_clean() {
    let src = "OUT_DIR=\"build/results\"\nmkdir -p \"$OUT_DIR\"\n";
    assert_eq!(taint_of(src, 1, "OUT_DIR"), TaintKind::Clean);
}

#[test]
fn test_GH227_transitive_literal_assignment_is_clean() {
    let src = "BASE=\"/srv\"\nSUB=\"$BASE/data\"\nmkdir -p \"$SUB\"\n";
    assert_eq!(taint_of(src, 2, "SUB"), TaintKind::Clean);
}

#[test]
fn test_GH227_positional_assignment_is_external() {
    let src = "P=\"$1\"\nmkdir -p \"$P\"\n";
    assert_eq!(taint_of(src, 1, "P"), TaintKind::External);
}

#[test]
fn test_GH227_assignment_is_not_visible_before_its_line() {
    let src = "mkdir -p \"$P\"\nP=\"/lit\"\n";
    assert_eq!(taint_of(src, 0, "P"), TaintKind::Ambient);
}

#[test]
fn test_GH227_is_sanitizer_rhs() {
    assert!(is_sanitizer_rhs("$(realpath -m \"$1\")"));
    assert!(is_sanitizer_rhs("$(readlink -f \"$1\")"));
    assert!(!is_sanitizer_rhs("$(basename \"$1\")"));
}

#[test]
fn test_GH227_cmd_sub_taint() {
    assert_eq!(cmd_sub_taint("$(curl -s http://x)"), TaintKind::External);
    assert_eq!(cmd_sub_taint("$(echo hi)"), TaintKind::Clean);
    assert_eq!(cmd_sub_taint("`basename \"$1\"`"), TaintKind::Clean);
    assert_eq!(cmd_sub_taint("plain text"), TaintKind::Clean);
    // word boundary: `curler` is not `curl`
    assert_eq!(cmd_sub_taint("$(curler -s http://x)"), TaintKind::Clean);
}

// ---------------------------------------------------------------------------
// read / for
// ---------------------------------------------------------------------------

#[test]
fn test_GH227_read_marks_variables_external() {
    let src = "read -r name\nmkdir -p \"$name\"\n";
    assert_eq!(taint_of(src, 1, "name"), TaintKind::External);
}

#[test]
fn test_GH227_read_skips_option_arguments() {
    let src = "read -r -p \"prompt: \" answer\necho \"$answer\"\n";
    assert_eq!(taint_of(src, 1, "answer"), TaintKind::External);
}

#[test]
fn test_GH227_for_loop_over_argv_is_external() {
    let src = "for f in \"$@\"; do\n  cat \"$f\"\ndone\n";
    assert_eq!(taint_of(src, 1, "f"), TaintKind::External);
}

#[test]
fn test_GH227_for_loop_over_glob_is_clean() {
    let src = "for f in *.txt; do\n  cat \"$f\"\ndone\n";
    assert_eq!(taint_of(src, 1, "f"), TaintKind::Clean);
}

// ---------------------------------------------------------------------------
// guards
// ---------------------------------------------------------------------------

#[test]
fn test_GH227_line_tests_traversal() {
    assert!(line_tests_traversal(r#"if [[ "$V" == *".."* ]]; then"#));
    assert!(line_tests_traversal(r#"if [[ "$V" == /* ]]; then"#));
    assert!(line_tests_traversal(r#"grep -qE '(^|/)\.\.(/|$)'"#));
    assert!(!line_tests_traversal("echo hello"));
    // a pattern with no comparison is not a test
    assert!(!line_tests_traversal("rm -rf ../old"));
}

#[test]
fn test_GH227_line_hard_fails() {
    assert!(line_hard_fails("exit 1"));
    assert!(line_hard_fails("  return 1"));
    assert!(line_hard_fails("die \"nope\""));
    assert!(!line_hard_fails("echo bad >&2"));
}

#[test]
fn test_GH227_guard_untaints_reports_the_closing_line() {
    let lines = ["case \"$1\" in", "  *..*) exit 1 ;;", "esac", "mkdir -p x"];
    let untaints = guard_untaints(&lines, &HashSet::new());
    // The guard dominates from the line AFTER `esac` (index 2), not after `case`.
    assert_eq!(
        untaints.get(&2).map(Vec::as_slice),
        Some(&["1".to_string()][..])
    );
    assert!(!untaints.contains_key(&0));
}

#[test]
fn test_GH227_inline_case_guard_untaints_positional() {
    let src = "case \"$1\" in *..*|/*) exit 2 ;; esac\nD=\"in/$1\"\n";
    assert_eq!(taint_of(src, 1, "1"), TaintKind::Clean);
}

#[test]
fn test_GH227_guard_without_hard_failure_does_not_untaint() {
    let src = "case \"$1\" in *..*) echo bad >&2 ;; esac\nD=\"in/$1\"\n";
    assert_eq!(taint_of(src, 1, "1"), TaintKind::External);
}

#[test]
fn test_GH227_positional_inside_function_is_ambient_not_external() {
    // A function's `$1` comes from its caller; we do not resolve call sites, so
    // it is unproven (`Ambient`), never `External`.
    let src = "f() {\n  local p=\"$1\"\n  mkdir -p \"$p\"\n}\n";
    assert_eq!(taint_of(src, 2, "p"), TaintKind::Ambient);
    assert_eq!(
        analyze(src).line_taint(2, "mkdir -p \"$1\""),
        TaintKind::Ambient
    );
}

#[test]
fn test_GH227_positional_at_top_level_stays_external() {
    let src = "p=\"$1\"\nmkdir -p \"$p\"\n";
    assert_eq!(taint_of(src, 1, "p"), TaintKind::External);
}

#[test]
fn test_GH227_guard_inside_function_body_is_ignored() {
    // The `case` guards the function's own `$1`, not the script's.
    let src = "check() {\n  case \"$1\" in *..*) return 1 ;; esac\n}\nD=\"in/$1\"\n";
    assert_eq!(taint_of(src, 3, "1"), TaintKind::External);
}

// ---------------------------------------------------------------------------
// validator functions
// ---------------------------------------------------------------------------

#[test]
fn test_GH227_body_is_path_validator() {
    assert!(!body_is_path_validator(&["validate_path() {", "  :", "}"]));
    assert!(!body_is_path_validator(&[
        "check_path() {",
        "  echo \"$1\"",
        "}"
    ]));
    assert!(body_is_path_validator(&[
        "validate_path() {",
        "  case \"$1\" in *..*) return 1 ;; esac",
        "}"
    ]));
    assert!(body_is_path_validator(&[
        "validate_path() {",
        "  if [[ \"$1\" == *\"..\"* ]]; then",
        "    exit 1",
        "  fi",
        "}"
    ]));
}

#[test]
fn test_GH227_collect_validator_functions_requires_body_evidence() {
    let noop = ["validate_path() {", "    :", "}"];
    assert!(collect_validator_functions(&noop).is_empty());

    let real = [
        "validate_path() {",
        "  case \"$1\" in *..*) return 1 ;; esac",
        "}",
    ];
    assert!(collect_validator_functions(&real).contains("validate_path"));
}

#[test]
fn test_GH227_validator_call_var_extracts_argument() {
    let mut validators = HashSet::new();
    validators.insert("validate_path".to_string());

    assert_eq!(
        validator_call_var(r#"validate_path "$RAID_PATH""#, &validators),
        Some("RAID_PATH".to_string())
    );
    assert_eq!(
        validator_call_var(r#"validate_path "${RAID_PATH}" || exit 1"#, &validators),
        Some("RAID_PATH".to_string())
    );
    // a definition is not a call
    assert_eq!(validator_call_var("validate_path() {", &validators), None);
    // a function not defined in this file is not evidence
    assert_eq!(validator_call_var(r#"check_path "$X""#, &validators), None);
}

#[test]
fn test_GH227_function_header_recognition() {
    assert_eq!(function_header("foo() {"), Some("foo".to_string()));
    assert_eq!(function_header("function foo() {"), Some("foo".to_string()));
    assert_eq!(function_header("arr=()"), None);
    assert_eq!(function_header("mkdir -p x"), None);
}

#[test]
fn test_GH227_function_body_lines_covers_one_liner_and_block() {
    let lines = ["f() { :; }", "g() {", "  echo hi", "}", "echo out"];
    let bodies = function_body_lines(&lines);
    assert!(bodies.contains(&0));
    assert!(bodies.contains(&1) && bodies.contains(&2) && bodies.contains(&3));
    assert!(!bodies.contains(&4));
}

// ---------------------------------------------------------------------------
// map queries
// ---------------------------------------------------------------------------

#[test]
fn test_GH227_line_taint_takes_the_maximum() {
    let src = "A=\"lit\"\nB=\"$1\"\ncp \"$A\" \"$B\"\n";
    let map = analyze(src);
    assert_eq!(map.line_taint(2, r#"cp "$A" "$B""#), TaintKind::External);
    assert_eq!(map.line_taint(2, r#"cp "$A" /dest/"#), TaintKind::Clean);
}

#[test]
fn test_GH227_path_taint_only_looks_at_path_components() {
    let src = "X=\"a\"\ncat \"/data/$X\"\n";
    let map = analyze(src);
    assert_eq!(map.path_taint(1, r#" "/data/$X""#), TaintKind::Clean);
    assert_eq!(map.path_taint(1, r#" "/data/$1""#), TaintKind::External);
}

#[test]
fn test_GH227_analyze_handles_empty_source() {
    let map = analyze("");
    assert_eq!(map.var_taint(0, "X"), TaintKind::Ambient);
}

#[test]
fn test_GH227_quoted_heredoc_body_is_not_analysed() {
    // The body is data, not shell: the `P=` inside must not clean `P`.
    let src = "P=\"$1\"\ncat <<'EOF'\nP=\"/literal\"\nEOF\nmkdir -p \"$P\"\n";
    assert_eq!(taint_of(src, 4, "P"), TaintKind::External);
}

#[test]
fn test_GH227_comments_are_not_analysed() {
    let src = "P=\"$1\"\n# P=\"/literal\"\nmkdir -p \"$P\"\n";
    assert_eq!(taint_of(src, 2, "P"), TaintKind::External);
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(64))]

        /// The pass is total: it never panics on arbitrary input.
        #[test]
        fn prop_GH227_analyze_is_total(s in ".*") {
            let map = analyze(&s);
            let _ = map.line_taint(0, &s);
            let _ = map.path_taint(0, &s);
        }

        /// A variable assigned only from string literals is always Clean, so
        /// SEC010/SEC014 can never fire on it.
        #[test]
        fn prop_GH227_literal_only_assignment_is_clean(
            name in "[A-Z][A-Z_]{0,8}",
            value in "[a-z/][a-z/]{0,20}",
        ) {
            let src = format!("{name}=\"{value}\"\nmkdir -p \"${name}\"\n");
            prop_assert_eq!(analyze(&src).var_taint(1, &name), TaintKind::Clean);
        }

        /// A variable assigned from a positional parameter is always External.
        #[test]
        fn prop_GH227_positional_assignment_is_external(name in "[A-Z][A-Z_]{0,8}") {
            let src = format!("{name}=\"$1\"\nmkdir -p \"${name}\"\n");
            prop_assert_eq!(analyze(&src).var_taint(1, &name), TaintKind::External);
        }
    }
}
