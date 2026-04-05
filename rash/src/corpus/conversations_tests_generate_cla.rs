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

    #[test]
    fn test_to_entrenar_jsonl_format() {
        let convs = vec![Conversation {
            id: "conv-test".to_string(),
            conversation_type: ConversationType::ClassifyExplain,
            turns: vec![
                Turn {
                    role: "system",
                    content: "You are a safety analyzer.".to_string(),
                },
                Turn {
                    role: "user",
                    content: "Is eval $x safe?".to_string(),
                },
                Turn {
                    role: "assistant",
                    content: "No, eval on untrusted input is unsafe.".to_string(),
                },
            ],
        }];
        let jsonl = to_entrenar_jsonl(&convs);
        let parsed: serde_json::Value = serde_json::from_str(jsonl.trim()).expect("valid JSON");
        assert_eq!(parsed["instruction"], "Is eval $x safe?");
        assert_eq!(parsed["response"], "No, eval on untrusted input is unsafe.");
        assert_eq!(parsed["system"], "You are a safety analyzer.");
        // Must have ChatML-formatted text field for entrenar tokenization
        let text = parsed["text"].as_str().expect("text field");
        assert!(text.contains("<|im_start|>system"));
        assert!(text.contains("<|im_start|>user"));
        assert!(text.contains("<|im_start|>assistant"));
        assert!(text.contains("<|im_end|>"));
        assert!(text.contains("Is eval $x safe?"));
        assert!(text.contains("No, eval on untrusted input is unsafe."));
    }

    #[test]
    fn test_to_entrenar_jsonl_skips_empty() {
        let convs = vec![Conversation {
            id: "conv-empty".to_string(),
            conversation_type: ConversationType::ConfirmSafe,
            turns: vec![Turn {
                role: "system",
                content: "sys".to_string(),
            }],
        }];
        let jsonl = to_entrenar_jsonl(&convs);
        assert!(
            jsonl.is_empty(),
            "Should skip conversations without user/assistant turns"
        );
    }
}
