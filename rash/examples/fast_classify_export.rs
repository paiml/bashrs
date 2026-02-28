#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

//! Fast classification dataset exporter.
//!
//! Exports corpus entries as classification JSONL for ML training.
//! Skips slow B3/MR/cross-shell checks — only transpiles + lints + labels.
//!
//! Run with: cargo run --release --example fast_classify_export [output_path]

use bashrs::corpus::dataset::derive_safety_label;
use bashrs::corpus::registry::{CorpusFormat, CorpusRegistry};
use bashrs::linter::diagnostic::Severity;
use bashrs::linter::rules::{lint_dockerfile, lint_makefile, lint_shell};
use bashrs::Config;
use serde::Serialize;
use std::io::Write;

#[derive(Serialize)]
struct ClassificationRow {
    input: String,
    label: u8,
}

fn check_lint(output: &str, format: CorpusFormat) -> bool {
    match format {
        CorpusFormat::Bash => {
            let result = lint_shell(output);
            !result
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Error)
        }
        CorpusFormat::Makefile => {
            let result = lint_makefile(output);
            !result.has_errors()
        }
        CorpusFormat::Dockerfile => {
            let result = lint_dockerfile(output);
            !result.has_errors()
        }
    }
}

fn main() {
    let output_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/ssc-corpus.jsonl".to_string());

    eprintln!("Loading corpus registry...");
    let registry = CorpusRegistry::load_full();
    eprintln!("Loaded {} entries", registry.entries.len());

    let config = Config::default();
    let mut file = std::fs::File::create(&output_path).expect("Failed to create output file");

    let mut exported = 0u32;
    let mut failed = 0u32;
    let total = registry.entries.len();

    for (i, entry) in registry.entries.iter().enumerate() {
        if i % 1000 == 0 {
            eprintln!("[{i}/{total}] transpiling...");
        }

        let transpile_result = match entry.format {
            CorpusFormat::Bash => bashrs::transpile(&entry.input, config.clone()),
            CorpusFormat::Makefile => bashrs::transpile_makefile(&entry.input, config.clone()),
            CorpusFormat::Dockerfile => bashrs::transpile_dockerfile(&entry.input, config.clone()),
        };

        match transpile_result {
            Ok(output) => {
                let lint_clean = check_lint(&output, entry.format);
                // Check determinism with a second transpile
                let deterministic = match entry.format {
                    CorpusFormat::Bash => bashrs::transpile(&entry.input, config.clone())
                        .map(|o2| o2 == output)
                        .unwrap_or(false),
                    CorpusFormat::Makefile => {
                        bashrs::transpile_makefile(&entry.input, config.clone())
                            .map(|o2| o2 == output)
                            .unwrap_or(false)
                    }
                    CorpusFormat::Dockerfile => {
                        bashrs::transpile_dockerfile(&entry.input, config.clone())
                            .map(|o2| o2 == output)
                            .unwrap_or(false)
                    }
                };

                let label = derive_safety_label(&output, true, lint_clean, deterministic);
                let row = ClassificationRow {
                    input: output,
                    label,
                };
                let json = serde_json::to_string(&row).unwrap();
                writeln!(file, "{json}").unwrap();
                exported += 1;
            }
            Err(_) => {
                // Failed transpilation → label 4 (unsafe), use raw input
                let row = ClassificationRow {
                    input: entry.input.clone(),
                    label: 4,
                };
                let json = serde_json::to_string(&row).unwrap();
                writeln!(file, "{json}").unwrap();
                exported += 1;
                failed += 1;
            }
        }
    }

    eprintln!(
        "Exported {exported} entries ({failed} failed transpilation) to {output_path}"
    );
}
