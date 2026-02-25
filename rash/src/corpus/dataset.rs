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
    /// Classification JSONL for ML fine-tuning ({"input":"...","label":N})
    Classification,
    /// Multi-label classification JSONL ({"input":"...","labels":[0.0, 1.0, ...]})
    MultiLabelClassification,
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JsonLines => write!(f, "jsonl"),
            Self::Csv => write!(f, "csv"),
            Self::Json => write!(f, "json"),
            Self::Classification => write!(f, "classification"),
            Self::MultiLabelClassification => write!(f, "multi-label-classification"),
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
    pub safety_index: u8,
    pub safety_label: String,
    pub bashrs_version: String,
    pub commit_sha: String,
    pub date: String,
}

/// Lightweight classification row for ML training (entrenar-compatible).
///
/// Format: `{"input": "<shell script>", "label": N}` where N is 0-4.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ClassificationRow {
    pub input: String,
    pub label: u8,
}

/// Multi-label classification row for ML training (SSC-021).
///
/// Format: `{"input": "<shell script>", "labels": [0.0, 1.0, 1.0, 0.0, 0.0]}`
#[derive(Debug, Clone, serde::Serialize)]
pub struct MultiLabelClassificationRow {
    pub input: String,
    pub labels: [f32; 5],
}

/// Safety class labels matching aprender `SafetyClass` enum.
pub const SAFETY_LABELS: [&str; 5] = [
    "safe",              // 0
    "needs-quoting",     // 1
    "non-deterministic", // 2
    "non-idempotent",    // 3
    "unsafe",            // 4
];

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
pub fn build_dataset(registry: &CorpusRegistry, score: &CorpusScore) -> Vec<DatasetRow> {
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

    let safety_index = derive_safety_label(&actual, transpiled, lint_clean, deterministic);

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
        safety_index,
        safety_label: SAFETY_LABELS[safety_index as usize].to_string(),
        bashrs_version: version.to_string(),
        commit_sha: commit.to_string(),
        date: date.to_string(),
    }
}

/// Derive safety class from transpiler output using a decision tree.
///
/// Decision tree (cascading priority):
/// 1. Not transpiled OR not lint-clean → unsafe (4)
/// 2. Not deterministic → non-deterministic (2)
/// 3. Non-idempotent patterns (mkdir without -p, rm without -f, ln without -sf) → non-idempotent (3)
/// 4. Unquoted variable expansion ($VAR without quotes) → needs-quoting (1)
/// 5. Otherwise → safe (0)
///
/// Returns safety class index (0-4).
pub fn derive_safety_label(
    shell_output: &str,
    transpiled: bool,
    lint_clean: bool,
    deterministic: bool,
) -> u8 {
    // Gate 1: failed transpilation or lint → unsafe
    if !transpiled || !lint_clean {
        return 4;
    }

    // Gate 2: non-deterministic → class 2
    if !deterministic {
        return 2;
    }

    // Gate 3: non-idempotent patterns in the shell output
    if has_non_idempotent_pattern(shell_output) {
        return 3;
    }

    // Gate 4: unquoted variable expansion → needs-quoting
    if has_unquoted_variable(shell_output) {
        return 1;
    }

    // Default: safe
    0
}

/// Check for non-idempotent shell patterns.
///
/// Detects:
/// - `mkdir` without `-p` flag
/// - `rm` without `-f` flag (non-force removes fail on missing files)
/// - `ln -s` without `-f` (fails if link exists)
pub fn has_non_idempotent_pattern(script: &str) -> bool {
    for line in script.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // mkdir without -p
        if trimmed.starts_with("mkdir ") && !trimmed.contains("-p") {
            return true;
        }

        // rm without -f (but not rm -rf, rm -f)
        if trimmed.starts_with("rm ") && !trimmed.contains("-f") && !trimmed.contains("-rf") {
            return true;
        }

        // ln -s without -f (non-idempotent symlink creation)
        if trimmed.starts_with("ln ") && trimmed.contains("-s") && !trimmed.contains("-sf")
            && !trimmed.contains("-f")
        {
            return true;
        }
    }
    false
}

/// Check for unquoted variable expansions in shell script.
///
/// Detects `$VAR` or `${VAR}` that appear outside of double quotes.
/// Simple heuristic: scans for `$` followed by alphanumeric/underscore
/// that is NOT within a double-quoted region on the same line.
pub fn has_unquoted_variable(script: &str) -> bool {
    for line in script.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if line_has_unquoted_var(trimmed) {
            return true;
        }
    }
    false
}

/// Check a single line for unquoted variable references.
fn line_has_unquoted_var(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];

        // Track quote state
        if b == b'\'' && !in_double_quotes {
            in_single_quotes = !in_single_quotes;
            i += 1;
            continue;
        }
        if b == b'"' && !in_single_quotes {
            in_double_quotes = !in_double_quotes;
            i += 1;
            continue;
        }

        // Skip escaped characters
        if b == b'\\' && i + 1 < bytes.len() {
            i += 2;
            continue;
        }

        // Check for $ outside quotes
        if b == b'$' && !in_single_quotes && !in_double_quotes {
            // Check if followed by alphanumeric/underscore or {
            if i + 1 < bytes.len() {
                let next = bytes[i + 1];
                if next.is_ascii_alphabetic() || next == b'_' || next == b'{' {
                    return true;
                }
            }
        }

        i += 1;
    }
    false
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

/// Export classification JSONL for entrenar fine-tuning.
///
/// Output format: `{"input":"<shell script>","label":N}` per line.
/// Uses `actual_output` (transpiled shell) as the input text and
/// `safety_index` as the label. Only includes entries that were
/// successfully transpiled.
pub fn export_classification_jsonl(rows: &[DatasetRow]) -> String {
    rows.iter()
        .filter(|row| row.transpiled)
        .map(|row| {
            let cr = ClassificationRow {
                input: row.actual_output.clone(),
                label: row.safety_index,
            };
            serde_json::to_string(&cr).unwrap_or_default()
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Export corpus as multi-label classification JSONL (SSC-021).
///
/// Each row has ALL applicable labels as a multi-hot vector, not just the primary one.
/// Output: `{"input":"...","labels":[0.0, 1.0, 1.0, 0.0, 0.0]}`
pub fn export_multi_label_classification_jsonl(rows: &[DatasetRow]) -> String {
    rows.iter()
        .filter(|row| row.transpiled)
        .map(|row| {
            let labels = derive_multi_label(&row.actual_output, row.transpiled, row.lint_clean, row.deterministic);
            let ml = MultiLabelClassificationRow {
                input: row.actual_output.clone(),
                labels,
            };
            serde_json::to_string(&ml).unwrap_or_default()
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Derive multi-hot label vector (SSC-021).
///
/// Unlike `derive_safety_label` which picks ONE priority class, this detects
/// ALL applicable classes independently:
/// - Class 0 (safe): no issues at all
/// - Class 1 (needs-quoting): has unquoted variable
/// - Class 2 (non-deterministic): not deterministic
/// - Class 3 (non-idempotent): has non-idempotent patterns
/// - Class 4 (unsafe): not lint_clean or not transpiled
pub fn derive_multi_label(
    shell_output: &str,
    transpiled: bool,
    lint_clean: bool,
    deterministic: bool,
) -> [f32; 5] {
    let mut labels = [0.0f32; 5];

    // Class 4: unsafe
    if !transpiled || !lint_clean {
        labels[4] = 1.0;
    }

    // Class 2: non-deterministic
    if !deterministic {
        labels[2] = 1.0;
    }

    // Class 3: non-idempotent
    if has_non_idempotent_pattern(shell_output) {
        labels[3] = 1.0;
    }

    // Class 1: needs-quoting
    if has_unquoted_variable(shell_output) {
        labels[1] = 1.0;
    }

    // Class 0: safe (only if nothing else is active)
    if labels.iter().all(|&v| v < 0.5) {
        labels[0] = 1.0;
    }

    labels
}

/// Export dataset rows as JSON array (pretty-printed)
pub fn export_json(rows: &[DatasetRow]) -> String {
    serde_json::to_string_pretty(rows).unwrap_or_else(|_| "[]".to_string())
}

/// Export dataset rows as CSV
pub fn export_csv(rows: &[DatasetRow]) -> String {
    let mut out = String::new();
    // Header
    out.push_str("id,name,tier,format,transpiled,output_correct,lint_clean,deterministic,score,grade,safety_index,safety_label,bashrs_version,date\n");
    for row in rows {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{:.1},{},{},{},{},{}\n",
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
            row.safety_index,
            row.safety_label,
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
    for fmt in &[
        CorpusFormat::Bash,
        CorpusFormat::Makefile,
        CorpusFormat::Dockerfile,
    ] {
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
        ("safety_index", "uint8", "Safety class (0=safe..4=unsafe)"),
        ("safety_label", "string", "Safety class label"),
        ("bashrs_version", "string", "e.g. 6.61.0"),
        ("commit_sha", "string", "Git commit SHA"),
        ("date", "string", "ISO 8601 date"),
    ]
}

/// Format dataset info as a table
pub fn format_dataset_info(info: &DatasetInfo) -> String {
    let mut out = String::new();
    let line = "\u{2500}".repeat(64);

    out.push_str(&format!(
        "bashrs v{} \u{2014} {}\n\n",
        info.bashrs_version, info.date
    ));

    out.push_str(&format!("Corpus: {} entries\n", info.total_entries));
    for (fmt, count) in &info.format_counts {
        out.push_str(&format!("  {:<14} {} entries\n", fmt, count));
    }

    out.push_str(&format!("\nDataset Schema (\u{00a7}10.3):\n{}\n", line));
    out.push_str(&format!(
        "{:<18} {:<10} {}\n",
        "Field", "Type", "Description"
    ));
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

    out.push_str(&format!(
        "{}\n{:<36} {:<10} {}\n{}\n",
        line, "Check", "Status", "Value", line
    ));
    for check in checks {
        let status = if check.passed {
            "\u{2713} PASS"
        } else {
            "\u{2717} FAIL"
        };
        out.push_str(&format!(
            "{:<36} {:<10} {}\n",
            check.name, status, check.value
        ));
    }
    out.push_str(&format!("{}\n", line));

    let all_pass = checks.iter().all(|c| c.passed);
    if all_pass {
        out.push_str("\nReady to publish to Hugging Face.\n");
    } else {
        let failed = checks.iter().filter(|c| !c.passed).count();
        out.push_str(&format!(
            "\n{} check(s) failed. Fix before publishing.\n",
            failed
        ));
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
        ExportFormat::Classification => export_classification_jsonl(&rows),
        ExportFormat::MultiLabelClassification => export_multi_label_classification_jsonl(&rows),
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
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
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
            expected_output: None,
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
        assert_eq!(format!("{}", ExportFormat::Classification), "classification");
        assert_eq!(
            format!("{}", ExportFormat::MultiLabelClassification),
            "multi-label-classification"
        );
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
        };
        let info = dataset_info(&registry);
        assert_eq!(info.total_entries, 3);
        assert_eq!(info.format_counts.len(), 3);
        assert_eq!(info.schema_fields.len(), 18);
    }

    #[test]
    fn test_format_dataset_info() {
        let entries = vec![make_entry("B-001", CorpusFormat::Bash)];
        let registry = CorpusRegistry {
            entries,
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
        let checks = vec![PublishCheck {
            name: "Test check",
            passed: true,
            value: "ok".to_string(),
        }];
        let table = format_publish_checks(&checks);
        assert!(table.contains("Test check"));
        assert!(table.contains("PASS"));
        assert!(table.contains("Ready to publish"));
    }

    #[test]
    fn test_format_publish_checks_failure() {
        let checks = vec![PublishCheck {
            name: "Failing check",
            passed: false,
            value: "bad".to_string(),
        }];
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
            safety_index: 0,
            safety_label: "safe".into(),
            bashrs_version: "6.61.0".into(),
            commit_sha: "abc1234".into(),
            date: "2026-02-09".into(),
        };

        let json = serde_json::to_string(&row);
        assert!(json.is_ok());
        let s = json.expect("serialization should succeed");
        assert!(s.contains("B-001"));
        assert!(s.contains("safety_index"));
        assert!(s.contains("safe"));
    }

    // ── Safety label derivation tests ───────────────────────────────

    #[test]
    fn test_derive_safety_label_safe() {
        // Clean transpiled output with quoted vars → safe (0)
        let script = "#!/bin/sh\necho \"hello world\"\nmkdir -p \"$HOME/tmp\"\n";
        assert_eq!(derive_safety_label(script, true, true, true), 0);
    }

    #[test]
    fn test_derive_safety_label_not_transpiled() {
        // Failed transpilation → unsafe (4)
        assert_eq!(derive_safety_label("", false, true, true), 4);
    }

    #[test]
    fn test_derive_safety_label_not_lint_clean() {
        // Lint failures → unsafe (4)
        assert_eq!(derive_safety_label("echo ok", true, false, true), 4);
    }

    #[test]
    fn test_derive_safety_label_not_deterministic() {
        // Non-deterministic → class 2
        assert_eq!(derive_safety_label("echo ok", true, true, false), 2);
    }

    #[test]
    fn test_derive_safety_label_non_idempotent_mkdir() {
        // mkdir without -p → non-idempotent (3)
        let script = "#!/bin/sh\nmkdir /tmp/build\n";
        assert_eq!(derive_safety_label(script, true, true, true), 3);
    }

    #[test]
    fn test_derive_safety_label_idempotent_mkdir() {
        // mkdir -p → safe (0)
        let script = "#!/bin/sh\nmkdir -p /tmp/build\n";
        assert_eq!(derive_safety_label(script, true, true, true), 0);
    }

    #[test]
    fn test_derive_safety_label_non_idempotent_rm() {
        // rm without -f → non-idempotent (3)
        let script = "#!/bin/sh\nrm /tmp/file\n";
        assert_eq!(derive_safety_label(script, true, true, true), 3);
    }

    #[test]
    fn test_derive_safety_label_idempotent_rm() {
        // rm -f → safe (0)
        let script = "#!/bin/sh\nrm -f /tmp/file\n";
        assert_eq!(derive_safety_label(script, true, true, true), 0);
    }

    #[test]
    fn test_derive_safety_label_non_idempotent_ln() {
        // ln -s without -f → non-idempotent (3)
        let script = "#!/bin/sh\nln -s /a /b\n";
        assert_eq!(derive_safety_label(script, true, true, true), 3);
    }

    #[test]
    fn test_derive_safety_label_unquoted_var() {
        // Unquoted $VAR → needs-quoting (1)
        let script = "#!/bin/sh\necho $HOME\n";
        assert_eq!(derive_safety_label(script, true, true, true), 1);
    }

    #[test]
    fn test_derive_safety_label_quoted_var() {
        // Quoted "$VAR" → safe (0)
        let script = "#!/bin/sh\necho \"$HOME\"\n";
        assert_eq!(derive_safety_label(script, true, true, true), 0);
    }

    #[test]
    fn test_derive_safety_label_single_quoted_var() {
        // Single-quoted '$VAR' → safe (0) — no expansion in single quotes
        let script = "#!/bin/sh\necho '$HOME'\n";
        assert_eq!(derive_safety_label(script, true, true, true), 0);
    }

    #[test]
    fn test_derive_safety_label_priority_unsafe_over_nondeterministic() {
        // Not lint clean AND not deterministic → unsafe (4) wins
        assert_eq!(derive_safety_label("echo ok", true, false, false), 4);
    }

    #[test]
    fn test_derive_safety_label_priority_nondeterministic_over_non_idempotent() {
        // Non-deterministic AND has mkdir → non-deterministic (2) wins
        let script = "#!/bin/sh\nmkdir /tmp/build\n";
        assert_eq!(derive_safety_label(script, true, true, false), 2);
    }

    #[test]
    fn test_has_non_idempotent_pattern_comments_ignored() {
        assert!(!has_non_idempotent_pattern("# mkdir /tmp/build\n"));
        assert!(!has_non_idempotent_pattern("  # rm file\n"));
    }

    #[test]
    fn test_line_has_unquoted_var_basic() {
        assert!(line_has_unquoted_var("echo $HOME"));
        assert!(line_has_unquoted_var("echo ${HOME}"));
        assert!(!line_has_unquoted_var("echo \"$HOME\""));
        assert!(!line_has_unquoted_var("echo '$HOME'"));
        assert!(!line_has_unquoted_var("echo hello"));
    }

    #[test]
    fn test_line_has_unquoted_var_dollar_special() {
        // $? $# $0 etc. are special — not flagged (no alpha/underscore/brace after $)
        assert!(!line_has_unquoted_var("echo $?"));
        assert!(!line_has_unquoted_var("echo $#"));
    }

    // ── Classification export tests ─────────────────────────────────

    #[test]
    fn test_export_classification_jsonl() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        let output = export_classification_jsonl(&[row]);
        assert!(output.contains("\"input\""));
        assert!(output.contains("\"label\""));
        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output)
            .expect("classification JSONL should be valid JSON");
        assert!(parsed.get("input").is_some());
        assert!(parsed.get("label").is_some());
    }

    #[test]
    fn test_export_classification_jsonl_skips_failed() {
        let entry = make_entry("B-002", CorpusFormat::Bash);
        let result = make_result("B-002", false);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        let output = export_classification_jsonl(&[row]);
        assert!(output.is_empty(), "Failed entries should not appear in classification export");
    }

    #[test]
    fn test_export_classification_jsonl_multiple() {
        let rows: Vec<DatasetRow> = ["B-001", "B-002", "B-003"]
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let entry = make_entry(id, CorpusFormat::Bash);
                let result = make_result(id, i != 1); // B-002 fails
                build_row(&entry, Some(&result), "6.61.0", "abc", "2026-02-09")
            })
            .collect();

        let output = export_classification_jsonl(&rows);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2, "Should have 2 lines (B-002 filtered out)");
    }

    #[test]
    fn test_classification_row_serializes() {
        let cr = ClassificationRow {
            input: "#!/bin/sh\necho ok\n".into(),
            label: 0,
        };
        let json = serde_json::to_string(&cr).expect("should serialize");
        assert!(json.contains("\"input\""));
        assert!(json.contains("\"label\":0"));
    }

    #[test]
    fn test_safety_labels_count() {
        assert_eq!(SAFETY_LABELS.len(), 5);
        assert_eq!(SAFETY_LABELS[0], "safe");
        assert_eq!(SAFETY_LABELS[4], "unsafe");
    }

    #[test]
    fn test_build_row_includes_safety() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        // Transpiled, lint clean, deterministic, no unsafe patterns → safe
        assert_eq!(row.safety_label, "safe");
        assert_eq!(row.safety_index, 0);
    }

    #[test]
    fn test_build_row_failed_is_unsafe() {
        let entry = make_entry("B-002", CorpusFormat::Bash);
        let result = make_result("B-002", false);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        assert_eq!(row.safety_label, "unsafe");
        assert_eq!(row.safety_index, 4);
    }

    #[test]
    fn test_csv_includes_safety_fields() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc1234", "2026-02-09");

        let output = export_csv(&[row]);
        assert!(output.contains("safety_index"));
        assert!(output.contains("safety_label"));
    }

    // ── Multi-label classification tests (SSC-021) ──────────────────

    #[test]
    fn test_derive_multi_label_safe() {
        let labels = derive_multi_label("#!/bin/sh\necho \"hello\"\n", true, true, true);
        assert_eq!(labels, [1.0, 0.0, 0.0, 0.0, 0.0], "Clean script should be safe only");
    }

    #[test]
    fn test_derive_multi_label_unsafe() {
        let labels = derive_multi_label("#!/bin/sh\necho hello\n", false, false, true);
        assert_eq!(labels[4], 1.0, "Not transpiled → unsafe");
    }

    #[test]
    fn test_derive_multi_label_nondet() {
        let labels = derive_multi_label("#!/bin/sh\necho \"hello\"\n", true, true, false);
        assert_eq!(labels[2], 1.0, "Non-deterministic should be set");
        assert_eq!(labels[0], 0.0, "Safe should NOT be set when nondet");
    }

    #[test]
    fn test_derive_multi_label_nonidempotent_and_unquoted() {
        let labels = derive_multi_label("mkdir $HOME/build\n", true, true, true);
        assert_eq!(labels[3], 1.0, "Non-idempotent pattern should be set");
        assert_eq!(labels[1], 1.0, "Needs-quoting should also be set");
        assert_eq!(labels[0], 0.0, "Safe should NOT be set");
    }

    #[test]
    fn test_derive_multi_label_multiple_issues() {
        // Not deterministic + has unquoted var → classes 1 and 2
        let labels = derive_multi_label("echo $HOME\n", true, true, false);
        assert_eq!(labels[1], 1.0, "Needs-quoting should be set");
        assert_eq!(labels[2], 1.0, "Non-deterministic should be set");
        assert_eq!(labels[0], 0.0, "Safe should NOT be set");
    }

    #[test]
    fn test_multi_label_row_serializes() {
        let ml = MultiLabelClassificationRow {
            input: "echo $HOME\n".into(),
            labels: [0.0, 1.0, 1.0, 0.0, 0.0],
        };
        let json = serde_json::to_string(&ml).expect("should serialize");
        assert!(json.contains("\"labels\""));
        assert!(json.contains("[0.0,1.0,1.0,0.0,0.0]"));
    }

    #[test]
    fn test_export_multi_label_classification_jsonl() {
        let entry = make_entry("B-001", CorpusFormat::Bash);
        let result = make_result("B-001", true);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc", "2026-02-09");

        let output = export_multi_label_classification_jsonl(&[row]);
        assert!(output.contains("\"input\""));
        assert!(output.contains("\"labels\""));
    }

    #[test]
    fn test_export_multi_label_skips_failed() {
        let entry = make_entry("B-002", CorpusFormat::Bash);
        let result = make_result("B-002", false);
        let row = build_row(&entry, Some(&result), "6.61.0", "abc", "2026-02-09");

        let output = export_multi_label_classification_jsonl(&[row]);
        assert!(output.is_empty(), "Failed entries should not appear");
    }
}

#[cfg(test)]
#[path = "dataset_tests.rs"]
mod dataset_tests;
