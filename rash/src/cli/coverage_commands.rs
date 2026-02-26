//! Coverage command functions extracted from commands.rs.
//!
//! Handles `rash coverage` subcommand: generating coverage reports for
//! bash scripts, with output in terminal, JSON, HTML, or LCOV format.

use crate::cli::args::CoverageOutputFormat;
use crate::cli::logic::coverage_class;
use crate::models::{Error, Result};
use std::fs;
use std::path::Path;

pub(crate) fn coverage_command(
    input: &Path,
    format: &CoverageOutputFormat,
    min: Option<u8>,
    detailed: bool,
    output: Option<&Path>,
) -> Result<()> {
    use crate::bash_quality::coverage::generate_coverage;

    // Read input file
    let source = fs::read_to_string(input)
        .map_err(|e| Error::Internal(format!("Failed to read {}: {}", input.display(), e)))?;

    // Generate coverage report
    let coverage = generate_coverage(&source)
        .map_err(|e| Error::Internal(format!("Failed to generate coverage: {}", e)))?;

    // Check minimum coverage if specified
    if let Some(min_percent) = min {
        let line_coverage = coverage.line_coverage_percent();
        if line_coverage < min_percent as f64 {
            return Err(Error::Internal(format!(
                "Coverage {:.1}% is below minimum {}%",
                line_coverage, min_percent
            )));
        }
    }

    // Output results
    match format {
        CoverageOutputFormat::Terminal => {
            print_terminal_coverage(&coverage, detailed, input);
        }
        CoverageOutputFormat::Json => {
            print_json_coverage(&coverage);
        }
        CoverageOutputFormat::Html => {
            print_html_coverage(&coverage, input, output);
        }
        CoverageOutputFormat::Lcov => {
            print_lcov_coverage(&coverage, input);
        }
    }

    Ok(())
}

/// Print terminal coverage output with ANSI colors
pub(crate) fn print_terminal_coverage(
    coverage: &crate::bash_quality::coverage::CoverageReport,
    detailed: bool,
    input: &Path,
) {
    use crate::cli::color::*;

    println!();
    println!(
        "{BOLD}Coverage Report:{RESET} {CYAN}{}{RESET}",
        input.display()
    );
    println!();

    let line_pct = coverage.line_coverage_percent();
    let func_pct = coverage.function_coverage_percent();

    // Overall coverage with progress bars
    let lc = score_color(line_pct);
    let fc = score_color(func_pct);
    let line_bar = progress_bar(coverage.covered_lines.len(), coverage.total_lines, 16);
    let func_bar = progress_bar(
        coverage.covered_functions.len(),
        coverage.all_functions.len(),
        16,
    );

    println!(
        "Lines:     {lc}{}/{}{RESET}  ({lc}{:.1}%{RESET})  {line_bar}",
        coverage.covered_lines.len(),
        coverage.total_lines,
        line_pct,
    );

    println!(
        "Functions: {fc}{}/{}{RESET}  ({fc}{:.1}%{RESET})  {func_bar}",
        coverage.covered_functions.len(),
        coverage.all_functions.len(),
        func_pct,
    );
    println!();

    // Show uncovered items
    let uncovered_lines = coverage.uncovered_lines();
    if !uncovered_lines.is_empty() {
        if detailed {
            println!(
                "{YELLOW}Uncovered Lines:{RESET} {}",
                uncovered_lines
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        } else {
            println!(
                "{YELLOW}Uncovered Lines:{RESET} {} lines",
                uncovered_lines.len()
            );
        }
        println!();
    }

    let uncovered_funcs = coverage.uncovered_functions();
    if !uncovered_funcs.is_empty() {
        if detailed {
            println!("{YELLOW}Uncovered Functions:{RESET}");
            for func in uncovered_funcs {
                println!("  {DIM}-{RESET} {}", func);
            }
        } else {
            println!(
                "{YELLOW}Uncovered Functions:{RESET} {}",
                uncovered_funcs.len()
            );
        }
        println!();
    }

    // Summary
    if coverage.total_lines == 0 {
        println!("{YELLOW}⚠ No executable code found{RESET}");
    } else if coverage.covered_lines.is_empty() {
        println!("{YELLOW}⚠ No tests found - 0% coverage{RESET}");
    } else if line_pct >= 80.0 {
        println!("{GREEN}✓ Good coverage!{RESET}");
    } else if line_pct >= 50.0 {
        println!("{YELLOW}⚠ Moderate coverage - consider adding more tests{RESET}");
    } else {
        println!("{BRIGHT_RED}✗ Low coverage - more tests needed{RESET}");
    }
}

/// Print JSON coverage output
pub(crate) fn print_json_coverage(coverage: &crate::bash_quality::coverage::CoverageReport) {
    use serde_json::json;

    let json_coverage = json!({
        "coverage": {
            "lines": {
                "total": coverage.total_lines,
                "covered": coverage.covered_lines.len(),
                "percent": coverage.line_coverage_percent(),
            },
            "functions": {
                "total": coverage.all_functions.len(),
                "covered": coverage.covered_functions.len(),
                "percent": coverage.function_coverage_percent(),
            },
            "uncovered_lines": coverage.uncovered_lines(),
            "uncovered_functions": coverage.uncovered_functions(),
        }
    });

    match serde_json::to_string_pretty(&json_coverage) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error serializing JSON: {}", e);
            std::process::exit(1);
        }
    }
}

/// Print HTML coverage output
pub(crate) fn print_html_coverage(
    coverage: &crate::bash_quality::coverage::CoverageReport,
    input: &Path,
    output: Option<&Path>,
) {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Coverage Report - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #333; }}
        .summary {{ background: #f5f5f5; padding: 15px; border-radius: 5px; }}
        .coverage {{ font-size: 24px; font-weight: bold; }}
        .good {{ color: #28a745; }}
        .medium {{ color: #ffc107; }}
        .poor {{ color: #dc3545; }}
        table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .covered {{ background-color: #d4edda; }}
        .uncovered {{ background-color: #f8d7da; }}
    </style>
</head>
<body>
    <h1>Coverage Report</h1>
    <h2>{}</h2>
    <div class="summary">
        <p><strong>Line Coverage:</strong> 
            <span class="coverage {}">{:.1}%</span> 
            ({}/{})</p>
        <p><strong>Function Coverage:</strong> 
            <span class="coverage {}">{:.1}%</span> 
            ({}/{})</p>
    </div>
    <h3>Uncovered Functions</h3>
    <ul>
        {}
    </ul>
</body>
</html>"#,
        input.display(),
        input.display(),
        coverage_class(coverage.line_coverage_percent()),
        coverage.line_coverage_percent(),
        coverage.covered_lines.len(),
        coverage.total_lines,
        coverage_class(coverage.function_coverage_percent()),
        coverage.function_coverage_percent(),
        coverage.covered_functions.len(),
        coverage.all_functions.len(),
        coverage
            .uncovered_functions()
            .iter()
            .map(|f| format!("<li>{}</li>", f))
            .collect::<Vec<_>>()
            .join("\n        ")
    );

    if let Some(output_path) = output {
        if let Err(e) = fs::write(output_path, html) {
            eprintln!("Error writing HTML report: {}", e);
            std::process::exit(1);
        }
        println!("HTML coverage report written to {}", output_path.display());
    } else {
        println!("{}", html);
    }
}

/// Print LCOV coverage output
pub(crate) fn print_lcov_coverage(
    coverage: &crate::bash_quality::coverage::CoverageReport,
    input: &Path,
) {
    println!("TN:");
    println!("SF:{}", input.display());

    // Function coverage
    for func in &coverage.all_functions {
        let covered = i32::from(coverage.covered_functions.contains(func));
        println!("FN:0,{}", func);
        println!("FNDA:{},{}", covered, func);
    }
    println!("FNF:{}", coverage.all_functions.len());
    println!("FNH:{}", coverage.covered_functions.len());

    // Line coverage
    for (line_num, &is_covered) in &coverage.line_coverage {
        let hit = i32::from(is_covered);
        println!("DA:{},{}", line_num, hit);
    }
    println!("LF:{}", coverage.total_lines);
    println!("LH:{}", coverage.covered_lines.len());

    println!("end_of_record");
}
