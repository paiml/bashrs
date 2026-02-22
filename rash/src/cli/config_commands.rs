use crate::cli::args::{ConfigCommands, ConfigOutputFormat};
use crate::cli::logic::generate_diff_lines;
use crate::models::{Error, Result};
use std::fs;
use std::path::Path;
use tracing::info;

pub(crate) fn handle_config_command(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Analyze { input, format } => {
            info!("Analyzing {}", input.display());
            config_analyze_command(&input, format)
        }
        ConfigCommands::Lint { input, format } => {
            info!("Linting {}", input.display());
            config_lint_command(&input, format)
        }
        ConfigCommands::Purify {
            input,
            output,
            fix,
            no_backup,
            dry_run,
        } => {
            info!("Purifying {}", input.display());
            config_purify_command(&input, output.as_deref(), fix, no_backup, dry_run)
        }
    }
}

pub(crate) fn config_analyze_command(input: &Path, format: ConfigOutputFormat) -> Result<()> {
    use crate::config::analyzer;

    let source = fs::read_to_string(input).map_err(Error::Io)?;
    let analysis = analyzer::analyze_config(&source, input.to_path_buf());

    match format {
        ConfigOutputFormat::Human => config_analyze_human(input, &analysis),
        ConfigOutputFormat::Json => config_analyze_json(input, &analysis),
    }

    Ok(())
}

fn config_analyze_human(input: &Path, analysis: &crate::config::ConfigAnalysis) {
    println!("Analysis: {}", input.display());
    println!(
        "=========={}=",
        "=".repeat(input.display().to_string().len())
    );
    println!();
    println!("Statistics:");
    println!("  - Lines: {}", analysis.line_count);
    println!("  - Complexity score: {}/10", analysis.complexity_score);
    println!("  - Config type: {:?}", analysis.config_type);
    println!();

    if !analysis.path_entries.is_empty() {
        println!("PATH Entries ({}):", analysis.path_entries.len());
        for entry in &analysis.path_entries {
            let marker = if entry.is_duplicate { "  ✗" } else { "  ✓" };
            println!("{}  Line {}: {}", marker, entry.line, entry.path);
        }
        println!();
    }

    if !analysis.performance_issues.is_empty() {
        println!(
            "Performance Issues ({}):",
            analysis.performance_issues.len()
        );
        for issue in &analysis.performance_issues {
            println!(
                "  - Line {}: {} (~{}ms)",
                issue.line, issue.command, issue.estimated_cost_ms
            );
            println!("    Suggestion: {}", issue.suggestion);
        }
        println!();
    }

    config_analyze_human_issues(&analysis.issues);
}

fn config_analyze_human_issues(issues: &[crate::config::ConfigIssue]) {
    if issues.is_empty() {
        println!("✓ No issues found");
        return;
    }
    println!("Issues Found: {}", issues.len());
    for issue in issues {
        let severity_marker = match issue.severity {
            crate::config::Severity::Error => "✗",
            crate::config::Severity::Warning => "⚠",
            crate::config::Severity::Info => "ℹ",
        };
        println!(
            "  {} [{}] Line {}: {}",
            severity_marker, issue.rule_id, issue.line, issue.message
        );
        if let Some(suggestion) = &issue.suggestion {
            println!("    → {}", suggestion);
        }
    }
}

fn config_analyze_json(input: &Path, analysis: &crate::config::ConfigAnalysis) {
    println!("{{");
    println!("  \"file\": \"{}\",", input.display());
    println!("  \"line_count\": {},", analysis.line_count);
    println!("  \"complexity_score\": {},", analysis.complexity_score);
    println!("  \"path_entries\": {},", analysis.path_entries.len());
    println!(
        "  \"performance_issues\": {},",
        analysis.performance_issues.len()
    );
    println!("  \"issues\": [");
    for (i, issue) in analysis.issues.iter().enumerate() {
        let comma = if i < analysis.issues.len() - 1 {
            ","
        } else {
            ""
        };
        println!("    {{");
        println!("      \"rule_id\": \"{}\",", issue.rule_id);
        println!("      \"line\": {},", issue.line);
        println!("      \"message\": \"{}\"", issue.message);
        println!("    }}{}", comma);
    }
    println!("  ]");
    println!("}}");
}

pub(crate) fn config_lint_command(input: &Path, format: ConfigOutputFormat) -> Result<()> {
    use crate::config::analyzer;

    // Read input file
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Analyze config
    let analysis = analyzer::analyze_config(&source, input.to_path_buf());

    // Output results
    match format {
        ConfigOutputFormat::Human => {
            if analysis.issues.is_empty() {
                println!("✓ No issues found in {}", input.display());
                return Ok(());
            }

            for issue in &analysis.issues {
                let severity = match issue.severity {
                    crate::config::Severity::Error => "error",
                    crate::config::Severity::Warning => "warning",
                    crate::config::Severity::Info => "info",
                };
                println!(
                    "{}:{}:{}: {}: {} [{}]",
                    input.display(),
                    issue.line,
                    issue.column,
                    severity,
                    issue.message,
                    issue.rule_id
                );
                if let Some(suggestion) = &issue.suggestion {
                    println!("  suggestion: {}", suggestion);
                }
            }
        }
        ConfigOutputFormat::Json => {
            println!("{{");
            println!("  \"file\": \"{}\",", input.display());
            println!("  \"issues\": [");
            for (i, issue) in analysis.issues.iter().enumerate() {
                let comma = if i < analysis.issues.len() - 1 {
                    ","
                } else {
                    ""
                };
                println!("    {{");
                println!("      \"rule_id\": \"{}\",", issue.rule_id);
                println!("      \"line\": {},", issue.line);
                println!("      \"column\": {},", issue.column);
                println!("      \"message\": \"{}\"", issue.message);
                println!("    }}{}", comma);
            }
            println!("  ]");
            println!("}}");
        }
    }

    // Exit with code 1 if there are warnings or errors
    if !analysis.issues.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

/// Check if output should go to stdout
pub(crate) fn should_output_to_stdout(output_path: &Path) -> bool {
    output_path.to_str() == Some("-")
}

/// Count duplicate PATH entries in analysis
pub(crate) fn count_duplicate_path_entries(analysis: &crate::config::ConfigAnalysis) -> usize {
    analysis
        .path_entries
        .iter()
        .filter(|e| e.is_duplicate)
        .count()
}

// generate_diff_lines moved to cli/logic.rs

/// Handle output to specific file or stdout
pub(crate) fn handle_output_to_file(output_path: &Path, purified: &str) -> Result<()> {
    if should_output_to_stdout(output_path) {
        // Output to stdout
        println!("{}", purified);
    } else {
        fs::write(output_path, purified).map_err(Error::Io)?;
        info!("Purified config written to {}", output_path.display());
    }
    Ok(())
}

/// Handle in-place fixing with backup
fn handle_inplace_fix(
    input: &Path,
    purified: &str,
    analysis: &crate::config::ConfigAnalysis,
    no_backup: bool,
) -> Result<()> {
    use chrono::Local;

    // Create backup unless --no-backup
    if !no_backup {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let backup_path = input.with_extension(format!("bak.{}", timestamp));
        fs::copy(input, &backup_path).map_err(Error::Io)?;
        info!("Backup: {}", backup_path.display());
    }

    // Write purified content
    fs::write(input, purified).map_err(Error::Io)?;

    let fixed_count = analysis.issues.len();
    println!("Applying {} fixes...", fixed_count);
    println!(
        "  ✓ Deduplicated {} PATH entries",
        count_duplicate_path_entries(analysis)
    );
    println!("✓ Done! {} has been purified.", input.display());

    if !no_backup {
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let backup_path = input.with_extension(format!("bak.{}", timestamp));
        println!(
            "\nTo rollback: cp {} {}",
            backup_path.display(),
            input.display()
        );
    }

    Ok(())
}

/// Handle dry-run mode (preview changes)
fn handle_dry_run(
    input: &Path,
    source: &str,
    purified: &str,
    analysis: &crate::config::ConfigAnalysis,
) {
    println!("Preview of changes to {}:", input.display());
    println!(
        "================================{}=",
        "=".repeat(input.display().to_string().len())
    );
    println!();

    if analysis.issues.is_empty() {
        println!("✓ No issues found - file is already clean!");
    } else {
        println!("Would fix {} issue(s):", analysis.issues.len());
        for issue in &analysis.issues {
            println!("  - {}: {}", issue.rule_id, issue.message);
        }
        println!();
        println!("--- {} (original)", input.display());
        println!("+++ {} (purified)", input.display());
        println!();

        // Simple diff output
        let diff_lines = generate_diff_lines(source, purified);
        for (line_num, orig, pure) in diff_lines {
            println!("-{}: {}", line_num, orig);
            println!("+{}: {}", line_num, pure);
        }

        println!();
        println!(
            "Apply fixes: bashrs config purify {} --fix",
            input.display()
        );
    }
}

fn config_purify_command(
    input: &Path,
    output: Option<&Path>,
    fix: bool,
    no_backup: bool,
    dry_run: bool,
) -> Result<()> {
    use crate::config::{analyzer, purifier};

    // Read input file
    let source = fs::read_to_string(input).map_err(Error::Io)?;

    // Analyze first
    let analysis = analyzer::analyze_config(&source, input.to_path_buf());

    // Purify
    let purified = purifier::purify_config(&source);

    // Determine mode
    if let Some(output_path) = output {
        handle_output_to_file(output_path, &purified)?;
    } else if fix && !dry_run {
        handle_inplace_fix(input, &purified, &analysis, no_backup)?;
    } else {
        handle_dry_run(input, &source, &purified, &analysis);
    }

    Ok(())
}
