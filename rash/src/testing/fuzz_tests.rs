//! Tests for the fuzz testing module

use super::fuzz::*;

#[test]
fn test_fuzz_tester_new() {
    let tester = FuzzTester::new();
    // Test that the tester can be created
    let result = tester.run_fuzz_tests();
    assert!(result.is_ok());
}

#[test]
fn test_fuzz_tester_default() {
    let tester = FuzzTester;
    let result = tester.run_fuzz_tests();
    assert!(result.is_ok());
}

#[test]
fn test_fuzz_tester_multiple_runs() {
    let tester = FuzzTester::new();

    // Test multiple consecutive runs
    for _ in 0..5 {
        let result = tester.run_fuzz_tests();
        assert!(result.is_ok());
    }
}
