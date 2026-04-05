
    #[test]
    fn test_check_budget_all_keys() {
        let config = MetricsConfig::default();
        // lint_ms boundary
        assert!(config.check_budget("lint_ms", 5000.0));
        assert!(!config.check_budget("lint_ms", 5000.1));
        // test_ms boundary
        assert!(config.check_budget("test_ms", 60000.0));
        assert!(!config.check_budget("test_ms", 60001.0));
        // coverage_ms boundary
        assert!(config.check_budget("coverage_ms", 120000.0));
        assert!(!config.check_budget("coverage_ms", 120001.0));
        // unknown always passes
        assert!(config.check_budget("nonexistent", f64::MAX));
        assert!(config.check_budget("", 999999.0));
    }

    #[test]
    fn test_check_budget_custom_thresholds() {
        let config = MetricsConfig {
            thresholds: MetricsThresholds {
                lint_ms: 1000.0,
                test_ms: 2000.0,
                coverage_ms: 3000.0,
                binary_size_kb: 512.0,
            },
            ..MetricsConfig::default()
        };
        assert!(config.check_budget("lint_ms", 999.0));
        assert!(!config.check_budget("lint_ms", 1001.0));
        assert!(config.check_budget("test_ms", 1999.0));
        assert!(!config.check_budget("test_ms", 2001.0));
    }

    // ── MetricsConfig serialization ──

    #[test]
    fn test_metrics_config_toml_roundtrip() {
        let original = MetricsConfig {
            thresholds: MetricsThresholds {
                lint_ms: 2000.0,
                test_ms: 30000.0,
                coverage_ms: 60000.0,
                binary_size_kb: 4096.0,
            },
            staleness: StalenessConfig { max_age_days: 14 },
            enforcement: MetricsEnforcement {
                fail_on_stale: false,
                fail_on_performance_regression: true,
            },
            trend_analysis: TrendAnalysisConfig {
                enabled: false,
                retention_days: 30,
            },
            quality_gates: MetricsQualityGates {
                min_coverage: 80.0,
                min_mutation_score: 75.0,
                min_tdg_grade: "B".to_string(),
            },
            performance: PerformanceBudget {
                max_transpile_ms_per_entry: 50.0,
                max_memory_mb_per_entry: 5.0,
            },
        };

        let toml_str = toml::to_string(&original).expect("serialize");
        let parsed: MetricsConfig = toml::from_str(&toml_str).expect("deserialize");

        assert!((parsed.thresholds.lint_ms - 2000.0).abs() < f64::EPSILON);
        assert_eq!(parsed.staleness.max_age_days, 14);
        assert!(!parsed.enforcement.fail_on_stale);
        assert!(!parsed.trend_analysis.enabled);
        assert_eq!(parsed.trend_analysis.retention_days, 30);
        assert!((parsed.quality_gates.min_coverage - 80.0).abs() < f64::EPSILON);
        assert_eq!(parsed.quality_gates.min_tdg_grade, "B");
        assert!((parsed.performance.max_transpile_ms_per_entry - 50.0).abs() < f64::EPSILON);
    }

    // ── GateConfigError Display variants ──

    #[test]
    fn test_gate_config_error_display() {
        let io_err = GateConfigError::Io {
            path: std::path::PathBuf::from("/some/path/gates.toml"),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied"),
        };
        let msg = format!("{}", io_err);
        assert!(msg.contains("/some/path/gates.toml") && msg.contains("Failed to read"));

        let parse_err = GateConfigError::Parse {
            path: std::path::PathBuf::from("/my/config.toml"),
            source: toml::from_str::<GateConfig>("[broken %%").unwrap_err(),
        };
        let msg = format!("{}", parse_err);
        assert!(msg.contains("/my/config.toml") && msg.contains("Failed to parse"));
    }

    // ── gates_for_tier with custom tiers ──

    #[test]
    fn test_gates_for_tier_custom_config() {
        let config = GateConfig {
            tiers: TiersConfig {
                tier1_gates: vec!["fmt".to_string()],
                tier2_gates: vec!["fmt".to_string(), "test".to_string()],
                tier3_gates: vec![
                    "fmt".to_string(),
                    "test".to_string(),
                    "mutation".to_string(),
                ],
            },
            ..GateConfig::default()
        };

        assert_eq!(config.gates_for_tier(Tier::Tier1), &["fmt"]);
        assert_eq!(config.gates_for_tier(Tier::Tier2), &["fmt", "test"]);
        assert_eq!(
            config.gates_for_tier(Tier::Tier3),
            &["fmt", "test", "mutation"]
        );
    }

    #[test]
    fn test_gates_for_tier_empty_gates() {
        let config = GateConfig {
            tiers: TiersConfig {
                tier1_gates: vec![],
                tier2_gates: vec![],
                tier3_gates: vec![],
            },
            ..GateConfig::default()
        };
        assert!(config.gates_for_tier(Tier::Tier1).is_empty());
        assert!(config.gates_for_tier(Tier::Tier2).is_empty());
        assert!(config.gates_for_tier(Tier::Tier3).is_empty());
    }

    // ── run_gate with unknown gate name ──

    #[test]
    fn test_run_gate_unknown_returns_failed_result() {
        let gate = QualityGate::with_defaults();
        let result = gate.run_gate("nonexistent_gate_name");
        assert!(!result.passed);
        assert!(result.message.contains("Unknown gate"));
        assert!(result.message.contains("nonexistent_gate_name"));
        assert!(result.violations.is_empty());
        assert!(result.metrics.is_empty());
    }

    // ── run_tier with all gates disabled ──

    #[test]
    fn test_run_tier1_all_disabled_all_pass() {
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
        assert_eq!(results.len(), 2);
        for r in &results {
            assert!(r.passed, "Disabled gate should pass: {}", r.gate_name);
            assert!(
                r.message.contains("disabled"),
                "Message should say disabled: {}",
                r.message
            );
        }
    }

    #[test]
    fn test_run_tier2_all_disabled() {
        let config = GateConfig {
            gates: GatesConfig {
                run_clippy: false,
                run_tests: false,
                check_coverage: false,
                satd: SatdConfig {
                    enabled: false,
                    ..SatdConfig::default()
                },
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let results = gate.run_tier(Tier::Tier2);
        assert_eq!(results.len(), 4);
        assert!(QualityGate::all_passed(&results));
    }

    // ── Partial TOML parsing (missing sections get defaults) ──

    #[test]
    fn test_partial_toml_uses_defaults_for_missing() {
        // Empty TOML
        let config: GateConfig = toml::from_str("").expect("empty TOML");
        assert_eq!(config.metadata.version, "1.0.0");
        assert!(config.gates.run_clippy);

        // Only metadata section
        let config: GateConfig =
            toml::from_str("[metadata]\nversion = \"5.0.0\"\ntool = \"my-tool\"").expect("parse");
        assert_eq!(config.metadata.version, "5.0.0");
        assert!(config.gates.run_clippy);
    }

    // ── MetricsConfig default values verified via struct defaults ──

    #[test]
    fn test_metrics_thresholds_default_values() {
        let t = MetricsThresholds::default();
        assert!((t.lint_ms - 5000.0).abs() < f64::EPSILON);
        assert!((t.test_ms - 60000.0).abs() < f64::EPSILON);
        assert!((t.coverage_ms - 120000.0).abs() < f64::EPSILON);
        assert!((t.binary_size_kb - 10240.0).abs() < f64::EPSILON);
    }

    // ── MetricsConfig sub-struct defaults (consolidated) ──

    #[test]
    fn test_metrics_sub_struct_defaults() {
        let e = MetricsEnforcement::default();
        assert!(e.fail_on_stale);
        assert!(e.fail_on_performance_regression);

        let p = PerformanceBudget::default();
        assert!((p.max_transpile_ms_per_entry - 100.0).abs() < f64::EPSILON);
        assert!((p.max_memory_mb_per_entry - 10.0).abs() < f64::EPSILON);

        let s = StalenessConfig::default();
        assert_eq!(s.max_age_days, 7);

        let t = TrendAnalysisConfig::default();
        assert!(t.enabled);
        assert_eq!(t.retention_days, 90);

        let q = MetricsQualityGates::default();
        assert!((q.min_coverage - 95.0).abs() < f64::EPSILON);
        assert!((q.min_mutation_score - 90.0).abs() < f64::EPSILON);
        assert_eq!(q.min_tdg_grade, "A");
    }

    // ── run_gate dispatch for each gate name (disabled config) ──

    /// Call run_clippy_gate via run_gate() with clippy disabled — hits the early-return branch.
    #[test]
    fn test_coverage_run_gate_clippy_disabled() {
        let config = GateConfig {
            gates: GatesConfig {
                run_clippy: false,
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let result = gate.run_gate("clippy");
        assert!(result.passed, "disabled clippy gate should pass");
        assert!(
            result.message.contains("disabled"),
            "disabled gate message should say disabled: {}",
            result.message
        );
        assert_eq!(result.gate_name, "clippy");
    }

    /// Call run_tests_gate via run_gate() with tests disabled — hits the early-return branch.
    #[test]
    fn test_coverage_run_gate_tests_disabled() {
        let config = GateConfig {
            gates: GatesConfig {
                run_tests: false,
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let result = gate.run_gate("tests");
        assert!(result.passed, "disabled tests gate should pass");
        assert!(
            result.message.contains("disabled"),
            "message: {}",
            result.message
        );
        assert_eq!(result.gate_name, "tests");
    }

    /// Call run_satd_gate via run_gate() with SATD disabled — hits the early-return branch.
    #[test]
    fn test_coverage_run_gate_satd_disabled() {
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
        assert!(result.passed, "disabled SATD gate should pass");
        assert!(
            result.message.contains("disabled"),
            "message: {}",
            result.message
        );
        assert_eq!(result.gate_name, "satd");
    }

    /// Call run_coverage_gate via run_gate() with coverage disabled — hits the early-return branch.
    #[test]
    fn test_coverage_run_gate_coverage_disabled() {
        let config = GateConfig {
            gates: GatesConfig {
                check_coverage: false,
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let result = gate.run_gate("coverage");
        assert!(result.passed, "disabled coverage gate should pass");
        assert!(
            result.message.contains("disabled"),
            "message: {}",
            result.message
        );
        assert_eq!(result.gate_name, "coverage");
    }

    /// Call run_complexity_gate via run_gate() with complexity disabled.
    #[test]
    fn test_coverage_run_gate_complexity_disabled() {
        let config = GateConfig {
            gates: GatesConfig {
                check_complexity: false,
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let result = gate.run_gate("complexity");
        assert!(result.passed, "disabled complexity gate should pass");
        assert!(
            result.message.contains("disabled"),
            "message: {}",
            result.message
        );
    }

    /// Call run_mutation_gate via run_gate() with mutation disabled.
    #[test]
    fn test_coverage_run_gate_mutation_disabled() {
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
        assert!(result.passed, "disabled mutation gate should pass");
        assert_eq!(result.gate_name, "mutation");
    }

    /// Call run_security_gate via run_gate() with security disabled.
    #[test]
    fn test_coverage_run_gate_security_disabled() {
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
        assert!(result.passed, "disabled security gate should pass");
        assert_eq!(result.gate_name, "security");
    }

    /// Call run_coverage_gate with coverage enabled (runs the placeholder path).
    #[test]
    fn test_coverage_run_gate_coverage_enabled_placeholder() {
        let config = GateConfig {
            gates: GatesConfig {
                check_coverage: true,
                min_coverage: 85.0,
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let result = gate.run_gate("coverage");
        // Coverage gate is a placeholder - always passes but provides a message
        assert!(result.passed);
        assert!(!result.message.is_empty());
        assert_eq!(result.gate_name, "coverage");
    }

    /// Call run_satd_gate with SATD enabled and empty pattern list — should pass with 0 count.
    #[test]
    fn test_coverage_run_gate_satd_enabled_empty_patterns() {
        let config = GateConfig {
            gates: GatesConfig {
                satd: SatdConfig {
                    enabled: true,
                    max_count: 100,
                    patterns: vec![], // No patterns = no violations found
                    fail_on_violation: true,
                    require_issue_links: false,
                },
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        let result = gate.run_gate("satd");
        // With no patterns, no grep calls happen, 0 violations found → passes
        assert!(result.passed);
        assert_eq!(result.gate_name, "satd");
        assert!(result.metrics.contains_key("count"));
    }

    /// Verify all gate name strings route to known gates (no typos).
    #[test]
    fn test_coverage_run_gate_all_known_names_return_named_results() {
        let gate = QualityGate::with_defaults();
        for name in &[
            "clippy",
            "complexity",
            "tests",
            "coverage",
            "satd",
            "mutation",
            "security",
        ] {
            let result = gate.run_gate(name);
            assert_eq!(result.gate_name, *name, "gate_name mismatch for {name}");
        }
    }
}
