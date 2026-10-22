use super::*;

#[cfg(test)]
use proptest::prelude::*;

// ===== Manual Property Tests =====
// Establish invariants before refactoring

#[test]
fn prop_sc2096_comments_never_diagnosed() {
    // Property: Comment lines should never produce diagnostics
    let test_cases = vec![
        "# command > file1.txt > file2.txt",
        "  # 2> err1.log 2> err2.log",
        "\t# >> a.txt >> b.txt",
    ];

    for code in test_cases {
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Comments should not be diagnosed: {}",
            code
        );
    }
}

#[test]
fn prop_sc2096_single_redirects_always_ok() {
    // Property: Single redirects of any type should never be diagnosed
    let test_cases = vec![
        "command > output.txt",
        "command 2> error.log",
        "command >> append.txt",
        "command &> combined.log",
    ];

    for code in test_cases {
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Single redirects should be OK: {}",
            code
        );
    }
}

#[test]
fn prop_sc2096_different_streams_always_ok() {
    // Property: Redirecting different streams (stdout vs stderr) is always OK
    let test_cases = vec![
        "command > out.txt 2> err.txt",
        "command 2> err.txt > out.txt",
        "cmd > /dev/null 2> /tmp/err",
    ];

    for code in test_cases {
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Different streams should be OK: {}",
            code
        );
    }
}

#[test]
fn prop_sc2096_multiple_same_stream_diagnosed() {
    // Property: Multiple redirects of the same stream should be diagnosed
    let test_cases = vec![
        ("command > file1 > file2", "stdout"),
        ("command 2> err1 2> err2", "stderr"),
        ("echo a >> f1 >> f2", "append"),
    ];

    for (code, stream_type) in test_cases {
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Multiple {} redirects should be diagnosed: {}",
            stream_type,
            code
        );
        assert_eq!(result.diagnostics[0].code, "SC2096");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }
}

#[test]
fn prop_sc2096_heredocs_never_diagnosed() {
    // Property: Heredocs should never be diagnosed as duplicate redirects
    let test_cases = vec![
        "cat <<EOF > output.txt",
        "cat <<-EOF > output.txt",
        "cat <<<STRING > output.txt",
    ];

    for code in test_cases {
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Heredocs should not be diagnosed: {}",
            code
        );
    }
}

#[test]
fn prop_sc2096_chained_commands_independent() {
    // Property: Redirects in separate commands should be independent
    let test_cases = vec![
        "cmd1 > file1.txt && cmd2 > file2.txt",
        "cmd1 > out1 ; cmd2 > out2",
        "cmd1 > f1 | cmd2 > f2",
    ];

    for code in test_cases {
        let result = check(code);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Separate commands should have independent redirects: {}",
            code
        );
    }
}

#[test]
fn prop_sc2096_empty_source_no_diagnostics() {
    // Property: Empty source should produce no diagnostics
    let result = check("");
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn prop_sc2096_diagnostic_code_always_sc2096() {
    // Property: All diagnostics must have code "SC2096"
    let code = "cmd > f1 > f2\ncmd 2> e1 2> e2\necho >> a >> b";
    let result = check(code);

    for diagnostic in &result.diagnostics {
        assert_eq!(diagnostic.code, "SC2096");
    }
}

#[test]
fn prop_sc2096_diagnostic_severity_always_warning() {
    // Property: All diagnostics must be Warning severity
    let code = "cmd > f1 > f2\ncmd 2> e1 2> e2";
    let result = check(code);

    for diagnostic in &result.diagnostics {
        assert_eq!(diagnostic.severity, Severity::Warning);
    }
}

// ===== Original Unit Tests =====

#[test]
fn test_sc2096_multiple_stdout() {
    let code = r#"command > file1.txt > file2.txt"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "SC2096");
    assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    assert!(result.diagnostics[0].message.contains("stdout"));
}

#[test]
fn test_sc2096_multiple_stderr() {
    let code = r#"command 2> err1.log 2> err2.log"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].message.contains("stderr"));
}

#[test]
fn test_sc2096_multiple_append() {
    let code = r#"echo "a" >> file1.txt >> file2.txt"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].message.contains("append"));
}

#[test]
fn test_sc2096_stdout_and_stderr_ok() {
    let code = r#"command > stdout.txt 2> stderr.txt"#;
    let result = check(code);
    // Different streams, this is OK
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2096_single_redirect_ok() {
    let code = r#"command > output.txt"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2096_pipe_ok() {
    let code = r#"command | grep pattern > output.txt"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2096_both_redirect_ok() {
    let code = r#"command &> all.log"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2096_heredoc_ok() {
    let code = r#"cat <<EOF > output.txt"#;
    let result = check(code);
    // Heredoc is not a duplicate redirect
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2096_three_redirects() {
    let code = r#"echo test > a.txt > b.txt > c.txt"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 1);
}

#[test]
fn test_sc2096_chained_commands_ok() {
    let code = r#"cmd1 > file1.txt && cmd2 > file2.txt"#;
    let result = check(code);
    // Different commands, not duplicate redirects
    assert_eq!(result.diagnostics.len(), 0);
}

// ===== Generative Property Tests =====
// Using proptest for random input generation (100 cases each)

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(10))]
    #[test]
    fn prop_gen_comments_never_diagnosed(comment in r"#[^\n]{0,50}") {
        // Property: Any line starting with # should never be diagnosed
        let result = check(&comment);
        prop_assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_gen_single_stdout_always_ok(
        cmd in r"[a-z]{1,10}",
        file in r"[a-z]{1,10}\.(txt|log)"
    ) {
        // Property: Single stdout redirect should never be diagnosed
        let code = format!("{} > {}", cmd, file);
        let result = check(&code);
        prop_assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_gen_single_stderr_always_ok(
        cmd in r"[a-z]{1,10}",
        file in r"[a-z]{1,10}\.(txt|log)"
    ) {
        // Property: Single stderr redirect should never be diagnosed
        let code = format!("{} 2> {}", cmd, file);
        let result = check(&code);
        prop_assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_gen_single_append_always_ok(
        cmd in r"[a-z]{1,10}",
        file in r"[a-z]{1,10}\.(txt|log)"
    ) {
        // Property: Single append redirect should never be diagnosed
        let code = format!("{} >> {}", cmd, file);
        let result = check(&code);
        prop_assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_gen_stdout_stderr_mix_always_ok(
        cmd in r"[a-z]{1,10}",
        out_file in r"[a-z]{1,10}\.txt",
        err_file in r"[a-z]{1,10}\.log"
    ) {
        // Property: Mixing stdout and stderr redirects is always OK
        let code = format!("{} > {} 2> {}", cmd, out_file, err_file);
        let result = check(&code);
        prop_assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_gen_double_stdout_always_diagnosed(
        cmd in r"[a-z]{1,10}",
        file1 in r"[a-z]{1,5}\.txt",
        file2 in r"[a-z]{1,5}\.log"
    ) {
        // Property: Double stdout redirects should always be diagnosed
        let code = format!("{} > {} > {}", cmd, file1, file2);
        let result = check(&code);
        prop_assert_eq!(result.diagnostics.len(), 1);
        prop_assert_eq!(&result.diagnostics[0].code, "SC2096");
        prop_assert!(result.diagnostics[0].message.contains("stdout"));
    }

    #[test]
    fn prop_gen_double_stderr_always_diagnosed(
        cmd in r"[a-z]{1,10}",
        file1 in r"[a-z]{1,5}\.txt",
        file2 in r"[a-z]{1,5}\.log"
    ) {
        // Property: Double stderr redirects should always be diagnosed
        let code = format!("{} 2> {} 2> {}", cmd, file1, file2);
        let result = check(&code);
        prop_assert_eq!(result.diagnostics.len(), 1);
        prop_assert_eq!(&result.diagnostics[0].code, "SC2096");
        prop_assert!(result.diagnostics[0].message.contains("stderr"));
    }

    #[test]
    fn prop_gen_double_append_always_diagnosed(
        cmd in r"[a-z]{1,10}",
        file1 in r"[a-z]{1,5}\.txt",
        file2 in r"[a-z]{1,5}\.log"
    ) {
        // Property: Double append redirects should always be diagnosed
        let code = format!("{} >> {} >> {}", cmd, file1, file2);
        let result = check(&code);
        prop_assert_eq!(result.diagnostics.len(), 1);
        prop_assert_eq!(&result.diagnostics[0].code, "SC2096");
        prop_assert!(result.diagnostics[0].message.contains("append"));
    }

    #[test]
    fn prop_gen_heredoc_never_diagnosed(
        cmd in r"[a-z]{1,10}",
        file in r"[a-z]{1,10}\.txt"
    ) {
        // Property: Heredocs should never be diagnosed
        let code = format!("{} <<EOF > {}", cmd, file);
        let result = check(&code);
        prop_assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_gen_chained_commands_independent(
        cmd1 in r"[a-z]{1,8}",
        cmd2 in r"[a-z]{1,8}",
        file1 in r"[a-z]{1,8}\.txt",
        file2 in r"[a-z]{1,8}\.log",
        separator in r"(&&|\|\||;)"
    ) {
        // Property: Redirects in chained commands are independent
        let code = format!("{} > {} {} {} > {}", cmd1, file1, separator, cmd2, file2);
        let result = check(&code);
        prop_assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn prop_gen_diagnostic_severity_always_warning(
        cmd in r"[a-z]{1,10}",
        file1 in r"[a-z]{1,5}\.txt",
        file2 in r"[a-z]{1,5}\.log"
    ) {
        // Property: All diagnostics must be Warning severity
        let test_cases = vec![
            format!("{} > {} > {}", cmd, file1, file2),
            format!("{} 2> {} 2> {}", cmd, file1, file2),
            format!("{} >> {} >> {}", cmd, file1, file2),
        ];

        for code in test_cases {
            let result = check(&code);
            if !result.diagnostics.is_empty() {
                for diagnostic in &result.diagnostics {
                    prop_assert_eq!(diagnostic.severity, Severity::Warning);
                }
            }
        }
    }

    #[test]
    fn prop_gen_empty_lines_never_diagnosed(whitespace in r"\s{0,20}") {
        // Property: Empty or whitespace-only lines never diagnosed
        let result = check(&whitespace);
        prop_assert_eq!(result.diagnostics.len(), 0);
    }
}

// ===== bashrs#431: count redirection OPERATORS, not `>` characters =====
//
// These tests use only `check`, the quoting API and `lint_shell`, so they
// compile against the rule before the fix as well. That is how they were
// shown to go red without it.

/// BRS0009 findings through the real entry point, which masks literals for
/// the rules that ask for it and migrates SC2096 to BRS0009.
fn brs0009(src: &str) -> usize {
    let script = format!("#!/bin/bash\n{src}\n");
    crate::linter::lint_shell(&script)
        .diagnostics
        .iter()
        .filter(|d| d.code == "BRS0009")
        .count()
}

/// SC2096 findings on the input the dispatcher hands this rule.
fn dispatched(src: &str) -> usize {
    let inputs = crate::linter::quoting::RuleInputs::new(src);
    check(inputs.for_code("SC2096")).diagnostics.len()
}

/// One fixture per FORM a `>` can take inside literal text. Each holds a
/// single real redirection, or none.
const PMAT431_QUOTED_FORMS: &[(&str, &str)] = &[
    // The line from bashrs#431, verbatim.
    ("issue", "printf '%s\\n' \"a <N>m b N>0\" >&2"),
    ("single_quoted", "printf '%s\\n' 'a <N>m b N>0' >&2"),
    ("double_quoted", "echo \"a > b\" > out"),
    ("ansi_c_quoted", "echo $'a > b' > out"),
    ("heredoc_body", "cat <<EOF\na > b > c\nEOF"),
];

#[test]
fn test_PMAT431_brs0009_gt_inside_a_literal_is_not_a_redirection() {
    for (form, src) in PMAT431_QUOTED_FORMS {
        assert_eq!(
            dispatched(src),
            0,
            "{form}: SC2096 counted a `>` inside a literal: {src}"
        );
        assert_eq!(
            brs0009(src),
            0,
            "{form}: lint_shell reported BRS0009: {src}"
        );
    }
}

/// The negative control for the test above. The same fixtures UNMASKED do read
/// as two redirections, so the silence above is the masking reaching this
/// rule, not a rule that has stopped counting.
#[test]
fn test_PMAT431_brs0009_quoted_forms_fire_on_the_raw_source() {
    for (form, src) in PMAT431_QUOTED_FORMS {
        assert!(
            !check(src).diagnostics.is_empty(),
            "{form}: unmasked, this must read as a double redirection: {src}"
        );
    }
}

#[test]
fn test_PMAT431_brs0009_quoted_stream_operators_are_text() {
    // `2>` and `>>` inside a literal are text too, for the stderr and append
    // checks as much as for the stdout one.
    for src in ["echo \"2>a\" 2>b", "echo \"a >> b\" >> log"] {
        assert_eq!(dispatched(src), 0, "{src}");
        assert_eq!(brs0009(src), 0, "{src}");
    }
}

#[test]
fn test_PMAT431_brs0009_gt_that_is_not_an_operator_in_code() {
    // Code, not literals: each has one redirection of a stream, or none.
    let forms = [
        ("arith_expansion", "echo $(( a > b )) > out"),
        ("arith_command", "(( a > b )) > out"),
        ("double_bracket", "[[ $a > $b ]] > out"),
        ("escaped", "echo a \\> b > out"),
        ("param_pattern", "echo ${x//>/y} > out"),
        ("other_fds", "exec 3>a 4>b"),
        ("process_substitution", "tee >(grep x) >(grep y) >/dev/null"),
        ("background", "foo >a & bar >b"),
        ("append_two_streams", "cmd >> a 2>> b"),
        ("read_write", "cmd <> f > a"),
    ];
    for (form, src) in forms {
        let masked = crate::linter::quoting::mask_literals(src);
        assert!(
            check(&masked).diagnostics.is_empty(),
            "{form}: reported a double redirection: {src}"
        );
        assert_eq!(
            brs0009(src),
            0,
            "{form}: lint_shell reported BRS0009: {src}"
        );
    }
}

#[test]
fn test_PMAT431_brs0009_true_positives_still_fire() {
    let forms = [
        ("stdout", "echo a >x >y", "stdout"),
        ("in_substitution", "x=$(cmd >a >b)", "stdout"),
        ("stderr", "cmd 2>e1 2>e2", "stderr"),
        ("append", "echo a >>f1 >>f2", "append"),
        ("across_stderr", "cmd > a 2> e > b", "stdout"),
    ];
    for (form, src, stream) in forms {
        let result = check(src);
        assert_eq!(result.diagnostics.len(), 1, "{form}: {src}");
        assert!(
            result.diagnostics[0].message.contains(stream),
            "{form}: {src} -> {}",
            result.diagnostics[0].message
        );
        assert_eq!(dispatched(src), 1, "{form}: masking hid it: {src}");
        assert_eq!(brs0009(src), 1, "{form}: lint_shell lost it: {src}");
    }
}

#[test]
fn test_PMAT431_brs0009_descriptor_copy_ends_the_run() {
    // `2>&1` reads stdout's target at that point, so `a` receives stderr and
    // the first redirection is used.
    for src in [
        "cmd > a 2>&1 > b",
        "cmd > a >&2 > b",
        "cmd > /dev/null 2>&1",
    ] {
        assert!(check(src).diagnostics.is_empty(), "{src}");
    }
    assert_eq!(check("cmd 2>&1 > a > b").diagnostics.len(), 1);
}

#[test]
fn test_PMAT431_brs0009_double_redirections_the_character_scan_missed() {
    // Real overrides the old scan skipped: a whole line skipped for `<<`, `>|`
    // split as a pipe, a `2` ending an argument read as a descriptor, and the
    // stdout check skipped whenever `>>` appeared anywhere on the line.
    for src in [
        "cat <<EOF > a > b",
        "cmd >| a > b",
        "echo file2>out > out2",
        "cmd > a >> b > c",
    ] {
        let result = check(src);
        assert_eq!(result.diagnostics.len(), 1, "{src}");
        assert!(result.diagnostics[0].message.contains("stdout"), "{src}");
    }
}
