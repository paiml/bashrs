//! Corpus configuration: domain analysis, tier management, quality gates, dataset export, and publishing.
use crate::cli::args::DatasetExportFormat;
use crate::models::{Config, Error, Result};
use std::path::{Path, PathBuf};
/// Display CWE taxonomy mapping for all bashrs linter rules (SSC v12 S14.2).
pub(crate) fn corpus_cwe_mapping(json: bool) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::cwe_mapping;

    if json {
        // JSON output for pipeline consumption
        let entries: Vec<serde_json::Value> = cwe_mapping::CWE_MAPPINGS
            .iter()
            .map(|m| {
                serde_json::json!({
                    "rule": m.rule,
                    "pattern": m.pattern,
                    "cwe": m.cwe,
                    "cwe_id": m.cwe_id,
                    "cvss_score": m.cvss_score,
                    "cvss_severity": m.cvss_severity.as_str(),
                    "owasp": m.owasp,
                })
            })
            .collect();
        let ood: Vec<serde_json::Value> = cwe_mapping::OOD_CWES
            .iter()
            .map(|o| {
                serde_json::json!({
                    "cwe": o.cwe,
                    "cwe_id": o.cwe_id,
                    "name": o.name,
                    "description": o.description,
                    "cvss_score": o.cvss_score,
                    "cvss_severity": o.cvss_severity.as_str(),
                })
            })
            .collect();
        let output = serde_json::json!({
            "linter_rules": entries,
            "ood_cwes": ood,
            "summary": {
                "total_rules": cwe_mapping::CWE_MAPPINGS.len(),
                "unique_cwes": cwe_mapping::linter_cwe_ids().len(),
                "ood_cwes": cwe_mapping::OOD_CWES.len(),
                "ood_disjoint": cwe_mapping::verify_ood_disjoint(),
            }
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Human-readable table
        println!("{BOLD}CWE Taxonomy Mapping (SSC v12 \u{00a7}14.2){RESET}");
        println!();
        let owasp_hdr = "OWASP";
        println!(
            "  {DIM}{:<8} {:<35} {:<10} {:>5} {:<10} {}{RESET}",
            "Rule", "Pattern", "CWE", "CVSS", "Severity", owasp_hdr
        );
        println!("  {}", "-".repeat(90));

        for m in cwe_mapping::CWE_MAPPINGS {
            let severity_color = match m.cvss_severity {
                cwe_mapping::CvssSeverity::Critical => RED,
                cwe_mapping::CvssSeverity::High => YELLOW,
                cwe_mapping::CvssSeverity::Medium => CYAN,
                cwe_mapping::CvssSeverity::Low => DIM,
                cwe_mapping::CvssSeverity::None => DIM,
            };
            println!(
                "  {:<8} {:<35} {:<10} {severity_color}{:>5.1}{RESET} {severity_color}{:<10}{RESET} {}",
                m.rule, m.pattern, m.cwe, m.cvss_score, m.cvss_severity, m.owasp
            );
        }

        println!();
        println!("{BOLD}OOD CWEs (eval-only, not in linter){RESET}");
        println!();
        for o in cwe_mapping::OOD_CWES {
            println!(
                "  {:<10} {:<40} {:>5.1} ({})",
                o.cwe, o.name, o.cvss_score, o.cvss_severity
            );
        }

        println!();
        println!("{DIM}{}{RESET}", cwe_mapping::summary());
    }

    Ok(())
}

/// Label external JSONL with linter findings + CWE mappings (SSC v12 pipeline).
///
/// Reads JSONL with "script" or "text" field, lints each entry, and outputs
/// labeled JSONL with safety classification, rule IDs, CWE mappings, and CVSS scores.
pub(crate) fn corpus_label(input: PathBuf, output: Option<PathBuf>) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::cwe_mapping;
    use crate::linter;

    let file = std::fs::File::open(&input)?;
    let reader = std::io::BufReader::new(file);

    let writer: Box<dyn std::io::Write> = if let Some(ref path) = output {
        Box::new(std::fs::File::create(path)?)
    } else {
        Box::new(std::io::stdout())
    };
    let mut buf = std::io::BufWriter::new(writer);

    let mut total = 0usize;
    let mut unsafe_count = 0usize;

    for line in std::io::BufRead::lines(reader) {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let entry: serde_json::Value = serde_json::from_str(&line)
            .map_err(|e| Error::Validation(format!("Invalid JSON on line {}: {e}", total + 1)))?;

        // Extract script text from "script", "text", "input", or "unsafe_script" field
        let script = entry
            .get("script")
            .or_else(|| entry.get("text"))
            .or_else(|| entry.get("input"))
            .or_else(|| entry.get("unsafe_script"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::Validation(format!(
                    "Line {}: missing 'script', 'text', 'input', or 'unsafe_script' field",
                    total + 1
                ))
            })?;

        // Lint the script
        let lint_result = linter::lint_shell(script);

        // Only SEC/DET/IDEM rules are safety-relevant
        let security_diags: Vec<&linter::Diagnostic> = lint_result
            .diagnostics
            .iter()
            .filter(|d| {
                d.code.starts_with("SEC") || d.code.starts_with("DET") || d.code.starts_with("IDEM")
            })
            .collect();

        let is_unsafe = !security_diags.is_empty();
        let label = if is_unsafe { 1 } else { 0 };

        // Build findings with CWE mappings
        let findings: Vec<serde_json::Value> = security_diags
            .iter()
            .map(|d| {
                let cwe_info = cwe_mapping::lookup_rule(&d.code);
                serde_json::json!({
                    "rule": d.code,
                    "message": d.message,
                    "cwe": cwe_info.map(|c| c.cwe).unwrap_or("unknown"),
                    "cwe_id": cwe_info.map(|c| c.cwe_id).unwrap_or(0),
                    "cvss_score": cwe_info.map(|c| c.cvss_score).unwrap_or(0.0),
                })
            })
            .collect();

        // Output labeled entry (preserving original fields)
        let mut labeled = entry.clone();
        if let Some(obj) = labeled.as_object_mut() {
            obj.insert("label".to_string(), serde_json::json!(label));
            obj.insert(
                "classification".to_string(),
                serde_json::json!(if is_unsafe { "unsafe" } else { "safe" }),
            );
            obj.insert("findings".to_string(), serde_json::json!(findings));
            obj.insert(
                "finding_count".to_string(),
                serde_json::json!(security_diags.len()),
            );
        }

        serde_json::to_writer(&mut buf, &labeled)?;
        std::io::Write::write_all(&mut buf, b"\n")?;

        total += 1;
        if is_unsafe {
            unsafe_count += 1;
        }
    }

    std::io::Write::flush(&mut buf)?;

    if output.is_some() {
        eprintln!("{BOLD}Label Summary{RESET}");
        eprintln!("  Total:  {total}");
        eprintln!(
            "  Safe:   {} ({:.1}%)",
            total - unsafe_count,
            100.0 * (total - unsafe_count) as f64 / total.max(1) as f64
        );
        eprintln!(
            "  Unsafe: {unsafe_count} ({:.1}%)",
            100.0 * unsafe_count as f64 / total.max(1) as f64
        );
    }

    Ok(())
}

/// Export corpus as ShellSafetyBench DPO-compatible JSONL (SSC v12 S14.4).
pub(crate) fn corpus_export_benchmark(output: Option<PathBuf>, limit: Option<usize>) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::benchmark_export;
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
    let (entries, summary) = benchmark_export::export_benchmark(&registry, limit);

    // Write JSONL
    let writer: Box<dyn std::io::Write> = if let Some(ref path) = output {
        Box::new(std::fs::File::create(path)?)
    } else {
        Box::new(std::io::stdout())
    };
    let mut buf = std::io::BufWriter::new(writer);
    for entry in &entries {
        serde_json::to_writer(&mut buf, entry)?;
        std::io::Write::write_all(&mut buf, b"\n")?;
    }
    std::io::Write::flush(&mut buf)?;

    // Print summary to stderr if outputting to file
    if output.is_some() {
        eprintln!("{BOLD}ShellSafetyBench Export Summary{RESET}");
        eprintln!("  Total:   {}", summary.total);
        eprintln!(
            "  Safe:    {} ({:.1}%)",
            summary.safe,
            100.0 * summary.safe as f64 / summary.total.max(1) as f64
        );
        eprintln!(
            "  Unsafe:  {} ({:.1}%)",
            summary.unsafe_count,
            100.0 * summary.unsafe_count as f64 / summary.total.max(1) as f64
        );
        eprintln!("  By lang:");
        for (lang, count) in &summary.by_lang {
            eprintln!("    {lang}: {count}");
        }
        eprintln!("  Unique CWEs: {}", summary.by_cwe.len());
        if let Some(path) = &output {
            eprintln!("  Written to: {}", path.display());
        }
    }

    Ok(())
}

pub(crate) fn corpus_domain_categories() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::domain_categories;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let stats = domain_categories::categorize_corpus(&registry, &score.results);

    println!("{BOLD}Domain-Specific Corpus Categories (\u{00a7}11.11){RESET}");
    println!();

    let report = domain_categories::format_categories_report(&stats);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("COMPLETE", &format!("{GREEN}COMPLETE{RESET}"))
            .replace("EMPTY", &format!("{RED}EMPTY{RESET}"))
            .replace("SPARSE", &format!("{YELLOW}SPARSE{RESET}"))
            .replace("PARTIAL", &format!("{CYAN}PARTIAL{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_domain_coverage() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::domain_categories;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let stats = domain_categories::categorize_corpus(&registry, &score.results);

    println!("{BOLD}Domain Coverage Analysis (\u{00a7}11.11){RESET}");
    println!();

    let report = domain_categories::format_domain_coverage(&stats, &score);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("COMPLETE", &format!("{GREEN}COMPLETE{RESET}"))
            .replace("EMPTY", &format!("{RED}EMPTY{RESET}"))
            .replace("SPARSE", &format!("{YELLOW}SPARSE{RESET}"))
            .replace("PARTIAL", &format!("{CYAN}PARTIAL{RESET}"))
            .replace("Coverage Gaps:", &format!("{YELLOW}Coverage Gaps:{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_domain_matrix() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::domain_categories;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let stats = domain_categories::categorize_corpus(&registry, &score.results);

    println!("{BOLD}Cross-Category Quality Matrix (\u{00a7}11.11.9){RESET}");
    println!();

    let report = domain_categories::format_quality_matrix(&stats);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("REQ", &format!("{GREEN}REQ{RESET}"))
            .replace("N/A", &format!("{DIM}N/A{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_tier_weights() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use crate::corpus::tier_analysis;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let analysis = tier_analysis::analyze_tiers(&registry, &score);

    println!("{BOLD}Tier-Weighted Corpus Scoring (\u{00a7}4.3){RESET}");
    println!();

    let report = tier_analysis::format_tier_weights(&analysis);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("100.0%", &format!("{GREEN}100.0%{RESET}"))
            .replace("Weighted Score:", &format!("{BOLD}Weighted Score:{RESET}"))
            .replace("Weight Effect:", &format!("{BOLD}Weight Effect:{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_tier_analysis() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use crate::corpus::tier_analysis;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let analysis = tier_analysis::analyze_tiers(&registry, &score);

    println!("{BOLD}Tier Difficulty Analysis (\u{00a7}4.3){RESET}");
    println!();

    let report = tier_analysis::format_tier_analysis(&analysis);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("No difference", &format!("{GREEN}No difference{RESET}"))
            .replace("Distribution:", &format!("{BOLD}Distribution:{RESET}"))
            .replace(
                "Scoring Comparison:",
                &format!("{BOLD}Scoring Comparison:{RESET}"),
            )
            .replace("Weight Impact", &format!("{BOLD}Weight Impact{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_tier_targets() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use crate::corpus::tier_analysis;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let analysis = tier_analysis::analyze_tiers(&registry, &score);

    println!("{BOLD}Tier Target Rate Comparison (\u{00a7}2.3/\u{00a7}4.3){RESET}");
    println!();

    let report = tier_analysis::format_tier_targets(&analysis);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("PASS", &format!("{GREEN}PASS{RESET}"))
            .replace("FAIL", &format!("{RED}FAIL{RESET}"))
            .replace("ALL TARGETS MET", &format!("{GREEN}ALL TARGETS MET{RESET}"))
            .replace("TARGETS NOT MET", &format!("{RED}TARGETS NOT MET{RESET}"))
            .replace("COMFORTABLE", &format!("{GREEN}COMFORTABLE{RESET}"))
            .replace("AT RISK", &format!("{YELLOW}AT RISK{RESET}"))
            .replace("MARGINAL", &format!("{YELLOW}MARGINAL{RESET}"))
            .replace("BELOW TARGET", &format!("{RED}BELOW TARGET{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_quality_gates() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::quality_gates;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);
    let history = CorpusRunner::load_convergence_log(&log_path).unwrap_or_default();
    let thresholds = quality_gates::QualityThresholds::default();
    let gates = quality_gates::check_quality_gates(&score, &history, &thresholds);

    println!("{BOLD}Corpus Quality Gates (\u{00a7}9 / \u{00a7}8.1){RESET}");
    println!();

    let report = quality_gates::format_quality_gates(&gates);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("PASS", &format!("{GREEN}PASS{RESET}"))
            .replace("FAIL", &format!("{RED}FAIL{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_metrics_check() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::quality_gates;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let start = std::time::Instant::now();
    let score = runner.run(&registry);
    let duration = start.elapsed();
    let history = CorpusRunner::load_convergence_log(&log_path).unwrap_or_default();
    let thresholds = quality_gates::PerformanceThresholds::default();
    let metrics = quality_gates::check_metrics(&score, duration, &history, &thresholds);

    println!("{BOLD}Corpus Performance Metrics (\u{00a7}9 / \u{00a7}8.2){RESET}");
    println!();

    let report = quality_gates::format_metrics_check(&metrics);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("PASS", &format!("{GREEN}PASS{RESET}"))
            .replace("FAIL", &format!("{RED}FAIL{RESET}"));
        println!("  {colored}");
    }


}

    include!("corpus_config_commands_part2_incl2.rs");
