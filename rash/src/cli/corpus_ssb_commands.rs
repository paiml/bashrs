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

/// Convert SSB split JSONL to entrenar ChatML format for chat model training.
///
/// Input format:  `{"input": "Evaluate this shell script.\n\n```bash\n...\n```", "label": 0|1}`
/// Output format: ChatML with `Classification: safe/unsafe` response prefix + linter analysis.
///
/// This produces training data aligned with the eval harness format (ssc_eval.rs)
/// which looks for `"Classification: safe"` / `"Classification: unsafe"` strings.
pub(crate) fn corpus_convert_ssb(
    input: std::path::PathBuf,
    output: Option<std::path::PathBuf>,
    limit: Option<usize>,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::conversations::SYSTEM_PROMPT;
    use crate::linter::lint_shell;
    use std::fmt::Write as _;

    let data = std::fs::read_to_string(&input).map_err(Error::Io)?;
    let lines: Vec<&str> = data.lines().collect();
    let max = limit.unwrap_or(lines.len()).min(lines.len());

    eprintln!(
        "{BOLD}Converting {max}/{} SSB entries to ChatML...{RESET}",
        lines.len()
    );

    let mut output_buf = String::new();
    let mut converted = 0usize;
    let mut safe_count = 0usize;
    let mut unsafe_count = 0usize;
    let mut skipped = 0usize;

    for line in lines.iter().take(max) {
        let entry: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };

        let input_text = match entry["input"].as_str() {
            Some(s) => s,
            None => {
                skipped += 1;
                continue;
            }
        };
        let label = entry["label"].as_i64().unwrap_or(-1);
        if label != 0 && label != 1 {
            skipped += 1;
            continue;
        }

        // Extract shell code from markdown block in input
        let shell_code = extract_shell_from_markdown(input_text);

        // Run linter to generate analysis
        let analysis = if !shell_code.is_empty() {
            let result = lint_shell(&shell_code);
            let diagnostics = &result.diagnostics;
            if diagnostics.is_empty() {
                if label == 1 {
                    "This script contains patterns that may pose security or reliability risks."
                        .to_string()
                } else {
                    "No security issues, non-deterministic patterns, or non-idempotent operations found."
                        .to_string()
                }
            } else {
                let mut analysis = String::new();
                for d in diagnostics {
                    let _ = writeln!(
                        analysis,
                        "- {} (line {}): {}",
                        d.code, d.span.start_line, d.message
                    );
                }
                analysis
            }
        } else {
            "Unable to extract shell code for analysis.".to_string()
        };

        // Build response with Classification: prefix (matches ssc_eval.rs format)
        let classification = if label == 0 { "safe" } else { "unsafe" };
        let response = format!("Classification: {classification}\n\n{analysis}");

        // Build ChatML text
        let text = format!(
            "<|im_start|>system\n{SYSTEM_PROMPT}<|im_end|>\n\
             <|im_start|>user\n{input_text}<|im_end|>\n\
             <|im_start|>assistant\n{response}<|im_end|>"
        );

        let sample = serde_json::json!({
            "text": text,
            "instruction": input_text,
            "response": response,
            "system": SYSTEM_PROMPT,
        });
        if let Ok(json) = serde_json::to_string(&sample) {
            output_buf.push_str(&json);
            output_buf.push('\n');
            converted += 1;
            if label == 0 {
                safe_count += 1;
            } else {
                unsafe_count += 1;
            }
        }
    }

    match output {
        Some(ref path) => {
            std::fs::write(path, &output_buf).map_err(Error::Io)?;
            eprintln!(
                "{GREEN}Wrote {converted} conversations to {}{RESET}",
                path.display()
            );
        }
        None => {
            print!("{output_buf}");
        }
    }

    eprintln!();
    eprintln!("{BOLD}Conversion Report:{RESET}");
    eprintln!("  Converted:  {converted}");
    eprintln!("  Safe:       {safe_count}");
    eprintln!("  Unsafe:     {unsafe_count}");
    eprintln!("  Skipped:    {skipped}");
    let unsafe_pct = if converted > 0 {
        100.0 * unsafe_count as f64 / converted as f64
    } else {
        0.0
    };
    eprintln!("  Unsafe %:   {unsafe_pct:.1}%");

    Ok(())
}

/// Run batch inference on test split and produce predictions JSONL for the eval harness.
///
/// Loads the model once, iterates through test entries, runs inference, parses
/// classification output, and writes `EvalPrediction`-compatible JSONL.
#[allow(unused_variables)]
pub(crate) fn corpus_batch_eval(
    model_dir: PathBuf,
    test_data: PathBuf,
    output: PathBuf,
    max_tokens: usize,
) -> Result<()> {
    #[cfg(not(feature = "ml"))]
    {
        Err(Error::Validation(
            "The `ml` feature is required for batch-eval. \
             Rebuild with: cargo build --features ml"
                .into(),
        ))
    }

    #[cfg(feature = "ml")]
    {
        use crate::cli::chat_inference::{format_explain_prompt, SYSTEM_PROMPT};
        use crate::cli::color::*;
        use crate::corpus::cwe_mapping;
        use entrenar::finetune::{GenerateConfig, InstructConfig, InstructPipeline};
        use entrenar::transformer::TransformerConfig;
        use std::io::Write;

        eprintln!(
            "{BOLD}Batch eval: loading model from {}...{RESET}",
            model_dir.display()
        );

        // Load model config
        let config_path = model_dir.join("config.json");
        let model_config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| Error::Validation(format!("Cannot read config.json: {e}")))?;
            let json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| Error::Validation(format!("Invalid config.json: {e}")))?;
            TransformerConfig {
                hidden_size: json["hidden_size"].as_u64().unwrap_or(1536) as usize,
                num_hidden_layers: json["num_hidden_layers"].as_u64().unwrap_or(28) as usize,
                num_attention_heads: json["num_attention_heads"].as_u64().unwrap_or(12) as usize,
                num_kv_heads: json["num_key_value_heads"].as_u64().unwrap_or(2) as usize,
                intermediate_size: json["intermediate_size"].as_u64().unwrap_or(8960) as usize,
                vocab_size: json["vocab_size"].as_u64().unwrap_or(151936) as usize,
                max_position_embeddings: json["max_position_embeddings"].as_u64().unwrap_or(32768)
                    as usize,
                rms_norm_eps: json["rms_norm_eps"].as_f64().unwrap_or(1e-6) as f32,
                rope_theta: json["rope_theta"].as_f64().unwrap_or(1000000.0) as f32,
                use_bias: json["use_bias"].as_bool().unwrap_or(false),
                head_dim_override: json["head_dim"].as_u64().map(|v| v as usize),
                architecture: Default::default(),
            }
        } else {
            // Default to Qwen2.5-Coder-1.5B-Instruct dimensions
            TransformerConfig {
                hidden_size: 1536,
                num_hidden_layers: 28,
                num_attention_heads: 12,
                num_kv_heads: 2,
                intermediate_size: 8960,
                vocab_size: 151936,
                max_position_embeddings: 32768,
                rms_norm_eps: 1e-6,
                rope_theta: 1000000.0,
                use_bias: false,
                head_dim_override: None,
                architecture: Default::default(),
            }
        };

        let instruct_config = InstructConfig {
            lora_rank: 16,
            lora_alpha: 32.0,
            max_seq_len: 2048,
            ..InstructConfig::default()
        };

        let pipeline =
            InstructPipeline::from_pretrained(&model_dir, &model_config, instruct_config)
                .map_err(|e| Error::Validation(format!("Failed to load model: {e}")))?;

        eprintln!("{GREEN}\u{2713}{RESET} Model loaded.");

        // Load test data
        let test_content = std::fs::read_to_string(&test_data).map_err(|e| {
            Error::Validation(format!(
                "Failed to read test data {}: {e}",
                test_data.display()
            ))
        })?;

        #[derive(serde::Deserialize)]
        struct TestEntry {
            input: String,
            #[serde(default)]
            label: u8,
        }

        let entries: Vec<TestEntry> = test_content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();

        if entries.is_empty() {
            return Err(Error::Validation(
                "No valid entries found in test data".to_string(),
            ));
        }

        eprintln!(
            "{BOLD}Running inference on {} entries (max_tokens={max_tokens})...{RESET}",
            entries.len()
        );

        let gen_config = GenerateConfig {
            max_new_tokens: max_tokens,
            temperature: 0.3,
            top_k: 40,
            stop_tokens: Vec::new(),
        };

        let mut out_file = std::fs::File::create(&output).map_err(|e| {
            Error::Validation(format!("Cannot create output {}: {e}", output.display()))
        })?;

        let total = entries.len();
        let mut safe_count = 0usize;
        let mut unsafe_count = 0usize;
        let mut parse_failures = 0usize;
        let start = std::time::Instant::now();

        for (i, entry) in entries.iter().enumerate() {
            let shell_code = extract_shell_from_markdown(&entry.input);
            let user_prompt = format_explain_prompt(&shell_code, "");

            let model_output =
                match pipeline.generate_chat(SYSTEM_PROMPT, &user_prompt, &gen_config) {
                    Ok(text) => text,
                    Err(e) => {
                        eprintln!("  [{}/{total}] ERROR: {e} — skipping", i + 1);
                        parse_failures += 1;
                        continue;
                    }
                };

            // Parse classification from model output
            let (classification, rules, cwes) = parse_batch_eval_output(&model_output);

            match classification.as_str() {
                "safe" => safe_count += 1,
                "unsafe" => unsafe_count += 1,
                _ => parse_failures += 1,
            }

            // Build EvalPrediction-compatible output
            let pred = serde_json::json!({
                "id": format!("test-{i}"),
                "classification": classification,
                "label": entry.label,
                "cited_rules": rules,
                "cited_cwes": cwes,
                "proposed_fix": "",
                "explanation": model_output,
                "script": shell_code,
                "ground_truth_rules": [],
                "ground_truth_cwes": [],
            });

            writeln!(
                out_file,
                "{}",
                serde_json::to_string(&pred).unwrap_or_default()
            )
            .map_err(|e| Error::Validation(format!("Write error: {e}")))?;

            // Progress
            let elapsed = start.elapsed().as_secs_f64();
            let rate = if elapsed > 0.0 {
                (i + 1) as f64 / elapsed
            } else {
                0.0
            };
            let eta_s = if rate > 0.0 {
                ((total - i - 1) as f64 / rate) as u64
            } else {
                0
            };
            eprintln!(
                "  [{}/{total}] {:.1}% | {:.2} entries/s | ETA: {}m {}s | cls={}",
                i + 1,
                100.0 * (i + 1) as f64 / total as f64,
                rate,
                eta_s / 60,
                eta_s % 60,
                classification,
            );
        }

        let elapsed = start.elapsed();
        eprintln!();
        eprintln!(
            "{GREEN}\u{2713}{RESET} {BOLD}Predictions written to {}{RESET}",
            output.display()
        );
        eprintln!("  Total:    {total}");
        eprintln!("  Safe:     {safe_count}");
        eprintln!("  Unsafe:   {unsafe_count}");
        eprintln!("  Failures: {parse_failures}");
        eprintln!("  Time:     {:.1}s", elapsed.as_secs_f64());

        Ok(())
    }
}

/// Parse model output to extract classification, rule IDs, and CWE mappings.
///
/// Looks for "Classification: safe" or "Classification: unsafe" in the output.
/// Extracts SEC/DET/IDEM/SC rule patterns and maps them to CWEs.
#[cfg_attr(not(feature = "ml"), allow(dead_code))]
fn parse_batch_eval_output(output: &str) -> (String, Vec<String>, Vec<String>) {
    use crate::corpus::cwe_mapping;

    // Parse classification
    let lower = output.to_lowercase();
    let classification =
        if lower.contains("classification: unsafe") || lower.contains("classification:unsafe") {
            "unsafe".to_string()
        } else if lower.contains("classification: safe") || lower.contains("classification:safe") {
            "safe".to_string()
        } else if lower.contains("unsafe") {
            // Fallback: look for the word "unsafe" anywhere
            "unsafe".to_string()
        } else {
            "safe".to_string()
        };

    // Extract rule IDs (SEC001, DET002, IDEM003, SC2039, etc.)
    let mut rules: Vec<String> = Vec::new();
    for word in output.split(|c: char| !c.is_alphanumeric()) {
        let is_rule = match word.len() {
            6 if word.starts_with("SEC") && word[3..].chars().all(|c| c.is_ascii_digit()) => true,
            6 if word.starts_with("DET") && word[3..].chars().all(|c| c.is_ascii_digit()) => true,
            7 if word.starts_with("IDEM") && word[4..].chars().all(|c| c.is_ascii_digit()) => true,
            6 if word.starts_with("SC") && word[2..].chars().all(|c| c.is_ascii_digit()) => true,
            _ => false,
        };
        if is_rule && !rules.contains(&word.to_string()) {
            rules.push(word.to_string());
        }
    }

    // Map rules to CWEs
    let mut cwes: Vec<String> = Vec::new();
    for rule in &rules {
        if let Some(mapping) = cwe_mapping::lookup_rule(rule) {
            let cwe = mapping.cwe.to_string();
            if !cwes.contains(&cwe) {
                cwes.push(cwe);
            }
        }
    }

    (classification, rules, cwes)
}

/// Extract shell code from markdown code block in SSB input text.
///
/// Handles formats like:
/// ```bash
/// some code here
/// ```
fn extract_shell_from_markdown(input: &str) -> String {
    // Find ```bash or ```sh or ``` block
    let start_markers = ["```bash\n", "```sh\n", "```shell\n", "```\n"];
    for marker in &start_markers {
        if let Some(start_idx) = input.find(marker) {
            let code_start = start_idx + marker.len();
            if let Some(end_idx) = input[code_start..].find("```") {
                return input[code_start..code_start + end_idx].to_string();
            }
            // No closing ```, take rest of input
            return input[code_start..].to_string();
        }
    }
    // No markdown block found — return entire input as-is
    input.to_string()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_parse_batch_eval_output_classification_unsafe() {
        let (cls, _rules, _cwes) =
            parse_batch_eval_output("Classification: unsafe\nThis script uses eval.");
        assert_eq!(cls, "unsafe");
    }

    #[test]
    fn test_parse_batch_eval_output_classification_safe() {
        let (cls, _rules, _cwes) =
            parse_batch_eval_output("Classification: safe\nNo issues found.");
        assert_eq!(cls, "safe");
    }

    #[test]
    fn test_parse_batch_eval_output_fallback_unsafe() {
        let (cls, _rules, _cwes) =
            parse_batch_eval_output("This script is unsafe because it uses eval.");
        assert_eq!(cls, "unsafe");
    }

    #[test]
    fn test_parse_batch_eval_output_fallback_safe() {
        let (cls, _rules, _cwes) = parse_batch_eval_output("The script looks fine, no problems.");
        assert_eq!(cls, "safe");
    }

    #[test]
    fn test_parse_batch_eval_output_extracts_rules() {
        let (_cls, rules, _cwes) = parse_batch_eval_output(
            "Classification: unsafe\nViolations: SEC001 (command injection), SEC002 (unquoted var).",
        );
        assert!(rules.contains(&"SEC001".to_string()));
        assert!(rules.contains(&"SEC002".to_string()));
    }

    #[test]
    fn test_parse_batch_eval_output_extracts_det_idem() {
        let (_cls, rules, _cwes) =
            parse_batch_eval_output("DET001 non-determinism, IDEM001 not idempotent.");
        assert!(rules.contains(&"DET001".to_string()));
        assert!(rules.contains(&"IDEM001".to_string()));
    }

    #[test]
    fn test_parse_batch_eval_output_maps_cwes() {
        let (_cls, rules, cwes) =
            parse_batch_eval_output("Classification: unsafe\nSEC001 command injection detected.");
        assert!(rules.contains(&"SEC001".to_string()));
        // SEC001 maps to CWE-78
        assert!(cwes.contains(&"CWE-78".to_string()));
    }

    #[test]
    fn test_parse_batch_eval_output_no_rules() {
        let (_cls, rules, cwes) = parse_batch_eval_output("Classification: safe\nAll good.");
        assert!(rules.is_empty());
        assert!(cwes.is_empty());
    }

    #[test]
    fn test_parse_batch_eval_output_deduplicates_rules() {
        let (_cls, rules, _cwes) =
            parse_batch_eval_output("SEC001 found here. SEC001 also found there.");
        assert_eq!(
            rules.iter().filter(|r| *r == "SEC001").count(),
            1,
            "SEC001 should appear exactly once"
        );
    }
}
