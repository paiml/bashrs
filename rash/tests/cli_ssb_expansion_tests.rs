#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(deprecated)]
#![allow(non_snake_case)]
//! SSB Expansion CLI Tests (PMAT-172, PMAT-176)
//!
//! Tests for publish-benchmark and generate-expansion commands.
//! Extracted from cli_ssc_tests.rs for file-size discipline.

use assert_cmd::Command;
use predicates::prelude::*;

fn bashrs_cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("bashrs")
}


// ============================================================================
// bashrs corpus publish-benchmark (PMAT-172)
// ============================================================================

#[test]
fn test_PMAT172_corpus_publish_benchmark_help() {
    bashrs_cmd()
        .args(["corpus", "publish-benchmark", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ShellSafetyBench"))
        .stdout(predicate::str::contains("--input"))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--version"));
}

#[test]
fn test_PMAT172_corpus_publish_benchmark_missing_args() {
    bashrs_cmd()
        .args(["corpus", "publish-benchmark"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--input"))
        .stderr(predicate::str::contains("--output"));
}

#[test]
fn test_PMAT172_corpus_publish_benchmark_creates_files() {
    let splits_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();

    // Write minimal split files
    std::fs::write(
        splits_dir.path().join("train.jsonl"),
        "{\"input\":\"echo hello\",\"label\":0}\n{\"input\":\"eval $cmd\",\"label\":1}\n",
    )
    .unwrap();
    std::fs::write(
        splits_dir.path().join("val.jsonl"),
        "{\"input\":\"ls -la\",\"label\":0}\n",
    )
    .unwrap();
    std::fs::write(
        splits_dir.path().join("test.jsonl"),
        "{\"input\":\"rm -rf /\",\"label\":1}\n",
    )
    .unwrap();

    bashrs_cmd()
        .args([
            "corpus",
            "publish-benchmark",
            "--input",
            splits_dir.path().to_str().unwrap(),
            "--output",
            output_dir.path().to_str().unwrap(),
            "--version",
            "1.0.0",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "ShellSafetyBench v1.0.0 published",
        ));

    // Verify all expected files
    assert!(output_dir.path().join("README.md").exists());
    assert!(output_dir.path().join("train.jsonl").exists());
    assert!(output_dir.path().join("validation.jsonl").exists());
    assert!(output_dir.path().join("test.jsonl").exists());
    assert!(output_dir.path().join("dataset_infos.json").exists());
}

#[test]
fn test_PMAT172_corpus_publish_benchmark_readme_has_hf_yaml() {
    let splits_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();

    std::fs::write(
        splits_dir.path().join("train.jsonl"),
        "{\"input\":\"echo hi\",\"label\":0}\n",
    )
    .unwrap();
    std::fs::write(
        splits_dir.path().join("val.jsonl"),
        "{\"input\":\"cat f\",\"label\":0}\n",
    )
    .unwrap();
    std::fs::write(
        splits_dir.path().join("test.jsonl"),
        "{\"input\":\"eval $x\",\"label\":1}\n",
    )
    .unwrap();

    bashrs_cmd()
        .args([
            "corpus",
            "publish-benchmark",
            "-i",
            splits_dir.path().to_str().unwrap(),
            "-o",
            output_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    let readme = std::fs::read_to_string(output_dir.path().join("README.md")).unwrap();
    assert!(readme.starts_with("---\n"));
    assert!(readme.contains("pretty_name: ShellSafetyBench"));
    assert!(readme.contains("license: apache-2.0"));
    assert!(readme.contains("binary-classification"));
    assert!(readme.contains("CWE"));
}

#[test]
fn test_PMAT172_corpus_publish_benchmark_validation_not_val() {
    // HuggingFace expects "validation.jsonl" not "val.jsonl"
    let splits_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();

    std::fs::write(
        splits_dir.path().join("train.jsonl"),
        "{\"input\":\"echo hi\",\"label\":0}\n",
    )
    .unwrap();
    std::fs::write(
        splits_dir.path().join("val.jsonl"),
        "{\"input\":\"cat f\",\"label\":0}\n",
    )
    .unwrap();
    std::fs::write(
        splits_dir.path().join("test.jsonl"),
        "{\"input\":\"pwd\",\"label\":0}\n",
    )
    .unwrap();

    bashrs_cmd()
        .args([
            "corpus",
            "publish-benchmark",
            "-i",
            splits_dir.path().to_str().unwrap(),
            "-o",
            output_dir.path().to_str().unwrap(),
        ])
        .assert()
        .success();

    // Must produce "validation.jsonl" (HF naming) not "val.jsonl"
    assert!(output_dir.path().join("validation.jsonl").exists());
    assert!(!output_dir.path().join("val.jsonl").exists());
}

#[test]
fn test_PMAT172_corpus_publish_benchmark_invalid_splits_dir() {
    let output_dir = tempfile::tempdir().unwrap();

    bashrs_cmd()
        .args([
            "corpus",
            "publish-benchmark",
            "-i",
            "/nonexistent/path",
            "-o",
            output_dir.path().to_str().unwrap(),
        ])
        .assert()
        .failure();
}

// ============================================================================
// bashrs corpus generate-expansion (PMAT-176)
// ============================================================================

#[test]
fn test_PMAT176_corpus_generate_expansion_help() {
    bashrs_cmd()
        .args(["corpus", "generate-expansion", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--count"))
        .stdout(predicate::str::contains("--output"));
}

#[test]
fn test_PMAT176_corpus_generate_expansion_bash() {
    let output_dir = tempfile::tempdir().unwrap();
    let output_path = output_dir.path().join("bash_expansion.jsonl");

    bashrs_cmd()
        .args([
            "corpus",
            "generate-expansion",
            "-f",
            "bash",
            "-c",
            "100",
            "-o",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Generated 100 bash entries"));

    assert!(output_path.exists());
    let content = std::fs::read_to_string(&output_path).unwrap();
    let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 100);
}

#[test]
fn test_PMAT176_corpus_generate_expansion_makefile() {
    let output_dir = tempfile::tempdir().unwrap();
    let output_path = output_dir.path().join("make_expansion.jsonl");

    bashrs_cmd()
        .args([
            "corpus",
            "generate-expansion",
            "-f",
            "makefile",
            "-c",
            "50",
            "-o",
            output_path.to_str().unwrap(),
            "-s",
            "99",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Generated 50 makefile entries"));
}

#[test]
fn test_PMAT176_corpus_generate_expansion_dockerfile() {
    let output_dir = tempfile::tempdir().unwrap();
    let output_path = output_dir.path().join("docker_expansion.jsonl");

    bashrs_cmd()
        .args([
            "corpus",
            "generate-expansion",
            "-f",
            "dockerfile",
            "-c",
            "50",
            "-o",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Generated 50 dockerfile entries"));
}

#[test]
fn test_PMAT176_corpus_generate_expansion_invalid_format() {
    let output_dir = tempfile::tempdir().unwrap();
    let output_path = output_dir.path().join("out.jsonl");

    bashrs_cmd()
        .args([
            "corpus",
            "generate-expansion",
            "-f",
            "invalid",
            "-o",
            output_path.to_str().unwrap(),
        ])
        .assert()
        .failure();
}
