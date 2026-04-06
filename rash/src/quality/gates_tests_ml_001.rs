#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_001_gate_config_default() {
        let config = GateConfig::default();

        assert_eq!(config.metadata.version, "1.0.0");
        assert_eq!(config.metadata.tool, "bashrs");
        assert!(config.gates.run_clippy);
        assert!(config.gates.clippy_strict);
        assert_eq!(config.gates.max_complexity, 10);
        assert_eq!(config.gates.min_coverage, 85.0);
    }

    #[test]
    fn test_ml_001_gate_config_parse() {
        let toml_content = r#"
[metadata]
version = "1.0.0"
tool = "bashrs"

[gates]
run_clippy = true
clippy_strict = true
min_coverage = 90.0
max_complexity = 8

[gates.satd]
enabled = true
max_count = 0
patterns = ["TODO", "FIXME"]

[tiers]
tier1_gates = ["clippy"]
tier2_gates = ["clippy", "tests"]
"#;

        let config: GateConfig = toml::from_str(toml_content).expect("valid toml");

        assert_eq!(config.gates.min_coverage, 90.0);
        assert_eq!(config.gates.max_complexity, 8);
        assert_eq!(config.gates.satd.patterns.len(), 2);
        assert_eq!(config.tiers.tier1_gates, vec!["clippy"]);
    }

    #[test]
    fn test_ml_001_tier_from_u8() {
        assert_eq!(Tier::from(1), Tier::Tier1);
        assert_eq!(Tier::from(2), Tier::Tier2);
        assert_eq!(Tier::from(3), Tier::Tier3);
        assert_eq!(Tier::from(99), Tier::Tier3); // Defaults to Tier3
    }

    #[test]
    fn test_ml_001_gates_for_tier() {
        let config = GateConfig::default();

        let tier1 = config.gates_for_tier(Tier::Tier1);
        assert!(tier1.contains(&"clippy".to_string()));
        assert!(tier1.contains(&"complexity".to_string()));

        let tier2 = config.gates_for_tier(Tier::Tier2);
        assert!(tier2.contains(&"tests".to_string()));
        assert!(tier2.contains(&"coverage".to_string()));

        let tier3 = config.gates_for_tier(Tier::Tier3);
        assert!(tier3.contains(&"mutation".to_string()));
        assert!(tier3.contains(&"security".to_string()));
    }

    #[test]
    fn test_ml_002_quality_gate_creation() {
        let gate = QualityGate::with_defaults();
        assert_eq!(gate.config.gates.max_complexity, 10);
    }

    #[test]
    fn test_ml_003_gate_summary() {
        let results = vec![
            GateResult {
                gate_name: "clippy".to_string(),
                passed: true,
                duration: Duration::from_millis(100),
                message: "OK".to_string(),
                metrics: HashMap::new(),
                violations: vec![],
            },
            GateResult {
                gate_name: "tests".to_string(),
                passed: false,
                duration: Duration::from_millis(200),
                message: "Failed".to_string(),
                metrics: HashMap::new(),
                violations: vec![],
            },
        ];

        let summary = QualityGate::summary(&results);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.total_duration, Duration::from_millis(300));
    }

    #[test]
    fn test_ml_003_all_passed() {
        let passing = vec![GateResult {
            gate_name: "test".to_string(),
            passed: true,
            duration: Duration::default(),
            message: String::new(),
            metrics: HashMap::new(),
            violations: vec![],
        }];

        let failing = vec![GateResult {
            gate_name: "test".to_string(),
            passed: false,
            duration: Duration::default(),
            message: String::new(),
            metrics: HashMap::new(),
            violations: vec![],
        }];

        assert!(QualityGate::all_passed(&passing));
        assert!(!QualityGate::all_passed(&failing));
    }

    // Additional tests for coverage

    #[test]
    fn test_tier_display() {
        assert_eq!(format!("{}", Tier::Tier1), "Tier 1 (ON-SAVE)");
        assert_eq!(format!("{}", Tier::Tier2), "Tier 2 (ON-COMMIT)");
        assert_eq!(format!("{}", Tier::Tier3), "Tier 3 (NIGHTLY)");
    }

    #[test]
    fn test_tier_ordering() {
        assert!(Tier::Tier1 < Tier::Tier2);
        assert!(Tier::Tier2 < Tier::Tier3);
        assert!(Tier::Tier1 < Tier::Tier3);
    }

    #[test]
    fn test_metadata_config_default() {
        let meta = MetadataConfig::default();
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.tool, "bashrs");
    }

    #[test]
    fn test_satd_config_default() {
        let satd = SatdConfig::default();
        assert!(satd.enabled);
        assert_eq!(satd.max_count, 0);
        assert!(satd.patterns.contains(&"TODO".to_string()));
        assert!(satd.patterns.contains(&"FIXME".to_string()));
        assert!(satd.patterns.contains(&"HACK".to_string()));
        assert!(satd.patterns.contains(&"XXX".to_string()));
        assert!(satd.require_issue_links);
        assert!(satd.fail_on_violation);
    }

    #[test]
    fn test_mutation_config_default() {
        let mutation = MutationConfig::default();
        assert!(!mutation.enabled);
        assert_eq!(mutation.min_score, 85.0);
        assert_eq!(mutation.tool, "cargo-mutants");
        assert_eq!(mutation.strategy, "incremental");
    }

    #[test]
    fn test_security_config_default() {
        let security = SecurityConfig::default();
        assert!(security.enabled);
        assert_eq!(security.audit_vulnerabilities, "deny");
        assert_eq!(security.audit_unmaintained, "warn");
        assert_eq!(security.max_unsafe_blocks, 0);
        assert!(security.fail_on_violation);
    }

    #[test]
    fn test_risk_based_config_default() {
        let risk = RiskBasedConfig::default();
        assert_eq!(risk.very_high_risk_mutation_target, 92.5);
        assert!(risk.very_high_risk_components.is_empty());
        assert_eq!(risk.high_risk_mutation_target, 87.5);
        assert!(risk.high_risk_components.is_empty());
    }

    #[test]
    fn test_tiers_config_default() {
        let tiers = TiersConfig::default();
        assert_eq!(tiers.tier1_gates, vec!["clippy", "complexity"]);
        assert_eq!(
            tiers.tier2_gates,
            vec!["clippy", "tests", "coverage", "satd"]
        );
        assert_eq!(
            tiers.tier3_gates,
            vec!["clippy", "tests", "coverage", "satd", "mutation", "security"]
        );
    }

    include!("gates_tests_extracted_gates.rs");
}
