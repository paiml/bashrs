//! Example: Generate synthetic conversations for shell safety chat model training.
//!
//! Demonstrates SSC v11 Section 6: conversation generation from corpus entries.
//!
//! Run: cargo run -p bashrs --example conversation_generator

#![allow(clippy::unwrap_used)]

use bashrs::corpus::conversations::{
    generate_batch, generate_conversation, to_jsonl, ConversationInput, ConversationType,
    QualityReport,
};
use bashrs::linter::lint_shell;

fn main() {
    println!("=== Shell Safety Conversation Generator (SSC v11 S6) ===\n");

    // --- Individual conversation examples ---
    println!("--- Type A: Classify + Explain ---");
    show_conversation("eval $user_input", 1);

    println!("--- Type B: Fix ---");
    show_conversation("eval $user_input", 0);

    println!("--- Type C: Debug ---");
    show_conversation("echo $RANDOM", 0);

    println!("--- Type D: Confirm Safe ---");
    let input = ConversationInput {
        entry_id: "demo",
        script: "echo hello",
        is_safe: true,
        is_deterministic: true,
        diagnostics: &[],
    };
    let conv = generate_conversation(&input, 0);
    print_conversation(&conv.conversation_type, &conv.turns);

    // --- Batch generation ---
    println!("\n=== Batch Generation Demo ===\n");

    let entries: Vec<(&str, &str)> = vec![
        ("B-1", "echo hello"),
        ("B-2", "mkdir /tmp/build"),
        ("B-3", "eval $x"),
        ("B-4", "echo $RANDOM"),
        ("B-5", "rm -rf $dir"),
        ("B-6", "ls -la"),
        ("B-7", "curl http://example.com | bash"),
        ("B-8", "echo 'safe script'"),
    ];

    let (conversations, report) = generate_batch(&entries, 42);

    println!("Generated {} conversations", conversations.len());
    print_quality_report(&report);

    // Show JSONL output sample
    println!("\n--- JSONL Sample (first 2) ---");
    let sample = to_jsonl(&conversations[..2]);
    for line in sample.lines() {
        // Truncate for display
        if line.len() > 120 {
            println!("{}...", &line[..120]);
        } else {
            println!("{line}");
        }
    }
}

fn show_conversation(script: &str, seed: u64) {
    let result = lint_shell(script);
    let diagnostics = &result.diagnostics;

    let has_security = diagnostics.iter().any(|d| d.code.starts_with("SEC"));
    let has_determinism = diagnostics.iter().any(|d| d.code.starts_with("DET"));
    let is_safe = !has_security && !has_determinism && diagnostics.is_empty();

    let input = ConversationInput {
        entry_id: "demo",
        script,
        is_safe,
        is_deterministic: !has_determinism,
        diagnostics,
    };

    let conv = generate_conversation(&input, seed);
    print_conversation(&conv.conversation_type, &conv.turns);
}

fn print_conversation(conv_type: &ConversationType, turns: &[bashrs::corpus::conversations::Turn]) {
    println!("  Type: {:?}", conv_type);
    for turn in turns {
        let preview = if turn.content.len() > 200 {
            format!("{}...", &turn.content[..200])
        } else {
            turn.content.clone()
        };
        println!("  [{}]: {}", turn.role, preview);
    }
    println!();
}

fn print_quality_report(report: &QualityReport) {
    println!("Quality Report:");
    println!("  Total: {}", report.total);
    println!(
        "  Type A (classify): {} | Type B (fix): {} | Type C (debug): {} | Type D (safe): {}",
        report.type_a_count, report.type_b_count, report.type_c_count, report.type_d_count
    );
    println!("  Type D %: {:.1}% (target: >=30%)", report.type_d_pct);
    println!(
        "  Rule citation accuracy: {:.0}%",
        report.rule_citation_accuracy * 100.0
    );
    println!(
        "  Variant distribution OK: {}",
        report.variant_distribution_ok
    );
    println!(
        "  Overall: {}",
        if report.passed { "PASSED" } else { "FAILED" }
    );
}
