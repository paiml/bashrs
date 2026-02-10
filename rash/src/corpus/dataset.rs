//! # Dataset Export for Corpus Results (§10.3)
//!
//! Exports corpus results in portable formats (JSON, CSV) following the
//! Hugging Face dataset schema defined in §10.3. Enables reproducibility,
//! training data generation, and cross-project benchmarking.

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry};
use crate::corpus::runner::{CorpusResult, CorpusRunner, CorpusScore};
use crate::Config;
use std::fmt;

/// Export format for corpus data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON Lines (one JSON object per line)
    JsonLines,
    /// CSV with headers
    Csv,
    /// JSON array (pretty-printed)
    Json,
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JsonLines => write!(f, "jsonl"),
            Self::Csv => write!(f, "csv"),
            Self::Json => write!(f, "json"),
        }
    }
}

/// A single row in the exported dataset (§10.3 schema)
#[derive(Debug, Clone, serde::Serialize)]
pub struct DatasetRow {
    pub id: String,
    pub name: String,
    pub tier: u8,
    pub format: String,
    pub input_rust: String,
    pub expected_output: String,
    pub actual_output: String,
    pub transpiled: bool,
    pub output_correct: bool,
    pub lint_clean: bool,
    pub deterministic: bool,
    pub score: f64,
    pub grade: String,
    pub bashrs_version: String,
    pub commit_sha: String,
    pub date: String,
}

/// Dataset metadata for the `dataset-info` command
#[derive(Debug, Clone)]
pub struct DatasetInfo {
    pub total_entries: usize,
    pub format_counts: Vec<(String, usize)>,
    pub schema_fields: Vec<(&'static str, &'static str, &'static str)>,
    pub bashrs_version: String,
    pub date: String,
}

/// Build dataset rows from corpus entries and results
pub fn build_dataset(
    registry: &CorpusRegistry,
    score: &CorpusScore,
) -> Vec<DatasetRow> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let date = current_date();
    let commit = current_commit();

    registry
        .entries
        .iter()
        .map(|entry| {
            let result = score.results.iter().find(|r| r.id == entry.id);
            build_row(entry, result, &version, &commit, &date)
        })
        .collect()
}

fn build_row(
    entry: &CorpusEntry,
    result: Option<&CorpusResult>,
    version: &str,
    commit: &str,
    date: &str,
) -> DatasetRow {
    let (transpiled, output_correct, lint_clean, deterministic, actual, score_val, grade) =
        match result {
            Some(r) => (
                r.transpiled,
                r.output_contains && r.output_exact,
                r.lint_clean,
                r.deterministic,
                r.actual_output.clone().unwrap_or_default(),
                r.score(),
                score_to_grade(r.score()),
            ),
            None => (false, false, false, false, String::new(), 0.0, "F".into()),
        };

    DatasetRow {
        id: entry.id.clone(),
        name: entry.name.clone(),
        tier: entry.tier as u8,
        format: entry.format.to_string(),
        input_rust: entry.input.clone(),
        expected_output: entry.expected_output.clone(),
        actual_output: actual,
        transpiled,
        output_correct,
        lint_clean,
        deterministic,
        score: score_val,
        grade,
        bashrs_version: version.to_string(),
        commit_sha: commit.to_string(),
        date: date.to_string(),
    }
}

fn score_to_grade(score: f64) -> String {
    match score as u32 {
        97..=100 => "A+",
        93..=96 => "A",
        90..=92 => "A-",
        87..=89 => "B+",
        83..=86 => "B",
        80..=82 => "B-",
        77..=79 => "C+",
        73..=76 => "C",
        70..=72 => "C-",
        60..=69 => "D",
        _ => "F",
    }
    .to_string()
}

/// Export dataset rows as JSON Lines
pub fn export_jsonl(rows: &[DatasetRow]) -> String {
    rows.iter()
        .filter_map(|row| serde_json::to_string(row).ok())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Export dataset rows as JSON array (pretty-printed)
pub fn export_json(rows: &[DatasetRow]) -> String {
    serde_json::to_string_pretty(rows).unwrap_or_else(|_| "[]".to_string())
}

/// Export dataset rows as CSV
pub fn export_csv(rows: &[DatasetRow]) -> String {
    let mut out = String::new();
    // Header
    out.push_str("id,name,tier,format,transpiled,output_correct,lint_clean,deterministic,score,grade,bashrs_version,date\n");
    for row in rows {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{:.1},{},{},{}\n",
            csv_escape(&row.id),
            csv_escape(&row.name),
            row.tier,
            row.format,
            row.transpiled,
            row.output_correct,
            row.lint_clean,
            row.deterministic,
            row.score,
            row.grade,
            row.bashrs_version,
            row.date,
        ));
    }
    out
}

/// Escape a CSV field (double-quote if it contains comma, quote, or newline)
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Get dataset info (schema and metadata)
pub fn dataset_info(registry: &CorpusRegistry) -> DatasetInfo {
    let mut format_counts: Vec<(String, usize)> = Vec::new();
    for fmt in &[CorpusFormat::Bash, CorpusFormat::Makefile, CorpusFormat::Dockerfile] {
        let count = registry.entries.iter().filter(|e| e.format == *fmt).count();
        format_counts.push((fmt.to_string(), count));
    }

    DatasetInfo {
        total_entries: registry.entries.len(),
        format_counts,
        schema_fields: dataset_schema_fields(),
        bashrs_version: env!("CARGO_PKG_VERSION").to_string(),
        date: current_date(),
    }
}

/// The §10.3 dataset schema definition
fn dataset_schema_fields() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("id", "string", "Entry ID (B-001, M-042, D-015)"),
        ("name", "string", "Human-readable name"),
        ("tier", "int32", "Difficulty tier (1-5)"),
        ("format", "string", "bash, makefile, dockerfile"),
        ("input_rust", "string", "Rust DSL source code"),
        ("expected_output", "string", "Ground truth expected output"),
        ("actual_output", "string", "Transpiler actual output"),
        ("transpiled", "bool", "Transpilation succeeded?"),
        ("output_correct", "bool", "Output matches expected?"),
        ("lint_clean", "bool", "Output passes linter?"),
        ("deterministic", "bool", "Output identical across runs?"),
        ("score", "float64", "Per-entry score (0-100)"),
        ("grade", "string", "A+, A, B, C, D, F"),
        ("bashrs_version", "string", "e.g. 6.61.0"),
        ("commit_sha", "string", "Git commit SHA"),
        ("date", "string", "ISO 8601 date"),
    ]
}

/// Format dataset info as a table
pub fn format_dataset_info(info: &DatasetInfo) -> String {
    let mut out = String::new();
    let line = "\u{2500}".repeat(64);

    out.push_str(&format!("bashrs v{} \u{2014} {}\n\n", info.bashrs_version, info.date));

    out.push_str(&format!("Corpus: {} entries\n", info.total_entries));
    for (fmt, count) in &info.format_counts {
        out.push_str(&format!("  {:<14} {} entries\n", fmt, count));
    }

    out.push_str(&format!("\nDataset Schema (\u{00a7}10.3):\n{}\n", line));
    out.push_str(&format!("{:<18} {:<10} {}\n", "Field", "Type", "Description"));
    out.push_str(&format!("{}\n", line));
    for (name, dtype, desc) in &info.schema_fields {
        out.push_str(&format!("{:<18} {:<10} {}\n", name, dtype, desc));
    }
    out.push_str(&format!("{}\n", line));

    out.push_str("\nExport formats: json, jsonl, csv\n");
    out.push_str("Usage: bashrs corpus export --format json --output corpus.json\n");

    out
}

/// Publish readiness checks (§10.3)
pub fn check_publish_readiness(score: &CorpusScore) -> Vec<PublishCheck> {
    vec![
        PublishCheck {
            name: "Transpilation rate \u{2265} 99%",
            passed: score.rate >= 0.99,
            value: format!("{:.1}%", score.rate * 100.0),
        },
        PublishCheck {
            name: "Score \u{2265} 90.0",
            passed: score.score >= 90.0,
            value: format!("{:.1}", score.score),
        },
        PublishCheck {
            name: "All formats present",
            passed: score.format_scores.len() >= 3,
            value: format!("{} formats", score.format_scores.len()),
        },
        PublishCheck {
            name: "No failed entries",
            passed: score.failed == 0,
            value: format!("{} failed", score.failed),
        },
        PublishCheck {
            name: "Corpus size \u{2265} 100",
            passed: score.total >= 100,
            value: format!("{} entries", score.total),
        },
    ]
}

/// A single publish readiness check
#[derive(Debug, Clone)]
pub struct PublishCheck {
    pub name: &'static str,
    pub passed: bool,
    pub value: String,
}

/// Format publish checks as a table
pub fn format_publish_checks(checks: &[PublishCheck]) -> String {
    let mut out = String::new();
    let line = "\u{2500}".repeat(56);

    out.push_str(&format!("{}\n{:<36} {:<10} {}\n{}\n", line, "Check", "Status", "Value", line));
    for check in checks {
        let status = if check.passed { "\u{2713} PASS" } else { "\u{2717} FAIL" };
        out.push_str(&format!("{:<36} {:<10} {}\n", check.name, status, check.value));
    }
    out.push_str(&format!("{}\n", line));

    let all_pass = checks.iter().all(|c| c.passed);
    if all_pass {
        out.push_str("\nReady to publish to Hugging Face.\n");
    } else {
        let failed = checks.iter().filter(|c| !c.passed).count();
        out.push_str(&format!("\n{} check(s) failed. Fix before publishing.\n", failed));
    }

    out
}

/// Run the full corpus and export
pub fn run_and_export(format: ExportFormat) -> (CorpusScore, String) {
    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let rows = build_dataset(&registry, &score);

    let output = match format {
        ExportFormat::JsonLines => export_jsonl(&rows),
        ExportFormat::Csv => export_csv(&rows),
        ExportFormat::Json => export_json(&rows),
    };

    (score, output)
}

fn current_date() -> String {
    // Use a simple date format without external crate
    let now = std::time::SystemTime::now();
    let since_epoch = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = since_epoch.as_secs();
    // Calculate date components
    let days = secs / 86400;
    let (year, month, day) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn days_to_ymd(total_days: u64) -> (u64, u64, u64) {
    // Days since Unix epoch (1970-01-01)
    let mut y = 1970;
    let mut remaining = total_days;

    loop {
        let days_in_year = if is_leap_year(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }

    let leap = is_leap_year(y);
    let month_days = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 0;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining < md {
            m = i;
            break;
        }
        remaining -= md;
    }

    (y, m as u64 + 1, remaining + 1)
}

fn is_leap_year(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn current_commit() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier};

    fn make_entry(id: &str, format: CorpusFormat) -> CorpusEntry {
        CorpusEntry {
            id: id.to_string(),
            name: format!("test-{id}"),
            description: "Test entry".to_string(),
            format,
            tier: CorpusTier::Trivial,
            input: "fn main() {}".to_string(),
            expected_output: "#!/bin/sh\necho ok\n".to_string(),
            shellcheck: true,
            deterministic: true,
            idempotent: true,
        }
    }

    fn make_result(id: &str, transpiled: bool) -> CorpusResult {
        CorpusResult {
            id: id.to_string(),
            transpiled,
            output_contains: transpiled,
            output_exact: transpiled,
            output_behavioral: transpiled,
            has_test: true,
            coverage_ratio: 0.95,
            schema_valid: true,
            lint_clean: transpiled,
            deterministic: transpiled,
            metamorphic_consistent: transpiled,
            cross_shell_agree: transpiled,
            actual_output: if transpiled {
                Some("#!/bin/sh\necho ok\n".to_string())
            } else {
                None
            },
            error: if transpiled {
                None
            } else {
                Some("test error".to_string())
            },
            error_category: None,
            error_confidence: None,
            decision_trace: None,
        }
    }

    #[test]
    fn test_export_format_display() {
        assert_eq!(format!("{}", ExportFormat::JsonLines), "jsonl");
        assert_eq!(format!("{}", ExportFormat::Csv), "csv");
        assert_eq!(format!("{}", ExportFormat::Json), "json");
    }

    #[test]
    fn test_score_to_grade() {
        assert_eq!(score_to_grade(100.0), "A+");
        assert_eq!(score_to_grade(97.0), "A+");
        assert_eq!(score_to_grade(93.0), "A");
        assert_eq!(score_to_grade(90.0), "A-");
        assert_eq!(score_to_grade(85.0), "B");
        assert_eq!(score_to_grade(75.0), "C");
        assert_eq!(score_to_grade(65.0), "D");
        assert_eq!(score_to_grade(50.0), "F");
    }

    #[test]
    fn test_csv_escape_plain() {
        assert_eq!(csv_escape("hello"), "hello");
    }

    #[test]
    fn test_csv_escape_comma() {
        assert_eq!(csv_escape("hello,world"), "\"hello,world\"");
    }

    #[test]
    fn test_csv_escape_quotes() {
        assert_eq!(csv_escape("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_csv_escape_newline() {
        assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn test_build_row_with_result() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        assert_eq!(row.id, "B-001");
        assert!(row.transpiled);
        assert!(row.output_correct);
        assert!(row.lint_clean);
        assert!(row.deterministic);
        assert_eq!(row.bashrs_version, "6.61.0");
        assert!(row.score > 0.0);
    }

    #[test]
    fn test_build_row_without_result() {
        let entry = make_entry("B-002", CorpusFormat::Bash);
        let row = build_row(&entry, None, "6.61.0", "abc1234", "2026-02-09");

        assert_eq!(row.id, "B-002");
        assert!(!row.transpiled);
        assert!(!row.output_correct);
        assert_eq!(row.score, 0.0);
        assert_eq!(row.grade, "F");
    }

    #[test]
    fn test_build_row_failed_result() {
        let entry = make_entry("B-003", CorpusFormat::Bash);
        let result = make_result("B-003", false);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        assert!(!row.transpiled);
        assert!(!row.output_correct);
        assert_eq!(row.grade, "F");
    }

    #[test]
    fn test_export_jsonl() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        let output = export_jsonl(&[row]);
        assert!(output.contains("\"id\":\"B-001\""));
        assert!(output.contains("\"transpiled\":true"));
        assert!(!output.contains('\n') || output.lines().count() == 1);
    }

    #[test]
    fn test_export_json() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        let output = export_json(&[row]);
        assert!(output.starts_with('['));
        assert!(output.contains("\"id\": \"B-001\""));
    }

    #[test]
    fn test_export_csv() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        let output = export_csv(&[row]);
        assert!(output.starts_with("id,name,tier,format"));
        assert!(output.contains("B-001"));
        assert!(output.contains("bash"));
    }

    #[test]
    fn test_export_csv_multiple_rows() {
        let rows: Vec<DatasetRow> = ["B-001", "M-001"]
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let fmt = if i == 0 {
                    CorpusFormat::Bash
                } else {
                    CorpusFormat::Makefile
                };
                let entry = make_entry(id, fmt);
                let result = make_result(id, true);
                build_row(&entry, Some(&result), "6.61.0", "abc", "2026-02-09")
            })
            .collect();

        let output = export_csv(&rows);
        assert_eq!(output.lines().count(), 3); // header + 2 rows
    }

    #[test]
    fn test_dataset_info() {
        let entries = vec![
            make_entry("B-001", CorpusFormat::Bash),
            make_entry("M-001", CorpusFormat::Makefile),
            make_entry("D-001", CorpusFormat::Dockerfile),
        ];
        let registry = CorpusRegistry {
            entries,
            ..Default::default()
        };
        let info = dataset_info(&registry);
        assert_eq!(info.total_entries, 3);
        assert_eq!(info.format_counts.len(), 3);
        assert_eq!(info.schema_fields.len(), 16);
    }

    #[test]
    fn test_format_dataset_info() {
        let entries = vec![
            make_entry("B-001", CorpusFormat::Bash),
        ];
        let registry = CorpusRegistry {
            entries,
            ..Default::default()
        };
        let info = dataset_info(&registry);
        let table = format_dataset_info(&info);
        assert!(table.contains("Dataset Schema"));
        assert!(table.contains("id"));
        assert!(table.contains("string"));
        assert!(table.contains("float64"));
    }

    #[test]
    fn test_publish_checks_all_pass() {
        let score = CorpusScore {
            total: 900,
            passed: 900,
            failed: 0,
            rate: 1.0,
            score: 99.9,
            grade: crate::corpus::registry::Grade::APlus,
            format_scores: vec![
                crate::corpus::runner::FormatScore {
                    format: CorpusFormat::Bash,
                    total: 500,
                    passed: 500,
                    rate: 1.0,
                    score: 99.7,
                    grade: crate::corpus::registry::Grade::APlus,
                },
                crate::corpus::runner::FormatScore {
                    format: CorpusFormat::Makefile,
                    total: 200,
                    passed: 200,
                    rate: 1.0,
                    score: 100.0,
                    grade: crate::corpus::registry::Grade::APlus,
                },
                crate::corpus::runner::FormatScore {
                    format: CorpusFormat::Dockerfile,
                    total: 200,
                    passed: 200,
                    rate: 1.0,
                    score: 100.0,
                    grade: crate::corpus::registry::Grade::APlus,
                },
            ],
            results: vec![],
        };

        let checks = check_publish_readiness(&score);
        assert!(checks.iter().all(|c| c.passed));
    }

    #[test]
    fn test_publish_checks_some_fail() {
        let score = CorpusScore {
            total: 50,
            passed: 45,
            failed: 5,
            rate: 0.90,
            score: 85.0,
            grade: crate::corpus::registry::Grade::B,
            format_scores: vec![],
            results: vec![],
        };

        let checks = check_publish_readiness(&score);
        assert!(!checks.iter().all(|c| c.passed));
        // rate < 99%, score < 90, no formats, failed > 0, size < 100
        let failed_count = checks.iter().filter(|c| !c.passed).count();
        assert!(failed_count >= 3);
    }

    #[test]
    fn test_format_publish_checks() {
        let checks = vec![
            PublishCheck {
                name: "Test check",
                passed: true,
                value: "ok".to_string(),
            },
        ];
        let table = format_publish_checks(&checks);
        assert!(table.contains("Test check"));
        assert!(table.contains("PASS"));
        assert!(table.contains("Ready to publish"));
    }

    #[test]
    fn test_format_publish_checks_failure() {
        let checks = vec![
            PublishCheck {
                name: "Failing check",
                passed: false,
                value: "bad".to_string(),
            },
        ];
        let table = format_publish_checks(&checks);
        assert!(table.contains("FAIL"));
        assert!(table.contains("check(s) failed"));
    }

    #[test]
    fn test_days_to_ymd_epoch() {
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
    }

    #[test]
    fn test_days_to_ymd_known_date() {
        // 2026-02-09 is day 20,493 since epoch
        let (y, m, d) = days_to_ymd(20_493);
        assert_eq!(y, 2026);
        assert_eq!(m, 2);
        assert_eq!(d, 9);
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2025));
    }

    #[test]
    fn test_current_date_format() {
        let date = current_date();
        assert_eq!(date.len(), 10);
        assert_eq!(&date[4..5], "-");
        assert_eq!(&date[7..8], "-");
    }

    #[test]
    fn test_dataset_row_serializes() {
        let row = DatasetRow {
            id: "B-001".into(),
            name: "test".into(),
            tier: 1,
            format: "bash".into(),
            input_rust: "fn main() {}".into(),
            expected_output: "#!/bin/sh\n".into(),
            actual_output: "#!/bin/sh\n".into(),
            transpiled: true,
            output_correct: true,
            lint_clean: true,
            deterministic: true,
            score: 100.0,
            grade: "A+".into(),
            bashrs_version: "6.61.0".into(),
            commit_sha: "abc1234".into(),
            date: "2026-02-09".into(),
        };

        let json = serde_json::to_string(&row);
        assert!(json.is_ok());
        let s = json.expect("serialization should succeed");
        assert!(s.contains("B-001"));
    }
}
