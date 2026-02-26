//! Audit command functions extracted from commands.rs.
//!
//! Handles `rash audit` subcommand: comprehensive quality audit combining
//! lint, test, and score checks, with output in human, JSON, or SARIF format.

use crate::cli::args::AuditOutputFormat;
use crate::models::{Error, Result};
use std::fs;
use std::path::Path;

/// Comprehensive quality audit results
#[derive(Debug)]
pub(crate) struct AuditResults {
    pub(crate) parse_success: bool,
    pub(crate) parse_error: Option<String>,
    pub(crate) lint_errors: usize,
    pub(crate) lint_warnings: usize,
    pub(crate) test_passed: usize,
    pub(crate) test_failed: usize,
    pub(crate) test_total: usize,
    pub(crate) score: Option<crate::bash_quality::scoring::QualityScore>,
    pub(crate) overall_pass: bool,
    pub(crate) failure_reason: Option<String>,
}

pub(crate) fn audit_command(
    input: &Path,
    format: &AuditOutputFormat,
    strict: bool,
    detailed: bool,
    min_grade: Option<&str>,
) -> Result<()> {
    use crate::linter::diagnostic::Severity;
    use crate::linter::rules::lint_shell;

    let source = fs::read_to_string(input)
        .map_err(|e| Error::Internal(format!("Failed to read {}: {}", input.display(), e)))?;

    let mut results = AuditResults {
        parse_success: true,
        parse_error: None,
        lint_errors: 0,
        lint_warnings: 0,
        test_passed: 0,
        test_failed: 0,
        test_total: 0,
        score: None,
        overall_pass: true,
        failure_reason: None,
    };

    // Lint check
    let lint_result = lint_shell(&source);
    results.lint_errors = lint_result
        .diagnostics
        .iter()
        .filter(|d| matches!(d.severity, Severity::Error))
        .count();
    results.lint_warnings = lint_result
        .diagnostics
        .iter()
        .filter(|d| matches!(d.severity, Severity::Warning))
        .count();

    audit_check_lint(&mut results, strict);
    audit_run_tests(&source, &mut results);
    audit_check_score(&source, min_grade, &mut results);

    // Output results
    match format {
        AuditOutputFormat::Human => print_human_audit_results(&results, detailed, input),
        AuditOutputFormat::Json => print_json_audit_results(&results),
        AuditOutputFormat::Sarif => print_sarif_audit_results(&results, input),
    }

    if !results.overall_pass {
        let reason = results
            .failure_reason
            .unwrap_or_else(|| "Quality audit failed".to_string());
        return Err(Error::Internal(reason));
    }

    Ok(())
}

pub(crate) fn audit_check_lint(results: &mut AuditResults, strict: bool) {
    if results.lint_errors > 0 {
        results.overall_pass = false;
        results.failure_reason = Some(format!("{} lint errors found", results.lint_errors));
    }
    if strict && results.lint_warnings > 0 {
        results.overall_pass = false;
        results.failure_reason = Some(format!(
            "Strict mode: {} warnings found",
            results.lint_warnings
        ));
    }
}

pub(crate) fn audit_run_tests(source: &str, results: &mut AuditResults) {
    use crate::bash_quality::testing::{discover_tests, run_tests, TestResult};

    let tests = match discover_tests(source) {
        Ok(t) => t,
        Err(_) => return,
    };
    let test_report = match run_tests(source, &tests) {
        Ok(r) => r,
        Err(_) => return,
    };

    results.test_total = test_report.results.len();
    results.test_passed = test_report
        .results
        .iter()
        .filter(|(_, result)| matches!(result, TestResult::Pass))
        .count();
    results.test_failed = test_report
        .results
        .iter()
        .filter(|(_, result)| matches!(result, TestResult::Fail(_)))
        .count();

    if results.test_failed > 0 {
        results.overall_pass = false;
        results.failure_reason = Some(format!(
            "{}/{} tests failed",
            results.test_failed, results.test_total
        ));
    }
}

pub(crate) fn audit_check_score(source: &str, min_grade: Option<&str>, results: &mut AuditResults) {
    use crate::bash_quality::scoring::score_script;

    let score = match score_script(source) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Warning: Failed to score script: {}", e);
            return;
        }
    };

    if let Some(min_grade_str) = min_grade {
        let grade_order = ["F", "D", "C", "C+", "B", "B+", "A", "A+"];
        let actual_grade_pos = grade_order.iter().position(|&g| g == score.grade.as_str());
        let min_grade_pos = grade_order.iter().position(|&g| g == min_grade_str);
        if let (Some(actual), Some(min)) = (actual_grade_pos, min_grade_pos) {
            if actual < min {
                results.overall_pass = false;
                results.failure_reason = Some(format!(
                    "Quality grade {} below minimum required grade {}",
                    score.grade, min_grade_str
                ));
            }
        }
    }

    results.score = Some(score);
}

/// Print human-readable audit results with ANSI colors
pub(crate) fn print_human_audit_results(results: &AuditResults, detailed: bool, input: &Path) {
    use crate::cli::color::*;

    println!();
    println!("{BOLD}Comprehensive Quality Audit{RESET}");
    println!("{DIM}══════════════════════════{RESET}");
    println!();
    println!("File: {CYAN}{}{RESET}", input.display());
    println!();
    println!("{BOLD}Check Results:{RESET}");
    println!("{DIM}──────────────{RESET}");

    // Parse
    if results.parse_success {
        println!("{GREEN}✓{RESET} Parse:    Valid bash syntax");
    } else {
        println!("{BRIGHT_RED}✗{RESET} Parse:    Syntax error");
        if let Some(err) = &results.parse_error {
            println!("           {DIM}{err}{RESET}");
        }
    }

    // Lint
    if results.lint_errors == 0 && results.lint_warnings == 0 {
        println!("{GREEN}✓{RESET} Lint:     No issues found");
    } else if results.lint_errors > 0 {
        println!(
            "{BRIGHT_RED}✗{RESET} Lint:     {BRIGHT_RED}{} errors{RESET}, {YELLOW}{} warnings{RESET}",
            results.lint_errors, results.lint_warnings
        );
    } else {
        println!(
            "{YELLOW}⚠{RESET} Lint:     {YELLOW}{} warnings{RESET}",
            results.lint_warnings
        );
    }

    // Test
    if results.test_total > 0 {
        if results.test_failed == 0 {
            println!(
                "{GREEN}✓{RESET} Test:     {GREEN}{}/{} tests passed{RESET}",
                results.test_passed, results.test_total
            );
        } else {
            println!(
                "{BRIGHT_RED}✗{RESET} Test:     {}/{} tests passed, {BRIGHT_RED}{} failed{RESET}",
                results.test_passed, results.test_total, results.test_failed
            );
        }
    } else {
        println!("{YELLOW}⚠{RESET} Test:     {DIM}No tests found{RESET}");
    }

    // Score
    if let Some(score) = &results.score {
        let gc = grade_color(&score.grade);
        println!(
            "{GREEN}✓{RESET} Score:    {gc}{}{RESET} ({WHITE}{:.1}/10.0{RESET})",
            score.grade, score.score
        );

        if detailed {
            println!();
            println!("  {BOLD}Dimension Breakdown:{RESET}");
            let dim_line = |name: &str, val: f64| {
                let sc = score_color(val * 10.0);
                println!("  {DIM}-{RESET} {:<17} {sc}{:.1}/10.0{RESET}", name, val);
            };
            dim_line("Complexity:", score.complexity);
            dim_line("Safety:", score.safety);
            dim_line("Maintainability:", score.maintainability);
            dim_line("Testing:", score.testing);
            dim_line("Documentation:", score.documentation);
        }
    }

    println!();
    if results.overall_pass {
        println!("Overall: {GREEN}{BOLD}✓ PASS{RESET}");
    } else {
        println!("Overall: {BRIGHT_RED}{BOLD}✗ FAIL{RESET}");
    }
    println!();

    // Suggestions
    if let Some(score) = &results.score {
        if !score.suggestions.is_empty() {
            println!("{BOLD}Improvement Suggestions:{RESET}");
            println!("{DIM}────────────────────────{RESET}");
            for (i, suggestion) in score.suggestions.iter().enumerate() {
                println!("{YELLOW}{}. {}{RESET}", i + 1, suggestion);
            }
            println!();
        }
    }
}

/// Print JSON audit results
pub(crate) fn print_json_audit_results(results: &AuditResults) {
    use serde_json::json;

    let json_results = json!({
        "audit": {
            "parse": {
                "success": results.parse_success,
                "error": results.parse_error,
            },
            "lint": {
                "errors": results.lint_errors,
                "warnings": results.lint_warnings,
            },
            "test": {
                "total": results.test_total,
                "passed": results.test_passed,
                "failed": results.test_failed,
            },
            "score": results.score.as_ref().map(|s| json!({
                "grade": s.grade,
                "score": s.score,
                "dimensions": {
                    "complexity": s.complexity,
                    "safety": s.safety,
                    "maintainability": s.maintainability,
                    "testing": s.testing,
                    "documentation": s.documentation,
                },
                "suggestions": s.suggestions,
            })),
            "overall_pass": results.overall_pass,
        }
    });

    match serde_json::to_string_pretty(&json_results) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error serializing JSON: {}", e);
            std::process::exit(1);
        }
    }
}

/// Print SARIF audit results (GitHub Code Scanning format)
pub(crate) fn print_sarif_audit_results(results: &AuditResults, input: &Path) {
    use serde_json::json;

    let mut sarif_results = vec![];

    // Add parse error if any
    if !results.parse_success {
        if let Some(err) = &results.parse_error {
            sarif_results.push(json!({
                "ruleId": "PARSE-001",
                "level": "error",
                "message": {
                    "text": format!("Parse error: {}", err)
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": input.display().to_string()
                        }
                    }
                }]
            }));
        }
    }

    // Add lint issues
    if results.lint_errors > 0 || results.lint_warnings > 0 {
        sarif_results.push(json!({
            "ruleId": "LINT-001",
            "level": if results.lint_errors > 0 { "error" } else { "warning" },
            "message": {
                "text": format!("{} errors, {} warnings", results.lint_errors, results.lint_warnings)
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": input.display().to_string()
                    }
                }
            }]
        }));
    }

    // Add test failures
    if results.test_failed > 0 {
        sarif_results.push(json!({
            "ruleId": "TEST-001",
            "level": "error",
            "message": {
                "text": format!("{}/{} tests failed", results.test_failed, results.test_total)
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": input.display().to_string()
                    }
                }
            }]
        }));
    }

    let sarif = json!({
        "version": "2.1.0",
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "bashrs audit",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/bashrs"
                }
            },
            "results": sarif_results
        }]
    });

    match serde_json::to_string_pretty(&sarif) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error serializing JSON: {}", e);
            std::process::exit(1);
        }
    }
}
