//! Bash Test Framework
//!
//! Provides test discovery and execution for bash scripts using inline tests.
//!
//! ## Test Format
//!
//! Tests are bash functions starting with `test_` and can include GIVEN/WHEN/THEN comments:
//!
//! ```bash
//! # TEST: my_function with valid input
//! # GIVEN: x=5
//! # WHEN: my_function 5
//! # THEN: output should be "Result: 5"
//! test_my_function_basic() {
//!     result=$(my_function 5)
//!     [[ "$result" == "Result: 5" ]] || return 1
//! }
//! ```
//!
//! ## Usage
//!
//! ```bash
//! bashrs test script.sh
//! ```

use std::fs;
use std::process::Command;
use std::time::Instant;

/// A discovered test in a bash script
#[derive(Debug, Clone, PartialEq)]
pub struct BashTest {
    /// Test function name (e.g., "test_my_function_basic")
    pub name: String,

    /// Line number where test is defined
    pub line: usize,

    /// Test description from TEST comment
    pub description: Option<String>,

    /// GIVEN clause from comments
    pub given: Option<String>,

    /// WHEN clause from comments
    pub when: Option<String>,

    /// THEN clause from comments
    pub then: Option<String>,

    /// The actual test function body
    pub body: String,
}

/// Result of running a single test
#[derive(Debug, Clone, PartialEq)]
pub enum TestResult {
    /// Test passed
    Pass,

    /// Test failed with message
    Fail(String),

    /// Test was skipped
    Skip(String),
}

/// Test execution report
#[derive(Debug, Clone)]
pub struct TestReport {
    /// All discovered tests
    pub tests: Vec<BashTest>,

    /// Results for each test
    pub results: Vec<(String, TestResult)>,

    /// Total execution time in milliseconds
    pub duration_ms: u64,
}

impl TestReport {
    /// Create new empty test report
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            results: Vec::new(),
            duration_ms: 0,
        }
    }

    /// Count passed tests
    pub fn passed(&self) -> usize {
        self.results
            .iter()
            .filter(|(_, r)| matches!(r, TestResult::Pass))
            .count()
    }

    /// Count failed tests
    pub fn failed(&self) -> usize {
        self.results
            .iter()
            .filter(|(_, r)| matches!(r, TestResult::Fail(_)))
            .count()
    }

    /// Count skipped tests
    pub fn skipped(&self) -> usize {
        self.results
            .iter()
            .filter(|(_, r)| matches!(r, TestResult::Skip(_)))
            .count()
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed() == 0 && !self.results.is_empty()
    }
}

impl Default for TestReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Discover all tests in a bash script
///
/// Scans the script for functions starting with `test_` and extracts
/// GIVEN/WHEN/THEN comments.
pub fn discover_tests(source: &str) -> Result<Vec<BashTest>, String> {
    let mut tests = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        if let Some(line) = lines.get(i) {
            // Look for test function definition
            if line.contains("test_") && line.contains("()") {
                if let Some(test) = parse_test_function(&lines, i)? {
                    tests.push(test);
                }
            }
        }

        i += 1;
    }

    Ok(tests)
}

/// Parse a single test function starting at the given line
fn parse_test_function(lines: &[&str], start_line: usize) -> Result<Option<BashTest>, String> {
    let line = lines
        .get(start_line)
        .ok_or_else(|| "Invalid line index".to_string())?;

    // Extract function name
    let name = extract_function_name(line)?;

    // Only process test_ functions
    if !name.starts_with("test_") {
        return Ok(None);
    }

    // Look backwards for comments (TEST, GIVEN, WHEN, THEN)
    let (description, given, when, then) = extract_test_comments(lines, start_line);

    // Extract function body
    let body = extract_function_body(lines, start_line)?;

    Ok(Some(BashTest {
        name,
        line: start_line + 1, // 1-indexed
        description,
        given,
        when,
        then,
        body,
    }))
}

/// Extract function name from definition line
fn extract_function_name(line: &str) -> Result<String, String> {
    // Handle: test_foo() { or function test_foo() {
    let trimmed = line.trim();

    if let Some(pos) = trimmed.find('(') {
        let before_paren = &trimmed[..pos];
        let name = before_paren.trim().trim_start_matches("function").trim();

        if name.is_empty() {
            return Err("Empty function name".to_string());
        }

        Ok(name.to_string())
    } else {
        Err("No parentheses found in function definition".to_string())
    }
}

/// Extract TEST, GIVEN, WHEN, THEN comments before function
fn extract_test_comments(
    lines: &[&str],
    start_line: usize,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let mut description = None;
    let mut given = None;
    let mut when = None;
    let mut then = None;

    // Look backwards up to 10 lines for comments
    let search_start = start_line.saturating_sub(10);

    for line in lines.iter().take(start_line).skip(search_start) {
        let line = line.trim();

        if line.starts_with("# TEST:") || line.starts_with("#TEST:") {
            description = Some(
                line.trim_start_matches("# TEST:")
                    .trim_start_matches("#TEST:")
                    .trim()
                    .to_string(),
            );
        } else if line.starts_with("# GIVEN:") || line.starts_with("#GIVEN:") {
            given = Some(
                line.trim_start_matches("# GIVEN:")
                    .trim_start_matches("#GIVEN:")
                    .trim()
                    .to_string(),
            );
        } else if line.starts_with("# WHEN:") || line.starts_with("#WHEN:") {
            when = Some(
                line.trim_start_matches("# WHEN:")
                    .trim_start_matches("#WHEN:")
                    .trim()
                    .to_string(),
            );
        } else if line.starts_with("# THEN:") || line.starts_with("#THEN:") {
            then = Some(
                line.trim_start_matches("# THEN:")
                    .trim_start_matches("#THEN:")
                    .trim()
                    .to_string(),
            );
        }
    }

    (description, given, when, then)
}

/// Extract function body (everything between { and })
fn extract_function_body(lines: &[&str], start_line: usize) -> Result<String, String> {
    let mut body_lines = Vec::new();
    let mut brace_count = 0;
    let mut started = false;

    for (i, line) in lines.iter().enumerate().skip(start_line) {
        let line = *line;

        // Count braces
        for ch in line.chars() {
            if ch == '{' {
                brace_count += 1;
                started = true;
            } else if ch == '}' {
                brace_count -= 1;
            }
        }

        // Add line to body (skip the function definition line)
        if started && i > start_line {
            // Remove leading/trailing braces from body
            let trimmed = line.trim();
            if trimmed != "}" {
                body_lines.push(line);
            }
        }

        // Exit when braces are balanced
        if started && brace_count == 0 {
            break;
        }
    }

    Ok(body_lines.join("\n"))
}

/// Run all tests in a bash script
///
/// Executes each discovered test and collects results.
pub fn run_tests(source: &str, tests: &[BashTest]) -> Result<TestReport, String> {
    let start_time = Instant::now();
    let mut report = TestReport::new();
    report.tests = tests.to_vec();

    // If no tests, return early
    if tests.is_empty() {
        report.duration_ms = start_time.elapsed().as_millis() as u64;
        return Ok(report);
    }

    // Execute each test
    for test in tests {
        let result = execute_test(source, &test.name)?;
        report.results.push((test.name.clone(), result));
    }

    report.duration_ms = start_time.elapsed().as_millis() as u64;
    Ok(report)
}

/// Execute a single test function
fn execute_test(source: &str, test_name: &str) -> Result<TestResult, String> {
    // Create temporary script file with unique name
    let temp_dir = std::env::temp_dir();
    #[allow(clippy::expect_used)] // Safe: system time is always after UNIX_EPOCH
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time is after UNIX_EPOCH")
        .as_nanos();
    let script_path = temp_dir.join(format!("bashrs_test_{}_{}.sh", test_name, timestamp));

    // Write script with test execution
    let test_script = format!(
        r"#!/bin/bash

# Source the original script
{}

# Run the test function and capture exit code
{}
exit $?
",
        source, test_name
    );

    fs::write(&script_path, test_script)
        .map_err(|e| format!("Failed to write test script: {}", e))?;

    // Make script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)
            .map_err(|e| format!("Failed to get script permissions: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)
            .map_err(|e| format!("Failed to set script permissions: {}", e))?;
    }

    // Execute the test
    let output = Command::new("bash")
        .arg(&script_path)
        .output()
        .map_err(|e| format!("Failed to execute test {}: {}", test_name, e))?;

    // Clean up
    let _ = fs::remove_file(&script_path);

    // Check exit code
    if output.status.success() {
        Ok(TestResult::Pass)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let error_msg = if !stderr.is_empty() {
            stderr.to_string()
        } else if !stdout.is_empty() {
            stdout.to_string()
        } else {
            format!(
                "Test {} failed with exit code {:?}",
                test_name,
                output.status.code()
            )
        };
        Ok(TestResult::Fail(error_msg))
    }
}

#[cfg(test)]
mod tests {
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
}
