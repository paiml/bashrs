#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let toml = r#"
            [gates]
            run_clippy = true
        "#;
        let config: GateConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.gates.test_timeout, 300);
        assert_eq!(config.gates.min_coverage, 80.0);
    }

    #[test]
    fn test_validation_error() {
        let toml = r#"
            [gates]
            min_coverage = 150.0
        "#;
        let config: Result<GateConfig> = toml::from_str::<GateConfig>(toml)
            .map_err(|e| Error::Internal(e.to_string()))
            .and_then(|c| {
                if c.gates.min_coverage > 100.0 {
                    Err(Error::Validation("Coverage too high".into()))
                } else {
                    Ok(c)
                }
            });

        assert!(config.is_err());
    }

    #[test]
    fn test_default_test_timeout() {
        assert_eq!(default_test_timeout(), 300);
    }

    #[test]
    fn test_default_min_coverage() {
        assert_eq!(default_min_coverage(), 80.0);
    }

    #[test]
    fn test_default_max_complexity() {
        assert_eq!(default_max_complexity(), 10);
    }

    #[test]
    fn test_full_gate_config() {
        let toml = r#"
            [metadata]
            version = "1.0.0"
            tool = "bashrs"

            [gates]
            run_clippy = true
            clippy_strict = true
            run_tests = true
            test_timeout = 600
            check_coverage = true
            min_coverage = 85.0
            check_complexity = true
            max_complexity = 15

            [gates.satd]
            enabled = true
            max_count = 10
            patterns = ["TODO", "FIXME"]

            [gates.mutation]
            enabled = true
            min_score = 80.0

            [gates.security]
            enabled = true
            max_unsafe_blocks = 5

            [tiers]
            tier1_gates = ["clippy", "tests"]
            tier2_gates = ["coverage", "complexity"]
            tier3_gates = ["mutation", "security"]
        "#;
        let config: GateConfig = toml::from_str(toml).unwrap();

        // Check metadata
        assert!(config.metadata.is_some());
        let metadata = config.metadata.unwrap();
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.tool, "bashrs");

        // Check gates
        assert!(config.gates.run_clippy);
        assert!(config.gates.clippy_strict);
        assert!(config.gates.run_tests);
        assert_eq!(config.gates.test_timeout, 600);
        assert!(config.gates.check_coverage);
        assert_eq!(config.gates.min_coverage, 85.0);
        assert!(config.gates.check_complexity);
        assert_eq!(config.gates.max_complexity, 15);

        // Check SATD gate
        let satd = config.gates.satd.unwrap();
        assert!(satd.enabled);
        assert_eq!(satd.max_count, 10);
        assert_eq!(satd.patterns, vec!["TODO", "FIXME"]);

        // Check mutation gate
        let mutation = config.gates.mutation.unwrap();
        assert!(mutation.enabled);
        assert_eq!(mutation.min_score, 80.0);

        // Check security gate
        let security = config.gates.security.unwrap();
        assert!(security.enabled);
        assert_eq!(security.max_unsafe_blocks, 5);

        // Check tiers
        assert_eq!(config.tiers.tier1_gates, vec!["clippy", "tests"]);
        assert_eq!(config.tiers.tier2_gates, vec!["coverage", "complexity"]);
        assert_eq!(config.tiers.tier3_gates, vec!["mutation", "security"]);
    }

    #[test]
    fn test_minimal_config() {
        let toml = r#"
            [gates]
        "#;
        let config: GateConfig = toml::from_str(toml).unwrap();

        // All booleans should default to false
        assert!(!config.gates.run_clippy);
        assert!(!config.gates.clippy_strict);
        assert!(!config.gates.run_tests);
        assert!(!config.gates.check_coverage);
        assert!(!config.gates.check_complexity);

        // Check defaults
        assert_eq!(config.gates.test_timeout, 300);
        assert_eq!(config.gates.min_coverage, 80.0);
        assert_eq!(config.gates.max_complexity, 10);

        // Optional fields should be None
        assert!(config.gates.satd.is_none());
        assert!(config.gates.mutation.is_none());
        assert!(config.gates.security.is_none());
        assert!(config.metadata.is_none());
    }

    #[test]
    fn test_satd_gate_defaults() {
        let gate = SatdGate::default();
        assert!(!gate.enabled);
        assert_eq!(gate.max_count, 0);
        assert!(gate.patterns.is_empty());
    }

    #[test]
    fn test_mutation_gate_defaults() {
        let gate = MutationGate::default();
        assert!(!gate.enabled);
        assert_eq!(gate.min_score, 0.0);
    }

    #[test]
    fn test_security_gate_defaults() {
        let gate = SecurityGate::default();
        assert!(!gate.enabled);
        assert_eq!(gate.max_unsafe_blocks, 0);
    }

    #[test]
    fn test_tiers_defaults() {
        let tiers = Tiers::default();
        assert!(tiers.tier1_gates.is_empty());
        assert!(tiers.tier2_gates.is_empty());
        assert!(tiers.tier3_gates.is_empty());
    }

    #[test]
    fn test_config_clone() {
        let toml = r#"
            [gates]
            run_clippy = true
        "#;
        let config: GateConfig = toml::from_str(toml).unwrap();
        let cloned = config.clone();
        assert_eq!(config.gates.run_clippy, cloned.gates.run_clippy);
    }

    #[test]
    fn test_negative_coverage_handling() {
        let toml = r#"
            [gates]
            min_coverage = -10.0
        "#;
        let config: GateConfig = toml::from_str(toml).unwrap();
        // The validation happens in load(), not parse
        // So we test that the value is parsed correctly
        assert_eq!(config.gates.min_coverage, -10.0);
    }
}

#[cfg(test)]
mod gates_tests_extracted_gates {
    use super::*;
}

include!("gates_tests_extracted_gates.rs");
