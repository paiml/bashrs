use super::*;

// ===== Manual Property Tests =====

#[test]
fn prop_sc2024_comments_never_diagnosed() {
    // Property: Comment lines should never produce diagnostics
    let test_cases = vec![
        "# sudo echo \"text\" > /root/file",
        "  # sudo cmd >> /var/log/app.log",
        "\t# sudo cat data >> /etc/hosts",
    ];

    for code in test_cases {
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}

#[test]
fn prop_sc2024_sudo_with_tee_never_diagnosed() {
    // Property: sudo with tee (correct usage) never diagnosed
    let test_cases = vec![
        "echo \"text\" | sudo tee /root/file",
        "cat data | sudo tee -a /etc/hosts",
    ];

    for code in test_cases {
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}

#[test]
fn prop_sc2024_sudo_tee_with_devnull_redirect() {
    // Property: sudo tee with >/dev/null should NOT be flagged
    // Issue #100 FIX: Piped sudo tee is correct usage
    let code = "cmd | sudo tee /var/log/app.log >/dev/null";
    let result = check(code);
    // Issue #100: No longer produces false positive
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn prop_sc2024_sudo_with_sh_c_never_diagnosed() {
    // Property: sudo sh -c (correct usage) never diagnosed
    // Issue #101 FIX: Redirect inside sh -c is handled by sudo
    let code = "sudo sh -c 'cmd > /var/log/app.log'";
    let result = check(code);
    // Issue #101: No longer produces false positive
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn prop_sc2024_stderr_redirect_never_diagnosed() {
    // Property: stderr redirects (2>, &>) never diagnosed
    let test_cases = vec![
        "sudo cmd 2> /var/log/error.log",
        "sudo command 2>> /var/log/error.log",
        "sudo app &> /dev/null",
    ];

    for code in test_cases {
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}

#[test]
fn prop_sc2024_no_sudo_never_diagnosed() {
    // Property: Redirects without sudo never diagnosed
    let test_cases = vec![
        "echo \"text\" > file.txt",
        "cat data >> output.log",
        "command > /tmp/file.txt",
    ];

    for code in test_cases {
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}

#[test]
fn prop_sc2024_sudo_with_stdout_redirect_always_diagnosed() {
    // Property: sudo with stdout redirect always diagnosed
    let test_cases = vec![
        "sudo echo \"text\" > /root/file",
        "sudo cat data >> /etc/hosts",
        "sudo cmd > /var/log/app.log",
    ];

    for code in test_cases {
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1, "Should diagnose: {}", code);
        assert!(result.diagnostics[0].message.contains("tee"));
    }
}

#[test]
fn prop_sc2024_multiple_violations_all_diagnosed() {
    // Property: Multiple sudo redirects should all be diagnosed
    let code = "sudo echo \"a\" > /root/a\nsudo echo \"b\" > /root/b";
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 2);
}

#[test]
fn prop_sc2024_diagnostic_code_always_sc2024() {
    // Property: All diagnostics must have code "SC2024"
    let code = "sudo echo \"a\" > /root/a\nsudo echo \"b\" > /root/b";
    let result = check(code);

    for diagnostic in &result.diagnostics {
        assert_eq!(&diagnostic.code, "SC2024");
    }
}

#[test]
fn prop_sc2024_diagnostic_severity_always_warning() {
    // Property: All diagnostics must be Warning severity
    let code = "sudo echo \"text\" > /root/file";
    let result = check(code);

    for diagnostic in &result.diagnostics {
        assert_eq!(diagnostic.severity, Severity::Warning);
    }
}

#[test]
fn prop_sc2024_empty_source_no_diagnostics() {
    // Property: Empty source should produce no diagnostics
    let result = check("");
    assert_eq!(result.diagnostics.len(), 0);
}

// ===== Original Unit Tests =====

#[test]
fn test_sc2024_sudo_redirect() {
    let code = r#"sudo echo "text" > /root/file"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, "SC2024");
    assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    assert!(result.diagnostics[0].message.contains("tee"));
}

#[test]
fn test_sc2024_sudo_append() {
    let code = r#"sudo cat data >> /etc/hosts"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].message.contains("-a"));
}

#[test]
fn test_sc2024_sudo_log() {
    let code = r#"sudo cmd > /var/log/app.log"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 1);
}

#[test]
fn test_sc2024_sudo_tee_ok() {
    let code = r#"echo "text" | sudo tee /root/file"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2024_sudo_sh_c_ok() {
    let code = r#"sudo sh -c 'cmd > /var/log/app.log'"#;
    let result = check(code);
    // Issue #101 FIX: sudo sh -c wraps the redirect, so sudo DOES affect it
    // This is now correctly recognized as valid usage
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2024_stderr_redirect_ok() {
    let code = r#"sudo cmd 2> /var/log/error.log"#;
    let result = check(code);
    // stderr redirect (2>) is different, not caught by this rule
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2024_pipe_ok() {
    let code = r#"sudo cmd | grep pattern"#;
    let result = check(code);
    // Pipe is not redirect
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2024_no_sudo_ok() {
    let code = r#"echo "text" > file"#;
    let result = check(code);
    // No sudo, no problem
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2024_multiple_issues() {
    let code = r#"
sudo echo "a" > /root/a
sudo echo "b" > /root/b
"#;
    let result = check(code);
    assert_eq!(result.diagnostics.len(), 2);
}

#[test]
fn test_sc2024_sudo_with_input_redirect_ok() {
    let code = r#"sudo cmd < /etc/config"#;
    let result = check(code);
    // Input redirect is OK
    assert_eq!(result.diagnostics.len(), 0);
}

// ===== Issue #101: sudo sh -c Tests =====
// Redirect inside sh -c is correct - sudo DOES affect it

#[test]
fn test_FP_101_sudo_sh_c_redirect_not_flagged() {
    let code = r#"sudo sh -c 'echo 10 > /proc/sys/vm/swappiness'"#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "SC2024 must NOT flag sudo sh -c 'cmd > file' - redirect is inside sh -c"
    );
}

#[test]
fn test_FP_101_sudo_bash_c_redirect_not_flagged() {
    let code = r#"sudo bash -c 'echo test > /etc/file'"#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "SC2024 must NOT flag sudo bash -c pattern"
    );
}

#[test]
fn test_FP_101_sudo_sh_c_append_not_flagged() {
    let code = r#"sudo sh -c 'echo line >> /etc/file'"#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "SC2024 must NOT flag sudo sh -c with append redirect"
    );
}

#[test]
fn test_FP_101_sudo_sh_c_double_quoted_not_flagged() {
    let code = r#"sudo sh -c "echo test > /etc/file""#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "SC2024 must NOT flag sudo sh -c with double quotes"
    );
}

#[test]
fn test_FP_101_sudo_dash_c_not_flagged() {
    let code = r#"sudo dash -c 'echo test > /etc/file'"#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "SC2024 must NOT flag sudo dash -c pattern"
    );
}

#[test]
fn test_FP_101_direct_sudo_redirect_still_flagged() {
    let code = r#"sudo echo test > /etc/file"#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        1,
        "Direct sudo redirect should still be flagged"
    );
}

// ===== Issue #100: sudo tee Tests =====
// cmd | sudo tee is the correct pattern

#[test]
fn test_FP_100_sudo_tee_devnull_not_flagged() {
    let code = r#"echo test | sudo tee /etc/file >/dev/null"#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "SC2024 must NOT flag 'cmd | sudo tee file >/dev/null'"
    );
}

#[test]
fn test_FP_100_sudo_tee_append_devnull_not_flagged() {
    let code = r#"printf '%s\n' "$VAR" | sudo tee -a /etc/fstab >/dev/null"#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "SC2024 must NOT flag sudo tee -a pattern"
    );
}

#[test]
fn test_FP_100_sudo_tee_no_devnull_not_flagged() {
    let code = r#"echo test | sudo tee /etc/file"#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "SC2024 must NOT flag sudo tee without >/dev/null"
    );
}

#[test]
fn test_FP_100_printf_sudo_tee_not_flagged() {
    let code = r#"printf '%s\n' "vm.swappiness=10" | sudo tee -a /etc/sysctl.conf >/dev/null"#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "SC2024 must NOT flag printf | sudo tee pattern"
    );
}

// ===== F004: sudo -u with user-writable target =====
// When redirecting to a user-writable location like /tmp, the warning is less relevant

#[test]
fn test_FP_004_sudo_u_tmp_not_flagged() {
    // sudo -u user redirecting to /tmp - user already has write access
    let code = r#"sudo -u user cmd > /tmp/output.txt"#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "SC2024 must NOT flag sudo redirect to /tmp (user-writable)"
    );
}

#[test]
fn test_FP_004_sudo_redirect_to_tmp_not_flagged() {
    // Any sudo redirect to /tmp should not be flagged
    let code = r#"sudo echo test > /tmp/file"#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "SC2024 must NOT flag sudo redirect to /tmp"
    );
}

#[test]
fn test_FP_004_sudo_redirect_to_var_tmp_not_flagged() {
    // sudo redirect to /var/tmp should not be flagged
    let code = r#"sudo cmd > /var/tmp/output"#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "SC2024 must NOT flag sudo redirect to /var/tmp"
    );
}

#[test]
fn test_FP_004_sudo_redirect_to_devnull_not_flagged() {
    // sudo redirect to /dev/null should not be flagged
    let code = r#"sudo cmd > /dev/null"#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        0,
        "SC2024 must NOT flag sudo redirect to /dev/null"
    );
}

#[test]
fn test_FP_004_sudo_redirect_to_root_still_flagged() {
    // sudo redirect to system directories should still be flagged
    let code = r#"sudo echo test > /root/file"#;
    let result = check(code);
    assert_eq!(
        result.diagnostics.len(),
        1,
        "SC2024 SHOULD flag sudo redirect to /root"
    );
}
