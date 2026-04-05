//! SSC v11 comprehensive status report.
//!
//! Single-command view of all Shell Safety Classifier readiness metrics.
//! Covers: contracts, baselines, generalization, label audit, dataset splits,
//! tokenizer validation, and conversation generation capacity.

use serde::Serialize;

/// Comprehensive SSC readiness report.
#[derive(Debug, Clone, Serialize)]
pub struct SscStatusReport {
    pub spec_version: String,
    pub corpus_size: usize,
    pub sections: Vec<SscSection>,
    pub overall_ready: bool,
}

/// A section of the SSC report.
#[derive(Debug, Clone, Serialize)]
pub struct SscSection {
    pub name: String,
    pub spec_ref: String,
    pub status: SscStatus,
    pub metrics: Vec<SscMetric>,
}

/// Status of a report section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SscStatus {
    Pass,
    Warn,
    Fail,
    NotApplicable,
}

/// A single metric in a report section.
#[derive(Debug, Clone, Serialize)]
pub struct SscMetric {
    pub name: String,
    pub value: String,
    pub target: String,
    pub passed: bool,
}

/// Generate the full SSC status report.
pub fn generate_ssc_report() -> SscStatusReport {
    use crate::corpus::baselines::corpus_baseline_entries_from;
    use crate::corpus::registry::CorpusRegistry;

    // Load corpus once, share across ALL sections (avoids double load).
    let registry = CorpusRegistry::load_full();
    let corpus_size = registry.entries.len();

    // Compute baseline entries once (lints entire corpus). Shared by baselines + dataset.
    // Uses _from variant to reuse the already-loaded registry.
    let baseline_entries = corpus_baseline_entries_from(&registry);

    let sections = vec![
        corpus_section_from(&registry),
        tokenizer_section(),
        label_section(),
        baselines_section_from(&baseline_entries),
        generalization_section(),
        dataset_section_from(baseline_entries),
        conversation_section_from(&registry),
        data_pipeline_section(),
        shellsafetybench_section(),
        wasm_section(),
    ];

    let overall_ready = sections.iter().all(|s| s.status != SscStatus::Fail);

    SscStatusReport {
        spec_version: "v11.0.0".to_string(),
        corpus_size,
        sections,
        overall_ready,
    }
}

fn corpus_section_from(registry: &crate::corpus::registry::CorpusRegistry) -> SscSection {
    let total = registry.entries.len();
    let bash = registry
        .entries
        .iter()
        .filter(|e| e.format == crate::corpus::CorpusFormat::Bash)
        .count();
    let makefile = registry
        .entries
        .iter()
        .filter(|e| e.format == crate::corpus::CorpusFormat::Makefile)
        .count();
    let dockerfile = registry
        .entries
        .iter()
        .filter(|e| e.format == crate::corpus::CorpusFormat::Dockerfile)
        .count();

    SscSection {
        name: "Corpus".to_string(),
        spec_ref: "S5.3".to_string(),
        status: if total >= 17000 {
            SscStatus::Pass
        } else {
            SscStatus::Warn
        },
        metrics: vec![
            SscMetric {
                name: "Total entries".to_string(),
                value: total.to_string(),
                target: ">=17,000".to_string(),
                passed: total >= 17000,
            },
            SscMetric {
                name: "Bash".to_string(),
                value: bash.to_string(),
                target: "majority".to_string(),
                passed: bash > makefile && bash > dockerfile,
            },
            SscMetric {
                name: "Makefile".to_string(),
                value: makefile.to_string(),
                target: ">0".to_string(),
                passed: makefile > 0,
            },
            SscMetric {
                name: "Dockerfile".to_string(),
                value: dockerfile.to_string(),
                target: ">0".to_string(),
                passed: dockerfile > 0,
            },
        ],
    }
}

fn tokenizer_section() -> SscSection {
    use crate::corpus::tokenizer_validation::run_validation;

    let report = run_validation(|construct| {
        construct
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    });

    SscSection {
        name: "Tokenizer (C-TOK-001)".to_string(),
        spec_ref: "S5.2".to_string(),
        status: if report.passed {
            SscStatus::Pass
        } else {
            SscStatus::Fail
        },
        metrics: vec![
            SscMetric {
                name: "Acceptable".to_string(),
                value: format!("{:.1}%", report.acceptable_pct),
                target: ">=70%".to_string(),
                passed: report.passed,
            },
            SscMetric {
                name: "Constructs tested".to_string(),
                value: report.total_constructs.to_string(),
                target: "20".to_string(),
                passed: report.total_constructs == 20,
            },
        ],
    }
}

fn label_section() -> SscSection {
    use crate::corpus::label_audit::run_corpus_label_audit;

    let report = run_corpus_label_audit(100);

    SscSection {
        name: "Label Audit (C-LABEL-001)".to_string(),
        spec_ref: "S5.3".to_string(),
        status: if report.passed {
            SscStatus::Pass
        } else {
            SscStatus::Fail
        },
        metrics: vec![
            SscMetric {
                name: "Accuracy".to_string(),
                value: format!("{:.1}%", report.accuracy_pct),
                target: ">=90%".to_string(),
                passed: report.passed,
            },
            SscMetric {
                name: "Audited".to_string(),
                value: report.total_audited.to_string(),
                target: ">=100".to_string(),
                passed: report.total_audited >= 100,
            },
            SscMetric {
                name: "False positives".to_string(),
                value: report.false_positives.to_string(),
                target: "<=10%".to_string(),
                passed: report.false_positives == 0
                    || (report.false_positives as f64 / report.total_audited as f64) <= 0.1,
            },
        ],
    }
}

fn baselines_section_from(owned: &[(String, u8)]) -> SscSection {
    use crate::corpus::baselines::run_all_baselines;

    let entries: Vec<(&str, u8)> = owned.iter().map(|(s, l)| (s.as_str(), *l)).collect();
    let reports = run_all_baselines(&entries);

    let mut metrics: Vec<SscMetric> = Vec::new();
    for r in &reports {
        metrics.push(SscMetric {
            name: r.name.clone(),
            value: format!(
                "MCC={:.3} acc={:.1}% rec={:.1}%",
                r.mcc,
                r.accuracy * 100.0,
                r.recall * 100.0
            ),
            target: "reference".to_string(),
            passed: true,
        });
    }
    // S5.5: Any ML classifier must beat these targets
    metrics.push(SscMetric {
        name: "Target: MCC CI lower".to_string(),
        value: ">0.2".to_string(),
        target: "for ML classifier".to_string(),
        passed: true,
    });
    metrics.push(SscMetric {
        name: "Target: accuracy".to_string(),
        value: ">93.5% (beat majority)".to_string(),
        target: "for ML classifier".to_string(),
        passed: true,
    });

    SscSection {
        name: "Baselines (C-CLF-001)".to_string(),
        spec_ref: "S5.5".to_string(),
        status: SscStatus::Pass,
        metrics,
    }
}

fn generalization_section() -> SscSection {
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
            // Any diagnostic indicates the linter detected an issue
            !r.diagnostics.is_empty()
        })
        .count();
    let pct = caught as f64 / total as f64 * 100.0;
    let passed = pct >= GENERALIZATION_TARGET_PCT;

    SscSection {
        name: "Generalization (OOD)".to_string(),
        spec_ref: "S5.6".to_string(),
        status: if passed {
            SscStatus::Pass
        } else {
            SscStatus::Fail
        },
        metrics: vec![SscMetric {
            name: "Caught".to_string(),
            value: format!("{caught}/{total} ({pct:.1}%)"),
            target: format!(">={GENERALIZATION_TARGET_PCT}%"),
            passed,
        }],
    }
}

fn dataset_section_from(owned: Vec<(String, u8)>) -> SscSection {
    use crate::corpus::dataset::{split_and_validate, ClassificationRow};

    let rows: Vec<ClassificationRow> = owned
        .into_iter()
        .map(|(input, label)| ClassificationRow { input, label })
        .collect();
    let total = rows.len();
    let result = split_and_validate(rows, 2);

    let train_pct = result.train.len() as f64 / total as f64 * 100.0;
    let val_pct = result.val.len() as f64 / total as f64 * 100.0;
    let test_pct = result.test.len() as f64 / total as f64 * 100.0;

    SscSection {
        name: "Dataset Splits".to_string(),
        spec_ref: "S5.3".to_string(),
        status: SscStatus::Pass,
        metrics: vec![
            SscMetric {
                name: "Train".to_string(),
                value: format!("{} ({:.1}%)", result.train.len(), train_pct),
                target: "~80%".to_string(),
                passed: (70.0..=90.0).contains(&train_pct),
            },
            SscMetric {
                name: "Val".to_string(),
                value: format!("{} ({:.1}%)", result.val.len(), val_pct),
                target: "~10%".to_string(),
                passed: (5.0..=20.0).contains(&val_pct),
            },
            SscMetric {
                name: "Test".to_string(),
                value: format!("{} ({:.1}%)", result.test.len(), test_pct),
                target: "~10%".to_string(),
                passed: (5.0..=20.0).contains(&test_pct),
            },
        ],
    }
}

fn conversation_section_from(registry: &crate::corpus::registry::CorpusRegistry) -> SscSection {
    use crate::corpus::conversations::generate_batch;

    // Stratified sample: 70 safe + 30 unsafe entries to exercise all conversation types.
    // Use cheap keyword heuristic for partitioning (not full lint_shell on 17k entries).
    // generate_batch() does accurate linting internally on the 100 sampled entries.

    let (safe_entries, unsafe_entries): (Vec<_>, Vec<_>) = registry
        .entries
        .iter()
        .partition(|e| !has_unsafe_keyword(&e.input));

    // Stratified sample: up to 30 unsafe + up to 70 safe
    let unsafe_stride = unsafe_entries.len().max(1) / 30;
    let safe_stride = safe_entries.len().max(1) / 70;
    let unsafe_sample: Vec<_> = unsafe_entries
        .iter()
        .step_by(unsafe_stride.max(1))
        .take(30)
        .map(|e| (e.id.as_str(), e.input.as_str()))
        .collect();
    let safe_sample: Vec<_> = safe_entries
        .iter()
        .step_by(safe_stride.max(1))
        .take(70)
        .map(|e| (e.id.as_str(), e.input.as_str()))
        .collect();

    let mut sample = unsafe_sample;
    sample.extend(safe_sample);

    let (conversations, report) = generate_batch(&sample, 42);

    SscSection {
        name: "Conversations (S6)".to_string(),
        spec_ref: "S6".to_string(),
        status: if report.passed {
            SscStatus::Pass
        } else {
            SscStatus::Warn
        },
        metrics: vec![
            SscMetric {
                name: "Generated".to_string(),
                value: conversations.len().to_string(),
                target: ">0".to_string(),
                passed: !conversations.is_empty(),
            },
            SscMetric {
                name: "Type A (classify+explain)".to_string(),
                value: report.type_a_count.to_string(),
                target: "informational".to_string(),
                passed: true, // Informational — depends on corpus composition
            },
            SscMetric {
                name: "Type B (fix)".to_string(),
                value: report.type_b_count.to_string(),
                target: "informational".to_string(),
                passed: true, // Informational — depends on corpus composition
            },
            SscMetric {
                name: "Type C (debug)".to_string(),
                value: report.type_c_count.to_string(),
                target: "informational".to_string(),
                passed: true, // Informational — depends on corpus composition
            },
            SscMetric {
                name: "Type D (confirm safe)".to_string(),
                value: format!("{} ({:.1}%)", report.type_d_count, report.type_d_pct),
                target: ">=30%".to_string(),
                passed: report.type_d_pct >= 30.0,
            },
            SscMetric {
                name: "Rule citation accuracy".to_string(),
                value: format!("{:.0}%", report.rule_citation_accuracy * 100.0),
                target: "100%".to_string(),
                passed: (report.rule_citation_accuracy - 1.0).abs() < 0.001,
            },
            SscMetric {
                name: "Variant distribution".to_string(),
                value: if report.variant_distribution_ok {
                    "balanced".to_string()
                } else {
                    "skewed".to_string()
                },
                target: "no variant >20%".to_string(),
                passed: report.variant_distribution_ok,
            },
            SscMetric {
                name: "Empty/trivial responses".to_string(),
                value: report.empty_responses.to_string(),
                target: "0".to_string(),
                passed: report.empty_responses == 0,
            },
        ],
    }
}

include!("ssc_report_data.rs");
