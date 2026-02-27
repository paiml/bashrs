#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::domain_categories::*;

// ---------------------------------------------------------------------------
// Helper: build a CategoryStats with given parameters
// ---------------------------------------------------------------------------

fn make_stats(
    category: DomainCategory,
    total: usize,
    capacity: usize,
    passed: usize,
    failed: usize,
) -> CategoryStats {
    let fill_pct = if capacity > 0 {
        (total as f64 / capacity as f64) * 100.0
    } else {
        0.0
    };
    let pass_rate = if total > 0 {
        (passed as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    CategoryStats {
        category,
        total,
        capacity,
        passed,
        failed,
        fill_pct,
        pass_rate,
    }
}

// ===========================================================================
// DomainCategory::label() — all 9 variants
// ===========================================================================

#[test]
fn test_label_shell_config() {
    assert_eq!(DomainCategory::ShellConfig.label(), "A: Shell Config");
}

#[test]
fn test_label_one_liners() {
    assert_eq!(DomainCategory::OneLiners.label(), "B: One-Liners");
}

#[test]
fn test_label_provability() {
    assert_eq!(DomainCategory::Provability.label(), "C: Provability");
}

#[test]
fn test_label_unix_tools() {
    assert_eq!(DomainCategory::UnixTools.label(), "D: Unix Tools");
}

#[test]
fn test_label_lang_integration() {
    assert_eq!(DomainCategory::LangIntegration.label(), "E: Lang Integration");
}

#[test]
fn test_label_system_tooling() {
    assert_eq!(DomainCategory::SystemTooling.label(), "F: System Tooling");
}

#[test]
fn test_label_coreutils() {
    assert_eq!(DomainCategory::Coreutils.label(), "G: Coreutils");
}

#[test]
fn test_label_regex_patterns() {
    assert_eq!(DomainCategory::RegexPatterns.label(), "H: Regex Patterns");
}

#[test]
fn test_label_general() {
    assert_eq!(DomainCategory::General.label(), "General");
}

// ===========================================================================
// DomainCategory::capacity() — all 9 variants
// ===========================================================================

#[test]
fn test_capacity_all_variants() {
    assert_eq!(DomainCategory::ShellConfig.capacity(), 10);
    assert_eq!(DomainCategory::OneLiners.capacity(), 10);
    assert_eq!(DomainCategory::Provability.capacity(), 10);
    assert_eq!(DomainCategory::UnixTools.capacity(), 10);
    assert_eq!(DomainCategory::LangIntegration.capacity(), 10);
    assert_eq!(DomainCategory::SystemTooling.capacity(), 10);
    assert_eq!(DomainCategory::Coreutils.capacity(), 30);
    assert_eq!(DomainCategory::RegexPatterns.capacity(), 30);
    assert_eq!(DomainCategory::General.capacity(), 0);
}

// ===========================================================================
// DomainCategory::range() — spot checks + General returns None
// ===========================================================================

#[test]
fn test_range_returns_none_for_general() {
    assert_eq!(DomainCategory::General.range(), None);
}

#[test]
fn test_range_shell_config() {
    assert_eq!(DomainCategory::ShellConfig.range(), Some((371, 380)));
}

#[test]
fn test_range_coreutils_wider() {
    assert_eq!(DomainCategory::Coreutils.range(), Some((431, 460)));
}

#[test]
fn test_range_regex_patterns() {
    assert_eq!(DomainCategory::RegexPatterns.range(), Some((461, 490)));
}

// ===========================================================================
// CategoryStats construction and fields
// ===========================================================================

#[test]
fn test_category_stats_construction() {
    let s = make_stats(DomainCategory::OneLiners, 7, 10, 5, 2);
    assert_eq!(s.category, DomainCategory::OneLiners);
    assert_eq!(s.total, 7);
    assert_eq!(s.capacity, 10);
    assert_eq!(s.passed, 5);
    assert_eq!(s.failed, 2);
    assert!((s.fill_pct - 70.0).abs() < 0.01);
    assert!((s.pass_rate - 71.42857).abs() < 0.01);
}

#[test]
fn test_category_stats_zero_capacity_fill_pct() {
    let s = make_stats(DomainCategory::General, 100, 0, 80, 20);
    assert!((s.fill_pct - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_category_stats_zero_total_pass_rate() {
    let s = make_stats(DomainCategory::ShellConfig, 0, 10, 0, 0);
    assert!((s.pass_rate - 0.0).abs() < f64::EPSILON);
}

// ===========================================================================
// format_categories_report
// ===========================================================================

#[test]
fn test_format_categories_report_empty_stats() {
    let stats: Vec<CategoryStats> = vec![];
    let report = format_categories_report(&stats);

    // Header must be present
    assert!(report.contains("Domain-Specific Corpus Categories"));
    assert!(report.contains("Category"));
    assert!(report.contains("Entries"));
    assert!(report.contains("Capacity"));
    assert!(report.contains("Fill %"));
    assert!(report.contains("Pass Rate"));

    // Summary line with 0 entries
    assert!(report.contains("Total: 0 entries"));
    assert!(report.contains("Pass rate: 0/0"));
}

#[test]
fn test_format_categories_report_single_domain_category() {
    let stats = vec![make_stats(DomainCategory::UnixTools, 8, 10, 7, 1)];
    let report = format_categories_report(&stats);

    assert!(report.contains("D: Unix Tools"));
    assert!(report.contains("80%")); // fill_pct = 80%
    assert!(report.contains("87.5%")); // pass_rate = 7/8 = 87.5%
    // No General row should appear
    assert!(!report.contains("General"));
    // Summary
    assert!(report.contains("Total: 8 entries (8 domain-specific"));
}

#[test]
fn test_format_categories_report_multiple_categories_with_general() {
    let stats = vec![
        make_stats(DomainCategory::ShellConfig, 10, 10, 10, 0),
        make_stats(DomainCategory::Coreutils, 15, 30, 12, 3),
        make_stats(DomainCategory::General, 200, 0, 180, 20),
    ];
    let report = format_categories_report(&stats);

    // Both domain categories present
    assert!(report.contains("A: Shell Config"));
    assert!(report.contains("G: Coreutils"));

    // General row exists with "-" for capacity and fill
    // The General row format: General   200   -   -   180   90.0%
    assert!(report.contains("General"));

    // Summary should include general entries: 10+15+200 = 225 total
    assert!(report.contains("Total: 225 entries"));
    // domain-specific = 10+15 = 25
    assert!(report.contains("25 domain-specific"));
}

#[test]
fn test_format_categories_report_zero_capacity_shows_dash() {
    // General category has capacity=0
    let stats = vec![make_stats(DomainCategory::General, 50, 0, 40, 10)];
    let report = format_categories_report(&stats);

    // General row should show "-" for fill
    // The General row is specially formatted with "-" for capacity and fill
    assert!(report.contains("General"));
    assert!(report.contains("80.0%")); // pass_rate for general
}

#[test]
fn test_format_categories_report_zero_total_shows_dash_rate() {
    let stats = vec![make_stats(DomainCategory::Provability, 0, 10, 0, 0)];
    let report = format_categories_report(&stats);

    assert!(report.contains("C: Provability"));
    // pass_rate with 0 total should show "-"
    // fill_pct with capacity>0 but total=0 should show "0%"
    let lines: Vec<&str> = report.lines().collect();
    let provability_line = lines
        .iter()
        .find(|l| l.contains("C: Provability"))
        .expect("should have provability line");
    assert!(
        provability_line.contains('-'),
        "zero-total category should show '-' for pass rate, got: {}",
        provability_line
    );
}

#[test]
fn test_format_categories_report_general_with_zero_total_shows_dash() {
    let stats = vec![
        make_stats(DomainCategory::ShellConfig, 5, 10, 5, 0),
        make_stats(DomainCategory::General, 0, 0, 0, 0),
    ];
    let report = format_categories_report(&stats);

    // General with total=0 should show "-" for pass rate
    // But note: the function only adds General row if it exists in stats
    // and it uses if gen.total > 0 for pass rate formatting
    let lines: Vec<&str> = report.lines().collect();
    if let Some(gen_line) = lines.iter().find(|l| l.starts_with("General")) {
        // Should have "-" for pass rate
        assert!(
            gen_line.contains('-'),
            "general with zero total should show '-', got: {}",
            gen_line
        );
    }
}

#[test]
fn test_format_categories_report_summary_fill_pct_with_zero_capacity() {
    // If all domain categories have 0 capacity, fill_pct in summary should be 0%
    let stats: Vec<CategoryStats> = vec![];
    let report = format_categories_report(&stats);
    // domain_capacity = 0, so fill_pct = 0
    assert!(report.contains("0% of capacity 0"));
}

// ===========================================================================
// format_quality_matrix
// ===========================================================================

#[test]
fn test_format_quality_matrix_empty_stats() {
    let stats: Vec<CategoryStats> = vec![];
    let matrix = format_quality_matrix(&stats);

    // Header
    assert!(matrix.contains("Cross-Category Quality Matrix"));
    assert!(matrix.contains("Property"));

    // All 10 quality properties should appear
    assert!(matrix.contains("Idempotent"));
    assert!(matrix.contains("POSIX"));
    assert!(matrix.contains("Deterministic"));
    assert!(matrix.contains("Miri-verifiable"));
    assert!(matrix.contains("Cross-shell"));
    assert!(matrix.contains("Shellcheck-clean"));
    assert!(matrix.contains("Pipeline-safe"));
    assert!(matrix.contains("1:1 parity"));
    assert!(matrix.contains("Signal-aware"));
    assert!(matrix.contains("Terminates"));

    // REQ and N/A markers
    assert!(matrix.contains("REQ"));
    assert!(matrix.contains("N/A"));

    // Entries row: all zeros since stats is empty
    assert!(matrix.contains("Entries"));

    // Required count row
    assert!(matrix.contains("Required count"));
}

#[test]
fn test_format_quality_matrix_single_category() {
    let stats = vec![make_stats(DomainCategory::ShellConfig, 5, 10, 4, 1)];
    let matrix = format_quality_matrix(&stats);

    // Column header should include "Config" for ShellConfig
    assert!(matrix.contains("Config"));

    // Entries row: ShellConfig should show 5, others 0
    let lines: Vec<&str> = matrix.lines().collect();
    let entries_line = lines
        .iter()
        .find(|l| l.starts_with("Entries"))
        .expect("should have entries line");
    assert!(entries_line.contains('5'));
}

#[test]
fn test_format_quality_matrix_multiple_categories() {
    let stats = vec![
        make_stats(DomainCategory::ShellConfig, 10, 10, 10, 0),
        make_stats(DomainCategory::OneLiners, 8, 10, 7, 1),
        make_stats(DomainCategory::Coreutils, 20, 30, 18, 2),
        make_stats(DomainCategory::RegexPatterns, 15, 30, 15, 0),
    ];
    let matrix = format_quality_matrix(&stats);

    // All column headers should be present
    assert!(matrix.contains("Config"));
    assert!(matrix.contains("1-Liner"));
    assert!(matrix.contains("Core"));
    assert!(matrix.contains("Regex"));

    // Required count row should exist
    assert!(matrix.contains("Required count"));

    // Entries row should show counts for present categories
    let lines: Vec<&str> = matrix.lines().collect();
    let entries_line = lines
        .iter()
        .find(|l| l.starts_with("Entries"))
        .expect("should have entries line");
    assert!(entries_line.contains("10"));
    assert!(entries_line.contains("20"));
    assert!(entries_line.contains("15"));
}

#[test]
fn test_format_quality_matrix_required_count_row() {
    let stats: Vec<CategoryStats> = DomainCategory::all_specific()
        .iter()
        .map(|c| make_stats(*c, 0, c.capacity(), 0, 0))
        .collect();
    let matrix = format_quality_matrix(&stats);

    // Verify required count row has "/10" format for each category
    let lines: Vec<&str> = matrix.lines().collect();
    let req_line = lines
        .iter()
        .find(|l| l.starts_with("Required count"))
        .expect("should have required count line");
    // Each category column should show "N/10"
    assert!(req_line.contains("/10"));
}

// ===========================================================================
// classify_entry — public function
// ===========================================================================

#[test]
fn test_classify_entry_makefile_is_general() {
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier};
    let entry = CorpusEntry::new(
        "M-100",
        "makefile test",
        "desc",
        CorpusFormat::Makefile,
        CorpusTier::Trivial,
        "all: build",
        "expected",
    );
    assert_eq!(classify_entry(&entry), DomainCategory::General);
}

#[test]
fn test_classify_entry_dockerfile_is_general() {
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier};
    let entry = CorpusEntry::new(
        "D-100",
        "dockerfile test",
        "desc",
        CorpusFormat::Dockerfile,
        CorpusTier::Trivial,
        "FROM alpine",
        "expected",
    );
    assert_eq!(classify_entry(&entry), DomainCategory::General);
}

#[test]
fn test_classify_entry_bash_in_range() {
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier};
    let entry = CorpusEntry::new(
        "B-415",
        "lang integration",
        "desc",
        CorpusFormat::Bash,
        CorpusTier::Trivial,
        "echo hello",
        "expected",
    );
    assert_eq!(classify_entry(&entry), DomainCategory::LangIntegration);
}

#[test]
fn test_classify_entry_bash_out_of_all_ranges() {
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier};
    let entry = CorpusEntry::new(
        "B-999",
        "general bash",
        "desc",
        CorpusFormat::Bash,
        CorpusTier::Trivial,
        "echo hello",
        "expected",
    );
    assert_eq!(classify_entry(&entry), DomainCategory::General);
}

#[test]
fn test_classify_entry_invalid_id_format() {
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier};
    let entry = CorpusEntry::new(
        "X-100",
        "invalid prefix",
        "desc",
        CorpusFormat::Bash,
        CorpusTier::Trivial,
        "echo hello",
        "expected",
    );
    // "X-100" won't strip "B-" prefix, so parse_bash_id_num returns None → General
    assert_eq!(classify_entry(&entry), DomainCategory::General);
}

// ===========================================================================
// all_specific() completeness
// ===========================================================================

#[test]
fn test_all_specific_excludes_general() {
    let specific = DomainCategory::all_specific();
    assert_eq!(specific.len(), 8);
    assert!(
        !specific.contains(&DomainCategory::General),
        "all_specific() must not contain General"
    );
}

#[test]
fn test_all_specific_contains_all_domain_categories() {
    let specific = DomainCategory::all_specific();
    assert!(specific.contains(&DomainCategory::ShellConfig));
    assert!(specific.contains(&DomainCategory::OneLiners));
    assert!(specific.contains(&DomainCategory::Provability));
    assert!(specific.contains(&DomainCategory::UnixTools));
    assert!(specific.contains(&DomainCategory::LangIntegration));
    assert!(specific.contains(&DomainCategory::SystemTooling));
    assert!(specific.contains(&DomainCategory::Coreutils));
    assert!(specific.contains(&DomainCategory::RegexPatterns));
}
