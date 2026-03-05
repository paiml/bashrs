//! Example: Run generalization test scripts against the rule-based classifier.
//!
//! Demonstrates SSC v11 Section 5.6: 50 hand-written unsafe scripts for
//! out-of-distribution testing. Shows what the linter catches vs. misses.
//!
//! Run: cargo run -p bashrs --example generalization_tests

#![allow(clippy::unwrap_used)]

use bashrs::corpus::generalization_tests::{category_summary, generalization_tests};
use bashrs::linter::lint_shell;

fn main() {
    println!("=== SSC v11 Generalization Tests (Section 5.6, F8 Mitigation) ===\n");

    let tests = generalization_tests();
    let mut caught = 0;
    let mut missed = 0;

    for t in &tests {
        let result = lint_shell(t.script);
        let has_findings = !result.diagnostics.is_empty();
        let has_security = result.diagnostics.iter().any(|d| d.code.starts_with("SEC"));
        let detected = has_security || has_findings;

        if detected {
            caught += 1;
        } else {
            missed += 1;
            println!("  MISSED: {} [{}] — {}", t.id, t.category, t.description);
            println!("    Script: {}", t.script.lines().next().unwrap_or(""));
        }
    }

    println!("\n=== Results ===\n");
    println!("  Total:   {}", tests.len());
    println!(
        "  Caught:  {} ({:.1}%)",
        caught,
        caught as f64 / tests.len() as f64 * 100.0
    );
    println!(
        "  Missed:  {} ({:.1}%)",
        missed,
        missed as f64 / tests.len() as f64 * 100.0
    );
    println!("  Target:  >= 50% (SSC v11 Section 5.5: generalization >= 50%)");
    println!(
        "  Status:  {}",
        if caught as f64 / tests.len() as f64 >= 0.5 {
            "PASSED"
        } else {
            "FAILED"
        }
    );

    println!("\n=== Category Breakdown ===\n");
    let summary = category_summary(&tests);
    for (category, count) in &summary {
        // Count how many in this category were caught
        let cat_caught = tests
            .iter()
            .filter(|t| t.category == *category)
            .filter(|t| !lint_shell(t.script).diagnostics.is_empty())
            .count();
        println!(
            "  {:<18} {}/{} caught ({:.0}%)",
            category,
            cat_caught,
            count,
            if *count > 0 {
                cat_caught as f64 / *count as f64 * 100.0
            } else {
                0.0
            }
        );
    }
}
