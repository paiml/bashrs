
    #[test]
    fn test_gates_debug_format() {
        let gates = Gates {
            run_clippy: true,
            clippy_strict: false,
            run_tests: true,
            test_timeout: 300,
            check_coverage: false,
            min_coverage: 80.0,
            check_complexity: false,
            max_complexity: 10,
            satd: None,
            mutation: None,
            security: None,
        };
        let debug_str = format!("{:?}", gates);
        assert!(debug_str.contains("run_clippy: true"));
    }

    #[test]
    fn test_metadata_debug_format() {
        let metadata = Metadata {
            version: "1.0".to_string(),
            tool: "test".to_string(),
        };
        let debug_str = format!("{:?}", metadata);
        assert!(debug_str.contains("version"));
        assert!(debug_str.contains("tool"));
    }

    #[test]
    fn test_satd_gate_clone() {
        let gate = SatdGate {
            enabled: true,
            max_count: 5,
            patterns: vec!["TODO".to_string(), "FIXME".to_string()],
        };
        let cloned = gate.clone();
        assert!(cloned.enabled);
        assert_eq!(cloned.max_count, 5);
        assert_eq!(cloned.patterns.len(), 2);
    }

    #[test]
    fn test_mutation_gate_clone() {
        let gate = MutationGate {
            enabled: true,
            min_score: 90.0,
        };
        let cloned = gate.clone();
        assert!(cloned.enabled);
        assert_eq!(cloned.min_score, 90.0);
    }

    #[test]
    fn test_security_gate_clone() {
        let gate = SecurityGate {
            enabled: true,
            max_unsafe_blocks: 3,
        };
        let cloned = gate.clone();
        assert!(cloned.enabled);
        assert_eq!(cloned.max_unsafe_blocks, 3);
    }

    #[test]
    fn test_tiers_clone() {
        let tiers = Tiers {
            tier1_gates: vec!["clippy".to_string()],
            tier2_gates: vec!["coverage".to_string()],
            tier3_gates: vec!["mutation".to_string()],
        };
        let cloned = tiers.clone();
        assert_eq!(cloned.tier1_gates, vec!["clippy"]);
        assert_eq!(cloned.tier2_gates, vec!["coverage"]);
        assert_eq!(cloned.tier3_gates, vec!["mutation"]);
    }

    #[test]
    fn test_metadata_clone() {
        let metadata = Metadata {
            version: "2.0".to_string(),
            tool: "bashrs".to_string(),
        };
        let cloned = metadata.clone();
        assert_eq!(cloned.version, "2.0");
        assert_eq!(cloned.tool, "bashrs");
    }

    #[test]
    fn test_gates_serialization() {
        let gates = Gates {
            run_clippy: true,
            clippy_strict: false,
            run_tests: true,
            test_timeout: 300,
            check_coverage: true,
            min_coverage: 85.0,
            check_complexity: true,
            max_complexity: 15,
            satd: None,
            mutation: None,
            security: None,
        };
        let serialized = toml::to_string(&gates).unwrap();
        assert!(serialized.contains("run_clippy = true"));
        assert!(serialized.contains("min_coverage = 85.0"));
    }

    #[test]
    fn test_gate_config_serialization() {
        let config = GateConfig {
            metadata: Some(Metadata {
                version: "1.0.0".to_string(),
                tool: "test".to_string(),
            }),
            gates: Gates {
                run_clippy: true,
                clippy_strict: false,
                run_tests: false,
                test_timeout: 300,
                check_coverage: false,
                min_coverage: 80.0,
                check_complexity: false,
                max_complexity: 10,
                satd: None,
                mutation: None,
                security: None,
            },
            tiers: Tiers::default(),
        };
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("[metadata]"));
        assert!(serialized.contains("[gates]"));
    }

    #[test]
    fn test_satd_gate_serialization() {
        let gate = SatdGate {
            enabled: true,
            max_count: 10,
            patterns: vec!["TODO".to_string(), "FIXME".to_string()],
        };
        let serialized = toml::to_string(&gate).unwrap();
        assert!(serialized.contains("enabled = true"));
        assert!(serialized.contains("max_count = 10"));
    }

    #[test]
    fn test_mutation_gate_serialization() {
        let gate = MutationGate {
            enabled: true,
            min_score: 80.0,
        };
        let serialized = toml::to_string(&gate).unwrap();
        assert!(serialized.contains("enabled = true"));
        assert!(serialized.contains("min_score = 80.0"));
    }

    #[test]
    fn test_security_gate_serialization() {
        let gate = SecurityGate {
            enabled: true,
            max_unsafe_blocks: 5,
        };
        let serialized = toml::to_string(&gate).unwrap();
        assert!(serialized.contains("enabled = true"));
        assert!(serialized.contains("max_unsafe_blocks = 5"));
    }

    #[test]
    fn test_tiers_serialization() {
        let tiers = Tiers {
            tier1_gates: vec!["clippy".to_string(), "tests".to_string()],
            tier2_gates: vec!["coverage".to_string()],
            tier3_gates: vec![],
        };
        let serialized = toml::to_string(&tiers).unwrap();
        assert!(serialized.contains("tier1_gates"));
        assert!(serialized.contains("clippy"));
    }

    #[test]
    fn test_gates_debug_all_fields() {
        let gates = Gates {
            run_clippy: true,
            clippy_strict: true,
            run_tests: true,
            test_timeout: 600,
            check_coverage: true,
            min_coverage: 90.0,
            check_complexity: true,
            max_complexity: 5,
            satd: Some(SatdGate::default()),
            mutation: Some(MutationGate::default()),
            security: Some(SecurityGate::default()),
        };
        let debug_str = format!("{:?}", gates);
        assert!(debug_str.contains("run_clippy"));
        assert!(debug_str.contains("clippy_strict"));
        assert!(debug_str.contains("run_tests"));
        assert!(debug_str.contains("satd"));
        assert!(debug_str.contains("mutation"));
        assert!(debug_str.contains("security"));
    }

    #[test]
    fn test_satd_gate_debug() {
        let gate = SatdGate {
            enabled: true,
            max_count: 5,
            patterns: vec!["TODO".to_string()],
        };
        let debug_str = format!("{:?}", gate);
        assert!(debug_str.contains("enabled"));
        assert!(debug_str.contains("max_count"));
        assert!(debug_str.contains("patterns"));
    }

    #[test]
    fn test_mutation_gate_debug() {
        let gate = MutationGate {
            enabled: false,
            min_score: 75.0,
        };
        let debug_str = format!("{:?}", gate);
        assert!(debug_str.contains("enabled"));
        assert!(debug_str.contains("min_score"));
    }

    #[test]
    fn test_security_gate_debug() {
        let gate = SecurityGate {
            enabled: true,
            max_unsafe_blocks: 2,
        };
        let debug_str = format!("{:?}", gate);
        assert!(debug_str.contains("enabled"));
        assert!(debug_str.contains("max_unsafe_blocks"));
    }

    #[test]
    fn test_tiers_debug() {
        let tiers = Tiers {
            tier1_gates: vec!["a".to_string()],
            tier2_gates: vec!["b".to_string()],
            tier3_gates: vec!["c".to_string()],
        };
        let debug_str = format!("{:?}", tiers);
        assert!(debug_str.contains("tier1_gates"));
        assert!(debug_str.contains("tier2_gates"));
        assert!(debug_str.contains("tier3_gates"));
    }

    #[test]
    fn test_gate_config_debug() {
        let config = GateConfig {
            metadata: None,
            gates: Gates {
                run_clippy: false,
                clippy_strict: false,
                run_tests: false,
                test_timeout: 300,
                check_coverage: false,
                min_coverage: 80.0,
                check_complexity: false,
                max_complexity: 10,
                satd: None,
                mutation: None,
                security: None,
            },
            tiers: Tiers::default(),
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("metadata"));
        assert!(debug_str.contains("gates"));
        assert!(debug_str.contains("tiers"));
    }

    #[test]
    fn test_config_with_all_optional_gates() {
        let toml = r#"
            [gates]
            run_clippy = true

            [gates.satd]
            enabled = false
            max_count = 0
            patterns = []

            [gates.mutation]
            enabled = false
            min_score = 0.0

            [gates.security]
            enabled = false
            max_unsafe_blocks = 0
        "#;
        let config: GateConfig = toml::from_str(toml).unwrap();
        assert!(config.gates.satd.is_some());
        assert!(config.gates.mutation.is_some());
        assert!(config.gates.security.is_some());

