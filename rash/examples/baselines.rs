//! Example: Run SSC v11 baseline classifiers (Section 5.5).
//!
//! Compares majority class, keyword regex, and linter baselines.
//! Any ML classifier must beat all three on MCC.
//!
//! Run: cargo run -p bashrs --example baselines

#![allow(clippy::unwrap_used)]

use bashrs::corpus::baselines::{corpus_baseline_entries, run_all_baselines};
use bashrs::corpus::evaluation::{format_comparison, format_report};

fn main() {
    println!("=== SSC v11 Baseline Comparison (Section 5.5) ===\n");

    let owned = corpus_baseline_entries();
    let entries: Vec<(&str, u8)> = owned.iter().map(|(s, l)| (s.as_str(), *l)).collect();

    let safe_count = entries.iter().filter(|(_, l)| *l == 0).count();
    let unsafe_count = entries.iter().filter(|(_, l)| *l == 1).count();
    println!(
        "Dataset: {} entries ({} safe, {} unsafe)\n",
        entries.len(),
        safe_count,
        unsafe_count
    );

    let reports = run_all_baselines(&entries);

    // Side-by-side comparison
    print!("{}", format_comparison(&reports));
    println!();

    // Detailed per-baseline reports
    for report in &reports {
        println!("--- {} ---", report.name);
        print!("{}", format_report(report));
        println!();
    }

    // Contract thresholds
    println!("Contract C-CLF-001 Thresholds:");
    println!("  MCC CI lower > 0.2");
    println!("  Accuracy > 93.5%");
    println!("  Generalization >= 50%");
    println!();
    println!("Any ML classifier must beat ALL three baselines on MCC.");
}
