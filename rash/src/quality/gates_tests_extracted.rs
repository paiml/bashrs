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

    #[test]
    fn test_gates_config_default() {
        let gates = GatesConfig::default();
        assert!(gates.run_clippy);
        assert!(gates.clippy_strict);
        assert!(gates.run_tests);
        assert_eq!(gates.test_timeout, 300);
        assert!(gates.check_coverage);
        assert_eq!(gates.min_coverage, 85.0);
        assert!(gates.check_complexity);
        assert_eq!(gates.max_complexity, 10);
    }

    #[test]
    fn test_gate_config_error_display() {
        let io_error = GateConfigError::Io {
            path: std::path::PathBuf::from("/test/path.toml"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        assert!(format!("{}", io_error).contains("/test/path.toml"));
        assert!(format!("{}", io_error).contains("Failed to read"));
    }

    #[test]
    fn test_gate_config_error_parse_display() {
        // Trigger a real parse error by parsing invalid TOML
        let result: Result<GateConfig, _> = toml::from_str("invalid { toml }");
        assert!(result.is_err());
        let parse_error = GateConfigError::Parse {
            path: std::path::PathBuf::from("/test/config.toml"),
            source: result.unwrap_err(),
        };
        assert!(format!("{}", parse_error).contains("/test/config.toml"));
        assert!(format!("{}", parse_error).contains("Failed to parse"));
    }

    #[test]
    fn test_violation_severity_variants() {
        let error = ViolationSeverity::Error;
        let warning = ViolationSeverity::Warning;
        let info = ViolationSeverity::Info;

        assert_eq!(error, ViolationSeverity::Error);
        assert_eq!(warning, ViolationSeverity::Warning);
        assert_eq!(info, ViolationSeverity::Info);
        assert_ne!(error, warning);
    }

    #[test]
    fn test_gate_violation_creation() {
        let violation = GateViolation {
            file: Some("test.rs".to_string()),
            line: Some(42),
            description: "Test violation".to_string(),
            severity: ViolationSeverity::Error,
        };
        assert_eq!(violation.file, Some("test.rs".to_string()));
        assert_eq!(violation.line, Some(42));
        assert_eq!(violation.description, "Test violation");
        assert_eq!(violation.severity, ViolationSeverity::Error);
    }

    #[test]
    fn test_gate_result_with_violations() {
        let violations = vec![
            GateViolation {
                file: Some("a.rs".to_string()),
                line: Some(1),
                description: "first".to_string(),
                severity: ViolationSeverity::Error,
            },
            GateViolation {
                file: None,
                line: None,
                description: "second".to_string(),
                severity: ViolationSeverity::Warning,
            },
        ];

        let mut metrics = HashMap::new();
        metrics.insert("count".to_string(), 2.0);

        let result = GateResult {
            gate_name: "test_gate".to_string(),
            passed: false,
            duration: Duration::from_secs(1),
            message: "2 violations found".to_string(),
            metrics,
            violations: violations.clone(),
        };

        assert_eq!(result.violations.len(), 2);
        assert_eq!(result.metrics.get("count"), Some(&2.0));
        assert!(!result.passed);
    }

    #[test]
    fn test_all_passed_empty_results() {
        let empty: Vec<GateResult> = vec![];
        assert!(QualityGate::all_passed(&empty));
    }

    #[test]
    fn test_summary_empty_results() {
        let empty: Vec<GateResult> = vec![];
        let summary = QualityGate::summary(&empty);
        assert_eq!(summary.total, 0);
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.total_duration, Duration::default());
    }

    #[test]
    fn test_quality_gate_new() {
        let config = GateConfig {
            gates: GatesConfig {
                max_complexity: 15,
                ..GatesConfig::default()
            },
            ..GateConfig::default()
        };
        let gate = QualityGate::new(config);
        assert_eq!(gate.config.gates.max_complexity, 15);
    }

    #[test]
    fn test_gate_config_load_nonexistent() {
        let result = GateConfig::load(std::path::Path::new("/nonexistent/path.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_gate_config_load_or_default() {
        // This should return default config since no .pmat-gates.toml exists
        let config = GateConfig::load_or_default();
        assert_eq!(config.metadata.version, "1.0.0");
    }

    #[test]
    fn test_comprehensive_gate_config_parse() {
        let toml_content = r#"
[metadata]
version = "2.0.0"
tool = "custom-tool"

[gates]
run_clippy = false
clippy_strict = false
run_tests = false
test_timeout = 600
check_coverage = false
min_coverage = 95.0
check_complexity = false
max_complexity = 15

[gates.satd]
enabled = false
max_count = 10
patterns = ["TODO"]
require_issue_links = false
fail_on_violation = false

[gates.mutation]
enabled = true
min_score = 90.0
tool = "mutagen"
strategy = "full"

[gates.security]
enabled = false
audit_vulnerabilities = "warn"
audit_unmaintained = "deny"
max_unsafe_blocks = 5
fail_on_violation = false

[tiers]
tier1_gates = ["fmt"]
tier2_gates = ["fmt", "clippy"]
tier3_gates = ["fmt", "clippy", "coverage"]

[risk_based]
very_high_risk_mutation_target = 95.0
very_high_risk_components = ["parser", "security"]
high_risk_mutation_target = 90.0
high_risk_components = ["linter"]
"#;

        let config: GateConfig = toml::from_str(toml_content).expect("valid toml");

        // Metadata
        assert_eq!(config.metadata.version, "2.0.0");
        assert_eq!(config.metadata.tool, "custom-tool");

        // Gates
        assert!(!config.gates.run_clippy);
        assert!(!config.gates.clippy_strict);
        assert!(!config.gates.run_tests);
        assert_eq!(config.gates.test_timeout, 600);
        assert!(!config.gates.check_coverage);
        assert_eq!(config.gates.min_coverage, 95.0);
        assert!(!config.gates.check_complexity);
        assert_eq!(config.gates.max_complexity, 15);

        // SATD
        assert!(!config.gates.satd.enabled);
        assert_eq!(config.gates.satd.max_count, 10);
        assert_eq!(config.gates.satd.patterns, vec!["TODO"]);
        assert!(!config.gates.satd.require_issue_links);
        assert!(!config.gates.satd.fail_on_violation);

        // Mutation
        assert!(config.gates.mutation.enabled);
        assert_eq!(config.gates.mutation.min_score, 90.0);
        assert_eq!(config.gates.mutation.tool, "mutagen");
        assert_eq!(config.gates.mutation.strategy, "full");

        // Security
        assert!(!config.gates.security.enabled);
        assert_eq!(config.gates.security.audit_vulnerabilities, "warn");
        assert_eq!(config.gates.security.audit_unmaintained, "deny");
        assert_eq!(config.gates.security.max_unsafe_blocks, 5);
        assert!(!config.gates.security.fail_on_violation);

        // Tiers
        assert_eq!(config.tiers.tier1_gates, vec!["fmt"]);
        assert_eq!(config.tiers.tier2_gates, vec!["fmt", "clippy"]);
        assert_eq!(config.tiers.tier3_gates, vec!["fmt", "clippy", "coverage"]);

        // Risk-based
        assert_eq!(config.risk_based.very_high_risk_mutation_target, 95.0);
        assert_eq!(
            config.risk_based.very_high_risk_components,
            vec!["parser", "security"]
        );
        assert_eq!(config.risk_based.high_risk_mutation_target, 90.0);
        assert_eq!(config.risk_based.high_risk_components, vec!["linter"]);
    }

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
}
