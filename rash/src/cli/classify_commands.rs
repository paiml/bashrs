//! Shell/Makefile/Dockerfile safety classification command (SSC-019, SSC-021, SSC-022)
//!
//! Classifies scripts into 5 safety categories using linter-based analysis:
//! - **safe** (0): Passes all checks, properly quoted
//! - **needs-quoting** (1): Unquoted variable expansions
//! - **non-deterministic** (2): Contains $RANDOM, timestamps, wildcards without sort
//! - **non-idempotent** (3): mkdir without -p, rm without -f, etc.
//! - **unsafe** (4): Security violations (eval, command injection, running as root)
//!
//! Supports bash, Makefile, and Dockerfile formats (SSC-022).
//! Format auto-detected from file extension or forced via `--format`.

use crate::cli::args::ClassifyFormat;
use crate::corpus::dataset::{derive_safety_label, SAFETY_LABELS};
use crate::linter::{lint_dockerfile_with_profile, lint_makefile, lint_shell, LintProfile};
use crate::models::{Error, Result};
use std::path::Path;

/// Single-label classification result for a script.
#[derive(Debug, serde::Serialize)]
struct ClassifyResult {
    /// Safety class label
    label: String,
    /// Safety class index (0-4)
    index: u8,
    /// Confidence score (0.0-1.0)
    confidence: f64,
    /// Per-class scores (probabilities)
    scores: [f64; 5],
    /// Detected format
    format: String,
    /// Number of lint diagnostics
    diagnostics: usize,
    /// Whether script has security violations
    has_security_issues: bool,
    /// Whether script has determinism violations
    has_determinism_issues: bool,
    /// Whether script has idempotency violations
    has_idempotency_issues: bool,
}

/// Multi-label classification result (SSC-021).
#[derive(Debug, serde::Serialize)]
struct MultiLabelClassifyResult {
    /// All active labels
    labels: Vec<String>,
    /// Multi-hot label vector
    label_indices: Vec<u8>,
    /// Per-class confidence scores
    scores: [f64; 5],
    /// Detected format
    format: String,
    /// Number of lint diagnostics
    diagnostics: usize,
    /// Whether script has security violations
    has_security_issues: bool,
    /// Whether script has determinism violations
    has_determinism_issues: bool,
    /// Whether script has idempotency violations
    has_idempotency_issues: bool,
}

/// Detect format from file extension.
fn detect_format(path: &Path) -> ClassifyFormat {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "sh" | "bash" | "zsh" | "ksh" | "dash" => ClassifyFormat::Bash,
        _ => {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            if name == "makefile" || name == "gnumakefile" || name.ends_with(".mk") {
                ClassifyFormat::Makefile
            } else if name == "dockerfile"
                || name.starts_with("dockerfile.")
                || name.ends_with(".dockerfile")
            {
                ClassifyFormat::Dockerfile
            } else {
                // Default to bash
                ClassifyFormat::Bash
            }
        }
    }
}

/// Format name string for output.
fn format_name(fmt: &ClassifyFormat) -> &'static str {
    match fmt {
        ClassifyFormat::Bash => "bash",
        ClassifyFormat::Makefile => "makefile",
        ClassifyFormat::Dockerfile => "dockerfile",
    }
}

/// Run lint analysis and return diagnostic signals by format.
pub(crate) struct LintSignals {
    pub(crate) has_security_issues: bool,
    pub(crate) has_determinism_issues: bool,
    pub(crate) has_idempotency_issues: bool,
    pub(crate) diagnostic_count: usize,
    pub(crate) sec_count: usize,
    pub(crate) det_count: usize,
    pub(crate) _idem_count: usize,
}

pub(crate) fn analyze_lint(source: &str, fmt: &ClassifyFormat) -> LintSignals {
    let diagnostics = match fmt {
        ClassifyFormat::Bash => lint_shell(source).diagnostics,
        ClassifyFormat::Makefile => lint_makefile(source).diagnostics,
        ClassifyFormat::Dockerfile => {
            lint_dockerfile_with_profile(source, LintProfile::Standard).diagnostics
        }
    };

    // Map format-specific rule prefixes to SEC/DET/IDEM categories
    let sec_count = diagnostics
        .iter()
        .filter(|d| {
            d.code.starts_with("SEC")
                || d.code == "MAKE003"   // shell injection in recipes
                || d.code == "DOCKER001" // running as root
                || d.code == "DOCKER006" // ADD instead of COPY
        })
        .count();

    let det_count = diagnostics
        .iter()
        .filter(|d| {
            d.code.starts_with("DET")
                || d.code == "MAKE001"   // non-deterministic wildcard
                || d.code == "DOCKER002" // unpinned base image (:latest)
        })
        .count();

    let idem_count = diagnostics
        .iter()
        .filter(|d| {
            d.code.starts_with("IDEM") || d.code == "MAKE002" // missing .PHONY
        })
        .count();

    LintSignals {
        has_security_issues: sec_count > 0,
        has_determinism_issues: det_count > 0,
        has_idempotency_issues: idem_count > 0,
        diagnostic_count: diagnostics.len(),
        sec_count,
        det_count,
        _idem_count: idem_count,
    }
}

/// Run the classify command on a script file.
#[allow(unused_variables)]
pub(crate) fn classify_command(
    input: &Path,
    json: bool,
    multi_label: bool,
    forced_format: Option<&ClassifyFormat>,
    probe_path: Option<&Path>,
    mlp_probe_path: Option<&Path>,
    model_path: Option<&Path>,
) -> Result<()> {
    let source = std::fs::read_to_string(input)
        .map_err(|e| Error::Validation(format!("Cannot read {}: {e}", input.display())))?;

    let fmt = forced_format
        .cloned()
        .unwrap_or_else(|| detect_format(input));

    // Stage 1: ML probe classification (MLP preferred over linear)
    let ml_label = match (mlp_probe_path, probe_path, model_path) {
        (Some(mlp), _, Some(model)) => ml_classify_with_mlp_probe(&source, mlp, model),
        (_, Some(probe), Some(model)) => ml_classify_with_probe(&source, probe, model),
        (Some(_), _, None) | (_, Some(_), None) => {
            eprintln!("  Note: --probe/--mlp-probe requires --model for Stage 1 ML classification");
            None
        }
        _ => None,
    };

    if multi_label {
        print_multi_label_result(&source, &fmt, json)?;
    } else {
        print_single_label_result(&source, &fmt, json)?;
    }

    // Print ML probe result if available
    if let Some((label, confidence)) = ml_label {
        if !json {
            println!(
                "  ML (Stage 1): {} (confidence: {:.1}%)",
                if label == 0 { "safe" } else { "unsafe" },
                confidence * 100.0
            );
        }
    }

    Ok(())
}

fn print_multi_label_result(source: &str, fmt: &ClassifyFormat, json: bool) -> Result<()> {
    let result = classify_script_multi_label(source, fmt);
    if json {
        let json_str = serde_json::to_string_pretty(&result)
            .map_err(|e| Error::Validation(format!("JSON serialization failed: {e}")))?;
        println!("{json_str}");
    } else {
        if result.labels.is_empty() {
            println!("safe (no issues detected)");
        } else {
            println!("{}", result.labels.join(" + "));
        }

        if result.diagnostics > 0 {
            println!("  {} lint diagnostic(s) found", result.diagnostics);
        }

        for (i, &score) in result.scores.iter().enumerate() {
            if score > 0.1 {
                println!("  {}: {:.1}%", SAFETY_LABELS[i], score * 100.0);
            }
        }
    }
    Ok(())
}

fn print_single_label_result(source: &str, fmt: &ClassifyFormat, json: bool) -> Result<()> {
    let result = classify_script(source, fmt);
    if json {
        let json_str = serde_json::to_string_pretty(&result)
            .map_err(|e| Error::Validation(format!("JSON serialization failed: {e}")))?;
        println!("{json_str}");
    } else {
        println!(
            "{} (confidence: {:.1}%)",
            result.label,
            result.confidence * 100.0
        );

        if result.diagnostics > 0 {
            println!("  {} lint diagnostic(s) found", result.diagnostics);
        }
        if result.has_security_issues {
            println!("  Security issues detected");
        }
        if result.has_determinism_issues {
            println!("  Determinism issues detected");
        }
        if result.has_idempotency_issues {
            println!("  Idempotency issues detected");
        }
    }
    Ok(())
}

/// Run Stage 1 classification: probe on pre-computed embedding-like features.
/// Classify a script using CodeBERT + trained linear probe.
///
/// With `ml` feature: loads CodeBERT, extracts [CLS] embedding, applies probe.
/// Without `ml` feature: reports that CodeBERT is required.
/// Returns (predicted_label, confidence) or None on error.
#[allow(unused_variables)]
fn ml_classify_with_probe(source: &str, probe_path: &Path, model_path: &Path) -> Option<(u8, f64)> {
    let probe = crate::corpus::classifier::load_probe(probe_path).ok()?;

    #[cfg(not(feature = "ml"))]
    {
        eprintln!("  Note: Stage 1 ML classification requires --features ml");
        None
    }

    #[cfg(feature = "ml")]
    {
        crate::corpus::classifier::classify_with_probe(source, &probe, model_path)
    }
}

/// Run Stage 1 classification using MLP probe on CodeBERT embeddings.
#[allow(unused_variables)]
fn ml_classify_with_mlp_probe(
    source: &str,
    probe_path: &Path,
    model_path: &Path,
) -> Option<(u8, f64)> {
    let weights = crate::corpus::classifier::load_mlp_probe(probe_path).ok()?;

    #[cfg(not(feature = "ml"))]
    {
        eprintln!("  Note: Stage 1 ML classification requires --features ml");
        None
    }

    #[cfg(feature = "ml")]
    {
        crate::corpus::classifier::classify_with_mlp_probe(source, &weights, model_path)
    }
}

/// Classify a script string into a single safety category.
fn classify_script(source: &str, fmt: &ClassifyFormat) -> ClassifyResult {
    let signals = analyze_lint(source, fmt);

    let lint_clean = !signals.has_security_issues;
    let deterministic = !signals.has_determinism_issues;

    let safety_index = derive_safety_label(source, true, lint_clean, deterministic);

    let confidence = compute_confidence(
        safety_index,
        signals.sec_count,
        signals.det_count,
        signals.has_idempotency_issues,
        signals.diagnostic_count,
    );

    let scores = build_score_distribution(safety_index, confidence);

    ClassifyResult {
        label: SAFETY_LABELS[safety_index as usize].to_string(),
        index: safety_index,
        confidence,
        scores,
        format: format_name(fmt).to_string(),
        diagnostics: signals.diagnostic_count,
        has_security_issues: signals.has_security_issues,
        has_determinism_issues: signals.has_determinism_issues,
        has_idempotency_issues: signals.has_idempotency_issues,
    }
}

/// Classify a script with multi-label detection (SSC-021).
fn classify_script_multi_label(source: &str, fmt: &ClassifyFormat) -> MultiLabelClassifyResult {
    let signals = analyze_lint(source, fmt);

    let mut scores = [0.0f64; 5];
    let mut labels = Vec::new();
    let mut label_indices = Vec::new();

    // Class 4: unsafe (security violations)
    if signals.has_security_issues {
        scores[4] = (0.85 + (signals.sec_count as f64 - 1.0).max(0.0) * 0.03).min(0.99);
        labels.push(SAFETY_LABELS[4].to_string());
        label_indices.push(4);
    }

    // Class 2: non-deterministic
    if signals.has_determinism_issues {
        scores[2] = (0.85 + (signals.det_count as f64 - 1.0).max(0.0) * 0.03).min(0.99);
        labels.push(SAFETY_LABELS[2].to_string());
        label_indices.push(2);
    }

    // Class 3: non-idempotent
    let has_idem_patterns = crate::corpus::dataset::has_non_idempotent_pattern(source);
    if signals.has_idempotency_issues || has_idem_patterns {
        scores[3] = if signals.has_idempotency_issues {
            0.90
        } else {
            0.80
        };
        labels.push(SAFETY_LABELS[3].to_string());
        label_indices.push(3);
    }

    // Class 1: needs-quoting (bash-specific; not applicable to Makefile/Dockerfile)
    if matches!(fmt, ClassifyFormat::Bash) {
        let has_unquoted = crate::corpus::dataset::has_unquoted_variable(source);
        if has_unquoted {
            scores[1] = 0.80;
            labels.push(SAFETY_LABELS[1].to_string());
            label_indices.push(1);
        }
    }

    // Class 0: safe (none of the above)
    if labels.is_empty() {
        scores[0] = if signals.diagnostic_count == 0 {
            0.95
        } else {
            0.85
        };
        labels.push(SAFETY_LABELS[0].to_string());
        label_indices.push(0);
    }

    MultiLabelClassifyResult {
        labels,
        label_indices,
        scores,
        format: format_name(fmt).to_string(),
        diagnostics: signals.diagnostic_count,
        has_security_issues: signals.has_security_issues,
        has_determinism_issues: signals.has_determinism_issues,
        has_idempotency_issues: signals.has_idempotency_issues,
    }
}

/// Compute confidence based on signal strength.
fn compute_confidence(
    safety_index: u8,
    sec_count: usize,
    det_count: usize,
    has_idem: bool,
    total_diagnostics: usize,
) -> f64 {
    match safety_index {
        4 => (0.85 + (sec_count as f64 - 1.0).max(0.0) * 0.03).min(0.99),
        2 => (0.85 + (det_count as f64 - 1.0).max(0.0) * 0.03).min(0.99),
        3 => {
            if has_idem {
                0.90
            } else {
                0.80
            }
        }
        1 => 0.80,
        0 => {
            if total_diagnostics == 0 {
                0.95
            } else {
                0.85
            }
        }
        _ => 0.50,
    }
}

/// Build a probability distribution over 5 classes.
fn build_score_distribution(predicted_class: u8, confidence: f64) -> [f64; 5] {
    let mut scores = [0.0f64; 5];
    let remaining = 1.0 - confidence;
    let per_other = remaining / 4.0;

    for (i, score) in scores.iter_mut().enumerate() {
        if i == predicted_class as usize {
            *score = confidence;
        } else {
            *score = per_other;
        }
    }
    scores
}

#[cfg(test)]
#[path = "classify_commands_tests_classify_saf.rs"]
mod tests_extracted;
