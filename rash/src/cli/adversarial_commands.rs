//! CLI handler for `bashrs generate-adversarial` command.

use crate::corpus::adversarial_generator::{self, AdversarialConfig};
use crate::corpus::dataset::ClassificationRow;
use crate::models::{Error, Result};
use std::path::Path;

/// Execute the generate-adversarial command.
pub(crate) fn generate_adversarial_command(
    output: &Path,
    seed: u64,
    count_per_class: usize,
    extra_needs_quoting: usize,
    verify: bool,
    show_stats: bool,
) -> Result<()> {
    let config = AdversarialConfig {
        seed,
        count_per_class,
        extra_needs_quoting,
        verify,
    };

    eprintln!(
        "Generating adversarial data: {} per class, {} extra needs-quoting (seed={})",
        count_per_class, extra_needs_quoting, seed
    );

    let result = adversarial_generator::generate_adversarial(&config);

    // Write JSONL output
    let jsonl = rows_to_jsonl(&result.rows)?;
    std::fs::write(output, &jsonl).map_err(|e| {
        Error::Validation(format!("Failed to write {}: {e}", output.display()))
    })?;

    eprintln!(
        "Wrote {} rows to {}",
        result.rows.len(),
        output.display()
    );

    if show_stats || verify {
        eprintln!();
        eprintln!("{}", adversarial_generator::format_stats(&result.stats));
    }

    if verify && result.stats.misclassified > 0 {
        eprintln!(
            "\nWarning: {} scripts did not match expected classification",
            result.stats.misclassified
        );
    }

    Ok(())
}

/// Serialize classification rows to JSONL format.
fn rows_to_jsonl(rows: &[ClassificationRow]) -> Result<String> {
    let lines: Vec<String> = rows
        .iter()
        .filter_map(|row| serde_json::to_string(row).ok())
        .collect();
    let mut output = lines.join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    Ok(output)
}
