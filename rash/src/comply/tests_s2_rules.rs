#[test]
fn test_idempotency_makefile_not_checked() {
    // Idempotency rules only apply to ShellScript and ShellConfig, not Makefile
    let content = "all:\n\tmkdir /tmp/build\n";
    let artifact = Artifact::new(
        PathBuf::from("Makefile"),
        Scope::Project,
        ArtifactKind::Makefile,
    );
    let result = check_rule(RuleId::Idempotency, content, &artifact);
    assert!(result.passed, "Makefile should skip idempotency check");
}

#[test]
fn test_idempotency_rm_rf_is_fine() {
    let content = "#!/bin/sh\nrm -rf /tmp/build\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Idempotency, content, &artifact);
    assert!(result.passed, "rm -rf should be considered idempotent");
}

#[test]
fn test_idempotency_ln_sf_is_fine() {
    let content = "#!/bin/sh\nln -sf /src /dst\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Idempotency, content, &artifact);
    assert!(result.passed, "ln -sf should be considered idempotent");
}

#[test]
fn test_idempotency_ln_s_without_f_fails() {
    let content = "#!/bin/sh\nln -s /src /dst\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Idempotency, content, &artifact);
    assert!(!result.passed, "ln -s without -f should fail idempotency");
}

#[test]
fn test_determinism_date_patterns() {
    let content = "#!/bin/sh\nTIMESTAMP=$(date +%s)\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Determinism, content, &artifact);
    assert!(
        !result.passed,
        "date +%s should be flagged as non-deterministic"
    );
}

#[test]
fn test_determinism_date_nano() {
    let content = "#!/bin/sh\nNANO=$(date +%N)\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Determinism, content, &artifact);
    assert!(
        !result.passed,
        "date +%N should be flagged as non-deterministic"
    );
}

#[test]
fn test_security_wget_pipe_sh() {
    let content = "#!/bin/sh\nwget -q https://example.com/setup.sh | sh\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "wget | sh should be flagged as SEC002");
}

#[test]
fn test_shellcheck_dangerous_rm_rf() {
    let content = "#!/bin/sh\nrm -rf /$DIR\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::ShellCheck, content, &artifact);
    assert!(
        !result.passed,
        "rm -rf with variable path should be flagged as SC2115"
    );
}

#[test]
fn test_dockerfile_add_http_is_ok() {
    let content = "FROM ubuntu:22.04\nADD https://example.com/file.tar.gz /app/\nUSER nobody\n";
    let artifact = Artifact::new(
        PathBuf::from("Dockerfile"),
        Scope::Project,
        ArtifactKind::Dockerfile,
    );
    let result = check_rule(RuleId::DockerfileBest, content, &artifact);
    assert!(result.passed, "ADD with HTTP URL should be allowed");
}

#[test]
fn test_pzsh_budget_always_passes() {
    let content = "anything here";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::PzshBudget, content, &artifact);
    assert!(
        result.passed,
        "PzshBudget should always pass (handled externally)"
    );
}

#[test]
fn test_rule_id_codes_complete() {
    // Verify all 10 rules have unique codes
    let rules = vec![
        RuleId::Posix,
        RuleId::Determinism,
        RuleId::Idempotency,
        RuleId::Security,
        RuleId::Quoting,
        RuleId::ShellCheck,
        RuleId::MakefileSafety,
        RuleId::DockerfileBest,
        RuleId::ConfigHygiene,
        RuleId::PzshBudget,
    ];
    let codes: Vec<&str> = rules.iter().map(|r| r.code()).collect();
    assert_eq!(codes.len(), 10);
    // Verify sequential COMPLY-001 through COMPLY-010
    for (i, code) in codes.iter().enumerate() {
        assert_eq!(*code, format!("COMPLY-{:03}", i + 1));
    }
}

#[test]
fn test_rule_weights_sum_to_110() {
    // Total weight pool: 20+15+15+20+10+10+5+5+5+5 = 110
    let rules = vec![
        RuleId::Posix,
        RuleId::Determinism,
        RuleId::Idempotency,
        RuleId::Security,
        RuleId::Quoting,
        RuleId::ShellCheck,
        RuleId::MakefileSafety,
        RuleId::DockerfileBest,
        RuleId::ConfigHygiene,
        RuleId::PzshBudget,
    ];
    let total: u32 = rules.iter().map(|r| r.weight()).sum();
    assert_eq!(total, 110, "Total weight pool should be 110");
}

#[test]
fn test_devcontainer_has_no_applicable_rules() {
    let rules = RuleId::applicable_rules(ArtifactKind::DevContainer);
    assert!(
        rules.is_empty(),
        "DevContainer should have no applicable rules"
    );
}

#[test]
fn test_workflow_only_has_security() {
    let rules = RuleId::applicable_rules(ArtifactKind::Workflow);
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0], RuleId::Security);
}

// ═══════════════════════════════════════════════════════════════
// COMPLY-005 quote tracker: escaped quotes and subshell handling
// ═══════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════
// SEC004: TLS verification disabled
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sec004_wget_no_check_certificate() {
    let content = "#!/bin/sh\nwget --no-check-certificate https://example.com/file\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(
        !result.passed,
        "SEC004: --no-check-certificate should be flagged"
    );
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("SEC004")));
}

#[test]
fn test_sec004_curl_insecure() {
    let content = "#!/bin/sh\ncurl --insecure https://api.example.com/data\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "SEC004: --insecure should be flagged");
}

#[test]
fn test_sec004_curl_k_flag() {
    let content = "#!/bin/sh\ncurl -k https://api.example.com/data\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(!result.passed, "SEC004: curl -k should be flagged");
}

#[test]
fn test_sec004_curl_without_k_is_ok() {
    let content = "#!/bin/sh\ncurl https://api.example.com/data\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(result.passed, "curl without TLS flags should pass");
}

// ═══════════════════════════════════════════════════════════════
// SEC005: Hardcoded secrets
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sec005_hardcoded_api_key() {
    let content = "#!/bin/sh\nAPI_KEY=\"sk-1234567890abcdef\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(
        !result.passed,
        "SEC005: hardcoded API_KEY should be flagged"
    );
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("SEC005")));
}

#[test]
fn test_sec005_hardcoded_password() {
    let content = "#!/bin/sh\nPASSWORD=\"MyS3cret!\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(
        !result.passed,
        "SEC005: hardcoded PASSWORD should be flagged"
    );
}

#[test]
fn test_sec005_github_token_prefix() {
    let content = "#!/bin/sh\nTOKEN=\"ghp_xxxxxxxxxxxxxxxxxxxx\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(
        !result.passed,
        "SEC005: ghp_ token prefix should be flagged"
    );
}

#[test]
fn test_sec005_variable_expansion_not_flagged() {
    let content = "#!/bin/sh\nAPI_KEY=\"$MY_API_KEY\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    // Should not flag variable expansion as hardcoded secret
    let sec005_violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("SEC005"))
        .collect();
    assert!(
        sec005_violations.is_empty(),
        "Variable expansion should not trigger SEC005: {:?}",
        sec005_violations
    );
}

#[test]
fn test_sec005_empty_value_not_flagged() {
    let content = "#!/bin/sh\nAPI_KEY=\"\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    let sec005_violations: Vec<_> = result
        .violations
        .iter()
        .filter(|v| v.message.contains("SEC005"))
        .collect();
    assert!(
        sec005_violations.is_empty(),
        "Empty value should not trigger SEC005"
    );
}

// ═══════════════════════════════════════════════════════════════
// SEC006: Unsafe temporary files
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sec006_unsafe_tmp_path() {
    let content = "#!/bin/sh\nTMPFILE=\"/tmp/myapp.tmp\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(
        !result.passed,
        "SEC006: /tmp/ literal path should be flagged"
    );
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("SEC006")));
}

#[test]
fn test_sec006_mktemp_is_ok() {
    let content = "#!/bin/sh\nTMPFILE=\"$(mktemp)\"\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(result.passed, "mktemp usage should not be flagged");
}

// ═══════════════════════════════════════════════════════════════
// SEC007: sudo with dangerous command
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sec007_sudo_rm_rf_unquoted() {
    let content = "#!/bin/sh\nsudo rm -rf $DIR\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(
        !result.passed,
        "SEC007: sudo rm -rf with unquoted var should be flagged"
    );
    assert!(result
        .violations
        .iter()
        .any(|v| v.message.contains("SEC007")));
}

#[test]
fn test_sec007_sudo_chmod_777() {
    let content = "#!/bin/sh\nsudo chmod 777 $FILE\n";
    let artifact = Artifact::new(
        PathBuf::from("test.sh"),
        Scope::Project,
        ArtifactKind::ShellScript,
    );
    let result = check_rule(RuleId::Security, content, &artifact);
    assert!(
        !result.passed,
        "SEC007: sudo chmod 777 with unquoted var should be flagged"
    );
}

#[test]
fn test_sec007_sudo_rm_rf_quoted_is_ok() {
    let content = "#!/bin/sh\nsudo rm -rf \"$DIR\"\n";
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
        "Quoted variable with sudo should not trigger SEC007"
    );
}
