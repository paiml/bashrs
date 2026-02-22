//! Test command functions extracted from commands.rs.
//!
//! Handles `rash test` subcommand: discovering and running bash tests,
//! then outputting results in human, JSON, or JUnit format.

use crate::models::{Error, Result};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

use crate::cli::args::TestOutputFormat;

pub(crate) fn test_command(
    input: &Path,
    format: TestOutputFormat,
    detailed: bool,
    pattern: Option<&str>,
) -> Result<()> {
    use crate::bash_quality::testing::{discover_tests, run_tests};

    // Read input file
    let source = fs::read_to_string(input)
        .map_err(|e| Error::Internal(format!("Failed to read {}: {}", input.display(), e)))?;

    // Discover tests
    let tests = discover_tests(&source)
        .map_err(|e| Error::Internal(format!("Failed to discover tests: {}", e)))?;

    if tests.is_empty() {
        warn!("No tests found in {}", input.display());
        println!("No tests found in {}", input.display());
        return Ok(());
    }

    // Filter tests by pattern if provided
    let tests_to_run: Vec<_> = if let Some(pat) = pattern {
        tests
            .iter()
            .filter(|t| t.name.contains(pat))
            .cloned()
            .collect()
    } else {
        tests.clone()
    };

    if tests_to_run.is_empty() {
        warn!("No tests matching pattern '{}'", pattern.unwrap_or(""));
        println!("No tests matching pattern '{}'", pattern.unwrap_or(""));
        return Ok(());
    }

    info!(
        "Running {} tests from {}",
        tests_to_run.len(),
        input.display()
    );

    // Run tests
    let report = run_tests(&source, &tests_to_run)
        .map_err(|e| Error::Internal(format!("Failed to run tests: {}", e)))?;

    // Output results
    match format {
        TestOutputFormat::Human => {
            print_human_test_results(&report, detailed);
        }
        TestOutputFormat::Json => {
            print_json_test_results(&report);
        }
        TestOutputFormat::Junit => {
            print_junit_test_results(&report);
        }
    }

    // Exit with error if tests failed
    if report.failed() > 0 {
        return Err(Error::Internal(format!(
            "{} test(s) failed",
            report.failed()
        )));
    }

    Ok(())
}

/// Print human-readable test results
pub(crate) fn print_human_test_results(
    report: &crate::bash_quality::testing::TestReport,
    detailed: bool,
) {
    use crate::bash_quality::testing::TestResult;

    println!();
    println!("Test Results");
    println!("============");
    println!();

    for (test_name, result) in &report.results {
        match result {
            TestResult::Pass => {
                println!("\u{2713} {}", test_name);
                if detailed {
                    print_test_detail(report, test_name, true);
                }
            }
            TestResult::Fail(msg) => {
                println!("\u{2717} {}", test_name);
                println!("  Error: {}", msg);
                if detailed {
                    print_test_detail(report, test_name, false);
                }
            }
            TestResult::Skip(reason) => {
                println!("\u{2298} {} (skipped: {})", test_name, reason);
            }
        }
    }

    print_test_summary(report);
}

pub(crate) fn print_test_detail(
    report: &crate::bash_quality::testing::TestReport,
    test_name: &str,
    full: bool,
) {
    let test = match report.tests.iter().find(|t| t.name == test_name) {
        Some(t) => t,
        None => return,
    };
    if let Some(desc) = &test.description {
        println!("  Description: {}", desc);
    }
    if full {
        if let Some(given) = &test.given {
            println!("  Given: {}", given);
        }
        if let Some(when) = &test.when {
            println!("  When: {}", when);
        }
        if let Some(then) = &test.then {
            println!("  Then: {}", then);
        }
    }
}

pub(crate) fn print_test_summary(report: &crate::bash_quality::testing::TestReport) {
    println!();
    println!("Summary");
    println!("-------");
    println!("Total:   {}", report.results.len());
    println!("Passed:  {}", report.passed());
    println!("Failed:  {}", report.failed());
    println!("Skipped: {}", report.skipped());
    println!("Time:    {}ms", report.duration_ms);
    println!();
    if report.all_passed() {
        println!("\u{2713} All tests passed!");
    } else {
        println!("\u{2717} {} test(s) failed", report.failed());
    }
}

/// Print JSON test results
pub(crate) fn print_json_test_results(report: &crate::bash_quality::testing::TestReport) {
    use serde_json::json;

    let json_report = json!({
        "tests": report.tests.iter().map(|t| json!({
            "name": t.name,
            "line": t.line,
            "description": t.description,
            "given": t.given,
            "when": t.when,
            "then": t.then,
        })).collect::<Vec<_>>(),
        "results": report.results.iter().map(|(name, result)| json!({
            "name": name,
            "result": match result {
                crate::bash_quality::testing::TestResult::Pass => "pass",
                crate::bash_quality::testing::TestResult::Fail(_) => "fail",
                crate::bash_quality::testing::TestResult::Skip(_) => "skip",
            },
            "message": match result {
                crate::bash_quality::testing::TestResult::Fail(msg) => Some(msg),
                crate::bash_quality::testing::TestResult::Skip(msg) => Some(msg),
                _ => None,
            },
        })).collect::<Vec<_>>(),
        "summary": {
            "total": report.results.len(),
            "passed": report.passed(),
            "failed": report.failed(),
            "skipped": report.skipped(),
            "duration_ms": report.duration_ms,
            "all_passed": report.all_passed(),
        }
    });

    match serde_json::to_string_pretty(&json_report) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error serializing JSON: {}", e);
            std::process::exit(1);
        }
    }
}

/// Print JUnit XML test results
pub(crate) fn print_junit_test_results(report: &crate::bash_quality::testing::TestReport) {
    println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    println!(
        "<testsuite tests=\"{}\" failures=\"{}\" skipped=\"{}\" time=\"{:.3}\">",
        report.results.len(),
        report.failed(),
        report.skipped(),
        report.duration_ms as f64 / 1000.0
    );

    for (test_name, result) in &report.results {
        match result {
            crate::bash_quality::testing::TestResult::Pass => {
                println!("  <testcase name=\"{}\" />", test_name);
            }
            crate::bash_quality::testing::TestResult::Fail(msg) => {
                println!("  <testcase name=\"{}\">", test_name);
                println!("    <failure message=\"{}\" />", msg.replace('"', "&quot;"));
                println!("  </testcase>");
            }
            crate::bash_quality::testing::TestResult::Skip(reason) => {
                println!("  <testcase name=\"{}\">", test_name);
                println!(
                    "    <skipped message=\"{}\" />",
                    reason.replace('"', "&quot;")
                );
                println!("  </testcase>");
            }
        }
    }

    println!("</testsuite>");
}
