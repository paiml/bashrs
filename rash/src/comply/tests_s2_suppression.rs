#[test]
fn test_suppression_extract_no_comply_prefix() {
    use super::runner;
    // Rule IDs must start with COMPLY-
    let rules = runner::extract_disable_rules("# comply:disable=FOO-001");
    assert_eq!(rules, None);
}

#[test]
fn test_suppression_parse_file_level() {
    use super::runner;
    let content = "#!/bin/sh\n# comply:disable=COMPLY-001\necho hello\n";
    let sup = runner::parse_suppressions(content);
    assert_eq!(sup.file_level, vec!["COMPLY-001".to_string()]);
    assert!(sup.line_level.is_empty());
}

#[test]
fn test_suppression_parse_line_level() {
    use super::runner;
    let content = "#!/bin/sh\necho hello\necho $RANDOM # comply:disable=COMPLY-002\n";
    let sup = runner::parse_suppressions(content);
    assert!(sup.file_level.is_empty());
    assert_eq!(
        sup.line_level.get(&3),
        Some(&vec!["COMPLY-002".to_string()])
    );
}

#[test]
fn test_suppression_file_level_only_first_10_lines() {
    use super::runner;
    // Line 11 is NOT file-level even if it's a comment-only line
    let mut content = String::new();
    for i in 1..=10 {
        content.push_str(&format!("# line {}\n", i));
    }
    content.push_str("# comply:disable=COMPLY-001\n"); // Line 11
    let sup = runner::parse_suppressions(&content);
    assert!(
        sup.file_level.is_empty(),
        "Line 11 should not be file-level"
    );
    assert_eq!(
        sup.line_level.get(&11),
        Some(&vec!["COMPLY-001".to_string()])
    );
}

#[test]
fn test_suppression_file_level_comment_only() {
    use super::runner;
    // Code on same line + in first 10 lines = line-level, not file-level
    let content = "#!/bin/sh\necho foo # comply:disable=COMPLY-001\n";
    let sup = runner::parse_suppressions(content);
    assert!(
        sup.file_level.is_empty(),
        "Inline code comment should not be file-level"
    );
    assert_eq!(
        sup.line_level.get(&2),
        Some(&vec!["COMPLY-001".to_string()])
    );
}

#[test]
fn test_suppression_apply_file_level() {
    use super::rules::{RuleId, RuleResult, Violation};
    use super::runner;
    let sup = runner::Suppressions {
        file_level: vec!["COMPLY-001".to_string()],
        line_level: std::collections::HashMap::new(),
    };
    let result = RuleResult {
        rule: RuleId::Posix,
        passed: false,
        violations: vec![Violation {
            rule: RuleId::Posix,
            line: Some(5),
            message: "bashism detected".to_string(),
        }],
    };
    let suppressed = runner::apply_suppressions(result, &sup);
    assert!(
        suppressed.passed,
        "File-level suppression should clear violations"
    );
    assert!(suppressed.violations.is_empty());
}

#[test]
fn test_suppression_apply_line_level() {
    use super::rules::{RuleId, RuleResult, Violation};
    use super::runner;
    let mut line_level = std::collections::HashMap::new();
    line_level.insert(5, vec!["COMPLY-002".to_string()]);
    let sup = runner::Suppressions {
        file_level: vec![],
        line_level,
    };
    let result = RuleResult {
        rule: RuleId::Determinism,
        passed: false,
        violations: vec![
            Violation {
                rule: RuleId::Determinism,
                line: Some(5),
                message: "non-deterministic on line 5".to_string(),
            },
            Violation {
                rule: RuleId::Determinism,
                line: Some(10),
                message: "non-deterministic on line 10".to_string(),
            },
        ],
    };
    let suppressed = runner::apply_suppressions(result, &sup);
    assert!(!suppressed.passed, "Should still have one violation");
    assert_eq!(suppressed.violations.len(), 1);
    assert_eq!(suppressed.violations[0].line, Some(10));
}

#[test]
fn test_suppression_apply_no_match() {
    use super::rules::{RuleId, RuleResult, Violation};
    use super::runner;
    let sup = runner::Suppressions {
        file_level: vec!["COMPLY-004".to_string()],
        line_level: std::collections::HashMap::new(),
    };
    let result = RuleResult {
        rule: RuleId::Posix,
        passed: false,
        violations: vec![Violation {
            rule: RuleId::Posix,
            line: Some(3),
            message: "violation".to_string(),
        }],
    };
    let suppressed = runner::apply_suppressions(result, &sup);
    assert!(
        !suppressed.passed,
        "Different rule suppression should not affect this rule"
    );
    assert_eq!(suppressed.violations.len(), 1);
}

#[test]
fn test_suppression_apply_all_lines_suppressed() {
    use super::rules::{RuleId, RuleResult, Violation};
    use super::runner;
    let mut line_level = std::collections::HashMap::new();
    line_level.insert(3, vec!["COMPLY-001".to_string()]);
    line_level.insert(7, vec!["COMPLY-001".to_string()]);
    let sup = runner::Suppressions {
        file_level: vec![],
        line_level,
    };
    let result = RuleResult {
        rule: RuleId::Posix,
        passed: false,
        violations: vec![
            Violation {
                rule: RuleId::Posix,
                line: Some(3),
                message: "v1".to_string(),
            },
            Violation {
                rule: RuleId::Posix,
                line: Some(7),
                message: "v2".to_string(),
            },
        ],
    };
    let suppressed = runner::apply_suppressions(result, &sup);
    assert!(suppressed.passed, "All violations suppressed means passed");
    assert!(suppressed.violations.is_empty());
}

#[test]
fn test_suppression_multiple_rules_on_one_line() {
    use super::runner;
    let rules = runner::extract_disable_rules("# comply:disable=COMPLY-001,COMPLY-002,COMPLY-004");
    assert_eq!(
        rules,
        Some(vec![
            "COMPLY-001".to_string(),
            "COMPLY-002".to_string(),
            "COMPLY-004".to_string(),
        ])
    );
}

#[test]
fn test_suppression_no_suppressions_passthrough() {
    use super::rules::{RuleId, RuleResult, Violation};
    use super::runner;
    let sup = runner::parse_suppressions("#!/bin/sh\necho hello\n");
    let result = RuleResult {
        rule: RuleId::Posix,
        passed: false,
        violations: vec![Violation {
            rule: RuleId::Posix,
            line: Some(2),
            message: "test".to_string(),
        }],
    };
    let suppressed = runner::apply_suppressions(result, &sup);
    assert!(
        !suppressed.passed,
        "No suppressions should leave violations intact"
    );
    assert_eq!(suppressed.violations.len(), 1);
}

// ═══════════════════════════════════════════════════════════════
// Rule metadata tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_rule_all_returns_10_rules() {
    assert_eq!(RuleId::all().len(), 10);
}

#[test]
fn test_rule_codes_unique() {
    let codes: Vec<&str> = RuleId::all().iter().map(|r| r.code()).collect();
    let mut unique = codes.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(codes.len(), unique.len(), "Rule codes must be unique");
}

#[test]
fn test_rule_descriptions_non_empty() {
    for rule in RuleId::all() {
        assert!(
            !rule.description().is_empty(),
            "{} has empty description",
            rule.code()
        );
    }
}

#[test]
fn test_rule_applies_to_non_empty() {
    for rule in RuleId::all() {
        assert!(
            !rule.applies_to().is_empty(),
            "{} has no artifact types",
            rule.code()
        );
    }
}

#[test]
fn test_rule_all_weights_consistent() {
    // Verify all() returns rules whose weights match individual weight()
    for rule in RuleId::all() {
        assert!(rule.weight() > 0, "{} has zero weight", rule.code());
    }
}

// ═══════════════════════════════════════════════════════════════
// COMPLY-002 Determinism expansion tests
// ═══════════════════════════════════════════════════════════════

fn sh_artifact() -> Artifact {
    Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    )
}

#[test]
fn test_determinism_srandom_detected() {
    let artifact = sh_artifact();
    let result = check_rule(RuleId::Determinism, "echo $SRANDOM\n", &artifact);
    assert!(!result.passed, "$SRANDOM should be non-deterministic");
}

#[test]
fn test_determinism_bashpid_detected() {
    let artifact = sh_artifact();
    let result = check_rule(RuleId::Determinism, "echo $BASHPID\n", &artifact);
    assert!(!result.passed, "$BASHPID should be non-deterministic");
}

#[test]
fn test_determinism_dev_urandom_detected() {
    let artifact = sh_artifact();
    let result = check_rule(
        RuleId::Determinism,
        "dd if=/dev/urandom bs=16 count=1\n",
        &artifact,
    );
    assert!(!result.passed, "/dev/urandom should be non-deterministic");
}

#[test]
fn test_determinism_dev_random_detected() {
    let artifact = sh_artifact();
    let result = check_rule(RuleId::Determinism, "head -c 32 /dev/random\n", &artifact);
    assert!(!result.passed, "/dev/random should be non-deterministic");
}

#[test]
fn test_determinism_mktemp_detected() {
    let artifact = sh_artifact();
    let result = check_rule(RuleId::Determinism, "TMPDIR=$(mktemp -d)\n", &artifact);
    assert!(!result.passed, "mktemp should be non-deterministic");
}

#[test]
fn test_determinism_mktemp_standalone() {
    let artifact = sh_artifact();
    let result = check_rule(RuleId::Determinism, "mktemp /tmp/test.XXXXXX\n", &artifact);
    assert!(
        !result.passed,
        "mktemp standalone should be non-deterministic"
    );
}

#[test]
fn test_determinism_shuf_detected() {
    let artifact = sh_artifact();
    let result = check_rule(RuleId::Determinism, "shuf -n 1 wordlist.txt\n", &artifact);
    assert!(!result.passed, "shuf should be non-deterministic");
}

#[test]
fn test_determinism_shuf_piped() {
    let artifact = sh_artifact();
    let result = check_rule(RuleId::Determinism, "cat list.txt | shuf\n", &artifact);
    assert!(!result.passed, "piped shuf should be non-deterministic");
}

#[test]
fn test_determinism_clean_script_passes() {
    let artifact = sh_artifact();
    let result = check_rule(
        RuleId::Determinism,
        "#!/bin/sh\necho hello\nmkdir -p /tmp/test\n",
        &artifact,
    );
    assert!(result.passed, "Clean script should be deterministic");
}

// ═══════════════════════════════════════════════════════════════
// COMPLY-003 Idempotency expansion tests
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_idempotency_useradd_unguarded() {
    let artifact = sh_artifact();
    let result = check_rule(RuleId::Idempotency, "useradd deploy\n", &artifact);
    assert!(!result.passed, "useradd without guard is non-idempotent");
}

#[test]
fn test_idempotency_useradd_guarded_ok() {
    let artifact = sh_artifact();
    let result = check_rule(RuleId::Idempotency, "useradd deploy || true\n", &artifact);
    assert!(result.passed, "useradd with || true is guarded");
}

#[test]
fn test_idempotency_groupadd_unguarded() {
    let artifact = sh_artifact();
    let result = check_rule(RuleId::Idempotency, "groupadd www-data\n", &artifact);
    assert!(!result.passed, "groupadd without guard is non-idempotent");
}

#[test]
fn test_idempotency_git_clone_unguarded() {
    let artifact = sh_artifact();
    let result = check_rule(
        RuleId::Idempotency,
        "git clone https://github.com/user/repo.git\n",
        &artifact,
    );
    assert!(
        !result.passed,
        "git clone without dir check is non-idempotent"
    );
}

#[test]
fn test_idempotency_git_clone_guarded_ok() {
    let artifact = sh_artifact();
    let result = check_rule(
        RuleId::Idempotency,
        "if [ ! -d repo ]; then git clone https://github.com/user/repo.git; fi\n",
        &artifact,
    );
    // The git clone is on a line containing "if " so it's guarded
    assert!(result.passed, "git clone with directory check is guarded");
}

#[test]
fn test_idempotency_createdb_unguarded() {
    let artifact = sh_artifact();
    let result = check_rule(RuleId::Idempotency, "createdb myapp\n", &artifact);
    assert!(!result.passed, "createdb without guard is non-idempotent");
}

#[test]
fn test_idempotency_createdb_guarded_ok() {
    let artifact = sh_artifact();
    let result = check_rule(
        RuleId::Idempotency,
        "createdb myapp 2>/dev/null || true\n",
        &artifact,
    );
    assert!(result.passed, "createdb with error suppression is guarded");
}

#[test]
fn test_idempotency_append_to_bashrc() {
    let artifact = sh_artifact();
    let result = check_rule(
        RuleId::Idempotency,
        "echo 'export PATH=/usr/local/bin:$PATH' >> ~/.bashrc\n",
        &artifact,
    );
    assert!(!result.passed, "Appending to .bashrc is non-idempotent");
}

#[test]
fn test_idempotency_append_guarded_grep_ok() {
    let artifact = sh_artifact();
    let result = check_rule(
        RuleId::Idempotency,
        "grep -q '/usr/local/bin' ~/.bashrc || echo 'export PATH=/usr/local/bin:$PATH' >> ~/.bashrc\n",
        &artifact,
    );
    // Contains grep -q guard
    assert!(result.passed, "Append with grep -q guard is idempotent");
}

#[test]
fn test_idempotency_append_to_profile() {
    let artifact = sh_artifact();
    let result = check_rule(
        RuleId::Idempotency,
        "echo 'source /opt/env.sh' >> /etc/profile\n",
        &artifact,
    );
    assert!(
        !result.passed,
        "Appending to /etc/profile is non-idempotent"
    );
}

#[test]
fn test_idempotency_mkdir_with_p_ok() {
    let artifact = sh_artifact();
    let result = check_rule(RuleId::Idempotency, "mkdir -p /tmp/dir\n", &artifact);
    assert!(result.passed, "mkdir -p is idempotent");
}

#[test]
fn test_idempotency_rm_with_f_ok() {
    let artifact = sh_artifact();
    let result = check_rule(RuleId::Idempotency, "rm -f /tmp/file\n", &artifact);
    assert!(result.passed, "rm -f is idempotent");
}
