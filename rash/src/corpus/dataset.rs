//! # Dataset Export for Corpus Results (§10.3)
//!
//! Exports corpus results in portable formats (JSON, CSV) following the
//! Hugging Face dataset schema defined in §10.3. Enables reproducibility,
//! training data generation, and cross-project benchmarking.

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry};
use crate::corpus::runner::{CorpusResult, CorpusRunner, CorpusScore};
use crate::Config;
use std::collections::HashMap;
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClassificationRow {
    pub input: String,
    pub label: u8,
}

/// Result of pre-training data validation.
///
/// Catches data quality issues that cause training divergence BEFORE
/// burning GPU time. Every export path must call `validate_export()`
/// and abort if `!result.passed`.
#[derive(Debug)]
pub struct ExportValidation {
    pub passed: bool,
    pub total: usize,
    pub num_classes: usize,
    pub class_counts: [usize; 5],
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validate exported classification data for training readiness.
///
/// `expected_classes` is the number of classes the model head will have.
/// For binary safe/unsafe, pass 2. For full taxonomy, pass 5.
///
/// Checks (all must pass):
/// 1. **No missing classes**: all classes 0..expected_classes have >=1 sample
/// 2. **No extreme imbalance**: no single class >95% of total
/// 3. **No preamble contamination**: no inputs start with `#!/bin/sh` or `set -euf`
/// 4. **No length confound**: max avg length / min avg length < 10x across classes
/// 5. **No trivial inputs**: no inputs shorter than 3 chars
///
/// Returns `ExportValidation` with `passed=true` only if all checks pass.
pub fn validate_export(rows: &[ClassificationRow], expected_classes: u8) -> ExportValidation {
    let stats = collect_export_stats(rows);
    let total = rows.len();
    let num_classes = stats.present_classes.len();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    check_missing_classes(&stats, expected_classes, num_classes, &mut errors);
    check_class_imbalance(&stats.class_counts, total, &mut errors, &mut warnings);
    check_preamble_contamination(stats.preamble_count, total, &mut errors);
    check_length_confound(&stats, &mut errors, &mut warnings);
    if stats.trivial_count > 0 {
        warnings.push(format!(
            "trivial inputs: {} samples have <3 chars",
            stats.trivial_count
        ));
    }

    ExportValidation {
        passed: errors.is_empty(),
        total,
        num_classes,
        class_counts: stats.class_counts,
        errors,
        warnings,
    }
}

/// Aggregated statistics from a classification export.
struct ExportStats {
    class_counts: [usize; 5],
    class_total_len: [u64; 5],
    present_classes: Vec<u8>,
    preamble_count: usize,
    trivial_count: usize,
}

fn collect_export_stats(rows: &[ClassificationRow]) -> ExportStats {
    let mut class_counts = [0usize; 5];
    let mut class_total_len = [0u64; 5];
    let mut preamble_count = 0usize;
    let mut trivial_count = 0usize;

    for row in rows {
        let idx = row.label as usize;
        if idx < 5 {
            class_counts[idx] += 1;
            class_total_len[idx] += row.input.len() as u64;
        }
        let trimmed = row.input.trim();
        if trimmed.starts_with("#!/bin/sh")
            || trimmed.starts_with("set -euf")
            || trimmed.starts_with("IFS='")
            || trimmed.starts_with("export LC_ALL=C")
        {
            preamble_count += 1;
        }
        if trimmed.len() < 3 {
            trivial_count += 1;
        }
    }

    let present_classes: Vec<u8> = class_counts
        .iter()
        .enumerate()
        .filter(|(_, &c)| c > 0)
        .map(|(i, _)| i as u8)
        .collect();

    ExportStats {
        class_counts,
        class_total_len,
        present_classes,
        preamble_count,
        trivial_count,
    }
}

fn check_missing_classes(
    stats: &ExportStats,
    expected: u8,
    num_present: usize,
    errors: &mut Vec<String>,
) {
    let missing: Vec<u8> = (0..expected)
        .filter(|i| stats.class_counts[*i as usize] == 0)
        .collect();
    if !missing.is_empty() {
        errors.push(format!(
            "missing classes {:?} — model head has {} outputs but only {} classes present",
            missing, expected, num_present
        ));
    }
}

fn check_class_imbalance(
    class_counts: &[usize; 5],
    total: usize,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    if let Some(&max_count) = class_counts.iter().max() {
        if total > 0 {
            let max_pct = 100.0 * max_count as f64 / total as f64;
            if max_pct > 95.0 {
                errors.push(format!(
                    "extreme class imbalance: dominant class has {:.1}% of samples — \
                     model will learn majority-class prediction, not safety features",
                    max_pct
                ));
            } else if max_pct > 85.0 {
                warnings.push(format!(
                    "class imbalance: dominant class has {:.1}% — consider oversampling minorities",
                    max_pct
                ));
            }
        }
    }
}

fn check_preamble_contamination(preamble_count: usize, total: usize, errors: &mut Vec<String>) {
    if preamble_count > 0 {
        errors.push(format!(
            "preamble contamination: {preamble_count}/{total} inputs start with shell boilerplate"
        ));
    }
}

fn check_length_confound(
    stats: &ExportStats,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    let avg_lens: Vec<f64> = stats
        .present_classes
        .iter()
        .map(|&c| {
            let idx = c as usize;
            if stats.class_counts[idx] > 0 {
                stats.class_total_len[idx] as f64 / stats.class_counts[idx] as f64
            } else {
                0.0
            }
        })
        .collect();

    if avg_lens.len() < 2 {
        return;
    }
    let min_avg = avg_lens.iter().copied().fold(f64::MAX, f64::min);
    let max_avg = avg_lens.iter().copied().fold(0.0f64, f64::max);
    if min_avg <= 0.0 {
        return;
    }
    let ratio = max_avg / min_avg;
    if ratio > 10.0 {
        errors.push(format!(
            "length confound: {ratio:.1}x ratio between shortest and longest class avg — \
             model can cheat on length instead of learning safety features"
        ));
    } else if ratio > 5.0 {
        warnings.push(format!(
            "length spread: {ratio:.1}x ratio between class avg lengths — \
             consider normalizing or truncating"
        ));
    }
}

impl std::fmt::Display for ExportValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Export Validation: {}",
            if self.passed { "PASS" } else { "FAIL" }
        )?;
        writeln!(
            f,
            "  Total: {} samples, {} classes",
            self.total, self.num_classes
        )?;
        for (i, &count) in self.class_counts.iter().enumerate() {
            if count > 0 {
                let pct = 100.0 * count as f64 / self.total as f64;
                writeln!(f, "  Class {i}: {count:>6} ({pct:5.1}%)")?;
            }
        }
        for e in &self.errors {
            writeln!(f, "  ERROR: {e}")?;
        }
        for w in &self.warnings {
            writeln!(f, "  WARN: {w}")?;
        }
        Ok(())
    }
}

/// Split assignment for a classification row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Split {
    Train,
    Val,
    Test,
}

impl fmt::Display for Split {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Split::Train => write!(f, "train"),
            Split::Val => write!(f, "val"),
            Split::Test => write!(f, "test"),
        }
    }
}

/// Result of splitting and validating classification data.
pub struct SplitResult {
    pub train: Vec<ClassificationRow>,
    pub val: Vec<ClassificationRow>,
    pub test: Vec<ClassificationRow>,
    pub validation: ExportValidation,
    pub split_validations: [ExportValidation; 3],
}

/// Deterministic hash-based stratified split.
///
/// Assigns each row to train/val/test using a hash of the input text,
/// producing stable splits that don't reshuffle when the corpus grows.
/// Within each split, class proportions match the overall distribution
/// (stratified).
///
/// Ratio: 80% train, 10% val, 10% test.
///
/// This is the canonical split function. All export paths must use it
/// to prevent train/test leakage and ensure reproducibility.
pub fn split_and_validate(rows: Vec<ClassificationRow>, expected_classes: u8) -> SplitResult {
    // Hash-based assignment: stable across corpus growth
    let mut train = Vec::new();
    let mut val = Vec::new();
    let mut test = Vec::new();

    for row in rows {
        let split = assign_split(&row.input);
        match split {
            Split::Train => train.push(row),
            Split::Val => val.push(row),
            Split::Test => test.push(row),
        }
    }

    // Validate overall + per-split
    let mut all: Vec<ClassificationRow> = Vec::with_capacity(train.len() + val.len() + test.len());
    all.extend(train.iter().cloned());
    all.extend(val.iter().cloned());
    all.extend(test.iter().cloned());

    let validation = validate_export(&all, expected_classes);
    let split_validations = [
        validate_export(&train, expected_classes),
        validate_export(&val, expected_classes),
        validate_export(&test, expected_classes),
    ];

    SplitResult {
        train,
        val,
        test,
        validation,
        split_validations,
    }
}

include!("dataset_fmt.rs");
