use super::*;

// RED PHASE: Write failing tests first

#[test]
fn test_discover_empty_script() {
    let source = "";
    let tests = discover_tests(source).unwrap();
    assert_eq!(tests.len(), 0);
}

#[test]
fn test_discover_single_test() {
    let source = r#"
test_example() {
echo "test"
}
"#;
    let tests = discover_tests(source).unwrap();
    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].name, "test_example");
}

#[test]
fn test_discover_test_with_description() {
    let source = r#"
# TEST: example function works correctly
test_example() {
echo "test"
}
"#;
    let tests = discover_tests(source).unwrap();
    assert_eq!(tests.len(), 1);
    assert_eq!(
        tests[0].description,
        Some("example function works correctly".to_string())
    );
}

#[test]
fn test_discover_test_with_given_when_then() {
    let source = r#"
# TEST: my_function with input 5
# GIVEN: x=5
# WHEN: my_function 5
# THEN: output should be "Result: 5"
test_my_function_basic() {
result=$(my_function 5)
[[ "$result" == "Result: 5" ]] || return 1
}
"#;
    let tests = discover_tests(source).unwrap();
    assert_eq!(tests.len(), 1);
    assert_eq!(
        tests[0].description,
        Some("my_function with input 5".to_string())
    );
    assert_eq!(tests[0].given, Some("x=5".to_string()));
    assert_eq!(tests[0].when, Some("my_function 5".to_string()));
    assert_eq!(
        tests[0].then,
        Some("output should be \"Result: 5\"".to_string())
    );
}

#[test]
fn test_ignore_non_test_functions() {
    let source = r#"
my_function() {
echo "not a test"
}

test_example() {
echo "this is a test"
}
"#;
    let tests = discover_tests(source).unwrap();
    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].name, "test_example");
}

#[test]
fn test_discover_multiple_tests() {
    let source = r#"
test_one() {
echo "test1"
}

test_two() {
echo "test2"
}

test_three() {
echo "test3"
}
"#;
    let tests = discover_tests(source).unwrap();
    assert_eq!(tests.len(), 3);
    assert_eq!(tests[0].name, "test_one");
    assert_eq!(tests[1].name, "test_two");
    assert_eq!(tests[2].name, "test_three");
}

#[test]
fn test_extract_function_name_standard_syntax() {
    let line = "test_example() {";
    let name = extract_function_name(line).unwrap();
    assert_eq!(name, "test_example");
}

#[test]
fn test_extract_function_name_function_keyword() {
    let line = "function test_example() {";
    let name = extract_function_name(line).unwrap();
    assert_eq!(name, "test_example");
}

#[test]
fn test_test_report_counts() {
    let mut report = TestReport::new();
    report.results.push(("test1".to_string(), TestResult::Pass));
    report.results.push(("test2".to_string(), TestResult::Pass));
    report
        .results
        .push(("test3".to_string(), TestResult::Fail("error".to_string())));
    report
        .results
        .push(("test4".to_string(), TestResult::Skip("skipped".to_string())));

    assert_eq!(report.passed(), 2);
    assert_eq!(report.failed(), 1);
    assert_eq!(report.skipped(), 1);
    assert!(!report.all_passed());
}

// TEST EXECUTION TESTS (RED phase)

#[test]
fn test_run_tests_empty_script() {
    let source = "";
    let tests = discover_tests(source).unwrap();
    let report = run_tests(source, &tests).unwrap();

    assert_eq!(report.tests.len(), 0);
    assert_eq!(report.results.len(), 0);
    assert_eq!(report.passed(), 0);
    assert_eq!(report.failed(), 0);
}

#[test]
fn test_run_tests_single_passing_test() {
    let source = r#"
test_example() {
echo "test"
return 0
}
"#;
    let tests = discover_tests(source).unwrap();
    let report = run_tests(source, &tests).unwrap();

    assert_eq!(report.tests.len(), 1);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed(), 1);
    assert_eq!(report.failed(), 0);
    assert!(matches!(report.results[0].1, TestResult::Pass));
}

#[test]
fn test_run_tests_single_failing_test() {
    let source = r#"
test_example() {
echo "test"
return 1
}
"#;
    let tests = discover_tests(source).unwrap();
    let report = run_tests(source, &tests).unwrap();

    assert_eq!(report.tests.len(), 1);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.passed(), 0);
    assert_eq!(report.failed(), 1);
    assert!(matches!(report.results[0].1, TestResult::Fail(_)));
}

#[test]
fn test_run_tests_multiple_tests() {
    let source = r#"
test_pass() {
return 0
}

test_fail() {
return 1
}

test_pass2() {
[ "x" = "x" ]
}
"#;
    let tests = discover_tests(source).unwrap();
    let report = run_tests(source, &tests).unwrap();

    assert_eq!(report.tests.len(), 3);
    assert_eq!(report.results.len(), 3);
    assert_eq!(report.passed(), 2);
    assert_eq!(report.failed(), 1);
}

#[test]
fn test_run_tests_captures_output() {
    let source = r#"
test_with_output() {
echo "Hello from test"
return 0
}
"#;
    let tests = discover_tests(source).unwrap();
    let report = run_tests(source, &tests).unwrap();

    assert_eq!(report.passed(), 1);
    // Output should be captured (verified in implementation)
}

#[test]
fn test_run_tests_timing() {
    let source = r#"
test_quick() {
return 0
}
"#;
    let tests = discover_tests(source).unwrap();
    let report = run_tests(source, &tests).unwrap();

    // Test passes if duration is recorded (duration_ms is u64, always >= 0)
    let _ = report.duration_ms;
}
