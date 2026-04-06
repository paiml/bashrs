//! Publish ShellSafetyBench to HuggingFace Hub (SSC v12 S14.7, Phase 10).
//!
//! Generates a complete HuggingFace Datasets-compatible repository structure
//! for `paiml/shell-safety-bench`. Reads pre-existing SSB split files (from
//! `export-splits --input merged.jsonl`) or falls back to corpus-only data.
//!
//! Output directory structure:
//! ```text
//! output/
//! ├── README.md          # Dataset card with HF YAML front matter
//! ├── train.jsonl        # Training split (80%)
//! ├── validation.jsonl   # Validation split (10%)
//! ├── test.jsonl         # Test split (10%)
//! └── dataset_infos.json # HuggingFace dataset metadata
//! ```

use crate::corpus::cwe_mapping;
use crate::models::{Error, Result};
use serde::Serialize;
use std::path::Path;

/// Statistics about the published benchmark.
#[derive(Debug)]
pub struct PublishSummary {
    pub train_count: usize,
    pub val_count: usize,
    pub test_count: usize,
    pub total: usize,
    pub unsafe_count: usize,
    pub safe_count: usize,
    pub unsafe_pct: f64,
}

/// Split entry read from existing JSONL files.
#[derive(Debug, serde::Deserialize, Serialize)]
pub struct SplitEntry {
    pub input: String,
    pub label: u8,
}

/// Read SSB split files from an input directory.
///
/// Expects `train.jsonl`, `val.jsonl`, `test.jsonl` in `splits_dir`.
pub fn read_splits(
    splits_dir: &Path,
) -> Result<(Vec<SplitEntry>, Vec<SplitEntry>, Vec<SplitEntry>)> {
    let train = read_jsonl(&splits_dir.join("train.jsonl"))?;
    let val = read_jsonl(&splits_dir.join("val.jsonl"))?;
    let test = read_jsonl(&splits_dir.join("test.jsonl"))?;
    Ok((train, val, test))
}

fn read_jsonl(path: &Path) -> Result<Vec<SplitEntry>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::Validation(format!("Cannot read {}: {e}", path.display())))?;
    let mut entries = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let entry: SplitEntry = serde_json::from_str(line).map_err(|e| {
            Error::Validation(format!("Invalid JSON at {}:{}: {e}", path.display(), i + 1))
        })?;
        entries.push(entry);
    }
    Ok(entries)
}

/// Write split entries to JSONL file.
fn write_jsonl(path: &Path, entries: &[SplitEntry]) -> Result<()> {
    use std::io::Write;
    let file = std::fs::File::create(path)
        .map_err(|e| Error::Validation(format!("Cannot create {}: {e}", path.display())))?;
    let mut writer = std::io::BufWriter::new(file);
    for entry in entries {
        let json = serde_json::to_string(entry)
            .map_err(|e| Error::Validation(format!("JSON serialize error: {e}")))?;
        writeln!(writer, "{json}").map_err(|e| Error::Validation(format!("Write error: {e}")))?;
    }
    Ok(())
}

/// Compute summary statistics from split data.
pub fn compute_summary(
    train: &[SplitEntry],
    val: &[SplitEntry],
    test: &[SplitEntry],
) -> PublishSummary {
    let total = train.len() + val.len() + test.len();
    let unsafe_count = train
        .iter()
        .chain(val)
        .chain(test)
        .filter(|e| e.label == 1)
        .count();
    let safe_count = total - unsafe_count;
    let unsafe_pct = if total > 0 {
        (unsafe_count as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    PublishSummary {
        train_count: train.len(),
        val_count: val.len(),
        test_count: test.len(),
        total,
        unsafe_count,
        safe_count,
        unsafe_pct,
    }
}

/// Generate HuggingFace dataset card (README.md) with YAML front matter.
///
/// This is the primary artifact for `paiml/shell-safety-bench` on HuggingFace.
pub fn generate_dataset_card(summary: &PublishSummary, version: &str) -> String {
    let cwe_summary = cwe_mapping::summary();
    let cwe_count = cwe_mapping::linter_cwe_ids().len();

    format!(
        r#"---
annotations_creators:
- machine-generated
language:
- code
language_creators:
- expert-generated
license: apache-2.0
multilinguality: monolingual
size_categories:
- 10K<n<100K
source_datasets:
- original
task_categories:
- text-classification
task_ids:
- binary-classification
pretty_name: ShellSafetyBench
tags:
- security
- shell
- bash
- makefile
- dockerfile
- code-safety
- cwe
- vulnerability-detection
configs:
- config_name: default
  data_files:
  - split: train
    path: train.jsonl
  - split: validation
    path: validation.jsonl
  - split: test
    path: test.jsonl
dataset_info:
  features:
  - name: input
    dtype: string
  - name: label
    dtype: int8
  splits:
  - name: train
    num_examples: {train}
  - name: validation
    num_examples: {val}
  - name: test
    num_examples: {test}
---

# ShellSafetyBench v{version}

The first ML benchmark for shell script security classification. Covers **Bash**, **Makefile**, and **Dockerfile** with {cwe_count} CWE-mapped vulnerability categories.

## Dataset Description

ShellSafetyBench is a binary classification benchmark for detecting unsafe patterns in infrastructure-as-code scripts. Each entry contains a shell/Make/Docker script and a safety label (0=safe, 1=unsafe).

- **Total entries**: {total}
- **Train**: {train} ({train_pct:.1}%)
- **Validation**: {val} ({val_pct:.1}%)
- **Test**: {test} ({test_pct:.1}%)
- **Unsafe ratio**: {unsafe_pct:.1}% ({unsafe_count} unsafe / {safe_count} safe)

## Labels

| Label | Meaning | Description |
|-------|---------|-------------|
| 0 | **safe** | No known unsafe patterns detected by bashrs linter |
| 1 | **unsafe** | Contains one or more security/determinism/idempotency violations |

## CWE Coverage

{cwe_summary}

## Data Sources

1. **bashrs corpus** — Curated shell/Make/Docker scripts with transpilation testing
2. **verificar mutations** — CWE-targeted mutation-generated unsafe variants

## Methodology

Labels are produced by the **bashrs deterministic linter** (14 rules: SEC001-SEC008, DET001-DET003, IDEM001-IDEM003). Each rule maps to a specific CWE identifier with CVSS v3.1 severity scoring.

### Label Quality

- **Ground truth**: Deterministic rule-based labeling (no human annotation noise)
- **Cross-validated**: >80% agreement with ShellCheck on overlapping rules
- **Balanced**: {unsafe_pct:.1}% unsafe (augmented via CWE-targeted mutations)

### Splitting Strategy

Hash-based deterministic split (FNV-1a mod 10) ensures:
- Stable splits across dataset growth
- No data leakage between splits
- Reproducible benchmarks

## Usage

```python
from datasets import load_dataset

ds = load_dataset("paiml/shell-safety-bench")
print(ds["test"][0])  # {{"input": "rm -rf $DIR", "label": 1}}
```

## Evaluation

Recommended metrics:
- **MCC** (Matthews Correlation Coefficient) — primary metric, handles class imbalance
- **Precision/Recall** — for understanding safety-critical false negative rate
- **Per-CWE recall** — ensures coverage across vulnerability categories

### Baselines

| Model | MCC | Accuracy | Notes |
|-------|-----|----------|-------|
| Majority class | 0.000 | {majority_acc:.1}% | Always predicts safe |
| Keyword heuristic | 0.448 | — | Pattern matching on known unsafe keywords |
| bashrs MLP probe | 0.754 | — | CodeBERT embeddings + MLP classifier |
| Qwen3-4B QLoRA (lm_head) | 0.618 | — | Fine-tuned, lm_head scoring on full test set |
| Qwen3-4B QLoRA (full-fwd) | 0.770 | — | Fine-tuned, full forward pass (200 entries) |

## Limitations

1. Labels are linter-derived — may miss novel vulnerability patterns not covered by the 14 rules
2. Scripts are transpiler output — may not perfectly represent hand-written production scripts
3. Binary classification only — does not distinguish severity levels (CVSS scores available in CWE mapping)

## Citation

```bibtex
@dataset{{shellsafetybench2026,
  title={{ShellSafetyBench: A Binary Classification Benchmark for Shell Script Security}},
  author={{PAIML}},
  year={{2026}},
  url={{https://huggingface.co/datasets/paiml/shell-safety-bench}},
  version={{{version}}},
}}
```

## License

Apache 2.0
"#,
        train = summary.train_count,
        val = summary.val_count,
        test = summary.test_count,
        total = summary.total,
        unsafe_count = summary.unsafe_count,
        safe_count = summary.safe_count,
        unsafe_pct = summary.unsafe_pct,
        train_pct = (summary.train_count as f64 / summary.total as f64) * 100.0,
        val_pct = (summary.val_count as f64 / summary.total as f64) * 100.0,
        test_pct = (summary.test_count as f64 / summary.total as f64) * 100.0,
        majority_acc = (summary.safe_count as f64 / summary.total as f64) * 100.0,
        version = version,
        cwe_summary = cwe_summary,
        cwe_count = cwe_count,
    )
}

/// Generate dataset_infos.json for HuggingFace auto-loading.
pub fn generate_dataset_infos(summary: &PublishSummary) -> String {
    let info = serde_json::json!({
        "default": {
            "description": "ShellSafetyBench: Binary classification benchmark for shell script security",
            "features": {
                "input": {"dtype": "string", "_type": "Value"},
                "label": {"dtype": "int8", "_type": "Value"}
            },
            "splits": {
                "train": {"name": "train", "num_examples": summary.train_count},
                "validation": {"name": "validation", "num_examples": summary.val_count},
                "test": {"name": "test", "num_examples": summary.test_count}
            },
            "homepage": "https://github.com/paiml/bashrs",
            "license": "apache-2.0"
        }
    });
    serde_json::to_string_pretty(&info).unwrap_or_default()
}

/// Publish SSB benchmark: read splits, generate HF repo structure, write to output dir.
pub fn publish_benchmark(
    splits_dir: &Path,
    output_dir: &Path,
    version: &str,
) -> Result<PublishSummary> {
    // 1. Read existing splits
    let (train, val, test) = read_splits(splits_dir)?;

    // 2. Compute summary
    let summary = compute_summary(&train, &val, &test);

    // 3. Create output directory
    std::fs::create_dir_all(output_dir)
        .map_err(|e| Error::Validation(format!("Cannot create {}: {e}", output_dir.display())))?;

    // 4. Write split files (HF expects "validation" not "val")
    write_jsonl(&output_dir.join("train.jsonl"), &train)?;
    write_jsonl(&output_dir.join("validation.jsonl"), &val)?;
    write_jsonl(&output_dir.join("test.jsonl"), &test)?;

    // 5. Generate dataset card
    let card = generate_dataset_card(&summary, version);
    std::fs::write(output_dir.join("README.md"), &card)
        .map_err(|e| Error::Validation(format!("Failed to write README.md: {e}")))?;

    // 6. Generate dataset_infos.json
    let infos = generate_dataset_infos(&summary);
    std::fs::write(output_dir.join("dataset_infos.json"), &infos)
        .map_err(|e| Error::Validation(format!("Failed to write dataset_infos.json: {e}")))?;

    Ok(summary)
}

#[cfg(test)]
#[path = "benchmark_publish_tests_extracted_write.rs"]
mod tests_extracted;
