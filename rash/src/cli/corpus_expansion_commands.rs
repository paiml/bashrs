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
