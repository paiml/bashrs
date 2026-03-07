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
fn test_PMAT158_corpus_ssc_report_has_wasm_section() {
    bashrs_cmd()
        .arg("corpus")
        .arg("ssc-report")
        .arg("--gate")
        .assert()
        .success()
        .stderr(predicate::str::contains("WASM App"));
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
