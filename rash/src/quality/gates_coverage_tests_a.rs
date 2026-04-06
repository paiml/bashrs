//! Additional coverage tests for quality/gates.rs
//!
//! These tests focus on data structures, configuration parsing, formatting,
//! threshold logic, and the disabled-gate paths that don't shell out to
//! external processes. NO external commands are invoked.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

#[cfg(test)]
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
