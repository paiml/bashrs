
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

}

        include!("audit_tests_extracted_AUDIT_AUDIT_112.rs");
