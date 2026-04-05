#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for corpus/dataset.rs uncovered branches.
//!
//! Targets: grade boundary values, export edge cases (empty, multi-row),
//! CSV escaping combos, publish readiness boundary conditions, date
//! conversion (leap years, year boundaries, Dec 31), and formatting.

use super::*;
use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier, Grade};
use crate::corpus::runner::{CorpusResult, CorpusScore, FormatScore};

#[test]
fn test_days_to_ymd_year_2026() {
    // 2026-02-23 - verify a recent date
    let (y, m, d) = days_to_ymd(20_507);
    assert_eq!(y, 2026);
    assert_eq!(m, 2);
    assert_eq!(d, 23);
}

// === is_leap_year ===

#[test]
fn test_leap_year_standard() {
    assert!(is_leap_year(2024));
    assert!(is_leap_year(2028));
}

#[test]
fn test_leap_year_century_non_leap() {
    assert!(!is_leap_year(1900));
    assert!(!is_leap_year(2100));
}

#[test]
fn test_leap_year_400_year() {
    assert!(is_leap_year(2000));
    assert!(is_leap_year(1600));
}

#[test]
fn test_non_leap_year() {
    assert!(!is_leap_year(2025));
    assert!(!is_leap_year(2023));
}

// === ExportFormat Display ===

#[test]
fn test_export_format_display_all() {
    assert_eq!(format!("{}", ExportFormat::JsonLines), "jsonl");
    assert_eq!(format!("{}", ExportFormat::Csv), "csv");
    assert_eq!(format!("{}", ExportFormat::Json), "json");
}

#[test]
fn test_export_format_equality() {
    assert_eq!(ExportFormat::JsonLines, ExportFormat::JsonLines);
    assert_ne!(ExportFormat::JsonLines, ExportFormat::Csv);
    assert_ne!(ExportFormat::Csv, ExportFormat::Json);
}

// === DatasetRow serialization ===

#[test]
fn test_dataset_row_all_fields_serialize() {
    let row = DatasetRow {
        id: "B-999".into(),
        name: "edge-case".into(),
        tier: 4,
        format: "bash".into(),
        input_rust: "fn main() { println!(\"hello\"); }".into(),
        expected_output: "#!/bin/sh\necho hello\n".into(),
        actual_output: "#!/bin/sh\necho hello\n".into(),
        transpiled: true,
        output_correct: true,
        lint_clean: true,
        deterministic: true,
        score: 95.0,
        grade: "A".into(),
        safety_index: 0,
        safety_label: "safe".into(),
        bashrs_version: "7.0.0".into(),
        commit_sha: "deadbeef".into(),
        date: "2026-02-23".into(),
    };
    let json = serde_json::to_string(&row).unwrap();
    assert!(json.contains("B-999"));
    assert!(json.contains("deadbeef"));
    assert!(json.contains("7.0.0"));
}

// === CSV export with special characters in fields ===

#[test]
fn test_csv_export_row_with_commas_in_name() {
    let mut entry = make_entry("B-050", CorpusFormat::Bash);
    entry.name = "test, with comma".to_string();
    let row = build_row(&entry, None, "1.0.0", "abc", "2026-01-01");
    let output = export_csv(&[row]);
    // The name field should be quoted because it contains a comma
    assert!(output.contains("\"test, with comma\""));
}

// === current_date format ===

#[test]
fn test_current_date_iso8601() {
    let date = current_date();
    // Should be YYYY-MM-DD format
    assert_eq!(date.len(), 10);
    assert_eq!(&date[4..5], "-");
    assert_eq!(&date[7..8], "-");
    let year: u32 = date[0..4].parse().unwrap();
    let month: u32 = date[5..7].parse().unwrap();
    let day: u32 = date[8..10].parse().unwrap();
    assert!(year >= 2024);
    assert!((1..=12).contains(&month));
    assert!((1..=31).contains(&day));
}
