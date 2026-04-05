fn test_publish_readiness_all_pass() {
    let score = make_score(500, 500, 0, 1.0, 99.0, Grade::APlus, 3);
    let checks = check_publish_readiness(&score);
    assert!(checks.iter().all(|c| c.passed));
}

#[test]
fn test_publish_readiness_rate_below_99() {
    let score = make_score(100, 95, 5, 0.95, 92.0, Grade::A, 3);
    let checks = check_publish_readiness(&score);
    let rate_check = &checks[0];
    assert!(!rate_check.passed);
    assert!(rate_check.value.contains("95.0"));
}

#[test]
fn test_publish_readiness_score_below_90() {
    let score = make_score(200, 200, 0, 1.0, 85.0, Grade::B, 3);
    let checks = check_publish_readiness(&score);
    let score_check = &checks[1];
    assert!(!score_check.passed);
}

#[test]
fn test_publish_readiness_missing_formats() {
    let score = make_score(200, 200, 0, 1.0, 99.0, Grade::APlus, 2);
    let checks = check_publish_readiness(&score);
    let format_check = &checks[2];
    assert!(!format_check.passed);
}

#[test]
fn test_publish_readiness_has_failures() {
    let score = make_score(200, 195, 5, 0.975, 92.0, Grade::A, 3);
    let checks = check_publish_readiness(&score);
    let fail_check = &checks[3];
    assert!(!fail_check.passed);
    assert!(fail_check.value.contains("5"));
}

#[test]
fn test_publish_readiness_corpus_too_small() {
    let score = make_score(50, 50, 0, 1.0, 99.0, Grade::APlus, 3);
    let checks = check_publish_readiness(&score);
    let size_check = &checks[4];
    assert!(!size_check.passed);
}

// === format_publish_checks ===

#[test]
fn test_format_publish_checks_all_pass() {
    let checks = vec![
        PublishCheck {
            name: "Check A",
            passed: true,
            value: "ok".to_string(),
        },
        PublishCheck {
            name: "Check B",
            passed: true,
            value: "ok".to_string(),
        },
    ];
    let table = format_publish_checks(&checks);
    assert!(table.contains("PASS"));
    assert!(table.contains("Ready to publish"));
    assert!(!table.contains("FAIL"));
}

#[test]
fn test_format_publish_checks_mixed() {
    let checks = vec![
        PublishCheck {
            name: "Good check",
            passed: true,
            value: "ok".to_string(),
        },
        PublishCheck {
            name: "Bad check 1",
            passed: false,
            value: "bad".to_string(),
        },
        PublishCheck {
            name: "Bad check 2",
            passed: false,
            value: "bad".to_string(),
        },
    ];
    let table = format_publish_checks(&checks);
    assert!(table.contains("PASS"));
    assert!(table.contains("FAIL"));
    assert!(table.contains("2 check(s) failed"));
    assert!(!table.contains("Ready to publish"));
}

// === dataset_info ===

#[test]
fn test_dataset_info_empty_registry() {
    let registry = CorpusRegistry { entries: vec![] };
    let info = dataset_info(&registry);
    assert_eq!(info.total_entries, 0);
    for (_fmt, count) in &info.format_counts {
        assert_eq!(*count, 0);
    }
}

#[test]
fn test_dataset_info_all_formats() {
    let entries = vec![
        make_entry("B-001", CorpusFormat::Bash),
        make_entry("B-002", CorpusFormat::Bash),
        make_entry("M-001", CorpusFormat::Makefile),
        make_entry("D-001", CorpusFormat::Dockerfile),
        make_entry("D-002", CorpusFormat::Dockerfile),
        make_entry("D-003", CorpusFormat::Dockerfile),
    ];
    let registry = CorpusRegistry { entries };
    let info = dataset_info(&registry);
    assert_eq!(info.total_entries, 6);
    assert_eq!(info.format_counts.len(), 3);
    // Bash=2, Makefile=1, Dockerfile=3
    assert_eq!(info.format_counts[0], ("bash".to_string(), 2));
    assert_eq!(info.format_counts[1], ("makefile".to_string(), 1));
    assert_eq!(info.format_counts[2], ("dockerfile".to_string(), 3));
}

#[test]
fn test_dataset_info_schema_fields() {
    let registry = CorpusRegistry { entries: vec![] };
    let info = dataset_info(&registry);
    assert_eq!(info.schema_fields.len(), 18);
    // Verify first and last fields
    assert_eq!(info.schema_fields[0].0, "id");
    assert_eq!(info.schema_fields[17].0, "date");
}

// === format_dataset_info ===

#[test]
fn test_format_dataset_info_contains_structure() {
    let entries = vec![
        make_entry("B-001", CorpusFormat::Bash),
        make_entry("M-001", CorpusFormat::Makefile),
    ];
    let registry = CorpusRegistry { entries };
    let info = dataset_info(&registry);
    let table = format_dataset_info(&info);
    assert!(table.contains("Corpus: 2 entries"));
    assert!(table.contains("Dataset Schema"));
    assert!(table.contains("Field"));
    assert!(table.contains("Type"));
    assert!(table.contains("Description"));
    assert!(table.contains("Export formats: json, jsonl, csv"));
}

#[test]
fn test_format_dataset_info_shows_format_counts() {
    let entries = vec![
        make_entry("B-001", CorpusFormat::Bash),
        make_entry("B-002", CorpusFormat::Bash),
    ];
    let registry = CorpusRegistry { entries };
    let info = dataset_info(&registry);
    let table = format_dataset_info(&info);
    assert!(table.contains("bash"));
    assert!(table.contains("2 entries"));
}

// === days_to_ymd edge cases ===

#[test]
fn test_days_to_ymd_epoch() {
    assert_eq!(days_to_ymd(0), (1970, 1, 1));
}

#[test]
fn test_days_to_ymd_day_one() {
    assert_eq!(days_to_ymd(1), (1970, 1, 2));
}

#[test]
fn test_days_to_ymd_end_of_january() {
    assert_eq!(days_to_ymd(30), (1970, 1, 31));
}

#[test]
fn test_days_to_ymd_february_1() {
    assert_eq!(days_to_ymd(31), (1970, 2, 1));
}

#[test]
fn test_days_to_ymd_dec_31_non_leap() {
    // 1970 is not a leap year: 365 days, so day 364 = Dec 31, 1970
    assert_eq!(days_to_ymd(364), (1970, 12, 31));
}

#[test]
fn test_days_to_ymd_jan_1_next_year() {
    // Day 365 = Jan 1, 1971
    assert_eq!(days_to_ymd(365), (1971, 1, 1));
}

#[test]
fn test_days_to_ymd_leap_year_feb_29() {
    // 1972 is the first leap year after epoch
    // Days from 1970-01-01 to 1972-02-29:
    // 1970: 365 days, 1971: 365 days = 730 days to 1972-01-01
    // Jan: 31, Feb 1-29: 28 more = 59 days into 1972
    // So day 730 + 59 - 1 = 788
    assert_eq!(days_to_ymd(789), (1972, 2, 29));
}

#[test]
fn test_days_to_ymd_leap_year_mar_1() {
    assert_eq!(days_to_ymd(790), (1972, 3, 1));
}

#[test]
fn test_days_to_ymd_year_2000_leap() {
    // 2000 is a leap year (divisible by 400)
    // Known: 2000-01-01 = day 10957
    assert_eq!(days_to_ymd(10957), (2000, 1, 1));
}

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
