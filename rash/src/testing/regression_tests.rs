//! Tests for the regression testing module

use super::regression::*;

#[test]
fn test_regression_tester_new() {
    let tester = RegressionTester::new();
    let result = tester.run_regression_tests();
    assert!(result.is_ok());
}

#[test]
fn test_regression_tester_default() {
    let tester = RegressionTester;
    let result = tester.run_regression_tests();
    assert!(result.is_ok());
}

#[test]
fn test_regression_tester_sequential_runs() {
    let tester = RegressionTester::new();

    // Run multiple times to ensure consistency
    let results: Vec<_> = (0..5).map(|_| tester.run_regression_tests()).collect();

    for result in results {
        assert!(result.is_ok());
    }
}

#[test]
fn test_regression_tester_instantiation_methods() {
    let tester1 = RegressionTester::new();
    let tester2 = RegressionTester;

    // Both should work identically
    assert!(tester1.run_regression_tests().is_ok());
    assert!(tester2.run_regression_tests().is_ok());
}
