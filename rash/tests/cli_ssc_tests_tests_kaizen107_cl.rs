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

include!("cli_ssc_tests_tests_ssb002_conve.rs");
