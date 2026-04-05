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

include!("conversations_generate.rs");
