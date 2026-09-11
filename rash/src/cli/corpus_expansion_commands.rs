//! SSB expansion CLI commands (PMAT-172, PMAT-176).
//!
//! Extracted from corpus_config_commands.rs for file-size discipline.

use crate::models::{Error, Result};
use std::path::PathBuf;

/// Publish ShellSafetyBench to HuggingFace (SSC v12 S14.7, Phase 10).
///
/// Reads pre-existing SSB split files and generates a complete HuggingFace
/// Datasets repository with dataset card, metadata, and split files.
pub(crate) fn corpus_publish_benchmark(
    input: PathBuf,
    output: PathBuf,
    version: String,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::benchmark_publish;

    eprintln!("{BOLD}Publishing ShellSafetyBench v{version} to HuggingFace...{RESET}");
    eprintln!("  Input splits: {}", input.display());
    eprintln!("  Output dir:   {}", output.display());

    let summary = benchmark_publish::publish_benchmark(&input, &output, &version)?;

    eprintln!(
        "\n{GREEN}\u{2713}{RESET} {BOLD}ShellSafetyBench v{version} published to {}{RESET}",
        output.display()
    );
    eprintln!("  README.md          \u{2014} Dataset card with HF YAML front matter");
    eprintln!(
        "  train.jsonl        \u{2014} {} entries ({:.1}%)",
        summary.train_count,
        (summary.train_count as f64 / summary.total as f64) * 100.0
    );
    eprintln!(
        "  validation.jsonl   \u{2014} {} entries ({:.1}%)",
        summary.val_count,
        (summary.val_count as f64 / summary.total as f64) * 100.0
    );
    eprintln!(
        "  test.jsonl         \u{2014} {} entries ({:.1}%)",
        summary.test_count,
        (summary.test_count as f64 / summary.total as f64) * 100.0
    );
    eprintln!("  dataset_infos.json \u{2014} HuggingFace metadata");
    eprintln!(
        "  Class balance: {:.1}% unsafe ({} / {})",
        summary.unsafe_pct, summary.unsafe_count, summary.total
    );
    eprintln!(
        "\nTo upload: `huggingface-cli upload paiml/shell-safety-bench {}`",
        output.display()
    );

    Ok(())
}

/// Generate expansion entries for ShellSafetyBench (Phase 9 #10).
///
/// Produces labeled JSONL from parameterized templates for Bash, Makefile,
/// and Dockerfile formats. Output is compatible with `merge-data` and
/// `export-splits`.
pub(crate) fn corpus_generate_expansion(
    format: String,
    count: usize,
    output: PathBuf,
    seed: u64,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::expansion_generator::{self, GenFormat};

    let gen_format = match format.as_str() {
        "bash" => GenFormat::Bash,
        "makefile" => GenFormat::Makefile,
        "dockerfile" => GenFormat::Dockerfile,
        _ => {
            return Err(Error::Validation(format!(
                "Unknown format: {format}. Use bash, makefile, or dockerfile."
            )));
        }
    };

    eprintln!("{BOLD}Generating {count} {format} entries (seed={seed})...{RESET}");

    let entries = expansion_generator::generate_expansion(gen_format, count, seed);
    let mut summary = expansion_generator::write_expansion(&entries, &output)?;
    summary.format = gen_format;

    eprintln!(
        "\n{GREEN}\u{2713}{RESET} {BOLD}Generated {count} {format} entries to {}{RESET}",
        output.display()
    );
    eprintln!(
        "  Safe:   {} ({:.1}%)",
        summary.safe,
        (summary.safe as f64 / summary.total as f64) * 100.0
    );
    eprintln!(
        "  Unsafe: {} ({:.1}%)",
        summary.unsafe_count,
        (summary.unsafe_count as f64 / summary.total as f64) * 100.0
    );
    eprintln!(
        "\nTo merge: `bashrs corpus merge-data --input {} -o merged.jsonl`",
        output.display()
    );

    Ok(())
}

// PMAT-257: neither function here ever touches CorpusRunner/CorpusRegistry --
// generation and publishing both work from small on-disk fixtures, so the
// whole file is safe to exercise directly with tempfile::TempDir.
#[cfg(test)]
mod pmat257_cov_tests {
    use super::*;

    #[test]
    fn test_PMAT257_cov_generate_expansion_each_format() {
        for fmt in ["bash", "makefile", "dockerfile"] {
            let dir = tempfile::TempDir::new().expect("tempdir");
            let out = dir.path().join("expansion.jsonl");
            corpus_generate_expansion(fmt.to_string(), 4, out.clone(), 7)
                .unwrap_or_else(|e| panic!("generating {fmt} entries must succeed: {e}"));
            let content = std::fs::read_to_string(&out).expect("output file must exist");
            assert!(
                !content.trim().is_empty(),
                "{fmt} expansion must write at least one JSONL row"
            );
        }
    }

    #[test]
    fn test_PMAT257_cov_generate_expansion_rejects_unknown_format() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let out = dir.path().join("expansion.jsonl");
        let err = corpus_generate_expansion("cobol".to_string(), 4, out, 1)
            .expect_err("an unknown format string must be rejected");
        assert!(format!("{err}").contains("Unknown format"));
    }

    #[test]
    fn test_PMAT257_cov_publish_benchmark_round_trips_real_splits() {
        let input_dir = tempfile::TempDir::new().expect("tempdir");
        let output_dir = tempfile::TempDir::new().expect("tempdir");
        for (name, label) in [("train", 0u8), ("val", 1u8), ("test", 0u8)] {
            let path = input_dir.path().join(format!("{name}.jsonl"));
            let row = serde_json::json!({"input": "echo hi", "label": label});
            std::fs::write(&path, format!("{row}\n")).expect("write split fixture");
        }
        corpus_publish_benchmark(
            input_dir.path().to_path_buf(),
            output_dir.path().to_path_buf(),
            "0.0.1-test".to_string(),
        )
        .expect("publishing a well-formed splits dir must succeed");
        assert!(
            output_dir.path().join("README.md").exists(),
            "publish_benchmark must write a dataset card"
        );
    }

    #[test]
    fn test_PMAT257_cov_publish_benchmark_missing_input_is_an_error() {
        let input_dir = tempfile::TempDir::new().expect("tempdir");
        let output_dir = tempfile::TempDir::new().expect("tempdir");
        let err = corpus_publish_benchmark(
            input_dir.path().to_path_buf(),
            output_dir.path().to_path_buf(),
            "0.0.1-test".to_string(),
        )
        .expect_err("a splits dir missing train/val/test.jsonl must fail");
        assert!(format!("{err}").contains("Cannot read"));
    }
}
