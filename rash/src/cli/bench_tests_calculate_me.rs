#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_mean() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_mean(&values), 3.0);
    }

    #[test]
    fn test_calculate_median_odd() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_median(&values), 3.0);
    }

    #[test]
    fn test_calculate_median_even() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(calculate_median(&values), 2.5);
    }

    #[test]
    fn test_calculate_variance() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mean = 3.0;
        assert_eq!(calculate_variance(&values, mean), 2.0);
    }

    #[test]
    fn test_statistics_calculate() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = Statistics::calculate(&values);
        assert_eq!(stats.mean_ms, 3.0);
        assert_eq!(stats.median_ms, 3.0);
        assert_eq!(stats.min_ms, 1.0);
        assert_eq!(stats.max_ms, 5.0);
    }

    #[test]
    fn test_truncate_path() {
        let path = "/very/long/path/to/some/script.sh";
        // max_len=20 -> "..." (3) + last 17 chars = "...to/some/script.sh" (20 total)
        assert_eq!(truncate_path(path, 20), "...to/some/script.sh");
        assert_eq!(truncate_path("short.sh", 20), "short.sh");
    }

    #[test]
    fn test_memory_statistics_calculate() {
        let memory_kb = vec![1024.0, 2048.0, 1536.0, 2048.0, 1024.0];
        let stats = MemoryStatistics::calculate(&memory_kb);
        assert_eq!(stats.mean_kb, 1536.0);
        assert_eq!(stats.median_kb, 1536.0);
        assert_eq!(stats.min_kb, 1024.0);
        assert_eq!(stats.max_kb, 2048.0);
        assert_eq!(stats.peak_kb, 2048.0);
    }

    #[test]
    fn test_statistics_with_memory() {
        let time_results = vec![10.0, 20.0, 15.0];
        let memory_results = vec![1024.0, 2048.0, 1536.0];
        let stats = Statistics::calculate_with_memory(&time_results, Some(&memory_results));

        assert_eq!(stats.mean_ms, 15.0);
        assert!(stats.memory.is_some());

        let mem = stats.memory.unwrap();
        assert_eq!(mem.mean_kb, 1536.0);
        assert_eq!(mem.median_kb, 1536.0);
    }

    #[test]
    fn test_statistics_without_memory() {
        let time_results = vec![10.0, 20.0, 15.0];
        let stats = Statistics::calculate(&time_results);

        assert_eq!(stats.mean_ms, 15.0);
        assert!(stats.memory.is_none());
    }

    // ============================================================================
    // Issue #12 Phase 1: MAD-based Outlier Detection Tests (RED Phase)
    // ============================================================================

    #[test]
    fn test_issue_012_mad_calculation() {
        // ARRANGE: Dataset with known MAD
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        // ACT
        let mad = calculate_mad(&values);

        // ASSERT: MAD = median(|xi - median(x)|)
        // median = 3.0, deviations = [2, 1, 0, 1, 2], MAD = 1.0
        assert_eq!(mad, 1.0);
    }

    #[test]
    fn test_issue_012_mad_with_outlier() {
        // ARRANGE: Dataset with outlier
        let values = vec![1.0, 2.0, 3.0, 4.0, 100.0]; // 100.0 is outlier

        // ACT
        let mad = calculate_mad(&values);

        // ASSERT: MAD should be robust to outlier
        // median = 3.0, deviations = [2, 1, 0, 1, 97], MAD = 1.0
        assert_eq!(mad, 1.0);
    }

    #[test]
    fn test_issue_012_detect_outliers_none() {
        // ARRANGE: Normal distribution, no outliers
        let values = vec![10.0, 11.0, 12.0, 13.0, 14.0];

        // ACT: Detect outliers with 3.0 threshold (standard)
        let outliers = detect_outliers(&values, 3.0);

        // ASSERT: No outliers
        assert!(outliers.is_empty());
    }

    #[test]
    fn test_issue_012_detect_outliers_single() {
        // ARRANGE: Dataset with one clear outlier
        let values = vec![10.0, 11.0, 12.0, 13.0, 100.0];

        // ACT
        let outliers = detect_outliers(&values, 3.0);

        // ASSERT: Index 4 (100.0) is outlier
        assert_eq!(outliers.len(), 1);
        assert_eq!(outliers[0], 4);
    }

    #[test]
    fn test_issue_012_detect_outliers_multiple() {
        // ARRANGE: Dataset with multiple outliers
        let values = vec![10.0, 11.0, 12.0, 100.0, 200.0];

        // ACT
        let outliers = detect_outliers(&values, 3.0);

        // ASSERT: Indices 3 and 4 are outliers
        assert_eq!(outliers.len(), 2);
        assert!(outliers.contains(&3));
        assert!(outliers.contains(&4));
    }

    #[test]
    fn test_issue_012_statistics_includes_mad() {
        // ARRANGE
        let values = vec![10.0, 11.0, 12.0, 13.0, 14.0];

        // ACT
        let stats = Statistics::calculate(&values);

        // ASSERT: Statistics should include MAD
        assert!(stats.mad_ms > 0.0);
    }

    #[test]
    fn test_issue_012_statistics_includes_outliers() {
        // ARRANGE: Dataset with outlier
        let values = vec![10.0, 11.0, 12.0, 13.0, 100.0];

        // ACT
        let stats = Statistics::calculate(&values);

        // ASSERT: Outliers should be detected
        assert_eq!(stats.outlier_indices.len(), 1);
        assert_eq!(stats.outlier_indices[0], 4);
    }

    // ============================================================================
    // Issue #12 Phase 1: Geometric & Harmonic Mean Tests (RED Phase)
    // ============================================================================

    #[test]
    fn test_issue_012_geometric_mean() {
        // ARRANGE
        let values = vec![1.0, 2.0, 4.0, 8.0];

        // ACT
        let geo_mean = calculate_geometric_mean(&values);

        // ASSERT: (1 * 2 * 4 * 8)^(1/4) = 2.828...
        assert!((geo_mean - 2.828).abs() < 0.01);
    }

    #[test]
    fn test_issue_012_harmonic_mean() {
        // ARRANGE
        let values = vec![1.0, 2.0, 4.0];

        // ACT
        let harm_mean = calculate_harmonic_mean(&values);

        // ASSERT: 3 / (1/1 + 1/2 + 1/4) = 3 / 1.75 = 1.714...
        assert!((harm_mean - 1.714).abs() < 0.01);
    }

}

include!("bench_tests_extracted_issue.rs");
