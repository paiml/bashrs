#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)] // Examples can use unwrap() for simplicity
                               // Quality Tools Demonstration
                               // Comprehensive example showing all bashrs quality tools on various shell file styles
                               //
                               // Run with: cargo run --example quality_tools_demo

use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

/// ANSI color codes for output
mod colors {
    pub(crate) const RESET: &str = "\x1b[0m";
    pub(crate) const BOLD: &str = "\x1b[1m";
    pub(crate) const GREEN: &str = "\x1b[32m";
    pub(crate) const BLUE: &str = "\x1b[34m";
    pub(crate) const YELLOW: &str = "\x1b[33m";
    pub(crate) const CYAN: &str = "\x1b[36m";
    pub(crate) const MAGENTA: &str = "\x1b[35m";
}

/// Sample shell files to test
const SAMPLE_FILES: &[(&str, &str)] = &[
    (
        "examples/sample_bashrc.sh",
        "User Configuration (.bashrc style)",
    ),
    ("examples/sample_zshrc.sh", "ZSH Configuration"),
    ("examples/sample_deploy.sh", "Deployment Automation Script"),
    ("examples/sample_installer.sh", "Software Installer Script"),
    ("examples/sample_ci.sh", "CI/CD Pipeline Script"),
];

fn print_header(text: &str) {
    println!(
        "\n{}{}═══════════════════════════════════════════════════════════════{}",
        colors::BOLD,
        colors::BLUE,
        colors::RESET
    );
    println!(
        "{}{}  {}  {}",
        colors::BOLD,
        colors::BLUE,
        text,
        colors::RESET
    );
    println!(
        "{}{}═══════════════════════════════════════════════════════════════{}",
        colors::BOLD,
        colors::BLUE,
        colors::RESET
    );
}

fn print_section(text: &str) {
    println!(
        "\n{}{}▸ {}{}",
        colors::BOLD,
        colors::CYAN,
        text,
        colors::RESET
    );
    println!(
        "{}{}───────────────────────────────────────────{}",
        colors::CYAN,
        colors::RESET,
        colors::RESET
    );
}

fn print_file_info(file_path: &str, description: &str) {
    println!(
        "\n{}{}📄 {} {}({})",
        colors::BOLD,
        colors::MAGENTA,
        description,
        colors::RESET,
        file_path
    );

    if let Ok(metadata) = fs::metadata(file_path) {
        let lines = fs::read_to_string(file_path)
            .map(|s| s.lines().count())
            .unwrap_or(0);
        println!("   Size: {} bytes | Lines: {}", metadata.len(), lines);
    }
}

fn run_command(args: &[&str]) -> Result<String> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--bin")
        .arg("bashrs")
        .arg("--")
        .args(args)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Ok(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[allow(dead_code)] // Demo function for future interactive examples
fn demo_parse(file_path: &str) -> Result<()> {
    print_section("PARSE - Syntax Validation");

    let output = run_command(&["parse", file_path])?;
    let lines: Vec<&str> = output.lines().collect();

    // Show first 15 lines of parse output
    for line in lines.iter().take(15) {
        println!("  {}", line);
    }

    if lines.len() > 15 {
        println!(
            "  {}... ({} more lines){}",
            colors::YELLOW,
            lines.len() - 15,
            colors::RESET
        );
    }

    Ok(())
}

fn demo_lint(file_path: &str) -> Result<()> {
    print_section("LINT - Code Quality Analysis");

    let output = run_command(&["lint", file_path])?;

    if output.contains("No issues found") {
        println!(
            "  {}✓ No issues found - Excellent code quality!{}",
            colors::GREEN,
            colors::RESET
        );
    } else {
        let lines: Vec<&str> = output.lines().collect();
        for line in lines.iter().take(20) {
            println!("  {}", line);
        }

        if lines.len() > 20 {
            println!(
                "  {}... ({} more lines){}",
                colors::YELLOW,
                lines.len() - 20,
                colors::RESET
            );
        }
    }

    Ok(())
}

fn demo_format(file_path: &str) -> Result<()> {
    print_section("FORMAT - Code Formatting");

    // Show first 10 lines of formatted output
    let output = run_command(&["format", file_path, "--check"])?;
    let lines: Vec<&str> = output.lines().collect();

    println!("  Formatted preview (first 10 lines):");
    for line in lines.iter().take(10) {
        println!("  {}", line);
    }

    if lines.len() > 10 {
        println!(
            "  {}... ({} more lines){}",
            colors::YELLOW,
            lines.len() - 10,
            colors::RESET
        );
    }

    Ok(())
}

fn demo_audit(file_path: &str) -> Result<()> {
    print_section("AUDIT - Comprehensive Quality Check");

    let output = run_command(&["audit", file_path])?;

    // Show audit results
    let lines: Vec<&str> = output.lines().collect();
    for line in lines.iter() {
        // Highlight key metrics
        if line.contains("Overall:") || line.contains("Score:") || line.contains("Lint:") {
            println!("  {}{}{}", colors::BOLD, line, colors::RESET);
        } else {
            println!("  {}", line);
        }
    }

    Ok(())
}

fn demo_coverage(file_path: &str) -> Result<()> {
    print_section("COVERAGE - Test Coverage Analysis");

    let output = run_command(&["coverage", file_path])?;
    let lines: Vec<&str> = output.lines().collect();

    // Show coverage summary
    for line in lines.iter().take(15) {
        if line.contains("Coverage:") || line.contains("Tests:") {
            println!("  {}{}{}", colors::BOLD, line, colors::RESET);
        } else {
            println!("  {}", line);
        }
    }

    if lines.len() > 15 {
        println!(
            "  {}... ({} more lines){}",
            colors::YELLOW,
            lines.len() - 15,
            colors::RESET
        );
    }

    Ok(())
}

fn process_file(file_path: &str, description: &str) -> Result<()> {
    print_file_info(file_path, description);

    if !Path::new(file_path).exists() {
        println!(
            "  {}⚠ File not found, skipping{}",
            colors::YELLOW,
            colors::RESET
        );
        return Ok(());
    }

    // Run all quality tools
    // Note: parse command not available yet, using lint/audit instead
    demo_lint(file_path)?;
    demo_format(file_path)?;
    demo_audit(file_path)?;
    demo_coverage(file_path)?;

    println!("\n{}", "─".repeat(70));

    Ok(())
}

fn main() -> Result<()> {
    print_header("Bashrs Quality Tools Demonstration");

    println!(
        "\n{}This example demonstrates all quality tools on various shell file styles:{}",
        colors::BOLD,
        colors::RESET
    );
    println!("  • LINT     - Identify code quality issues");
    println!("  • FORMAT   - Show formatted code");
    println!("  • AUDIT    - Comprehensive quality assessment");
    println!("  • COVERAGE - Test coverage analysis");

    println!(
        "\n{}Testing {} sample shell files:{}",
        colors::BOLD,
        SAMPLE_FILES.len(),
        colors::RESET
    );

    for (i, (path, desc)) in SAMPLE_FILES.iter().enumerate() {
        println!("  {}. {} - {}", i + 1, desc, path);
    }

    // Process each sample file
    for (file_path, description) in SAMPLE_FILES {
        if let Err(e) = process_file(file_path, description) {
            eprintln!(
                "{}Error processing {}: {}{}",
                colors::YELLOW,
                file_path,
                e,
                colors::RESET
            );
        }
    }

    print_header("Summary");

    println!(
        "\n{}All quality tools demonstrated successfully!{}",
        colors::GREEN,
        colors::RESET
    );
    println!("\n{}Key Takeaways:{}", colors::BOLD, colors::RESET);
    println!(
        "  1. {} files can use different styles (bash, zsh, POSIX)",
        colors::YELLOW
    );
    println!("  2. Quality tools work on all shell script types");
    println!("  3. Comprehensive analysis helps maintain code quality");
    println!("  4. Formatting ensures consistency across projects");
    println!("  5. Coverage tracking ensures good test coverage");

    println!("\n{}Next Steps:{}", colors::BOLD, colors::RESET);
    println!(
        "  • Try running: {}cargo run -- lint <your-script.sh>{}",
        colors::CYAN,
        colors::RESET
    );
    println!(
        "  • Format your code: {}cargo run -- format <your-script.sh>{}",
        colors::CYAN,
        colors::RESET
    );
    println!(
        "  • Get quality report: {}cargo run -- audit <your-script.sh>{}",
        colors::CYAN,
        colors::RESET
    );

    println!(
        "\n{}For more information, visit: https://github.com/paiml/bashrs{}",
        colors::BLUE,
        colors::RESET
    );

    Ok(())
}
