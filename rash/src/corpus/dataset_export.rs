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

/// The S10.3 dataset schema definition
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

/// Publish readiness checks (S10.3)
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
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string())
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "dataset_tests_make_entry.rs"]
// FIXME(PMAT-238): mod tests_extracted;
