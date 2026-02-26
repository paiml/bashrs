//! Additional coverage tests for quality/gates.rs
//!
//! These tests focus on data structures, configuration parsing, formatting,
//! threshold logic, and the disabled-gate paths that don't shell out to
//! external processes. NO external commands are invoked.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

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
            GateResult { gate_name: "a".to_string(), passed: true, duration: Duration::from_millis(10), message: String::new(), metrics: HashMap::new(), violations: vec![] },
            GateResult { gate_name: "b".to_string(), passed: false, duration: Duration::from_millis(20), message: String::new(), metrics: HashMap::new(), violations: vec![] },
            GateResult { gate_name: "c".to_string(), passed: true, duration: Duration::from_millis(30), message: String::new(), metrics: HashMap::new(), violations: vec![] },
        ];
        let summary = QualityGate::summary(&mixed);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.total_duration, Duration::from_millis(60));
        assert!(!QualityGate::all_passed(&mixed));

        // Single failure with violations
        let single_fail = vec![GateResult {
            gate_name: "only".to_string(), passed: false, duration: Duration::from_secs(10),
            message: "Failed hard".to_string(), metrics: HashMap::new(),
            violations: vec![GateViolation { file: None, line: None, description: "critical".to_string(), severity: ViolationSeverity::Error }],
        }];
        let summary = QualityGate::summary(&single_fail);
        assert_eq!(summary.total, 1);
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.failed, 1);
    }

    // ── MetricsConfig budget checking ──

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
                lint_ms: 1000.0, test_ms: 2000.0, coverage_ms: 3000.0, binary_size_kb: 512.0,
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
        let config: GateConfig = toml::from_str("[metadata]\nversion = \"5.0.0\"\ntool = \"my-tool\"").expect("parse");
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
        assert!(result.message.contains("disabled"), "disabled gate message should say disabled: {}", result.message);
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
        assert!(result.message.contains("disabled"), "message: {}", result.message);
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
        assert!(result.message.contains("disabled"), "message: {}", result.message);
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
        assert!(result.message.contains("disabled"), "message: {}", result.message);
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
        assert!(result.message.contains("disabled"), "message: {}", result.message);
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
                    patterns: vec![],  // No patterns = no violations found
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
        for name in &["clippy", "complexity", "tests", "coverage", "satd", "mutation", "security"] {
            let result = gate.run_gate(name);
            assert_eq!(result.gate_name, *name, "gate_name mismatch for {name}");
        }
    }
}
