//! Example: Natural-language safety explanation for shell scripts.
//!
//! Demonstrates the `bashrs explain` pipeline that generates human-readable
//! safety explanations from linter findings (SSC v11 Section 8.1).
//!
//! Run: cargo run -p bashrs --example explain_demo

#![allow(clippy::unwrap_used)]

fn main() {
    println!("=== Shell Safety Explanation (SSC v11 Section 8.1) ===\n");

    let scripts = [
        ("safe", "#!/bin/sh\necho \"hello world\"\n"),
        (
            "injection-risk",
            "#!/bin/bash\neval \"$user_input\"\n",
        ),
        (
            "non-deterministic",
            "#!/bin/bash\necho $RANDOM\navail=$(df -m / | awk 'NR==2{print $4}')\n",
        ),
        (
            "non-idempotent",
            "#!/bin/sh\nmkdir /tmp/build\nln /tmp/a /tmp/b\n",
        ),
        (
            "multi-issue",
            "#!/bin/bash\neval \"$1\"\neval \"$2\"\neval \"$3\"\nmkdir /tmp/dir\n",
        ),
    ];

    for (name, script) in &scripts {
        println!("--- Script: {name} ---");
        println!("```");
        print!("{script}");
        println!("```\n");

        // Run linter
        let result = bashrs::linter::lint_shell(script);

        // Categorize findings
        let sec = result
            .diagnostics
            .iter()
            .filter(|d| d.code.starts_with("SEC"))
            .count();
        let det = result
            .diagnostics
            .iter()
            .filter(|d| d.code.starts_with("DET"))
            .count();
        let idem = result
            .diagnostics
            .iter()
            .filter(|d| d.code.starts_with("IDEM"))
            .count();
        let other = result.diagnostics.len() - sec - det - idem;

        // Determine risk level
        let risk = if sec >= 3 {
            "CRITICAL"
        } else if sec > 0 {
            "HIGH"
        } else if det > 0 {
            "MEDIUM"
        } else if idem > 0 {
            "LOW"
        } else if result.diagnostics.is_empty() {
            "SAFE"
        } else {
            "LOW"
        };

        println!("  Risk Level: {risk}");
        println!(
            "  Findings: {} total ({sec} security, {det} determinism, {idem} idempotency, {other} other)",
            result.diagnostics.len()
        );

        // Print explanations for SEC/DET/IDEM findings
        for d in &result.diagnostics {
            if d.code.starts_with("SEC")
                || d.code.starts_with("DET")
                || d.code.starts_with("IDEM")
            {
                println!("  [{}] L{}: {}", d.code, d.span.start_line, d.message);
            }
        }
        println!();
    }

    println!("Use `bashrs explain <file>` for full natural-language explanations.");
    println!("Use `bashrs explain <file> --json` for machine-readable output.");
}
