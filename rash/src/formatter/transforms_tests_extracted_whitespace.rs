
    #[test]
    fn test_whitespace_context_clone() {
        let ctx = WhitespaceContext::HereDoc {
            delimiter: "EOF",
            strip_tabs: false,
        };
        let cloned = ctx;
        assert_eq!(ctx, cloned);
    }

    // ===== QuoteKind Tests =====

    #[test]
    fn test_quote_kind_variants() {
        let kinds = [
            QuoteKind::Single,
            QuoteKind::Double,
            QuoteKind::Backslash,
            QuoteKind::None,
        ];
        for kind in kinds {
            let _ = format!("{:?}", kind);
        }
    }

    #[test]
    fn test_quote_kind_equality() {
        assert_eq!(QuoteKind::Single, QuoteKind::Single);
        assert_ne!(QuoteKind::Single, QuoteKind::Double);
    }

    // ===== QuoteReason Tests =====

    #[test]
    fn test_quote_reason_variants() {
        let reasons = [
            QuoteReason::WordSplitting,
            QuoteReason::GlobExpansion,
            QuoteReason::ParameterExpansion,
            QuoteReason::CommandSubstitution,
        ];
        for reason in reasons {
            let _ = format!("{:?}", reason);
        }
    }

    #[test]
    fn test_quote_reason_equality() {
        assert_eq!(QuoteReason::WordSplitting, QuoteReason::WordSplitting);
        assert_ne!(QuoteReason::WordSplitting, QuoteReason::GlobExpansion);
    }

    // ===== QuoteType Tests =====

    #[test]
    fn test_quote_type_variants() {
        let types = [
            QuoteType::Single,
            QuoteType::Double,
            QuoteType::DollarSingle,
            QuoteType::DollarDouble,
        ];
        for qt in types {
            let _ = format!("{:?}", qt);
        }
    }

    #[test]
    fn test_quote_type_equality() {
        assert_eq!(QuoteType::Single, QuoteType::Single);
        assert_ne!(QuoteType::Single, QuoteType::DollarSingle);
    }

    // ===== OverflowSemantics Tests =====

    #[test]
    fn test_overflow_semantics_variants() {
        let semantics = [
            OverflowSemantics::Wrap,
            OverflowSemantics::Saturate,
            OverflowSemantics::Trap,
        ];
        for s in semantics {
            let _ = format!("{:?}", s);
        }
    }

    #[test]
    fn test_overflow_semantics_equality() {
        assert_eq!(OverflowSemantics::Wrap, OverflowSemantics::Wrap);
        assert_ne!(OverflowSemantics::Wrap, OverflowSemantics::Saturate);
    }

    // ===== SemanticDelta Tests =====

    #[test]
    fn test_semantic_delta_is_preserving() {
        assert!(SemanticDelta::None.is_preserving());
        assert!(!SemanticDelta::ShortCircuitLost.is_preserving());
        assert!(!SemanticDelta::ArraySemantics.is_preserving());
        assert!(!SemanticDelta::ArithmeticPrecision(32).is_preserving());
        assert!(!SemanticDelta::SignalHandling.is_preserving());
        assert!(!SemanticDelta::ExitCodePropagation.is_preserving());
    }

    #[test]
    fn test_semantic_delta_compose_different_types() {
        // Composing different non-None types results in ArraySemantics
        let delta1 = SemanticDelta::ShortCircuitLost;
        let delta2 = SemanticDelta::SignalHandling;
        let composed = delta1.compose(&delta2);
        assert_eq!(composed, SemanticDelta::ArraySemantics);
    }

    #[test]
    fn test_semantic_delta_descriptions_all() {
        assert_eq!(
            SemanticDelta::ArithmeticPrecision(64).description(),
            "arithmetic precision changed"
        );
        assert_eq!(
            SemanticDelta::SignalHandling.description(),
            "signal handling semantics differ"
        );
        assert_eq!(
            SemanticDelta::ExitCodePropagation.description(),
            "exit code propagation differs"
        );
    }

    #[test]
    fn test_semantic_delta_equality() {
        assert_eq!(SemanticDelta::None, SemanticDelta::None);
        assert_eq!(
            SemanticDelta::ArithmeticPrecision(16),
            SemanticDelta::ArithmeticPrecision(16)
        );
        assert_ne!(
            SemanticDelta::ArithmeticPrecision(16),
            SemanticDelta::ArithmeticPrecision(32)
        );
    }

    // ===== Transform Compose Tests =====

    #[test]
    fn test_transform_compose_sequence_with_single() {
        let seq = Transform::Sequence(vec![Transform::Identity]);
        let single = Transform::WhitespaceNormalize {
            context: WhitespaceContext::Command,
            preserved: IntervalSet::new(),
        };

        let composed = seq.compose(single);
        if let Transform::Sequence(v) = composed {
            assert_eq!(v.len(), 2);
        } else {
            panic!("Expected Sequence");
        }
    }

    #[test]
    fn test_transform_compose_single_with_sequence() {
        let single = Transform::WhitespaceNormalize {
            context: WhitespaceContext::Command,
            preserved: IntervalSet::new(),
        };
        let seq = Transform::Sequence(vec![Transform::Identity]);

        let composed = single.compose(seq);
        if let Transform::Sequence(v) = composed {
            assert_eq!(v.len(), 2);
        } else {
            panic!("Expected Sequence");
        }
    }

    #[test]
    fn test_transform_compose_two_singles() {
        let t1 = Transform::ArithToTest {
            preserve_short_circuit: true,
            overflow_behavior: OverflowSemantics::Wrap,
        };
        let t2 = Transform::ArithToTest {
            preserve_short_circuit: false,
            overflow_behavior: OverflowSemantics::Saturate,
        };

        let composed = t1.compose(t2);
        if let Transform::Sequence(v) = composed {
            assert_eq!(v.len(), 2);
        } else {
            panic!("Expected Sequence");
        }
    }

    #[test]
    fn test_transform_identity_right() {
        let t = Transform::ArithToTest {
            preserve_short_circuit: true,
            overflow_behavior: OverflowSemantics::Trap,
        };
        let composed = t.compose(Transform::Identity);
        assert!(matches!(composed, Transform::ArithToTest { .. }));
    }

    // ===== Transform semantic_delta Tests =====

include!("transforms_tests_extracted_whitespace_transform.rs");
