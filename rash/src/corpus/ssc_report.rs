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
    let sections = vec![
        corpus_section(),
        tokenizer_section(),
        label_section(),
        baselines_section(),
        generalization_section(),
        dataset_section(),
        conversation_section(),
    ];

    let overall_ready = sections.iter().all(|s| s.status != SscStatus::Fail);

    SscStatusReport {
        spec_version: "v11.0.0".to_string(),
        corpus_size: crate::corpus::registry::CorpusRegistry::load_full()
            .entries
            .len(),
        sections,
        overall_ready,
    }
}

fn corpus_section() -> SscSection {
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
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

fn baselines_section() -> SscSection {
    use crate::corpus::baselines::{corpus_baseline_entries, run_all_baselines};

    let owned = corpus_baseline_entries();
    let entries: Vec<(&str, u8)> = owned.iter().map(|(s, l)| (s.as_str(), *l)).collect();
    let reports = run_all_baselines(&entries);

    let metrics: Vec<SscMetric> = reports
        .iter()
        .map(|r| SscMetric {
            name: r.name.clone(),
            value: format!("MCC={:.3}", r.mcc),
            target: "reference".to_string(),
            passed: true,
        })
        .collect();

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
            r.diagnostics
                .iter()
                .any(|d| d.code.starts_with("SEC") || d.code.starts_with("DET"))
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

fn dataset_section() -> SscSection {
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

fn conversation_section() -> SscSection {
    use crate::corpus::conversations::generate_batch;
    use crate::corpus::registry::CorpusRegistry;

    // Sample 100 entries to check conversation generation
    let registry = CorpusRegistry::load_full();
    let sample: Vec<(&str, &str)> = registry
        .entries
        .iter()
        .take(100)
        .map(|e| (e.id.as_str(), e.input.as_str()))
        .collect();

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
                name: "Rule citation accuracy".to_string(),
                value: format!("{:.0}%", report.rule_citation_accuracy * 100.0),
                target: "100%".to_string(),
                passed: (report.rule_citation_accuracy - 1.0).abs() < 0.001,
            },
            SscMetric {
                name: "Quality gate".to_string(),
                value: if report.passed {
                    "passed".to_string()
                } else {
                    "failed".to_string()
                },
                target: "passed".to_string(),
                passed: report.passed,
            },
        ],
    }
}

/// Format the report as a human-readable string.
pub fn format_ssc_report(report: &SscStatusReport) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "SSC Spec Version: {}", report.spec_version);
    let _ = writeln!(out, "Corpus Size:      {}", report.corpus_size);
    let _ = writeln!(
        out,
        "Overall Ready:    {}\n",
        if report.overall_ready { "YES" } else { "NO" }
    );

    for section in &report.sections {
        let status = match section.status {
            SscStatus::Pass => "PASS",
            SscStatus::Warn => "WARN",
            SscStatus::Fail => "FAIL",
            SscStatus::NotApplicable => "N/A",
        };
        let _ = writeln!(out, "[{status}] {} ({})", section.name, section.spec_ref);
        for m in &section.metrics {
            let check = if m.passed { "+" } else { "-" };
            let _ = writeln!(
                out,
                "  [{check}] {:<30} {:<25} target: {}",
                m.name, m.value, m.target
            );
        }
        let _ = writeln!(out);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssc_status_enum_values() {
        assert_ne!(SscStatus::Pass, SscStatus::Fail);
        assert_ne!(SscStatus::Warn, SscStatus::NotApplicable);
    }

    #[test]
    fn test_format_ssc_report_structure() {
        let report = SscStatusReport {
            spec_version: "v11.0.0".to_string(),
            corpus_size: 100,
            sections: vec![SscSection {
                name: "Test".to_string(),
                spec_ref: "S1".to_string(),
                status: SscStatus::Pass,
                metrics: vec![SscMetric {
                    name: "metric".to_string(),
                    value: "42".to_string(),
                    target: ">0".to_string(),
                    passed: true,
                }],
            }],
            overall_ready: true,
        };
        let formatted = format_ssc_report(&report);
        assert!(formatted.contains("SSC Spec Version: v11.0.0"));
        assert!(formatted.contains("[PASS] Test"));
        assert!(formatted.contains("Overall Ready:    YES"));
    }

    #[test]
    fn test_report_serializable() {
        let report = SscStatusReport {
            spec_version: "v11.0.0".to_string(),
            corpus_size: 0,
            sections: vec![],
            overall_ready: true,
        };
        let json = serde_json::to_string(&report);
        assert!(json.is_ok());
    }
}
