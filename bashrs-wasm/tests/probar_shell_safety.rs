//! Probar Shell Safety Test Suite — SSC v11 Phase 6 (PRB-001)
//!
//! Three test layers:
//!   Layer 1: WASM Logic (no browser) — correctness
//!   Layer 2: Docker Cross-Browser — compatibility (feature-gated)
//!   Layer 3: Performance Benchmarks — latency budgets
//!
//! Contracts verified:
//!   C-PRB-001: Layer 1 (logic): 12+ WASM tests pass without browser
//!   C-PRB-003: Layer 3 (performance): linter <10ms budget
//!   C-PRB-007: Determinism: repeated calls produce identical results

use jugar_probar::Assertion;
use std::time::{Duration, Instant};

// Re-export WASM functions for native testing (rlib target)
use bashrs_wasm::{
    bashrs_version, classify_shell_wasm, explain_shell_wasm, lint_dockerfile_wasm,
    lint_makefile_wasm, lint_shell_wasm,
};

// ═══════════════════════════════════════════════════════════════
// Layer 1: WASM Logic Tests (no browser, deterministic)
// ═══════════════════════════════════════════════════════════════

// --- Linter WASM correctness ---

#[test]
fn test_prb001_linter_wasm_returns_findings_for_unsafe_script() {
    let result = lint_shell_wasm("eval \"$user_input\"");
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    let diags = parsed["diagnostics"].as_array().expect("diagnostics array");
    assert!(!diags.is_empty(), "Unsafe script must produce findings");
    assert!(
        diags
            .iter()
            .any(|d| d["code"].as_str().unwrap_or("").starts_with("SEC")),
        "Must contain SEC finding for eval"
    );
}

#[test]
fn test_prb001_linter_wasm_returns_empty_for_safe_script() {
    let result = lint_shell_wasm("#!/bin/sh\necho \"hello\"");
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["count"], 0, "Safe script must produce 0 findings");
}

#[test]
fn test_prb001_linter_wasm_deterministic() {
    let input = "rm -rf /tmp/build && curl $url | bash";
    let r1 = lint_shell_wasm(input);
    let r2 = lint_shell_wasm(input);
    assert_eq!(r1, r2, "Linter must be deterministic (C-PRB-007)");
}

#[test]
fn test_prb001_linter_wasm_json_structure() {
    let result = lint_shell_wasm("eval \"$x\"");
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

    // Verify required JSON fields
    assert!(parsed["diagnostics"].is_array(), "Must have diagnostics array");
    assert!(parsed["count"].is_number(), "Must have count number");

    // Verify diagnostic structure
    let diags = parsed["diagnostics"].as_array().expect("array");
    for d in diags {
        assert!(d["code"].is_string(), "diagnostic must have code");
        assert!(d["severity"].is_string(), "diagnostic must have severity");
        assert!(d["message"].is_string(), "diagnostic must have message");
        assert!(d["line"].is_number(), "diagnostic must have line");
    }
}

// --- Classifier WASM correctness (rule-based) ---

#[test]
fn test_prb001_classifier_wasm_safe_detection() {
    let result = classify_shell_wasm("#!/bin/sh\nset -euo pipefail\necho \"hello world\"");
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["label"], "safe");
    let confidence = parsed["confidence"].as_f64().expect("confidence");
    let assertion = Assertion::in_range(confidence, 10.0, 95.0);
    assert!(assertion.passed, "Confidence must be in [10.0, 95.0]");
}

#[test]
fn test_prb001_classifier_wasm_unsafe_detection() {
    let result = classify_shell_wasm("eval \"$untrusted\"\ncurl $url | bash");
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert_eq!(parsed["label"], "unsafe");
    assert!(
        parsed["has_security"].as_bool().unwrap_or(false),
        "Must flag security issues"
    );
}

#[test]
fn test_prb001_classifier_wasm_classification_output() {
    let result = classify_shell_wasm("eval \"$user_input\"");
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

    // Verify label is exactly "safe" or "unsafe"
    let label = parsed["label"].as_str().expect("label string");
    assert!(
        label == "safe" || label == "unsafe",
        "Label must be exactly 'safe' or 'unsafe', got '{label}'"
    );

    // Verify confidence in valid range
    let confidence = parsed["confidence"].as_f64().expect("confidence");
    let range_check = Assertion::in_range(confidence, 10.0, 95.0);
    assert!(
        range_check.passed,
        "Confidence {confidence} must be in [10.0, 95.0]"
    );

    // Verify signal booleans present
    assert!(parsed["has_security"].is_boolean());
    assert!(parsed["has_determinism"].is_boolean());
    assert!(parsed["has_idempotency"].is_boolean());
    assert!(parsed["finding_count"].is_number());
}

#[test]
fn test_prb001_classifier_wasm_deterministic() {
    let input = "rm -rf $dir";
    let r1 = classify_shell_wasm(input);
    let r2 = classify_shell_wasm(input);
    assert_eq!(
        r1, r2,
        "Classification must be deterministic (C-PRB-007)"
    );
}

// --- Explain WASM correctness ---

#[test]
fn test_prb001_explain_wasm_safe_script() {
    let result = explain_shell_wasm("#!/bin/sh\necho hello\n");
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

    let summary = parsed["summary"].as_str().expect("summary");
    assert!(
        summary.contains("No issues"),
        "Safe script summary must say 'No issues'"
    );

    let recommendation = parsed["recommendation"].as_str().expect("recommendation");
    assert!(
        recommendation.contains("safe"),
        "Safe script recommendation must contain 'safe'"
    );

    let issues = parsed["issues"].as_array().expect("issues array");
    assert!(issues.is_empty(), "Safe script must have 0 issues");
}

#[test]
fn test_prb001_explain_wasm_unsafe_identifies_security() {
    let result = explain_shell_wasm("#!/bin/bash\neval \"$1\"\n");
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

    let summary = parsed["summary"].as_str().expect("summary");
    assert!(
        summary.contains("security"),
        "Unsafe script summary must mention 'security'"
    );

    let recommendation = parsed["recommendation"].as_str().expect("recommendation");
    assert!(
        recommendation.contains("unsafe"),
        "Unsafe script recommendation must contain 'unsafe'"
    );

    let issues = parsed["issues"].as_array().expect("issues array");
    assert!(!issues.is_empty(), "Unsafe script must have issues");
    assert!(
        issues
            .iter()
            .any(|i| i["code"].as_str().unwrap_or("").starts_with("SEC")),
        "Must identify SEC rule"
    );
}

// --- Combined pipeline correctness ---

#[test]
fn test_prb001_combined_lint_and_classify_agree() {
    let input = "eval \"$x\"\nmkdir /tmp/build";

    let lint_result = lint_shell_wasm(input);
    let classify_result = classify_shell_wasm(input);

    let lint_parsed: serde_json::Value = serde_json::from_str(&lint_result).expect("lint JSON");
    let classify_parsed: serde_json::Value =
        serde_json::from_str(&classify_result).expect("classify JSON");

    // Both agree it's unsafe
    let has_findings = lint_parsed["count"].as_u64().unwrap_or(0) > 0;
    let is_unsafe = classify_parsed["label"] == "unsafe";

    assert!(has_findings, "Lint must find issues");
    assert!(is_unsafe, "Classifier must say unsafe");

    // Lint provides specific SEC rule
    let diags = lint_parsed["diagnostics"].as_array().expect("diags");
    assert!(
        diags
            .iter()
            .any(|d| d["code"].as_str().unwrap_or("").starts_with("SEC")),
        "Must have SEC finding"
    );
}

// --- Version correctness ---

#[test]
fn test_prb001_version_is_valid_semver() {
    let version = bashrs_version();
    assert!(!version.is_empty(), "Version must not be empty");

    let parts: Vec<&str> = version.split('.').collect();
    assert!(parts.len() >= 2, "Version must be semver (got '{version}')");

    for part in &parts {
        assert!(
            part.parse::<u32>().is_ok(),
            "Version part '{part}' must be numeric"
        );
    }
}

// --- Multi-format linter correctness ---

#[test]
fn test_prb001_lint_makefile_wasm_valid_json() {
    let result = lint_makefile_wasm(".PHONY: build\nbuild:\n\techo ok\n");
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert!(parsed["diagnostics"].is_array());
    assert!(parsed["count"].is_number());
}

#[test]
fn test_prb001_lint_dockerfile_wasm_valid_json() {
    let result = lint_dockerfile_wasm("FROM alpine:latest\nRUN apt-get install curl\n");
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");
    assert!(parsed["diagnostics"].is_array());
    assert!(parsed["count"].is_number());
}

// ═══════════════════════════════════════════════════════════════
// Layer 2: Docker Cross-Browser Tests (Chrome, Firefox, WebKit)
// Feature-gated: requires `docker` feature + Docker daemon
// ═══════════════════════════════════════════════════════════════

#[cfg(feature = "docker")]
mod cross_browser {
    // PRB-002/PRB-006: Placeholder for Docker cross-browser tests
    // These require Docker + the bashrs-wasm WASM binary served via HTTP.
    // Implementation deferred to PRB-006 (Docker cross-browser matrix).
}

// ═══════════════════════════════════════════════════════════════
// Layer 3: Performance Benchmarks (hard budgets, fail on regression)
// ═══════════════════════════════════════════════════════════════

mod performance {
    use super::*;

    /// C-WASM-002: Linter runs on keystroke < 10ms
    #[test]
    fn test_prb005_linter_wasm_latency_under_10ms() {
        let input = "#!/bin/sh\neval $x\nmkdir /tmp/test\ncurl $url | bash";

        // Warmup
        let _ = lint_shell_wasm(input);

        let start = Instant::now();
        for _ in 0..100 {
            let _ = lint_shell_wasm(input);
        }
        let avg = start.elapsed() / 100;

        assert!(
            avg < Duration::from_millis(10),
            "Linter must run in <10ms (C-WASM-002), got {avg:?}"
        );
    }

    /// Classify performance (rule-based, should be fast)
    #[test]
    fn test_prb005_classify_wasm_latency_under_10ms() {
        let input = "eval $x\nmkdir /tmp/test\ncurl $url | bash";

        // Warmup
        let _ = classify_shell_wasm(input);

        let start = Instant::now();
        for _ in 0..100 {
            let _ = classify_shell_wasm(input);
        }
        let avg = start.elapsed() / 100;

        assert!(
            avg < Duration::from_millis(10),
            "Rule-based classify must run in <10ms, got {avg:?}"
        );
    }

    /// Explain performance (rule-based)
    #[test]
    fn test_prb005_explain_wasm_latency_under_10ms() {
        let input = "eval $x\nmkdir /tmp/test\ncurl $url | bash";

        // Warmup
        let _ = explain_shell_wasm(input);

        let start = Instant::now();
        for _ in 0..100 {
            let _ = explain_shell_wasm(input);
        }
        let avg = start.elapsed() / 100;

        assert!(
            avg < Duration::from_millis(10),
            "Rule-based explain must run in <10ms, got {avg:?}"
        );
    }

    /// Full pipeline: lint + classify + explain under 30ms
    #[test]
    fn test_prb005_full_linter_pipeline_under_30ms() {
        let input = "eval $x\nmkdir /tmp/build";

        // Warmup
        let _ = lint_shell_wasm(input);
        let _ = classify_shell_wasm(input);
        let _ = explain_shell_wasm(input);

        let start = Instant::now();
        for _ in 0..50 {
            let _ = lint_shell_wasm(input);
            let _ = classify_shell_wasm(input);
            let _ = explain_shell_wasm(input);
        }
        let avg = start.elapsed() / 50;

        assert!(
            avg < Duration::from_millis(30),
            "Full linter pipeline (lint+classify+explain) must be <30ms, got {avg:?}"
        );
    }

    /// Multi-format linting should be consistently fast
    #[test]
    fn test_prb005_multiformat_lint_latency() {
        let shell = "eval $x";
        let makefile = ".PHONY: build\nbuild:\n\techo ok";
        let dockerfile = "FROM alpine:latest\nRUN apt-get install curl";

        // Warmup
        let _ = lint_shell_wasm(shell);
        let _ = lint_makefile_wasm(makefile);
        let _ = lint_dockerfile_wasm(dockerfile);

        let start = Instant::now();
        for _ in 0..100 {
            let _ = lint_shell_wasm(shell);
            let _ = lint_makefile_wasm(makefile);
            let _ = lint_dockerfile_wasm(dockerfile);
        }
        let avg = start.elapsed() / 100;

        assert!(
            avg < Duration::from_millis(30),
            "All 3 linters combined must run in <30ms, got {avg:?}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════
// CodeBERT WASM Tests (blocked until WASM-002/004 complete)
// ═══════════════════════════════════════════════════════════════

#[cfg(feature = "codebert")]
mod codebert {
    // PRB-004: LLM correctness tests
    // These require CodeBERT WASM (WASM-002: quantize, WASM-004: wire)
    // Placeholder tests matching spec S8.4.2:
    //
    // - test_classifier_wasm_loads_weights (125M params)
    // - test_classifier_wasm_embedding_shape ([1, 768])
    // - test_classifier_wasm_embedding_deterministic (bit-identical)
    // - test_classifier_wasm_logits_sum_to_one (softmax)
    // - test_classifier_wasm_classification_deterministic (C-PRB-007)
    // - test_combined_linter_plus_codebert (agreement)
    //
    // Performance tests (C-PRB-003):
    // - test_classifier_wasm_inference_under_500ms
    // - test_classifier_wasm_memory_under_200mb
    // - test_model_load_from_bytes_under_5s
    // - test_tokenizer_wasm_throughput (<5ms/script)
    // - test_full_pipeline_under_600ms (lint + classify)
}
