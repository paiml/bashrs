use super::*;
use crate::linter::{Diagnostic, Span};

#[test]
fn test_output_format_from_str() {
    assert_eq!(
        "human".parse::<OutputFormat>().unwrap(),
        OutputFormat::Human
    );
    assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
    assert_eq!(
        "sarif".parse::<OutputFormat>().unwrap(),
        OutputFormat::Sarif
    );
    assert!("invalid".parse::<OutputFormat>().is_err());
}

#[test]
fn test_human_output_no_issues() {
    let result = LintResult::new();
    let mut buffer = Vec::new();

    write_human(&mut buffer, &result, "test.sh").unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("No issues found"));
}

#[test]
fn test_human_output_with_diagnostics() {
    let mut result = LintResult::new();
    let span = Span::new(1, 5, 1, 10);
    result.add(Diagnostic::new(
        "SC2086",
        Severity::Warning,
        "Test warning",
        span,
    ));

    let mut buffer = Vec::new();
    write_human(&mut buffer, &result, "test.sh").unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("SC2086"));
    assert!(output.contains("Test warning"));
    assert!(output.contains("1 warning"));
}

#[test]
fn test_json_output() {
    let mut result = LintResult::new();
    let span = Span::new(1, 5, 1, 10);
    result.add(Diagnostic::new("SC2086", Severity::Warning, "Test", span));

    let mut buffer = Vec::new();
    write_json(&mut buffer, &result, "test.sh").unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("SC2086"));
    assert!(output.contains("\"file\": \"test.sh\""));

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed["diagnostics"].is_array());
}

#[test]
fn test_sarif_output() {
    let mut result = LintResult::new();
    let span = Span::new(1, 5, 1, 10);
    result.add(Diagnostic::new("SC2086", Severity::Error, "Test", span));

    let mut buffer = Vec::new();
    write_sarif(&mut buffer, &result, "test.sh").unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("2.1.0"));
    assert!(output.contains("SC2086"));

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["version"], "2.1.0");
    assert!(parsed["runs"].is_array());
}

#[test]
fn test_sarif_output_has_partial_fingerprints() {
    let mut result = LintResult::new();
    let span = Span::new(1, 5, 1, 10);
    result.add(Diagnostic::new(
        "SEC001",
        Severity::Error,
        "Injection risk",
        span,
    ));

    let mut buffer = Vec::new();
    write_sarif(&mut buffer, &result, "test.sh").unwrap();

    let output = String::from_utf8(buffer).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    let first_result = &parsed["runs"][0]["results"][0];
    assert!(
        first_result["partialFingerprints"]["primaryLocationLineHash"].is_string(),
        "Should have partialFingerprints: {output}"
    );
}

#[test]
fn test_sarif_output_has_rule_descriptors() {
    let mut result = LintResult::new();
    let span = Span::new(1, 5, 1, 10);
    result.add(Diagnostic::new(
        "SEC001",
        Severity::Error,
        "Injection risk",
        span,
    ));

    let mut buffer = Vec::new();
    write_sarif(&mut buffer, &result, "test.sh").unwrap();

    let output = String::from_utf8(buffer).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    let rules = &parsed["runs"][0]["tool"]["driver"]["rules"];
    assert!(rules.is_array(), "Should have rules array: {output}");
    let rules_arr = rules.as_array().expect("rules is array");
    assert!(!rules_arr.is_empty(), "Rules should not be empty");
    let first_rule = &rules_arr[0];
    assert_eq!(first_rule["id"], "SEC001");
    assert!(first_rule["shortDescription"]["text"].is_string());
    assert!(first_rule["helpUri"]
        .as_str()
        .expect("helpUri")
        .contains("docs/rules"));
}

#[test]
fn test_sarif_output_has_fixes() {
    let mut result = LintResult::new();
    let span = Span::new(1, 5, 1, 10);
    let mut diag = Diagnostic::new("IDEM001", Severity::Warning, "mkdir without -p", span);
    diag.fix = Some(crate::linter::Fix::new("mkdir -p /tmp/foo"));
    result.add(diag);

    let mut buffer = Vec::new();
    write_sarif(&mut buffer, &result, "test.sh").unwrap();

    let output = String::from_utf8(buffer).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    let first_result = &parsed["runs"][0]["results"][0];
    let fixes = first_result["fixes"].as_array().expect("fixes is array");
    assert_eq!(fixes.len(), 1);
    assert!(fixes[0]["description"]["text"]
        .as_str()
        .expect("desc")
        .contains("mkdir -p"));
}
