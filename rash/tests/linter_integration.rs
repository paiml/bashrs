//! Integration tests for bashrs linting workflow
//!
//! Tests end-to-end linting behavior including:
//! - Multiple rule detection
//! - Inline suppression
//! - Rule interactions
//! - Real-world script analysis

#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity

use bashrs::linter::{lint_shell, Severity};

#[test]
fn test_full_linting_workflow_detects_multiple_issues() {
    let script = r#"
#!/bin/bash
echo $var  # SC2086
cat file.txt | grep pattern  # SC2002
    "#;

    let result = lint_shell(script);

    // Should detect both issues
    assert!(
        result.diagnostics.len() >= 2,
        "Expected at least 2 diagnostics, got {}",
        result.diagnostics.len()
    );
    assert!(
        result.diagnostics.iter().any(|d| d.code == "SC2086"),
        "Should detect SC2086 (unquoted variable)"
    );
    assert!(
        result.diagnostics.iter().any(|d| d.code == "SC2002"),
        "Should detect SC2002 (useless cat)"
    );
}

#[test]
fn test_inline_suppression_integration() {
    let script = r#"
#!/bin/bash
# bashrs disable-next-line=SC2086
echo $var
    "#;

    let result = lint_shell(script);

    // SC2086 should be suppressed
    let has_sc2086 = result.diagnostics.iter().any(|d| d.code == "SC2086");
    assert!(!has_sc2086, "SC2086 should be suppressed but was detected");
}

#[test]
fn test_file_level_suppression() {
    let script = r#"
# bashrs disable-file=SC2086,DET002
timestamp=$(date +%s)
echo $var
echo `echo test`
    "#;

    let result = lint_shell(script);

    // SC2086 and DET002 should be suppressed, but SC2006 (backticks) should still trigger
    // Note: SC2006 only triggers in non-assignment context (F080 design)
    assert!(
        !result.diagnostics.iter().any(|d| d.code == "SC2086"),
        "SC2086 should be suppressed"
    );
    assert!(
        !result.diagnostics.iter().any(|d| d.code == "DET002"),
        "DET002 should be suppressed"
    );
    assert!(
        result.diagnostics.iter().any(|d| d.code == "SC2006"),
        "SC2006 should not be suppressed"
    );
}

#[test]
fn test_determinism_and_idempotency_rules() {
    let script = r#"
#!/bin/bash
# Non-deterministic operations
timestamp=$(date +%s)
random_num=$RANDOM

# Non-idempotent operations (should suggest -p flag)
mkdir /tmp/test
    "#;

    let result = lint_shell(script);

    // Should detect DET001 ($RANDOM)
    assert!(
        result.diagnostics.iter().any(|d| d.code == "DET001"),
        "Should detect DET001 (RANDOM usage)"
    );

    // Should detect DET002 (timestamp)
    assert!(
        result.diagnostics.iter().any(|d| d.code == "DET002"),
        "Should detect DET002 (timestamp)"
    );

    // Should detect IDEM001 (mkdir without -p)
    assert!(
        result.diagnostics.iter().any(|d| d.code == "IDEM001"),
        "Should detect IDEM001 (mkdir without -p)"
    );
}

#[test]
fn test_security_rules_integration() {
    let script = r#"
#!/bin/bash
# SQL injection risk
query="SELECT * FROM users WHERE name = '$user_input'"

# Command injection risk
eval "rm -rf $user_dir"

# Hardcoded credentials
password="secret123"
    "#;

    let result = lint_shell(script);

    // Should detect security issues
    let has_security_issues = result.diagnostics.iter().any(|d| d.code.starts_with("SEC"));

    assert!(
        has_security_issues,
        "Should detect security issues (SEC rules)"
    );
}

#[test]
fn test_clean_script_has_no_issues() {
    let script = r#"
#!/bin/sh
set -euo pipefail

name="${1:-default}"
echo "Hello, ${name}"

if [ -f "config.txt" ]; then
    echo "Config exists"
fi

mkdir -p "/tmp/safe_dir"
exit 0
    "#;

    let result = lint_shell(script);

    // Clean script should have minimal or no issues
    let error_count = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();

    assert!(
        error_count == 0,
        "Clean script should have 0 errors, got {}",
        error_count
    );
}

#[test]
fn test_shellcheck_equivalent_rules() {
    let script = r#"
#!/bin/bash
# Test multiple ShellCheck-equivalent rules

# SC2086: Unquoted expansion
echo $var

# SC2002: Useless cat
cat file | grep pattern

# SC2006: Backticks (non-assignment context, F080 allows assignments)
echo `date`

# SC2046: Unquoted command substitution
for file in $(ls *.txt); do
    echo $file
done
    "#;

    let result = lint_shell(script);

    // Verify all expected rules trigger
    let detected_codes: Vec<&str> = result.diagnostics.iter().map(|d| d.code.as_str()).collect();

    assert!(detected_codes.contains(&"SC2086"), "Should detect SC2086");
    assert!(detected_codes.contains(&"SC2002"), "Should detect SC2002");
    assert!(detected_codes.contains(&"SC2006"), "Should detect SC2006");
    assert!(detected_codes.contains(&"SC2046"), "Should detect SC2046");
}

#[test]
fn test_linting_preserves_line_numbers() {
    let script = "#!/bin/bash\n\necho $var\n";

    let result = lint_shell(script);

    // Find SC2086 diagnostic
    let sc2086 = result
        .diagnostics
        .iter()
        .find(|d| d.code == "SC2086")
        .expect("Should detect SC2086");

    // Should report line 3 (where echo $var is)
    assert_eq!(
        sc2086.span.start_line, 3,
        "Should report correct line number"
    );
}

#[test]
fn test_fix_suggestions_are_provided() {
    let script = "echo $var";

    let result = lint_shell(script);

    // Find SC2086
    let sc2086 = result
        .diagnostics
        .iter()
        .find(|d| d.code == "SC2086")
        .expect("Should detect SC2086");

    // Should have a fix suggestion
    assert!(
        sc2086.fix.is_some(),
        "SC2086 should provide a fix suggestion"
    );

    let fix = sc2086.fix.as_ref().unwrap();
    assert!(
        fix.replacement.contains("\"$var\""),
        "Fix should quote the variable"
    );
}
