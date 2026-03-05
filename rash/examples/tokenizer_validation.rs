//! Example: Tokenizer validation protocol for CodeBERT (SSC v11 Section 5.2).
//!
//! Validates that a BPE tokenizer handles shell constructs adequately.
//! Uses a simple whitespace tokenizer as a stand-in for the real BPE tokenizer.
//!
//! Run: cargo run -p bashrs --example tokenizer_validation

#![allow(clippy::unwrap_used)]

use bashrs::corpus::tokenizer_validation::{run_validation, shell_constructs};

fn main() {
    println!("=== SSC v11 Tokenizer Validation (Section 5.2, C-TOK-001) ===\n");

    // Show the 20 critical constructs
    let constructs = shell_constructs();
    println!("Shell constructs to validate ({} total):\n", constructs.len());
    for c in &constructs {
        println!("  {}: {:30} — {}", c.id, c.construct, c.description);
    }

    // Run with a simple whitespace tokenizer (stand-in for BPE)
    println!("\n--- Whitespace Tokenizer (baseline) ---\n");
    let report = run_validation(|construct| {
        construct
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    });

    println!("Total constructs: {}", report.total_constructs);
    println!("Acceptable:       {} ({:.1}%)", report.acceptable_count, report.acceptable_pct);
    println!("Unacceptable:     {}", report.unacceptable_count);
    println!("Target:           >= 70% (C-TOK-001)");
    println!(
        "Status:           {}",
        if report.passed { "PASSED" } else { "FAILED" }
    );

    // Show per-construct results
    println!("\n--- Per-Construct Results ---\n");
    for r in &report.results {
        let status = if r.acceptable { "PASS" } else { "FAIL" };
        println!(
            "  [{}] {:8} {:30} tokens=[{}]",
            status,
            r.id,
            r.construct,
            r.tokens.join(", ")
        );
        if !r.acceptable {
            println!("           Reason: {}", r.reason);
        }
    }

    // Now run with a character-level tokenizer (should fail)
    println!("\n--- Character Tokenizer (should fail) ---\n");
    let bad_report = run_validation(|construct| {
        construct.chars().map(|c| c.to_string()).collect()
    });

    println!(
        "Acceptable: {} ({:.1}%) — {}",
        bad_report.acceptable_count,
        bad_report.acceptable_pct,
        if bad_report.passed { "PASSED" } else { "FAILED (expected)" }
    );
}
