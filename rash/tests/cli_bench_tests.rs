#![allow(clippy::unwrap_used)] // Tests can use unwrap() for simplicity
// CLI Bench Command Tests - EXTREME TDD
// Testing bashrs bench sub-command with NASA-quality standards

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper function to create rash command
fn bashrs_cmd() -> Command {
    Command::cargo_bin("bashrs").expect("Failed to find bashrs binary")
}

/// Create a simple deterministic test script
fn create_test_script(dir: &TempDir, name: &str, content: &str) -> String {
    let script_path = dir.path().join(name);
    fs::write(&script_path, content).unwrap();
    script_path.to_str().unwrap().to_string()
}

// ===== BASIC FUNCTIONALITY TESTS =====

#[test]
fn test_bench_basic_execution() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(&temp_dir, "simple.sh", "#!/bin/bash\necho 'Hello World'\n");

    bashrs_cmd()
        .arg("bench")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains("Benchmarking"))
        .stdout(predicate::str::contains("Results for"))
        .stdout(predicate::str::contains("Mean"))
        .stdout(predicate::str::contains("Median"));
}

#[test]
fn test_bench_custom_iterations() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(&temp_dir, "simple.sh", "#!/bin/bash\necho 'test'\n");

    bashrs_cmd()
        .arg("bench")
        .arg(&script)
        .arg("--iterations")
        .arg("5")
        .assert()
        .success()
        .stdout(predicate::str::contains("5 iterations"));
}

#[test]
fn test_bench_custom_warmup() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(&temp_dir, "simple.sh", "#!/bin/bash\necho 'test'\n");

    bashrs_cmd()
        .arg("bench")
        .arg(&script)
        .arg("--warmup")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Warmup"));
}

#[test]
fn test_bench_multiple_scripts() {
    let temp_dir = TempDir::new().unwrap();
    let script1 = create_test_script(&temp_dir, "script1.sh", "#!/bin/bash\necho 'fast'\n");
    let script2 = create_test_script(
        &temp_dir,
        "script2.sh",
        "#!/bin/bash\nsleep 0.01\necho 'slow'\n",
    );

    bashrs_cmd()
        .arg("bench")
        .arg(&script1)
        .arg(&script2)
        .assert()
        .success()
        .stdout(predicate::str::contains("Comparison Results"))
        .stdout(predicate::str::contains("script1.sh"))
        .stdout(predicate::str::contains("script2.sh"));
}

// ===== OUTPUT FORMAT TESTS =====

#[test]
fn test_bench_console_output() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(&temp_dir, "test.sh", "#!/bin/bash\necho 'output'\n");

    bashrs_cmd()
        .arg("bench")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains("📊 Benchmarking"))
        .stdout(predicate::str::contains("⏱️  Measuring"))
        .stdout(predicate::str::contains("📈 Results"));
}

#[test]
fn test_bench_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(&temp_dir, "test.sh", "#!/bin/bash\necho 'json test'\n");
    let output_file = temp_dir.path().join("results.json");

    bashrs_cmd()
        .arg("bench")
        .arg(&script)
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .assert()
        .success();

    // Verify JSON file was created
    assert!(output_file.exists(), "JSON output file should be created");

    // Verify JSON structure
    let content = fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("\"version\""), "JSON should have version");
    assert!(
        content.contains("\"timestamp\""),
        "JSON should have timestamp"
    );
    assert!(
        content.contains("\"environment\""),
        "JSON should have environment"
    );
    assert!(
        content.contains("\"benchmarks\""),
        "JSON should have benchmarks"
    );
    assert!(content.contains("\"mean_ms\""), "JSON should have mean_ms");
    assert!(
        content.contains("\"median_ms\""),
        "JSON should have median_ms"
    );
    assert!(
        content.contains("\"stddev_ms\""),
        "JSON should have stddev_ms"
    );
}

#[test]
fn test_bench_comparison_output() {
    let temp_dir = TempDir::new().unwrap();
    let script1 = create_test_script(&temp_dir, "fast.sh", "#!/bin/bash\necho 'fast'\n");
    let script2 = create_test_script(
        &temp_dir,
        "slow.sh",
        "#!/bin/bash\nsleep 0.01\necho 'slow'\n",
    );

    bashrs_cmd()
        .arg("bench")
        .arg(&script1)
        .arg(&script2)
        .assert()
        .success()
        .stdout(predicate::str::contains("Speedup"))
        .stdout(predicate::str::contains("🏆 Winner"));
}

// ===== QUALITY GATES TESTS =====

#[test]
fn test_bench_strict_mode_lint_fail() {
    let temp_dir = TempDir::new().unwrap();
    // Script with linting issues (unquoted variable)
    let script = create_test_script(&temp_dir, "bad.sh", "#!/bin/bash\nvar=test\necho $var\n");

    bashrs_cmd()
        .arg("bench")
        .arg(&script)
        .arg("--strict")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Quality gate failed"));
}

#[test]
fn test_bench_strict_mode_lint_pass() {
    let temp_dir = TempDir::new().unwrap();
    // Script with no linting issues (no variables, just echo)
    let script = create_test_script(&temp_dir, "good.sh", "#!/bin/sh\necho 'test'\n");

    bashrs_cmd()
        .arg("bench")
        .arg(&script)
        .arg("--strict")
        .assert()
        .success()
        .stdout(predicate::str::contains("Results for"));
}

#[test]
fn test_bench_determinism_verification_pass() {
    let temp_dir = TempDir::new().unwrap();
    // Deterministic script
    let script = create_test_script(
        &temp_dir,
        "deterministic.sh",
        "#!/bin/bash\necho 'always same'\n",
    );

    bashrs_cmd()
        .arg("bench")
        .arg(&script)
        .arg("--verify-determinism")
        .assert()
        .success()
        .stdout(predicate::str::contains("Determinism verified"));
}

#[test]
fn test_bench_determinism_verification_fail() {
    let temp_dir = TempDir::new().unwrap();
    // Non-deterministic script (uses $RANDOM)
    let script = create_test_script(&temp_dir, "random.sh", "#!/bin/bash\necho $RANDOM\n");

    bashrs_cmd()
        .arg("bench")
        .arg(&script)
        .arg("--verify-determinism")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Determinism verification failed"))
        .stderr(predicate::str::contains("Output differs"));
}

// ===== STATISTICS TESTS =====

#[test]
fn test_bench_statistics_accuracy() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(&temp_dir, "stats.sh", "#!/bin/bash\necho 'test'\n");

    let output_file = temp_dir.path().join("stats.json");

    bashrs_cmd()
        .arg("bench")
        .arg(&script)
        .arg("--iterations")
        .arg("10")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .assert()
        .success();

    // Verify statistics in JSON
    let content = fs::read_to_string(&output_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    let stats = &json["benchmarks"][0]["statistics"];
    assert!(stats["mean_ms"].is_number(), "Mean should be a number");
    assert!(stats["median_ms"].is_number(), "Median should be a number");
    assert!(stats["stddev_ms"].is_number(), "StdDev should be a number");
    assert!(stats["min_ms"].is_number(), "Min should be a number");
    assert!(stats["max_ms"].is_number(), "Max should be a number");

    // Verify raw results array
    let raw_results = &json["benchmarks"][0]["raw_results_ms"];
    assert!(raw_results.is_array(), "Raw results should be array");
    assert_eq!(
        raw_results.as_array().unwrap().len(),
        10,
        "Should have 10 raw results"
    );
}

#[test]
fn test_bench_environment_capture() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(&temp_dir, "env.sh", "#!/bin/bash\necho 'env test'\n");

    let output_file = temp_dir.path().join("env.json");

    bashrs_cmd()
        .arg("bench")
        .arg(&script)
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .assert()
        .success();

    // Verify environment metadata in JSON
    let content = fs::read_to_string(&output_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    let env = &json["environment"];
    assert!(env["cpu"].is_string(), "CPU should be captured");
    assert!(env["ram"].is_string(), "RAM should be captured");
    assert!(env["os"].is_string(), "OS should be captured");
    assert!(env["hostname"].is_string(), "Hostname should be captured");
    assert!(
        env["bashrs_version"].is_string(),
        "bashrs version should be captured"
    );
}

// ===== ERROR HANDLING TESTS =====

#[test]
fn test_bench_nonexistent_script() {
    bashrs_cmd()
        .arg("bench")
        .arg("nonexistent_script.sh")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such file")));
}

#[test]
fn test_bench_invalid_iterations() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(&temp_dir, "test.sh", "#!/bin/bash\necho 'test'\n");

    bashrs_cmd()
        .arg("bench")
        .arg(&script)
        .arg("--iterations")
        .arg("0")
        .assert()
        .failure()
        .stderr(predicate::str::contains("must be").or(predicate::str::contains("invalid")));
}

#[test]
fn test_bench_show_raw_results() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(&temp_dir, "test.sh", "#!/bin/bash\necho 'test'\n");

    bashrs_cmd()
        .arg("bench")
        .arg(&script)
        .arg("--show-raw")
        .assert()
        .success()
        .stdout(predicate::str::contains("Raw results"));
}

#[test]
fn test_bench_quiet_mode() {
    let temp_dir = TempDir::new().unwrap();
    let script = create_test_script(&temp_dir, "test.sh", "#!/bin/bash\necho 'test'\n");

    let output_file = temp_dir.path().join("quiet.json");

    bashrs_cmd()
        .arg("bench")
        .arg(&script)
        .arg("--quiet")
        .arg("--output")
        .arg(output_file.to_str().unwrap())
        .assert()
        .success()
        // Quiet mode should suppress bench output (but INFO logs are okay)
        .stdout(predicate::str::contains("📊").not())
        .stdout(predicate::str::contains("📈").not())
        .stdout(predicate::str::contains("🔥").not());

    // But JSON file should still be created
    assert!(
        output_file.exists(),
        "JSON output should be created even in quiet mode"
    );
}
