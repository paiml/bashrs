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

include!("cli_ssc_tests_tests_kaizen107_cl.rs");
