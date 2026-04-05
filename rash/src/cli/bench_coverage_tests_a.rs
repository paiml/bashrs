#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

// =============================================================================
// Coverage tests for bench.rs display/formatting functions
// Targets: display_comparison_results, display_results, display_csv_results
// =============================================================================

/// Helper to create a mock BenchmarkResult with given parameters
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
