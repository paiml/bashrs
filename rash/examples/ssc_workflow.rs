//! Example: Full SSC v11 workflow (classify -> explain -> fix).
//!
//! Demonstrates the complete Stage 0 (rule-based) pipeline:
//! 1. Classify: determine safety label and confidence
//! 2. Explain: generate natural-language safety analysis
//! 3. Fix: apply auto-fixes to remediate issues
//!
//! Run: cargo run -p bashrs --example ssc_workflow

#![allow(clippy::unwrap_used)]

use bashrs::corpus::dataset::{derive_safety_label, SAFETY_LABELS};
use bashrs::linter::{autofix, lint_shell};

fn main() {
    println!("=== SSC v11 Stage 0 Workflow ===\n");

    let script = r#"#!/bin/bash
eval "$user_input"
echo $RANDOM
mkdir /tmp/build
echo $HOME
"#;

    println!("Input script:");
    println!("```");
    print!("{script}");
    println!("```\n");

    // Step 1: Classify
    println!("--- Step 1: Classify ---");
    let result = lint_shell(script);
    let lint_clean = !result.diagnostics.iter().any(|d| d.code.starts_with("SEC"));
    let deterministic = !result.diagnostics.iter().any(|d| d.code.starts_with("DET"));
    let label_idx = derive_safety_label(script, true, lint_clean, deterministic);
    let label = SAFETY_LABELS[label_idx as usize];
    println!("  Label: {label} (class {label_idx})");
    println!("  Findings: {}", result.diagnostics.len());
    println!();

    // Step 2: Explain
    println!("--- Step 2: Explain ---");
    let sec_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code.starts_with("SEC"))
        .count();
    let det_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code.starts_with("DET"))
        .count();
    let idem_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code.starts_with("IDEM"))
        .count();

    let risk = if sec_count >= 3 {
        "CRITICAL"
    } else if sec_count > 0 {
        "HIGH"
    } else if det_count > 0 {
        "MEDIUM"
    } else if idem_count > 0 {
        "LOW"
    } else {
        "SAFE"
    };

    println!("  Risk: {risk}");
    println!("  Security: {sec_count}, Determinism: {det_count}, Idempotency: {idem_count}");
    for d in &result.diagnostics {
        if d.code.starts_with("SEC") || d.code.starts_with("DET") || d.code.starts_with("IDEM") {
            println!("  [{}] L{}: {}", d.code, d.span.start_line, d.message);
        }
    }
    println!();

    // Step 3: Fix
    println!("--- Step 3: Fix ---");
    let options = autofix::FixOptions {
        dry_run: false,
        apply_assumptions: true,
        create_backup: false,
        ..Default::default()
    };
    let fix_result = autofix::apply_fixes(script, &result, &options).unwrap();
    println!("  Fixes applied: {}", fix_result.fixes_applied);
    if let Some(ref fixed) = fix_result.modified_source {
        println!("\nFixed script:");
        println!("```");
        print!("{fixed}");
        println!("```");

        // Re-classify the fixed version
        let fixed_result = lint_shell(fixed);
        let fixed_lint_clean = !fixed_result
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("SEC"));
        let fixed_det = !fixed_result
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("DET"));
        let fixed_label_idx = derive_safety_label(fixed, true, fixed_lint_clean, fixed_det);
        let fixed_label = SAFETY_LABELS[fixed_label_idx as usize];
        println!("\n  After fix: {fixed_label} (was: {label})");
        println!("  Remaining findings: {}", fixed_result.diagnostics.len());
    }

    println!("\n=== Pipeline: classify -> explain -> fix -> re-classify ===");
    println!("Use `bashrs classify`, `bashrs explain`, `bashrs fix` CLI commands.");
}
