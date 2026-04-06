
    #[test]
    fn test_gate_summary_debug() {
        let summary = GateSummary {
            total: 5,
            passed: 3,
            failed: 2,
            total_duration: Duration::from_millis(500),
        };
        let debug_output = format!("{:?}", summary);
        assert!(debug_output.contains("total: 5"));
        assert!(debug_output.contains("passed: 3"));
        assert!(debug_output.contains("failed: 2"));
    }

    #[test]
    fn test_default_helper_functions() {
        assert!(default_true());
        assert_eq!(default_timeout(), 300);
        assert_eq!(default_coverage(), 85.0);
        assert_eq!(default_complexity(), 10);
        assert_eq!(default_mutation_score(), 85.0);
        assert_eq!(default_mutation_tool(), "cargo-mutants");
        assert_eq!(default_mutation_strategy(), "incremental");
        assert_eq!(default_audit_vulnerabilities(), "deny");
        assert_eq!(default_audit_unmaintained(), "warn");
        assert_eq!(default_very_high_risk_target(), 92.5);
        assert_eq!(default_high_risk_target(), 87.5);
        assert_eq!(default_version(), "1.0.0");
        assert_eq!(default_tool(), "bashrs");
    }

    // ===== GATE EXECUTION TESTS =====

    #[test]
    fn test_run_gate_unknown() {
        let gate = QualityGate::with_defaults();
        let result = gate.run_gate("unknown_gate");
        assert!(!result.passed);
        assert!(result.message.contains("Unknown gate"));
        assert_eq!(result.gate_name, "unknown_gate");
    }

    #[test]
    fn test_run_clippy_gate_disabled() {
        let config = GateConfig {
            gates: GatesConfig {
                run_clippy: false,
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let result = gate.run_gate("clippy");
        assert!(result.passed);
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn test_run_complexity_gate_disabled() {
        let config = GateConfig {
            gates: GatesConfig {
                check_complexity: false,
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let result = gate.run_gate("complexity");
        assert!(result.passed);
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn test_run_tests_gate_disabled() {
        let config = GateConfig {
            gates: GatesConfig {
                run_tests: false,
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let result = gate.run_gate("tests");
        assert!(result.passed);
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn test_run_coverage_gate_disabled() {
        let config = GateConfig {
            gates: GatesConfig {
                check_coverage: false,
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let result = gate.run_gate("coverage");
        assert!(result.passed);
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn test_run_satd_gate_disabled() {
        let config = GateConfig {
            gates: GatesConfig {
                satd: SatdConfig {
                    enabled: false,
                    ..SatdConfig::default()
                },
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let result = gate.run_gate("satd");
        assert!(result.passed);
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn test_run_mutation_gate_disabled() {
        let config = GateConfig {
            gates: GatesConfig {
                mutation: MutationConfig {
                    enabled: false,
                    ..MutationConfig::default()
                },
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let result = gate.run_gate("mutation");
        assert!(result.passed);
        assert!(result.message.contains("disabled") || result.message.contains("Tier 3"));
    }

    #[test]
    fn test_run_security_gate_disabled() {
        let config = GateConfig {
            gates: GatesConfig {
                security: SecurityConfig {
                    enabled: false,
                    ..SecurityConfig::default()
                },
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let result = gate.run_gate("security");
        assert!(result.passed);
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn test_run_tier_with_disabled_gates() {
        let config = GateConfig {
            gates: GatesConfig {
                run_clippy: false,
                check_complexity: false,
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let results = gate.run_tier(Tier::Tier1);

        // Tier1 has clippy and complexity gates
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_run_coverage_gate_enabled() {
        let gate = QualityGate::with_defaults();
        let result = gate.run_gate("coverage");
        // Coverage gate returns placeholder message
        assert!(result.passed);
        assert!(result.message.contains("coverage") || result.message.contains("Coverage"));
        assert!(result.metrics.contains_key("target"));
    }

    #[test]
    fn test_run_mutation_gate_enabled() {
        let config = GateConfig {
            gates: GatesConfig {
                mutation: MutationConfig {
                    enabled: true,
                    min_score: 80.0,
                    ..MutationConfig::default()
                },
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let result = gate.run_gate("mutation");
        // Mutation gate returns placeholder when enabled
        assert!(result.passed);
        assert!(result.metrics.contains_key("target"));
    }

    #[test]
    fn test_gate_result_clone() {
        let result = GateResult {
            gate_name: "test".to_string(),
            passed: true,
            duration: Duration::from_millis(100),
            message: "OK".to_string(),
            metrics: HashMap::new(),
            violations: vec![],
        };
        let cloned = result.clone();
        assert_eq!(cloned.gate_name, "test");
        assert!(cloned.passed);
    }

    #[test]
    fn test_gate_violation_clone() {
        let violation = GateViolation {
            file: Some("test.rs".to_string()),
            line: Some(10),
            description: "desc".to_string(),
            severity: ViolationSeverity::Warning,
        };
        let cloned = violation.clone();
        assert_eq!(cloned.file, Some("test.rs".to_string()));
        assert_eq!(cloned.severity, ViolationSeverity::Warning);
    }

    #[test]
    fn test_gate_summary_clone() {
        let summary = GateSummary {
            total: 10,
            passed: 8,
            failed: 2,
            total_duration: Duration::from_secs(5),
        };
        let cloned = summary.clone();
        assert_eq!(cloned.total, 10);
        assert_eq!(cloned.passed, 8);
        assert_eq!(cloned.failed, 2);
    }

    #[test]
    fn test_tier_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Tier::Tier1);
        set.insert(Tier::Tier2);
        set.insert(Tier::Tier3);
        assert_eq!(set.len(), 3);
        assert!(set.contains(&Tier::Tier1));
    }

    #[test]
    fn test_gate_config_error_std_error() {
        let io_error = GateConfigError::Io {
            path: std::path::PathBuf::from("/test.toml"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        // Verify it implements std::error::Error
        let err: &dyn std::error::Error = &io_error;
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn test_satd_default_patterns() {
        let patterns = default_satd_patterns();
        assert_eq!(patterns.len(), 4);
        assert!(patterns.contains(&"TODO".to_string()));
        assert!(patterns.contains(&"FIXME".to_string()));
        assert!(patterns.contains(&"HACK".to_string()));
        assert!(patterns.contains(&"XXX".to_string()));
    }

    #[test]
    fn test_tier1_default_gates() {
        let gates = default_tier1_gates();
        assert_eq!(gates.len(), 2);
        assert!(gates.contains(&"clippy".to_string()));
        assert!(gates.contains(&"complexity".to_string()));
    }

    #[test]
    fn test_tier2_default_gates() {
        let gates = default_tier2_gates();
        assert_eq!(gates.len(), 4);
        assert!(gates.contains(&"clippy".to_string()));
        assert!(gates.contains(&"tests".to_string()));
        assert!(gates.contains(&"coverage".to_string()));
        assert!(gates.contains(&"satd".to_string()));
    }

    #[test]
    fn test_tier3_default_gates() {
        let gates = default_tier3_gates();
        assert_eq!(gates.len(), 6);
        assert!(gates.contains(&"mutation".to_string()));
        assert!(gates.contains(&"security".to_string()));
    }

    #[test]
    fn test_METRICS_001_default_values() {
        let config = MetricsConfig::default();
        assert!((config.thresholds.lint_ms - 5000.0).abs() < f64::EPSILON);
        assert!((config.thresholds.test_ms - 60000.0).abs() < f64::EPSILON);
        assert!((config.quality_gates.min_coverage - 95.0).abs() < f64::EPSILON);
        assert!((config.quality_gates.min_mutation_score - 90.0).abs() < f64::EPSILON);
        assert_eq!(config.quality_gates.min_tdg_grade, "A");
        assert_eq!(config.staleness.max_age_days, 7);
        assert!(config.trend_analysis.enabled);
        assert_eq!(config.trend_analysis.retention_days, 90);
    }

    #[test]
    fn test_METRICS_002_parse_toml() {
        let toml_str = r#"
[thresholds]
lint_ms = 3000
test_ms = 30000
coverage_ms = 60000
binary_size_kb = 5120

[staleness]
max_age_days = 14

[enforcement]
fail_on_stale = false
fail_on_performance_regression = true

[trend_analysis]
enabled = true
retention_days = 180

[quality_gates]
min_coverage = 90.0
min_mutation_score = 85.0
min_tdg_grade = "B"

[performance]
max_transpile_ms_per_entry = 200
max_memory_mb_per_entry = 20
"#;
        let config: MetricsConfig = toml::from_str(toml_str).expect("parse");
        assert!((config.thresholds.lint_ms - 3000.0).abs() < f64::EPSILON);
        assert_eq!(config.staleness.max_age_days, 14);
        assert!(!config.enforcement.fail_on_stale);
        assert_eq!(config.trend_analysis.retention_days, 180);
        assert!((config.quality_gates.min_coverage - 90.0).abs() < f64::EPSILON);
        assert_eq!(config.quality_gates.min_tdg_grade, "B");
        assert!((config.performance.max_transpile_ms_per_entry - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_METRICS_003_load_or_default_missing() {
        // When file doesn't exist, returns defaults
        let config = MetricsConfig::load_or_default();
        assert!((config.thresholds.lint_ms - 5000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_METRICS_004_check_budget() {
        let config = MetricsConfig::default();
        assert!(config.check_budget("lint_ms", 4000.0));
        assert!(!config.check_budget("lint_ms", 6000.0));
        assert!(config.check_budget("test_ms", 50000.0));
        assert!(!config.check_budget("test_ms", 70000.0));
        assert!(config.check_budget("unknown", 9999.0)); // unknown key always passes
    }
