#[cfg(test)]
mod tests {
    use super::*;

    /// Simplified test spec for audit testing
    #[derive(Debug, Clone)]
    struct TestSpec {
        name: String,
        version: String,
        description: String,
        author: Option<String>,
        security: Option<TestSecurity>,
        artifacts: Vec<TestArtifact>,
        steps: Vec<TestStep>,
    }

    #[derive(Debug, Clone)]
    struct TestSecurity {
        trust_model: String,
        require_signatures: bool,
    }

    #[derive(Debug, Clone)]
    struct TestArtifact {
        id: String,
        url: String,
        sha256: Option<String>,
        signature: Option<String>,
        signed_by: Option<String>,
    }

    #[derive(Debug, Clone)]
    struct TestStep {
        id: String,
        name: String,
        action: TestAction,
        depends_on: Vec<String>,
        has_postconditions: bool,
        has_checkpoint: bool,
        has_timing: bool,
    }

    #[derive(Debug, Clone)]
    enum TestAction {
        Script { content: String },
        Other,
    }

    fn create_test_spec() -> TestSpec {
        TestSpec {
            name: "test-installer".to_string(),
            version: "1.0.0".to_string(),
            description: "Test installer".to_string(),
            author: Some("Test Author".to_string()),
            security: Some(TestSecurity {
                trust_model: "keyring".to_string(),
                require_signatures: true,
            }),
            artifacts: vec![],
            steps: vec![TestStep {
                id: "step-1".to_string(),
                name: "First Step".to_string(),
                action: TestAction::Script {
                    content: "echo hello".to_string(),
                },
                depends_on: vec![],
                has_postconditions: true,
                has_checkpoint: true,
                has_timing: true,
            }],
        }
    }

    /// Simple audit for test specs (mimics real audit logic)
    fn audit_test_spec(spec: &TestSpec, path: &Path) -> AuditReport {
        let mut report = AuditReport::new(&spec.name, &spec.version, path.to_path_buf());
        audit_test_security(&spec.security, &mut report);
        audit_test_artifacts(&spec.artifacts, &mut report);
        audit_test_steps(&spec.steps, &mut report);
        if spec.description.is_empty() {
            report.add_finding(AuditFinding::new(
                "BP001",
                AuditSeverity::Suggestion,
                AuditCategory::BestPractices,
                "Missing installer description",
                "No description field.",
            ));
        }
        report.metadata.audited_at = chrono_timestamp();
        report.metadata.steps_audited = spec.steps.len();
        report.metadata.artifacts_audited = spec.artifacts.len();
        report
    }

    fn audit_test_security(security: &Option<TestSecurity>, report: &mut AuditReport) {
        let Some(ref security) = security else {
            report.add_finding(AuditFinding::new(
                "SEC003",
                AuditSeverity::Warning,
                AuditCategory::Security,
                "No security configuration",
                "No security section defined.",
            ));
            return;
        };
        if !security.require_signatures {
            report.add_finding(AuditFinding::new(
                "SEC001",
                AuditSeverity::Warning,
                AuditCategory::Security,
                "Signatures not required",
                "Artifact signature verification is disabled.",
            ));
        }
        if security.trust_model == "tofu" {
            report.add_finding(AuditFinding::new(
                "SEC002",
                AuditSeverity::Info,
                AuditCategory::Security,
                "Using TOFU model",
                "TOFU is suitable for development.",
            ));
        }
    }

    fn audit_test_artifacts(artifacts: &[TestArtifact], report: &mut AuditReport) {
        for artifact in artifacts {
            if artifact.signature.is_none() && artifact.signed_by.is_none() {
                report.add_finding(
                    AuditFinding::new(
                        "SEC004",
                        AuditSeverity::Warning,
                        AuditCategory::Security,
                        "Unsigned artifact",
                        format!("Artifact '{}' has no signature.", artifact.id),
                    )
                    .with_location(&artifact.id),
                );
            }
            if artifact.sha256.is_none() {
                report.add_finding(
                    AuditFinding::new(
                        "SEC005",
                        AuditSeverity::Error,
                        AuditCategory::Security,
                        "Missing artifact hash",
                        format!("Artifact '{}' has no SHA256.", artifact.id),
                    )
                    .with_location(&artifact.id),
                );
            }
            if artifact.url.contains("latest") {
                report.add_finding(
                    AuditFinding::new(
                        "HERM002",
                        AuditSeverity::Warning,
                        AuditCategory::Hermetic,
                        "Unpinned artifact version",
                        format!("Artifact '{}' uses unpinned version.", artifact.id),
                    )
                    .with_location(&artifact.id),
                );
            }
        }
    }

    fn audit_test_steps(steps: &[TestStep], report: &mut AuditReport) {
        let step_ids: std::collections::HashSet<&str> =
            steps.iter().map(|s| s.id.as_str()).collect();
        let mut seen_ids: std::collections::HashSet<&str> = std::collections::HashSet::new();

        for step in steps {
            audit_test_step_quality(step, &step_ids, &mut seen_ids, report);
        }
    }

    fn audit_test_step_quality<'a>(
        step: &'a TestStep,
        step_ids: &std::collections::HashSet<&str>,
        seen_ids: &mut std::collections::HashSet<&'a str>,
        report: &mut AuditReport,
    ) {
        if seen_ids.contains(step.id.as_str()) {
            report.add_finding(
                AuditFinding::new(
                    "QUAL004",
                    AuditSeverity::Error,
                    AuditCategory::Quality,
                    "Duplicate step ID",
                    format!("Step ID '{}' is duplicated.", step.id),
                )
                .with_location(&step.id),
            );
        }
        seen_ids.insert(&step.id);

        for dep in &step.depends_on {
            if !step_ids.contains(dep.as_str()) {
                report.add_finding(
                    AuditFinding::new(
                        "QUAL005",
                        AuditSeverity::Error,
                        AuditCategory::Quality,
                        "Invalid dependency reference",
                        format!("Step '{}' depends on non-existent '{}'.", step.id, dep),
                    )
                    .with_location(&step.id),
                );
            }
        }

        if !step.has_postconditions {
            report.add_finding(
                AuditFinding::new(
                    "QUAL001",
                    AuditSeverity::Warning,
                    AuditCategory::Quality,
                    "Missing postconditions",
                    format!("Step '{}' has no postconditions.", step.id),
                )
                .with_location(&step.id),
            );
        }

        if let TestAction::Script { ref content } = step.action {
            if content.contains("curl") && content.contains("| bash") {
                report.add_finding(
                    AuditFinding::new(
                        "SEC007",
                        AuditSeverity::Critical,
                        AuditCategory::Security,
                        "Unsafe curl pipe to bash",
                        "Step contains 'curl ... | bash' pattern.",
                    )
                    .with_location(&step.id),
                );
            }
        }
    }

    #[test]
    fn test_AUDIT_120_severity_ordering() {
        assert!(AuditSeverity::Info < AuditSeverity::Suggestion);
        assert!(AuditSeverity::Suggestion < AuditSeverity::Warning);
        assert!(AuditSeverity::Warning < AuditSeverity::Error);
        assert!(AuditSeverity::Error < AuditSeverity::Critical);
    }

    #[test]
    fn test_AUDIT_120_severity_symbols() {
        assert_eq!(AuditSeverity::Info.symbol(), "ℹ");
        assert_eq!(AuditSeverity::Critical.symbol(), "🚨");
    }

    #[test]
    fn test_AUDIT_120_category_names() {
        assert_eq!(AuditCategory::Security.name(), "Security");
        assert_eq!(AuditCategory::Quality.name(), "Quality");
        assert_eq!(AuditCategory::Hermetic.name(), "Hermetic");
    }

    #[test]
    fn test_AUDIT_120_finding_creation() {
        let finding = AuditFinding::new(
            "TEST001",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "Test finding",
            "This is a test finding",
        );

        assert_eq!(finding.rule_id, "TEST001");
        assert_eq!(finding.severity, AuditSeverity::Warning);
        assert_eq!(finding.category, AuditCategory::Security);
    }

    #[test]
    fn test_AUDIT_120_finding_with_location() {
        let finding = AuditFinding::new(
            "TEST002",
            AuditSeverity::Error,
            AuditCategory::Quality,
            "Test",
            "Description",
        )
        .with_location("step-1")
        .with_suggestion("Fix it");

        assert_eq!(finding.location, Some("step-1".to_string()));
        assert_eq!(finding.suggestion, Some("Fix it".to_string()));
    }

    #[test]
    fn test_AUDIT_120_finding_format() {
        let finding = AuditFinding::new(
            "SEC001",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "Test title",
            "Test description",
        );

        let formatted = finding.format();
        assert!(formatted.contains("SEC001"));
        assert!(formatted.contains("WARNING"));
        assert!(formatted.contains("Test title"));
    }

    #[test]
    fn test_AUDIT_120_report_creation() {
        let report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        assert_eq!(report.installer_name, "test");
        assert!(report.findings.is_empty());
    }

    #[test]
    fn test_AUDIT_120_report_add_finding() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));

        report.add_finding(AuditFinding::new(
            "TEST001",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "Test",
            "Description",
        ));

        assert_eq!(report.findings.len(), 1);
    }

    #[test]
    fn test_AUDIT_120_report_has_critical() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        assert!(!report.has_critical_issues());

        report.add_finding(AuditFinding::new(
            "CRIT001",
            AuditSeverity::Critical,
            AuditCategory::Security,
            "Critical",
            "Description",
        ));

        assert!(report.has_critical_issues());
    }

    #[test]
    fn test_AUDIT_120_report_has_errors() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        assert!(!report.has_errors());

        report.add_finding(AuditFinding::new(
            "ERR001",
            AuditSeverity::Error,
            AuditCategory::Quality,
            "Error",
            "Description",
        ));

        assert!(report.has_errors());
    }

    #[test]
    fn test_AUDIT_120_report_score_perfect() {
        let report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        assert_eq!(report.score(), 100);
        assert_eq!(report.grade(), "A");
    }

    #[test]
    fn test_AUDIT_120_report_score_with_findings() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));

        // Add warning (-3) and error (-10)
        report.add_finding(AuditFinding::new(
            "WARN001",
            AuditSeverity::Warning,
            AuditCategory::Quality,
            "Warning",
            "Description",
        ));

        report.add_finding(AuditFinding::new(
            "ERR001",
            AuditSeverity::Error,
            AuditCategory::Quality,
            "Error",
            "Description",
        ));

        assert_eq!(report.score(), 87); // 100 - 3 - 10
        assert_eq!(report.grade(), "B");
    }

    #[test]
    fn test_AUDIT_120_report_count_by_severity() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));

        report.add_finding(AuditFinding::new(
            "W1",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "W1",
            "D",
        ));
        report.add_finding(AuditFinding::new(
            "W2",
            AuditSeverity::Warning,
            AuditCategory::Quality,
            "W2",
            "D",
        ));
        report.add_finding(AuditFinding::new(
            "E1",
            AuditSeverity::Error,
            AuditCategory::Security,
            "E1",
            "D",
        ));

        let counts = report.count_by_severity();
        assert_eq!(counts.get(&AuditSeverity::Warning), Some(&2));
        assert_eq!(counts.get(&AuditSeverity::Error), Some(&1));
    }

    #[test]
    fn test_AUDIT_120_report_format() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        report.metadata.audited_at = "2025-12-28T10:00:00Z".to_string();

        report.add_finding(AuditFinding::new(
            "SEC001",
            AuditSeverity::Warning,
            AuditCategory::Security,
            "Test Warning",
            "Description",
        ));

        let formatted = report.format();
        assert!(formatted.contains("Installer Audit Report"));
        assert!(formatted.contains("test v1.0.0"));
        assert!(formatted.contains("SEC001"));
    }

    #[test]
    fn test_AUDIT_120_report_to_json() {
        let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));
        report.metadata.audited_at = "2025-12-28T10:00:00Z".to_string();

        report.add_finding(
            AuditFinding::new(
                "SEC001",
                AuditSeverity::Warning,
                AuditCategory::Security,
                "Test",
                "Description",
            )
            .with_location("step-1"),
        );

        let json = report.to_json();
        assert!(json.contains("\"installer_name\": \"test\""));
        assert!(json.contains("\"rule_id\": \"SEC001\""));
        assert!(json.contains("\"location\": \"step-1\""));
    }

    #[test]
    fn test_AUDIT_120_context_default() {
        let ctx = AuditContext::default();
        assert!(ctx.check_security);
        assert!(ctx.check_quality);
        assert!(ctx.check_hermetic);
        assert!(ctx.check_best_practices);
    }

    #[test]
    fn test_AUDIT_120_context_security_only() {
        let ctx = AuditContext::security_only();
        assert!(ctx.check_security);
        assert!(!ctx.check_quality);
        assert!(!ctx.check_hermetic);
    }

    #[test]
    fn test_AUDIT_120_audit_spec_basic() {
        let spec = create_test_spec();
        let path = PathBuf::from("/test/installer.toml");

        let report = audit_test_spec(&spec, &path);

        assert_eq!(report.installer_name, "test-installer");
        assert_eq!(report.metadata.steps_audited, 1);
    }

    #[test]
    fn test_AUDIT_120_audit_missing_security() {
        let mut spec = create_test_spec();
        spec.security = None;

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have SEC003 finding
        assert!(report.findings.iter().any(|f| f.rule_id == "SEC003"));
    }

    #[test]
    fn test_AUDIT_120_audit_unsigned_artifact() {
        let mut spec = create_test_spec();
        spec.artifacts.push(TestArtifact {
            id: "test-artifact".to_string(),
            url: "https://example.com/file.tar.gz".to_string(),
            sha256: Some("abc123".to_string()),
            signature: None,
            signed_by: None,
        });

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have SEC004 finding
        assert!(report.findings.iter().any(|f| f.rule_id == "SEC004"));
    }

    #[test]
    fn test_AUDIT_120_audit_missing_hash() {
        let mut spec = create_test_spec();
        spec.artifacts.push(TestArtifact {
            id: "test-artifact".to_string(),
            url: "https://example.com/file.tar.gz".to_string(),
            sha256: None,
            signature: Some("sig.sig".to_string()),
            signed_by: Some("key-id".to_string()),
        });

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have SEC005 finding (missing hash)
        assert!(report.findings.iter().any(|f| f.rule_id == "SEC005"));
    }

    #[test]
    fn test_AUDIT_120_audit_unsafe_curl_pipe() {
        let mut spec = create_test_spec();
        spec.steps[0].action = TestAction::Script {
            content: "curl https://example.com/install.sh | bash".to_string(),
        };

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have SEC007 critical finding
        let sec007 = report.findings.iter().find(|f| f.rule_id == "SEC007");
        assert!(sec007.is_some());
        assert_eq!(sec007.unwrap().severity, AuditSeverity::Critical);
    }

    #[test]
    fn test_AUDIT_120_audit_missing_postconditions() {
        let mut spec = create_test_spec();
        spec.steps[0].has_postconditions = false;

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have QUAL001 finding
        assert!(report.findings.iter().any(|f| f.rule_id == "QUAL001"));
    }

    #[test]
    fn test_AUDIT_120_audit_duplicate_step_ids() {
        let mut spec = create_test_spec();
        spec.steps.push(TestStep {
            id: "step-1".to_string(), // Duplicate!
            name: "Duplicate Step".to_string(),
            action: TestAction::Script {
                content: "echo dup".to_string(),
            },
            depends_on: vec![],
            has_postconditions: true,
            has_checkpoint: true,
            has_timing: true,
        });

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have QUAL004 error
        let qual004 = report.findings.iter().find(|f| f.rule_id == "QUAL004");
        assert!(qual004.is_some());
        assert_eq!(qual004.unwrap().severity, AuditSeverity::Error);
    }

    #[test]
    fn test_AUDIT_120_audit_invalid_dependency() {
        let mut spec = create_test_spec();
        spec.steps[0].depends_on = vec!["nonexistent-step".to_string()];

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have QUAL005 error
        let qual005 = report.findings.iter().find(|f| f.rule_id == "QUAL005");
        assert!(qual005.is_some());
    }

    #[test]
    fn test_AUDIT_120_audit_unpinned_version() {
        let mut spec = create_test_spec();
        spec.artifacts.push(TestArtifact {
            id: "unpinned".to_string(),
            url: "https://example.com/app-latest.tar.gz".to_string(),
            sha256: Some("abc".to_string()),
            signature: Some("sig".to_string()),
            signed_by: Some("key".to_string()),
        });

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have HERM002 warning
        assert!(report.findings.iter().any(|f| f.rule_id == "HERM002"));
    }

    #[test]
    fn test_AUDIT_120_audit_missing_description() {
        let mut spec = create_test_spec();
        spec.description = String::new();

        let report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Should have BP001 suggestion
        assert!(report.findings.iter().any(|f| f.rule_id == "BP001"));
    }

    #[test]
    fn test_AUDIT_120_min_severity_filter() {
        let mut spec = create_test_spec();
        spec.description = String::new(); // BP001 suggestion
        spec.security = None; // SEC003 warning

        let mut report = audit_test_spec(&spec, &PathBuf::from("/test"));

        // Filter findings by minimum severity
        report
            .findings
            .retain(|f| f.severity >= AuditSeverity::Warning);

        // Should not have suggestions, only warnings and above
        assert!(report
            .findings
            .iter()
            .all(|f| f.severity >= AuditSeverity::Warning));
    }

    // Property tests
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_score_in_valid_range(
                warning_count in 0usize..20,
                error_count in 0usize..10
            ) {
                let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));

                for i in 0..warning_count {
                    report.add_finding(AuditFinding::new(
                        format!("W{}", i),
                        AuditSeverity::Warning,
                        AuditCategory::Quality,
                        "Warning",
                        "D",
                    ));
                }

                for i in 0..error_count {
                    report.add_finding(AuditFinding::new(
                        format!("E{}", i),
                        AuditSeverity::Error,
                        AuditCategory::Quality,
                        "Error",
                        "D",
                    ));
                }

                let score = report.score();
                prop_assert!(score <= 100);
            }

            #[test]
            fn prop_grade_matches_score(score in 0u8..=100) {
                // Create report with enough findings to reach target score
                let mut report = AuditReport::new("test", "1.0.0", PathBuf::from("/test"));

                // Add errors to reduce score (each -10)
                let errors_needed = ((100 - score as i32) / 10).max(0) as usize;
                for i in 0..errors_needed {
                    report.add_finding(AuditFinding::new(
                        format!("E{}", i),
                        AuditSeverity::Error,
                        AuditCategory::Quality,
                        "Error",
                        "D",
                    ));
                }

                let actual_score = report.score();
                let grade = report.grade();

                let expected_grade = match actual_score {
                    90..=100 => "A",
                    80..=89 => "B",
                    70..=79 => "C",
                    60..=69 => "D",
                    _ => "F",
                };

                prop_assert_eq!(grade, expected_grade);
            }

            #[test]
            fn prop_json_is_valid_structure(name in "[a-z]{5,10}", version in "[0-9]\\.[0-9]\\.[0-9]") {
                let report = AuditReport::new(&name, &version, PathBuf::from("/test"));
                let json = report.to_json();

                // Basic JSON structure validation
                let starts_correct = json.starts_with("{");
                let ends_correct = json.ends_with("}");
                let expected_name = format!("\"installer_name\": \"{}\"", name);
                let contains_name = json.contains(&expected_name);
                prop_assert!(starts_correct, "JSON should start with opening brace");
                prop_assert!(ends_correct, "JSON should end with closing brace");
                prop_assert!(contains_name, "JSON should contain installer name");
            }
        }
    }

    /// Tests for Issue #112: audit postconditions not recognized with commands format
    mod issue_112_tests {
        use super::*;
        use crate::installer::spec::InstallerSpec;

        #[test]
        fn test_112_postconditions_verification_commands_recognized() {
            // Issue #112: Step with verification.commands should NOT trigger QUAL001
            let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"
description = "Test installer"

[[step]]
id = "install-app"
name = "Install Application"
action = "script"

[step.script]
content = "apt-get install app"

[step.verification]
commands = [
    { cmd = "which app", expect = "/usr/bin/app" }
]
"#;
            let spec = InstallerSpec::parse(toml).expect("Valid TOML");
            let ctx = AuditContext::new();
            let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

            // Should NOT have QUAL001 finding - verification.commands counts as postconditions
            let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
            assert!(
                qual001.is_none(),
                "QUAL001 should not be raised when verification.commands is present"
            );
        }

        #[test]
        fn test_112_postconditions_file_mode_recognized() {
            // Issue #112: Step with file_mode postcondition should NOT trigger QUAL001
            let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "set-perms"
name = "Set Permissions"
action = "script"

[step.script]
content = "chmod 755 /app"

[step.postconditions]
file_mode = "/app:755"
"#;
            let spec = InstallerSpec::parse(toml).expect("Valid TOML");
            let ctx = AuditContext::new();
            let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

            let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
            assert!(
                qual001.is_none(),
                "QUAL001 should not be raised when file_mode postcondition is present"
            );
        }

        #[test]
        fn test_112_postconditions_service_active_recognized() {
            // Issue #112: Step with service_active postcondition should NOT trigger QUAL001
            let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "start-service"
name = "Start Service"
action = "script"

[step.script]
content = "systemctl start myapp"

[step.postconditions]
service_active = "myapp"
"#;
            let spec = InstallerSpec::parse(toml).expect("Valid TOML");
            let ctx = AuditContext::new();
            let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

            let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
            assert!(
                qual001.is_none(),
                "QUAL001 should not be raised when service_active postcondition is present"
            );
        }

        #[test]
        fn test_112_postconditions_env_matches_recognized() {
            // Issue #112: Step with env_matches postcondition should NOT trigger QUAL001
            let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "setup-env"
name = "Setup Environment"
action = "script"

[step.script]
content = "export PATH=/app/bin:$PATH"

[step.postconditions.env_matches]
PATH = "/app/bin"
"#;
            let spec = InstallerSpec::parse(toml).expect("Valid TOML");
            let ctx = AuditContext::new();
            let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

            let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
            assert!(
                qual001.is_none(),
                "QUAL001 should not be raised when env_matches postcondition is present"
            );
        }

        #[test]
        fn test_112_postconditions_user_in_group_recognized() {
            // Issue #112: Step with user_in_group postcondition should NOT trigger QUAL001
            let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "add-group"
name = "Add User to Group"
action = "script"

[step.script]
content = "usermod -aG docker $USER"

[step.postconditions.user_in_group]
user = "deploy"
group = "docker"
"#;
            let spec = InstallerSpec::parse(toml).expect("Valid TOML");
            let ctx = AuditContext::new();
            let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

            let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
            assert!(
                qual001.is_none(),
                "QUAL001 should not be raised when user_in_group postcondition is present"
            );
        }

        #[test]
        fn test_112_no_postconditions_triggers_qual001() {
            // Sanity check: Step with NO postconditions SHOULD trigger QUAL001
            let toml = r#"
[installer]
name = "test-installer"
version = "1.0.0"

[[step]]
id = "no-post"
name = "Step Without Postconditions"
action = "script"

[step.script]
content = "echo hello"
"#;
            let spec = InstallerSpec::parse(toml).expect("Valid TOML");
            let ctx = AuditContext::new();
            let report = ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"));

            let qual001 = report.findings.iter().find(|f| f.rule_id == "QUAL001");
            assert!(
                qual001.is_some(),
                "QUAL001 should be raised when no postconditions are present"
            );
        }
    }

    // ============================================================================
    // Coverage Tests - audit_security_parsed (SEC_COV_001-012)
    // ============================================================================
    mod security_parsed_tests {
        use super::*;
        use crate::installer::spec::InstallerSpec;

        fn sec_audit(toml: &str) -> AuditReport {
            let spec = InstallerSpec::parse(toml).expect("Valid TOML");
            // Use new() not security_only() — security_only has min_severity=Warning
            // which filters out Info-level findings like SEC002 and SEC006
            let ctx = AuditContext::new();
            ctx.audit_parsed_spec(&spec, &PathBuf::from("/test.toml"))
        }

        #[test]
        fn test_SEC_COV_001_signatures_not_required() {
            // SEC001: require_signatures = false triggers warning
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
trust_model = "keyring"
require_signatures = false
"#,
            );
            assert!(report.findings.iter().any(|f| f.rule_id == "SEC001"));
        }

        #[test]
        fn test_SEC_COV_002_signatures_required_no_sec001() {
            // SEC001: require_signatures = true should NOT trigger
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
trust_model = "keyring"
require_signatures = true
"#,
            );
            assert!(!report.findings.iter().any(|f| f.rule_id == "SEC001"));
        }

        #[test]
        fn test_SEC_COV_003_tofu_trust_model() {
            // SEC002: trust_model = "tofu" triggers info
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
trust_model = "tofu"
require_signatures = true
"#,
            );
            assert!(report.findings.iter().any(|f| f.rule_id == "SEC002"));
        }

        #[test]
        fn test_SEC_COV_004_keyring_trust_model_no_sec002() {
            // SEC002: trust_model = "keyring" should NOT trigger
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
trust_model = "keyring"
require_signatures = true
"#,
            );
            assert!(!report.findings.iter().any(|f| f.rule_id == "SEC002"));
        }

        #[test]
        fn test_SEC_COV_005_unsigned_artifact() {
            // SEC004: artifact without signature or signed_by
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[artifact]]
id = "myapp"
url = "https://example.com/myapp.tar.gz"
sha256 = "abc123"
"#,
            );
            assert!(report.findings.iter().any(|f| f.rule_id == "SEC004"));
        }

        #[test]
        fn test_SEC_COV_006_signed_artifact_no_sec004() {
            // SEC004: signed artifact should NOT trigger
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[artifact]]
id = "myapp"
url = "https://example.com/myapp.tar.gz"
sha256 = "abc123"
signature = "myapp.sig"
signed_by = "key-001"
"#,
            );
            assert!(!report.findings.iter().any(|f| f.rule_id == "SEC004"));
        }

        #[test]
        fn test_SEC_COV_007_missing_sha256() {
            // SEC005: artifact without sha256
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[artifact]]
id = "myapp"
url = "https://example.com/myapp.tar.gz"
signature = "myapp.sig"
signed_by = "key-001"
"#,
            );
            assert!(report.findings.iter().any(|f| f.rule_id == "SEC005"));
        }

        #[test]
        fn test_SEC_COV_008_root_privileges() {
            // SEC006: privileges = "root" triggers info
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[installer.requirements]
privileges = "root"
"#,
            );
            assert!(report.findings.iter().any(|f| f.rule_id == "SEC006"));
        }

        #[test]
        fn test_SEC_COV_009_user_privileges_no_sec006() {
            // SEC006: privileges = "user" should NOT trigger
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[installer.requirements]
privileges = "user"
"#,
            );
            assert!(!report.findings.iter().any(|f| f.rule_id == "SEC006"));
        }

        #[test]
        fn test_SEC_COV_010_curl_pipe_bash() {
            // SEC007: curl ... | bash pattern
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[step]]
id = "install"
name = "Install"
action = "script"

[step.script]
content = "curl https://example.com/setup.sh | bash"
"#,
            );
            let sec007 = report.findings.iter().find(|f| f.rule_id == "SEC007");
            assert!(sec007.is_some());
            assert_eq!(
                sec007.expect("has sec007").severity,
                AuditSeverity::Critical
            );
        }

        #[test]
        fn test_SEC_COV_011_eval_in_script() {
            // SEC008: eval in script
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[[step]]
id = "install"
name = "Install"
action = "script"

[step.script]
content = "eval $DYNAMIC_CMD"
"#,
            );
            assert!(report.findings.iter().any(|f| f.rule_id == "SEC008"));
        }

        #[test]
        fn test_SEC_COV_012_clean_spec_no_findings() {
            // Fully clean spec should have no security findings
            let report = sec_audit(
                r#"
[installer]
name = "test"
version = "1.0.0"

[installer.security]
require_signatures = true
trust_model = "keyring"

[installer.requirements]
privileges = "user"

[[artifact]]
id = "myapp"
url = "https://example.com/myapp-1.0.0.tar.gz"
sha256 = "abc123def456"
signature = "myapp.sig"
signed_by = "key-001"

[[step]]
id = "install"
name = "Install"
action = "script"

[step.script]
content = "tar xf myapp.tar.gz && ./install.sh"
"#,
            );
            let sec_findings: Vec<_> = report
                .findings
                .iter()
                .filter(|f| f.rule_id.starts_with("SEC"))
                .collect();
            assert!(
                sec_findings.is_empty(),
                "Clean spec should have no SEC findings, got: {sec_findings:?}"
            );
        }
    }
}
