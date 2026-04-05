
#[test]
fn test_sec007_sudo_safe_command_not_flagged() {
    let content = "#!/bin/sh\nsudo apt-get update\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    let sec007_violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("SEC007"))
        .collect();
    assert!(
        sec007_violations.is_empty(),
        "sudo with safe command should not trigger SEC007"
    );
}

#[test]
fn test_quoting_escaped_quotes_no_false_positive() {
    // echo "echo \"Line $i: Hello\"" — $i is inside double quotes (escaped inner quotes)
    let content = "#!/bin/sh\necho \"echo \\\"Line $i: Hello\\\"\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Quoting, content, &artifact);
    assert!(
        result.passed,
        "Escaped quotes should not cause false positive: {:?}",
        result.violations
    );
}

#[test]
fn test_quoting_subshell_no_false_positive() {
    // SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    let content = "#!/bin/sh\nSCRIPT_DIR=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && pwd)\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Quoting, content, &artifact);
    assert!(
        result.passed,
        "Subshell with nested quotes should not flag: {:?}",
        result.violations
    );
}

#[test]
fn test_quoting_simple_subshell_not_flagged() {
    // OUTPUT=$(date +%Y-%m-%d) — inside $() is a separate context
    let content = "#!/bin/sh\nOUTPUT=\"$(date +%Y-%m-%d)\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Quoting, content, &artifact);
    assert!(result.passed, "Variable in subshell should not be flagged");
}

#[test]
fn test_quoting_unquoted_still_detected() {
    // Plain unquoted $VAR should still be detected
    let content = "#!/bin/sh\necho $UNQUOTED\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Quoting, content, &artifact);
    assert!(
        !result.passed,
        "Unquoted $UNQUOTED should still be detected"
    );
}

#[test]
fn test_quoting_backslash_dollar_not_flagged() {
    // \$VAR is literal, not an expansion
    let content = "#!/bin/sh\necho \\$NOTAVAR\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Quoting, content, &artifact);
    assert!(
        result.passed,
        "Escaped \\$VAR should not be flagged: {:?}",
        result.violations
    );
}

// ─── COMPLY-001 Bashism Detection Expansion ───

#[test]
fn test_posix_function_keyword_detected() {
    let content = "#!/bin/sh\nfunction greet {\n  echo hello\n}\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        !result.passed,
        "function keyword should be detected as bashism"
    );
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("function keyword")));
}

#[test]
fn test_posix_function_keyword_with_parens_detected() {
    let content = "#!/bin/sh\nfunction greet() {\n  echo hello\n}\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        !result.passed,
        "function greet() should be detected as bashism"
    );
}

#[test]
fn test_posix_name_parens_no_false_positive() {
    // POSIX-valid function definition: name() { ... }
    let content = "#!/bin/sh\ngreet() {\n  echo hello\n}\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        result.passed,
        "POSIX name() should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_posix_standalone_arithmetic_detected() {
    let content = "#!/bin/sh\n(( i++ ))\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "(( )) should be detected as bashism");
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("(( ))")));
}

#[test]
fn test_posix_dollar_arithmetic_no_false_positive() {
    // $(( )) is POSIX arithmetic expansion
    let content = "#!/bin/sh\nresult=$(( 1 + 2 ))\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        result.passed,
        "$(( )) should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_posix_arithmetic_after_semicolon() {
    let content = "#!/bin/sh\necho start; (( count++ ))\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "(( )) after semicolon should be detected");
}

#[test]
fn test_posix_herestring_detected() {
    let content = "#!/bin/sh\nread x <<< \"hello\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        !result.passed,
        "<<< here-string should be detected as bashism"
    );
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("here-string")));
}

#[test]
fn test_posix_heredoc_no_false_positive() {
    // << heredoc is POSIX
    let content = "#!/bin/sh\ncat << EOF\nhello\nEOF\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        result.passed,
        "<< heredoc should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_posix_select_statement_detected() {
    let content = "#!/bin/sh\nselect opt in a b c; do echo $opt; done\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        !result.passed,
        "select statement should be detected as bashism"
    );
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("select")));
}

#[test]
fn test_posix_pattern_substitution_detected() {
    let content = "#!/bin/sh\necho ${var//old/new}\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(!result.passed, "pattern substitution should be detected");
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("pattern substitution")));
}

#[test]
fn test_posix_single_pattern_substitution_detected() {
    let content = "#!/bin/sh\necho ${var/old/new}\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        !result.passed,
        "single pattern substitution should be detected"
    );
}

#[test]
fn test_posix_default_expansion_no_false_positive() {
    // ${var:-default} is POSIX
    let content = "#!/bin/sh\necho ${var:-default}\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        result.passed,
        "POSIX default expansion should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_posix_default_with_path_no_false_positive() {
    // ${TMPDIR:-/tmp} is POSIX default expansion containing a path — NOT pattern substitution
    let content = "#!/bin/sh\ntrap 'rm -rf \"${TMPDIR:-/tmp}/rash\"' EXIT\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        result.passed,
        "POSIX default with path value should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_posix_prefix_removal_no_false_positive() {
    // ${var#*/} is POSIX prefix removal — NOT pattern substitution
    let content = "#!/bin/sh\necho ${path#*/}\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        result.passed,
        "POSIX prefix removal should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_posix_suffix_removal_no_false_positive() {
    // ${var%/*} is POSIX suffix removal — NOT pattern substitution
    let content = "#!/bin/sh\necho ${path%/*}\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        result.passed,
        "POSIX suffix removal should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_posix_error_expansion_no_false_positive() {
    // ${var:?error} is POSIX
    let content = "#!/bin/sh\necho ${var:?error}\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        result.passed,
        "POSIX error expansion should not be flagged: {:?}",
        result.violations
    );
}

#[test]
fn test_posix_case_modification_lower_detected() {
    let content = "#!/bin/sh\necho ${var,,}\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        !result.passed,
        "lowercase case modification should be detected"
    );
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("case modification")));
}

#[test]
fn test_posix_case_modification_upper_detected() {
    let content = "#!/bin/sh\necho ${var^^}\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        !result.passed,
        "uppercase case modification should be detected"
    );
}

#[test]
fn test_posix_pipefail_detected() {
    let content = "#!/bin/sh\nset -o pipefail\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        !result.passed,
        "set -o pipefail should be detected as bashism"
    );
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("pipefail")));
}

#[test]
fn test_posix_euo_pipefail_detected() {
    let content = "#!/bin/sh\nset -euo pipefail\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        !result.passed,
        "set -euo pipefail should be detected as bashism"
    );
}

#[test]
fn test_posix_set_e_no_false_positive() {
    // set -e is POSIX
    let content = "#!/bin/sh\nset -e\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Posix, content, &artifact);
    assert!(
        result.passed,
        "set -e should not be flagged: {:?}",
        result.violations
    );
}

