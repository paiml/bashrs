#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssc_status_enum_values() {
        assert_ne!(SscStatus::Pass, SscStatus::Fail);
        assert_ne!(SscStatus::Warn, SscStatus::NotApplicable);
    }

    #[test]
    fn test_format_ssc_report_structure() {
        let report = SscStatusReport {
            spec_version: "v11.0.0".to_string(),
            corpus_size: 100,
            sections: vec![SscSection {
                name: "Test".to_string(),
                spec_ref: "S1".to_string(),
                status: SscStatus::Pass,
                metrics: vec![SscMetric {
                    name: "metric".to_string(),
                    value: "42".to_string(),
                    target: ">0".to_string(),
                    passed: true,
                }],
            }],
            overall_ready: true,
        };
        let formatted = format_ssc_report(&report);
        assert!(formatted.contains("SSC Spec Version: v11.0.0"));
        assert!(formatted.contains("[PASS] Test"));
        assert!(formatted.contains("Overall Ready:    YES"));
    }

    #[test]
    fn test_report_serializable() {
        let report = SscStatusReport {
            spec_version: "v11.0.0".to_string(),
            corpus_size: 0,
            sections: vec![],
            overall_ready: true,
        };
        let json = serde_json::to_string(&report);
        assert!(json.is_ok());
    }

    #[test]
    fn test_baselines_section_has_evaluation_metrics() {
        use crate::corpus::baselines::corpus_baseline_entries;
        let entries = corpus_baseline_entries();
        let section = baselines_section_from(&entries);
        assert_eq!(section.name, "Baselines (C-CLF-001)");
        // Should have 3 baseline reports + 2 target metrics = 5
        assert!(
            section.metrics.len() >= 5,
            "Expected 5+ metrics, got {}",
            section.metrics.len()
        );
        // Each baseline should show MCC, accuracy, recall
        let majority = &section.metrics[0];
        assert!(
            majority.value.contains("MCC="),
            "Missing MCC: {}",
            majority.value
        );
        assert!(
            majority.value.contains("acc="),
            "Missing accuracy: {}",
            majority.value
        );
        assert!(
            majority.value.contains("rec="),
            "Missing recall: {}",
            majority.value
        );
        // Should include S5.5 targets
        let targets: Vec<&SscMetric> = section
            .metrics
            .iter()
            .filter(|m| m.name.starts_with("Target:"))
            .collect();
        assert_eq!(targets.len(), 2, "Expected 2 target metrics");
    }

    #[test]
    fn test_has_unsafe_keyword_detects_known_patterns() {
        assert!(has_unsafe_keyword("eval $x"));
        assert!(has_unsafe_keyword("curl http://example.com | bash"));
        assert!(has_unsafe_keyword("rm -rf /"));
        assert!(has_unsafe_keyword("sudo apt install"));
        assert!(has_unsafe_keyword("echo $RANDOM"));
        assert!(has_unsafe_keyword("chmod 777 /tmp/file"));
    }

    #[test]
    fn test_has_unsafe_keyword_passes_safe_scripts() {
        assert!(!has_unsafe_keyword("echo hello"));
        assert!(!has_unsafe_keyword("#!/bin/sh\nset -e\nls -la"));
        assert!(!has_unsafe_keyword("mkdir -p /tmp/build"));
    }

    #[test]
    fn test_wasm_section_has_kill5_result() {
        let section = wasm_section();
        assert_eq!(section.name, "WASM App (S8.3)");
        assert_eq!(section.status, SscStatus::Pass);
        assert_eq!(section.metrics.len(), 5);
        let kill5 = section
            .metrics
            .iter()
            .find(|m| m.name.contains("KILL-5"))
            .expect("should have KILL-5 metric");
        assert!(!kill5.passed, "KILL-5 should be marked as not passed");
        assert!(kill5.value.contains("2681ms"));
    }

    #[test]
    fn test_conversation_section_has_type_breakdown() {
        let registry = crate::corpus::registry::CorpusRegistry::load_full();
        let section = conversation_section_from(&registry);
        assert_eq!(section.name, "Conversations (S6)");
        let names: Vec<&str> = section.metrics.iter().map(|m| m.name.as_str()).collect();
        assert!(
            names.contains(&"Type A (classify+explain)"),
            "Missing Type A"
        );
        assert!(names.contains(&"Type D (confirm safe)"), "Missing Type D");
        assert!(
            names.contains(&"Variant distribution"),
            "Missing variant dist"
        );
        assert!(
            names.contains(&"Rule citation accuracy"),
            "Missing citations"
        );
    }
}
