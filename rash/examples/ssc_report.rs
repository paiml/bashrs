//! Example: Generate comprehensive SSC v11 readiness report.
//!
//! Shows all SSC sections: corpus, tokenizer, label audit, baselines,
//! generalization, dataset splits, and conversation generation.
//!
//! Run: cargo run -p bashrs --example ssc_report

#![allow(clippy::unwrap_used)]

use bashrs::corpus::ssc_report::{format_ssc_report, generate_ssc_report};

fn main() {
    println!("=== SSC v11 Comprehensive Readiness Report ===\n");

    let report = generate_ssc_report();
    print!("{}", format_ssc_report(&report));

    // Also show JSON for programmatic consumption
    let json = serde_json::to_string_pretty(&report).unwrap();
    println!("--- JSON ---");
    println!("{json}");
}
