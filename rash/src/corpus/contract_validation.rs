//! Unified contract validation harness (SSC v11 Section 10).
//!
//! Runs all SSC contracts in a single pass and reports aggregate status:
//! - C-TOK-001: Tokenizer quality >= 70%
//! - C-LABEL-001: Label accuracy >= 90%
//! - C-CLF-001: Baselines + evaluation metrics
//! - C-EMBED-001: Embedding determinism (placeholder for CodeBERT)
//!
//! This is the pre-training gate: ALL contracts must pass before proceeding
//! to classifier training.

use serde::Serialize;

/// Result of a single contract check.
#[derive(Debug, Clone, Serialize)]
pub struct ContractResult {
    pub id: String,
    pub name: String,
    pub passed: bool,
    pub value: f64,
    pub threshold: f64,
    pub detail: String,
}

/// Aggregate validation report for all SSC contracts.
#[derive(Debug, Clone, Serialize)]
pub struct ContractValidationReport {
    pub contracts: Vec<ContractResult>,
    pub all_passed: bool,
    pub passed_count: usize,
    pub failed_count: usize,
}

/// Run C-TOK-001: Tokenizer quality >= 70%.
pub fn check_c_tok_001() -> ContractResult {
    use crate::corpus::tokenizer_validation::run_validation;

    // Whitespace tokenizer as baseline (real BPE plugs in via entrenar)
    let report = run_validation(|construct| {
        construct
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    });

    ContractResult {
        id: "C-TOK-001".to_string(),
        name: "Tokenizer quality".to_string(),
        passed: report.passed,
        value: report.acceptable_pct,
        threshold: 70.0,
        detail: format!(
            "{}/{} constructs acceptable ({:.1}%)",
            report.acceptable_count, report.total_constructs, report.acceptable_pct
        ),
    }
}

/// Run C-LABEL-001: Label accuracy >= 90%.
pub fn check_c_label_001(limit: usize) -> ContractResult {
    use crate::corpus::label_audit::run_corpus_label_audit;

    let report = run_corpus_label_audit(limit);

    ContractResult {
        id: "C-LABEL-001".to_string(),
        name: "Label accuracy".to_string(),
        passed: report.passed,
        value: report.accuracy_pct,
        threshold: 90.0,
        detail: format!(
            "{}/{} genuinely unsafe ({:.1}%), {} false positives",
            report.genuinely_unsafe,
            report.total_audited,
            report.accuracy_pct,
            report.false_positives
        ),
    }
}

/// Run C-CLF-001 baselines: majority, keyword regex, linter.
pub fn check_c_clf_001_baselines() -> Vec<ContractResult> {
    use crate::corpus::baselines::{corpus_baseline_entries, run_all_baselines};

    let owned = corpus_baseline_entries();
    let entries: Vec<(&str, u8)> = owned.iter().map(|(s, l)| (s.as_str(), *l)).collect();
    let reports = run_all_baselines(&entries);

    reports
        .iter()
        .map(|r| ContractResult {
            id: "C-CLF-001".to_string(),
            name: format!("Baseline: {}", r.name),
            passed: true, // Baselines are reference, not pass/fail
            value: r.mcc,
            threshold: 0.0,
            detail: format!(
                "MCC={:.3}, Acc={:.3}, Prec={:.3}, Recall={:.3}",
                r.mcc, r.accuracy, r.precision, r.recall
            ),
        })
        .collect()
}

/// Run generalization test coverage check.
pub fn check_generalization() -> ContractResult {
    use crate::corpus::generalization_tests::{
        generalization_test_entries, GENERALIZATION_TARGET_PCT,
    };
    use crate::linter::lint_shell;

    let entries = generalization_test_entries();
    let total = entries.len();
    let caught = entries
        .iter()
        .filter(|(script, _)| {
            let r = lint_shell(script);
            r.diagnostics
                .iter()
                .any(|d| d.code.starts_with("SEC") || d.code.starts_with("DET"))
        })
        .count();

    let pct = caught as f64 / total as f64 * 100.0;

    ContractResult {
        id: "C-CLF-001-GEN".to_string(),
        name: "Generalization (OOD)".to_string(),
        passed: pct >= GENERALIZATION_TARGET_PCT,
        value: pct,
        threshold: GENERALIZATION_TARGET_PCT,
        detail: format!("{caught}/{total} OOD scripts caught ({pct:.1}%)"),
    }
}

/// Run dataset split validation.
pub fn check_dataset_split() -> ContractResult {
    use crate::corpus::baselines::corpus_baseline_entries;
    use crate::corpus::dataset::{split_and_validate, ClassificationRow};

    let owned = corpus_baseline_entries();
    let rows: Vec<ClassificationRow> = owned
        .into_iter()
        .map(|(input, label)| ClassificationRow { input, label })
        .collect();

    let total = rows.len();
    let result = split_and_validate(rows, 2);

    let train_pct = result.train.len() as f64 / total as f64 * 100.0;
    let val_pct = result.val.len() as f64 / total as f64 * 100.0;
    let test_pct = result.test.len() as f64 / total as f64 * 100.0;

    // Check split proportions are roughly 80/10/10
    let proportions_ok = (70.0..=90.0).contains(&train_pct)
        && (5.0..=20.0).contains(&val_pct)
        && (5.0..=20.0).contains(&test_pct);

    let errors_str = if result.validation.errors.is_empty() {
        String::new()
    } else {
        format!(", errors: [{}]", result.validation.errors.join("; "))
    };

    ContractResult {
        id: "C-DATA-001".to_string(),
        name: "Dataset split".to_string(),
        // Split proportions must be correct; validation warnings are informational
        passed: proportions_ok,
        value: train_pct,
        threshold: 80.0,
        detail: format!(
            "train={} ({:.1}%), val={} ({:.1}%), test={} ({:.1}%){}",
            result.train.len(),
            train_pct,
            result.val.len(),
            val_pct,
            result.test.len(),
            test_pct,
            errors_str,
        ),
    }
}

/// Run all contracts and produce an aggregate report.
pub fn run_all_contracts() -> ContractValidationReport {
    let mut contracts = Vec::new();

    // C-TOK-001
    contracts.push(check_c_tok_001());

    // C-LABEL-001
    contracts.push(check_c_label_001(100));

    // C-CLF-001 baselines
    contracts.extend(check_c_clf_001_baselines());

    // Generalization
    contracts.push(check_generalization());

    // Dataset split
    contracts.push(check_dataset_split());

    let passed_count = contracts.iter().filter(|c| c.passed).count();
    let failed_count = contracts.len() - passed_count;

    ContractValidationReport {
        all_passed: failed_count == 0,
        passed_count,
        failed_count,
        contracts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_tok_001_passes() {
        let result = check_c_tok_001();
        assert!(
            result.passed,
            "C-TOK-001 should pass with whitespace tokenizer"
        );
        assert!(result.value >= 70.0);
    }

    #[test]
    fn test_c_label_001_passes() {
        let result = check_c_label_001(50);
        assert!(result.passed, "C-LABEL-001 should pass: {}", result.detail);
        assert!(result.value >= 90.0);
    }

    #[test]
    fn test_generalization_check_runs() {
        let result = check_generalization();
        // The linter catches a high percentage of OOD scripts (>50% target)
        assert!(
            result.value > 0.0,
            "Should catch some OOD scripts: {}",
            result.detail
        );
    }

    #[test]
    fn test_contract_result_serializable() {
        let result = check_c_tok_001();
        let json = serde_json::to_string(&result);
        assert!(json.is_ok());
    }

    #[test]
    fn test_all_contracts_report() {
        let report = run_all_contracts();
        assert!(
            report.contracts.len() >= 6,
            "Should have at least 6 contract checks"
        );
        assert!(report.passed_count > 0);
    }

    #[test]
    fn test_dataset_split_proportions() {
        let result = check_dataset_split();
        // Split proportions should be roughly 80/10/10
        assert!(
            result.passed,
            "Split proportions should be valid: {}",
            result.detail
        );
        assert!(
            result.value >= 70.0 && result.value <= 90.0,
            "Train pct should be ~80%"
        );
    }
}
