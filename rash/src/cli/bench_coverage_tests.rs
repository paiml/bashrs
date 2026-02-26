#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

// =============================================================================
// Coverage tests for bench.rs display/formatting functions
// Targets: display_comparison_results, display_results, display_csv_results
// =============================================================================

/// Helper to create a mock BenchmarkResult with given parameters
fn make_bench_result(script: &str, mean: f64, stddev: f64, iterations: usize) -> BenchmarkResult {
    let raw = vec![mean - stddev, mean, mean + stddev];
    BenchmarkResult {
        script: script.to_string(),
        iterations,
        warmup: 3,
        statistics: Statistics {
            mean_ms: mean,
            median_ms: mean,
            stddev_ms: stddev,
            min_ms: mean - stddev,
            max_ms: mean + stddev,
            variance_ms: stddev * stddev,
            mad_ms: stddev * 0.6745,
            geometric_mean_ms: mean * 0.99,
            harmonic_mean_ms: mean * 0.98,
            outlier_indices: vec![],
            memory: None,
        },
        raw_results_ms: raw,
        quality: Quality {
            lint_passed: true,
            determinism_score: 1.0,
            output_identical: true,
        },
    }
}

/// Helper to create a BenchmarkResult with memory statistics
fn make_bench_result_with_memory(
    script: &str,
    mean: f64,
    stddev: f64,
    mem_mean: f64,
    mem_peak: f64,
) -> BenchmarkResult {
    let mut result = make_bench_result(script, mean, stddev, 10);
    result.statistics.memory = Some(MemoryStatistics {
        mean_kb: mem_mean,
        median_kb: mem_mean,
        min_kb: mem_mean * 0.9,
        max_kb: mem_peak,
        peak_kb: mem_peak,
    });
    result
}

/// Helper to create a mock Environment
fn make_environment() -> Environment {
    Environment {
        cpu: "Test CPU i7-9900K".to_string(),
        ram: "32GB".to_string(),
        os: "Linux 6.8".to_string(),
        hostname: "testhost".to_string(),
        bashrs_version: "1.0.0-test".to_string(),
    }
}

// =============================================================================
// display_csv_results tests
// =============================================================================

#[test]
fn test_display_csv_results_single_result_no_memory() {
    let results = vec![make_bench_result("script_a.sh", 15.5, 2.3, 10)];
    let result = display_csv_results(&results);
    assert!(result.is_ok());
}

#[test]
fn test_display_csv_results_multiple_results_no_memory() {
    let results = vec![
        make_bench_result("fast.sh", 5.0, 0.5, 10),
        make_bench_result("medium.sh", 15.0, 1.5, 10),
        make_bench_result("slow.sh", 50.0, 5.0, 10),
    ];
    let result = display_csv_results(&results);
    assert!(result.is_ok());
}

#[test]
fn test_display_csv_results_with_memory() {
    let results = vec![
        make_bench_result_with_memory("mem_test.sh", 10.0, 1.0, 1024.0, 2048.0),
        make_bench_result_with_memory("mem_test2.sh", 20.0, 2.0, 2048.0, 4096.0),
    ];
    let result = display_csv_results(&results);
    assert!(result.is_ok());
}

#[test]
fn test_display_csv_results_mixed_memory() {
    // One result has memory, one does not -- has_memory should be true
    let mut results = vec![
        make_bench_result("no_mem.sh", 10.0, 1.0, 10),
        make_bench_result_with_memory("with_mem.sh", 20.0, 2.0, 512.0, 1024.0),
    ];
    let result = display_csv_results(&results);
    assert!(result.is_ok());

    // Also test the reverse order
    results.reverse();
    let result2 = display_csv_results(&results);
    assert!(result2.is_ok());
}

#[test]
fn test_display_csv_results_empty() {
    let results: Vec<BenchmarkResult> = vec![];
    // Empty results should still work (just prints header)
    let result = display_csv_results(&results);
    assert!(result.is_ok());
}

#[test]
fn test_display_csv_results_single_result_with_memory() {
    let results =
        vec![make_bench_result_with_memory("single.sh", 42.0, 3.0, 768.0, 1536.0)];
    let result = display_csv_results(&results);
    assert!(result.is_ok());
}

// =============================================================================
// display_comparison_results tests
// =============================================================================

#[test]
fn test_display_comparison_results_two_scripts() {
    let results = vec![
        make_bench_result("baseline.sh", 100.0, 10.0, 10),
        make_bench_result("optimized.sh", 50.0, 5.0, 10),
    ];
    let result = display_comparison_results(&results);
    assert!(result.is_ok());
}

#[test]
fn test_display_comparison_results_three_scripts() {
    let results = vec![
        make_bench_result("slow.sh", 200.0, 20.0, 10),
        make_bench_result("medium.sh", 100.0, 10.0, 10),
        make_bench_result("fast.sh", 50.0, 5.0, 10),
    ];
    let result = display_comparison_results(&results);
    assert!(result.is_ok());
}

#[test]
fn test_display_comparison_results_with_memory() {
    let results = vec![
        make_bench_result_with_memory("baseline.sh", 100.0, 10.0, 4096.0, 8192.0),
        make_bench_result_with_memory("optimized.sh", 50.0, 5.0, 2048.0, 4096.0),
    ];
    let result = display_comparison_results(&results);
    assert!(result.is_ok());
}

#[test]
fn test_display_comparison_results_identical_performance() {
    let results = vec![
        make_bench_result("script_a.sh", 100.0, 10.0, 10),
        make_bench_result("script_b.sh", 100.0, 10.0, 10),
    ];
    let result = display_comparison_results(&results);
    assert!(result.is_ok());
}

#[test]
fn test_display_comparison_results_long_script_names() {
    let results = vec![
        make_bench_result(
            "/very/long/path/to/some/deeply/nested/script_baseline.sh",
            100.0,
            10.0,
            10,
        ),
        make_bench_result(
            "/very/long/path/to/some/deeply/nested/script_optimized.sh",
            50.0,
            5.0,
            10,
        ),
    ];
    let result = display_comparison_results(&results);
    assert!(result.is_ok());
}

#[test]
fn test_display_comparison_results_mixed_memory() {
    // One has memory, one does not
    let r1 = make_bench_result("no_mem.sh", 100.0, 10.0, 10);
    let r2 = make_bench_result_with_memory("with_mem.sh", 50.0, 5.0, 2048.0, 4096.0);
    // has_memory will be true since at least one result has memory
    let results = vec![r1, r2];
    let result = display_comparison_results(&results);
    assert!(result.is_ok());
}

// =============================================================================
// display_results tests
// =============================================================================

#[test]
fn test_display_results_single_script_no_raw() {
    let results = vec![make_bench_result("test.sh", 25.0, 3.0, 10)];
    let env = make_environment();
    let options = BenchOptions {
        scripts: vec![PathBuf::from("test.sh")],
        warmup: 3,
        iterations: 10,
        output: None,
        strict: false,
        verify_determinism: false,
        show_raw: false,
        quiet: false,
        measure_memory: false,
        csv: false,
        no_color: false,
    };
    let result = display_results(&results, &env, &options);
    assert!(result.is_ok());
}

#[test]
fn test_display_results_single_script_with_raw() {
    let results = vec![make_bench_result("test.sh", 25.0, 3.0, 10)];
    let env = make_environment();
    let options = BenchOptions {
        scripts: vec![PathBuf::from("test.sh")],
        warmup: 3,
        iterations: 10,
        output: None,
        strict: false,
        verify_determinism: false,
        show_raw: true,
        quiet: false,
        measure_memory: false,
        csv: false,
        no_color: false,
    };
    let result = display_results(&results, &env, &options);
    assert!(result.is_ok());
}

#[test]
fn test_display_results_single_script_with_memory() {
    let results = vec![make_bench_result_with_memory(
        "mem_test.sh",
        25.0,
        3.0,
        1024.0,
        2048.0,
    )];
    let env = make_environment();
    let options = BenchOptions {
        scripts: vec![PathBuf::from("mem_test.sh")],
        warmup: 3,
        iterations: 10,
        output: None,
        strict: false,
        verify_determinism: false,
        show_raw: false,
        quiet: false,
        measure_memory: true,
        csv: false,
        no_color: false,
    };
    let result = display_results(&results, &env, &options);
    assert!(result.is_ok());
}

#[test]
fn test_display_results_multiple_scripts_triggers_comparison() {
    // When there are multiple results, display_results calls display_comparison_results
    let results = vec![
        make_bench_result("fast.sh", 10.0, 1.0, 10),
        make_bench_result("slow.sh", 50.0, 5.0, 10),
    ];
    let env = make_environment();
    let options = BenchOptions {
        scripts: vec![PathBuf::from("fast.sh"), PathBuf::from("slow.sh")],
        warmup: 3,
        iterations: 10,
        output: None,
        strict: false,
        verify_determinism: false,
        show_raw: false,
        quiet: false,
        measure_memory: false,
        csv: false,
        no_color: false,
    };
    let result = display_results(&results, &env, &options);
    assert!(result.is_ok());
}

#[test]
fn test_display_results_single_with_memory_and_raw() {
    let results = vec![make_bench_result_with_memory(
        "full.sh",
        30.0,
        4.0,
        2048.0,
        4096.0,
    )];
    let env = make_environment();
    let options = BenchOptions {
        scripts: vec![PathBuf::from("full.sh")],
        warmup: 5,
        iterations: 20,
        output: None,
        strict: false,
        verify_determinism: false,
        show_raw: true,
        quiet: false,
        measure_memory: true,
        csv: false,
        no_color: false,
    };
    let result = display_results(&results, &env, &options);
    assert!(result.is_ok());
}

#[test]
fn test_display_results_multiple_with_memory() {
    let results = vec![
        make_bench_result_with_memory("script_a.sh", 10.0, 1.0, 512.0, 1024.0),
        make_bench_result_with_memory("script_b.sh", 20.0, 2.0, 1024.0, 2048.0),
        make_bench_result_with_memory("script_c.sh", 30.0, 3.0, 2048.0, 4096.0),
    ];
    let env = make_environment();
    let options = BenchOptions {
        scripts: vec![
            PathBuf::from("script_a.sh"),
            PathBuf::from("script_b.sh"),
            PathBuf::from("script_c.sh"),
        ],
        warmup: 3,
        iterations: 10,
        output: None,
        strict: false,
        verify_determinism: false,
        show_raw: false,
        quiet: false,
        measure_memory: true,
        csv: false,
        no_color: false,
    };
    let result = display_results(&results, &env, &options);
    assert!(result.is_ok());
}

// =============================================================================
// Edge case and integration tests
// =============================================================================

#[test]
fn test_display_csv_results_zero_mean_baseline() {
    // Edge case: all results have zero mean (baseline_mean = 0)
    let results = vec![
        make_bench_result("zero.sh", 0.0, 0.0, 10),
        make_bench_result("also_zero.sh", 0.0, 0.0, 10),
    ];
    let result = display_csv_results(&results);
    assert!(result.is_ok());
}

#[test]
fn test_display_comparison_results_very_large_speedup() {
    let results = vec![
        make_bench_result("slow.sh", 10000.0, 100.0, 10),
        make_bench_result("fast.sh", 1.0, 0.1, 10),
    ];
    let result = display_comparison_results(&results);
    assert!(result.is_ok());
}

#[test]
fn test_display_comparison_results_nearly_equal() {
    let results = vec![
        make_bench_result("a.sh", 100.001, 0.001, 10),
        make_bench_result("b.sh", 100.002, 0.001, 10),
    ];
    let result = display_comparison_results(&results);
    assert!(result.is_ok());
}

#[test]
fn test_display_csv_results_special_characters_in_script_name() {
    let results = vec![make_bench_result("path/with spaces/script.sh", 10.0, 1.0, 5)];
    let result = display_csv_results(&results);
    assert!(result.is_ok());
}

#[test]
fn test_display_results_environment_info_displayed() {
    // Verify that environment info section is reached for single script
    let results = vec![make_bench_result("env_test.sh", 15.0, 2.0, 10)];
    let env = Environment {
        cpu: "AMD Ryzen 9 7950X".to_string(),
        ram: "64GB".to_string(),
        os: "Ubuntu 24.04".to_string(),
        hostname: "build-server".to_string(),
        bashrs_version: "6.48.0".to_string(),
    };
    let options = BenchOptions::new(vec![PathBuf::from("env_test.sh")]);
    let result = display_results(&results, &env, &options);
    assert!(result.is_ok());
}

// =============================================================================
// run_quality_gates tests (private fn, accessible via super::*)
// =============================================================================

#[test]
fn test_run_quality_gates_no_strict_no_determinism_returns_default_quality() {
    // When strict=false and verify_determinism=false, run_quality_gates
    // returns Quality { lint_passed: true, determinism_score: 1.0, output_identical: true }
    // without reading any file.
    let options = BenchOptions {
        scripts: vec![PathBuf::from("dummy.sh")],
        warmup: 0,
        iterations: 1,
        output: None,
        strict: false,
        verify_determinism: false,
        show_raw: false,
        quiet: true,
        measure_memory: false,
        csv: false,
        no_color: false,
    };
    // Pass a path that doesn't exist — should succeed because neither strict
    // nor verify_determinism is enabled, so no file read happens.
    let result = run_quality_gates(Path::new("/nonexistent/dummy.sh"), &options);
    assert!(result.is_ok(), "run_quality_gates with all disabled should succeed");
    let quality = result.unwrap();
    assert!(quality.lint_passed);
    assert!((quality.determinism_score - 1.0).abs() < f64::EPSILON);
    assert!(quality.output_identical);
}

#[test]
fn test_run_quality_gates_strict_mode_clean_script_passes() {
    use std::io::Write;
    // Create a temp file with a clean bash script (no lint violations)
    let mut tmpfile = tempfile::NamedTempFile::new().expect("create tmpfile");
    writeln!(tmpfile, "#!/bin/sh\necho hello").expect("write tmpfile");
    let path = tmpfile.path().to_path_buf();

    let options = BenchOptions {
        scripts: vec![path.clone()],
        warmup: 0,
        iterations: 1,
        output: None,
        strict: true,
        verify_determinism: false,
        show_raw: false,
        quiet: true,
        measure_memory: false,
        csv: false,
        no_color: false,
    };
    let result = run_quality_gates(&path, &options);
    assert!(result.is_ok(), "strict mode with clean script should pass: {:?}", result);
    assert!(result.unwrap().lint_passed);
}

#[test]
fn test_run_quality_gates_strict_mode_missing_file_fails() {
    let options = BenchOptions {
        scripts: vec![PathBuf::from("/nonexistent/script.sh")],
        warmup: 0,
        iterations: 1,
        output: None,
        strict: true,
        verify_determinism: false,
        show_raw: false,
        quiet: true,
        measure_memory: false,
        csv: false,
        no_color: false,
    };
    let result = run_quality_gates(Path::new("/nonexistent/script.sh"), &options);
    assert!(result.is_err(), "strict mode with missing file should fail");
}

// =============================================================================
// BenchOptions::new tests
// =============================================================================

#[test]
fn test_bench_options_new_defaults() {
    let scripts = vec![PathBuf::from("test.sh")];
    let options = BenchOptions::new(scripts.clone());
    assert_eq!(options.scripts, scripts);
    assert!(!options.strict);
    assert!(!options.verify_determinism);
    assert!(!options.show_raw);
    assert!(!options.quiet);
    assert!(!options.measure_memory);
    assert!(!options.csv);
    assert!(!options.no_color);
    assert!(options.output.is_none());
}

// =============================================================================
// Quality and Statistics struct field tests
// =============================================================================

#[test]
fn test_quality_struct_fields() {
    let q = Quality {
        lint_passed: false,
        determinism_score: 0.5,
        output_identical: false,
    };
    assert!(!q.lint_passed);
    assert!((q.determinism_score - 0.5).abs() < f64::EPSILON);
    assert!(!q.output_identical);
}

#[test]
fn test_statistics_outlier_indices() {
    let stats = Statistics {
        mean_ms: 10.0,
        median_ms: 9.5,
        stddev_ms: 1.0,
        min_ms: 8.0,
        max_ms: 15.0,
        variance_ms: 1.0,
        mad_ms: 0.7,
        geometric_mean_ms: 9.8,
        harmonic_mean_ms: 9.6,
        outlier_indices: vec![2, 5],
        memory: None,
    };
    assert_eq!(stats.outlier_indices.len(), 2);
    assert_eq!(stats.outlier_indices[0], 2);
    assert_eq!(stats.outlier_indices[1], 5);
    assert!(stats.memory.is_none());
}

#[test]
fn test_memory_statistics_fields() {
    let mem = MemoryStatistics {
        mean_kb: 1024.0,
        median_kb: 1000.0,
        min_kb: 900.0,
        max_kb: 1200.0,
        peak_kb: 1200.0,
    };
    assert!((mem.mean_kb - 1024.0).abs() < f64::EPSILON);
    assert!((mem.peak_kb - 1200.0).abs() < f64::EPSILON);
}
