//! Example: Combined safety check (lint + classify) on shell scripts.
//!
//! Demonstrates the `bashrs safety-check` pipeline that combines linter
//! findings with rule-based classification in a single pass.
//!
//! Run: cargo run -p bashrs --example safety_check

#![allow(clippy::unwrap_used)]

fn main() {
    println!("=== Shell Safety Check (SSC v11 Section 8.2) ===\n");

    let scripts = [
        ("safe", "#!/bin/sh\necho \"hello world\"\n"),
        ("needs-quoting", "#!/bin/sh\necho $HOME\n"),
        ("non-deterministic", "#!/bin/bash\necho $RANDOM\n"),
        ("non-idempotent", "#!/bin/sh\nmkdir /tmp/build\n"),
        ("unsafe-eval", "#!/bin/bash\neval \"$user_input\"\n"),
        (
            "multi-issue",
            "#!/bin/bash\neval \"$RANDOM\"\nmkdir /tmp/dir\n",
        ),
    ];

    for (name, script) in &scripts {
        println!("  [{name}]");

        // Run lint
        let result = bashrs::linter::lint_shell(script);
        let findings_count = result.diagnostics.len();

        // Classify
        let lint_clean = !result
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("SEC"));
        let deterministic = !result
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("DET"));
        let label_idx =
            bashrs::corpus::dataset::derive_safety_label(script, true, lint_clean, deterministic);
        let label = bashrs::corpus::dataset::SAFETY_LABELS[label_idx as usize];
        let binary = if label_idx == 0 { "safe" } else { "unsafe" };

        println!("    Binary: {binary}");
        println!("    Detailed: {label} (class {label_idx})");
        println!("    Findings: {findings_count}");

        for d in &result.diagnostics {
            println!(
                "      {} L{}: {}",
                d.code, d.span.start_line, d.message
            );
        }
        println!();
    }
}
