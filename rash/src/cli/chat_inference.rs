//! Chat model inference for `bashrs explain --chat-model` and `bashrs fix --chat-model`.
//!
//! Loads a Qwen-1.5B + LoRA model via entrenar's InstructPipeline and generates
//! natural-language safety analysis. Requires the `ml` feature.
//!
//! # Architecture (SSC v11 Phase 4 CLI-002)
//!
//! ```text
//! bashrs explain --chat-model /path/to/model script.sh
//!     ├── lint (<1ms) ──> findings
//!     ├── format ChatML prompt (system + user with script + findings)
//!     ├── InstructPipeline::generate_chat() ──> Qwen response
//!     v
//!     Output: natural language explanation from chat model
//! ```

use crate::models::{Error, Result};
use std::path::Path;

/// System prompt for shell safety analysis.
pub(crate) const SYSTEM_PROMPT: &str = "\
You are a shell script safety analyzer. You classify scripts as safe or unsafe, \
explain security vulnerabilities, non-deterministic patterns, and idempotency issues, \
and suggest specific fixes. Be concise and accurate. If the script is safe, say so briefly. \
Never hallucinate issues that don't exist in the script.";

/// Format a prompt for the explain command.
pub(crate) fn format_explain_prompt(source: &str, findings_summary: &str) -> String {
    if findings_summary.is_empty() {
        format!(
            "Analyze this shell script for safety issues:\n\n```bash\n{source}\n```\n\n\
             Explain whether it's safe or unsafe, and why."
        )
    } else {
        format!(
            "Analyze this shell script for safety issues:\n\n```bash\n{source}\n```\n\n\
             The linter found these issues:\n{findings_summary}\n\n\
             Explain each issue in plain language and suggest how to fix them."
        )
    }
}

/// Format a prompt for the fix command.
pub(crate) fn format_fix_prompt(source: &str, findings_summary: &str) -> String {
    format!(
        "Fix the safety issues in this shell script:\n\n```bash\n{source}\n```\n\n\
         Issues found:\n{findings_summary}\n\n\
         Provide the corrected script with all issues fixed. Only output the fixed script."
    )
}

/// Generate text using entrenar's InstructPipeline.
///
/// # Errors
/// Returns error if model cannot be loaded or generation fails.
#[cfg(feature = "ml")]
pub(crate) fn chat_generate(
    model_dir: &Path,
    system: &str,
    user_message: &str,
    max_tokens: usize,
) -> Result<String> {
    use entrenar::finetune::{GenerateConfig, InstructConfig, InstructPipeline};
    use entrenar::transformer::TransformerConfig;

    // Detect model config from the model directory
    let config_path = model_dir.join("config.json");
    let model_config = if config_path.exists() {
        load_model_config(&config_path)?
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

    eprintln!("Loading model from {}...", model_dir.display());
    let pipeline = InstructPipeline::from_pretrained(model_dir, &model_config, instruct_config)
        .map_err(|e| Error::Validation(format!("Failed to load model: {e}")))?;

    eprintln!("Generating response (max {} tokens)...", max_tokens);
    let gen_config = GenerateConfig {
        max_new_tokens: max_tokens,
        temperature: 0.3, // Low temperature for factual analysis
        top_k: 40,
        stop_tokens: Vec::new(),
    };

    pipeline
        .generate_chat(system, user_message, &gen_config)
        .map_err(|e| Error::Validation(format!("Generation failed: {e}")))
}

/// Load TransformerConfig from a HuggingFace config.json.
#[cfg(feature = "ml")]
fn load_model_config(path: &Path) -> Result<entrenar::transformer::TransformerConfig> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::Validation(format!("Cannot read config.json: {e}")))?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| Error::Validation(format!("Invalid config.json: {e}")))?;

    Ok(entrenar::transformer::TransformerConfig {
        hidden_size: json["hidden_size"].as_u64().unwrap_or(1536) as usize,
        num_hidden_layers: json["num_hidden_layers"].as_u64().unwrap_or(28) as usize,
        num_attention_heads: json["num_attention_heads"].as_u64().unwrap_or(12) as usize,
        num_kv_heads: json["num_key_value_heads"].as_u64().unwrap_or(2) as usize,
        intermediate_size: json["intermediate_size"].as_u64().unwrap_or(8960) as usize,
        vocab_size: json["vocab_size"].as_u64().unwrap_or(151936) as usize,
        max_position_embeddings: json["max_position_embeddings"].as_u64().unwrap_or(32768) as usize,
        rms_norm_eps: json["rms_norm_eps"].as_f64().unwrap_or(1e-6) as f32,
        rope_theta: json["rope_theta"].as_f64().unwrap_or(1000000.0) as f32,
        use_bias: json["use_bias"].as_bool().unwrap_or(false),
        head_dim_override: json["head_dim"].as_u64().map(|v| v as usize),
        architecture: Default::default(),
    })
}

/// Stub for when `ml` feature is not enabled.
#[cfg(not(feature = "ml"))]
pub(crate) fn chat_generate(
    _model_dir: &Path,
    _system: &str,
    _user_message: &str,
    _max_tokens: usize,
) -> Result<String> {
    Err(Error::Validation(
        "Chat model inference requires the `ml` feature. \
         Rebuild with: cargo build --features ml"
            .to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_explain_prompt_with_findings() {
        let prompt = format_explain_prompt("echo $VAR", "SEC002: Unquoted variable");
        assert!(prompt.contains("echo $VAR"));
        assert!(prompt.contains("SEC002"));
        assert!(prompt.contains("Explain each issue"));
    }

    #[test]
    fn test_format_explain_prompt_no_findings() {
        let prompt = format_explain_prompt("echo hello", "");
        assert!(prompt.contains("echo hello"));
        assert!(prompt.contains("safe or unsafe"));
    }

    #[test]
    fn test_format_fix_prompt() {
        let prompt = format_fix_prompt("echo $VAR", "SEC002: Unquoted variable");
        assert!(prompt.contains("Fix the safety issues"));
        assert!(prompt.contains("echo $VAR"));
    }

    #[test]
    fn test_system_prompt_content() {
        assert!(SYSTEM_PROMPT.contains("shell script safety"));
        assert!(SYSTEM_PROMPT.contains("Never hallucinate"));
    }

    #[cfg(not(feature = "ml"))]
    #[test]
    fn test_chat_generate_without_ml_feature() {
        let result = chat_generate(Path::new("/tmp/fake"), "sys", "user", 100);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("ml"));
    }
}
