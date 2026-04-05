pub(crate) fn corpus_batch_eval(
    _model_dir: PathBuf,
    _test_data: PathBuf,
    _output: PathBuf,
    _max_tokens: usize,
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


include!("corpus_ssb_commands_tests_parse.rs");
