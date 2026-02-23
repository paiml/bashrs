//! Corpus convergence, mining, fix gaps, org patterns, and schema validation.

use crate::cli::args::CorpusFormatArg;
use crate::models::{Error, Result};
use std::path::PathBuf;

pub(crate) fn corpus_converge_table() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::convergence;
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let entries = CorpusRunner::load_convergence_log(&log_path)
        .map_err(|e| Error::Internal(format!("Failed to read convergence log: {e}")))?;
    if entries.is_empty() {
        println!("No convergence history. Run `bashrs corpus run --log` first.");
        return Ok(());
    }

    println!("{BOLD}Multi-Corpus Convergence Table (\u{00a7}11.10.5){RESET}");
    println!();

    let table = convergence::format_convergence_table(&entries);
    for line in table.lines() {
        println!("  {line}");
    }

    Ok(())
}

/// Per-format delta between two iterations (§11.10.5).
pub(crate) fn corpus_converge_diff(from: Option<u32>, to: Option<u32>) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::convergence;
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let entries = CorpusRunner::load_convergence_log(&log_path)
        .map_err(|e| Error::Internal(format!("Failed to read convergence log: {e}")))?;
    if entries.len() < 2 {
        println!("Need at least 2 iterations for diff. Run `bashrs corpus run --log` more.");
        return Ok(());
    }

    let from_entry = match from {
        Some(n) => entries
            .iter()
            .find(|e| e.iteration == n)
            .ok_or_else(|| Error::Internal(format!("Iteration #{n} not found in log")))?,
        None => &entries[entries.len() - 2],
    };

    let to_entry = match to {
        Some(n) => entries
            .iter()
            .find(|e| e.iteration == n)
            .ok_or_else(|| Error::Internal(format!("Iteration #{n} not found in log")))?,
        None => entries
            .last()
            .ok_or_else(|| Error::Internal("convergence log is empty".into()))?,
    };

    let diff = convergence::compare_iterations(from_entry, to_entry);

    println!("{BOLD}Convergence Diff (\u{00a7}11.10.5){RESET}");
    println!();

    let table = convergence::format_iteration_diff(&diff);
    for line in table.lines() {
        // Colorize delta arrows
        let colored = line
            .replace('\u{2191}', &format!("{GREEN}\u{2191}{RESET}"))
            .replace('\u{2193}', &format!("{RED}\u{2193}{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

/// Per-format convergence status with trend (§11.10.5).
pub(crate) fn corpus_converge_status() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::convergence;
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let entries = CorpusRunner::load_convergence_log(&log_path)
        .map_err(|e| Error::Internal(format!("Failed to read convergence log: {e}")))?;
    if entries.is_empty() {
        println!("No convergence history. Run `bashrs corpus run --log` first.");
        return Ok(());
    }

    let statuses = convergence::convergence_status(&entries);

    println!("{BOLD}Per-Format Convergence Status (\u{00a7}11.10.5){RESET}");
    println!();

    let output = convergence::format_convergence_status(&statuses);
    for line in output.lines() {
        // Colorize trend arrows and status keywords
        let colored = line
            .replace("CONVERGED", &format!("{BRIGHT_GREEN}CONVERGED{RESET}"))
            .replace("REGRESSING", &format!("{BRIGHT_RED}REGRESSING{RESET}"))
            .replace("IMPROVING", &format!("{YELLOW}IMPROVING{RESET}"))
            .replace(
                "\u{2191} Improving",
                &format!("{GREEN}\u{2191} Improving{RESET}"),
            )
            .replace("\u{2192} Stable", &format!("{CYAN}\u{2192} Stable{RESET}"))
            .replace(
                "\u{2193} Regressing",
                &format!("{RED}\u{2193} Regressing{RESET}"),
            );
        println!("  {colored}");
    }

    Ok(())
}

/// Mine fix patterns from git history (§11.9.1).
pub(crate) fn corpus_mine(limit: usize) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::oip;
    use crate::corpus::registry::CorpusRegistry;

    // Run git log to find fix commits
    let output = std::process::Command::new("git")
        .args([
            "log",
            "--oneline",
            "--all",
            &format!("--max-count={limit}"),
            "--grep=fix:",
            "--format=%h|%as|%s",
        ])
        .output()
        .map_err(|e| Error::Internal(format!("Failed to run git log: {e}")))?;

    let log_text = String::from_utf8_lossy(&output.stdout);
    let mut commits = oip::parse_fix_commits(&log_text);

    // Cross-reference with corpus entries
    let registry = CorpusRegistry::load_full();
    let descriptions: Vec<String> = registry
        .entries
        .iter()
        .map(|e| e.description.clone())
        .collect();

    for commit in &mut commits {
        commit.has_corpus_entry = oip::has_matching_corpus_entry(&commit.message, &descriptions);
    }

    println!("{BOLD}OIP Fix Pattern Mining (\u{00a7}11.9){RESET}");
    println!();

    let table = oip::format_mine_table(&commits);
    for line in table.lines() {
        let colored = line
            .replace("\u{2713}", &format!("{GREEN}\u{2713}{RESET}"))
            .replace("\u{2717}", &format!("{RED}\u{2717}{RESET}"))
            .replace("HIGH", &format!("{BRIGHT_RED}HIGH{RESET}"))
            .replace("ASTTransform", &format!("{CYAN}ASTTransform{RESET}"))
            .replace(
                "SecurityVulnerabilities",
                &format!("{BRIGHT_RED}SecurityVulnerabilities{RESET}"),
            )
            .replace(
                "OperatorPrecedence",
                &format!("{YELLOW}OperatorPrecedence{RESET}"),
            );
        println!("  {colored}");
    }

    Ok(())
}

/// Find fix commits without regression corpus entries (§11.9.3).
pub(crate) fn corpus_fix_gaps(limit: usize) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::oip;
    use crate::corpus::registry::CorpusRegistry;

    // Run git log to find fix commits
    let output = std::process::Command::new("git")
        .args([
            "log",
            "--oneline",
            "--all",
            &format!("--max-count={limit}"),
            "--grep=fix:",
            "--format=%h|%as|%s",
        ])
        .output()
        .map_err(|e| Error::Internal(format!("Failed to run git log: {e}")))?;

    let log_text = String::from_utf8_lossy(&output.stdout);
    let mut commits = oip::parse_fix_commits(&log_text);

    // Cross-reference with corpus entries
    let registry = CorpusRegistry::load_full();
    let descriptions: Vec<String> = registry
        .entries
        .iter()
        .map(|e| e.description.clone())
        .collect();

    for commit in &mut commits {
        commit.has_corpus_entry = oip::has_matching_corpus_entry(&commit.message, &descriptions);
    }

    let gaps = oip::find_fix_gaps(&commits, &descriptions);

    println!("{BOLD}Fix-Driven Corpus Gaps (\u{00a7}11.9.3){RESET}");
    println!();

    if gaps.is_empty() {
        println!("  {GREEN}No gaps found!{RESET} All high/medium priority fix commits have corpus coverage.");
        return Ok(());
    }

    let table = oip::format_fix_gaps_table(&gaps);
    for line in table.lines() {
        let colored = line
            .replace("HIGH", &format!("{BRIGHT_RED}HIGH{RESET}"))
            .replace("MEDIUM", &format!("{YELLOW}MEDIUM{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

/// Cross-project defect pattern analysis (§11.9.4).
pub(crate) fn corpus_org_patterns() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::oip;

    let patterns = oip::known_org_patterns();

    println!("{BOLD}Cross-Project Defect Patterns (\u{00a7}11.9.4){RESET}");
    println!();

    let table = oip::format_org_patterns_table(&patterns);
    for line in table.lines() {
        let colored = line
            .replace("ASTTransform", &format!("{CYAN}ASTTransform{RESET}"))
            .replace(
                "ComprehensionBugs",
                &format!("{YELLOW}ComprehensionBugs{RESET}"),
            )
            .replace(
                "OperatorPrecedence",
                &format!("{YELLOW}OperatorPrecedence{RESET}"),
            )
            .replace(
                "ConfigurationErrors",
                &format!("{YELLOW}ConfigurationErrors{RESET}"),
            )
            .replace("TypeErrors", &format!("{YELLOW}TypeErrors{RESET}"))
            .replace(
                "IntegrationFailures",
                &format!("{YELLOW}IntegrationFailures{RESET}"),
            );
        println!("  {colored}");
    }

    // Show coverage summary
    println!("  {BOLD}Corpus Coverage:{RESET}");
    for p in &patterns {
        let coverage = if p.covered_entries.is_empty() {
            format!("{RED}\u{2717} No coverage{RESET}")
        } else {
            format!("{GREEN}\u{2713} {}{RESET}", p.covered_entries.join(", "))
        };
        println!("    {:<30} {}", p.name, coverage);
    }

    Ok(())
}


pub(crate) fn corpus_schema_validate() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::schema_enforcement;

    let registry = CorpusRegistry::load_full();
    let report = schema_enforcement::validate_corpus(&registry);

    println!("{BOLD}Formal Schema Enforcement (\u{00a7}11.8){RESET}");
    println!();

    let table = schema_enforcement::format_schema_report(&report);
    for line in table.lines() {
        let colored = line
            .replace("Bash", &format!("{CYAN}Bash{RESET}"))
            .replace("Makefile", &format!("{YELLOW}Makefile{RESET}"))
            .replace("Dockerfile", &format!("{GREEN}Dockerfile{RESET}"));
        println!("  {colored}");
    }

    // Layer coverage summary
    println!();
    println!("  {BOLD}Layer Coverage:{RESET}");
    for layer in &["L1:Lexical", "L2:Syntactic", "L3:Semantic"] {
        let passed = report
            .results
            .iter()
            .filter(|r| r.layers_passed.iter().any(|l| format!("{l}") == *layer))
            .count();
        let pct = if report.total_entries > 0 {
            (passed as f64 / report.total_entries as f64) * 100.0
        } else {
            0.0
        };
        let color = if pct >= 99.0 {
            GREEN
        } else if pct >= 90.0 {
            YELLOW
        } else {
            RED
        };
        println!(
            "    {:<16} {color}{}/{} ({:.1}%){RESET}",
            layer, passed, report.total_entries, pct,
        );
    }

    println!();
    let status = if report.total_violations == 0 {
        format!("{GREEN}\u{2713} All entries pass schema validation{RESET}")
    } else {
        format!(
            "{YELLOW}\u{26a0} {} violation(s) in {} entries{RESET}",
            report.total_violations,
            report.total_entries - report.valid_entries,
        )
    };
    println!("  {status}");

    Ok(())
}


pub(crate) fn corpus_grammar_errors() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::schema_enforcement;

    let registry = CorpusRegistry::load_full();
    let report = schema_enforcement::validate_corpus(&registry);

    println!("{BOLD}Grammar Violations by Category (\u{00a7}11.8.5){RESET}");
    println!();

    let table = schema_enforcement::format_grammar_errors(&report);
    for line in table.lines() {
        let colored = line
            .replace("GRAM-001", &format!("{CYAN}GRAM-001{RESET}"))
            .replace("GRAM-002", &format!("{CYAN}GRAM-002{RESET}"))
            .replace("GRAM-003", &format!("{YELLOW}GRAM-003{RESET}"))
            .replace("GRAM-004", &format!("{GREEN}GRAM-004{RESET}"))
            .replace("GRAM-005", &format!("{YELLOW}GRAM-005{RESET}"))
            .replace("GRAM-006", &format!("{CYAN}GRAM-006{RESET}"))
            .replace("GRAM-007", &format!("{GREEN}GRAM-007{RESET}"))
            .replace("GRAM-008", &format!("{YELLOW}GRAM-008{RESET}"));
        println!("  {colored}");
    }

    // Show fix pattern suggestions
    if !report.violations_by_category.is_empty() {
        println!();
        println!("  {BOLD}Fix Patterns:{RESET}");
        for (cat, count) in &report.violations_by_category {
            if *count > 0 {
                println!(
                    "    {} ({} violations): {}",
                    cat.code(),
                    count,
                    cat.fix_pattern(),
                );
            }
        }
    }

    Ok(())
}


pub(crate) fn corpus_format_grammar(format: CorpusFormatArg) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusFormat;
    use crate::corpus::schema_enforcement;

    let corpus_format = match format {
        CorpusFormatArg::Bash => CorpusFormat::Bash,
        CorpusFormatArg::Makefile => CorpusFormat::Makefile,
        CorpusFormatArg::Dockerfile => CorpusFormat::Dockerfile,
    };

    let spec = schema_enforcement::format_grammar_spec(corpus_format);

    println!("{BOLD}Formal Grammar Specification (\u{00a7}11.8){RESET}");
    println!();

    for line in spec.lines() {
        let colored = if line.contains("Validation Layers:") {
            format!("{BOLD}{line}{RESET}")
        } else if line.starts_with("  L") {
            let parts: Vec<&str> = line.splitn(2, '\u{2014}').collect();
            if parts.len() == 2 {
                format!("{CYAN}{}{RESET}\u{2014}{}", parts[0], parts[1])
            } else {
                format!("{CYAN}{line}{RESET}")
            }
        } else if line.contains("Grammar") {
            format!("{BOLD}{line}{RESET}")
        } else {
            line.to_string()
        };
        println!("  {colored}");
    }

    Ok(())
}
