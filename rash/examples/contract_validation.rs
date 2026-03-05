//! Example: Run all SSC v11 contract validations (pre-training gate).
//!
//! Checks C-TOK-001, C-LABEL-001, baselines, generalization, and dataset splits.
//! ALL must pass before proceeding to classifier training.
//!
//! Run: cargo run -p bashrs --example contract_validation

#![allow(clippy::unwrap_used)]

use bashrs::corpus::contract_validation::run_all_contracts;

fn main() {
    println!("=== SSC v11 Contract Validation (Pre-Training Gate) ===\n");

    let report = run_all_contracts();

    for c in &report.contracts {
        let status = if c.passed { "PASS" } else { "FAIL" };
        println!(
            "  [{status}] {:<15} {:<25} value={:.1} threshold={:.1}",
            c.id, c.name, c.value, c.threshold
        );
        println!("         {}", c.detail);
    }

    println!();
    println!(
        "Result: {}/{} contracts passed",
        report.passed_count,
        report.contracts.len()
    );

    if report.all_passed {
        println!("\nAll contracts passed. Ready for classifier training.");
    } else {
        println!("\nSome contracts FAILED. Fix before proceeding to training.");
        std::process::exit(1);
    }
}
