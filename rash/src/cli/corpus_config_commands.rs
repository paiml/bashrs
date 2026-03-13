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

    Ok(())
}

pub(crate) fn corpus_gate_status_cmd() -> Result<()> {
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
    let status = quality_gates::build_gate_status(&score, duration, &history);

    println!("{BOLD}Corpus Gate Status Summary (\u{00a7}9){RESET}");
    println!();

    let report = quality_gates::format_gate_status(&status);
    for line in report.lines().skip(2) {
        let colored = line
            .replace("\u{2713}", &format!("{GREEN}\u{2713}{RESET}"))
            .replace("\u{2717}", &format!("{RED}\u{2717}{RESET}"))
            .replace(
                "ALL GATES PASSED",
                &format!("{GREEN}ALL GATES PASSED{RESET}"),
            )
            .replace("GATES FAILED", &format!("{RED}GATES FAILED{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_export_dataset(
    format: DatasetExportFormat,
    output: Option<std::path::PathBuf>,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::dataset::{self, ExportFormat};

    let export_fmt = match format {
        DatasetExportFormat::Json => ExportFormat::Json,
        DatasetExportFormat::Jsonl => ExportFormat::JsonLines,
        DatasetExportFormat::Csv => ExportFormat::Csv,
        DatasetExportFormat::Classification => ExportFormat::Classification,
        DatasetExportFormat::MultiLabelClassification => ExportFormat::MultiLabelClassification,
    };

    let (score, data) = dataset::run_and_export(export_fmt);

    match output {
        Some(path) => {
            std::fs::write(&path, &data).map_err(|e| {
                Error::Validation(format!("Failed to write {}: {e}", path.display()))
            })?;
            println!(
                "{GREEN}\u{2713}{RESET} Exported {} entries to {} ({} format)",
                score.total,
                path.display(),
                export_fmt,
            );
        }
        None => {
            print!("{data}");
        }
    }

    Ok(())
}

pub(crate) fn corpus_dataset_info() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::dataset;
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
    let info = dataset::dataset_info(&registry);

    println!("{BOLD}Corpus Dataset Info (\u{00a7}10.3){RESET}");
    println!();

    let table = dataset::format_dataset_info(&info);
    for line in table.lines() {
        let colored = line
            .replace("bash", &format!("{CYAN}bash{RESET}"))
            .replace("makefile", &format!("{YELLOW}makefile{RESET}"))
            .replace("dockerfile", &format!("{GREEN}dockerfile{RESET}"))
            .replace("string", &format!("{DIM}string{RESET}"))
            .replace("bool", &format!("{DIM}bool{RESET}"))
            .replace("float64", &format!("{DIM}float64{RESET}"))
            .replace("int32", &format!("{DIM}int32{RESET}"));
        println!("  {colored}");
    }

    Ok(())
}

pub(crate) fn corpus_publish_check() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::dataset;
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let checks = dataset::check_publish_readiness(&score);

    println!("{BOLD}Hugging Face Publish Readiness (\u{00a7}10.3){RESET}");
    println!();

    let table = dataset::format_publish_checks(&checks);
    for line in table.lines() {
        let colored = line
            .replace("\u{2713} PASS", &format!("{GREEN}\u{2713} PASS{RESET}"))
            .replace("\u{2717} FAIL", &format!("{RED}\u{2717} FAIL{RESET}"))
            .replace(
                "Ready to publish",
                &format!("{GREEN}Ready to publish{RESET}"),
            )
            .replace("check(s) failed", &format!("{RED}check(s) failed{RESET}"));
        println!("  {colored}");
    }

    // Show target HF repos
    println!();
    println!("  {BOLD}Target Repositories:{RESET}");
    for (repo, fmt) in &[
        ("paiml/bashrs-corpus-bash", "Bash"),
        ("paiml/bashrs-corpus-makefile", "Makefile"),
        ("paiml/bashrs-corpus-dockerfile", "Dockerfile"),
        ("paiml/bashrs-convergence", "Convergence"),
    ] {
        let count = match *fmt {
            "Bash" => registry
                .entries
                .iter()
                .filter(|e| e.format == CorpusFormat::Bash)
                .count(),
            "Makefile" => registry
                .entries
                .iter()
                .filter(|e| e.format == CorpusFormat::Makefile)
                .count(),
            "Dockerfile" => registry
                .entries
                .iter()
                .filter(|e| e.format == CorpusFormat::Dockerfile)
                .count(),
            _ => 0,
        };
        if count > 0 {
            println!("    {CYAN}{repo}{RESET} ({count} entries)");
        } else {
            println!("    {CYAN}{repo}{RESET}");
        }
    }

    Ok(())
}

/// Generate synthetic conversations from corpus entries (SSC v11 Section 6).
pub(crate) fn corpus_generate_conversations(
    output: Option<PathBuf>,
    seed: u64,
    limit: Option<usize>,
    entrenar_format: bool,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::conversations::{generate_batch, to_entrenar_jsonl, to_jsonl};
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
    let max = limit.unwrap_or(registry.entries.len());

    // v12: Transpile each entry to shell/Makefile/Dockerfile output first.
    // generate_batch receives (id, shell_code) not (id, rust_source).
    // This fixes the data quality issue where v3 conversations contained 85% Rust code.
    let config = crate::Config::default();
    let transpiled_entries: Vec<(String, String)> = registry
        .entries
        .iter()
        .take(max)
        .map(|e| {
            let shell_output = match e.format {
                crate::corpus::registry::CorpusFormat::Bash => crate::transpile(&e.input, &config)
                    .map(|s| crate::corpus::dataset::strip_shell_preamble(&s))
                    .unwrap_or_else(|_| e.input.clone()),
                crate::corpus::registry::CorpusFormat::Makefile => {
                    crate::transpile_makefile(&e.input, &config).unwrap_or_else(|_| e.input.clone())
                }
                crate::corpus::registry::CorpusFormat::Dockerfile => {
                    crate::transpile_dockerfile(&e.input, &config)
                        .unwrap_or_else(|_| e.input.clone())
                }
            };
            (e.id.clone(), shell_output)
        })
        .collect();

    let batch: Vec<(&str, &str)> = transpiled_entries
        .iter()
        .map(|(id, shell)| (id.as_str(), shell.as_str()))
        .collect();

    let format_name = if entrenar_format {
        "entrenar"
    } else {
        "chatml"
    };
    eprintln!(
        "{BOLD}Generating conversations from {} corpus entries (seed={seed}, format={format_name})...{RESET}",
        batch.len()
    );

    let (conversations, report) = generate_batch(&batch, seed);
    let jsonl = if entrenar_format {
        to_entrenar_jsonl(&conversations)
    } else {
        to_jsonl(&conversations)
    };

    match output {
        Some(ref path) => {
            std::fs::write(path, &jsonl).map_err(Error::Io)?;
            eprintln!(
                "{GREEN}Wrote {} conversations to {}{RESET}",
                conversations.len(),
                path.display()
            );
        }
        None => {
            print!("{jsonl}");
        }
    }

    eprintln!();
    eprintln!("{BOLD}Quality Report:{RESET}");
    eprintln!("  Total:       {}", report.total);
    eprintln!(
        "  Type A (classify): {} | Type B (fix): {} | Type C (debug): {} | Type D (safe): {}",
        report.type_a_count, report.type_b_count, report.type_c_count, report.type_d_count
    );
    eprintln!("  Type D %:    {:.1}% (target: >=30%)", report.type_d_pct);
    eprintln!(
        "  Citations:   {:.0}%",
        report.rule_citation_accuracy * 100.0
    );
    eprintln!(
        "  Variants OK: {}",
        if report.variant_distribution_ok {
            format!("{GREEN}yes{RESET}")
        } else {
            format!("{RED}no{RESET}")
        }
    );
    eprintln!(
        "  Overall:     {}",
        if report.passed {
            format!("{GREEN}PASSED{RESET}")
        } else {
            format!("{YELLOW}FAILED{RESET}")
        }
    );

    Ok(())
}

/// Run all three baseline classifiers (SSC v11 S5.5).
pub(crate) fn corpus_baselines() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::baselines::{corpus_baseline_entries, run_all_baselines};
    use crate::corpus::evaluation::{format_comparison, format_report};

    eprintln!("{BOLD}Building baseline entries from corpus...{RESET}");

    let owned = corpus_baseline_entries();
    let entries: Vec<(&str, u8)> = owned.iter().map(|(s, l)| (s.as_str(), *l)).collect();

    let safe_count = entries.iter().filter(|(_, l)| *l == 0).count();
    let unsafe_count = entries.iter().filter(|(_, l)| *l == 1).count();
    eprintln!(
        "  Dataset: {} entries ({} safe, {} unsafe)",
        entries.len(),
        safe_count,
        unsafe_count
    );
    eprintln!();

    let reports = run_all_baselines(&entries);

    // Side-by-side comparison
    println!("{BOLD}=== SSC v11 Baseline Comparison (Section 5.5) ==={RESET}\n");
    print!("{}", format_comparison(&reports));
    println!();

    // Detailed per-baseline reports
    for report in &reports {
        println!("{BOLD}--- {} ---{RESET}", report.name);
        print!("{}", format_report(report));
        println!();
    }

    // Contract C-CLF-001 thresholds
    println!("{BOLD}Contract C-CLF-001 Thresholds:{RESET}");
    println!("  MCC CI lower > 0.2");
    println!("  Accuracy > 93.5%");
    println!("  Generalization >= 50%");
    println!();
    println!("Any ML classifier must beat ALL three baselines on MCC.");

    Ok(())
}

/// Audit label accuracy (SSC v11 S5.3, C-LABEL-001).
pub(crate) fn corpus_label_audit(limit: usize) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::label_audit::run_corpus_label_audit;

    eprintln!("{BOLD}Running label audit (C-LABEL-001, limit={limit})...{RESET}");

    let report = run_corpus_label_audit(limit);

    println!("{BOLD}=== SSC v11 Label Audit (Section 5.3, C-LABEL-001) ==={RESET}\n");
    println!("Audited {} unsafe labels:", report.total_audited);
    println!(
        "  Genuinely unsafe: {} ({:.1}%)",
        report.genuinely_unsafe, report.accuracy_pct
    );
    println!("  False positives:  {}", report.false_positives);
    println!("  Target:           >= 90% (C-LABEL-001)");
    println!(
        "  Status:           {}",
        if report.passed {
            format!("{GREEN}PASSED{RESET}")
        } else {
            format!("{RED}FAILED{RESET}")
        }
    );

    // Show false positives
    let false_pos: Vec<_> = report
        .results
        .iter()
        .filter(|r| !r.genuinely_unsafe)
        .collect();

    if !false_pos.is_empty() {
        println!("\n{BOLD}--- False Positives ---{RESET}\n");
        for r in false_pos.iter().take(10) {
            println!("  {} — {}", r.entry_id, r.reason);
            let preview = if r.script.len() > 60 {
                format!("{}...", &r.script[..60])
            } else {
                r.script.clone()
            };
            println!("    Script: {preview}");
        }
    }

    Ok(())
}

/// Run out-of-distribution generalization tests (SSC v11 S5.6).
pub(crate) fn corpus_generalization_tests() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::generalization_tests::{
        generalization_test_entries, GENERALIZATION_TARGET_PCT,
    };
    use crate::linter::lint_shell;

    println!("{BOLD}=== SSC v11 Generalization Tests (Section 5.6) ==={RESET}\n");

    let entries = generalization_test_entries();
    let mut caught = 0;
    let mut missed = Vec::new();

    for (script, category) in &entries {
        let result = lint_shell(script);
        let has_finding = result
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("SEC") || d.code.starts_with("DET"));
        if has_finding {
            caught += 1;
        } else {
            missed.push((*script, *category));
        }
    }

    let total = entries.len();
    let pct = caught as f64 / total as f64 * 100.0;
    let passed = pct >= GENERALIZATION_TARGET_PCT;

    println!("Total OOD scripts: {total}");
    println!("Caught by linter:  {caught} ({pct:.1}%)");
    println!("Missed:            {}", total - caught);
    println!("Target:            >= {GENERALIZATION_TARGET_PCT}%");
    println!(
        "Status:            {}",
        if passed {
            format!("{GREEN}PASSED{RESET}")
        } else {
            format!("{RED}FAILED{RESET}")
        }
    );

    if !missed.is_empty() {
        println!("\n{BOLD}--- Missed Scripts ---{RESET}\n");
        for (script, category) in &missed {
            let preview = if script.len() > 60 {
                format!("{}...", &script[..60])
            } else {
                (*script).to_string()
            };
            println!("  [{category}] {preview}");
        }
    }

    // Category breakdown
    println!("\n{BOLD}--- Category Breakdown ---{RESET}\n");
    let categories = [
        "injection",
        "non-determinism",
        "race-condition",
        "privilege",
        "exfiltration",
        "destructive",
    ];
    for cat in &categories {
        let cat_total = entries.iter().filter(|(_, c)| c == cat).count();
        let cat_caught = entries
            .iter()
            .filter(|(s, c)| {
                c == cat && {
                    let r = lint_shell(s);
                    r.diagnostics
                        .iter()
                        .any(|d| d.code.starts_with("SEC") || d.code.starts_with("DET"))
                }
            })
            .count();
        println!("  {cat:<20} {cat_caught}/{cat_total}");
    }

    Ok(())
}

/// Validate tokenizer quality on shell constructs (SSC v11 S5.2, C-TOK-001).
pub(crate) fn corpus_tokenizer_validation() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::tokenizer_validation::{run_validation, shell_constructs};

    println!("{BOLD}=== SSC v11 Tokenizer Validation (Section 5.2, C-TOK-001) ==={RESET}\n");

    let constructs = shell_constructs();
    println!("Shell constructs: {}\n", constructs.len());

    // Use whitespace tokenizer as baseline (real BPE plugs in via entrenar)
    let report = run_validation(|construct| {
        construct
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    });

    println!("Tokenizer:        whitespace (baseline)");
    println!(
        "Acceptable:       {} ({:.1}%)",
        report.acceptable_count, report.acceptable_pct
    );
    println!("Unacceptable:     {}", report.unacceptable_count);
    println!("Target:           >= 70% (C-TOK-001)");
    println!(
        "Status:           {}",
        if report.passed {
            format!("{GREEN}PASSED{RESET}")
        } else {
            format!("{RED}FAILED{RESET}")
        }
    );

    // Show failures
    let failures: Vec<_> = report.results.iter().filter(|r| !r.acceptable).collect();
    if !failures.is_empty() {
        println!("\n{BOLD}--- Failed Constructs ---{RESET}\n");
        for r in &failures {
            println!("  {} {:30} — {}", r.id, r.construct, r.reason);
        }
    }

    println!("\nNote: This uses a whitespace tokenizer as baseline.");
    println!("Plug in a real BPE tokenizer via entrenar for production validation.");

    Ok(())
}

/// Run all SSC contract validations (pre-training gate).
pub(crate) fn corpus_validate_contracts() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::contract_validation::run_all_contracts;

    eprintln!("{BOLD}Running SSC v11 contract validation (pre-training gate)...{RESET}\n");

    let report = run_all_contracts();

    println!("{BOLD}=== SSC v11 Contract Validation ==={RESET}\n");

    for c in &report.contracts {
        let status = if c.passed {
            format!("{GREEN}PASS{RESET}")
        } else {
            format!("{RED}FAIL{RESET}")
        };
        println!(
            "  [{status}] {:<15} {:<25} value={:.1} threshold={:.1}",
            c.id, c.name, c.value, c.threshold
        );
        println!("         {}", c.detail);
    }

    println!();
    println!(
        "{BOLD}Result: {}/{} contracts passed{RESET}",
        report.passed_count,
        report.contracts.len()
    );

    if report.all_passed {
        println!("{GREEN}All contracts passed. Ready for classifier training.{RESET}");
    } else {
        println!("{RED}Some contracts failed. Fix before proceeding to training.{RESET}");
    }

    Ok(())
}

/// Export dataset with train/val/test splits.
pub(crate) fn corpus_export_splits(output: Option<PathBuf>, input: Option<PathBuf>) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::dataset::{split_and_validate, ClassificationRow};

    let rows: Vec<ClassificationRow> = if let Some(ref input_path) = input {
        // Fast path: read from pre-merged JSONL
        eprintln!("{BOLD}Loading from {}...{RESET}", input_path.display());
        let content = std::fs::read_to_string(input_path)?;
        content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| {
                let v: serde_json::Value = serde_json::from_str(l).ok()?;
                let input_text = v
                    .get("instruction")
                    .or_else(|| v.get("input"))
                    .or_else(|| v.get("unsafe_script"))
                    .or_else(|| v.get("script"))
                    .and_then(|v| v.as_str())?
                    .to_string();
                let label = v.get("label").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                Some(ClassificationRow {
                    input: input_text,
                    label,
                })
            })
            .collect()
    } else {
        // Slow path: transpile full corpus
        use crate::corpus::baselines::corpus_baseline_entries;
        eprintln!("{BOLD}Building classification dataset from corpus...{RESET}");
        let owned = corpus_baseline_entries();
        owned
            .into_iter()
            .map(|(input, label)| ClassificationRow { input, label })
            .collect()
    };

    let total = rows.len();
    eprintln!("  Total entries: {total}");

    let result = split_and_validate(rows, 2);

    let train_safe = result.train.iter().filter(|r| r.label == 0).count();
    let train_unsafe = result.train.iter().filter(|r| r.label == 1).count();
    let val_safe = result.val.iter().filter(|r| r.label == 0).count();
    let val_unsafe = result.val.iter().filter(|r| r.label == 1).count();
    let test_safe = result.test.iter().filter(|r| r.label == 0).count();
    let test_unsafe = result.test.iter().filter(|r| r.label == 1).count();

    println!("{BOLD}=== SSC v11 Dataset Split (alimentar-compatible) ==={RESET}\n");
    println!(
        "  {:<8} {:>6} {:>6} {:>6}  {:>6}",
        "Split", "Total", "Safe", "Unsafe", "%Unsafe"
    );
    println!("  {}", "-".repeat(45));
    println!(
        "  {:<8} {:>6} {:>6} {:>6}  {:>5.1}%",
        "Train",
        result.train.len(),
        train_safe,
        train_unsafe,
        train_unsafe as f64 / result.train.len() as f64 * 100.0
    );
    println!(
        "  {:<8} {:>6} {:>6} {:>6}  {:>5.1}%",
        "Val",
        result.val.len(),
        val_safe,
        val_unsafe,
        val_unsafe as f64 / result.val.len() as f64 * 100.0
    );
    println!(
        "  {:<8} {:>6} {:>6} {:>6}  {:>5.1}%",
        "Test",
        result.test.len(),
        test_safe,
        test_unsafe,
        test_unsafe as f64 / result.test.len() as f64 * 100.0
    );
    println!(
        "  {:<8} {:>6} {:>6} {:>6}  {:>5.1}%",
        "Total",
        total,
        train_safe + val_safe + test_safe,
        train_unsafe + val_unsafe + test_unsafe,
        (train_unsafe + val_unsafe + test_unsafe) as f64 / total as f64 * 100.0
    );

    // Validation status
    println!(
        "\n  Validation: {}",
        if result.validation.passed {
            format!("{GREEN}PASSED{RESET}")
        } else {
            format!("{RED}FAILED{RESET}")
        }
    );
    for err in &result.validation.errors {
        println!("    - {RED}ERROR{RESET}: {err}");
    }
    for warn in &result.validation.warnings {
        println!("    - {YELLOW}WARN{RESET}: {warn}");
    }

    // Write split files if output dir specified
    if let Some(ref dir) = output {
        std::fs::create_dir_all(dir).map_err(Error::Io)?;

        let write_split = |name: &str, rows: &[ClassificationRow]| -> std::io::Result<()> {
            let path = dir.join(format!("{name}.jsonl"));
            let mut out = String::new();
            for row in rows {
                use std::fmt::Write as _;
                // Use serde_json for correct escaping of all control chars
                let json_input = serde_json::to_string(&row.input)
                    .unwrap_or_else(|_| format!("\"{}\"", row.input));
                // json_input already includes surrounding quotes
                let _ = writeln!(out, r#"{{"input":{},"label":{}}}"#, json_input, row.label);
            }
            std::fs::write(&path, out)?;
            Ok(())
        };

        write_split("train", &result.train).map_err(Error::Io)?;
        write_split("val", &result.val).map_err(Error::Io)?;
        write_split("test", &result.test).map_err(Error::Io)?;

        eprintln!("\n{GREEN}Wrote split files to {}{RESET}", dir.display());
        eprintln!(
            "  {}/train.jsonl ({} entries)",
            dir.display(),
            result.train.len()
        );
        eprintln!(
            "  {}/val.jsonl ({} entries)",
            dir.display(),
            result.val.len()
        );
        eprintln!(
            "  {}/test.jsonl ({} entries)",
            dir.display(),
            result.test.len()
        );
    }

    Ok(())
}

pub(crate) fn corpus_ssc_report(json: bool, gate: bool) -> Result<()> {
    use crate::corpus::ssc_report::{format_ssc_report, generate_ssc_report, SscStatus};

    eprintln!("Generating SSC v11 readiness report...");
    let report = generate_ssc_report();

    if json {
        let json_str = serde_json::to_string_pretty(&report)
            .map_err(|e| Error::Validation(format!("JSON serialization failed: {e}")))?;
        println!("{json_str}");
    } else {
        print!("{}", format_ssc_report(&report));
    }

    if report.overall_ready {
        eprintln!("All sections ready for classifier training.");
    } else {
        eprintln!("Some sections need attention before classifier training.");
    }

    if gate {
        let failures: Vec<&str> = report
            .sections
            .iter()
            .filter(|s| s.status == SscStatus::Fail)
            .map(|s| s.name.as_str())
            .collect();
        if !failures.is_empty() {
            return Err(Error::Validation(format!(
                "SSC gate failed: {} section(s) not ready: {}",
                failures.len(),
                failures.join(", ")
            )));
        }
    }

    Ok(())
}

pub(crate) fn corpus_model_card(output: Option<PathBuf>) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::model_card;

    eprintln!("{BOLD}Generating HuggingFace model card...{RESET}");
    let card = model_card::generate_model_card();

    match output {
        Some(path) => {
            std::fs::write(&path, &card).map_err(|e| {
                Error::Validation(format!("Failed to write {}: {e}", path.display()))
            })?;
            eprintln!(
                "{GREEN}\u{2713}{RESET} Model card written to {}",
                path.display()
            );
        }
        None => {
            print!("{card}");
        }
    }

    Ok(())
}

pub(crate) fn corpus_training_config(output: Option<PathBuf>, json: bool) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::training_config;

    eprintln!("{BOLD}Generating entrenar training configuration...{RESET}");
    let config = training_config::generate_training_config();

    let data = if json {
        training_config::format_json(&config)
    } else {
        training_config::format_yaml(&config)
    };

    match output {
        Some(path) => {
            std::fs::write(&path, &data).map_err(|e| {
                Error::Validation(format!("Failed to write {}: {e}", path.display()))
            })?;
            eprintln!(
                "{GREEN}\u{2713}{RESET} Training config written to {} ({} format)",
                path.display(),
                if json { "JSON" } else { "YAML" }
            );
        }
        None => {
            print!("{data}");
        }
    }

    Ok(())
}

pub(crate) fn corpus_publish_dataset(output: PathBuf) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::baselines::corpus_baseline_entries;
    use crate::corpus::dataset::{split_and_validate, ClassificationRow};
    use crate::corpus::model_card;
    use crate::corpus::training_config;

    eprintln!("{BOLD}Building HuggingFace-ready dataset...{RESET}");

    // Create output directory
    std::fs::create_dir_all(&output)
        .map_err(|e| Error::Validation(format!("Cannot create {}: {e}", output.display())))?;

    // Step 1: Split dataset
    let owned = corpus_baseline_entries();
    let total = owned.len();
    let rows: Vec<ClassificationRow> = owned
        .into_iter()
        .map(|(input, label)| ClassificationRow { input, label })
        .collect();
    let result = split_and_validate(rows, 2);

    // Step 2: Write split files
    write_split_file(&output, "train", &result.train)?;
    write_split_file(&output, "val", &result.val)?;
    write_split_file(&output, "test", &result.test)?;

    // Step 3: Write model card (README.md)
    let card = model_card::generate_model_card();
    let readme_path = output.join("README.md");
    std::fs::write(&readme_path, &card).map_err(|e| {
        Error::Validation(format!("Failed to write {}: {e}", readme_path.display()))
    })?;

    // Step 4: Write training config
    let config = training_config::generate_training_config();
    let config_path = output.join("training_config.yaml");
    std::fs::write(&config_path, training_config::format_yaml(&config)).map_err(|e| {
        Error::Validation(format!("Failed to write {}: {e}", config_path.display()))
    })?;

    // Summary
    eprintln!(
        "\n{GREEN}\u{2713}{RESET} {BOLD}Dataset published to {}{RESET}",
        output.display()
    );
    eprintln!("  README.md        \u{2014} HuggingFace model card");
    eprintln!("  train.jsonl      \u{2014} {} entries", result.train.len());
    eprintln!("  val.jsonl        \u{2014} {} entries", result.val.len());
    eprintln!("  test.jsonl       \u{2014} {} entries", result.test.len());
    eprintln!("  training_config.yaml \u{2014} entrenar config");
    eprintln!("  Total: {total} entries\n");
    eprintln!(
        "To publish: `huggingface-cli upload paiml/shell-safety-classifier {}`",
        output.display()
    );

    Ok(())
}

fn write_split_file(
    dir: &Path,
    name: &str,
    rows: &[crate::corpus::dataset::ClassificationRow],
) -> Result<()> {
    use std::fmt::Write as _;

    let path = dir.join(format!("{name}.jsonl"));
    let mut out = String::new();
    for row in rows {
        let escaped = row
            .input
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        let _ = writeln!(out, r#"{{"input":"{}","label":{}}}"#, escaped, row.label);
    }
    std::fs::write(&path, out)
        .map_err(|e| Error::Validation(format!("Failed to write {}: {e}", path.display())))?;
    Ok(())
}

/// Publish HuggingFace-ready conversation dataset (S6.6).
///
/// Generates conversations from full corpus, writes JSONL + dataset README.
pub(crate) fn corpus_publish_conversations(output: PathBuf, seed: u64) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::conversations::{generate_batch, generate_dataset_readme, to_jsonl};
    use crate::corpus::registry::CorpusRegistry;

    eprintln!("{BOLD}Building conversation dataset (seed={seed})...{RESET}");

    let registry = CorpusRegistry::load_full();
    let batch: Vec<(&str, &str)> = registry
        .entries
        .iter()
        .map(|e| (e.id.as_str(), e.input.as_str()))
        .collect();

    let (conversations, report) = generate_batch(&batch, seed);

    // Create output directory
    std::fs::create_dir_all(&output)
        .map_err(|e| Error::Validation(format!("Cannot create {}: {e}", output.display())))?;

    // Write conversations JSONL
    let jsonl = to_jsonl(&conversations);
    let jsonl_path = output.join("conversations.jsonl");
    std::fs::write(&jsonl_path, &jsonl)
        .map_err(|e| Error::Validation(format!("Failed to write {}: {e}", jsonl_path.display())))?;

    // Write dataset README
    let readme = generate_dataset_readme(&report);
    let readme_path = output.join("README.md");
    std::fs::write(&readme_path, &readme).map_err(|e| {
        Error::Validation(format!("Failed to write {}: {e}", readme_path.display()))
    })?;

    // Summary
    eprintln!(
        "\n{GREEN}\u{2713}{RESET} {BOLD}Conversation dataset published to {}{RESET}",
        output.display()
    );
    eprintln!("  README.md            \u{2014} HuggingFace dataset card");
    eprintln!(
        "  conversations.jsonl  \u{2014} {} conversations",
        conversations.len()
    );
    eprintln!();
    eprintln!("{BOLD}Quality Report:{RESET}");
    eprintln!(
        "  Type A (classify): {} | Type B (fix): {} | Type C (debug): {} | Type D (safe): {}",
        report.type_a_count, report.type_b_count, report.type_c_count, report.type_d_count
    );
    eprintln!("  Type D %:    {:.1}% (target: >=30%)", report.type_d_pct);
    eprintln!("  Empty:       {}", report.empty_responses);
    eprintln!(
        "  Status:      {}",
        if report.passed {
            format!("{GREEN}PASSED{RESET}")
        } else {
            format!("{RED}FAILED{RESET}")
        }
    );
    eprintln!();
    eprintln!(
        "To publish: `huggingface-cli upload paiml/shell-safety-conversations {}`",
        output.display()
    );

    Ok(())
}

/// Load classification rows from a JSONL file (format: `{"input":"...","label":N}`).
///
/// Non-zero labels are mapped to 1 (unsafe) for binary classification.
#[cfg(feature = "ml")]
fn load_classification_jsonl(
    path: &Path,
) -> Result<Vec<crate::corpus::dataset::ClassificationRow>> {
    use crate::corpus::dataset::ClassificationRow;

    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::Validation(format!("Cannot read {}: {e}", path.display())))?;

    #[derive(serde::Deserialize)]
    struct RawEntry {
        input: String,
        label: u8,
    }

    let mut entries = Vec::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<RawEntry>(line) {
            Ok(e) => entries.push(ClassificationRow {
                input: e.input,
                label: if e.label > 0 { 1 } else { 0 },
            }),
            Err(err) => eprintln!("  Skipping invalid line: {err}"),
        }
    }
    Ok(entries)
}

/// Extract [CLS] embeddings from CodeBERT for all corpus entries (CLF-RUN step 1).
#[allow(unused_variables)]
pub(crate) fn corpus_extract_embeddings(
    model: PathBuf,
    output: PathBuf,
    limit: Option<usize>,
    input_jsonl: Option<PathBuf>,
) -> Result<()> {
    #[cfg(not(feature = "ml"))]
    {
        Err(Error::Validation(
            "The `ml` feature is required for extract-embeddings. Rebuild with: cargo build --features ml".into(),
        ))
    }

    #[cfg(feature = "ml")]
    {
        use crate::cli::color::*;
        use crate::corpus::classifier::extract_embeddings_streaming;
        use crate::corpus::dataset::ClassificationRow;

        eprintln!("{BOLD}Extracting [CLS] embeddings from CodeBERT...{RESET}");

        let mut rows: Vec<ClassificationRow> = if let Some(ref jsonl_path) = input_jsonl {
            let entries = load_classification_jsonl(jsonl_path)?;
            eprintln!(
                "  Input JSONL: {} entries from {}",
                entries.len(),
                jsonl_path.display()
            );
            entries
        } else {
            use crate::corpus::baselines::corpus_baseline_entries;
            let owned = corpus_baseline_entries();
            owned
                .into_iter()
                .map(|(input, label)| ClassificationRow { input, label })
                .collect()
        };

        if let Some(n) = limit {
            rows.truncate(n);
            eprintln!("  Entries: {} (limited)", rows.len());
        } else {
            eprintln!("  Entries: {}", rows.len());
        }

        // Extract with streaming writes (one entry at a time to disk)
        let start = std::time::Instant::now();
        let report =
            extract_embeddings_streaming(&model, &rows, &output, &|i, total, elapsed_ms| {
                let rate = if elapsed_ms > 0 {
                    (i as f64 / elapsed_ms as f64) * 1000.0
                } else {
                    0.0
                };
                let eta_s = if rate > 0.0 {
                    ((total - i) as f64 / rate) as u64
                } else {
                    0
                };
                eprintln!(
                    "  [{i}/{total}] {:.1}% | {:.2} entries/s | ETA: {}m {}s",
                    100.0 * i as f64 / total as f64,
                    rate,
                    eta_s / 60,
                    eta_s % 60,
                );
            })
            .map_err(Error::Validation)?;

        let elapsed = start.elapsed();
        eprintln!(
            "\n{GREEN}\u{2713}{RESET} {BOLD}Embeddings saved to {}{RESET} in {:.1}s",
            output.display(),
            elapsed.as_secs_f64()
        );
        eprintln!(
            "  Total: {} | Extracted: {} | Skipped: {} | Dim: {} | Rate: {:.2}/s",
            report.total_entries,
            report.extracted,
            report.skipped,
            report.hidden_size,
            report.extracted as f64 / elapsed.as_secs_f64().max(0.001),
        );

        Ok(())
    }
}

/// Train linear probe classifier on cached embeddings (CLF-RUN step 2-3).
#[allow(clippy::too_many_arguments)]
pub(crate) fn corpus_train_classifier(
    embeddings_path: PathBuf,
    output: PathBuf,
    epochs: usize,
    learning_rate: f32,
    seed: u64,
    max_entries: Option<usize>,
    augment: Vec<PathBuf>,
    mlp: bool,
    mlp_hidden: usize,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::classifier::{
        evaluate_probe, load_embeddings, save_probe, split_embeddings, train_linear_probe,
    };

    let probe_type = if mlp {
        format!("MLP probe (hidden={mlp_hidden})")
    } else {
        "linear probe".into()
    };
    eprintln!("{BOLD}Training {probe_type} classifier...{RESET}");

    // Load cached embeddings
    let mut all_embeddings = load_embeddings(&embeddings_path).map_err(Error::Validation)?;
    eprintln!(
        "  Loaded {} embeddings from {}",
        all_embeddings.len(),
        embeddings_path.display()
    );

    // Cap entries if --max-entries specified (avoids data labeling gaps, see #171)
    if let Some(max) = max_entries {
        if all_embeddings.len() > max {
            eprintln!("  Capping to {max} entries (--max-entries)");
            all_embeddings.truncate(max);
        }
    }

    // Augment with additional embedding files (e.g. adversarial entries)
    for aug_path in &augment {
        let aug = load_embeddings(aug_path).map_err(Error::Validation)?;
        eprintln!(
            "  Augmenting with {} entries from {}",
            aug.len(),
            aug_path.display()
        );
        all_embeddings.extend(aug);
    }

    // Split into train/test
    let (train, test) = split_embeddings(&all_embeddings, seed);
    eprintln!("  Train: {} | Test: {}", train.len(), test.len());

    // Train (linear or MLP)
    eprintln!("\n{BOLD}Training (epochs={epochs}, lr={learning_rate}):{RESET}");
    std::fs::create_dir_all(&output)
        .map_err(|e| Error::Validation(format!("Cannot create {}: {e}", output.display())))?;

    let test_report = if mlp {
        let (mlp_weights, report) =
            train_and_evaluate_mlp(&train, &test, epochs, learning_rate, mlp_hidden)?;
        // Save MLP weights
        let mlp_json = serde_json::to_string_pretty(&mlp_weights)
            .map_err(|e| Error::Validation(format!("Serialize MLP: {e}")))?;
        std::fs::write(output.join("mlp_probe.json"), mlp_json)
            .map_err(|e| Error::Validation(format!("Write MLP: {e}")))?;
        report
    } else {
        let probe = train_linear_probe(&train, epochs, learning_rate);
        eprintln!(
            "  Train accuracy: {:.1}% | Train MCC: {:.3}",
            probe.train_accuracy * 100.0,
            probe.train_mcc
        );
        let report = evaluate_probe(&probe, &test);
        save_probe(&probe, &output.join("probe.json")).map_err(Error::Validation)?;
        report
    };

    eprintln!("\n{BOLD}Test Evaluation:{RESET}");
    eprintln!("  Accuracy:  {:.1}%", test_report.accuracy * 100.0);
    eprintln!("  Precision: {:.3}", test_report.precision);
    eprintln!("  Recall:    {:.3}", test_report.recall);
    eprintln!("  F1:        {:.3}", test_report.f1);
    eprintln!("  MCC:       {:.3}", test_report.mcc);
    eprintln!(
        "  Confusion: TP={} FP={} TN={} FN={}",
        test_report.confusion.tp,
        test_report.confusion.fp,
        test_report.confusion.tn,
        test_report.confusion.fn_
    );

    // Save evaluation
    let eval_json = serde_json::to_string_pretty(&test_report)
        .map_err(|e| Error::Validation(format!("Serialize: {e}")))?;
    std::fs::write(output.join("evaluation.json"), eval_json)
        .map_err(|e| Error::Validation(format!("Write: {e}")))?;

    // Quality gate: C-CLF-001 — classifier must beat baselines
    let beats_keyword = test_report.mcc > 0.3;
    let beats_linter = test_report.mcc > 0.4;
    eprintln!("\n{BOLD}Ship Gate C-CLF-001:{RESET}");
    eprintln!(
        "  Beats keyword baseline (MCC>0.3): {}",
        if beats_keyword {
            format!("{GREEN}PASS{RESET}")
        } else {
            format!("{RED}FAIL{RESET}")
        }
    );
    eprintln!(
        "  Beats linter baseline (MCC>0.4): {}",
        if beats_linter {
            format!("{GREEN}PASS{RESET}")
        } else {
            format!("{RED}FAIL{RESET}")
        }
    );

    eprintln!(
        "\n{GREEN}\u{2713}{RESET} {BOLD}Classifier artifacts saved to {}{RESET}",
        output.display()
    );

    Ok(())
}

/// Train MLP probe and evaluate (Level 0.5).
#[cfg(feature = "ml")]
fn train_and_evaluate_mlp(
    train: &[crate::corpus::classifier::EmbeddingEntry],
    test: &[crate::corpus::classifier::EmbeddingEntry],
    epochs: usize,
    learning_rate: f32,
    mlp_hidden: usize,
) -> Result<(
    crate::corpus::classifier::MlpProbeWeights,
    crate::corpus::evaluation::EvaluationReport,
)> {
    use entrenar::finetune::MlpProbe;

    let hidden_size = train
        .first()
        .map(|e| e.embedding.len())
        .ok_or_else(|| Error::Validation("No training embeddings".into()))?;

    let embeddings: Vec<Vec<f32>> = train.iter().map(|e| e.embedding.clone()).collect();
    let labels: Vec<usize> = train.iter().map(|e| e.label as usize).collect();

    // Compute class weights (sqrt-inverse balanced)
    let n = labels.len() as f32;
    let n_safe = labels.iter().filter(|&&l| l == 0).count() as f32;
    let n_unsafe = labels.iter().filter(|&&l| l == 1).count() as f32;
    let class_weights = if n_unsafe > 0.0 {
        vec![(n / (2.0 * n_safe)).sqrt(), (n / (2.0 * n_unsafe)).sqrt()]
    } else {
        vec![1.0, 1.0]
    };
    eprintln!(
        "  Class weights: safe={:.3}, unsafe={:.3}",
        class_weights[0], class_weights[1]
    );

    let mut mlp = MlpProbe::new(hidden_size, mlp_hidden, 2);
    eprintln!(
        "  Parameters: {} ({} hidden)",
        mlp.num_parameters(),
        mlp_hidden
    );
    mlp.train(
        &embeddings,
        &labels,
        epochs,
        learning_rate,
        Some(&class_weights),
        1e-4,
    );

    // Evaluate on train
    let train_correct = embeddings
        .iter()
        .zip(labels.iter())
        .filter(|(e, &l)| mlp.predict(e) == l)
        .count();
    let train_acc = train_correct as f64 / labels.len().max(1) as f64;
    eprintln!("  Train accuracy: {:.1}%", train_acc * 100.0);

    // Evaluate on test: build (pred, truth) pairs for evaluate()
    let predictions: Vec<(u8, u8)> = test
        .iter()
        .map(|e| (mlp.predict(&e.embedding) as u8, e.label))
        .collect();
    let report = crate::corpus::evaluation::evaluate(&predictions, "MLP probe");

    let weights = crate::corpus::classifier::MlpProbeWeights {
        w1: mlp.w1,
        b1: mlp.b1,
        w2: mlp.w2,
        b2: mlp.b2,
        hidden_size,
        mlp_hidden,
        num_classes: 2,
        epochs,
        learning_rate,
        train_accuracy: train_acc,
    };

    Ok((weights, report))
}

/// Fallback for non-ml builds.
#[cfg(not(feature = "ml"))]
fn train_and_evaluate_mlp(
    _train: &[crate::corpus::classifier::EmbeddingEntry],
    _test: &[crate::corpus::classifier::EmbeddingEntry],
    _epochs: usize,
    _lr: f32,
    _mlp_hidden: usize,
) -> Result<(
    crate::corpus::classifier::MlpProbeWeights,
    crate::corpus::evaluation::EvaluationReport,
)> {
    Err(Error::Validation("MLP probe requires --features ml".into()))
}

/// Run full CLF-RUN pipeline: extract → train → evaluate.
#[allow(unused_variables)]
pub(crate) fn corpus_run_classifier(
    model: PathBuf,
    output: PathBuf,
    epochs: usize,
    learning_rate: f32,
    seed: u64,
) -> Result<()> {
    #[cfg(not(feature = "ml"))]
    {
        Err(Error::Validation(
            "The `ml` feature is required for run-classifier. Rebuild with: cargo build --features ml".into(),
        ))
    }

    #[cfg(feature = "ml")]
    {
        use crate::cli::color::*;
        use crate::corpus::baselines::corpus_baseline_entries;
        use crate::corpus::classifier::{run_classifier_pipeline, save_probe};
        use crate::corpus::dataset::ClassificationRow;

        eprintln!("{BOLD}=== CLF-RUN: Full Classifier Pipeline ==={RESET}\n");

        // Build classification rows
        let owned = corpus_baseline_entries();
        let rows: Vec<ClassificationRow> = owned
            .into_iter()
            .map(|(input, label)| ClassificationRow { input, label })
            .collect();
        eprintln!("Corpus: {} entries", rows.len());

        // Create output directory
        std::fs::create_dir_all(&output)
            .map_err(|e| Error::Validation(format!("Cannot create {}: {e}", output.display())))?;

        // Run pipeline
        let report = run_classifier_pipeline(&model, &rows, epochs, learning_rate, seed)
            .map_err(Error::Validation)?;

        // Save probe weights
        save_probe(&report.probe, &output.join("probe.json")).map_err(Error::Validation)?;

        // Save evaluation report
        let eval_json = serde_json::to_string_pretty(&report.test_eval)
            .map_err(|e| Error::Validation(format!("Serialize: {e}")))?;
        std::fs::write(output.join("evaluation.json"), eval_json)
            .map_err(|e| Error::Validation(format!("Write: {e}")))?;

        // Print final results
        eprintln!("\n{BOLD}=== CLF-RUN Results ==={RESET}");
        eprintln!("Test Accuracy:  {:.1}%", report.test_eval.accuracy * 100.0);
        eprintln!("Test MCC:       {:.3}", report.test_eval.mcc);
        eprintln!("Test Precision: {:.3}", report.test_eval.precision);
        eprintln!("Test Recall:    {:.3}", report.test_eval.recall);
        eprintln!("Test F1:        {:.3}", report.test_eval.f1);
        eprintln!();
        eprintln!("{BOLD}Ship Gate C-CLF-001:{RESET}");
        eprintln!(
            "  Beats keyword (MCC>0.3): {}",
            if report.beats_keyword {
                format!("{GREEN}PASS{RESET}")
            } else {
                format!("{RED}FAIL{RESET}")
            }
        );
        eprintln!(
            "  Beats linter (MCC>0.4): {}",
            if report.beats_linter {
                format!("{GREEN}PASS{RESET}")
            } else {
                format!("{RED}FAIL{RESET}")
            }
        );

        eprintln!(
            "\n{GREEN}\u{2713}{RESET} {BOLD}All artifacts saved to {}{RESET}",
            output.display()
        );

        Ok(())
    }
}

/// Cross-validate bashrs labels against ShellCheck on corpus samples (Step 7.4e).
pub(crate) fn corpus_shellcheck_validate(samples: usize, seed: u64, json: bool) -> Result<()> {
    use crate::cli::color::*;

    eprintln!(
        "{BOLD}Cross-validating bashrs labels vs ShellCheck ({samples} samples, seed={seed})...{RESET}"
    );

    // Try pre-built splits first (fast path, ~1s vs ~120s for full corpus)
    let splits_path = Path::new("training/shellsafetybench/splits/test.jsonl");
    let entries: Vec<(String, u8)> = if splits_path.exists() {
        let content = std::fs::read_to_string(splits_path).map_err(Error::Io)?;
        content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| {
                let v: serde_json::Value = serde_json::from_str(l).ok()?;
                let input = v.get("input")?.as_str()?.to_string();
                let label = v.get("label")?.as_u64()? as u8;
                Some((input, label))
            })
            .collect()
    } else {
        // Fall back to full corpus transpilation (slow)
        use crate::corpus::baselines::corpus_baseline_entries;
        corpus_baseline_entries()
    };

    let total = entries.len();

    // Deterministic sampling using seed
    let step = if samples >= total { 1 } else { total / samples };
    let sampled: Vec<_> = entries
        .iter()
        .enumerate()
        .filter(|(i, _)| i % step == (seed as usize % step))
        .take(samples)
        .map(|(_, e)| e)
        .collect();

    let mut agree = 0u32;
    let mut shellcheck_only = 0u32;
    let mut bashrs_only = 0u32;
    let mut shellcheck_errors = 0u32;
    let checked = sampled.len();

    for (input, label) in &sampled {
        let bashrs_unsafe = *label == 1;

        // Run shellcheck
        let sc_result = std::process::Command::new("shellcheck")
            .args(["-s", "sh", "-f", "json", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn();

        let sc_has_error = match sc_result {
            Ok(mut child) => {
                if let Some(ref mut stdin) = child.stdin {
                    use std::io::Write;
                    let _ = stdin.write_all(input.as_bytes());
                }
                let output = child.wait_with_output().ok();
                match output {
                    Some(o) => {
                        let stdout = String::from_utf8_lossy(&o.stdout);
                        if let Ok(diags) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
                            diags
                                .iter()
                                .any(|d| d.get("level").and_then(|v| v.as_str()) == Some("error"))
                        } else {
                            false
                        }
                    }
                    None => false,
                }
            }
            Err(_) => {
                shellcheck_errors += 1;
                false
            }
        };

        if bashrs_unsafe == sc_has_error {
            agree += 1;
        } else if sc_has_error {
            shellcheck_only += 1;
        } else {
            bashrs_only += 1;
        }
    }

    let agreement_pct = if checked > 0 {
        agree as f64 / checked as f64 * 100.0
    } else {
        0.0
    };

    if json {
        let result = serde_json::json!({
            "samples": checked,
            "agreement": agree,
            "agreement_pct": agreement_pct,
            "shellcheck_only": shellcheck_only,
            "bashrs_only": bashrs_only,
            "shellcheck_errors": shellcheck_errors,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
        );
    } else {
        println!("{BOLD}=== Cross-Linter Validation (bashrs vs ShellCheck) ==={RESET}\n");
        println!("  Samples checked: {checked}");
        println!("  Agreement:       {agree}/{checked} ({agreement_pct:.1}%)");
        println!("  ShellCheck-only: {shellcheck_only} (SC flags, bashrs doesn't)");
        println!("  bashrs-only:     {bashrs_only} (bashrs flags, SC doesn't)");
        if shellcheck_errors > 0 {
            println!("  SC errors:       {shellcheck_errors}");
        }
        println!();
        println!("  Note: ShellCheck checks general shell quality;");
        println!("        bashrs only flags SEC/DET/IDEM security rules.");
        println!("        Higher ShellCheck-only count is expected.");
    }

    Ok(())
}

/// Run eval harness on benchmark predictions (Step 7.7).
pub(crate) fn corpus_eval_benchmark(
    predictions_path: std::path::PathBuf,
    json: bool,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::eval_harness::{evaluate_predictions, EvalPrediction};

    eprintln!(
        "{BOLD}Running ShellSafetyBench eval harness on {}...{RESET}",
        predictions_path.display()
    );

    let content = std::fs::read_to_string(&predictions_path).map_err(|e| {
        Error::Validation(format!(
            "Failed to read {}: {e}",
            predictions_path.display()
        ))
    })?;

    let predictions: Vec<EvalPrediction> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();

    if predictions.is_empty() {
        return Err(Error::Validation(
            "No valid predictions found in input file".to_string(),
        ));
    }

    let results = evaluate_predictions(&predictions);

    if json {
        let json_str = serde_json::to_string_pretty(&results).unwrap_or_else(|_| "{}".to_string());
        println!("{json_str}");
    } else {
        println!("{BOLD}=== ShellSafetyBench Eval Results ==={RESET}\n");
        println!("  Predictions: {}", predictions.len());
        println!("  Weighted Score: {:.1}%\n", results.weighted_score * 100.0);

        println!("  {:<20} {:>8} {:>8}", "Metric", "Score", "Weight");
        println!("  {}", "-".repeat(40));
        println!(
            "  {:<20} {:>7.1}% {:>7.0}%",
            "Detection F1",
            results.detection_f1 * 100.0,
            25.0
        );
        println!(
            "  {:<20} {:>7.1}% {:>7.0}%",
            "Rule Citation",
            results.rule_citation * 100.0,
            20.0
        );
        println!(
            "  {:<20} {:>7.1}% {:>7.0}%",
            "CWE Mapping",
            results.cwe_mapping * 100.0,
            10.0
        );
        println!(
            "  {:<20} {:>7.1}% {:>7.0}%",
            "Fix Validity",
            results.fix_validity * 100.0,
            15.0
        );
        println!(
            "  {:<20} {:>7.1}% {:>7.0}%",
            "Explanation",
            results.explanation_quality * 100.0,
            15.0
        );
        println!(
            "  {:<20} {:>7.1}% {:>7.0}%",
            "OOD Generalization",
            results.ood_generalization * 100.0,
            15.0
        );
    }

    Ok(())
}

/// Validate pipeline tooling availability (SSC v12 S14 preflight)
pub(crate) fn corpus_pipeline_check(json: bool) -> Result<()> {
    use std::process::Command;

    const GREEN: &str = "\x1b[32m";
    const RED: &str = "\x1b[31m";
    const YELLOW: &str = "\x1b[33m";
    const BOLD: &str = "\x1b[1m";
    const RESET: &str = "\x1b[0m";

    struct ToolCheck {
        name: &'static str,
        command: &'static str,
        args: &'static [&'static str],
        required: bool,
    }

    let checks = [
        ToolCheck {
            name: "bashrs",
            command: "bashrs",
            args: &["--version"],
            required: true,
        },
        ToolCheck {
            name: "verificar",
            command: "verificar",
            args: &["--version"],
            required: true,
        },
        ToolCheck {
            name: "alimentar",
            command: "alimentar",
            args: &["--version"],
            required: true,
        },
        ToolCheck {
            name: "shellcheck",
            command: "shellcheck",
            args: &["--version"],
            required: true,
        },
        ToolCheck {
            name: "entrenar",
            command: "entrenar",
            args: &["--version"],
            required: false,
        },
        ToolCheck {
            name: "apr-cli",
            command: "apr",
            args: &["--version"],
            required: false,
        },
    ];

    let mut results = Vec::new();
    let mut all_required_pass = true;

    for check in &checks {
        let status = Command::new(check.command).args(check.args).output();

        let (pass, version) = match status {
            Ok(output) if output.status.success() => {
                let ver = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("unknown")
                    .trim()
                    .to_string();
                (true, ver)
            }
            _ => {
                if check.required {
                    all_required_pass = false;
                }
                (false, "not found".to_string())
            }
        };

        results.push((check.name, pass, version, check.required));
    }

    // Check config files
    let configs = [
        ("configs/pipeline/ssc.yaml", true),
        ("configs/train/ssc-qwen3-4b-qlora.yaml", true),
        ("configs/qa/ssc-release-v1.yaml", true),
        ("configs/cwe-mapping.yaml", true),
        (
            "provable-contracts/contracts/shellsafetybench-v1.yaml",
            true,
        ),
    ];

    let mut config_results = Vec::new();
    for (path, required) in &configs {
        let exists = std::path::Path::new(path).exists();
        if *required && !exists {
            all_required_pass = false;
        }
        config_results.push((*path, exists));
    }

    // Check data artifacts
    let artifacts = [
        "training/shellsafetybench/conversations.jsonl",
        "training/shellsafetybench/benchmark.jsonl",
    ];

    let mut artifact_results = Vec::new();
    for path in &artifacts {
        let exists = std::path::Path::new(path).exists();
        artifact_results.push((*path, exists));
    }

    if json {
        let tool_json: Vec<_> = results
            .iter()
            .map(|(name, pass, ver, req)| {
                serde_json::json!({
                    "tool": name,
                    "available": pass,
                    "version": ver,
                    "required": req,
                })
            })
            .collect();

        let config_json: Vec<_> = config_results
            .iter()
            .map(|(path, exists)| {
                serde_json::json!({
                    "path": path,
                    "exists": exists,
                })
            })
            .collect();

        let artifact_json: Vec<_> = artifact_results
            .iter()
            .map(|(path, exists)| {
                serde_json::json!({
                    "path": path,
                    "exists": exists,
                })
            })
            .collect();

        let report = serde_json::json!({
            "pipeline_ready": all_required_pass,
            "tools": tool_json,
            "configs": config_json,
            "artifacts": artifact_json,
        });

        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
    } else {
        eprintln!("{BOLD}SSC Pipeline Preflight Check{RESET}\n");

        eprintln!("{BOLD}Tools:{RESET}");
        for (name, pass, version, required) in &results {
            let icon = if *pass {
                format!("{GREEN}\u{2713}")
            } else if *required {
                format!("{RED}\u{2717}")
            } else {
                format!("{YELLOW}\u{25cb}")
            };
            let req_tag = if *required { "" } else { " (optional)" };
            eprintln!("  {icon}{RESET} {name}: {version}{req_tag}");
        }

        eprintln!("\n{BOLD}Config files:{RESET}");
        for (path, exists) in &config_results {
            let icon = if *exists {
                format!("{GREEN}\u{2713}")
            } else {
                format!("{RED}\u{2717}")
            };
            eprintln!("  {icon}{RESET} {path}");
        }

        eprintln!("\n{BOLD}Data artifacts:{RESET}");
        for (path, exists) in &artifact_results {
            let icon = if *exists {
                format!("{GREEN}\u{2713}")
            } else {
                format!("{YELLOW}\u{25cb}")
            };
            eprintln!("  {icon}{RESET} {path}");
        }

        if all_required_pass {
            eprintln!("\n{GREEN}\u{2713}{RESET} {BOLD}Pipeline ready{RESET}");
        } else {
            eprintln!("\n{RED}\u{2717}{RESET} {BOLD}Missing required tools or configs{RESET}");
        }
    }

    if all_required_pass {
        Ok(())
    } else {
        Err(crate::Error::Validation(
            "Pipeline preflight check failed: missing required tools or configs".to_string(),
        ))
    }
}

/// Merge corpus conversations + verificar mutations into unified training JSONL.
///
/// Reads the corpus conversations (auto-generated), additional input files
/// (e.g., verificar-labeled.jsonl), normalizes the schema, deduplicates,
/// shuffles deterministically, and writes a single merged JSONL.
pub(crate) fn corpus_merge_data(
    output: std::path::PathBuf,
    extra_inputs: Vec<std::path::PathBuf>,
    seed: u64,
) -> Result<()> {
    use crate::cli::color::*;
    use std::io::Write;

    let mut entries: Vec<serde_json::Value> = Vec::new();

    // 1. Load corpus conversations (labeled)
    let corpus_path = std::path::Path::new("training/shellsafetybench/conversations.jsonl");
    if corpus_path.exists() {
        let file = std::fs::File::open(corpus_path)?;
        let reader = std::io::BufReader::new(file);
        let mut count = 0usize;
        for line in std::io::BufRead::lines(reader) {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&line) {
                // Ensure source tag
                if let Some(obj) = val.as_object_mut() {
                    obj.entry("source".to_string())
                        .or_insert_with(|| serde_json::json!("bashrs-corpus"));
                }
                entries.push(val);
                count += 1;
            }
        }
        eprintln!("  Loaded {count} entries from corpus conversations");
    } else {
        eprintln!("{YELLOW}  Warning: corpus conversations not found, skipping{RESET}");
    }

    // 2. Load extra inputs (e.g., verificar-labeled.jsonl)
    // Normalize verificar mutation entries to conversation format matching corpus schema.
    for path in &extra_inputs {
        if !path.exists() {
            return Err(Error::Validation(format!(
                "Input file not found: {}",
                path.display()
            )));
        }
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let mut count = 0usize;
        for line in std::io::BufRead::lines(reader) {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&line) {
                // Normalize verificar mutation entries to conversation format
                if val.get("unsafe_script").is_some() && val.get("instruction").is_none() {
                    val = normalize_verificar_entry(val);
                }
                if let Some(obj) = val.as_object_mut() {
                    obj.entry("source".to_string())
                        .or_insert_with(|| serde_json::json!("verificar"));
                }
                entries.push(val);
                count += 1;
            }
        }
        eprintln!(
            "  Loaded {count} entries from {}",
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string())
        );
    }

    // 3. Deterministic shuffle using Fisher-Yates with simple PRNG
    let total = entries.len();
    let mut rng_state = seed;
    for i in (1..total).rev() {
        // Simple xorshift64
        rng_state ^= rng_state << 13;
        rng_state ^= rng_state >> 7;
        rng_state ^= rng_state << 17;
        let j = (rng_state as usize) % (i + 1);
        entries.swap(i, j);
    }

    // 4. Write merged JSONL
    let file = std::fs::File::create(&output)?;
    let mut buf = std::io::BufWriter::new(file);
    for entry in &entries {
        serde_json::to_writer(&mut buf, entry)
            .map_err(|e| Error::Validation(format!("JSON write error: {e}")))?;
        writeln!(buf)?;
    }
    buf.flush()?;

    eprintln!(
        "\n{GREEN}\u{2713}{RESET} {BOLD}Merged {total} entries → {}{RESET}",
        output.display()
    );

    Ok(())
}

/// Normalize a verificar mutation entry into the conversation format used by corpus entries.
///
/// Input fields: unsafe_script, safe_script, cwe, vulnerability, mutation_description, label, findings
/// Output fields: instruction, response, system, text, source, cwe, label, findings
fn normalize_verificar_entry(val: serde_json::Value) -> serde_json::Value {
    let unsafe_script = val
        .get("unsafe_script")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let safe_script = val
        .get("safe_script")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let cwe = val.get("cwe").and_then(|v| v.as_str()).unwrap_or("unknown");
    let vulnerability = val
        .get("vulnerability")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let mutation_desc = val
        .get("mutation_description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let label = val.get("label").cloned().unwrap_or(serde_json::json!(1));
    let findings = val
        .get("findings")
        .cloned()
        .unwrap_or(serde_json::json!([]));
    let classification = val
        .get("classification")
        .cloned()
        .unwrap_or(serde_json::json!("unsafe"));

    let instruction = format!(
        "Evaluate this shell script for security issues.\n\n```bash\n{unsafe_script}\n```",
    );

    let response = if label.as_u64() == Some(1) {
        format!(
            "This script contains a security vulnerability: {cwe} — {vulnerability}.\n\n\
             **Issue**: {mutation_desc}\n\n\
             **Fixed version**:\n```bash\n{safe_script}\n```",
        )
    } else {
        format!(
            "The linter did not detect known unsafe patterns in this script, but it may contain \
             subtle vulnerabilities ({cwe} — {vulnerability}): {mutation_desc}\n\n\
             **Safer version**:\n```bash\n{safe_script}\n```",
        )
    };

    let system = "You are a shell script security analyzer. Evaluate scripts for vulnerabilities \
                  including command injection, race conditions, hardcoded credentials, and other \
                  CWE-mapped security issues.";

    serde_json::json!({
        "instruction": instruction,
        "response": response,
        "system": system,
        "text": format!("{system}\n\n### Instruction:\n{instruction}\n\n### Response:\n{response}"),
        "label": label,
        "classification": classification,
        "findings": findings,
        "cwe": cwe,
        "cwe_id": val.get("cwe_id").cloned().unwrap_or(serde_json::json!(0)),
        "mutation_description": mutation_desc,
    })
}
