#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for corpus comparison, analysis, and diagnostics helper functions.
//! Tests internal helpers WITHOUT running CorpusRunner::run().

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier};
use crate::corpus::runner::{ConvergenceEntry, CorpusResult};

// ── Mock data builders ──────────────────────────────────────────────────────

fn mock_result(id: &str, all_pass: bool) -> CorpusResult {
    CorpusResult {
        id: id.to_string(),
        transpiled: all_pass,
        output_contains: all_pass,
        output_exact: all_pass,
        output_behavioral: all_pass,
        has_test: true,
        coverage_ratio: if all_pass { 0.95 } else { 0.0 },
        schema_valid: true,
        lint_clean: all_pass,
        deterministic: all_pass,
        metamorphic_consistent: all_pass,
        cross_shell_agree: all_pass,
        expected_output: None,
        actual_output: if all_pass {
            Some("echo hello".into())
        } else {
            None
        },
        error: if all_pass {
            None
        } else {
            Some("transpile failed".into())
        },
        error_category: None,
        error_confidence: None,
        decision_trace: None,
    }
}

fn mock_entry(id: &str, name: &str, format: CorpusFormat, tier: CorpusTier) -> CorpusEntry {
    CorpusEntry::new(
        id,
        name,
        "test description",
        format,
        tier,
        "fn main() { println!(\"hello\"); }",
        "echo hello",
    )
}

// ── corpus_compare_commands tests ───────────────────────────────────────────

#[test]
fn test_percentile_empty() {
    use super::corpus_compare_commands::percentile;
    assert!((percentile(&[], 50.0) - 0.0).abs() < 0.01);
}

#[test]
fn test_percentile_single() {
    use super::corpus_compare_commands::percentile;
    assert!((percentile(&[42.0], 50.0) - 42.0).abs() < 0.01);
}

#[test]
fn test_percentile_sorted_data() {
    use super::corpus_compare_commands::percentile;
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let p50 = percentile(&data, 50.0);
    // idx = (50/100 * 9).round() = 4.5.round() = 4 => data[4] = 5.0 (or 6.0 depending on rounding)
    assert!(p50 >= 5.0 && p50 <= 6.0, "P50 should be ~5.5, got {p50}");
    let p0 = percentile(&data, 0.0);
    assert!((p0 - 1.0).abs() < 0.01);
    let p100 = percentile(&data, 100.0);
    assert!((p100 - 10.0).abs() < 0.01);
}

#[test]
fn test_percentile_p90() {
    use super::corpus_compare_commands::percentile;
    let data: Vec<f64> = (1..=100).map(|i| i as f64).collect();
    let p90 = percentile(&data, 90.0);
    assert!(p90 >= 89.0 && p90 <= 91.0, "P90 should be ~90, got {p90}");
}

#[test]
fn test_percentile_two_elements() {
    use super::corpus_compare_commands::percentile;
    let data = vec![10.0, 20.0];
    let p50 = percentile(&data, 50.0);
    assert!((p50 - 15.0).abs() < 6.0, "P50 of [10,20] got {p50}");
}

// ── corpus_analysis_commands tests ──────────────────────────────────────────

#[test]
fn test_count_format_bash() {
    use super::corpus_analysis_commands::count_format;
    let registry = crate::corpus::registry::CorpusRegistry {
        entries: vec![
            mock_entry("B-001", "t1", CorpusFormat::Bash, CorpusTier::Standard),
            mock_entry("B-002", "t2", CorpusFormat::Bash, CorpusTier::Trivial),
            mock_entry("M-001", "t3", CorpusFormat::Makefile, CorpusTier::Standard),
        ],
    };
    assert_eq!(count_format(&registry, &CorpusFormat::Bash), 2);
    assert_eq!(count_format(&registry, &CorpusFormat::Makefile), 1);
    assert_eq!(count_format(&registry, &CorpusFormat::Dockerfile), 0);
}

#[test]
fn test_count_format_empty_registry() {
    use super::corpus_analysis_commands::count_format;
    let registry = crate::corpus::registry::CorpusRegistry { entries: vec![] };
    assert_eq!(count_format(&registry, &CorpusFormat::Bash), 0);
}

#[test]
fn test_validate_corpus_entry_valid_bash() {
    use super::corpus_analysis_commands::validate_corpus_entry;
    let entry = mock_entry(
        "B-001",
        "hello-world",
        CorpusFormat::Bash,
        CorpusTier::Standard,
    );
    let mut seen = std::collections::HashSet::new();
    let issues = validate_corpus_entry(&entry, &mut seen);
    assert!(issues.is_empty(), "Expected no issues, got: {:?}", issues);
}

#[test]
fn test_validate_corpus_entry_valid_makefile() {
    use super::corpus_analysis_commands::validate_corpus_entry;
    let entry = CorpusEntry::new(
        "M-001",
        "makefile-test",
        "desc",
        CorpusFormat::Makefile,
        CorpusTier::Standard,
        "let x = 5;",
        "X := 5",
    );
    let mut seen = std::collections::HashSet::new();
    let issues = validate_corpus_entry(&entry, &mut seen);
    // Makefile entries don't need fn main()
    assert!(issues.is_empty(), "Got issues: {:?}", issues);
}

#[test]
fn test_validate_corpus_entry_duplicate_id() {
    use super::corpus_analysis_commands::validate_corpus_entry;
    let entry = mock_entry("B-001", "test", CorpusFormat::Bash, CorpusTier::Standard);
    let mut seen = std::collections::HashSet::new();
    seen.insert("B-001".to_string());
    let issues = validate_corpus_entry(&entry, &mut seen);
    assert!(issues.iter().any(|i| i.contains("Duplicate")));
}

#[test]
fn test_validate_corpus_entry_wrong_prefix() {
    use super::corpus_analysis_commands::validate_corpus_entry;
    let entry = CorpusEntry::new(
        "M-001",
        "wrong-prefix",
        "desc",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "fn main() { }",
        "echo hello",
    );
    let mut seen = std::collections::HashSet::new();
    let issues = validate_corpus_entry(&entry, &mut seen);
    assert!(issues.iter().any(|i| i.contains("prefix")));
}

#[test]
fn test_validate_corpus_entry_empty_name() {
    use super::corpus_analysis_commands::validate_corpus_entry;
    let entry = CorpusEntry::new(
        "B-001",
        "",
        "desc",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "fn main() { }",
        "echo hello",
    );
    let mut seen = std::collections::HashSet::new();
    let issues = validate_corpus_entry(&entry, &mut seen);
    assert!(issues.iter().any(|i| i.contains("Empty name")));
}

#[test]
fn test_validate_corpus_entry_empty_description() {
    use super::corpus_analysis_commands::validate_corpus_entry;
    let entry = CorpusEntry::new(
        "B-001",
        "test",
        "",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "fn main() { }",
        "echo hello",
    );
    let mut seen = std::collections::HashSet::new();
    let issues = validate_corpus_entry(&entry, &mut seen);
    assert!(issues.iter().any(|i| i.contains("Empty description")));
}

#[test]
fn test_validate_corpus_entry_empty_input() {
    use super::corpus_analysis_commands::validate_corpus_entry;
    let entry = CorpusEntry::new(
        "B-001",
        "test",
        "desc",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "",
        "echo hello",
    );
    let mut seen = std::collections::HashSet::new();
    let issues = validate_corpus_entry(&entry, &mut seen);
    assert!(issues.iter().any(|i| i.contains("Empty input")));
}

#[test]
fn test_validate_corpus_entry_empty_expected_output() {
    use super::corpus_analysis_commands::validate_corpus_entry;
    let entry = CorpusEntry::new(
        "B-001",
        "test",
        "desc",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "fn main() { }",
        "",
    );
    let mut seen = std::collections::HashSet::new();
    let issues = validate_corpus_entry(&entry, &mut seen);
    assert!(issues.iter().any(|i| i.contains("Empty expected_output")));
}

#[test]
fn test_validate_corpus_entry_bash_missing_fn_main() {
    use super::corpus_analysis_commands::validate_corpus_entry;
    let entry = CorpusEntry::new(
        "B-001",
        "test",
        "desc",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "let x = 5;",
        "echo hello",
    );
    let mut seen = std::collections::HashSet::new();
    let issues = validate_corpus_entry(&entry, &mut seen);
    assert!(issues.iter().any(|i| i.contains("fn main()")));
}

#[test]
fn test_validate_corpus_entry_dockerfile_valid() {
    use super::corpus_analysis_commands::validate_corpus_entry;
    let entry = CorpusEntry::new(
        "D-001",
        "docker-test",
        "desc",
        CorpusFormat::Dockerfile,
        CorpusTier::Standard,
        "let x = 5;",
        "FROM alpine",
    );
    let mut seen = std::collections::HashSet::new();
    let issues = validate_corpus_entry(&entry, &mut seen);
    // Dockerfiles don't need fn main()
    assert!(issues.is_empty(), "Got issues: {:?}", issues);
}

#[test]
fn test_validate_corpus_entry_multiple_issues() {
    use super::corpus_analysis_commands::validate_corpus_entry;
    let entry = CorpusEntry::new(
        "X-001",
        "",
        "",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "",
        "",
    );
    let mut seen = std::collections::HashSet::new();
    let issues = validate_corpus_entry(&entry, &mut seen);
    // Should have: wrong prefix, empty name, empty description, empty input, empty expected_output, missing fn main()
    assert!(
        issues.len() >= 5,
        "Expected >= 5 issues, got {}: {:?}",
        issues.len(),
        issues
    );
}

// ── CorpusTier tests ────────────────────────────────────────────────────────

#[test]
fn test_corpus_tier_weight() {
    assert!((CorpusTier::Trivial.weight() - 1.0).abs() < 0.01);
    assert!((CorpusTier::Standard.weight() - 1.5).abs() < 0.01);
    assert!((CorpusTier::Complex.weight() - 2.0).abs() < 0.01);
    assert!((CorpusTier::Adversarial.weight() - 2.5).abs() < 0.01);
    assert!((CorpusTier::Production.weight() - 3.0).abs() < 0.01);
}

#[test]
fn test_corpus_tier_target_rate() {
    assert!((CorpusTier::Trivial.target_rate() - 1.0).abs() < 0.01);
    assert!((CorpusTier::Standard.target_rate() - 0.99).abs() < 0.01);
    assert!((CorpusTier::Complex.target_rate() - 0.98).abs() < 0.01);
    assert!((CorpusTier::Adversarial.target_rate() - 0.95).abs() < 0.01);
    assert!((CorpusTier::Production.target_rate() - 0.95).abs() < 0.01);
}

// ── CorpusFormat display tests ──────────────────────────────────────────────

#[test]
fn test_corpus_format_display() {
    assert_eq!(CorpusFormat::Bash.to_string(), "bash");
    assert_eq!(CorpusFormat::Makefile.to_string(), "makefile");
    assert_eq!(CorpusFormat::Dockerfile.to_string(), "dockerfile");
}

// ── CorpusEntry creation tests ──────────────────────────────────────────────

#[test]
fn test_corpus_entry_new_defaults() {
    let entry = CorpusEntry::new(
        "B-001",
        "test",
        "desc",
        CorpusFormat::Bash,
        CorpusTier::Standard,
        "fn main() {}",
        "echo hello",
    );
    assert_eq!(entry.id, "B-001");
    assert!(entry.shellcheck); // bash entries get shellcheck=true
    assert!(entry.deterministic);
    assert!(entry.idempotent);
}

#[test]
fn test_corpus_entry_new_makefile_no_shellcheck() {
    let entry = CorpusEntry::new(
        "M-001",
        "make-test",
        "desc",
        CorpusFormat::Makefile,
        CorpusTier::Standard,
        "let x = 5;",
        "X := 5",
    );
    assert!(!entry.shellcheck); // non-bash entries get shellcheck=false
    assert!(entry.deterministic);
}

// ── CorpusRegistry tests ────────────────────────────────────────────────────

#[test]
fn test_corpus_registry_new_empty() {
    let registry = crate::corpus::registry::CorpusRegistry::new();
    assert!(registry.entries.is_empty());
}

#[test]
fn test_corpus_registry_add_and_by_format() {
    let mut registry = crate::corpus::registry::CorpusRegistry::new();
    registry.add(mock_entry(
        "B-001",
        "t1",
        CorpusFormat::Bash,
        CorpusTier::Standard,
    ));
    registry.add(mock_entry(
        "M-001",
        "t2",
        CorpusFormat::Makefile,
        CorpusTier::Standard,
    ));
    registry.add(mock_entry(
        "B-002",
        "t3",
        CorpusFormat::Bash,
        CorpusTier::Trivial,
    ));

    assert_eq!(registry.by_format(CorpusFormat::Bash).len(), 2);
    assert_eq!(registry.by_format(CorpusFormat::Makefile).len(), 1);
    assert_eq!(registry.by_format(CorpusFormat::Dockerfile).len(), 0);
}

#[test]
fn test_corpus_registry_by_tier() {
    let mut registry = crate::corpus::registry::CorpusRegistry::new();
    registry.add(mock_entry(
        "B-001",
        "t1",
        CorpusFormat::Bash,
        CorpusTier::Standard,
    ));
    registry.add(mock_entry(
        "B-002",
        "t2",
        CorpusFormat::Bash,
        CorpusTier::Trivial,
    ));
    registry.add(mock_entry(
        "B-003",
        "t3",
        CorpusFormat::Bash,
        CorpusTier::Standard,
    ));

    assert_eq!(registry.by_tier(CorpusTier::Standard).len(), 2);
    assert_eq!(registry.by_tier(CorpusTier::Trivial).len(), 1);
    assert_eq!(registry.by_tier(CorpusTier::Complex).len(), 0);
}

// ── ConvergenceEntry tests ──────────────────────────────────────────────────

#[test]
fn test_convergence_entry_default() {
    let e = ConvergenceEntry::default();
    assert_eq!(e.iteration, 0);
    assert_eq!(e.total, 0);
    assert!((e.score - 0.0).abs() < 0.01);
    assert!(e.grade.is_empty());
}

#[test]
fn test_convergence_entry_serialization_roundtrip() {
    let e = ConvergenceEntry {
        iteration: 42,
        date: "2025-06-15".to_string(),
        total: 1000,
        passed: 999,
        failed: 1,
        rate: 0.999,
        delta: 0.001,
        notes: "test run".to_string(),
        bash_passed: 500,
        bash_total: 500,
        makefile_passed: 300,
        makefile_total: 300,
        dockerfile_passed: 199,
        dockerfile_total: 200,
        score: 99.2,
        grade: "A+".to_string(),
        bash_score: 99.5,
        makefile_score: 100.0,
        dockerfile_score: 98.0,
        lint_passed: 998,
        lint_rate: 0.998,
    };
    let json = serde_json::to_string(&e).unwrap();
    let parsed: ConvergenceEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.iteration, 42);
    assert_eq!(parsed.total, 1000);
    assert!((parsed.score - 99.2).abs() < 0.01);
    assert_eq!(parsed.grade, "A+");
}

// ── CorpusResult score edge cases ───────────────────────────────────────────

#[test]
fn test_corpus_result_score_v1_all_pass() {
    let r = mock_result("B-001", true);
    let v1 = r.score_v1();
    // A(40) + B(25) + C(0.95*15=14.25) + D(10) + E(10) = 99.25
    assert!((v1 - 99.25).abs() < 0.01, "V1 expected 99.25, got {v1}");
}

#[test]
fn test_corpus_result_score_v1_fail() {
    let r = mock_result("B-001", false);
    assert!((r.score_v1() - 0.0).abs() < 0.01);
}

#[test]
fn test_corpus_result_score_transpiled_but_not_contains() {
    let mut r = mock_result("B-001", true);
    r.output_contains = false;
    let s = r.score();
    // A(30) + B1(0) + B2(0, gated by B1) + B3(0, gated by B1) + C(14.25) + D(10) + E(10) + F(5) + G(5) = 74.25
    assert!((s - 74.25).abs() < 0.01, "Expected 74.25, got {s}");
}

#[test]
fn test_corpus_result_score_contains_but_not_exact() {
    let mut r = mock_result("B-001", true);
    r.output_exact = false;
    let s = r.score();
    // A(30) + B1(10) + B2(0) + B3(7) + C(14.25) + D(10) + E(10) + F(5) + G(5) = 91.25
    assert!((s - 91.25).abs() < 0.01, "Expected 91.25, got {s}");
}

#[test]
fn test_corpus_result_default() {
    let r = CorpusResult::default();
    assert!(!r.transpiled);
    assert!((r.score() - 0.0).abs() < 0.01);
    assert!(r.id.is_empty());
}
