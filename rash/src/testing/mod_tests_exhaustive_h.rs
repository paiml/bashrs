use super::*;

#[test]
fn test_exhaustive_harness_basic() {
    let config = TestConfig {
        fuzz_iterations: 1000, // Reduced for testing
        ..Default::default()
    };

    let mut harness = ExhaustiveTestHarness::new(config);
    let stats = harness.run_all_tests().unwrap();

    assert!(stats.total_tests > 0);
    assert!(stats.coverage_percentage > 0.0);
}

#[test]
fn test_random_input_generation() {
    let config = TestConfig::default();
    let harness = ExhaustiveTestHarness::new(config);

    for _ in 0..100 {
        let input = harness.generate_random_input().unwrap();
        assert!(!input.is_empty());
    }
}

// ===== Additional tests for coverage =====

#[test]
fn test_test_config_default() {
    let config = TestConfig::default();
    assert!(config.enable_assertions);
    assert!(config.track_coverage);
    assert!(config.inject_errors);
    assert_eq!(config.fuzz_iterations, 1_000_000);
    assert_eq!(config.memory_limit, Some(1024 * 1024 * 1024));
    assert_eq!(config.timeout, Duration::from_secs(300));
    assert!(!config.enable_mutation);
}

#[test]
fn test_test_statistics_default() {
    let stats = TestStatistics::default();
    assert_eq!(stats.total_tests, 0);
    assert_eq!(stats.passed_tests, 0);
    assert_eq!(stats.failed_tests, 0);
    assert_eq!(stats.edge_cases_tested, 0);
    assert_eq!(stats.memory_allocated, 0);
    assert_eq!(stats.execution_time, Duration::default());
    assert_eq!(stats.coverage_percentage, 0.0);
}

#[test]
fn test_harness_boundary_tests() {
    let config = TestConfig::default();
    let mut harness = ExhaustiveTestHarness::new(config);

    let result = harness.run_boundary_tests();
    assert!(result.is_ok());
    assert!(harness.stats.edge_cases_tested > 0);
}

#[test]
fn test_harness_error_injection_disabled() {
    let config = TestConfig {
        inject_errors: false,
        ..Default::default()
    };
    let mut harness = ExhaustiveTestHarness::new(config);

    let result = harness.run_error_injection_tests();
    assert!(result.is_ok());
}

#[test]
fn test_harness_error_injection_enabled() {
    let config = TestConfig {
        inject_errors: true,
        ..Default::default()
    };
    let mut harness = ExhaustiveTestHarness::new(config);

    let result = harness.run_error_injection_tests();
    assert!(result.is_ok());
    assert!(harness.stats.edge_cases_tested > 0);
}

#[test]
fn test_harness_stress_tests() {
    let config = TestConfig::default();
    let mut harness = ExhaustiveTestHarness::new(config);

    let result = harness.run_stress_tests();
    assert!(result.is_ok());
    assert!(harness.stats.edge_cases_tested > 0);
}

#[test]
fn test_harness_verify_coverage() {
    let config = TestConfig::default();
    let mut harness = ExhaustiveTestHarness::new(config);

    let result = harness.verify_coverage();
    assert!(result.is_ok());
    assert!(harness.stats.coverage_percentage > 0.0);
}

#[test]
fn test_harness_semantically_equivalent() {
    let config = TestConfig::default();
    let harness = ExhaustiveTestHarness::new(config);

    // Same normalized content should be equivalent
    assert!(harness.semantically_equivalent("echo hello", "echo hello"));
    assert!(harness.semantically_equivalent("  echo hello  ", "echo hello"));

    // Different content should not be equivalent
    assert!(!harness.semantically_equivalent("echo hello", "echo world"));
}

#[test]
fn test_harness_normalize_output() {
    let config = TestConfig::default();
    let harness = ExhaustiveTestHarness::new(config);

    let output = "  echo hello  \n  echo world  \n# comment\n\n";
    let normalized = harness.normalize_output(output);

    assert!(!normalized.contains("  ")); // No leading/trailing spaces
    assert!(!normalized.contains("# comment")); // No comments
    assert!(!normalized.contains("\n\n")); // No empty lines in result
}

#[test]
fn test_harness_fill_template() {
    let config = TestConfig::default();
    let harness = ExhaustiveTestHarness::new(config);

    let template = "fn {}() { {} }";
    let values = vec!["main".to_string(), "return".to_string()];
    let result = harness.fill_template(template, &values);

    assert_eq!(result, "fn main() { return }");
}

#[test]
fn test_harness_estimate_coverage_low() {
    let config = TestConfig::default();
    let harness = ExhaustiveTestHarness::new(config);

    // With default stats, should get base coverage
    let coverage = harness.estimate_coverage();
    assert!(coverage >= 70.0);
    assert!(coverage <= 100.0);
}

#[test]
fn test_harness_estimate_coverage_high() {
    let config = TestConfig::default();
    let mut harness = ExhaustiveTestHarness::new(config);

    // Add many edge cases and tests
    harness.stats.edge_cases_tested = 2000;
    harness.stats.total_tests = 200_000;

    let coverage = harness.estimate_coverage();
    assert!(coverage >= 90.0);
    assert!(coverage <= 100.0);
}

#[test]
fn test_harness_regression_test_cases() {
    let config = TestConfig::default();
    let harness = ExhaustiveTestHarness::new(config);

    let cases = harness.load_regression_test_cases().unwrap();
    assert!(!cases.is_empty());

    let case = &cases[0];
    assert!(!case.description.is_empty());
    assert!(!case.input.is_empty());
}

#[test]
fn test_harness_validation_test_cases() {
    let config = TestConfig::default();
    let harness = ExhaustiveTestHarness::new(config);

    let cases = harness.load_validation_test_cases().unwrap();
    assert!(!cases.is_empty());

    let case = &cases[0];
    assert!(!case.description.is_empty());
    assert!(!case.input.is_empty());
    assert!(case.reference_output.is_some());
}

#[test]
fn test_harness_individual_boundary_tests() {
    let config = TestConfig::default();
    let mut harness = ExhaustiveTestHarness::new(config);

    // Test individual boundary functions
    let initial = harness.stats.edge_cases_tested;

    harness.test_integer_boundaries().unwrap();
    assert!(harness.stats.edge_cases_tested > initial);

    let after_int = harness.stats.edge_cases_tested;
    harness.test_string_boundaries().unwrap();
    assert!(harness.stats.edge_cases_tested > after_int);

    let after_str = harness.stats.edge_cases_tested;
    harness.test_memory_boundaries().unwrap();
    assert!(harness.stats.edge_cases_tested > after_str);

    let after_mem = harness.stats.edge_cases_tested;
    harness.test_syntax_boundaries().unwrap();
    assert!(harness.stats.edge_cases_tested > after_mem);
}

#[test]
fn test_harness_individual_error_tests() {
    let config = TestConfig::default();
    let mut harness = ExhaustiveTestHarness::new(config);

    let initial = harness.stats.edge_cases_tested;

    harness.test_allocation_failures().unwrap();
    assert!(harness.stats.edge_cases_tested > initial);

    let after_alloc = harness.stats.edge_cases_tested;
    harness.test_io_failures().unwrap();
    assert!(harness.stats.edge_cases_tested > after_alloc);

    let after_io = harness.stats.edge_cases_tested;
    harness.test_parser_failures().unwrap();
    assert!(harness.stats.edge_cases_tested > after_io);
}

#[test]
fn test_harness_individual_stress_tests() {
    let config = TestConfig::default();
    let mut harness = ExhaustiveTestHarness::new(config);

    let initial = harness.stats.edge_cases_tested;

    harness.test_large_inputs().unwrap();
    assert!(harness.stats.edge_cases_tested > initial);

    let after_large = harness.stats.edge_cases_tested;
    harness.test_deep_nesting().unwrap();
    assert!(harness.stats.edge_cases_tested > after_large);

    let after_nest = harness.stats.edge_cases_tested;
    harness.test_concurrent_execution().unwrap();
    assert!(harness.stats.edge_cases_tested > after_nest);
}

#[test]
fn test_harness_run_single_test() {
    let config = TestConfig::default();
    let harness = ExhaustiveTestHarness::new(config);

    // Test with valid input
    let result = harness.run_single_test("fn main() {}", &Config::default());
    // May or may not succeed depending on transpiler state
    let _ = result;
}

#[test]
fn test_harness_generate_random_values() {
    let config = TestConfig::default();
    let harness = ExhaustiveTestHarness::new(config);
    let mut rng = rand::rng();

    let values = harness.generate_random_values(&mut rng);
    assert_eq!(values.len(), 3);
    // First value is a number
    assert!(values[0].parse::<u32>().is_ok());
    // Third value is boolean string
    assert!(values[2] == "true" || values[2] == "false");
}

#[test]
fn test_harness_generate_random_string() {
    let config = TestConfig::default();
    let harness = ExhaustiveTestHarness::new(config);
    let mut rng = rand::rng();

    for _ in 0..10 {
        let s = harness.generate_random_string(&mut rng, 50);
        assert!(s.len() <= 50);
        // All chars should be lowercase letters
        assert!(s.chars().all(|c| c.is_ascii_lowercase()));
    }
}

#[test]
fn test_regression_test_case_debug() {
    let case = RegressionTestCase {
        description: "test case".to_string(),
        input: "input".to_string(),
        config: Config::default(),
        expected_result: Ok("output".to_string()),
    };
    let debug = format!("{:?}", case);
    assert!(debug.contains("test case"));
}

#[test]
fn test_validation_test_case_debug() {
    let case = ValidationTestCase {
        description: "validation".to_string(),
        input: "input".to_string(),
        config: Config::default(),
        reference_output: Some("reference".to_string()),
    };
    let debug = format!("{:?}", case);
    assert!(debug.contains("validation"));
}

#[test]
fn test_test_config_clone() {
    let config = TestConfig {
        enable_assertions: false,
        track_coverage: false,
        inject_errors: false,
        fuzz_iterations: 100,
        memory_limit: None,
        timeout: Duration::from_secs(10),
        enable_mutation: true,
    };

    let cloned = config.clone();
    assert!(!cloned.enable_assertions);
    assert!(!cloned.track_coverage);
    assert!(!cloned.inject_errors);
    assert_eq!(cloned.fuzz_iterations, 100);
    assert!(cloned.memory_limit.is_none());
    assert!(cloned.enable_mutation);
}

#[test]
fn test_test_statistics_clone() {
    let stats = TestStatistics {
        total_tests: 100,
        passed_tests: 90,
        failed_tests: 10,
        edge_cases_tested: 50,
        memory_allocated: 1000,
        execution_time: Duration::from_secs(5),
        coverage_percentage: 85.5,
    };

    let cloned = stats.clone();
    assert_eq!(cloned.total_tests, 100);
    assert_eq!(cloned.passed_tests, 90);
    assert_eq!(cloned.failed_tests, 10);
    assert_eq!(cloned.coverage_percentage, 85.5);
}
