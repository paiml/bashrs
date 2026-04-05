
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

include!("audit_tests_extracted_AUDIT_AUDIT.rs");
