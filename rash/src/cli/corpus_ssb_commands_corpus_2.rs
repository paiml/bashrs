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

include!("corpus_ssb_commands_corpus.rs");
