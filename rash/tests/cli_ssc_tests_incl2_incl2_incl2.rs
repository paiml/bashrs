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
