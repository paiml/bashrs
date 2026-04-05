#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_classify_failure_signals_all_pass() {
        let result = super::super::runner::CorpusResult {
            transpiled: true,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            lint_clean: true,
            cross_shell_agree: true,
            schema_valid: true,
            deterministic: true,
            ..Default::default()
        };
        let signals = classify_failure_signals(&result);
        assert!(signals.is_empty());
    }

    #[test]
    fn test_classify_failure_signals_transpile_fail() {
        let result = super::super::runner::CorpusResult {
            transpiled: false,
            ..Default::default()
        };
        let signals = classify_failure_signals(&result);
        assert_eq!(signals, vec!["A_transpile_fail"]);
    }

    #[test]
    fn test_classify_failure_signals_b3_and_g_fail() {
        let result = super::super::runner::CorpusResult {
            transpiled: true,
            output_contains: true,
            output_exact: true,
            output_behavioral: false,
            lint_clean: true,
            cross_shell_agree: false,
            schema_valid: true,
            deterministic: true,
            ..Default::default()
        };
        let signals = classify_failure_signals(&result);
        assert_eq!(signals.len(), 2);
        assert!(signals.contains(&"B3_behavioral_fail".to_string()));
        assert!(signals.contains(&"G_cross_shell_fail".to_string()));
    }

    #[test]
    fn test_classify_failure_signals_lint_fail() {
        let result = super::super::runner::CorpusResult {
            transpiled: true,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            lint_clean: false,
            cross_shell_agree: true,
            schema_valid: true,
            deterministic: true,
            ..Default::default()
        };
        let signals = classify_failure_signals(&result);
        assert_eq!(signals, vec!["D_lint_fail"]);
    }

    #[test]
    fn test_classify_failure_signals_containment_fail() {
        let result = super::super::runner::CorpusResult {
            transpiled: true,
            output_contains: false,
            output_exact: true,
            output_behavioral: true,
            lint_clean: true,
            cross_shell_agree: true,
            schema_valid: true,
            deterministic: true,
            ..Default::default()
        };
        let signals = classify_failure_signals(&result);
        assert!(signals.contains(&"B1_containment_fail".to_string()));
    }

    #[test]
    fn test_derive_fix_type_quoting() {
        assert_eq!(
            derive_fix_type("assignment_value:single_quote"),
            "quoting_strategy"
        );
    }

    #[test]
    fn test_derive_fix_type_ir_dispatch() {
        assert_eq!(derive_fix_type("ir_dispatch:Let"), "ir_node_handling");
    }

    #[test]
    fn test_derive_fix_type_string_emit() {
        assert_eq!(derive_fix_type("string_emit:literal"), "string_handling");
    }

    #[test]
    fn test_derive_fix_type_variable() {
        assert_eq!(
            derive_fix_type("variable_expansion:braced"),
            "expansion_strategy"
        );
    }

    #[test]
    fn test_derive_fix_type_redirect() {
        assert_eq!(derive_fix_type("redirect:file"), "redirect_handling");
        assert_eq!(derive_fix_type("redirect_emit:file"), "redirect_handling");
    }

    #[test]
    fn test_derive_fix_type_pipe() {
        assert_eq!(derive_fix_type("pipe_emit:chain"), "pipe_handling");
    }

    #[test]
    fn test_derive_fix_type_arithmetic() {
        assert_eq!(derive_fix_type("arithmetic:expr"), "arithmetic_strategy");
        assert_eq!(
            derive_fix_type("arithmetic_emit:expr"),
            "arithmetic_strategy"
        );
    }

    #[test]
    fn test_derive_fix_type_conditional() {
        assert_eq!(derive_fix_type("conditional:if"), "conditional_handling");
        assert_eq!(derive_fix_type("if_emit:elif"), "conditional_handling");
    }

    #[test]
    fn test_derive_fix_type_loop() {
        assert_eq!(derive_fix_type("loop_emit:for"), "loop_handling");
        assert_eq!(derive_fix_type("for_emit:range"), "loop_handling");
        assert_eq!(derive_fix_type("while_emit:cond"), "loop_handling");
    }

    #[test]
    fn test_derive_fix_type_function() {
        assert_eq!(derive_fix_type("function_emit:define"), "function_handling");
    }

    #[test]
    fn test_derive_fix_type_unknown() {
        assert_eq!(derive_fix_type("some_other:thing"), "some_other_strategy");
    }

    #[test]
    fn test_derive_fix_type_string_interpolation() {
        assert_eq!(
            derive_fix_type("string_interpolation:double"),
            "string_handling"
        );
    }

    #[test]
    fn test_derive_fix_type_command_substitution() {
        assert_eq!(
            derive_fix_type("command_substitution:backtick"),
            "substitution_strategy"
        );
    }

    #[test]
    fn test_pattern_store_empty() {
        let store = PatternStore {
            patterns: Vec::new(),
            total_entries: 100,
            total_failures: 0,
            version: "1.0.0".to_string(),
        };
        assert!(store.patterns.is_empty());
        assert_eq!(store.total_entries, 100);
    }

    #[test]
    fn test_shell_fix_pattern_serialization() {
        let pattern = ShellFixPattern {
            error_signal: "B3_behavioral_fail".to_string(),
            causal_decision: "assignment_value:single_quote".to_string(),
            fix_type: "quoting_strategy".to_string(),
            confidence: 0.85,
            evidence_ids: vec!["B-143".to_string()],
        };

        let json = serde_json::to_string(&pattern).unwrap();
        let deserialized: ShellFixPattern = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.error_signal, "B3_behavioral_fail");
        assert_eq!(deserialized.confidence, 0.85);
    }

    #[test]
    fn test_pattern_store_serialization() {
        let store = PatternStore {
            patterns: vec![ShellFixPattern {
                error_signal: "D_lint_fail".to_string(),
                causal_decision: "string_emit:unquoted".to_string(),
                fix_type: "string_handling".to_string(),
                confidence: 0.72,
                evidence_ids: vec!["B-100".to_string(), "B-200".to_string()],
            }],
            total_entries: 900,
            total_failures: 3,
            version: "1.0.0".to_string(),
        };

        let json = serde_json::to_string_pretty(&store).unwrap();
        let deserialized: PatternStore = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.patterns.len(), 1);
        assert_eq!(deserialized.total_entries, 900);
    }

    #[test]
    fn test_classify_multiple_failures() {
        let result = super::super::runner::CorpusResult {
            transpiled: true,
            output_contains: false,
            output_exact: false,
            output_behavioral: false,
            lint_clean: false,
            cross_shell_agree: false,
            schema_valid: true,
            deterministic: true,
            ..Default::default()
        };
        let signals = classify_failure_signals(&result);
        assert_eq!(signals.len(), 5);
    }
}
