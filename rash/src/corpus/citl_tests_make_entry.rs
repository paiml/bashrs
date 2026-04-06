use super::*;
use crate::corpus::registry::{CorpusEntry, CorpusTier};

fn make_entry(id: &str, format: CorpusFormat) -> CorpusEntry {
    CorpusEntry {
        id: id.to_string(),
        name: format!("test-{id}"),
        description: "Test entry".to_string(),
        format,
        tier: CorpusTier::Trivial,
        input: String::new(),
        expected_output: "#!/bin/sh\necho ok\n".to_string(),
        shellcheck: true,
        deterministic: true,
        idempotent: true,
    }
}

fn make_result(id: &str, transpiled: bool, lint_clean: bool) -> CorpusResult {
    CorpusResult {
        id: id.to_string(),
        transpiled,
        output_contains: transpiled,
        output_exact: transpiled,
        output_behavioral: transpiled,
        has_test: true,
        coverage_ratio: 0.95,
        schema_valid: true,
        lint_clean,
        deterministic: transpiled,
        metamorphic_consistent: transpiled,
        cross_shell_agree: transpiled,
        expected_output: None,
        actual_output: if transpiled {
            Some("#!/bin/sh\n".into())
        } else {
            None
        },
        error: if !lint_clean {
            Some("SEC003: unquoted variable".into())
        } else if !transpiled {
            Some("transpilation failed".into())
        } else {
            None
        },
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    }
}

fn make_convergence(
    iter: u32,
    rate: f64,
    delta: f64,
    total: usize,
    passed: usize,
) -> ConvergenceEntry {
    ConvergenceEntry {
        iteration: iter,
        date: "2026-02-09".into(),
        total,
        passed,
        failed: total - passed,
        rate,
        delta,
        notes: String::new(),
        ..Default::default()
    }
}

#[test]
fn test_extract_lint_rule_sec() {
    assert_eq!(extract_lint_rule("SEC003: unquoted variable"), "SEC003");
}

#[test]
fn test_extract_lint_rule_det() {
    assert_eq!(extract_lint_rule("DET001: non-deterministic"), "DET001");
}

#[test]
fn test_extract_lint_rule_make() {
    assert_eq!(extract_lint_rule("MAKE005 tab issue"), "MAKE005");
}

#[test]
fn test_extract_lint_rule_docker() {
    assert_eq!(extract_lint_rule("DOCKER003: no USER"), "DOCKER003");
}

#[test]
fn test_extract_lint_rule_unknown() {
    assert_eq!(extract_lint_rule("some other error"), "LINT-UNKNOWN");
}

#[test]
fn test_generate_entry_name_security() {
    let name = generate_entry_name("SEC003", "B-001");
    assert_eq!(name, "security-violation-from-B-001");
}

#[test]
fn test_generate_entry_name_determinism() {
    let name = generate_entry_name("DET001", "B-050");
    assert_eq!(name, "determinism-violation-from-B-050");
}

#[test]
fn test_generate_entry_name_dockerfile() {
    let name = generate_entry_name("DOCKER003", "D-010");
    assert_eq!(name, "dockerfile-lint-violation-from-D-010");
}

#[test]
fn test_find_max_corpus_id() {
    let registry = CorpusRegistry {
        entries: vec![
            make_entry("B-001", CorpusFormat::Bash),
            make_entry("B-500", CorpusFormat::Bash),
            make_entry("M-200", CorpusFormat::Makefile),
        ],
    };
    assert_eq!(find_max_corpus_id(&registry), 500);
}

#[test]
fn test_find_max_corpus_id_empty() {
    let registry = CorpusRegistry::default();
    assert_eq!(find_max_corpus_id(&registry), 0);
}

#[test]
fn test_lint_pipeline_clean() {
    let registry = CorpusRegistry {
        entries: vec![make_entry("B-001", CorpusFormat::Bash)],
    };
    let score = CorpusScore {
        total: 1,
        passed: 1,
        failed: 0,
        rate: 1.0,
        score: 100.0,
        grade: crate::corpus::registry::Grade::APlus,
        format_scores: vec![],
        results: vec![make_result("B-001", true, true)],
    };

    let suggestions = lint_pipeline(&registry, &score);
    assert!(suggestions.is_empty());
}

#[test]
fn test_lint_pipeline_violation() {
    let registry = CorpusRegistry {
        entries: vec![make_entry("B-001", CorpusFormat::Bash)],
    };
    let score = CorpusScore {
        total: 1,
        passed: 1,
        failed: 0,
        rate: 1.0,
        score: 80.0,
        grade: crate::corpus::registry::Grade::B,
        format_scores: vec![],
        results: vec![make_result("B-001", true, false)],
    };

    let suggestions = lint_pipeline(&registry, &score);
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].source_id, "B-001");
    assert_eq!(suggestions[0].rule, "SEC003");
    assert!(suggestions[0].suggested_id.starts_with("B-"));
}

#[test]
fn test_lint_pipeline_not_transpiled() {
    let registry = CorpusRegistry {
        entries: vec![make_entry("B-001", CorpusFormat::Bash)],
    };
    let score = CorpusScore {
        total: 1,
        passed: 0,
        failed: 1,
        rate: 0.0,
        score: 0.0,
        grade: crate::corpus::registry::Grade::F,
        format_scores: vec![],
        results: vec![make_result("B-001", false, false)],
    };

    // Entries that didn't transpile shouldn't generate lint suggestions
    let suggestions = lint_pipeline(&registry, &score);
    assert!(suggestions.is_empty());
}

#[test]
fn test_check_regressions_clean() {
    let score = CorpusScore {
        total: 10,
        passed: 10,
        failed: 0,
        rate: 1.0,
        score: 99.9,
        grade: crate::corpus::registry::Grade::APlus,
        format_scores: vec![],
        results: vec![],
    };
    let history = vec![make_convergence(1, 1.0, 0.0, 10, 10)];

    let report = check_regressions(&score, &history);
    assert!(report.regressions.is_empty());
    assert!(!report.andon_triggered);
}

#[test]
fn test_check_regressions_detected() {
    let score = CorpusScore {
        total: 10,
        passed: 8,
        failed: 2,
        rate: 0.8,
        score: 80.0,
        grade: crate::corpus::registry::Grade::B,
        format_scores: vec![],
        results: vec![
            make_result("B-001", false, false),
            make_result("B-002", false, false),
        ],
    };
    let history = vec![make_convergence(1, 1.0, 0.0, 10, 10)];

    let report = check_regressions(&score, &history);
    assert_eq!(report.regressions.len(), 2);
    assert!(report.andon_triggered);
}

#[test]
fn test_check_regressions_no_history() {
    let score = CorpusScore {
        total: 10,
        passed: 9,
        failed: 1,
        rate: 0.9,
        score: 90.0,
        grade: crate::corpus::registry::Grade::A,
        format_scores: vec![],
        results: vec![make_result("B-001", false, false)],
    };

    let report = check_regressions(&score, &[]);
    assert!(!report.andon_triggered);
}

#[test]
fn test_convergence_criteria_met() {
    let score = CorpusScore {
        total: 900,
        passed: 900,
        failed: 0,
        rate: 1.0,
        score: 99.9,
        grade: crate::corpus::registry::Grade::APlus,
        format_scores: vec![],
        results: vec![],
    };
    let history = vec![
        make_convergence(1, 1.0, 0.0, 900, 900),
        make_convergence(2, 1.0, 0.0, 900, 900),
        make_convergence(3, 1.0, 0.0, 900, 900),
    ];

    let criteria = check_convergence(&score, &history);
    assert!(criteria.rate_met);
    assert!(criteria.stability_met);
    assert!(criteria.growth_met);
    assert!(criteria.no_regressions);
    assert!(criteria.converged);
}

#[test]
fn test_convergence_criteria_rate_not_met() {
    let score = CorpusScore {
        total: 900,
        passed: 890,
        failed: 10,
        rate: 0.989,
        score: 90.0,
        grade: crate::corpus::registry::Grade::A,
        format_scores: vec![],
        results: vec![],
    };
    let history = vec![
        make_convergence(1, 0.95, -0.05, 900, 855),
        make_convergence(2, 0.98, 0.03, 900, 882),
        make_convergence(3, 0.989, 0.009, 900, 890),
    ];

    let criteria = check_convergence(&score, &history);
    assert!(!criteria.rate_met);
    assert!(!criteria.converged);
}

#[test]
fn test_convergence_criteria_growth_not_met() {
    let score = CorpusScore {
        total: 500,
        passed: 500,
        failed: 0,
        rate: 1.0,
        score: 99.9,
        grade: crate::corpus::registry::Grade::APlus,
        format_scores: vec![],
        results: vec![],
    };
    let history = vec![
        make_convergence(1, 1.0, 0.0, 500, 500),
        make_convergence(2, 1.0, 0.0, 500, 500),
        make_convergence(3, 1.0, 0.0, 500, 500),
    ];

    let criteria = check_convergence(&score, &history);
    assert!(criteria.rate_met);
    assert!(criteria.stability_met);
    assert!(!criteria.growth_met);
    assert!(!criteria.converged);
}

#[test]
fn test_convergence_insufficient_history() {
    let score = CorpusScore {
        total: 900,
        passed: 900,
        failed: 0,
        rate: 1.0,
        score: 99.9,
        grade: crate::corpus::registry::Grade::APlus,
        format_scores: vec![],
        results: vec![],
    };
    let history = vec![make_convergence(1, 1.0, 0.0, 900, 900)];

    let criteria = check_convergence(&score, &history);
    assert!(!criteria.rate_met); // need 3 entries
    assert!(!criteria.converged);
}

#[test]
fn test_format_lint_pipeline_clean() {
    let table = format_lint_pipeline(&[]);
    assert!(table.contains("No lint violations"));
    assert!(table.contains("CITL loop clean"));
}

#[test]
fn test_format_lint_pipeline_with_suggestions() {
    let suggestions = vec![LintPipelineEntry {
        source_id: "B-001".into(),
        rule: "SEC003".into(),
        message: "unquoted variable".into(),
        suggested_id: "B-501".into(),
        suggested_name: "security-violation-from-B-001".into(),
        format: CorpusFormat::Bash,
    }];
    let table = format_lint_pipeline(&suggestions);
    assert!(table.contains("B-001"));
    assert!(table.contains("SEC003"));
    assert!(table.contains("B-501"));
    assert!(table.contains("1 suggestion(s)"));
}

#[test]
fn test_format_regression_report_clean() {
    let report = RegressionReport {
        regressions: vec![],
        improvements: vec![],
        total: 900,
        andon_triggered: false,
    };
    let table = format_regression_report(&report);
    assert!(table.contains("No regressions"));
    assert!(table.contains("OK"));
}

#[test]
fn test_format_regression_report_with_regressions() {
    let report = RegressionReport {
        regressions: vec![RegressionEntry {
            id: "B-143".into(),
            format: CorpusFormat::Bash,
            error: "behavioral check failed".into(),
        }],
        improvements: vec![],
        total: 900,
        andon_triggered: true,
    };
    let table = format_regression_report(&report);
    assert!(table.contains("REGRESSIONS DETECTED"));
    assert!(table.contains("B-143"));
    assert!(table.contains("ANDON CORD"));
}

#[test]
fn test_format_convergence_criteria_converged() {
    let criteria = ConvergenceCriteria {
        rate_met: true,
        rate_values: vec![1.0, 1.0, 1.0],
        stability_met: true,
        delta_values: vec![0.0, 0.0, 0.0],
        growth_met: true,
        corpus_size: 900,
        target_size: 900,
        no_regressions: true,
        converged: true,
    };
    let table = format_convergence_criteria(&criteria);
    assert!(table.contains("CONVERGED"));
    assert!(table.contains("Shewhart"));
}

#[test]
fn test_format_convergence_criteria_not_converged() {
    let criteria = ConvergenceCriteria {
        rate_met: true,
        rate_values: vec![1.0, 1.0, 1.0],
        stability_met: false,
        delta_values: vec![0.01, -0.02, 0.03],
        growth_met: true,
        corpus_size: 900,
        target_size: 900,
        no_regressions: true,
        converged: false,
    };
    let table = format_convergence_criteria(&criteria);
    assert!(table.contains("NOT CONVERGED"));
    assert!(table.contains("stability"));
}

#[test]
fn test_guess_format() {
    assert_eq!(guess_format("B-001"), CorpusFormat::Bash);
    assert_eq!(guess_format("M-042"), CorpusFormat::Makefile);
    assert_eq!(guess_format("D-015"), CorpusFormat::Dockerfile);
}

#[test]
fn test_format_prefix() {
    assert_eq!(format_prefix(CorpusFormat::Bash), "B");
    assert_eq!(format_prefix(CorpusFormat::Makefile), "M");
    assert_eq!(format_prefix(CorpusFormat::Dockerfile), "D");
}

#[test]
fn test_status_str() {
    assert!(status_str(true).contains("PASS"));
    assert!(status_str(false).contains("FAIL"));
}
