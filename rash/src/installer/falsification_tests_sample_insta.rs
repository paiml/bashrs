
use super::*;

fn sample_installer() -> InstallerInfo {
    InstallerInfo {
        name: "test-installer".to_string(),
        version: "1.0.0".to_string(),
        steps: vec![
            StepInfo {
                id: "install-deps".to_string(),
                name: "Install Dependencies".to_string(),
                has_rollback: true,
                preconditions: vec!["network_available".to_string()],
                postconditions: vec!["deps_installed".to_string()],
                max_duration_ms: Some(120000),
            },
            StepInfo {
                id: "configure".to_string(),
                name: "Configure Application".to_string(),
                has_rollback: true,
                preconditions: vec![],
                postconditions: vec!["config_valid".to_string()],
                max_duration_ms: None,
            },
        ],
    }
}

#[test]
fn test_FALSIFY_001_generator_new() {
    let gen = FalsificationGenerator::new();
    assert!(gen.config.test_idempotency);
    assert!(gen.config.test_determinism);
}

#[test]
fn test_FALSIFY_002_config_all() {
    let config = FalsificationConfig::all();
    assert!(config.test_idempotency);
    assert!(config.test_rollback);
    assert!(config.test_performance);
    assert_eq!(config.max_tests_per_category, 10);
}

#[test]
fn test_FALSIFY_003_config_minimal() {
    let config = FalsificationConfig::minimal();
    assert!(config.test_idempotency);
    assert!(config.test_determinism);
    assert!(!config.test_rollback);
    assert!(!config.test_performance);
}

#[test]
fn test_FALSIFY_004_generate_hypotheses() {
    let gen = FalsificationGenerator::new();
    let installer = sample_installer();
    let hypotheses = gen.generate_hypotheses(&installer);

    assert!(!hypotheses.is_empty());

    // Should have idempotency hypotheses for each step
    let idem_count = hypotheses
        .iter()
        .filter(|h| h.category == HypothesisCategory::Idempotency)
        .count();
    assert_eq!(idem_count, 2);
}

#[test]
fn test_FALSIFY_005_generate_determinism_hypotheses() {
    let gen = FalsificationGenerator::new();
    let installer = sample_installer();
    let hypotheses = gen.generate_hypotheses(&installer);

    let det_count = hypotheses
        .iter()
        .filter(|h| h.category == HypothesisCategory::Determinism)
        .count();
    assert_eq!(det_count, 2);
}

#[test]
fn test_FALSIFY_006_generate_rollback_hypotheses() {
    let gen = FalsificationGenerator::new();
    let installer = sample_installer();
    let hypotheses = gen.generate_hypotheses(&installer);

    let roll_count = hypotheses
        .iter()
        .filter(|h| h.category == HypothesisCategory::RollbackCompleteness)
        .count();
    assert_eq!(roll_count, 2); // Both steps have rollback
}

#[test]
fn test_FALSIFY_007_generate_precondition_hypotheses() {
    let gen = FalsificationGenerator::new();
    let installer = sample_installer();
    let hypotheses = gen.generate_hypotheses(&installer);

    let pre_count = hypotheses
        .iter()
        .filter(|h| h.category == HypothesisCategory::PreconditionGuard)
        .count();
    assert_eq!(pre_count, 1); // Only first step has preconditions
}

#[test]
fn test_FALSIFY_008_generate_postcondition_hypotheses() {
    let gen = FalsificationGenerator::new();
    let installer = sample_installer();
    let hypotheses = gen.generate_hypotheses(&installer);

    let post_count = hypotheses
        .iter()
        .filter(|h| h.category == HypothesisCategory::PostconditionValidity)
        .count();
    assert_eq!(post_count, 2); // Both steps have postconditions
}

#[test]
fn test_FALSIFY_009_generate_performance_hypotheses() {
    let gen = FalsificationGenerator::new();
    let installer = sample_installer();
    let hypotheses = gen.generate_hypotheses(&installer);

    let perf_count = hypotheses
        .iter()
        .filter(|h| h.category == HypothesisCategory::PerformanceBound)
        .count();
    assert_eq!(perf_count, 1); // Only first step has max_duration
}

#[test]
fn test_FALSIFY_010_hypothesis_priority() {
    let gen = FalsificationGenerator::new();
    let installer = sample_installer();
    let hypotheses = gen.generate_hypotheses(&installer);

    // Hypotheses should be sorted by priority (highest first)
    let priorities: Vec<u8> = hypotheses.iter().map(|h| h.priority).collect();
    for i in 1..priorities.len() {
        assert!(priorities[i - 1] >= priorities[i]);
    }
}

#[test]
fn test_FALSIFY_011_generate_tests() {
    let gen = FalsificationGenerator::new();
    let installer = sample_installer();
    let hypotheses = gen.generate_hypotheses(&installer);
    let tests = gen.generate_tests(&hypotheses);

    assert_eq!(tests.len(), hypotheses.len());
}

#[test]
fn test_FALSIFY_012_generate_rust_tests() {
    let gen = FalsificationGenerator::new();
    let installer = sample_installer();
    let hypotheses = gen.generate_hypotheses(&installer);
    let code = gen.generate_rust_tests(&hypotheses);

    assert!(code.contains("#[test]"));
    assert!(code.contains("test_falsify_"));
    assert!(code.contains("FALSIFIABLE"));
}

#[test]
fn test_FALSIFY_013_hypothesis_category_description() {
    assert!(!HypothesisCategory::Idempotency.description().is_empty());
    assert!(!HypothesisCategory::Determinism.description().is_empty());
    assert!(!HypothesisCategory::RollbackCompleteness
        .description()
        .is_empty());
}

#[test]
fn test_FALSIFY_014_falsification_report() {
    let results = vec![
        FalsificationResult {
            test_name: "test_1".to_string(),
            hypothesis_id: "IDEM-install-deps".to_string(),
            falsified: false,
            evidence: vec![],
            error: None,
            duration_ms: 100,
        },
        FalsificationResult {
            test_name: "test_2".to_string(),
            hypothesis_id: "DET-configure".to_string(),
            falsified: true,
            evidence: vec![Evidence {
                observation: "Output differs".to_string(),
                expected: Some("hash1".to_string()),
                actual: Some("hash2".to_string()),
                supports_hypothesis: false,
            }],
            error: None,
            duration_ms: 200,
        },
    ];

    let hypotheses = vec![
        FalsificationHypothesis {
            id: "IDEM-install-deps".to_string(),
            claim: "Step is idempotent".to_string(),
            category: HypothesisCategory::Idempotency,
            falsification_method: "Execute twice".to_string(),
            step_ids: vec!["install-deps".to_string()],
            expected_evidence: "States equal".to_string(),
            falsifying_evidence: "States differ".to_string(),
            priority: 9,
        },
        FalsificationHypothesis {
            id: "DET-configure".to_string(),
            claim: "Step is deterministic".to_string(),
            category: HypothesisCategory::Determinism,
            falsification_method: "Execute twice".to_string(),
            step_ids: vec!["configure".to_string()],
            expected_evidence: "Outputs equal".to_string(),
            falsifying_evidence: "Outputs differ".to_string(),
            priority: 9,
        },
    ];

    let report = FalsificationReport::from_results("test", results, &hypotheses);

    assert_eq!(report.total_hypotheses, 2);
    assert_eq!(report.validated_count, 1);
    assert_eq!(report.falsified_count, 1);
}

#[test]
fn test_FALSIFY_015_report_format() {
    let results = vec![FalsificationResult {
        test_name: "test_1".to_string(),
        hypothesis_id: "IDEM-step1".to_string(),
        falsified: false,
        evidence: vec![],
        error: None,
        duration_ms: 100,
    }];

    let report = FalsificationReport::from_results("test-installer", results, &[]);
    let formatted = report.format();

    assert!(formatted.contains("Falsification Report"));
    assert!(formatted.contains("test-installer"));
    assert!(formatted.contains("Validated"));
}
