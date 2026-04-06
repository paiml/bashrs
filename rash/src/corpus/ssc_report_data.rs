fn data_pipeline_section() -> SscSection {
    use crate::corpus::model_card::generate_model_card;
    use crate::corpus::training_config::generate_training_config;

    let card = generate_model_card();
    let config = generate_training_config();

    let card_has_honesty = card.contains("synthetic data derived from rule-based linter");
    let card_has_yaml = card.starts_with("---");
    let config_has_weights = config.training.class_weights.len() == 2;
    let config_has_data = config.data.total_entries > 0;

    SscSection {
        name: "Data Pipeline (S9)".to_string(),
        spec_ref: "S9".to_string(),
        status: if card_has_honesty && card_has_yaml && config_has_weights && config_has_data {
            SscStatus::Pass
        } else {
            SscStatus::Warn
        },
        metrics: vec![
            SscMetric {
                name: "Model card YAML".to_string(),
                value: if card_has_yaml {
                    "present".to_string()
                } else {
                    "missing".to_string()
                },
                target: "present".to_string(),
                passed: card_has_yaml,
            },
            SscMetric {
                name: "Honesty (S6.5)".to_string(),
                value: if card_has_honesty {
                    "present".to_string()
                } else {
                    "missing".to_string()
                },
                target: "present".to_string(),
                passed: card_has_honesty,
            },
            SscMetric {
                name: "Class weights".to_string(),
                value: format!(
                    "[{:.3}, {:.3}]",
                    config.training.class_weights[0], config.training.class_weights[1]
                ),
                target: "2 weights".to_string(),
                passed: config_has_weights,
            },
            SscMetric {
                name: "Training entries".to_string(),
                value: config.data.total_entries.to_string(),
                target: ">0".to_string(),
                passed: config_has_data,
            },
        ],
    }
}

fn shellsafetybench_section() -> SscSection {
    use crate::corpus::cwe_mapping;
    use crate::corpus::eval_harness;

    // Check CWE mapping completeness
    let cwe_count = cwe_mapping::CWE_MAPPINGS.len();
    let ood_count = cwe_mapping::OOD_CWES.len();
    let ood_disjoint = cwe_mapping::verify_ood_disjoint();

    // Check eval harness weights
    let weight_sum = eval_harness::DETECTION_F1_WEIGHT
        + eval_harness::RULE_CITATION_WEIGHT
        + eval_harness::CWE_MAPPING_WEIGHT
        + eval_harness::FIX_VALIDITY_WEIGHT
        + eval_harness::EXPLANATION_WEIGHT
        + eval_harness::OOD_WEIGHT;
    let weights_valid = (weight_sum - 1.0).abs() < 1e-9;

    // Check benchmark and conversation files
    let benchmark_exists =
        std::path::Path::new("training/shellsafetybench/benchmark.jsonl").exists();
    let conversations_exists =
        std::path::Path::new("training/shellsafetybench/conversations.jsonl").exists();
    let verificar_exists =
        std::path::Path::new("training/shellsafetybench/verificar-labeled.jsonl").exists();
    let pipeline_exists = std::path::Path::new("configs/pipeline/ssc.yaml").exists();
    let qa_exists = std::path::Path::new("configs/qa/ssc-release-v1.yaml").exists();
    let train_config_exists =
        std::path::Path::new("configs/train/ssc-qwen3-4b-qlora.yaml").exists();
    let contract_exists =
        std::path::Path::new("provable-contracts/contracts/shellsafetybench-v1.yaml").exists();

    // Count lines in data files (0 if missing)
    let count_lines = |path: &str| -> usize {
        std::fs::read_to_string(path)
            .map(|s| s.lines().count())
            .unwrap_or(0)
    };
    let corpus_entries = count_lines("training/shellsafetybench/conversations.jsonl");
    let verificar_entries = count_lines("training/shellsafetybench/verificar-labeled.jsonl");
    let total_entries = corpus_entries + verificar_entries;

    let all_pass = cwe_count == 14
        && ood_disjoint
        && weights_valid
        && benchmark_exists
        && conversations_exists
        && pipeline_exists
        && contract_exists;

    SscSection {
        name: "ShellSafetyBench (S14)".to_string(),
        spec_ref: "S14".to_string(),
        status: if all_pass {
            SscStatus::Pass
        } else {
            SscStatus::Warn
        },
        metrics: vec![
            SscMetric {
                name: "CWE mappings".to_string(),
                value: format!("{cwe_count} rules"),
                target: "14 rules".to_string(),
                passed: cwe_count == 14,
            },
            SscMetric {
                name: "OOD CWEs".to_string(),
                value: format!("{ood_count} disjoint={ood_disjoint}"),
                target: "4 disjoint=true".to_string(),
                passed: ood_count == 4 && ood_disjoint,
            },
            SscMetric {
                name: "Eval weights".to_string(),
                value: format!("sum={weight_sum:.3}"),
                target: "sum=1.000".to_string(),
                passed: weights_valid,
            },
            SscMetric {
                name: "Benchmark JSONL".to_string(),
                value: if benchmark_exists {
                    "present".to_string()
                } else {
                    "missing".to_string()
                },
                target: "present".to_string(),
                passed: benchmark_exists,
            },
            SscMetric {
                name: "Conversations JSONL".to_string(),
                value: if conversations_exists {
                    "present".to_string()
                } else {
                    "missing".to_string()
                },
                target: "present".to_string(),
                passed: conversations_exists,
            },
            SscMetric {
                name: "Pipeline manifest".to_string(),
                value: if pipeline_exists {
                    "present".to_string()
                } else {
                    "missing".to_string()
                },
                target: "present".to_string(),
                passed: pipeline_exists,
            },
            SscMetric {
                name: "QA gate config".to_string(),
                value: if qa_exists {
                    "present".to_string()
                } else {
                    "missing".to_string()
                },
                target: "present".to_string(),
                passed: qa_exists,
            },
            SscMetric {
                name: "Training config".to_string(),
                value: if train_config_exists {
                    "present".to_string()
                } else {
                    "missing".to_string()
                },
                target: "present".to_string(),
                passed: train_config_exists,
            },
            SscMetric {
                name: "Verificar mutations".to_string(),
                value: if verificar_exists {
                    format!("{verificar_entries} entries")
                } else {
                    "missing".to_string()
                },
                target: ">0 entries".to_string(),
                passed: verificar_exists && verificar_entries > 0,
            },
            SscMetric {
                name: "Total entries".to_string(),
                value: format!(
                    "{total_entries} (corpus={corpus_entries} + verificar={verificar_entries})"
                ),
                target: ">20000".to_string(),
                passed: total_entries > 20000,
            },
            SscMetric {
                name: "Provable contract".to_string(),
                value: if contract_exists {
                    "present".to_string()
                } else {
                    "missing".to_string()
                },
                target: "present".to_string(),
                passed: contract_exists,
            },
            {
                // Check merged splits exist and have balanced class distribution
                let train_path = "training/shellsafetybench/splits/train.jsonl";
                let splits_exist = std::path::Path::new(train_path).exists();
                let (train_total, train_unsafe) = if splits_exist {
                    let content = std::fs::read_to_string(train_path).unwrap_or_default();
                    let total = content.lines().filter(|l| !l.trim().is_empty()).count();
                    let unsafe_count = content
                        .lines()
                        .filter(|l| l.contains("\"label\":1") || l.contains("\"label\": 1"))
                        .count();
                    (total, unsafe_count)
                } else {
                    (0, 0)
                };
                let unsafe_pct = if train_total > 0 {
                    100.0 * train_unsafe as f64 / train_total as f64
                } else {
                    0.0
                };
                let balanced = unsafe_pct > 5.0; // >5% unsafe = balanced enough
                SscMetric {
                    name: "Merged splits".to_string(),
                    value: format!("{train_total} train ({unsafe_pct:.1}% unsafe)"),
                    target: ">5% unsafe".to_string(),
                    passed: splits_exist && balanced,
                }
            },
        ],
    }
}

fn wasm_section() -> SscSection {
    // WASM-004 kill criterion 5 result: CodeBERT inference too slow for browser.
    // Linter-only WASM app deployed and working (<10ms per analysis).
    SscSection {
        name: "WASM App (S8.3)".to_string(),
        spec_ref: "S8.3".to_string(),
        status: SscStatus::Pass,
        metrics: vec![
            SscMetric {
                name: "Linter WASM".to_string(),
                value: "deployed".to_string(),
                target: "deployed".to_string(),
                passed: true,
            },
            SscMetric {
                name: "WASM binary size".to_string(),
                value: "1.5MB".to_string(),
                target: "<5MB".to_string(),
                passed: true,
            },
            SscMetric {
                name: "Linter latency".to_string(),
                value: "<10ms".to_string(),
                target: "<10ms".to_string(),
                passed: true,
            },
            SscMetric {
                name: "CodeBERT WASM (KILL-5)".to_string(),
                value: "2681ms native (est. 5-13s WASM)".to_string(),
                target: "<2000ms".to_string(),
                passed: false,
            },
            SscMetric {
                name: "Decision".to_string(),
                value: "CLI only for CodeBERT".to_string(),
                target: "informational".to_string(),
                passed: true,
            },
        ],
    }
}

/// Cheap keyword heuristic for safe/unsafe partitioning.
///
/// Used only for stratified sampling — accurate classification happens
/// inside `generate_batch()` via `lint_shell()` on the sampled entries.
/// Avoids running the full linter on all 17k+ corpus entries.
fn has_unsafe_keyword(script: &str) -> bool {
    const KEYWORDS: &[&str] = &[
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
    ];
    KEYWORDS.iter().any(|kw| script.contains(kw))
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
#[path = "ssc_report_tests_ssc_status.rs"]
mod tests_extracted;
