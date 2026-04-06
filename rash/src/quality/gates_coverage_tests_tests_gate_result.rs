#[cfg(test)]
mod tests {
    use crate::quality::gates::*;
    use std::collections::HashMap;
    use std::time::Duration;

    // ── GateResult construction and field access ──

    #[test]
    fn test_gate_result_fields_populated() {
        let mut metrics = HashMap::new();
        metrics.insert("coverage".to_string(), 92.5);
        metrics.insert("violations".to_string(), 3.0);

        let violations = vec![
            GateViolation {
                file: Some("src/main.rs".to_string()),
                line: Some(10),
                description: "unused import".to_string(),
                severity: ViolationSeverity::Warning,
            },
            GateViolation {
                file: Some("src/lib.rs".to_string()),
                line: Some(55),
                description: "unreachable code".to_string(),
                severity: ViolationSeverity::Error,
            },
            GateViolation {
                file: None,
                line: None,
                description: "general note".to_string(),
                severity: ViolationSeverity::Info,
            },
        ];

        let result = GateResult {
            gate_name: "custom_gate".to_string(),
            passed: false,
            duration: Duration::from_millis(1234),
            message: "3 issues found".to_string(),
            metrics: metrics.clone(),
            violations: violations.clone(),
        };

        assert_eq!(result.gate_name, "custom_gate");
        assert!(!result.passed);
        assert_eq!(result.duration, Duration::from_millis(1234));
        assert_eq!(result.message, "3 issues found");
        assert_eq!(result.violations.len(), 3);
        assert_eq!(result.metrics.get("coverage"), Some(&92.5));
        assert_eq!(result.metrics.get("violations"), Some(&3.0));
    }

    #[test]
    fn test_gate_result_serialization_roundtrip() {
        let result = GateResult {
            gate_name: "clippy".to_string(),
            passed: true,
            duration: Duration::from_millis(500),
            message: "All clear".to_string(),
            metrics: HashMap::new(),
            violations: vec![],
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: GateResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.gate_name, "clippy");
        assert!(deserialized.passed);
        assert_eq!(deserialized.message, "All clear");
    }

    #[test]
    fn test_gate_violation_serialization_roundtrip() {
        let v = GateViolation {
            file: Some("test.rs".to_string()),
            line: Some(99),
            description: "test issue".to_string(),
            severity: ViolationSeverity::Error,
        };
        let json = serde_json::to_string(&v).expect("serialize");
        let deserialized: GateViolation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.file, Some("test.rs".to_string()));
        assert_eq!(deserialized.line, Some(99));
        assert_eq!(deserialized.severity, ViolationSeverity::Error);
    }

    #[test]
    fn test_violation_severity_serialization() {
        let err_json = serde_json::to_string(&ViolationSeverity::Error).unwrap();
        let warn_json = serde_json::to_string(&ViolationSeverity::Warning).unwrap();
        let info_json = serde_json::to_string(&ViolationSeverity::Info).unwrap();

        let err: ViolationSeverity = serde_json::from_str(&err_json).unwrap();
        let warn: ViolationSeverity = serde_json::from_str(&warn_json).unwrap();
        let info: ViolationSeverity = serde_json::from_str(&info_json).unwrap();

        assert_eq!(err, ViolationSeverity::Error);
        assert_eq!(warn, ViolationSeverity::Warning);
        assert_eq!(info, ViolationSeverity::Info);
    }

    // ── GateConfig TOML serialization ──

    #[test]
    fn test_gate_config_default_serializes_to_toml() {
        let config = GateConfig::default();
        let toml_str = toml::to_string(&config).expect("serialize to TOML");
        assert!(toml_str.contains("version"));
        assert!(toml_str.contains("bashrs"));
    }

    #[test]
    fn test_gate_config_roundtrip_toml() {
        let original = GateConfig {
            metadata: MetadataConfig {
                version: "3.0.0".to_string(),
                tool: "custom".to_string(),
            },
            gates: GatesConfig {
                run_clippy: false,
                clippy_strict: false,
                run_tests: false,
                test_timeout: 120,
                check_coverage: false,
                min_coverage: 99.9,
                check_complexity: false,
                max_complexity: 5,
                satd: SatdConfig {
                    enabled: false,
                    max_count: 50,
                    patterns: vec!["REVISIT".to_string()],
                    require_issue_links: false,
                    fail_on_violation: false,
                },
                mutation: MutationConfig {
                    enabled: true,
                    min_score: 95.0,
                    tool: "custom-mutants".to_string(),
                    strategy: "full".to_string(),
                },
                security: SecurityConfig {
                    enabled: false,
                    audit_vulnerabilities: "warn".to_string(),
                    audit_unmaintained: "deny".to_string(),
                    max_unsafe_blocks: 10,
                    fail_on_violation: false,
                },
            },
            tiers: TiersConfig {
                tier1_gates: vec!["fmt".to_string()],
                tier2_gates: vec!["fmt".to_string(), "clippy".to_string()],
                tier3_gates: vec!["all".to_string()],
            },
            risk_based: RiskBasedConfig {
                very_high_risk_mutation_target: 99.0,
                very_high_risk_components: vec!["core".to_string()],
                high_risk_mutation_target: 95.0,
                high_risk_components: vec!["parser".to_string()],
            },
        };

        let toml_str = toml::to_string(&original).expect("serialize");
        let parsed: GateConfig = toml::from_str(&toml_str).expect("deserialize");

        assert_eq!(parsed.metadata.version, "3.0.0");
        assert_eq!(parsed.metadata.tool, "custom");
        assert!(!parsed.gates.run_clippy);
        assert_eq!(parsed.gates.min_coverage, 99.9);
        assert_eq!(parsed.gates.satd.max_count, 50);
        assert_eq!(parsed.gates.mutation.tool, "custom-mutants");
        assert_eq!(parsed.tiers.tier1_gates, vec!["fmt"]);
        assert_eq!(parsed.risk_based.very_high_risk_mutation_target, 99.0);
    }

    // ── Tier enum behavior ──

    #[test]
    fn test_tier_edge_cases() {
        // from(0) falls to catch-all Tier3
        assert_eq!(Tier::from(0), Tier::Tier3);

        // Copy semantics
        let t = Tier::Tier2;
        let t2 = t;
        assert_eq!(t, t2);

        // Serde roundtrip
        for tier in &[Tier::Tier1, Tier::Tier2, Tier::Tier3] {
            let json = serde_json::to_string(tier).unwrap();
            let back: Tier = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, tier);
        }

        // Display
        assert_eq!(format!("{}", Tier::Tier1), "Tier 1 (ON-SAVE)");
        assert_eq!(format!("{}", Tier::Tier2), "Tier 2 (ON-COMMIT)");
        assert_eq!(format!("{}", Tier::Tier3), "Tier 3 (NIGHTLY)");
    }

    // ── QualityGate: summary and all_passed edge cases ──

    #[test]
    fn test_summary_all_passing_and_mixed() {
        // All passing
        let all_pass: Vec<GateResult> = (0..5)
            .map(|i| GateResult {
                gate_name: format!("gate_{}", i),
                passed: true,
                duration: Duration::from_millis(50 * (i + 1) as u64),
                message: "OK".to_string(),
                metrics: HashMap::new(),
                violations: vec![],
            })
            .collect();
        let summary = QualityGate::summary(&all_pass);
        assert_eq!(summary.total, 5);
        assert_eq!(summary.passed, 5);
        assert_eq!(summary.failed, 0);
        assert!(QualityGate::all_passed(&all_pass));

        // Mixed results
        let mixed = vec![
            GateResult {
                gate_name: "a".to_string(),
                passed: true,
                duration: Duration::from_millis(10),
                message: String::new(),
                metrics: HashMap::new(),
                violations: vec![],
            },
            GateResult {
                gate_name: "b".to_string(),
                passed: false,
                duration: Duration::from_millis(20),
                message: String::new(),
                metrics: HashMap::new(),
                violations: vec![],
            },
            GateResult {
                gate_name: "c".to_string(),
                passed: true,
                duration: Duration::from_millis(30),
                message: String::new(),
                metrics: HashMap::new(),
                violations: vec![],
            },
        ];
        let summary = QualityGate::summary(&mixed);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.total_duration, Duration::from_millis(60));
        assert!(!QualityGate::all_passed(&mixed));

        // Single failure with violations
        let single_fail = vec![GateResult {
            gate_name: "only".to_string(),
            passed: false,
            duration: Duration::from_secs(10),
            message: "Failed hard".to_string(),
            metrics: HashMap::new(),
            violations: vec![GateViolation {
                file: None,
                line: None,
                description: "critical".to_string(),
                severity: ViolationSeverity::Error,
            }],
        }];
        let summary = QualityGate::summary(&single_fail);
        assert_eq!(summary.total, 1);
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.failed, 1);
    }

    // ── MetricsConfig budget checking ──
}

#[cfg(test)]
mod gates_coverage_tests_tests_extracted_check {
    use super::*;
        include!("gates_coverage_tests_tests_extracted_check.rs");
}
