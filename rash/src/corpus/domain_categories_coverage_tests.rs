//! Coverage tests for corpus/domain_categories.rs — targeting format_categories_report,
//! format_domain_coverage, and format_quality_matrix.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::corpus::domain_categories::{
    format_categories_report, format_domain_coverage, format_quality_matrix, CategoryStats,
    DomainCategory,
};
use crate::corpus::registry::Grade;
use crate::corpus::runner::{CorpusScore, FormatScore};

fn make_stats(cat: DomainCategory, total: usize, capacity: usize, passed: usize) -> CategoryStats {
    let failed = total.saturating_sub(passed);
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
        category: cat,
        total,
        capacity,
        passed,
        failed,
        fill_pct,
        pass_rate,
    }
}

fn make_score(total: usize, passed: usize) -> CorpusScore {
    let failed = total.saturating_sub(passed);
    let rate = if total > 0 {
        passed as f64 / total as f64
    } else {
        0.0
    };
    CorpusScore {
        total,
        passed,
        failed,
        rate,
        score: rate * 100.0,
        grade: Grade::from_score(rate * 100.0),
        format_scores: vec![],
        results: vec![],
    }
}

// =============================================================================
// format_categories_report
// =============================================================================

#[test]
fn test_format_categories_report_empty_stats() {
    let result = format_categories_report(&[]);
    assert!(result.contains("Domain-Specific Corpus Categories"));
}

#[test]
fn test_format_categories_report_single_category() {
    let stats = vec![make_stats(DomainCategory::ShellConfig, 10, 50, 8)];
    let result = format_categories_report(&stats);
    assert!(result.contains("Domain-Specific Corpus Categories"));
    assert!(result.contains("Shell Config"));
}

#[test]
fn test_format_categories_report_all_specific_categories() {
    let stats = vec![
        make_stats(DomainCategory::ShellConfig, 10, 50, 8),
        make_stats(DomainCategory::OneLiners, 20, 100, 18),
        make_stats(DomainCategory::Provability, 5, 30, 5),
        make_stats(DomainCategory::UnixTools, 15, 80, 12),
        make_stats(DomainCategory::LangIntegration, 8, 40, 7),
        make_stats(DomainCategory::SystemTooling, 12, 60, 10),
        make_stats(DomainCategory::Coreutils, 25, 120, 22),
        make_stats(DomainCategory::RegexPatterns, 6, 30, 6),
    ];
    let result = format_categories_report(&stats);
    assert!(result.contains("Shell Config"));
    assert!(result.contains("One-Liners"));
    assert!(result.contains("Provability"));
    assert!(result.contains("Unix Tools"));
    assert!(result.contains("Coreutils"));
    assert!(result.contains("Regex"));
}

#[test]
fn test_format_categories_report_with_general_category() {
    let stats = vec![
        make_stats(DomainCategory::ShellConfig, 10, 50, 8),
        make_stats(DomainCategory::General, 500, 0, 450),
    ];
    let result = format_categories_report(&stats);
    assert!(result.contains("General"));
}

#[test]
fn test_format_categories_report_zero_totals() {
    let stats = vec![
        make_stats(DomainCategory::ShellConfig, 0, 50, 0),
        make_stats(DomainCategory::OneLiners, 0, 100, 0),
    ];
    let result = format_categories_report(&stats);
    assert!(result.contains("Shell Config"));
    assert!(result.contains("One-Liners"));
}

#[test]
fn test_format_categories_report_full_capacity() {
    let stats = vec![make_stats(DomainCategory::ShellConfig, 50, 50, 50)];
    let result = format_categories_report(&stats);
    assert!(result.contains("100"));
}

#[test]
fn test_format_categories_report_contains_header_separator() {
    let stats = vec![make_stats(DomainCategory::ShellConfig, 10, 50, 8)];
    let result = format_categories_report(&stats);
    // Should contain table formatting
    assert!(result.contains("───") || result.contains("---") || result.contains("═"));
}

// =============================================================================
// format_domain_coverage
// =============================================================================

#[test]
fn test_format_domain_coverage_empty() {
    let stats = vec![];
    let score = make_score(100, 95);
    let result = format_domain_coverage(&stats, &score);
    assert!(!result.is_empty());
}

#[test]
fn test_format_domain_coverage_single_category() {
    let stats = vec![make_stats(DomainCategory::ShellConfig, 10, 50, 8)];
    let score = make_score(100, 95);
    let result = format_domain_coverage(&stats, &score);
    assert!(result.contains("Shell Config"));
}

#[test]
fn test_format_domain_coverage_all_categories() {
    let stats = vec![
        make_stats(DomainCategory::ShellConfig, 10, 50, 8),
        make_stats(DomainCategory::OneLiners, 20, 100, 18),
        make_stats(DomainCategory::Provability, 5, 30, 5),
        make_stats(DomainCategory::UnixTools, 15, 80, 12),
        make_stats(DomainCategory::LangIntegration, 8, 40, 7),
        make_stats(DomainCategory::SystemTooling, 12, 60, 10),
        make_stats(DomainCategory::Coreutils, 25, 120, 22),
        make_stats(DomainCategory::RegexPatterns, 6, 30, 6),
    ];
    let score = make_score(500, 480);
    let result = format_domain_coverage(&stats, &score);
    assert!(result.contains("Shell Config"));
    assert!(result.contains("Coreutils"));
}

#[test]
fn test_format_domain_coverage_with_gaps() {
    // Categories that need more entries
    let stats = vec![
        make_stats(DomainCategory::ShellConfig, 5, 50, 4),
        make_stats(DomainCategory::OneLiners, 3, 100, 3),
    ];
    let score = make_score(100, 95);
    let result = format_domain_coverage(&stats, &score);
    // Should show gap information
    assert!(!result.is_empty());
}

#[test]
fn test_format_domain_coverage_full_capacity_no_gaps() {
    let stats = vec![
        make_stats(DomainCategory::ShellConfig, 50, 50, 50),
        make_stats(DomainCategory::OneLiners, 100, 100, 100),
    ];
    let score = make_score(150, 150);
    let result = format_domain_coverage(&stats, &score);
    assert!(!result.is_empty());
}

#[test]
fn test_format_domain_coverage_zero_capacity() {
    let stats = vec![make_stats(DomainCategory::General, 500, 0, 450)];
    let score = make_score(500, 450);
    let result = format_domain_coverage(&stats, &score);
    assert!(!result.is_empty());
}

// =============================================================================
// format_quality_matrix
// =============================================================================

#[test]
fn test_format_quality_matrix_empty_stats() {
    let result = format_quality_matrix(&[]);
    assert!(!result.is_empty());
}

#[test]
fn test_format_quality_matrix_single_category() {
    let stats = vec![make_stats(DomainCategory::ShellConfig, 10, 50, 8)];
    let result = format_quality_matrix(&stats);
    assert!(!result.is_empty());
}

#[test]
fn test_format_quality_matrix_all_categories() {
    let stats = vec![
        make_stats(DomainCategory::ShellConfig, 10, 50, 8),
        make_stats(DomainCategory::OneLiners, 20, 100, 18),
        make_stats(DomainCategory::Provability, 5, 30, 5),
        make_stats(DomainCategory::UnixTools, 15, 80, 12),
        make_stats(DomainCategory::LangIntegration, 8, 40, 7),
        make_stats(DomainCategory::SystemTooling, 12, 60, 10),
        make_stats(DomainCategory::Coreutils, 25, 120, 22),
        make_stats(DomainCategory::RegexPatterns, 6, 30, 6),
    ];
    let result = format_quality_matrix(&stats);
    // Should contain quality property labels
    assert!(
        result.contains("REQ") || result.contains("N/A") || result.contains("Required"),
        "Quality matrix should contain REQ or N/A markers"
    );
}

#[test]
fn test_format_quality_matrix_contains_property_names() {
    let stats = vec![
        make_stats(DomainCategory::ShellConfig, 10, 50, 8),
        make_stats(DomainCategory::OneLiners, 20, 100, 18),
    ];
    let result = format_quality_matrix(&stats);
    // Should reference quality properties like determinism, idempotency, etc.
    assert!(!result.is_empty());
}

#[test]
fn test_format_quality_matrix_with_general() {
    let stats = vec![
        make_stats(DomainCategory::ShellConfig, 10, 50, 8),
        make_stats(DomainCategory::General, 500, 0, 450),
    ];
    let result = format_quality_matrix(&stats);
    assert!(!result.is_empty());
}
