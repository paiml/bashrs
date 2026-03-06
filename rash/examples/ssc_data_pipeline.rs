//! Example: SSC v11 data pipeline — model card + training config + splits.
//!
//! Demonstrates the complete data preparation workflow for classifier training:
//! 1. Generate HuggingFace model card (README.md)
//! 2. Generate entrenar training configuration
//! 3. Show dataset split statistics
//!
//! Run: cargo run -p bashrs --example ssc_data_pipeline

#![allow(clippy::unwrap_used)]

use bashrs::corpus::baselines::corpus_baseline_entries;
use bashrs::corpus::dataset::{split_and_validate, ClassificationRow};
use bashrs::corpus::model_card;
use bashrs::corpus::training_config;

fn main() {
    println!("=== SSC v11 Data Pipeline ===\n");

    // Step 1: Dataset overview
    println!("--- Step 1: Dataset Overview ---");
    let entries = corpus_baseline_entries();
    let total = entries.len();
    let safe = entries.iter().filter(|(_, l)| *l == 0).count();
    let unsafe_count = entries.iter().filter(|(_, l)| *l == 1).count();
    println!("  Total entries: {total}");
    println!("  Safe (0): {safe}");
    println!("  Unsafe (1): {unsafe_count}");
    println!(
        "  Imbalance: {:.1}% unsafe\n",
        100.0 * unsafe_count as f64 / total as f64
    );

    // Step 2: Split dataset
    println!("--- Step 2: Train/Val/Test Split ---");
    let rows: Vec<ClassificationRow> = entries
        .into_iter()
        .map(|(input, label)| ClassificationRow { input, label })
        .collect();
    let split = split_and_validate(rows, 2);
    println!(
        "  Train: {} ({} unsafe)",
        split.train.len(),
        split.train.iter().filter(|r| r.label == 1).count()
    );
    println!(
        "  Val:   {} ({} unsafe)",
        split.val.len(),
        split.val.iter().filter(|r| r.label == 1).count()
    );
    println!(
        "  Test:  {} ({} unsafe)\n",
        split.test.len(),
        split.test.iter().filter(|r| r.label == 1).count()
    );

    // Step 3: Training config
    println!("--- Step 3: Training Configuration ---");
    let config = training_config::generate_training_config();
    println!("  Model: {} ({})", config.model.base_model, config.model.architecture);
    println!("  Classes: {}", config.model.num_classes);
    println!("  Epochs: {}", config.training.epochs);
    println!("  Batch size: {}", config.training.batch_size);
    println!("  LR: {}", config.training.learning_rate);
    println!(
        "  Class weights: safe={:.3}, unsafe={:.3}",
        config.training.class_weights[0], config.training.class_weights[1]
    );
    println!("  Target MCC CI lower: {}", config.evaluation.mcc_ci_lower_target);
    println!("  Target accuracy: {}\n", config.evaluation.accuracy_target);

    // Step 4: Model card preview
    println!("--- Step 4: Model Card (first 20 lines) ---");
    let card = model_card::generate_model_card();
    for (i, line) in card.lines().enumerate() {
        if i >= 20 {
            println!("  ... ({} more lines)", card.lines().count() - 20);
            break;
        }
        println!("  {line}");
    }

    println!("\n=== Pipeline: dataset -> split -> config -> model card ===");
    println!("Use `bashrs corpus export-splits`, `bashrs corpus training-config`,");
    println!("and `bashrs corpus model-card` CLI commands.");
}
