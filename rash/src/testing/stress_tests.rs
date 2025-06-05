//! Tests for the stress testing module

use super::stress::*;
use crate::models::{Config, ShellDialect, VerificationLevel};

fn get_test_config() -> Config {
    Config {
        target: ShellDialect::Posix,
        verify: VerificationLevel::Basic,
        emit_proof: false,
        optimize: true,
        strict_mode: false,
        validation_level: None,
    }
}

#[test]
fn test_stress_tester_new() {
    let config = get_test_config();
    let tester = StressTester::new(config);

    // This would be a real stress test, but we'll make it quick for CI
    let result = tester.run_stress_tests();
    assert!(result.is_ok());
}

#[test]
fn test_stress_test_results_success_rate() {
    let results = StressTestResults {
        total_operations: 100,
        successful_operations: 90,
        failed_operations: 10,
        average_latency_ms: 10.5,
        max_latency_ms: 25.0,
        min_latency_ms: 5.0,
        memory_usage_mb: 45.2,
        concurrent_threads: 4,
        test_duration_secs: 30.0,
        operations_per_second: 3.33,
        error_details: vec!["Error 1".to_string(), "Error 2".to_string()],
    };

    assert_eq!(results.success_rate(), 90.0);
}

#[test]
fn test_stress_test_results_zero_operations() {
    let results = StressTestResults {
        total_operations: 0,
        successful_operations: 0,
        failed_operations: 0,
        average_latency_ms: 0.0,
        max_latency_ms: 0.0,
        min_latency_ms: 0.0,
        memory_usage_mb: 0.0,
        concurrent_threads: 0,
        test_duration_secs: 0.0,
        operations_per_second: 0.0,
        error_details: vec![],
    };

    assert_eq!(results.success_rate(), 0.0);
}

#[test]
fn test_stress_test_results_perfect_success() {
    let results = StressTestResults {
        total_operations: 50,
        successful_operations: 50,
        failed_operations: 0,
        average_latency_ms: 8.2,
        max_latency_ms: 15.0,
        min_latency_ms: 3.0,
        memory_usage_mb: 30.0,
        concurrent_threads: 8,
        test_duration_secs: 60.0,
        operations_per_second: 0.83,
        error_details: vec![],
    };

    assert_eq!(results.success_rate(), 100.0);
}

#[test]
fn test_stress_test_results_all_failures() {
    let results = StressTestResults {
        total_operations: 20,
        successful_operations: 0,
        failed_operations: 20,
        average_latency_ms: 100.0,
        max_latency_ms: 200.0,
        min_latency_ms: 50.0,
        memory_usage_mb: 80.0,
        concurrent_threads: 2,
        test_duration_secs: 45.0,
        operations_per_second: 0.44,
        error_details: vec!["Error".to_string(); 20],
    };

    assert_eq!(results.success_rate(), 0.0);
}

#[test]
fn test_stress_tester_with_different_configs() {
    let configs = vec![
        Config {
            target: ShellDialect::Posix,
            verify: VerificationLevel::Basic,
            emit_proof: false,
            optimize: true,
            strict_mode: false,
            validation_level: None,
        },
        Config {
            target: ShellDialect::Bash,
            verify: VerificationLevel::Strict,
            emit_proof: false,
            optimize: false,
            strict_mode: false,
            validation_level: None,
        },
        Config {
            target: ShellDialect::Dash,
            verify: VerificationLevel::None,
            emit_proof: true,
            optimize: true,
            strict_mode: false,
            validation_level: None,
        },
    ];

    for config in configs {
        let tester = StressTester::new(config);
        let result = tester.run_stress_tests();
        assert!(result.is_ok());

        if let Ok(results) = result {
            assert!(results.total_operations > 0);
            assert!(results.success_rate() >= 0.0 && results.success_rate() <= 100.0);
        }
    }
}

#[test]
fn test_stress_test_results_clone() {
    let original = StressTestResults {
        total_operations: 10,
        successful_operations: 8,
        failed_operations: 2,
        average_latency_ms: 12.5,
        max_latency_ms: 20.0,
        min_latency_ms: 8.0,
        memory_usage_mb: 25.0,
        concurrent_threads: 4,
        test_duration_secs: 15.0,
        operations_per_second: 0.67,
        error_details: vec!["Test error".to_string()],
    };

    let cloned = original.clone();

    assert_eq!(original.total_operations, cloned.total_operations);
    assert_eq!(original.successful_operations, cloned.successful_operations);
    assert_eq!(original.success_rate(), cloned.success_rate());
}

#[test]
fn test_stress_test_results_debug() {
    let results = StressTestResults {
        total_operations: 5,
        successful_operations: 4,
        failed_operations: 1,
        average_latency_ms: 7.5,
        max_latency_ms: 12.0,
        min_latency_ms: 5.0,
        memory_usage_mb: 20.0,
        concurrent_threads: 2,
        test_duration_secs: 10.0,
        operations_per_second: 0.5,
        error_details: vec!["Debug test error".to_string()],
    };

    let debug_string = format!("{results:?}");
    assert!(debug_string.contains("total_operations: 5"));
    assert!(debug_string.contains("successful_operations: 4"));
}
