//! Property command handler (Sprint 12, PMAT-218).
//!
//! Handles `bashrs property` subcommand: analyzes bash scripts
//! for idempotency, determinism, POSIX compliance, and safety properties.

use crate::bash_quality::property::{self, Property};
use crate::cli::args::PropertyOutputFormat;
use crate::models::{Error, Result};
use std::fs;
use std::path::Path;

pub(crate) fn property_command(
    input: &Path,
    properties: Option<&str>,
    custom: Option<&Path>,
    iterations: usize,
    format: PropertyOutputFormat,
) -> Result<()> {
    let source = fs::read_to_string(input)
        .map_err(|e| Error::Internal(format!("Failed to read {}: {}", input.display(), e)))?;

    let props = match properties {
        Some(p) => {
            let parsed = Property::parse_list(p);
            if parsed.is_empty() {
                return Err(Error::Internal(format!(
                    "No valid properties in '{}'. Valid: idempotency,determinism,posix,safety",
                    p
                )));
            }
            parsed
        }
        None => Property::all().to_vec(),
    };

    let report = property::analyze_properties(&source, &props, iterations);

    // Load and evaluate custom properties if --custom is given
    let custom_results = if let Some(custom_path) = custom {
        let defs = property::load_custom_properties(custom_path)
            .map_err(Error::Internal)?;
        property::analyze_custom_properties(&source, &defs)
    } else {
        Vec::new()
    };

    match format {
        PropertyOutputFormat::Human => {
            print_human(&report, &custom_results, input);
        }
        PropertyOutputFormat::Json => {
            print_json(&report, &custom_results, input);
        }
    }

    let custom_violations: usize = custom_results.iter().map(|r| r.violations.len()).sum();
    let total_violations = report.total_violations() + custom_violations;

    if total_violations == 0 {
        Ok(())
    } else {
        Err(Error::Internal(format!(
            "{} property violation(s) found",
            total_violations
        )))
    }
}

fn print_human(
    report: &property::PropertyReport,
    custom_results: &[property::CustomPropertyResult],
    input: &Path,
) {
    use crate::cli::color::*;

    println!();
    println!(
        "{BOLD}Property Report:{RESET} {CYAN}{}{RESET}",
        input.display()
    );
    println!();

    for result in &report.results {
        let status = if result.passed {
            format!("{GREEN}PASS{RESET}")
        } else {
            format!("{BRIGHT_RED}FAIL{RESET}")
        };
        println!(
            "  {} {BOLD}{}{RESET} — {}",
            status,
            result.property.name(),
            result.property.description(),
        );

        for v in &result.violations {
            println!(
                "      {YELLOW}line {}:{RESET} {}",
                v.line, v.message
            );
            if let Some(ref sug) = v.suggestion {
                println!("      {DIM}suggestion: {}{RESET}", sug);
            }
        }
    }

    // Custom properties
    if !custom_results.is_empty() {
        println!();
        println!("{BOLD}Custom Properties:{RESET}");
        for result in custom_results {
            let status = if result.passed {
                format!("{GREEN}PASS{RESET}")
            } else {
                format!("{BRIGHT_RED}FAIL{RESET}")
            };
            let desc = if result.description.is_empty() {
                String::new()
            } else {
                format!(" — {}", result.description)
            };
            println!("  {} {BOLD}{}{RESET}{}", status, result.name, desc);

            for v in &result.violations {
                if v.line > 0 {
                    println!("      {YELLOW}line {}:{RESET} {}", v.line, v.message);
                } else {
                    println!("      {YELLOW}global:{RESET} {}", v.message);
                }
                if let Some(ref sug) = v.suggestion {
                    println!("      {DIM}suggestion: {}{RESET}", sug);
                }
            }
        }
    }

    let custom_violations: usize = custom_results.iter().map(|r| r.violations.len()).sum();
    let total = report.total_violations() + custom_violations;

    println!();
    if total == 0 {
        println!("{GREEN}All properties satisfied.{RESET}");
    } else {
        println!("{BRIGHT_RED}{} violation(s) found.{RESET}", total);
    }
}

fn print_json(
    report: &property::PropertyReport,
    custom_results: &[property::CustomPropertyResult],
    input: &Path,
) {
    use serde_json::json;

    let custom_violations: usize = custom_results.iter().map(|r| r.violations.len()).sum();
    let all_passed = report.all_passed() && custom_results.iter().all(|r| r.passed);
    let total_violations = report.total_violations() + custom_violations;

    let mut json_report = json!({
        "file": input.display().to_string(),
        "all_passed": all_passed,
        "total_violations": total_violations,
        "properties": report.results.iter().map(|r| json!({
            "name": r.property.name(),
            "passed": r.passed,
            "iterations": r.iterations,
            "violations": r.violations.iter().map(|v| json!({
                "line": v.line,
                "message": v.message,
                "suggestion": v.suggestion,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    });

    if !custom_results.is_empty() {
        json_report["custom_properties"] = json!(custom_results.iter().map(|r| json!({
            "name": r.name,
            "description": r.description,
            "passed": r.passed,
            "violations": r.violations.iter().map(|v| json!({
                "line": v.line,
                "message": v.message,
                "suggestion": v.suggestion,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>());
    }

    match serde_json::to_string_pretty(&json_report) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing JSON: {}", e),
    }
}
