//! Example: Tokenizer validation protocol for CodeBERT (SSC v11 Section 5.2).
//!
//! Validates that a BPE tokenizer handles shell constructs adequately.
//! Uses real CodeBERT tokenizer if files are available at /tmp/codebert-base/,
//! otherwise falls back to whitespace tokenizer baseline.
//!
//! Run: cargo run -p bashrs --example tokenizer_validation
//! With CodeBERT: cargo run -p bashrs --features ml --example tokenizer_validation

#![allow(clippy::unwrap_used)]

use bashrs::corpus::tokenizer_validation::{
    run_validation, shell_constructs, TokenizerValidationReport,
};

fn print_report_summary(report: &TokenizerValidationReport) {
    println!("Total constructs: {}", report.total_constructs);
    println!(
        "Acceptable:       {} ({:.1}%)",
        report.acceptable_count, report.acceptable_pct
    );
    println!("Unacceptable:     {}", report.unacceptable_count);
    println!("Target:           >= 70% (C-TOK-001)");
    println!(
        "Status:           {}",
        if report.passed { "PASSED" } else { "FAILED" }
    );
}

fn print_detailed_results(report: &TokenizerValidationReport) {
    println!("\n--- Per-Construct Results ---\n");
    for r in &report.results {
        let status = if r.acceptable { "PASS" } else { "FAIL" };
        let tokens_str = if r.tokens.len() > 8 {
            format!(
                "{} ... ({} tokens)",
                r.tokens[..4].join(", "),
                r.tokens.len()
            )
        } else {
            r.tokens.join(", ")
        };
        println!(
            "  [{}] {:8} {:30} tokens=[{}]",
            status, r.id, r.construct, tokens_str
        );
        if !r.acceptable {
            println!("           Reason: {}", r.reason);
        }
    }
}

#[cfg(feature = "ml")]
fn try_codebert_validation() -> Option<TokenizerValidationReport> {
    let vocab_path = std::path::Path::new("/tmp/codebert-base/vocab.json");
    let merges_path = std::path::Path::new("/tmp/codebert-base/merges.txt");

    if !vocab_path.exists() || !merges_path.exists() {
        println!("\nCodeBERT files not found at /tmp/codebert-base/");
        println!("Download with: huggingface-cli download microsoft/codebert-base vocab.json merges.txt config.json --local-dir /tmp/codebert-base");
        println!("Falling back to whitespace tokenizer.\n");
        return None;
    }

    println!("\n--- CodeBERT Tokenizer (real, 50265 vocab) ---\n");
    match bashrs::corpus::tokenizer_validation::validate_codebert_tokenizer(vocab_path, merges_path)
    {
        Ok(report) => Some(report),
        Err(e) => {
            println!("CodeBERT load failed: {e}");
            println!("Falling back to whitespace tokenizer.\n");
            None
        }
    }
}

fn whitespace_baseline() -> TokenizerValidationReport {
    println!("--- Whitespace Tokenizer (baseline) ---\n");
    run_validation(|construct| {
        construct
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    })
}

fn main() {
    println!("=== SSC v11 Tokenizer Validation (Section 5.2, C-TOK-001) ===\n");

    let constructs = shell_constructs();
    println!(
        "Shell constructs to validate ({} total):\n",
        constructs.len()
    );
    for c in &constructs {
        println!("  {}: {:30} — {}", c.id, c.construct, c.description);
    }

    #[cfg(feature = "ml")]
    {
        if let Some(report) = try_codebert_validation() {
            print_report_summary(&report);
            print_detailed_results(&report);
            return;
        }
    }

    #[cfg(not(feature = "ml"))]
    {
        println!("\nNote: Build with --features ml to use real CodeBERT tokenizer.");
        println!("Falling back to whitespace tokenizer.\n");
    }

    let report = whitespace_baseline();
    print_report_summary(&report);
    print_detailed_results(&report);
}
