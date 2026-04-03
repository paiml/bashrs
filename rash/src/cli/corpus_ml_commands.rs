//! ML pipeline commands: embedding extraction, classifier training, and evaluation.
use crate::models::{Error, Result};
#[cfg(feature = "ml")]
use std::path::Path;
use std::path::PathBuf;

/// Load classification rows from a JSONL file (format: `{"input":"...","label":N}`).
///
/// Non-zero labels are mapped to 1 (unsafe) for binary classification.
#[cfg(feature = "ml")]
fn load_classification_jsonl(
    path: &Path,
) -> Result<Vec<crate::corpus::dataset::ClassificationRow>> {
    use crate::corpus::dataset::ClassificationRow;

    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::Validation(format!("Cannot read {}: {e}", path.display())))?;

    #[derive(serde::Deserialize)]
    struct RawEntry {
        input: String,
        label: u8,
    }

    let mut entries = Vec::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<RawEntry>(line) {
            Ok(e) => entries.push(ClassificationRow {
                input: e.input,
                label: if e.label > 0 { 1 } else { 0 },
            }),
            Err(err) => eprintln!("  Skipping invalid line: {err}"),
        }
    }
    Ok(entries)
}

/// Extract [CLS] embeddings from CodeBERT for all corpus entries (CLF-RUN step 1).
#[allow(unused_variables)]
pub(crate) fn corpus_extract_embeddings(
    model: PathBuf,
    output: PathBuf,
    limit: Option<usize>,
    input_jsonl: Option<PathBuf>,
) -> Result<()> {
    #[cfg(not(feature = "ml"))]
    {
        Err(Error::Validation(
            "The `ml` feature is required for extract-embeddings. Rebuild with: cargo build --features ml".into(),
        ))
    }

    #[cfg(feature = "ml")]
    {
        use crate::cli::color::*;
        use crate::corpus::classifier::extract_embeddings_streaming;
        use crate::corpus::dataset::ClassificationRow;

        eprintln!("{BOLD}Extracting [CLS] embeddings from CodeBERT...{RESET}");

        let mut rows: Vec<ClassificationRow> = if let Some(ref jsonl_path) = input_jsonl {
            let entries = load_classification_jsonl(jsonl_path)?;
            eprintln!(
                "  Input JSONL: {} entries from {}",
                entries.len(),
                jsonl_path.display()
            );
            entries
        } else {
            use crate::corpus::baselines::corpus_baseline_entries;
            let owned = corpus_baseline_entries();
            owned
                .into_iter()
                .map(|(input, label)| ClassificationRow { input, label })
                .collect()
        };

        if let Some(n) = limit {
            rows.truncate(n);
            eprintln!("  Entries: {} (limited)", rows.len());
        } else {
            eprintln!("  Entries: {}", rows.len());
        }

        // Extract with streaming writes (one entry at a time to disk)
        let start = std::time::Instant::now();
        let report =
            extract_embeddings_streaming(&model, &rows, &output, &|i, total, elapsed_ms| {
                let rate = if elapsed_ms > 0 {
                    (i as f64 / elapsed_ms as f64) * 1000.0
                } else {
                    0.0
                };
                let eta_s = if rate > 0.0 {
                    ((total - i) as f64 / rate) as u64
                } else {
                    0
                };
                eprintln!(
                    "  [{i}/{total}] {:.1}% | {:.2} entries/s | ETA: {}m {}s",
                    100.0 * i as f64 / total as f64,
                    rate,
                    eta_s / 60,
                    eta_s % 60,
                );
            })
            .map_err(Error::Validation)?;

        let elapsed = start.elapsed();
        eprintln!(
            "\n{GREEN}\u{2713}{RESET} {BOLD}Embeddings saved to {}{RESET} in {:.1}s",
            output.display(),
            elapsed.as_secs_f64()
        );
        eprintln!(
            "  Total: {} | Extracted: {} | Skipped: {} | Dim: {} | Rate: {:.2}/s",
            report.total_entries,
            report.extracted,
            report.skipped,
            report.hidden_size,
            report.extracted as f64 / elapsed.as_secs_f64().max(0.001),
        );

        Ok(())
    }
}

/// Train linear probe classifier on cached embeddings (CLF-RUN step 2-3).
#[allow(clippy::too_many_arguments)]
pub(crate) fn corpus_train_classifier(
    embeddings_path: PathBuf,
    output: PathBuf,
    epochs: usize,
    learning_rate: f32,
    seed: u64,
    max_entries: Option<usize>,
    augment: Vec<PathBuf>,
    mlp: bool,
    mlp_hidden: usize,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::classifier::{
        evaluate_probe, load_embeddings, save_probe, split_embeddings, train_linear_probe,
    };

    let probe_type = if mlp {
        format!("MLP probe (hidden={mlp_hidden})")
    } else {
        "linear probe".into()
    };
    eprintln!("{BOLD}Training {probe_type} classifier...{RESET}");

    // Load cached embeddings
    let mut all_embeddings = load_embeddings(&embeddings_path).map_err(Error::Validation)?;
    eprintln!(
        "  Loaded {} embeddings from {}",
        all_embeddings.len(),
        embeddings_path.display()
    );

    // Cap entries if --max-entries specified (avoids data labeling gaps, see #171)
    if let Some(max) = max_entries {
        if all_embeddings.len() > max {
            eprintln!("  Capping to {max} entries (--max-entries)");
            all_embeddings.truncate(max);
        }
    }

    // Augment with additional embedding files (e.g. adversarial entries)
    for aug_path in &augment {
        let aug = load_embeddings(aug_path).map_err(Error::Validation)?;
        eprintln!(
            "  Augmenting with {} entries from {}",
            aug.len(),
            aug_path.display()
        );
        all_embeddings.extend(aug);
    }

    // Split into train/test
    let (train, test) = split_embeddings(&all_embeddings, seed);
    eprintln!("  Train: {} | Test: {}", train.len(), test.len());

    // Train (linear or MLP)
    eprintln!("\n{BOLD}Training (epochs={epochs}, lr={learning_rate}):{RESET}");
    std::fs::create_dir_all(&output)
        .map_err(|e| Error::Validation(format!("Cannot create {}: {e}", output.display())))?;

    let test_report = if mlp {
        let (mlp_weights, report) =
            train_and_evaluate_mlp(&train, &test, epochs, learning_rate, mlp_hidden)?;
        // Save MLP weights
        let mlp_json = serde_json::to_string_pretty(&mlp_weights)
            .map_err(|e| Error::Validation(format!("Serialize MLP: {e}")))?;
        std::fs::write(output.join("mlp_probe.json"), mlp_json)
            .map_err(|e| Error::Validation(format!("Write MLP: {e}")))?;
        report
    } else {
        let probe = train_linear_probe(&train, epochs, learning_rate);
        eprintln!(
            "  Train accuracy: {:.1}% | Train MCC: {:.3}",
            probe.train_accuracy * 100.0,
            probe.train_mcc
        );
        let report = evaluate_probe(&probe, &test);
        save_probe(&probe, &output.join("probe.json")).map_err(Error::Validation)?;
        report
    };

    eprintln!("\n{BOLD}Test Evaluation:{RESET}");
    eprintln!("  Accuracy:  {:.1}%", test_report.accuracy * 100.0);
    eprintln!("  Precision: {:.3}", test_report.precision);
    eprintln!("  Recall:    {:.3}", test_report.recall);
    eprintln!("  F1:        {:.3}", test_report.f1);
    eprintln!("  MCC:       {:.3}", test_report.mcc);
    eprintln!(
        "  Confusion: TP={} FP={} TN={} FN={}",
        test_report.confusion.tp,
        test_report.confusion.fp,
        test_report.confusion.tn,
        test_report.confusion.fn_
    );

    // Save evaluation
    let eval_json = serde_json::to_string_pretty(&test_report)
        .map_err(|e| Error::Validation(format!("Serialize: {e}")))?;
    std::fs::write(output.join("evaluation.json"), eval_json)
        .map_err(|e| Error::Validation(format!("Write: {e}")))?;

    // Quality gate: C-CLF-001 — classifier must beat baselines
    let beats_keyword = test_report.mcc > 0.3;
    let beats_linter = test_report.mcc > 0.4;
    eprintln!("\n{BOLD}Ship Gate C-CLF-001:{RESET}");
    eprintln!(
        "  Beats keyword baseline (MCC>0.3): {}",
        if beats_keyword {
            format!("{GREEN}PASS{RESET}")
        } else {
            format!("{RED}FAIL{RESET}")
        }
    );
    eprintln!(
        "  Beats linter baseline (MCC>0.4): {}",
        if beats_linter {
            format!("{GREEN}PASS{RESET}")
        } else {
            format!("{RED}FAIL{RESET}")
        }
    );

    eprintln!(
        "\n{GREEN}\u{2713}{RESET} {BOLD}Classifier artifacts saved to {}{RESET}",
        output.display()
    );

    Ok(())
}

/// Train MLP probe and evaluate (Level 0.5).
#[cfg(feature = "ml")]
fn train_and_evaluate_mlp(
    train: &[crate::corpus::classifier::EmbeddingEntry],
    test: &[crate::corpus::classifier::EmbeddingEntry],
    epochs: usize,
    learning_rate: f32,
    mlp_hidden: usize,
) -> Result<(
    crate::corpus::classifier::MlpProbeWeights,
    crate::corpus::evaluation::EvaluationReport,
)> {
    use entrenar::finetune::MlpProbe;

    let hidden_size = train
        .first()
        .map(|e| e.embedding.len())
        .ok_or_else(|| Error::Validation("No training embeddings".into()))?;

    let embeddings: Vec<Vec<f32>> = train.iter().map(|e| e.embedding.clone()).collect();
    let labels: Vec<usize> = train.iter().map(|e| e.label as usize).collect();

    // Compute class weights (sqrt-inverse balanced)
    let n = labels.len() as f32;
    let n_safe = labels.iter().filter(|&&l| l == 0).count() as f32;
    let n_unsafe = labels.iter().filter(|&&l| l == 1).count() as f32;
    let class_weights = if n_unsafe > 0.0 {
        vec![(n / (2.0 * n_safe)).sqrt(), (n / (2.0 * n_unsafe)).sqrt()]
    } else {
        vec![1.0, 1.0]
    };
    eprintln!(
        "  Class weights: safe={:.3}, unsafe={:.3}",
        class_weights[0], class_weights[1]
    );

    let mut mlp = MlpProbe::new(hidden_size, mlp_hidden, 2);
    eprintln!(
        "  Parameters: {} ({} hidden)",
        mlp.num_parameters(),
        mlp_hidden
    );
    mlp.train(
        &embeddings,
        &labels,
        epochs,
        learning_rate,
        Some(&class_weights),
        1e-4,
    );

    // Evaluate on train
    let train_correct = embeddings
        .iter()
        .zip(labels.iter())
        .filter(|(e, &l)| mlp.predict(e) == l)
        .count();
    let train_acc = train_correct as f64 / labels.len().max(1) as f64;
    eprintln!("  Train accuracy: {:.1}%", train_acc * 100.0);

    // Evaluate on test: build (pred, truth) pairs for evaluate()
    let predictions: Vec<(u8, u8)> = test
        .iter()
        .map(|e| (mlp.predict(&e.embedding) as u8, e.label))
        .collect();
    let report = crate::corpus::evaluation::evaluate(&predictions, "MLP probe");

    let weights = crate::corpus::classifier::MlpProbeWeights {
        w1: mlp.w1,
        b1: mlp.b1,
        w2: mlp.w2,
        b2: mlp.b2,
        hidden_size,
        mlp_hidden,
        num_classes: 2,
        epochs,
        learning_rate,
        train_accuracy: train_acc,
    };

    Ok((weights, report))
}

/// Fallback for non-ml builds.
#[cfg(not(feature = "ml"))]
fn train_and_evaluate_mlp(
    _train: &[crate::corpus::classifier::EmbeddingEntry],
    _test: &[crate::corpus::classifier::EmbeddingEntry],
    _epochs: usize,
    _lr: f32,
    _mlp_hidden: usize,
) -> Result<(
    crate::corpus::classifier::MlpProbeWeights,
    crate::corpus::evaluation::EvaluationReport,
)> {
    Err(Error::Validation("MLP probe requires --features ml".into()))
}

/// Run full CLF-RUN pipeline: extract → train → evaluate.
#[allow(unused_variables)]
pub(crate) fn corpus_run_classifier(
    model: PathBuf,
    output: PathBuf,
    epochs: usize,
    learning_rate: f32,
    seed: u64,
) -> Result<()> {
    #[cfg(not(feature = "ml"))]
    {
        Err(Error::Validation(
            "The `ml` feature is required for run-classifier. Rebuild with: cargo build --features ml".into(),
        ))
    }

    #[cfg(feature = "ml")]
    {
        use crate::cli::color::*;
        use crate::corpus::baselines::corpus_baseline_entries;
        use crate::corpus::classifier::{run_classifier_pipeline, save_probe};
        use crate::corpus::dataset::ClassificationRow;

        eprintln!("{BOLD}=== CLF-RUN: Full Classifier Pipeline ==={RESET}\n");

        // Build classification rows
        let owned = corpus_baseline_entries();
        let rows: Vec<ClassificationRow> = owned
            .into_iter()
            .map(|(input, label)| ClassificationRow { input, label })
            .collect();
        eprintln!("Corpus: {} entries", rows.len());

        // Create output directory
        std::fs::create_dir_all(&output)
            .map_err(|e| Error::Validation(format!("Cannot create {}: {e}", output.display())))?;

        // Run pipeline
        let report = run_classifier_pipeline(&model, &rows, epochs, learning_rate, seed)
            .map_err(Error::Validation)?;

        // Save probe weights
        save_probe(&report.probe, &output.join("probe.json")).map_err(Error::Validation)?;

        // Save evaluation report
        let eval_json = serde_json::to_string_pretty(&report.test_eval)
            .map_err(|e| Error::Validation(format!("Serialize: {e}")))?;
        std::fs::write(output.join("evaluation.json"), eval_json)
            .map_err(|e| Error::Validation(format!("Write: {e}")))?;

        // Print final results
        eprintln!("\n{BOLD}=== CLF-RUN Results ==={RESET}");
        eprintln!("Test Accuracy:  {:.1}%", report.test_eval.accuracy * 100.0);
        eprintln!("Test MCC:       {:.3}", report.test_eval.mcc);
        eprintln!("Test Precision: {:.3}", report.test_eval.precision);
        eprintln!("Test Recall:    {:.3}", report.test_eval.recall);
        eprintln!("Test F1:        {:.3}", report.test_eval.f1);
        eprintln!();
        eprintln!("{BOLD}Ship Gate C-CLF-001:{RESET}");
        eprintln!(
            "  Beats keyword (MCC>0.3): {}",
            if report.beats_keyword {
                format!("{GREEN}PASS{RESET}")
            } else {
                format!("{RED}FAIL{RESET}")
            }
        );
        eprintln!(
            "  Beats linter (MCC>0.4): {}",
            if report.beats_linter {
                format!("{GREEN}PASS{RESET}")
            } else {
                format!("{RED}FAIL{RESET}")
            }
        );

        eprintln!(
            "\n{GREEN}\u{2713}{RESET} {BOLD}All artifacts saved to {}{RESET}",
            output.display()
        );

        Ok(())
    }
}
