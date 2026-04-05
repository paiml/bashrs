//! ShellSafetyBench (SSB) pipeline commands: validation, evaluation, data merging, and conversion.
use crate::models::{Error, Result};
use std::path::{Path, PathBuf};

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

include!("corpus_ssb_commands_corpus_2.rs");
