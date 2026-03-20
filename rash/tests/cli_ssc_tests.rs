#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(deprecated)]
#![allow(non_snake_case)]
// SSC v11 CLI Integration Tests
// Tests classify, explain, fix, safety-check, and corpus SSC commands
// Uses assert_cmd (MANDATORY per CLAUDE.md)

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}

fn write_temp_script(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("create temp");
    f.write_all(content.as_bytes()).expect("write temp");
    f.flush().expect("flush temp");
    f
}

// ============================================================================
// bashrs classify
// ============================================================================

#[test]
fn test_PMAT142_classify_safe_script() {
    let f = write_temp_script("#!/bin/sh\necho \"hello world\"\n");
    bashrs_cmd()
        .arg("classify")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("safe"));
}

#[test]
fn test_PMAT142_classify_unsafe_script() {
    let f = write_temp_script("#!/bin/bash\neval \"$user_input\"\n");
    bashrs_cmd()
        .arg("classify")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("unsafe"));
}

#[test]
fn test_PMAT142_classify_json_output() {
    let f = write_temp_script("#!/bin/sh\necho hello\n");
    bashrs_cmd()
        .arg("classify")
        .arg("--json")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"label\""));
}

#[test]
fn test_PMAT142_classify_makefile() {
    let f = write_temp_script("all:\n\techo hello\n");
    bashrs_cmd()
        .arg("classify")
        .arg("--format")
        .arg("makefile")
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT142_classify_dockerfile() {
    let f = write_temp_script("FROM alpine:latest\nRUN echo hello\n");
    bashrs_cmd()
        .arg("classify")
        .arg("--format")
        .arg("dockerfile")
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT152_classify_multi_label_safe() {
    let f = write_temp_script("#!/bin/sh\necho \"hello world\"\n");
    bashrs_cmd()
        .arg("classify")
        .arg("--multi-label")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("safe"));
}

#[test]
fn test_PMAT152_classify_multi_label_unsafe() {
    let f = write_temp_script("#!/bin/bash\neval \"$x\"\nmkdir /tmp/build\necho $RANDOM\n");
    bashrs_cmd()
        .arg("classify")
        .arg("--multi-label")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("unsafe"));
}

#[test]
fn test_PMAT152_classify_multi_label_json() {
    let f = write_temp_script("#!/bin/bash\neval \"$x\"\n");
    bashrs_cmd()
        .arg("classify")
        .arg("--multi-label")
        .arg("--json")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"labels\""));
}

#[test]
fn test_PMAT152_classify_nonexistent_file() {
    bashrs_cmd()
        .arg("classify")
        .arg("/nonexistent/script.sh")
        .assert()
        .failure();
}

// ============================================================================
// bashrs explain
// ============================================================================

#[test]
fn test_PMAT142_explain_safe_script() {
    let f = write_temp_script("#!/bin/sh\necho \"hello world\"\n");
    bashrs_cmd()
        .arg("explain")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SAFE"));
}

#[test]
fn test_PMAT142_explain_unsafe_script() {
    let f = write_temp_script("#!/bin/bash\neval \"$user_input\"\n");
    bashrs_cmd()
        .arg("explain")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("SEC001"));
}

#[test]
fn test_PMAT142_explain_json_output() {
    let f = write_temp_script("#!/bin/bash\neval $x\n");
    bashrs_cmd()
        .arg("explain")
        .arg("--json")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"risk_level\""));
}

#[test]
fn test_PMAT142_explain_determinism_findings() {
    let f = write_temp_script("#!/bin/bash\necho $RANDOM\n");
    bashrs_cmd()
        .arg("explain")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("DET001"));
}

#[test]
fn test_PMAT142_explain_idempotency_findings() {
    let f = write_temp_script("#!/bin/sh\nmkdir /tmp/build\n");
    bashrs_cmd()
        .arg("explain")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("IDEM001"));
}

#[test]
fn test_PMAT152_explain_makefile() {
    let f = write_temp_script("all:\n\techo hello\n");
    bashrs_cmd()
        .arg("explain")
        .arg("--format")
        .arg("makefile")
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT152_explain_nonexistent_file() {
    bashrs_cmd()
        .arg("explain")
        .arg("/nonexistent/script.sh")
        .assert()
        .failure();
}

// ============================================================================
// bashrs fix
// ============================================================================

#[test]
fn test_PMAT142_fix_dry_run() {
    let f = write_temp_script("#!/bin/bash\necho $HOME\nmkdir /tmp/test\n");
    bashrs_cmd()
        .arg("fix")
        .arg("--dry-run")
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT142_fix_with_output() {
    let f = write_temp_script("#!/bin/bash\necho $HOME\n");
    let out = tempfile::NamedTempFile::new().unwrap();
    bashrs_cmd()
        .arg("fix")
        .arg("--output")
        .arg(out.path())
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT142_fix_assumptions_flag() {
    let f = write_temp_script("#!/bin/sh\nmkdir /tmp/build\n");
    bashrs_cmd()
        .arg("fix")
        .arg("--dry-run")
        .arg("--assumptions")
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT142_fix_safe_script_no_changes() {
    let f = write_temp_script("#!/bin/sh\necho \"hello\"\n");
    bashrs_cmd()
        .arg("fix")
        .arg("--dry-run")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("0 fixes"));
}

#[test]
fn test_PMAT152_fix_nonexistent_file() {
    bashrs_cmd()
        .arg("fix")
        .arg("--dry-run")
        .arg("/nonexistent/script.sh")
        .assert()
        .failure();
}

// ============================================================================
// bashrs safety-check
// ============================================================================

#[test]
fn test_PMAT142_safety_check_safe() {
    let f = write_temp_script("#!/bin/sh\necho \"hello world\"\n");
    bashrs_cmd()
        .arg("safety-check")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("safe"));
}

#[test]
fn test_PMAT142_safety_check_unsafe() {
    let f = write_temp_script("#!/bin/bash\neval \"$x\"\n");
    bashrs_cmd()
        .arg("safety-check")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("unsafe"));
}

#[test]
fn test_PMAT142_safety_check_json() {
    let f = write_temp_script("#!/bin/sh\necho hello\n");
    bashrs_cmd()
        .arg("safety-check")
        .arg("--json")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"label\""));
}

#[test]
fn test_PMAT152_safety_check_makefile() {
    let f = write_temp_script("all:\n\techo hello\n");
    bashrs_cmd()
        .arg("safety-check")
        .arg("--format")
        .arg("makefile")
        .arg(f.path())
        .assert()
        .success();
}

#[test]
fn test_PMAT152_safety_check_nonexistent_file() {
    bashrs_cmd()
        .arg("safety-check")
        .arg("/nonexistent/script.sh")
        .assert()
        .failure();
}

// ============================================================================
// bashrs corpus model-card
// ============================================================================

#[test]
fn test_PMAT142_corpus_model_card_stdout() {
    bashrs_cmd()
        .arg("corpus")
        .arg("model-card")
        .assert()
        .success()
        .stdout(predicate::str::contains("---"))
        .stdout(predicate::str::contains("Shell Safety"));
}

#[test]
fn test_PMAT142_corpus_model_card_to_file() {
    let out = tempfile::NamedTempFile::new().unwrap();
    bashrs_cmd()
        .arg("corpus")
        .arg("model-card")
        .arg("--output")
        .arg(out.path())
        .assert()
        .success();
    let content = std::fs::read_to_string(out.path()).unwrap();
    assert!(content.starts_with("---"));
    assert!(content.contains("license: apache-2.0"));
}

// ============================================================================
// bashrs corpus training-config
// ============================================================================

#[test]
fn test_PMAT142_corpus_training_config_yaml() {
    bashrs_cmd()
        .arg("corpus")
        .arg("training-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("architecture: encoder"))
        .stdout(predicate::str::contains("class_weights:"));
}

#[test]
fn test_PMAT142_corpus_training_config_json() {
    bashrs_cmd()
        .arg("corpus")
        .arg("training-config")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"architecture\""))
        .stdout(predicate::str::contains("\"class_weights\""));
}

#[test]
fn test_PMAT142_corpus_training_config_to_file() {
    let out = tempfile::NamedTempFile::new().unwrap();
    bashrs_cmd()
        .arg("corpus")
        .arg("training-config")
        .arg("--output")
        .arg(out.path())
        .assert()
        .success();
    let content = std::fs::read_to_string(out.path()).unwrap();
    assert!(content.contains("architecture: encoder"));
}

// ============================================================================
// bashrs corpus publish-dataset
// ============================================================================

#[test]
fn test_PMAT143_corpus_publish_dataset() {
    let dir = tempfile::tempdir().unwrap();
    bashrs_cmd()
        .arg("corpus")
        .arg("publish-dataset")
        .arg("--output")
        .arg(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Dataset published"))
        .stderr(predicate::str::contains("train.jsonl"))
        .stderr(predicate::str::contains("README.md"));
    // Verify files exist
    assert!(dir.path().join("README.md").exists());
    assert!(dir.path().join("train.jsonl").exists());
    assert!(dir.path().join("val.jsonl").exists());
    assert!(dir.path().join("test.jsonl").exists());
    assert!(dir.path().join("training_config.yaml").exists());
}

#[test]
fn test_PMAT143_corpus_publish_dataset_readme_has_yaml() {
    let dir = tempfile::tempdir().unwrap();
    bashrs_cmd()
        .arg("corpus")
        .arg("publish-dataset")
        .arg("--output")
        .arg(dir.path())
        .assert()
        .success();
    let readme = std::fs::read_to_string(dir.path().join("README.md")).unwrap();
    assert!(readme.contains("language:"));
    assert!(readme.contains("shell-safety"));
    assert!(readme.contains("binary-classification"));
}

#[test]
fn test_PMAT143_corpus_publish_dataset_splits_valid_jsonl() {
    let dir = tempfile::tempdir().unwrap();
    bashrs_cmd()
        .arg("corpus")
        .arg("publish-dataset")
        .arg("--output")
        .arg(dir.path())
        .assert()
        .success();
    // Verify train.jsonl has valid JSON lines
    let train = std::fs::read_to_string(dir.path().join("train.jsonl")).unwrap();
    let first_line = train.lines().next().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(first_line).unwrap();
    assert!(parsed.get("input").is_some());
    assert!(parsed.get("label").is_some());
}

#[test]
fn test_PMAT143_corpus_publish_dataset_config_yaml() {
    let dir = tempfile::tempdir().unwrap();
    bashrs_cmd()
        .arg("corpus")
        .arg("publish-dataset")
        .arg("--output")
        .arg(dir.path())
        .assert()
        .success();
    let config = std::fs::read_to_string(dir.path().join("training_config.yaml")).unwrap();
    assert!(config.contains("architecture:"));
    assert!(config.contains("class_weights:"));
}

// ============================================================================
// bashrs corpus publish-conversations
// ============================================================================

#[test]
fn test_PMAT153_corpus_publish_conversations() {
    let dir = tempfile::tempdir().unwrap();
    bashrs_cmd()
        .arg("corpus")
        .arg("publish-conversations")
        .arg("--output")
        .arg(dir.path())
        .arg("--seed")
        .arg("42")
        .assert()
        .success()
        .stderr(predicate::str::contains("PASSED"));
    // Verify output files exist
    assert!(dir.path().join("conversations.jsonl").exists());
    assert!(dir.path().join("README.md").exists());
    // Verify README has HuggingFace front matter
    let readme = std::fs::read_to_string(dir.path().join("README.md")).unwrap();
    assert!(readme.starts_with("---\n"));
    assert!(readme.contains("Shell Safety Conversations"));
    // Verify JSONL has system turns
    let first_line = std::fs::read_to_string(dir.path().join("conversations.jsonl"))
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();
    assert!(first_line.contains("\"system\""));
}

// ============================================================================
// bashrs corpus ssc-report --gate
// ============================================================================

#[test]
fn test_PMAT147_corpus_ssc_report_gate_passes() {
    bashrs_cmd()
        .arg("corpus")
        .arg("ssc-report")
        .arg("--gate")
        .assert()
        .success()
        .stderr(predicate::str::contains("All sections ready"));
}

#[test]
fn test_PMAT147_corpus_ssc_report_gate_json() {
    bashrs_cmd()
        .arg("corpus")
        .arg("ssc-report")
        .arg("--gate")
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"overall_ready\": true"));
}

#[test]
#[ignore] // ssc-report lints entire 17,942-entry corpus (~8 min)
fn test_PMAT158_corpus_ssc_report_has_wasm_section() {
    bashrs_cmd()
        .arg("corpus")
        .arg("ssc-report")
        .arg("--json")
        .timeout(std::time::Duration::from_secs(600))
        .assert()
        .success()
        .stdout(predicate::str::contains("wasm"));
}

#[test]
fn test_CHAT001_generate_conversations_entrenar_format() {
    bashrs_cmd()
        .arg("corpus")
        .arg("generate-conversations")
        .arg("--limit")
        .arg("3")
        .arg("--entrenar")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"instruction\""))
        .stdout(predicate::str::contains("\"response\""))
        .stdout(predicate::str::contains("\"system\""));
}

// ============================================================================
// bashrs corpus train-classifier (CLF-RUN step 2-3)
// ============================================================================

#[test]
fn test_PMAT154_corpus_train_classifier_from_synthetic() {
    // Create synthetic embeddings JSONL
    let dir = tempfile::tempdir().unwrap();
    let emb_path = dir.path().join("embeddings.jsonl");
    let mut f = std::fs::File::create(&emb_path).unwrap();
    use std::io::Write as _;
    // Write 20 synthetic embedding entries (10 safe, 10 unsafe)
    for i in 0..20 {
        let label = if i < 10 { 0 } else { 1 };
        let emb: Vec<f32> = (0..32)
            .map(|j| {
                if label == 0 {
                    if j < 16 {
                        1.0
                    } else {
                        -1.0
                    }
                } else {
                    if j < 16 {
                        -1.0
                    } else {
                        1.0
                    }
                }
            })
            .collect();
        let entry = serde_json::json!({
            "id": format!("test_{i}"),
            "embedding": emb,
            "label": label
        });
        writeln!(f, "{}", serde_json::to_string(&entry).unwrap()).unwrap();
    }
    drop(f);

    let out_dir = dir.path().join("output");
    bashrs_cmd()
        .arg("corpus")
        .arg("train-classifier")
        .arg("--embeddings")
        .arg(&emb_path)
        .arg("--output")
        .arg(&out_dir)
        .arg("--epochs")
        .arg("10")
        .arg("--seed")
        .arg("42")
        .assert()
        .success()
        .stderr(predicate::str::contains("Training linear probe"));

    // Verify output artifacts
    assert!(out_dir.join("probe.json").exists());
    assert!(out_dir.join("evaluation.json").exists());

    // Verify probe.json is valid
    let probe: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out_dir.join("probe.json")).unwrap())
            .unwrap();
    assert!(probe.get("weights").is_some());
    assert!(probe.get("bias").is_some());
    assert!(probe.get("epochs").is_some());
}

#[test]
fn test_PMAT154_classify_with_probe_flag() {
    // The --probe flag should be accepted even without ml feature
    let f = write_temp_script("#!/bin/sh\necho hello\n");
    let dir = tempfile::tempdir().unwrap();
    let probe_path = dir.path().join("probe.json");
    // Write a minimal probe (too small to be a real CodeBERT probe)
    std::fs::write(
        &probe_path,
        r#"{"weights":[0.5,-0.5],"bias":0.0,"epochs":1,"learning_rate":0.01,"train_accuracy":0.5,"train_mcc":0.0}"#,
    )
    .unwrap();
    bashrs_cmd()
        .arg("classify")
        .arg("--probe")
        .arg(&probe_path)
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("safe"));
}

#[test]
fn test_PMAT154_classify_with_probe_and_model_flags() {
    // --probe + --model should be accepted; without ml feature, shows note
    let f = write_temp_script("#!/bin/sh\necho hello\n");
    let dir = tempfile::tempdir().unwrap();
    let probe_path = dir.path().join("probe.json");
    std::fs::write(
        &probe_path,
        r#"{"weights":[0.5,-0.5],"bias":0.0,"epochs":1,"learning_rate":0.01,"train_accuracy":0.5,"train_mcc":0.0}"#,
    )
    .unwrap();
    bashrs_cmd()
        .arg("classify")
        .arg("--probe")
        .arg(&probe_path)
        .arg("--model")
        .arg("/tmp/nonexistent")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("safe"));
}

#[test]
fn test_PMAT154_classify_probe_without_model_shows_note() {
    // --probe without --model should show a helpful note
    let f = write_temp_script("#!/bin/sh\necho hello\n");
    let dir = tempfile::tempdir().unwrap();
    let probe_path = dir.path().join("probe.json");
    std::fs::write(
        &probe_path,
        r#"{"weights":[0.5,-0.5],"bias":0.0,"epochs":1,"learning_rate":0.01,"train_accuracy":0.5,"train_mcc":0.0}"#,
    )
    .unwrap();
    bashrs_cmd()
        .arg("classify")
        .arg("--probe")
        .arg(&probe_path)
        .arg(f.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("requires --model"));
}

#[test]
fn test_PMAT154_corpus_extract_embeddings_requires_ml() {
    // Without ml feature, extract-embeddings should fail with helpful message
    // (the test binary is built without --features ml)
    let dir = tempfile::tempdir().unwrap();
    bashrs_cmd()
        .arg("corpus")
        .arg("extract-embeddings")
        .arg("--model")
        .arg("/tmp/nonexistent-model")
        .arg("--output")
        .arg(dir.path().join("embeddings.jsonl"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("ml"));
}

// ============================================================================
// MLP probe CLI args (KAIZEN-107)
// ============================================================================

#[test]
fn test_KAIZEN107_classify_mlp_probe_requires_model() {
    let f = write_temp_script("#!/bin/sh\necho hello\n");
    // --mlp-probe without --model should print note and succeed (Stage 0 still runs)
    bashrs_cmd()
        .arg("classify")
        .arg("--mlp-probe")
        .arg("/tmp/nonexistent.json")
        .arg(f.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("requires --model"));
}

#[test]
fn test_KAIZEN107_train_classifier_mlp_flag_accepted() {
    // Verify --mlp and --mlp-hidden flags are accepted (will fail on missing embeddings)
    bashrs_cmd()
        .arg("corpus")
        .arg("train-classifier")
        .arg("--embeddings")
        .arg("/tmp/nonexistent.jsonl")
        .arg("--output")
        .arg("/tmp/test_mlp_out")
        .arg("--mlp")
        .arg("--mlp-hidden")
        .arg("16")
        .assert()
        .failure(); // Fails on missing file, not arg parsing
}

// ============================================================================
// bashrs explain --chat-model / bashrs fix --chat-model (Phase 4 CLI-002)
// ============================================================================

#[test]
fn test_CLI002_explain_chat_model_nonexistent_dir() {
    let f = write_temp_script("#!/bin/sh\necho $VAR\n");
    bashrs_cmd()
        .arg("explain")
        .arg(f.path())
        .arg("--chat-model")
        .arg("/tmp/nonexistent_model_dir_12345")
        .assert()
        .failure();
}

#[test]
fn test_CLI002_explain_chat_model_flag_accepted() {
    // Verify the --chat-model flag is accepted by the argument parser
    bashrs_cmd()
        .arg("explain")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("chat-model"));
}

#[test]
fn test_CLI002_fix_chat_model_flag_accepted() {
    // Verify the --chat-model flag is accepted by the argument parser
    bashrs_cmd()
        .arg("fix")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("chat-model"));
}

#[test]
fn test_CLI002_fix_chat_model_nonexistent_dir() {
    let f = write_temp_script("#!/bin/sh\necho $VAR\n");
    bashrs_cmd()
        .arg("fix")
        .arg(f.path())
        .arg("--chat-model")
        .arg("/tmp/nonexistent_model_dir_12345")
        .assert()
        .failure();
}

#[test]
fn test_CLI002_explain_without_chat_model_still_works() {
    // Rule-based explain should still work without --chat-model
    let f = write_temp_script("#!/bin/sh\necho \"hello\"\n");
    bashrs_cmd()
        .arg("explain")
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("safe").or(predicate::str::contains("SAFE")));
}

#[test]
fn test_CLI002_fix_without_chat_model_still_works() {
    // Rule-based fix should still work without --chat-model
    let f = write_temp_script("#!/bin/sh\necho \"hello\"\n");
    bashrs_cmd().arg("fix").arg(f.path()).assert().success();
}

// ============================================================================
// bashrs corpus cwe-mapping (SSC v12 S14.2)
// ============================================================================

#[test]
fn test_SSB001_corpus_cwe_mapping_human_output() {
    // FALSIFY-SSB-001: CWE mapping covers all linter rules
    bashrs_cmd()
        .args(["corpus", "cwe-mapping"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CWE Taxonomy Mapping"))
        .stdout(predicate::str::contains("SEC001"))
        .stdout(predicate::str::contains("SEC006"))
        .stdout(predicate::str::contains("CWE-829"))
        .stdout(predicate::str::contains("IDEM002"))
        .stdout(predicate::str::contains("OOD CWEs"))
        .stdout(predicate::str::contains("CWE-426"))
        .stdout(predicate::str::contains("disjoint=true"));
}

#[test]
fn test_SSB001_corpus_cwe_mapping_json_output() {
    // FALSIFY-SSB-001: JSON output has all required fields
    let output = bashrs_cmd()
        .args(["corpus", "cwe-mapping", "--json"])
        .output()
        .expect("cwe-mapping --json");
    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    let rules = json["linter_rules"].as_array().expect("linter_rules array");
    assert_eq!(rules.len(), 14, "Must have 14 linter rules");

    // Verify each rule has required fields
    for rule in rules {
        assert!(rule["rule"].is_string());
        assert!(rule["cwe"].is_string());
        assert!(rule["cvss_score"].is_number());
        assert!(rule["owasp"].is_string());
    }

    let ood = json["ood_cwes"].as_array().expect("ood_cwes array");
    assert_eq!(ood.len(), 4, "Must have 4 OOD CWEs");

    let summary = &json["summary"];
    assert_eq!(summary["total_rules"], 14);
    assert_eq!(summary["ood_disjoint"], true);
}

#[test]
fn test_SSB003_corpus_export_benchmark_produces_jsonl() {
    // FALSIFY-SSB-003: Benchmark export has DPO schema
    let output = bashrs_cmd()
        .args(["corpus", "export-benchmark", "--limit", "5"])
        .output()
        .expect("export-benchmark");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert!(lines.len() >= 1, "Should have at least 1 entry");

    // Verify first line is valid JSON with required DPO fields
    let entry: serde_json::Value = serde_json::from_str(lines[0]).expect("valid JSON");
    assert!(entry["id"].is_string(), "Must have id");
    assert!(entry["lang"].is_string(), "Must have lang");
    assert!(entry["script"].is_string(), "Must have script");
    assert!(entry["chosen"].is_string(), "Must have chosen");
    assert!(entry["rejected"].is_string(), "Must have rejected");
    assert!(entry["source"].is_string(), "Must have source");
    assert!(
        entry["conversation_type"].is_string(),
        "Must have conversation_type"
    );
}

#[test]
fn test_SSB005_corpus_cwe_mapping_cvss_scores_valid() {
    // FALSIFY-SSB-005: All CVSS scores in valid range
    let output = bashrs_cmd()
        .args(["corpus", "cwe-mapping", "--json"])
        .output()
        .expect("cwe-mapping --json");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    for rule in json["linter_rules"].as_array().unwrap() {
        let score = rule["cvss_score"].as_f64().unwrap();
        assert!(
            (0.0..=10.0).contains(&score),
            "CVSS score {} out of range for {}",
            score,
            rule["rule"]
        );
    }
}

#[test]
fn test_SSB002_conversations_contain_shell_not_rust() {
    // FALSIFY-SSB-002: Conversations must contain shell code, not Rust DSL
    let output = bashrs_cmd()
        .args([
            "corpus",
            "generate-conversations",
            "--limit",
            "10",
            "--seed",
            "42",
        ])
        .output()
        .expect("generate-conversations");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.trim().lines() {
        let entry: serde_json::Value = serde_json::from_str(line).expect("valid JSON");
        for turn in entry["turns"].as_array().unwrap() {
            if turn["role"].as_str() == Some("user") {
                let content = turn["content"].as_str().unwrap_or("");
                // Must not contain Rust DSL patterns (fn main with let)
                assert!(
                    !content.contains("fn main() {") || content.contains("echo 'fn main"),
                    "User turn contains Rust DSL: {}",
                    &content[..content.len().min(100)]
                );
            }
        }
    }
}

#[test]
fn test_SSB006_eval_harness_weights_sum_to_one() {
    // Eval harness weights must sum to 1.0
    use bashrs::corpus::eval_harness;
    let sum = eval_harness::DETECTION_F1_WEIGHT
        + eval_harness::RULE_CITATION_WEIGHT
        + eval_harness::CWE_MAPPING_WEIGHT
        + eval_harness::FIX_VALIDITY_WEIGHT
        + eval_harness::EXPLANATION_WEIGHT
        + eval_harness::OOD_WEIGHT;
    assert!(
        (sum - 1.0).abs() < 1e-9,
        "Eval weights must sum to 1.0, got {sum}"
    );
}

#[test]
fn test_SSB007_corpus_label_external_jsonl() {
    // Label command should add classification, findings, and CWE mappings
    let test_input = r#"{"script": "eval $user_data", "id": "test-001"}
{"script": "echo 'hello'", "id": "test-002"}"#;

    let input_path = std::env::temp_dir().join("bashrs_test_label.jsonl");
    std::fs::write(&input_path, test_input).expect("write test input");

    let output = bashrs_cmd()
        .args(["corpus", "label", "--input"])
        .arg(&input_path)
        .output()
        .expect("label command");
    assert!(output.status.success(), "label must succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 2, "Should output 2 labeled entries");

    // First entry (eval) should be unsafe with CWE
    let entry1: serde_json::Value = serde_json::from_str(lines[0]).expect("valid JSON");
    assert_eq!(entry1["label"], 1, "eval should be unsafe");
    assert_eq!(entry1["classification"], "unsafe");
    assert!(
        entry1["findings"]
            .as_array()
            .map_or(false, |a| !a.is_empty()),
        "Should have findings"
    );

    // Second entry should be safe
    let entry2: serde_json::Value = serde_json::from_str(lines[1]).expect("valid JSON");
    assert_eq!(entry2["label"], 0, "echo should be safe");
    assert_eq!(entry2["classification"], "safe");

    std::fs::remove_file(&input_path).ok();
}

// ============================================================================
// bashrs corpus pipeline-check
// ============================================================================

/// Get project root directory for tests that need config file access
fn project_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn test_KAIZEN095_pipeline_check_runs() {
    bashrs_cmd()
        .current_dir(project_root())
        .args(["corpus", "pipeline-check"])
        .assert()
        .success();
}

#[test]
fn test_KAIZEN095_pipeline_check_json_output() {
    let output = bashrs_cmd()
        .current_dir(project_root())
        .args(["corpus", "pipeline-check", "--json"])
        .output()
        .expect("pipeline-check should run");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON output");

    assert!(json["pipeline_ready"].is_boolean());
    assert!(json["tools"].is_array());
    assert!(json["configs"].is_array());
    assert!(json["artifacts"].is_array());

    // bashrs must always be available
    let tools = json["tools"].as_array().expect("tools array");
    let bashrs_tool = tools
        .iter()
        .find(|t| t["tool"] == "bashrs")
        .expect("bashrs in tools");
    assert_eq!(bashrs_tool["available"], true);
}

// ── FALSIFY-SSB-010: Verificar label integration ──

#[test]
fn test_KAIZEN095_verificar_label_integration() {
    // Create a temp file with verificar mutation format
    let mut input = NamedTempFile::new().expect("temp file");
    let mutation = serde_json::json!({
        "safe_script": "#!/bin/sh\ncp \"${src}\" \"${dst}\"",
        "unsafe_script": "#!/bin/sh\nrm -rf $dir/tmp",
        "cwe": "CWE-78",
        "cwe_id": 78,
        "vulnerability": "OS Command Injection",
        "mutation_description": "Unquoted variable in rm"
    });
    writeln!(input, "{}", serde_json::to_string(&mutation).unwrap()).unwrap();
    input.flush().unwrap();

    let output_file = NamedTempFile::new().expect("temp output");
    let output_path = output_file.path().to_str().unwrap();

    bashrs_cmd()
        .args([
            "corpus",
            "label",
            "--input",
            input.path().to_str().unwrap(),
            "-o",
            output_path,
        ])
        .assert()
        .success();

    let contents = std::fs::read_to_string(output_path).expect("read output");
    let labeled: serde_json::Value = serde_json::from_str(contents.trim()).expect("valid JSON");

    // Should have labeled the unsafe_script
    assert!(labeled["label"].is_number(), "should have label field");
    assert!(
        labeled["classification"].is_string(),
        "should have classification"
    );
    assert!(labeled["findings"].is_array(), "should have findings");
    // Preserves original verificar fields
    assert_eq!(labeled["cwe"], "CWE-78");
    assert_eq!(labeled["cwe_id"], 78);
}

// ── merge-data tests ──

#[test]
fn test_KAIZEN095_merge_data_combines_sources() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input_a = dir.path().join("a.jsonl");
    let input_b = dir.path().join("b.jsonl");
    let output = dir.path().join("merged.jsonl");

    // Write two JSONL files
    std::fs::write(
        &input_a,
        "{\"script\":\"echo a\",\"label\":0}\n{\"script\":\"echo b\",\"label\":0}\n",
    )
    .unwrap();
    std::fs::write(&input_b, "{\"unsafe_script\":\"rm -rf $x\",\"label\":1}\n").unwrap();

    bashrs_cmd()
        .current_dir(project_root())
        .args([
            "corpus",
            "merge-data",
            "--input",
            input_a.to_str().unwrap(),
            "--input",
            input_b.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();

    let contents = std::fs::read_to_string(&output).expect("read merged");
    let lines: Vec<&str> = contents.lines().filter(|l| !l.is_empty()).collect();
    // Should have at least the 3 entries from input files
    // (corpus conversations may or may not exist depending on env)
    assert!(
        lines.len() >= 3,
        "should merge at least 3 entries, got {}",
        lines.len()
    );
}

#[test]
fn test_KAIZEN095_merge_data_normalizes_verificar_schema() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input_verif = dir.path().join("verif.jsonl");
    let output = dir.path().join("merged.jsonl");

    // Verificar mutation format (unsafe_script, safe_script, cwe, etc.)
    let verif_entry = serde_json::json!({
        "unsafe_script": "#!/bin/sh\nrm -rf $dir/tmp",
        "safe_script": "#!/bin/sh\nrm -rf \"${dir:?}\"/tmp",
        "cwe": "CWE-78",
        "cwe_id": 78,
        "vulnerability": "OS Command Injection",
        "mutation_description": "Unquoted variable",
        "label": 1,
        "classification": "unsafe",
        "findings": [{"rule": "SEC010"}]
    });
    std::fs::write(
        &input_verif,
        format!("{}\n", serde_json::to_string(&verif_entry).unwrap()),
    )
    .unwrap();

    bashrs_cmd()
        .current_dir(project_root())
        .args([
            "corpus",
            "merge-data",
            "--input",
            input_verif.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();

    let contents = std::fs::read_to_string(&output).expect("read merged");
    // Find the verificar entry (has source=verificar)
    for line in contents.lines() {
        if line.is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(line).expect("valid JSON");
        if v.get("source").and_then(|s| s.as_str()) == Some("verificar") {
            // Must have conversation fields
            assert!(
                v.get("instruction").is_some(),
                "must have instruction field"
            );
            assert!(v.get("response").is_some(), "must have response field");
            assert!(v.get("system").is_some(), "must have system field");
            assert!(v.get("text").is_some(), "must have text field");
            // Instruction should contain the unsafe script in a bash code block
            let instr = v["instruction"].as_str().expect("instruction is string");
            assert!(
                instr.contains("```bash"),
                "instruction should contain bash code block"
            );
            assert!(
                instr.contains("rm -rf"),
                "instruction should contain the script"
            );
            // Response should reference the CWE
            let resp = v["response"].as_str().expect("response is string");
            assert!(resp.contains("CWE-78"), "response should reference CWE");
            assert!(resp.contains("Fixed version"), "response should have fix");
            return;
        }
    }
    panic!("no verificar entry found in merged output");
}

// ── ShellSafetyBench cross-validation tests ──

#[test]
fn test_SSB006_shellcheck_validate_json_output() {
    // Create a minimal splits file so the fast path is used
    let dir = tempfile::tempdir().expect("create tempdir");
    let splits_dir = dir.path().join("training/shellsafetybench/splits");
    std::fs::create_dir_all(&splits_dir).expect("create splits dir");
    let test_jsonl = splits_dir.join("test.jsonl");
    std::fs::write(
        &test_jsonl,
        "{\"input\":\"echo hello\",\"label\":0}\n{\"input\":\"rm -rf $dir\",\"label\":1}\n{\"input\":\"echo world\",\"label\":0}\n",
    )
    .expect("write test.jsonl");

    bashrs_cmd()
        .current_dir(dir.path())
        .args(["corpus", "shellcheck-validate", "--samples", "3", "--json"])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
}

#[test]
fn test_SSB007_eval_benchmark_requires_predictions_file() {
    bashrs_cmd()
        .args([
            "corpus",
            "eval-benchmark",
            "--predictions",
            "/nonexistent/predictions.jsonl",
        ])
        .assert()
        .failure();
}

#[test]
fn test_SSB008_eval_benchmark_with_synthetic_predictions() {
    // Create synthetic predictions JSONL
    let mut f = NamedTempFile::new().expect("create temp");
    let pred1 = serde_json::json!({
        "id": "SSB-00001",
        "classification": "unsafe",
        "label": 1,
        "cited_rules": ["SEC001"],
        "cited_cwes": ["CWE-78"],
        "explanation": "This script is unsafe. SEC001: unquoted variable allows injection. Use double quotes instead.",
        "ground_truth_rules": ["SEC001"],
        "ground_truth_cwes": ["CWE-78"],
        "script": "#!/bin/sh\nrm -rf $dir"
    });
    let pred2 = serde_json::json!({
        "id": "SSB-00002",
        "classification": "safe",
        "label": 0,
        "explanation": "This script is safe.",
        "script": "#!/bin/sh\necho hello"
    });
    writeln!(f, "{}", serde_json::to_string(&pred1).unwrap()).unwrap();
    writeln!(f, "{}", serde_json::to_string(&pred2).unwrap()).unwrap();
    f.flush().unwrap();

    bashrs_cmd()
        .args([
            "corpus",
            "eval-benchmark",
            "--predictions",
            f.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("detection_f1"))
        .stdout(predicate::str::contains("weighted_score"));
}

// ============================================================================
// bashrs corpus batch-eval (SSC v12 S14 — batch inference bridge)
// ============================================================================

#[test]
fn test_SSC_batch_eval_requires_ml_feature() {
    // Without --features ml, batch-eval should fail with a clear message.
    // We provide valid paths (even though model doesn't exist) to test
    // the feature gate, not the file loading.
    let test_data = write_temp_script(
        r#"{"input": "echo hello", "label": 0}
{"input": "eval \"$x\"", "label": 1}
"#,
    );
    let output = tempfile::NamedTempFile::new().unwrap();
    bashrs_cmd()
        .args([
            "corpus",
            "batch-eval",
            "--model",
            "/tmp/nonexistent-model",
            "--test-data",
            test_data.path().to_str().unwrap(),
            "--output",
            output.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ml"));
}

#[test]
fn test_SSC_batch_eval_help() {
    bashrs_cmd()
        .args(["corpus", "batch-eval", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--model"))
        .stdout(predicate::str::contains("--test-data"))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--max-tokens"));
}

#[test]
fn test_SSC_batch_eval_missing_args() {
    bashrs_cmd()
        .args(["corpus", "batch-eval"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--model"))
        .stderr(predicate::str::contains("--test-data"))
        .stderr(predicate::str::contains("--output"));
}
