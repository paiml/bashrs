//! Example: Binary classification evaluation metrics (SSC v11 Section 5.5).
//!
//! Demonstrates MCC, accuracy, precision, recall, F1, and confusion matrix
//! on synthetic prediction data.
//!
//! Run: cargo run -p bashrs --example evaluation_metrics

#![allow(clippy::unwrap_used)]

use bashrs::corpus::evaluation::{evaluate, format_comparison, format_report};

fn main() {
    println!("=== SSC v11 Evaluation Metrics (Section 5.5) ===\n");

    // Perfect classifier
    let perfect = evaluate(
        &[(1, 1), (0, 0), (1, 1), (0, 0), (1, 1), (0, 0)],
        "perfect",
    );

    // Good classifier (1 false positive, 1 false negative)
    let good = evaluate(
        &[(1, 1), (0, 0), (1, 0), (0, 1), (1, 1), (0, 0)],
        "good (2 errors)",
    );

    // All-safe baseline (majority class)
    let majority = evaluate(
        &[(0, 1), (0, 0), (0, 1), (0, 0), (0, 1), (0, 0)],
        "majority (all-safe)",
    );

    // Random-ish classifier
    let random = evaluate(
        &[(1, 0), (0, 1), (1, 1), (0, 0), (1, 0), (0, 1)],
        "random",
    );

    let reports = [&perfect, &good, &majority, &random];

    // Side-by-side comparison
    println!("{}", format_comparison(&reports.iter().map(|r| (*r).clone()).collect::<Vec<_>>()));
    println!();

    // Detailed reports
    for report in &reports {
        println!("--- {} ---", report.name);
        print!("{}", format_report(report));
        println!();
    }

    // Explain MCC
    println!("=== Understanding MCC ===\n");
    println!("MCC (Matthews Correlation Coefficient) ranges from -1 to +1:");
    println!("  +1.0: Perfect prediction");
    println!("   0.0: Random guessing (or always predicting same class)");
    println!("  -1.0: Total disagreement\n");
    println!("SSC v11 Contract C-CLF-001 requires MCC CI lower bound > 0.2");
    println!("This means the classifier must be significantly better than random.");
}
