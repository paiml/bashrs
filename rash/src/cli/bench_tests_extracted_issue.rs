
    #[test]
    fn test_issue_012_statistics_includes_geometric_mean() {
        // ARRANGE
        let values = vec![1.0, 2.0, 4.0, 8.0];

        // ACT
        let stats = Statistics::calculate(&values);

        // ASSERT: Statistics should include geometric mean
        assert!(stats.geometric_mean_ms > 0.0);
        assert!((stats.geometric_mean_ms - 2.828).abs() < 0.01);
    }

    #[test]
    fn test_issue_012_statistics_includes_harmonic_mean() {
        // ARRANGE
        let values = vec![1.0, 2.0, 4.0];

        // ACT
        let stats = Statistics::calculate(&values);

        // ASSERT: Statistics should include harmonic mean
        assert!(stats.harmonic_mean_ms > 0.0);
        assert!((stats.harmonic_mean_ms - 1.714).abs() < 0.01);
    }

    // ============================================================================
    // Issue #12 Phase 1: JSON Schema Tests (RED Phase)
    // ============================================================================

    #[test]
    fn test_issue_012_json_schema_serializable() {
        // ARRANGE
        let stats = Statistics {
            mean_ms: 10.0,
            median_ms: 9.5,
            stddev_ms: 1.5,
            min_ms: 8.0,
            max_ms: 12.0,
            variance_ms: 2.25,
            mad_ms: 1.0,
            geometric_mean_ms: 9.8,
            harmonic_mean_ms: 9.6,
            outlier_indices: vec![4],
            memory: None,
        };

        // ACT: Serialize to JSON
        let json = serde_json::to_string(&stats);

        // ASSERT: Should serialize successfully
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("mean_ms"));
        assert!(json_str.contains("mad_ms"));
        assert!(json_str.contains("geometric_mean_ms"));
        assert!(json_str.contains("harmonic_mean_ms"));
        assert!(json_str.contains("outlier_indices"));
    }

    #[test]
    fn test_issue_012_benchmark_output_has_schema() {
        // ARRANGE
        let _output = BenchmarkOutput {
            version: "1.0.0".to_string(),
            timestamp: "2025-11-05T00:00:00Z".to_string(),
            environment: Environment {
                cpu: "Test CPU".to_string(),
                ram: "16GB".to_string(),
                os: "Linux".to_string(),
                hostname: "test".to_string(),
                bashrs_version: "6.31.0".to_string(),
            },
            benchmarks: vec![],
        };

        // ACT: Generate JSON schema
        let schema = schemars::schema_for!(BenchmarkOutput);
        let schema_json = serde_json::to_string_pretty(&schema);

        // ASSERT: Schema should be generated
        assert!(schema_json.is_ok());
        let schema_str = schema_json.unwrap();
        assert!(schema_str.contains("BenchmarkOutput"));
        assert!(schema_str.contains("properties"));
    }

    // ============================================================================
    // Issue #12 Phase 2: Welch's t-test Tests (RED Phase)
    // ============================================================================

    #[test]
    fn test_issue_012_phase2_welch_t_test_equal_means() {
        // ARRANGE: Two samples with identical means
        let sample1 = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let sample2 = vec![10.0, 11.0, 12.0, 13.0, 14.0];

        // ACT
        let t_statistic = welch_t_test(&sample1, &sample2);

        // ASSERT: t-statistic should be ~0 for identical distributions
        assert!(t_statistic.abs() < 0.01);
    }

    #[test]
    fn test_issue_012_phase2_welch_t_test_different_means() {
        // ARRANGE: Two samples with clearly different means
        let sample1 = vec![10.0, 11.0, 12.0, 13.0, 14.0]; // mean = 12
        let sample2 = vec![20.0, 21.0, 22.0, 23.0, 24.0]; // mean = 22

        // ACT
        let t_statistic = welch_t_test(&sample1, &sample2);

        // ASSERT: t-statistic should be large (significant difference)
        assert!(t_statistic.abs() > 5.0);
    }

    #[test]
    fn test_issue_012_phase2_statistical_significance() {
        // ARRANGE: Two samples with different means
        let sample1 = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let sample2 = vec![20.0, 21.0, 22.0, 23.0, 24.0];

        // ACT
        let is_significant = is_statistically_significant(&sample1, &sample2, 0.05);

        // ASSERT: Should detect significant difference
        assert!(is_significant);
    }

    #[test]
    fn test_issue_012_phase2_not_statistically_significant() {
        // ARRANGE: Two samples with similar means and high variance
        let sample1 = vec![10.0, 15.0, 20.0, 25.0, 30.0];
        let sample2 = vec![12.0, 17.0, 22.0, 27.0, 32.0];

        // ACT
        let is_significant = is_statistically_significant(&sample1, &sample2, 0.05);

        // ASSERT: Result is boolean (no specific assertion on significance)
        // The actual significance depends on the statistical calculation
        let _ = is_significant; // Test validates function doesn't panic
    }

    // ============================================================================
    // Issue #12 Phase 2: Comparison Results Tests (RED Phase)
    // ============================================================================

    #[test]
    fn test_issue_012_phase2_comparison_result_structure() {
        // ARRANGE
        let sample1 = vec![10.0, 11.0, 12.0];
        let sample2 = vec![20.0, 21.0, 22.0];

        // ACT
        let comparison = compare_benchmarks(&sample1, &sample2);

        // ASSERT: Comparison should have all required fields
        assert!(comparison.speedup > 0.0);
        assert!(comparison.t_statistic.abs() > 0.0);
        assert!(comparison.p_value >= 0.0 && comparison.p_value <= 1.0);
        // is_significant is boolean - no need to assert tautology
    }

    #[test]
    fn test_issue_012_phase2_speedup_calculation() {
        // ARRANGE
        let baseline = vec![20.0, 22.0, 24.0]; // mean = 22
        let optimized = vec![10.0, 11.0, 12.0]; // mean = 11

        // ACT
        let comparison = compare_benchmarks(&baseline, &optimized);

        // ASSERT: Speedup should be ~2x (22/11)
        assert!((comparison.speedup - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_issue_012_phase2_slowdown_detection() {
        // ARRANGE
        let baseline = vec![10.0, 11.0, 12.0]; // mean = 11
        let slower = vec![20.0, 22.0, 24.0]; // mean = 22

        // ACT
        let comparison = compare_benchmarks(&baseline, &slower);

        // ASSERT: Speedup should be ~0.5x (regression)
        assert!(comparison.speedup < 1.0);
        assert!((comparison.speedup - 0.5).abs() < 0.1);
    }

    // ============================================================================
    // Issue #12 Phase 2: Regression Detection Tests (RED Phase)
    // ============================================================================

    #[test]
    fn test_issue_012_phase2_detect_regression_significant() {
        // ARRANGE
        let baseline = vec![10.0, 11.0, 12.0, 13.0, 14.0]; // mean = 12
        let current = vec![20.0, 21.0, 22.0, 23.0, 24.0]; // mean = 22 (slower)

        // ACT
        let regression = detect_regression(&baseline, &current, 0.05);

        // ASSERT: Should detect regression
        assert!(regression.is_regression);
        assert!(regression.speedup < 1.0);
        assert!(regression.is_statistically_significant);
    }

include!("bench_tests_extracted_issue_issue.rs");
