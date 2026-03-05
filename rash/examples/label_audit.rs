//! Example: Audit safety labels for accuracy (SSC v11 Section 5.3, F7).
//!
//! Validates that "unsafe" labels are genuinely unsafe (C-LABEL-001: >=90%).
//!
//! Run: cargo run -p bashrs --example label_audit

#![allow(clippy::unwrap_used)]

use bashrs::corpus::label_audit::run_corpus_label_audit;

fn main() {
    println!("=== SSC v11 Label Audit (Section 5.3, C-LABEL-001) ===\n");

    let report = run_corpus_label_audit(100);

    println!("Audited {} unsafe labels:", report.total_audited);
    println!(
        "  Genuinely unsafe: {} ({:.1}%)",
        report.genuinely_unsafe, report.accuracy_pct
    );
    println!("  False positives:  {}", report.false_positives);
    println!("  Target:           >= 90% (C-LABEL-001)");
    println!(
        "  Status:           {}",
        if report.passed { "PASSED" } else { "FAILED" }
    );

    // Show false positives (if any)
    let false_pos: Vec<_> = report
        .results
        .iter()
        .filter(|r| !r.genuinely_unsafe)
        .collect();

    if !false_pos.is_empty() {
        println!("\n--- False Positives ---\n");
        for r in false_pos.iter().take(10) {
            println!("  {} — {}", r.entry_id, r.reason);
            let preview = if r.script.len() > 60 {
                format!("{}...", &r.script[..60])
            } else {
                r.script.clone()
            };
            println!("    Script: {}", preview);
        }
    }

    // Show signal distribution
    println!("\n--- Signal Distribution ---\n");
    let mut signal_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for r in &report.results {
        for s in &r.signals {
            *signal_counts.entry(s.clone()).or_default() += 1;
        }
    }
    let mut sorted: Vec<_> = signal_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (signal, count) in sorted.iter().take(15) {
        println!("  {:<35} {}", signal, count);
    }
}
