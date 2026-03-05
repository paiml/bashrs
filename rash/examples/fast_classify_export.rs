#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

//! Fast binary classification dataset exporter for SSC training.
//!
//! Exports corpus entries as classification JSONL for ML training.
//! Binary labels: safe (0) = transpiled + lint-clean + deterministic, unsafe (1) = otherwise.
//! Model input: original script text (what users feed at inference time).
//!
//! Produces a single `corpus.jsonl` file. Splitting into train/val/test is owned
//! by alimentar (`alimentar fed plan` + `alimentar fed split`) — NOT this exporter.
//!
//! Skips slow B3/MR/cross-shell checks — only transpiles + lints + labels.
//!
//! Delegates to [`bashrs::corpus::dataset::classify_single`] for labeling — all paths
//! share a single implementation to prevent divergent logic.
//!
//! Run with: cargo run --release --example fast_classify_export [output_dir]

use bashrs::corpus::dataset::{classify_single, validate_export, ClassificationRow};
use bashrs::corpus::registry::{CorpusFormat, CorpusRegistry};
use bashrs::linter::diagnostic::Severity;
use bashrs::linter::rules::{lint_dockerfile, lint_makefile, lint_shell};
use bashrs::Config;
use std::io::Write;
use std::path::Path;

/// Expected number of classes for the model head.
const NUM_CLASSES: u8 = 2;

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

fn write_jsonl(path: &Path, rows: &[ClassificationRow]) {
    let mut file = std::fs::File::create(path)
        .unwrap_or_else(|e| panic!("Failed to create {}: {e}", path.display()));
    for row in rows {
        let json = serde_json::to_string(row).unwrap();
        writeln!(file, "{json}").unwrap();
    }
}

fn main() {
    let output_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/ssc-export".to_string());

    std::fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    eprintln!("Loading corpus registry...");
    let registry = CorpusRegistry::load_full();
    eprintln!("Loaded {} entries", registry.entries.len());

    let config = Config::default();

    let mut all_rows: Vec<ClassificationRow> = Vec::with_capacity(registry.entries.len());
    let mut failed = 0u32;
    let total = registry.entries.len();

    for (i, entry) in registry.entries.iter().enumerate() {
        if i % 1000 == 0 {
            eprintln!("[{i}/{total}] transpiling...");
        }

        let transpile_result = match entry.format {
            CorpusFormat::Bash => bashrs::transpile(&entry.input, &config),
            CorpusFormat::Makefile => bashrs::transpile_makefile(&entry.input, &config),
            CorpusFormat::Dockerfile => bashrs::transpile_dockerfile(&entry.input, &config),
        };

        let row = match transpile_result {
            Ok(output) => {
                let lint_clean = check_lint(&output, entry.format);
                // Determinism is a transpiler invariant (verified by E-category corpus tests).
                // Successful transpilation implies deterministic output — no need to transpile
                // a second time. This halves the export runtime.
                classify_single(&entry.input, true, lint_clean, true)
            }
            Err(_) => {
                failed += 1;
                classify_single(&entry.input, false, false, false)
            }
        };

        all_rows.push(row);
    }

    eprintln!(
        "Classified {} entries ({failed} failed transpilation)",
        all_rows.len()
    );

    // DataOps gate: validate before writing
    let validation = validate_export(&all_rows, NUM_CLASSES);
    eprintln!("\n{validation}");

    if !validation.passed {
        eprintln!("BLOCKED: Fix data quality errors above before training.");
        std::process::exit(1);
    }

    // Write single corpus file — alimentar owns train/val/test splitting
    let corpus_path = Path::new(&output_dir).join("corpus.jsonl");
    write_jsonl(&corpus_path, &all_rows);

    eprintln!("Wrote {} rows to {}", all_rows.len(), corpus_path.display());
    eprintln!();
    eprintln!("Next: use alimentar for splitting:");
    eprintln!(
        "  alimentar convert {0}/corpus.jsonl {0}/corpus.parquet",
        output_dir
    );
    eprintln!(
        "  alimentar fed manifest {0}/corpus.parquet -o {0}/manifest.json -n bashrs",
        output_dir
    );
    eprintln!("  alimentar fed plan {0}/manifest.json -o {0}/plan.json -s stratified -r 0.8 --test-ratio 0.1 --validation-ratio 0.1 --stratify-column label", output_dir);
    eprintln!("  alimentar fed split {0}/corpus.parquet -p {0}/plan.json -n bashrs --train-output {0}/train.parquet --test-output {0}/test.parquet --validation-output {0}/val.parquet", output_dir);
}
