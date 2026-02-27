//! Tests for formatting functions across corpus modules:
//! - `tier_analysis`: `format_tier_weights`, `format_tier_analysis`, `format_tier_targets`
//! - `citl`: `format_convergence_criteria`, `format_lint_pipeline`, `format_regression_report`
//! - `schema_enforcement`: `format_schema_report`, `format_grammar_errors`, `format_grammar_spec`
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::corpus::citl::{
    format_convergence_criteria, format_lint_pipeline, format_regression_report,
    ConvergenceCriteria, LintPipelineEntry, RegressionEntry, RegressionReport,
};
use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier, Grade};
use crate::corpus::runner::{CorpusResult, CorpusScore};
use crate::corpus::schema_enforcement::{
    format_grammar_errors, format_grammar_spec, format_schema_report, validate_corpus,
    validate_entry, GrammarCategory, SchemaReport, ValidationLayer,
};
use crate::corpus::tier_analysis::{
    format_tier_analysis, format_tier_targets, format_tier_weights, TierStats, TierWeightedScore,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_tier_stats(tier: CorpusTier, total: usize, passed: usize) -> TierStats {
    let failed = total.saturating_sub(passed);
    let weight = tier.weight();
    let pass_rate = if total > 0 {
        passed as f64 / total as f64
    } else {
        0.0
    };
    let weighted_score = pass_rate * weight * total as f64;
    let target_rate = tier.target_rate();
    TierStats {
        tier,
        total,
        passed,
        failed,
        weight,
        pass_rate,
        weighted_score,
        target_rate,
        meets_target: pass_rate >= target_rate,
    }
}

fn make_analysis(tiers: Vec<TierStats>) -> TierWeightedScore {
    let total_weighted_pass: f64 = tiers.iter().map(|t| t.weighted_score).sum();
    let total_weight: f64 = tiers.iter().map(|t| t.weight * t.total as f64).sum();
    let total_pass: usize = tiers.iter().map(|t| t.passed).sum();
    let total_entries: usize = tiers.iter().map(|t| t.total).sum();

    let weighted_score = if total_weight > 0.0 {
        (total_weighted_pass / total_weight) * 100.0
    } else {
        0.0
    };
    let unweighted_score = if total_entries > 0 {
        (total_pass as f64 / total_entries as f64) * 100.0
    } else {
        0.0
    };
    let all_targets_met = tiers.iter().all(|t| t.total == 0 || t.meets_target);

    TierWeightedScore {
        tiers,
        weighted_score,
        unweighted_score,
        weight_delta: weighted_score - unweighted_score,
        all_targets_met,
    }
}

fn make_corpus_entry(id: &str, format: CorpusFormat, output: &str) -> CorpusEntry {
    CorpusEntry {
        id: id.to_string(),
        name: format!("test-{id}"),
        description: "Test entry".to_string(),
        format,
        tier: CorpusTier::Trivial,
        input: String::new(),
        expected_output: output.to_string(),
        shellcheck: true,
        deterministic: true,
        idempotent: true,
    }
}

// ========================
// tier_analysis: format_tier_targets
// ========================

#[test]
fn test_format_tier_targets_empty_all_tiers() {
    let tiers: Vec<TierStats> = [
        CorpusTier::Trivial,
        CorpusTier::Standard,
        CorpusTier::Complex,
        CorpusTier::Adversarial,
        CorpusTier::Production,
    ]
    .iter()
    .map(|t| make_tier_stats(*t, 0, 0))
    .collect();

    let analysis = make_analysis(tiers);
    let report = format_tier_targets(&analysis);

    // Every tier should show EMPTY
    assert_eq!(report.matches("EMPTY").count(), 5);
    assert!(report.contains("ALL TARGETS MET"));
    // No risk ranking when all tiers are empty
    assert!(!report.contains("Risk ranking"));
}

#[test]
fn test_format_tier_targets_single_tier_passing() {
    let tiers = vec![make_tier_stats(CorpusTier::Trivial, 100, 100)];
    let analysis = make_analysis(tiers);
    let report = format_tier_targets(&analysis);

    assert!(report.contains("100.0%"));
    assert!(report.contains("PASS"));
    assert!(report.contains("ALL TARGETS MET"));
    // Trivial target_rate is 1.0, so margin = 0.0 => "AT RISK" (margin < 0.02)
    assert!(report.contains("AT RISK"));
}

#[test]
fn test_format_tier_targets_single_tier_failing() {
    let tiers = vec![make_tier_stats(CorpusTier::Trivial, 100, 50)];
    let analysis = make_analysis(tiers);
    let report = format_tier_targets(&analysis);

    assert!(report.contains("50.0%"));
    assert!(report.contains("FAIL"));
    assert!(report.contains("TARGETS NOT MET"));
    assert!(report.contains("BELOW TARGET"));
}

#[test]
fn test_format_tier_targets_mixed_pass_fail() {
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 100, 100),
        make_tier_stats(CorpusTier::Standard, 100, 95),
        make_tier_stats(CorpusTier::Complex, 50, 49),
        make_tier_stats(CorpusTier::Adversarial, 20, 19),
        make_tier_stats(CorpusTier::Production, 10, 5),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_targets(&analysis);

    // Standard is 95%, target 99% -> FAIL
    assert!(report.contains("FAIL"));
    assert!(report.contains("TARGETS NOT MET"));
    assert!(report.contains("Risk ranking"));
}

#[test]
fn test_format_tier_targets_at_risk_margin() {
    // 1% margin above target -> AT RISK
    let mut ts = make_tier_stats(CorpusTier::Adversarial, 100, 96);
    // Adversarial target: 95%, actual: 96%, margin = 1% = 0.01
    ts.meets_target = true;
    let tiers = vec![ts];
    let analysis = make_analysis(tiers);
    let report = format_tier_targets(&analysis);

    assert!(report.contains("AT RISK"));
}

#[test]
fn test_format_tier_targets_marginal() {
    // 3% margin above target -> MARGINAL
    let mut ts = make_tier_stats(CorpusTier::Adversarial, 100, 98);
    // Adversarial target: 95%, actual: 98%, margin = 3% = 0.03
    ts.meets_target = true;
    let tiers = vec![ts];
    let analysis = make_analysis(tiers);
    let report = format_tier_targets(&analysis);

    assert!(report.contains("MARGINAL"));
}

#[test]
fn test_format_tier_targets_delta_formatting() {
    let tiers = vec![make_tier_stats(CorpusTier::Production, 100, 100)];
    let analysis = make_analysis(tiers);
    let report = format_tier_targets(&analysis);

    // Production target: 95%, actual: 100% -> delta should be +5.0%
    assert!(report.contains("+5.0%"));
}

// ========================
// tier_analysis: format_tier_weights
// ========================

#[test]
fn test_format_tier_weights_empty_tiers() {
    let tiers: Vec<TierStats> = [
        CorpusTier::Trivial,
        CorpusTier::Standard,
        CorpusTier::Complex,
        CorpusTier::Adversarial,
        CorpusTier::Production,
    ]
    .iter()
    .map(|t| make_tier_stats(*t, 0, 0))
    .collect();

    let analysis = make_analysis(tiers);
    let report = format_tier_weights(&analysis);

    assert!(report.contains("Tier-Weighted"));
    assert!(report.contains("Weighted Score:"));
    assert!(report.contains("Unweighted Score:"));
    assert!(report.contains("Weight Effect:"));
    // Empty tiers show "-" for rate and weighted
    assert!(report.matches('-').count() >= 5);
}

#[test]
fn test_format_tier_weights_positive_delta() {
    // Higher tiers pass more -> positive weight delta
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 100, 50),
        make_tier_stats(CorpusTier::Production, 100, 100),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_weights(&analysis);

    assert!(report.contains("+"));
}

#[test]
fn test_format_tier_weights_negative_delta() {
    // Higher tiers fail more -> negative weight delta
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 100, 100),
        make_tier_stats(CorpusTier::Production, 100, 50),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_weights(&analysis);

    // Negative delta shows no "+" prefix
    assert!(report.contains("Weight Effect:"));
}

// ========================
// tier_analysis: format_tier_analysis
// ========================

#[test]
fn test_format_tier_analysis_distribution_bars() {
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 50, 50),
        make_tier_stats(CorpusTier::Standard, 30, 30),
        make_tier_stats(CorpusTier::Complex, 10, 10),
        make_tier_stats(CorpusTier::Adversarial, 5, 5),
        make_tier_stats(CorpusTier::Production, 5, 5),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_analysis(&analysis);

    assert!(report.contains("Distribution:"));
    assert!(report.contains("T1: Trivial"));
    assert!(report.contains("T2: Standard"));
    assert!(report.contains("T3: Complex"));
    assert!(report.contains("T4: Adversarial"));
    assert!(report.contains("T5: Production"));
    assert!(report.contains("Scoring Comparison"));
    assert!(report.contains("Weight Impact"));
}

#[test]
fn test_format_tier_analysis_zero_delta_interpretation() {
    // All 100% pass -> delta is 0
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 50, 50),
        make_tier_stats(CorpusTier::Production, 50, 50),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_analysis(&analysis);

    assert!(report.contains("No difference"));
}

#[test]
fn test_format_tier_analysis_positive_delta_interpretation() {
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 100, 50),
        make_tier_stats(CorpusTier::Production, 100, 100),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_analysis(&analysis);

    assert!(report.contains("Higher tiers performing better"));
}

#[test]
fn test_format_tier_analysis_negative_delta_interpretation() {
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 100, 100),
        make_tier_stats(CorpusTier::Production, 100, 50),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_analysis(&analysis);

    assert!(report.contains("Lower tiers performing better"));
}

#[test]
fn test_format_tier_analysis_empty_tiers_skipped_in_impact() {
    let tiers = vec![
        make_tier_stats(CorpusTier::Trivial, 0, 0),
        make_tier_stats(CorpusTier::Standard, 10, 10),
    ];
    let analysis = make_analysis(tiers);
    let report = format_tier_analysis(&analysis);

    // Trivial has 0 entries, so it should not appear in impact section
    // but it DOES appear in distribution. Impact section skips zero-total tiers.
    assert!(report.contains("Weight Impact"));
    // Standard should appear in impact
    assert!(report.contains("T2: Standard"));
}

// ========================
// citl: format_convergence_criteria
// ========================

#[test]
fn test_format_convergence_criteria_all_met() {
    let criteria = ConvergenceCriteria {
        rate_met: true,
        rate_values: vec![0.99, 1.0, 1.0],
        stability_met: true,
        delta_values: vec![0.001, 0.0, 0.001],
        growth_met: true,
        corpus_size: 1000,
        target_size: 900,
        no_regressions: true,
        converged: true,
    };
    let table = format_convergence_criteria(&criteria);

    assert!(table.contains("CONVERGED"));
    assert!(table.contains("Shewhart 1931"));
    assert!(table.contains("Add harder entries"));
    assert!(table.contains("PASS"));
    assert!(!table.contains("FAIL"));
    assert!(table.contains("1000/900"));
    assert!(table.contains("clean"));
}

#[test]
fn test_format_convergence_criteria_none_met() {
    let criteria = ConvergenceCriteria {
        rate_met: false,
        rate_values: vec![0.80, 0.85, 0.90],
        stability_met: false,
        delta_values: vec![0.05, 0.03, 0.02],
        growth_met: false,
        corpus_size: 200,
        target_size: 900,
        no_regressions: false,
        converged: false,
    };
    let table = format_convergence_criteria(&criteria);

    assert!(table.contains("NOT CONVERGED"));
    assert!(table.contains("4 criteria failing"));
    assert!(table.contains("rate"));
    assert!(table.contains("stability"));
    assert!(table.contains("growth"));
    assert!(table.contains("regressions"));
}

#[test]
fn test_format_convergence_criteria_empty_history() {
    let criteria = ConvergenceCriteria {
        rate_met: false,
        rate_values: vec![],
        stability_met: false,
        delta_values: vec![],
        growth_met: false,
        corpus_size: 0,
        target_size: 900,
        no_regressions: true,
        converged: false,
    };
    let table = format_convergence_criteria(&criteria);

    assert!(table.contains("no history"));
    assert!(table.contains("NOT CONVERGED"));
}

#[test]
fn test_format_convergence_criteria_partial_met() {
    let criteria = ConvergenceCriteria {
        rate_met: true,
        rate_values: vec![0.99, 1.0, 1.0],
        stability_met: true,
        delta_values: vec![0.001, 0.0, 0.001],
        growth_met: false,
        corpus_size: 500,
        target_size: 900,
        no_regressions: true,
        converged: false,
    };
    let table = format_convergence_criteria(&criteria);

    assert!(table.contains("NOT CONVERGED"));
    assert!(table.contains("1 criteria failing"));
    assert!(table.contains("growth"));
    // rate and stability should PASS
    assert!(table.matches("PASS").count() >= 3);
}

#[test]
fn test_format_convergence_criteria_rate_values_displayed() {
    let criteria = ConvergenceCriteria {
        rate_met: true,
        rate_values: vec![0.993, 0.997, 1.0],
        stability_met: true,
        delta_values: vec![0.002, 0.001, 0.0],
        growth_met: true,
        corpus_size: 900,
        target_size: 900,
        no_regressions: true,
        converged: true,
    };
    let table = format_convergence_criteria(&criteria);

    // Rate values should be formatted as percentages
    assert!(table.contains("99.3%"));
    assert!(table.contains("99.7%"));
    assert!(table.contains("100.0%"));
}

#[test]
fn test_format_convergence_criteria_regressions_found() {
    let criteria = ConvergenceCriteria {
        rate_met: true,
        rate_values: vec![1.0, 1.0, 1.0],
        stability_met: true,
        delta_values: vec![0.0, 0.0, 0.0],
        growth_met: true,
        corpus_size: 900,
        target_size: 900,
        no_regressions: false,
        converged: false,
    };
    let table = format_convergence_criteria(&criteria);

    assert!(table.contains("regressions found"));
    assert!(table.contains("NOT CONVERGED"));
}

// ========================
// citl: format_lint_pipeline
// ========================

#[test]
fn test_format_lint_pipeline_multiple_suggestions() {
    let suggestions = vec![
        LintPipelineEntry {
            source_id: "B-001".into(),
            rule: "SEC003".into(),
            message: "unquoted variable".into(),
            suggested_id: "B-501".into(),
            suggested_name: "security-violation-from-B-001".into(),
            format: CorpusFormat::Bash,
        },
        LintPipelineEntry {
            source_id: "M-010".into(),
            rule: "MAKE005".into(),
            message: "tab issue".into(),
            suggested_id: "M-201".into(),
            suggested_name: "makefile-lint-violation-from-M-010".into(),
            format: CorpusFormat::Makefile,
        },
    ];
    let table = format_lint_pipeline(&suggestions);

    assert!(table.contains("B-001"));
    assert!(table.contains("M-010"));
    assert!(table.contains("SEC003"));
    assert!(table.contains("MAKE005"));
    assert!(table.contains("2 suggestion(s)"));
}

// ========================
// citl: format_regression_report
// ========================

#[test]
fn test_format_regression_report_with_long_error() {
    let report = RegressionReport {
        regressions: vec![RegressionEntry {
            id: "B-999".into(),
            format: CorpusFormat::Bash,
            error: "This is a very long error message that definitely exceeds forty characters in length and should be truncated".into(),
        }],
        improvements: vec![],
        total: 100,
        andon_triggered: true,
    };
    let table = format_regression_report(&report);

    // Long errors are truncated to 37 chars + "..."
    assert!(table.contains("..."));
    assert!(table.contains("ANDON CORD"));
}

#[test]
fn test_format_regression_report_with_improvements() {
    let report = RegressionReport {
        regressions: vec![],
        improvements: vec!["5 new entries added (895 -> 900)".into()],
        total: 900,
        andon_triggered: false,
    };
    let table = format_regression_report(&report);

    assert!(table.contains("No regressions"));
    assert!(table.contains("Improvements"));
    assert!(table.contains("5 new entries"));
    assert!(table.contains("OK"));
}

// ========================
// schema_enforcement: format_schema_report
// ========================

#[test]
fn test_format_schema_report_all_formats() {
    let entries = vec![
        make_corpus_entry("B-001", CorpusFormat::Bash, "#!/bin/sh\necho \"ok\"\n"),
        make_corpus_entry("M-001", CorpusFormat::Makefile, "all:\n\techo ok\n"),
        make_corpus_entry(
            "D-001",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nRUN echo ok\n",
        ),
    ];
    let registry = crate::corpus::registry::CorpusRegistry { entries };
    let report = validate_corpus(&registry);
    let table = format_schema_report(&report);

    assert!(table.contains("Bash"));
    assert!(table.contains("Makefile"));
    assert!(table.contains("Dockerfile"));
    assert!(table.contains("Total"));
    assert!(table.contains("100.0%"));
}

#[test]
fn test_format_schema_report_mixed_validity() {
    let entries = vec![
        make_corpus_entry("B-001", CorpusFormat::Bash, "#!/bin/sh\necho \"ok\"\n"),
        make_corpus_entry(
            "B-002",
            CorpusFormat::Bash,
            "#!/bin/sh\nif [[ 1 ]]; then echo ok; fi\n",
        ),
    ];
    let registry = crate::corpus::registry::CorpusRegistry { entries };
    let report = validate_corpus(&registry);
    let table = format_schema_report(&report);

    // 1 valid, 1 invalid = 50% pass rate
    assert!(table.contains("50.0%"));
}

#[test]
fn test_format_schema_report_empty_corpus() {
    let registry = crate::corpus::registry::CorpusRegistry {
        entries: vec![],
    };
    let report = validate_corpus(&registry);
    let table = format_schema_report(&report);

    assert!(table.contains("Total"));
    assert!(table.contains("0.0%"));
}

// ========================
// schema_enforcement: format_grammar_errors
// ========================

#[test]
fn test_format_grammar_errors_all_categories_shown() {
    let entries = vec![make_corpus_entry(
        "B-001",
        CorpusFormat::Bash,
        "#!/bin/sh\necho \"ok\"\n",
    )];
    let registry = crate::corpus::registry::CorpusRegistry { entries };
    let report = validate_corpus(&registry);
    let table = format_grammar_errors(&report);

    // All 8 GRAM codes should be listed
    for i in 1..=8 {
        assert!(
            table.contains(&format!("GRAM-{i:03}")),
            "Missing GRAM-{i:03} in table"
        );
    }
}

#[test]
fn test_format_grammar_errors_multiple_violations() {
    let entries = vec![
        make_corpus_entry(
            "B-001",
            CorpusFormat::Bash,
            "#!/bin/sh\nif [[ -f file ]]; then echo $var; fi\n",
        ),
        make_corpus_entry("D-001", CorpusFormat::Dockerfile, "RUN apt-get update\n"),
    ];
    let registry = crate::corpus::registry::CorpusRegistry { entries };
    let report = validate_corpus(&registry);
    let table = format_grammar_errors(&report);

    assert!(table.contains("Entries with violations"));
    assert!(table.contains("B-001"));
    assert!(table.contains("D-001"));
}

// ========================
// schema_enforcement: format_grammar_spec
// ========================

#[test]
fn test_format_grammar_spec_all_formats() {
    let bash_spec = format_grammar_spec(CorpusFormat::Bash);
    assert!(bash_spec.contains("POSIX Shell Grammar"));
    assert!(bash_spec.contains("complete_command"));
    assert!(bash_spec.contains("L1: Lexical"));
    assert!(bash_spec.contains("L4: Behavioral"));

    let make_spec = format_grammar_spec(CorpusFormat::Makefile);
    assert!(make_spec.contains("GNU Make Grammar"));
    assert!(make_spec.contains("recipe"));
    assert!(make_spec.contains("assignment_op"));

    let docker_spec = format_grammar_spec(CorpusFormat::Dockerfile);
    assert!(docker_spec.contains("Dockerfile Grammar"));
    assert!(docker_spec.contains("FROM"));
    assert!(docker_spec.contains("exec_form"));
    assert!(docker_spec.contains("shell_form"));
}

// ========================
// schema_enforcement: SchemaReport::pass_rate edge cases
// ========================

#[test]
fn test_schema_report_pass_rate_all_valid() {
    let report = SchemaReport {
        results: vec![],
        total_entries: 100,
        valid_entries: 100,
        total_violations: 0,
        violations_by_category: vec![],
    };
    assert!((report.pass_rate() - 100.0).abs() < 0.01);
}

#[test]
fn test_schema_report_pass_rate_none_valid() {
    let report = SchemaReport {
        results: vec![],
        total_entries: 50,
        valid_entries: 0,
        total_violations: 50,
        violations_by_category: vec![],
    };
    assert!((report.pass_rate() - 0.0).abs() < 0.01);
}

// ========================
// schema_enforcement: GrammarCategory exhaustive coverage
// ========================

#[test]
fn test_grammar_category_fix_pattern_all() {
    for cat in GrammarCategory::all() {
        let fix = cat.fix_pattern();
        assert!(!fix.is_empty(), "fix_pattern for {:?} should not be empty", cat);
    }
}

#[test]
fn test_grammar_category_description_all() {
    for cat in GrammarCategory::all() {
        let desc = cat.description();
        assert!(!desc.is_empty(), "description for {:?} should not be empty", cat);
    }
}

#[test]
fn test_grammar_category_applicable_format_coverage() {
    let mut saw_bash = false;
    let mut saw_makefile = false;
    let mut saw_dockerfile = false;

    for cat in GrammarCategory::all() {
        match cat.applicable_format() {
            CorpusFormat::Bash => saw_bash = true,
            CorpusFormat::Makefile => saw_makefile = true,
            CorpusFormat::Dockerfile => saw_dockerfile = true,
        }
    }

    assert!(saw_bash, "At least one category should apply to Bash");
    assert!(saw_makefile, "At least one category should apply to Makefile");
    assert!(
        saw_dockerfile,
        "At least one category should apply to Dockerfile"
    );
}

// ========================
// schema_enforcement: ValidationLayer display
// ========================

#[test]
fn test_validation_layer_display_all() {
    assert_eq!(format!("{}", ValidationLayer::Lexical), "L1:Lexical");
    assert_eq!(format!("{}", ValidationLayer::Syntactic), "L2:Syntactic");
    assert_eq!(format!("{}", ValidationLayer::Semantic), "L3:Semantic");
    assert_eq!(format!("{}", ValidationLayer::Behavioral), "L4:Behavioral");
}

// ========================
// schema_enforcement: validate_entry layer tracking
// ========================

#[test]
fn test_validate_entry_bash_all_layers_pass() {
    let entry = make_corpus_entry(
        "B-100",
        CorpusFormat::Bash,
        "#!/bin/sh\nset -eu\necho \"hello world\"\n",
    );
    let result = validate_entry(&entry);
    assert!(result.valid);
    assert!(result.layers_passed.contains(&ValidationLayer::Lexical));
    assert!(result.layers_passed.contains(&ValidationLayer::Syntactic));
    assert!(result.layers_passed.contains(&ValidationLayer::Semantic));
}

#[test]
fn test_validate_entry_dockerfile_all_layers_pass() {
    let entry = make_corpus_entry(
        "D-100",
        CorpusFormat::Dockerfile,
        "FROM alpine:3.18\nRUN apk add curl\nCMD [\"curl\", \"https://example.com\"]\n",
    );
    let result = validate_entry(&entry);
    assert!(result.valid);
    assert!(result.layers_passed.contains(&ValidationLayer::Lexical));
    assert!(result.layers_passed.contains(&ValidationLayer::Syntactic));
    assert!(result.layers_passed.contains(&ValidationLayer::Semantic));
}

#[test]
fn test_validate_entry_makefile_all_layers_pass() {
    let entry = make_corpus_entry(
        "M-100",
        CorpusFormat::Makefile,
        "CC := gcc\n\nall:\n\t$(CC) -o main main.c\n",
    );
    let result = validate_entry(&entry);
    assert!(result.valid);
    assert!(result.layers_passed.contains(&ValidationLayer::Lexical));
    assert!(result.layers_passed.contains(&ValidationLayer::Syntactic));
    assert!(result.layers_passed.contains(&ValidationLayer::Semantic));
}
