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


            include!("dataset_part3_incl2.rs");
