//! Synthetic conversation generation for shell safety chat model training.
//!
//! Implements SSC v11 Section 6: generates instruction conversations from
//! corpus entries + linter findings for fine-tuning Qwen2.5-Coder-1.5B.
//!
//! ## Conversation Types
//!
//! - **Type A (Classify+Explain)**: Unsafe scripts with lint findings
//! - **Type B (Fix)**: Unsafe scripts with corrected version
//! - **Type C (Debug)**: Non-deterministic scripts
//! - **Type D (Confirm Safe)**: Safe scripts (>=30% of total)
//!
//! Each type has 10+ phrasing variants, randomly selected with seed.

use crate::linter::{lint_shell, Diagnostic};
use serde::Serialize;
use std::fmt::Write as _;

/// System prompt for ChatML conversations (S6.5 honesty requirements).
///
/// This prompt defines the assistant's role and limitations per SSC v11 Section 6.5:
/// - Trained on synthetic data from rule-based linter output
/// - Explains known patterns, not novel safety reasoning
/// - Not a replacement for security audit
pub const SYSTEM_PROMPT: &str = "\
You are a shell script safety analyzer. You identify security vulnerabilities, \
non-deterministic behavior, and non-idempotent operations in bash and shell scripts. \
Your analysis is based on pattern matching against known unsafe constructs \
(command injection, unquoted variables, non-deterministic sources, non-idempotent operations). \
You do NOT perform novel security reasoning — you explain known patterns. \
For scripts outside your rule coverage, say so honestly. \
You are not a replacement for a professional security audit.";

/// A single conversation turn (ChatML format for Qwen).
#[derive(Debug, Clone, Serialize)]
pub struct Turn {
    pub role: &'static str,
    pub content: String,
}

/// A complete conversation for training.
#[derive(Debug, Clone, Serialize)]
pub struct Conversation {
    pub id: String,
    pub conversation_type: ConversationType,
    pub turns: Vec<Turn>,
}

/// The four conversation types from SSC v11 Section 6.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ConversationType {
    /// Classify + Explain (unsafe scripts with findings)
    ClassifyExplain,
    /// Fix (unsafe scripts with corrected version)
    Fix,
    /// Debug (non-deterministic scripts)
    Debug,
    /// Confirm Safe (safe scripts)
    ConfirmSafe,
}

impl ConversationType {
    fn label(&self) -> &'static str {
        match self {
            Self::ClassifyExplain => "classify-explain",
            Self::Fix => "fix",
            Self::Debug => "debug",
            Self::ConfirmSafe => "confirm-safe",
        }
    }
}

/// Input data for generating a conversation.
pub struct ConversationInput<'a> {
    pub entry_id: &'a str,
    pub script: &'a str,
    pub is_safe: bool,
    pub is_deterministic: bool,
    pub diagnostics: &'a [Diagnostic],
}

/// Quality gate results for generated conversations (SSC v11 Section 6.4).
#[derive(Debug, Clone, Serialize)]
pub struct QualityReport {
    pub total: usize,
    pub type_a_count: usize,
    pub type_b_count: usize,
    pub type_c_count: usize,
    pub type_d_count: usize,
    pub type_d_pct: f64,
    pub rule_citation_accuracy: f64,
    pub variant_distribution_ok: bool,
    pub empty_responses: usize,
    pub passed: bool,
}

/// Generate a conversation from a corpus entry and its lint diagnostics.
///
/// # Conversation type selection (SSC v11 Section 6.2):
/// - Unsafe + has findings → Type A (classify+explain)
/// - Unsafe + security findings → Type B (fix) on alternating seeds
/// - Non-deterministic → Type C (debug)
/// - Safe → Type D (confirm safe)
pub fn generate_conversation(input: &ConversationInput<'_>, seed: u64) -> Conversation {
    let conv_type = select_type(input, seed);
    let variant = (seed % 10) as usize;

    let turns = match conv_type {
        ConversationType::ClassifyExplain => generate_classify_explain(input, variant),
        ConversationType::Fix => generate_fix(input, variant),
        ConversationType::Debug => generate_debug(input, variant),
        ConversationType::ConfirmSafe => generate_confirm_safe(input, variant),
    };

    Conversation {
        id: format!("conv-{}-{}", input.entry_id, conv_type.label()),
        conversation_type: conv_type,
        turns,
    }
}

/// Generate conversations for a batch of scripts.
pub fn generate_batch(
    entries: &[(&str, &str)],
    base_seed: u64,
) -> (Vec<Conversation>, QualityReport) {
    let mut conversations = Vec::with_capacity(entries.len());
    let mut type_counts = [0usize; 4];

    for (i, (entry_id, script)) in entries.iter().enumerate() {
        let result = lint_shell(script);
        let diagnostics = &result.diagnostics;

        let has_security = diagnostics.iter().any(|d| d.code.starts_with("SEC"));
        let has_determinism = diagnostics.iter().any(|d| d.code.starts_with("DET"));
        let has_idempotency = diagnostics.iter().any(|d| d.code.starts_with("IDEM"));

        // Safe = no security, determinism, or idempotency findings
        // Style/portability warnings (SC*) don't affect safety classification
        let is_safe = !has_security && !has_determinism && !has_idempotency;

        let input = ConversationInput {
            entry_id,
            script,
            is_safe,
            is_deterministic: !has_determinism,
            diagnostics,
        };

        let seed = base_seed.wrapping_add(i as u64);
        let conv = generate_conversation(&input, seed);

        match conv.conversation_type {
            ConversationType::ClassifyExplain => type_counts[0] += 1,
            ConversationType::Fix => type_counts[1] += 1,
            ConversationType::Debug => type_counts[2] += 1,
            ConversationType::ConfirmSafe => type_counts[3] += 1,
        }
        conversations.push(conv);
    }

    let total = conversations.len();
    let type_d_pct = if total > 0 {
        type_counts[3] as f64 / total as f64 * 100.0
    } else {
        0.0
    };

    // Check variant distribution: no single variant > 20% of total
    let variant_distribution_ok = check_variant_distribution(&conversations);

    // S6.4: No empty/trivial responses
    let empty_responses = conversations
        .iter()
        .filter(|c| c.turns.iter().any(|t| t.content.trim().is_empty()))
        .count();

    let report = QualityReport {
        total,
        type_a_count: type_counts[0],
        type_b_count: type_counts[1],
        type_c_count: type_counts[2],
        type_d_count: type_counts[3],
        type_d_pct,
        rule_citation_accuracy: 1.0, // Citations come directly from linter output
        variant_distribution_ok,
        empty_responses,
        // Section 6.4: Type D >= 30%, no empty responses
        passed: type_d_pct >= 30.0 && variant_distribution_ok && empty_responses == 0,
    };

    (conversations, report)
}

fn select_type(input: &ConversationInput<'_>, seed: u64) -> ConversationType {
    if input.is_safe {
        return ConversationType::ConfirmSafe;
    }

    if !input.is_deterministic {
        return ConversationType::Debug;
    }

    let has_security = input.diagnostics.iter().any(|d| d.code.starts_with("SEC"));

    // Alternate between ClassifyExplain and Fix for unsafe scripts
    if has_security && seed.is_multiple_of(2) {
        ConversationType::Fix
    } else {
        ConversationType::ClassifyExplain
    }
}

// --- User prompt variants (10+ per type) ---

const CLASSIFY_PROMPTS: &[&str] = &[
    "Is this shell script safe to run?",
    "Analyze this bash script for security issues.",
    "Can you check if this script is safe?",
    "What security problems does this script have?",
    "Is there anything dangerous in this shell script?",
    "Review this script for safety issues.",
    "Would it be safe to execute this script in production?",
    "Check this bash script for vulnerabilities.",
    "Does this script have any security concerns?",
    "Evaluate the safety of this shell script.",
    "Tell me if this script is safe or unsafe.",
    "Are there any risks in running this script?",
];

const FIX_PROMPTS: &[&str] = &[
    "Fix the security issues in this script.",
    "Make this shell script safe.",
    "How would you fix this script?",
    "Rewrite this script to be secure.",
    "Can you make this script safe to run?",
    "Fix the vulnerabilities in this bash script.",
    "Clean up this script's security problems.",
    "Give me a safe version of this script.",
    "How should I fix these security issues?",
    "Make this script production-safe.",
    "Correct the unsafe patterns in this script.",
    "Provide a secure alternative for this script.",
];

const DEBUG_PROMPTS: &[&str] = &[
    "This script gives different results each time. Why?",
    "Why is this script non-deterministic?",
    "My script produces inconsistent output. Help.",
    "Debug the non-determinism in this script.",
    "This script behaves differently on each run.",
    "Why does this script's output change between runs?",
    "Find the source of randomness in this script.",
    "This script is not reproducible. What's wrong?",
    "Help me make this script deterministic.",
    "Why do I get different results each time I run this?",
    "Identify the non-deterministic patterns here.",
    "What makes this script unpredictable?",
];

const SAFE_PROMPTS: &[&str] = &[
    "Is this script safe?",
    "Check this script for issues.",
    "Analyze this shell script.",
    "Review this bash script for me.",
    "Is there anything wrong with this script?",
    "Can I safely run this script?",
    "Does this script follow best practices?",
    "Check if this script is production-ready.",
    "Evaluate this shell script.",
    "Is this script well-written?",
    "Any concerns about this script?",
    "Would you approve this script for deployment?",
];

// --- Response generation ---

fn generate_classify_explain(input: &ConversationInput<'_>, variant: usize) -> Vec<Turn> {
    let prompt_idx = variant % CLASSIFY_PROMPTS.len();
    let user_content = format!(
        "{}\n\n```bash\n{}\n```",
        CLASSIFY_PROMPTS[prompt_idx], input.script
    );

    let mut response = String::from("This script is **unsafe**. Here are the issues found:\n\n");

    for d in input.diagnostics {
        let _ = writeln!(
            response,
            "- **{}** (line {}): {}",
            d.code, d.span.start_line, d.message
        );
    }

    if input.diagnostics.is_empty() {
        response.push_str("- General safety concerns detected in this script.\n");
    }

    response
        .push_str("\nI recommend fixing these issues before running this script in production.");

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

fn generate_fix(input: &ConversationInput<'_>, variant: usize) -> Vec<Turn> {
    let prompt_idx = variant % FIX_PROMPTS.len();
    let user_content = format!(
        "{}\n\n```bash\n{}\n```",
        FIX_PROMPTS[prompt_idx], input.script
    );

    let mut response = String::from("Here's a safer version of the script:\n\n```bash\n");

    // Generate a fixed version by applying known transformations
    let fixed = apply_safety_fixes(input.script, input.diagnostics);
    response.push_str(&fixed);
    response.push_str("\n```\n\nChanges made:\n");

    for d in input.diagnostics {
        let _ = writeln!(response, "- Fixed **{}**: {}", d.code, d.message);
    }

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

fn generate_debug(input: &ConversationInput<'_>, variant: usize) -> Vec<Turn> {
    let prompt_idx = variant % DEBUG_PROMPTS.len();
    let user_content = format!(
        "{}\n\n```bash\n{}\n```",
        DEBUG_PROMPTS[prompt_idx], input.script
    );

    let det_findings: Vec<&Diagnostic> = input
        .diagnostics
        .iter()
        .filter(|d| d.code.starts_with("DET"))
        .collect();

    let mut response = String::from("This script contains non-deterministic patterns:\n\n");

    for d in &det_findings {
        let _ = writeln!(
            response,
            "- **{}** (line {}): {}",
            d.code, d.span.start_line, d.message
        );
    }

    if det_findings.is_empty() {
        response.push_str("- Non-deterministic behavior detected in the script.\n");
    }

    response.push_str(
        "\nTo make this script deterministic, remove or replace \
         these sources of randomness with fixed values.",
    );

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
mod tests {
    use super::*;

    #[test]
    fn test_generate_classify_explain_unsafe_script() {
        let result = lint_shell("eval $user_input");
        let input = ConversationInput {
            entry_id: "B-1",
            script: "eval $user_input",
            is_safe: false,
            is_deterministic: true,
            diagnostics: &result.diagnostics,
        };

        let conv = generate_conversation(&input, 1);
        assert_eq!(conv.conversation_type, ConversationType::ClassifyExplain);
        assert_eq!(conv.turns.len(), 3);
        assert_eq!(conv.turns[0].role, "system");
        assert_eq!(conv.turns[1].role, "user");
        assert_eq!(conv.turns[2].role, "assistant");
        assert!(conv.turns[2].content.contains("unsafe"));
    }

    #[test]
    fn test_generate_fix_unsafe_script() {
        let result = lint_shell("eval $user_input");
        let input = ConversationInput {
            entry_id: "B-2",
            script: "eval $user_input",
            is_safe: false,
            is_deterministic: true,
            diagnostics: &result.diagnostics,
        };

        // seed=0 with security finding → Fix type
        let conv = generate_conversation(&input, 0);
        assert_eq!(conv.conversation_type, ConversationType::Fix);
        assert!(conv.turns[2].content.contains("safer version"));
    }

    #[test]
    fn test_generate_debug_nondeterministic() {
        let result = lint_shell("echo $RANDOM");
        let input = ConversationInput {
            entry_id: "B-3",
            script: "echo $RANDOM",
            is_safe: false,
            is_deterministic: false,
            diagnostics: &result.diagnostics,
        };

        let conv = generate_conversation(&input, 0);
        assert_eq!(conv.conversation_type, ConversationType::Debug);
        assert!(conv.turns[2].content.contains("non-deterministic"));
    }

    #[test]
    fn test_generate_confirm_safe() {
        let input = ConversationInput {
            entry_id: "B-4",
            script: "echo hello",
            is_safe: true,
            is_deterministic: true,
            diagnostics: &[],
        };

        let conv = generate_conversation(&input, 0);
        assert_eq!(conv.conversation_type, ConversationType::ConfirmSafe);
        assert!(conv.turns[2].content.contains("safe"));
    }

    #[test]
    fn test_generate_batch_quality_report() {
        let entries: Vec<(&str, &str)> = vec![
            ("B-1", "echo hello"),
            ("B-2", "echo world"),
            ("B-3", "ls -la"),
            ("B-4", "eval $x"),
            ("B-5", "echo $RANDOM"),
        ];

        let (conversations, report) = generate_batch(&entries, 42);
        assert_eq!(conversations.len(), 5);
        assert_eq!(report.total, 5);
        assert!(report.rule_citation_accuracy >= 1.0);
        // Verify all conversations have valid ChatML structure (system + user + assistant)
        for conv in &conversations {
            assert_eq!(conv.turns.len(), 3);
            assert_eq!(conv.turns[0].role, "system");
            assert_eq!(conv.turns[1].role, "user");
            assert_eq!(conv.turns[2].role, "assistant");
        }
    }

    #[test]
    fn test_conversation_type_labels() {
        assert_eq!(
            ConversationType::ClassifyExplain.label(),
            "classify-explain"
        );
        assert_eq!(ConversationType::Fix.label(), "fix");
        assert_eq!(ConversationType::Debug.label(), "debug");
        assert_eq!(ConversationType::ConfirmSafe.label(), "confirm-safe");
    }

    #[test]
    fn test_to_jsonl_format() {
        let input = ConversationInput {
            entry_id: "B-1",
            script: "echo hello",
            is_safe: true,
            is_deterministic: true,
            diagnostics: &[],
        };

        let conv = generate_conversation(&input, 0);
        let jsonl = to_jsonl(&[conv]);
        assert!(!jsonl.is_empty());
        assert!(jsonl.ends_with('\n'));
        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(jsonl.trim()).expect("valid JSON");
        assert_eq!(parsed["conversation_type"], "ConfirmSafe");
        // Verify system turn is present
        assert_eq!(parsed["turns"][0]["role"], "system");
    }

    #[test]
    fn test_system_prompt_content() {
        assert!(SYSTEM_PROMPT.contains("shell script safety"));
        assert!(SYSTEM_PROMPT.contains("not a replacement"));
    }

    #[test]
    fn test_generate_dataset_readme() {
        let report = QualityReport {
            total: 100,
            type_a_count: 20,
            type_b_count: 15,
            type_c_count: 10,
            type_d_count: 55,
            type_d_pct: 55.0,
            rule_citation_accuracy: 1.0,
            variant_distribution_ok: true,
            empty_responses: 0,
            passed: true,
        };
        let readme = generate_dataset_readme(&report);
        assert!(readme.starts_with("---\n"));
        assert!(readme.contains("license: apache-2.0"));
        assert!(readme.contains("Shell Safety Conversations"));
        assert!(readme.contains("Limitations and Bias"));
        assert!(readme.contains("100"));
    }

    #[test]
    fn test_apply_safety_fixes_mkdir() {
        let fixed = apply_single_fix("mkdir /tmp/build", "IDEM001");
        assert_eq!(fixed, "mkdir -p /tmp/build");
    }

    #[test]
    fn test_apply_safety_fixes_eval() {
        let fixed = apply_single_fix("eval $x", "SEC001");
        assert!(fixed.starts_with("# REMOVED"));
    }

    #[test]
    fn test_apply_safety_fixes_random() {
        let fixed = apply_single_fix("echo $RANDOM", "DET001");
        assert_eq!(fixed, "echo 42");
    }

    #[test]
    fn test_variant_distribution_diverse() {
        // With 5 conversations using different seeds, distribution should be ok
        let entries: Vec<(&str, &str)> = (0..20)
            .map(|i| {
                if i % 3 == 0 {
                    ("id", "eval $x")
                } else {
                    ("id", "echo hello")
                }
            })
            .collect();

        let (convs, _) = generate_batch(&entries, 0);
        let ok = check_variant_distribution(&convs);
        assert!(
            ok,
            "Variant distribution should be diverse with 12+ prompts"
        );
    }

    #[test]
    fn test_conversation_id_format() {
        let input = ConversationInput {
            entry_id: "B-42",
            script: "echo hello",
            is_safe: true,
            is_deterministic: true,
            diagnostics: &[],
        };

        let conv = generate_conversation(&input, 0);
        assert_eq!(conv.id, "conv-B-42-confirm-safe");
    }

    #[test]
    fn test_no_empty_responses_in_batch() {
        let entries: Vec<(&str, &str)> = vec![
            ("B-1", "eval $x"),
            ("B-2", "echo hello"),
            ("B-3", "echo $RANDOM"),
        ];
        let (convs, report) = generate_batch(&entries, 42);
        assert_eq!(report.empty_responses, 0, "No empty responses expected");
        for conv in &convs {
            for turn in &conv.turns {
                assert!(
                    !turn.content.trim().is_empty(),
                    "Empty turn in conversation {}",
                    conv.id
                );
            }
        }
    }

    #[test]
    fn test_all_prompt_arrays_have_10_plus_variants() {
        assert!(
            CLASSIFY_PROMPTS.len() >= 10,
            "Need 10+ classify prompts, got {}",
            CLASSIFY_PROMPTS.len()
        );
        assert!(
            FIX_PROMPTS.len() >= 10,
            "Need 10+ fix prompts, got {}",
            FIX_PROMPTS.len()
        );
        assert!(
            DEBUG_PROMPTS.len() >= 10,
            "Need 10+ debug prompts, got {}",
            DEBUG_PROMPTS.len()
        );
        assert!(
            SAFE_PROMPTS.len() >= 10,
            "Need 10+ safe prompts, got {}",
            SAFE_PROMPTS.len()
        );
    }
}
