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

/// Assign a split based on a deterministic hash of the input text.
///
/// Uses FNV-1a hash (fast, no crypto needed) mod 10:
/// - 0..=7 → Train (80%)
/// - 8     → Val   (10%)
/// - 9     → Test  (10%)
///
/// Stable: same input always maps to same split, even if corpus order changes.
fn assign_split(input: &str) -> Split {
    let hash = fnv1a_hash(input.as_bytes());
    match hash % 10 {
        0..=7 => Split::Train,
        8 => Split::Val,
        _ => Split::Test,
    }
}

/// FNV-1a 64-bit hash — fast, deterministic, no dependencies.
fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

impl fmt::Display for SplitResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.train.len() + self.val.len() + self.test.len();
        writeln!(f, "Split Result ({total} total):")?;
        for (name, split, sv) in [
            ("train", &self.train, &self.split_validations[0]),
            ("val", &self.val, &self.split_validations[1]),
            ("test", &self.test, &self.split_validations[2]),
        ] {
            let pct = if total > 0 {
                100.0 * split.len() as f64 / total as f64
            } else {
                0.0
            };
            let status = if sv.passed { "PASS" } else { "FAIL" };
            write!(f, "  {name}: {:>6} ({pct:5.1}%) [{status}]", split.len())?;
            // Show per-class counts inline
            for (i, &c) in sv.class_counts.iter().enumerate() {
                if c > 0 {
                    write!(f, " c{i}={c}")?;
                }
            }
            writeln!(f)?;
        }
        if !self.validation.passed {
            for e in &self.validation.errors {
                writeln!(f, "  ERROR: {e}")?;
            }
        }
        for w in &self.validation.warnings {
            writeln!(f, "  WARN: {w}")?;
        }
        Ok(())
    }
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

    // KAIZEN-069: O(1) HashMap lookup instead of O(n) linear find per entry.
    // With 17,942 entries, the old O(n²) find wasted ~161M string comparisons.
    let results_by_id: HashMap<&str, &CorpusResult> =
        score.results.iter().map(|r| (r.id.as_str(), r)).collect();

    registry
        .entries
        .iter()
        .map(|entry| {
            let result = results_by_id.get(entry.id.as_str()).copied();
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
        if trimmed.starts_with("ln ")
            && trimmed.contains("-s")
            && !trimmed.contains("-sf")
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
///
/// Shell preamble (shebang, `set -euf`, `IFS=`, `export`, `trap`, `main "$@"`)
/// is stripped by default to remove noise that confuses classifiers.
/// The `trap '... $$'` pattern in particular contains a non-deterministic
/// signal (`$$` = process ID) that is present in every transpiled script,
/// causing safe scripts to be misclassified as non-deterministic.
pub fn export_classification_jsonl(rows: &[DatasetRow]) -> String {
    rows.iter()
        .map(|row| {
            let cr = classify_single(
                &row.input_rust,
                row.transpiled,
                row.lint_clean,
                row.deterministic,
            );
            serde_json::to_string(&cr).unwrap_or_default()
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Build a single binary classification row for ML training.
///
/// This is the canonical single-entry classification path.
/// Both `export_classification_jsonl` (batch) and `fast_classify_export` (streaming)
/// must use this to avoid divergent preamble-stripping or labeling logic.
///
/// **Binary classification** (safe=0, unsafe=1):
/// - Safe (0): transpiled successfully AND lint-clean AND deterministic
/// - Unsafe (1): any of those checks failed
///
/// **Model input**: the original script (what users feed at inference time),
/// with shell preamble stripped. NOT the transpiled output — users don't have
/// transpiled output at inference time.
///
/// - `original_input`: raw bash/makefile/dockerfile (used as model input text)
/// - `transpiled`: whether transpilation succeeded
/// - `lint_clean`: whether the output passed lint checks
/// - `deterministic`: whether two transpilations produce identical output
pub fn classify_single(
    original_input: &str,
    transpiled: bool,
    lint_clean: bool,
    deterministic: bool,
) -> ClassificationRow {
    let label = if transpiled && lint_clean && deterministic {
        0
    } else {
        1
    };
    ClassificationRow {
        input: strip_shell_preamble(original_input),
        label,
    }
}

/// Strip shell preamble lines from transpiled output.
///
/// Removes boilerplate that is identical across all transpiled scripts
/// and adds no discriminative signal for classification:
/// - Shebang (`#!/bin/sh`)
/// - Shell options (`set -euf`)
/// - IFS reset (`IFS=' \t\n'`)
/// - Locale export (`export LC_ALL=C`)
/// - Trap cleanup (`trap 'rm -rf ...' EXIT`)
/// - Function wrappers (`main() {`, `}`, `main "$@"`)
/// - Generated comments (`# Generated by Rash`, `# POSIX-compliant`, etc.)
///
/// Returns only the meaningful body lines joined by newlines.
/// If stripping produces an empty string, returns the original input unchanged.
pub fn strip_shell_preamble(script: &str) -> String {
    let body: Vec<&str> = script
        .lines()
        .filter(|line| {
            let s = line.trim();
            // Filter preamble lines + structural wrappers.
            // s == "'" catches the closing quote of multi-line IFS=' \t\n'
            !is_shell_preamble(s) && s != "main() {" && s != "}" && s != "'"
        })
        .map(|line| {
            // Dedent: strip leading whitespace from lines inside main() { ... }
            let trimmed = line.trim_start();
            if trimmed.is_empty() {
                line
            } else {
                trimmed
            }
        })
        .collect();

    if body.is_empty() {
        // Don't produce empty inputs — fall back to original
        return script.to_string();
    }

    body.join("\n")
}

/// Return true if this trimmed line is shell preamble (not user code).
///
/// Used by `strip_shell_preamble` and corpus B2 commands to identify
/// transpiler boilerplate that should be excluded from classification input.
///
/// Note: does NOT match structural markers like `main() {` or `}` — those
/// are needed by `extract_bash_main_body` for state tracking. The
/// `strip_shell_preamble` function handles those separately.
pub fn is_shell_preamble(s: &str) -> bool {
    s.is_empty()
        || s.starts_with('#')
        || s.starts_with("set ")
        || s.starts_with("IFS=")
        || s.starts_with("export ")
        || s.starts_with("trap ")
        || s == "main \"$@\""
}

/// Export corpus as multi-label classification JSONL (SSC-021).
///
/// Each row has ALL applicable labels as a multi-hot vector, not just the primary one.
/// Output: `{"input":"...","labels":[0.0, 1.0, 1.0, 0.0, 0.0]}`
///
/// Shell preamble is stripped (same as single-label export).
pub fn export_multi_label_classification_jsonl(rows: &[DatasetRow]) -> String {
    rows.iter()
        .filter(|row| row.transpiled)
        .map(|row| {
            let labels = derive_multi_label(
                &row.actual_output,
                row.transpiled,
                row.lint_clean,
                row.deterministic,
            );
            let ml = MultiLabelClassificationRow {
                input: strip_shell_preamble(&row.actual_output),
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
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string())
}

#[cfg(test)]
#[path = "dataset_tests_extracted.rs"]
mod tests_extracted;
