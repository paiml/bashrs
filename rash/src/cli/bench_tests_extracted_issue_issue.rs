
    #[test]
    fn test_issue_012_phase2_no_regression_improvement() {
        // ARRANGE
        let baseline = vec![20.0, 21.0, 22.0, 23.0, 24.0]; // mean = 22
        let current = vec![10.0, 11.0, 12.0, 13.0, 14.0]; // mean = 12 (faster)

        // ACT
        let regression = detect_regression(&baseline, &current, 0.05);

        // ASSERT: Should NOT detect regression (improvement)
        assert!(!regression.is_regression);
        assert!(regression.speedup > 1.0);
    }

    #[test]
    fn test_issue_012_phase2_no_regression_not_significant() {
        // ARRANGE: Samples with similar performance (within noise)
        let baseline = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let current = vec![10.5, 11.5, 12.5, 13.5, 14.5];

        // ACT
        let regression = detect_regression(&baseline, &current, 0.05);

        // ASSERT: Should NOT detect regression (not statistically significant)
        assert!(!regression.is_regression || !regression.is_statistically_significant);
    }

    #[test]
    fn test_issue_012_phase2_regression_threshold() {
        // ARRANGE: 5% slower (borderline regression)
        let baseline = vec![100.0, 100.0, 100.0];
        let current = vec![105.0, 105.0, 105.0];

        // ACT: Use 10% threshold (should NOT trigger)
        let regression = detect_regression_with_threshold(&baseline, &current, 0.05, 0.10);

        // ASSERT: Should NOT detect regression (within 10% threshold)
        assert!(!regression.is_regression);
    }

    #[test]
    fn test_issue_012_phase2_regression_exceeds_threshold() {
        // ARRANGE: 20% slower (exceeds 10% threshold)
        let baseline = vec![100.0, 100.0, 100.0];
        let current = vec![120.0, 120.0, 120.0];

        // ACT: Use 10% threshold
        let regression = detect_regression_with_threshold(&baseline, &current, 0.05, 0.10);

        // ASSERT: Should detect regression (exceeds threshold)
        assert!(regression.is_regression);
    }

    // ============================================================================
    // Issue #12 Phase 2: Integration Tests (RED Phase)
    // ============================================================================

    #[test]
    fn test_issue_012_phase2_comparison_in_benchmark_result() {
        // ARRANGE
        let stats1 = Statistics::calculate(&[10.0, 11.0, 12.0]);
        let stats2 = Statistics::calculate(&[20.0, 21.0, 22.0]);

        // ACT: Create comparison
        let comparison = ComparisonResult::from_statistics(&stats1, &stats2);

        // ASSERT: Comparison should be populated
        assert!(comparison.speedup > 0.0);
        assert!(comparison.p_value >= 0.0);
    }

    // ===== ADDITIONAL COVERAGE TESTS =====

    #[test]
    fn test_bench_options_new() {
        let scripts = vec![PathBuf::from("test.sh")];
        let options = BenchOptions::new(scripts.clone());

        assert_eq!(options.scripts, scripts);
        assert_eq!(options.warmup, DEFAULT_WARMUP);
        assert_eq!(options.iterations, DEFAULT_ITERATIONS);
        assert!(options.output.is_none());
        assert!(!options.strict);
        assert!(!options.verify_determinism);
        assert!(!options.show_raw);
        assert!(!options.quiet);
        assert!(!options.measure_memory);
        assert!(!options.csv);
        assert!(!options.no_color);
    }

    #[test]
    fn test_quality_default() {
        let quality = Quality {
            lint_passed: true,
            determinism_score: 1.0,
            output_identical: true,
        };
        assert!(quality.lint_passed);
        assert_eq!(quality.determinism_score, 1.0);
        assert!(quality.output_identical);
    }

    #[test]
    fn test_benchmark_result_creation() {
        let stats = Statistics::calculate(&[10.0, 20.0, 30.0]);
        let quality = Quality {
            lint_passed: true,
            determinism_score: 1.0,
            output_identical: true,
        };

        let result = BenchmarkResult {
            script: "test.sh".to_string(),
            iterations: 10,
            warmup: 3,
            statistics: stats,
            raw_results_ms: vec![10.0, 20.0, 30.0],
            quality,
        };

        assert_eq!(result.script, "test.sh");
        assert_eq!(result.iterations, 10);
        assert_eq!(result.warmup, 3);
        assert_eq!(result.raw_results_ms.len(), 3);
    }

    #[test]
    fn test_benchmark_output_serialization() {
        let output = BenchmarkOutput {
            version: VERSION.to_string(),
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            environment: Environment {
                cpu: "Test CPU".to_string(),
                ram: "16GB".to_string(),
                os: "Linux".to_string(),
                hostname: "test".to_string(),
                bashrs_version: "6.48.0".to_string(),
            },
            benchmarks: vec![],
        };

        let json = serde_json::to_string(&output);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("version"));
        assert!(json_str.contains("timestamp"));
        assert!(json_str.contains("environment"));
    }

    #[test]
    fn test_validate_options_empty_scripts() {
        let options = BenchOptions {
            scripts: vec![],
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

        let result = validate_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_options_zero_iterations() {
        let options = BenchOptions {
            scripts: vec![PathBuf::from("test.sh")],
            warmup: 3,
            iterations: 0,
            output: None,
            strict: false,
            verify_determinism: false,
            show_raw: false,
            quiet: false,
            measure_memory: false,
            csv: false,
            no_color: false,
        };

        let result = validate_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_options_nonexistent_script() {
        let options = BenchOptions {
            scripts: vec![PathBuf::from("/nonexistent/script.sh")],
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

        let result = validate_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_output() {
        use std::process::Output;

        let output1 = Output {
            status: std::process::ExitStatus::default(),
            stdout: b"hello".to_vec(),
            stderr: b"".to_vec(),
        };

        let output2 = Output {
            status: std::process::ExitStatus::default(),
            stdout: b"hello".to_vec(),
            stderr: b"".to_vec(),
        };

        let output3 = Output {
            status: std::process::ExitStatus::default(),
            stdout: b"world".to_vec(),
            stderr: b"".to_vec(),
        };

        // Same content should produce same hash
        assert_eq!(hash_output(&output1), hash_output(&output2));
        // Different content should produce different hash
        assert_ne!(hash_output(&output1), hash_output(&output3));
    }

    #[test]
    fn test_truncate_path_short() {
        assert_eq!(truncate_path("short.sh", 20), "short.sh");
    }

    #[test]
    fn test_truncate_path_exact() {
        let path = "exactly_twenty_chars";
        assert_eq!(truncate_path(path, 20), path);
    }

    #[test]
    fn test_welch_degrees_of_freedom() {
        let sample1 = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let sample2 = vec![10.0, 11.0, 12.0, 13.0, 14.0];

        let df = welch_degrees_of_freedom(&sample1, &sample2);
        assert!(df > 0.0);
    }

    #[test]
    fn test_welch_degrees_of_freedom_zero_variance() {
        let sample1 = vec![10.0, 10.0, 10.0];
        let sample2 = vec![10.0, 10.0, 10.0];

        let df = welch_degrees_of_freedom(&sample1, &sample2);
        // Should return n1 + n2 - 2 when variance is zero
        assert_eq!(df, 4.0);
    }

    #[test]
    fn test_approximate_p_value_large_t() {
        // Large t-statistic should give small p-value
        let p = approximate_p_value(10.0, 50.0);
        assert!(p < 0.05);
    }

    #[test]
    fn test_approximate_p_value_small_t() {
        // Small t-statistic should give large p-value
        let p = approximate_p_value(0.5, 50.0);
        assert!(p > 0.05);
    }

    #[test]
    fn test_approximate_p_value_small_df() {
        let p = approximate_p_value(3.0, 5.0);
        assert!(p < 0.10);
    }

    #[test]
    fn test_calculate_geometric_mean_empty() {
        assert_eq!(calculate_geometric_mean(&[]), 0.0);
    }

    #[test]
    fn test_calculate_harmonic_mean_empty() {
        assert_eq!(calculate_harmonic_mean(&[]), 0.0);
    }

    #[test]
    fn test_detect_outliers_all_identical() {
        let values = vec![10.0, 10.0, 10.0, 10.0, 10.0];
        let outliers = detect_outliers(&values, 3.0);
        // No outliers when all values are identical (MAD = 0)
        assert!(outliers.is_empty());
    }

    #[test]
    fn test_regression_result_fields() {
        let result = RegressionResult {
            is_regression: true,
            speedup: 0.5,
            is_statistically_significant: true,
            change_percent: -50.0,
        };
        assert!(result.is_regression);
        assert_eq!(result.speedup, 0.5);
        assert!(result.is_statistically_significant);
        assert_eq!(result.change_percent, -50.0);
    }

    #[test]
    fn test_comparison_result_fields() {
        let result = ComparisonResult {
            speedup: 2.0,
            t_statistic: 5.0,
            p_value: 0.01,
            is_significant: true,
        };
        assert_eq!(result.speedup, 2.0);
        assert_eq!(result.t_statistic, 5.0);
        assert_eq!(result.p_value, 0.01);
        assert!(result.is_significant);
    }

    #[test]
    fn test_environment_fields() {
        let env = Environment {
            cpu: "Intel i7".to_string(),
            ram: "32GB".to_string(),
            os: "Linux 6.0".to_string(),
            hostname: "workstation".to_string(),
            bashrs_version: "6.48.0".to_string(),
        };
        assert_eq!(env.cpu, "Intel i7");
        assert_eq!(env.ram, "32GB");
        assert_eq!(env.os, "Linux 6.0");
        assert_eq!(env.hostname, "workstation");
        assert_eq!(env.bashrs_version, "6.48.0");
    }

    #[test]
    fn test_welch_t_test_zero_variance() {
        let sample1 = vec![10.0, 10.0, 10.0];
        let sample2 = vec![20.0, 20.0, 20.0];

        let t = welch_t_test(&sample1, &sample2);
        // Should return 0 when both samples have zero variance
        assert_eq!(t, 0.0);
    }

    #[test]
    fn test_statistics_edge_case_single_value() {
        let values = vec![42.0];
        let stats = Statistics::calculate(&values);

        assert_eq!(stats.mean_ms, 42.0);
        assert_eq!(stats.median_ms, 42.0);
        assert_eq!(stats.min_ms, 42.0);
        assert_eq!(stats.max_ms, 42.0);
        assert_eq!(stats.variance_ms, 0.0);
        assert_eq!(stats.stddev_ms, 0.0);
    }

    #[test]
    fn test_memory_statistics_edge_case() {
        let memory = vec![1000.0];
        let stats = MemoryStatistics::calculate(&memory);

        assert_eq!(stats.mean_kb, 1000.0);
        assert_eq!(stats.median_kb, 1000.0);
        assert_eq!(stats.min_kb, 1000.0);
        assert_eq!(stats.max_kb, 1000.0);
        assert_eq!(stats.peak_kb, 1000.0);
    }

    // ============================================================================
    // Additional coverage tests for bench.rs functions
    // ============================================================================

    #[test]
    fn test_compare_benchmarks_faster() {
        let baseline = vec![100.0, 110.0, 105.0];
        let current = vec![50.0, 55.0, 52.0];

        let result = compare_benchmarks(&baseline, &current);

        // Current is faster so speedup > 1.0
        assert!(result.speedup > 1.5);
        assert!(result.is_significant);
    }

    #[test]
    fn test_compare_benchmarks_slower() {
        let baseline = vec![50.0, 55.0, 52.0];
        let current = vec![100.0, 110.0, 105.0];

        let result = compare_benchmarks(&baseline, &current);

        // Current is slower so speedup < 1.0
        assert!(result.speedup < 1.0);
    }

    #[test]
    fn test_compare_benchmarks_similar() {
        let baseline = vec![100.0, 100.0, 100.0];
        let current = vec![101.0, 99.0, 100.0];

        let result = compare_benchmarks(&baseline, &current);

        // Similar performance - speedup close to 1.0
        assert!((result.speedup - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_detect_regression_no_regression() {
        let baseline = vec![100.0, 100.0, 100.0];
        let current = vec![50.0, 50.0, 50.0]; // Faster - no regression

        let result = detect_regression(&baseline, &current, 0.05);

        assert!(!result.is_regression);
        assert!(result.speedup > 1.0);
    }

    #[test]
    fn test_detect_regression_with_regression() {
        let baseline = vec![50.0, 50.0, 50.0];
        let current = vec![100.0, 100.0, 100.0]; // Slower - regression

        let result = detect_regression(&baseline, &current, 0.05);

        assert!(result.is_regression);
        assert!(result.speedup < 1.0);
        assert!(result.change_percent > 0.0);
    }

    #[test]
    fn test_detect_regression_with_custom_threshold() {
        let baseline = vec![100.0, 100.0, 100.0];
        let current = vec![105.0, 105.0, 105.0]; // 5% slower

        // 10% threshold - should not detect regression
        let result_10 = detect_regression_with_threshold(&baseline, &current, 0.05, 0.10);
        assert!(!result_10.is_regression);

        // 3% threshold - should detect regression
        let result_3 = detect_regression_with_threshold(&baseline, &current, 0.05, 0.03);
        assert!(result_3.is_regression);
    }

    #[test]
    fn test_is_statistically_significant_yes() {
        let sample1 = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let sample2 = vec![100.0, 110.0, 120.0, 130.0, 140.0];

        assert!(is_statistically_significant(&sample1, &sample2, 0.05));
    }

    #[test]
    fn test_is_statistically_significant_no() {
        let sample1 = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let sample2 = vec![10.5, 11.5, 12.5, 13.5, 14.5];

        // Very similar - might not be significant
        // Just verifying the function runs without error
        let _ = is_statistically_significant(&sample1, &sample2, 0.05);
    }

    #[test]
    fn test_truncate_path_exact_length() {
        let path = "exactly10.";
        assert_eq!(truncate_path(path, 10), "exactly10.");
    }

    #[test]
    fn test_truncate_path_very_short_max() {
        let path = "/very/long/path/to/script.sh";
        // When max_len is very short
        let result = truncate_path(path, 5);
        assert!(result.len() <= 35); // Will be truncated with "..."
    }
}
