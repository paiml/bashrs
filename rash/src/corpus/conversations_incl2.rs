fn generate_confirm_safe(input: &ConversationInput<'_>, variant: usize) -> Vec<Turn> {
    let prompt_idx = variant % SAFE_PROMPTS.len();
    let user_content = format!(
        "{}\n\n```bash\n{}\n```",
        SAFE_PROMPTS[prompt_idx], input.script
    );

    let openings = [
        "This script looks safe.",
        "This script appears to be well-written and safe.",
        "I don't see any security issues in this script.",
        "This script follows good practices.",
        "This script is safe to run.",
        "No security concerns found in this script.",
        "This script looks good.",
        "This is a clean, safe script.",
        "No issues detected in this script.",
        "This script appears production-ready.",
    ];

    let opening_idx = variant % openings.len();
    let mut response = String::from(openings[opening_idx]);
    response.push_str(" It doesn't contain known unsafe patterns like ");
    response
        .push_str("command injection, non-deterministic operations, or non-idempotent commands.");

    vec![
        system_turn(),
        Turn {
            role: "user",
            content: user_content,
        },
        Turn {
            role: "assistant",
            content: response,
        },
    ]
}

/// Create the system turn for ChatML conversations.
fn system_turn() -> Turn {
    Turn {
        role: "system",
        content: SYSTEM_PROMPT.to_string(),
    }
}

/// Apply basic safety fixes to a script based on linter diagnostics.
fn apply_safety_fixes(script: &str, diagnostics: &[Diagnostic]) -> String {
    let mut lines: Vec<String> = script.lines().map(|l| l.to_string()).collect();

    for d in diagnostics {
        let line_idx = d.span.start_line.saturating_sub(1);
        if line_idx < lines.len() {
            let line = &lines[line_idx];
            let fixed = apply_single_fix(line, &d.code);
            lines[line_idx] = fixed;
        }
    }

    lines.join("\n")
}

fn apply_single_fix(line: &str, code: &str) -> String {
    match code {
        // IDEM001: mkdir without -p
        "IDEM001" => line.replace("mkdir ", "mkdir -p "),
        // IDEM002: rm without -f
        "IDEM002" => line.replace("rm ", "rm -f "),
        // IDEM003: ln -s without -f
        "IDEM003" => line.replace("ln -s ", "ln -sf "),
        // SEC001: eval usage — comment it out
        "SEC001" => format!("# REMOVED (unsafe): {line}"),
        // SEC002: curl|bash — comment it out
        "SEC002" => format!("# REMOVED (unsafe): {line}"),
        // DET001: $RANDOM
        "DET001" => line.replace("$RANDOM", "42"),
        // DET002: date/timestamps
        "DET002" => line.replace("$(date)", "\"2026-01-01\""),
        _ => line.to_string(),
    }
}

fn check_variant_distribution(conversations: &[Conversation]) -> bool {
    if conversations.is_empty() {
        return true;
    }

    // Count conversations by variant (user prompt text determines variant)
    let mut variant_counts = std::collections::HashMap::new();
    for conv in conversations {
        // User turn is at index 1 (after system turn)
        if let Some(user_turn) = conv.turns.get(1) {
            // Extract first line as variant key
            let key = user_turn.content.lines().next().unwrap_or("").to_string();
            *variant_counts.entry(key).or_insert(0usize) += 1;
        }
    }

    let total = conversations.len();
    let max_pct = 0.20;

    for count in variant_counts.values() {
        if *count as f64 / total as f64 > max_pct {
            return false;
        }
    }

    true
}

/// Serialize conversations to JSONL format (one JSON object per line).
pub fn to_jsonl(conversations: &[Conversation]) -> String {
    let mut output = String::new();
    for conv in conversations {
        if let Ok(json) = serde_json::to_string(conv) {
            output.push_str(&json);
            output.push('\n');
        }
    }
    output
}

/// Convert conversations to entrenar-compatible JSONL format.
///
/// Each conversation is formatted as a complete ChatML text sequence:
/// `<|im_start|>system\n...<|im_end|>\n<|im_start|>user\n...<|im_end|>\n<|im_start|>assistant\n...<|im_end|>`
///
/// The `text` field is what entrenar tokenizes for causal LM training.
/// Also includes `instruction`, `response`, `system` for metadata/evaluation.
pub fn to_entrenar_jsonl(conversations: &[Conversation]) -> String {
    let mut output = String::new();
    for conv in conversations {
        let system = conv
            .turns
            .iter()
            .find(|t| t.role == "system")
            .map(|t| t.content.as_str())
            .unwrap_or(SYSTEM_PROMPT);
        let instruction = conv
            .turns
            .iter()
            .find(|t| t.role == "user")
            .map(|t| t.content.as_str())
            .unwrap_or("");
        let response = conv
            .turns
            .iter()
            .find(|t| t.role == "assistant")
            .map(|t| t.content.as_str())
            .unwrap_or("");
        if instruction.is_empty() || response.is_empty() {
            continue;
        }
        // Full ChatML-formatted text for causal LM training
        let text = format!(
            "<|im_start|>system\n{system}<|im_end|>\n\
             <|im_start|>user\n{instruction}<|im_end|>\n\
             <|im_start|>assistant\n{response}<|im_end|>"
        );
        let sample = serde_json::json!({
            "text": text,
            "instruction": instruction,
            "response": response,
            "system": system,
        });
        if let Ok(json) = serde_json::to_string(&sample) {
            output.push_str(&json);
            output.push('\n');
        }
    }
    output
}

/// Generate a HuggingFace dataset README with YAML front matter (S6.6).
///
/// Returns a complete README.md for `paiml/shell-safety-conversations`.
pub fn generate_dataset_readme(report: &QualityReport) -> String {
    let mut readme = String::new();

    // YAML front matter
    let _ = write!(
        readme,
        "---\n\
        language:\n\
        - en\n\
        license: apache-2.0\n\
        task_categories:\n\
        - text-classification\n\
        - text-generation\n\
        tags:\n\
        - shell\n\
        - bash\n\
        - security\n\
        - safety\n\
        - code-analysis\n\
        - synthetic\n\
        size_categories:\n\
        - 10K<n<100K\n\
        ---\n\n"
    );

    // Title and description
    readme.push_str("# Shell Safety Conversations\n\n");
    readme.push_str(
        "Synthetic instruction-following conversations for shell script safety analysis. \
        Generated from the bashrs corpus using rule-based linter findings.\n\n",
    );

    // Dataset summary
    readme.push_str("## Dataset Summary\n\n");
    let _ = writeln!(readme, "- **Total conversations**: {}", report.total);
    let _ = writeln!(
        readme,
        "- **Type A (Classify+Explain)**: {}",
        report.type_a_count
    );
    let _ = writeln!(readme, "- **Type B (Fix)**: {}", report.type_b_count);
    let _ = writeln!(readme, "- **Type C (Debug)**: {}", report.type_c_count);
    let _ = writeln!(
        readme,
        "- **Type D (Confirm Safe)**: {} ({:.1}%)",
        report.type_d_count, report.type_d_pct
    );
    let _ = writeln!(readme, "- **Format**: ChatML (system + user + assistant)");
    let _ = writeln!(
        readme,
        "- **Quality gate**: {}",
        if report.passed { "PASSED" } else { "FAILED" }
    );
    readme.push('\n');

    // Honesty requirements (S6.5)
    readme.push_str("## Limitations and Bias\n\n");
    readme.push_str(
        "This dataset is generated from **rule-based linter output**, not from human security experts \
        or independent safety reasoning. The conversations:\n\n\
        - Explain known unsafe patterns (SEC001-SEC024, DET001-DET006, IDEM001-IDEM006)\n\
        - Do NOT perform novel security reasoning\n\
        - May produce generic responses for scripts outside rule coverage\n\
        - Are NOT a replacement for professional security audit\n\
        - Use synthetic phrasing variants (12 per type) for diversity\n\n",
    );

    // Format
    readme.push_str("## Data Format\n\n");
    readme.push_str("Each entry is a JSON object with:\n\n");
    readme.push_str("```json\n");
    readme.push_str("{\n");
    readme.push_str("  \"id\": \"conv-B-1234-classify-explain\",\n");
    readme.push_str("  \"conversation_type\": \"ClassifyExplain\",\n");
    readme.push_str("  \"turns\": [\n");
    readme.push_str(
        "    {\"role\": \"system\", \"content\": \"You are a shell script safety analyzer...\"},\n",
    );
    readme.push_str("    {\"role\": \"user\", \"content\": \"Is this script safe?\\n\\n```bash\\neval $x\\n```\"},\n");
    readme
        .push_str("    {\"role\": \"assistant\", \"content\": \"This script is **unsafe**...\"}\n");
    readme.push_str("  ]\n");
    readme.push_str("}\n");
    readme.push_str("```\n\n");

    // Source
    readme.push_str("## Source\n\n");
    readme.push_str(
        "Generated by `bashrs corpus generate-conversations` from the bashrs corpus \
        (17,942 shell script entries). See [bashrs](https://github.com/paiml/bashrs) \
        and the [SSC v11 specification](https://github.com/paiml/bashrs/blob/main/docs/specifications/shell-safety-inference.md).\n",
    );

    readme
}

#[cfg(test)]
#[path = "conversations_tests_extracted.rs"]
mod tests_extracted;
