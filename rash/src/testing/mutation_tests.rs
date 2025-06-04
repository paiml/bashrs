//! Tests for the mutation testing module

use super::mutation::*;

#[test]
fn test_mutation_tester_new() {
    let tester = MutationTester::new();
    let result = tester.run_mutation_tests();
    assert!(result.is_ok());
}

#[test]
fn test_mutation_tester_default() {
    let tester = MutationTester;
    let result = tester.run_mutation_tests();
    assert!(result.is_ok());
}

#[test]
fn test_mutation_tester_consistency() {
    let tester1 = MutationTester::new();
    let tester2 = MutationTester;

    let result1 = tester1.run_mutation_tests();
    let result2 = tester2.run_mutation_tests();

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[test]
fn test_mutation_tester_repeated_runs() {
    let tester = MutationTester::new();

    for _ in 0..3 {
        let result = tester.run_mutation_tests();
        assert!(result.is_ok());
    }
}
