//! # Shell Safety Classifier Example
//!
//! Demonstrates bashrs's rule-based shell safety classification pipeline.
//! Classifies scripts as safe, needs-quoting, non-deterministic, non-idempotent,
//! or unsafe using the built-in linter rules (SEC, DET, IDEM).
//!
//! ## Usage
//!
//! ```bash
//! cargo run -p bashrs --example shell_safety_classifier
//! ```
//!
//! ## Architecture (SSC v11 -- shell-safety-inference.md)
//!
//! The rule-based classifier is Stage 0 of the three-stage pipeline:
//!
//! ```text
//! Stage 0: Rule-based linter (this example) -- <1ms, zero dependencies
//! Stage 1: CodeBERT classifier (125M encoder) -- ~20ms, WASM-deployable
//! Stage 2: Qwen-1.5B chat model -- ~2s, explains findings in natural language
//! ```

#![allow(clippy::unwrap_used)]

use bashrs::corpus::dataset::{derive_safety_label, SAFETY_LABELS};
use bashrs::linter::lint_shell;

/// Classify a shell script and print the result.
fn classify_and_print(name: &str, script: &str) {
    let result = lint_shell(script);
    let diagnostics = &result.diagnostics;

    let has_security = diagnostics.iter().any(|d| d.code.starts_with("SEC"));
    let has_determinism = diagnostics.iter().any(|d| d.code.starts_with("DET"));

    let label_idx = derive_safety_label(script, true, !has_security, !has_determinism);
    let label = SAFETY_LABELS[label_idx as usize];

    println!("  [{name}]");
    println!("    Label: {label} (class {label_idx})");
    println!("    Diagnostics: {}", diagnostics.len());

    if !diagnostics.is_empty() {
        for d in diagnostics.iter().take(3) {
            println!("      - {}: {} (line {})", d.code, d.message, d.span.start_line);
        }
        if diagnostics.len() > 3 {
            println!("      ... and {} more", diagnostics.len() - 3);
        }
    }
    println!();
}

fn main() {
    println!("=== Shell Safety Classifier (Rule-Based) ===\n");

    classify_and_print(
        "safe",
        "#!/bin/sh\nset -euf\necho \"Hello, World!\"\nmkdir -p /tmp/build\n",
    );

    classify_and_print(
        "needs-quoting",
        "#!/bin/sh\necho $HOME\ncd $WORKSPACE\n",
    );

    classify_and_print(
        "non-deterministic",
        "#!/bin/bash\ntoken=$RANDOM\necho \"Session: $token\"\n",
    );

    classify_and_print(
        "non-idempotent",
        "#!/bin/sh\nmkdir /tmp/build\nln -s /usr/bin/app /usr/local/bin/app\n",
    );

    classify_and_print(
        "unsafe-eval",
        "#!/bin/bash\neval \"$user_input\"\n",
    );

    classify_and_print(
        "unsafe-curl-pipe",
        "#!/bin/bash\ncurl -sSL https://example.com/install.sh | bash\n",
    );

    println!("=== Classification Priority ===");
    println!("  unsafe > non-deterministic > non-idempotent > needs-quoting > safe");
    println!();
    println!("=== SSC v11 Pipeline ===");
    println!("  Stage 0: Rule-based linter (this example)  -- <1ms, built-in");
    println!("  Stage 1: CodeBERT classifier (125M)        -- ~20ms, WASM-deployable");
    println!("  Stage 2: Qwen-1.5B chat model              -- ~2s, explains + fixes");
    println!();
    println!("See docs/specifications/shell-safety-inference.md for full spec.");
}
