
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

// FIXME(PMAT-238): include!("gates_tests_extracted_gates_gate.rs");
