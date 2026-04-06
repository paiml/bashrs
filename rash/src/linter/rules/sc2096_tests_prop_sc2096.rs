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
