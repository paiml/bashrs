
    #[test]
    fn test_transform_semantic_delta_arith_no_short_circuit() {
        let t = Transform::ArithToTest {
            preserve_short_circuit: false,
            overflow_behavior: OverflowSemantics::Wrap,
        };
        let delta = t.semantic_delta();
        assert_eq!(delta, Some(SemanticDelta::ShortCircuitLost));
    }

    #[test]
    fn test_transform_semantic_delta_arith_with_short_circuit() {
        let t = Transform::ArithToTest {
            preserve_short_circuit: true,
            overflow_behavior: OverflowSemantics::Wrap,
        };
        let delta = t.semantic_delta();
        assert_eq!(delta, None);
    }

    #[test]
    fn test_transform_semantic_delta_dialect_migration() {
        let t = Transform::DialectMigration {
            source: ShellDialect::Bash5_2,
            target: ShellDialect::Posix,
            feature: SyntaxFeature::BashArrays,
            semantic_delta: Some(SemanticDelta::ArraySemantics),
        };
        let delta = t.semantic_delta();
        assert_eq!(delta, Some(SemanticDelta::ArraySemantics));
    }

    #[test]
    fn test_transform_semantic_delta_dialect_migration_none() {
        let t = Transform::DialectMigration {
            source: ShellDialect::Bash5_2,
            target: ShellDialect::Posix,
            feature: SyntaxFeature::BashArrays,
            semantic_delta: None,
        };
        let delta = t.semantic_delta();
        assert_eq!(delta, None);
    }

    #[test]
    fn test_transform_semantic_delta_sequence() {
        let seq = Transform::Sequence(vec![
            Transform::ArithToTest {
                preserve_short_circuit: false,
                overflow_behavior: OverflowSemantics::Wrap,
            },
            Transform::Identity,
        ]);
        let delta = seq.semantic_delta();
        assert_eq!(delta, Some(SemanticDelta::ShortCircuitLost));
    }

    #[test]
    fn test_transform_semantic_delta_sequence_multiple() {
        let seq = Transform::Sequence(vec![
            Transform::ArithToTest {
                preserve_short_circuit: false,
                overflow_behavior: OverflowSemantics::Wrap,
            },
            Transform::DialectMigration {
                source: ShellDialect::Bash5_2,
                target: ShellDialect::Posix,
                feature: SyntaxFeature::BashArrays,
                semantic_delta: Some(SemanticDelta::SignalHandling),
            },
        ]);
        let delta = seq.semantic_delta();
        // Composing ShortCircuitLost with SignalHandling gives ArraySemantics
        assert_eq!(delta, Some(SemanticDelta::ArraySemantics));
    }

    #[test]
    fn test_transform_semantic_delta_identity() {
        assert_eq!(Transform::Identity.semantic_delta(), None);
    }

    #[test]
    fn test_transform_semantic_delta_whitespace() {
        let t = Transform::WhitespaceNormalize {
            context: WhitespaceContext::Command,
            preserved: IntervalSet::new(),
        };
        assert_eq!(t.semantic_delta(), None);
    }

    // ===== Transform description Tests =====

    #[test]
    fn test_transform_description_identity() {
        assert_eq!(Transform::Identity.description(), "identity");
    }

    #[test]
    fn test_transform_description_arith_with_short_circuit() {
        let t = Transform::ArithToTest {
            preserve_short_circuit: true,
            overflow_behavior: OverflowSemantics::Wrap,
        };
        assert!(t.description().contains("preserving short-circuit"));
    }

    #[test]
    fn test_transform_description_arith_no_short_circuit() {
        let t = Transform::ArithToTest {
            preserve_short_circuit: false,
            overflow_behavior: OverflowSemantics::Wrap,
        };
        assert!(t.description().contains("losing short-circuit"));
    }

    #[test]
    fn test_transform_description_sequence() {
        let seq = Transform::Sequence(vec![Transform::Identity, Transform::Identity]);
        let desc = seq.description();
        assert!(desc.contains("sequence"));
        assert!(desc.contains("→"));
    }

    #[test]
    fn test_transform_description_dialect_migration() {
        let t = Transform::DialectMigration {
            source: ShellDialect::Bash5_2,
            target: ShellDialect::Posix,
            feature: SyntaxFeature::BashArrays,
            semantic_delta: None,
        };
        let desc = t.description();
        assert!(desc.contains("migrate"));
        assert!(desc.contains("Bash"));
        assert!(desc.contains("POSIX"));
    }

    #[test]
    fn test_transform_description_whitespace() {
        let t = Transform::WhitespaceNormalize {
            context: WhitespaceContext::Arithmetic,
            preserved: IntervalSet::new(),
        };
        let desc = t.description();
        assert!(desc.contains("whitespace"));
        assert!(desc.contains("Arithmetic"));
    }

    // ===== Transform is_semantic_preserving Tests =====

    #[test]
    fn test_transform_is_semantic_preserving_quote_expansion() {
        let t = Transform::QuoteExpansion {
            kind: QuoteKind::Single,
            reason: QuoteReason::GlobExpansion,
            proof: SexprProof::identity(),
        };
        assert!(t.is_semantic_preserving());
    }

    #[test]
    fn test_transform_is_semantic_preserving_dialect_migration() {
        let t = Transform::DialectMigration {
            source: ShellDialect::Bash5_2,
            target: ShellDialect::Posix,
            feature: SyntaxFeature::BashArrays,
            semantic_delta: None,
        };
        assert!(!t.is_semantic_preserving());
    }

    #[test]
    fn test_transform_is_semantic_preserving_sequence_all_preserving() {
        let seq = Transform::Sequence(vec![
            Transform::Identity,
            Transform::WhitespaceNormalize {
                context: WhitespaceContext::Command,
                preserved: IntervalSet::new(),
            },
        ]);
        assert!(seq.is_semantic_preserving());
    }

    #[test]
    fn test_transform_is_semantic_preserving_sequence_one_not() {
        let seq = Transform::Sequence(vec![
            Transform::Identity,
            Transform::ArithToTest {
                preserve_short_circuit: false,
                overflow_behavior: OverflowSemantics::Wrap,
            },
        ]);
        assert!(!seq.is_semantic_preserving());
    }

    // ===== IntervalSet Tests =====

    #[test]
    fn test_interval_set_default() {
        let set: IntervalSet<BytePos> = IntervalSet::default();
        assert!(!set.contains(BytePos(0)));
    }

    #[test]
    fn test_interval_set_empty() {
        let set: IntervalSet<BytePos> = IntervalSet::new();
        assert!(!set.contains(BytePos(100)));
    }

    #[test]
    fn test_interval_set_single_interval() {
        let mut set = IntervalSet::new();
        set.insert(BytePos(10)..BytePos(20));
        assert!(!set.contains(BytePos(9)));
        assert!(set.contains(BytePos(10)));
        assert!(set.contains(BytePos(15)));
        assert!(set.contains(BytePos(19)));
        assert!(!set.contains(BytePos(20)));
    }

    #[test]
    fn test_interval_set_merge_adjacent() {
        let mut set = IntervalSet::new();
        set.insert(BytePos(0)..BytePos(10));
        set.insert(BytePos(10)..BytePos(20));
        // Should merge since they're adjacent
        assert!(set.contains(BytePos(5)));
        assert!(set.contains(BytePos(15)));
    }

    #[test]
    fn test_interval_set_no_merge_gap() {
        let mut set = IntervalSet::new();
        set.insert(BytePos(0)..BytePos(10));
        set.insert(BytePos(20)..BytePos(30));
        assert!(set.contains(BytePos(5)));
        assert!(!set.contains(BytePos(15)));
        assert!(set.contains(BytePos(25)));
    }

    // ===== TransformId Tests =====

    #[test]
    fn test_transform_id_default() {
        let id1 = TransformId::default();
        let id2 = TransformId::default();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_transform_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        let id1 = TransformId::new();
        let id2 = TransformId::new();
        set.insert(id1);
        set.insert(id2);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_transform_id_debug() {
        let id = TransformId::new();
        let debug_str = format!("{:?}", id);
        assert!(debug_str.contains("TransformId"));
    }

    // ===== SexprProof Tests =====

    #[test]
    fn test_sexpr_proof_clone() {
        let proof = SexprProof::new("(test)".to_string());
        let cloned = proof.clone();
        assert_eq!(proof.formula, cloned.formula);
        assert_eq!(proof.is_valid, cloned.is_valid);
    }

    #[test]
    fn test_sexpr_proof_debug() {
        let proof = SexprProof::identity();
        let debug_str = format!("{:?}", proof);
        assert!(debug_str.contains("SexprProof"));
    }

    // ===== Transform Clone/Debug Tests =====

    #[test]
    fn test_transform_clone() {
        let t = Transform::QuoteExpansion {
            kind: QuoteKind::Double,
            reason: QuoteReason::ParameterExpansion,
            proof: SexprProof::identity(),
        };
        let cloned = t.clone();
        assert!(matches!(cloned, Transform::QuoteExpansion { .. }));
    }

    #[test]
    fn test_transform_debug() {
        let transforms = vec![
            Transform::Identity,
            Transform::WhitespaceNormalize {
                context: WhitespaceContext::Command,
                preserved: IntervalSet::new(),
            },
            Transform::QuoteExpansion {
                kind: QuoteKind::Single,
                reason: QuoteReason::WordSplitting,
                proof: SexprProof::identity(),
            },
            Transform::ArithToTest {
                preserve_short_circuit: true,
                overflow_behavior: OverflowSemantics::Wrap,
            },
            Transform::Sequence(vec![Transform::Identity]),
            Transform::DialectMigration {
                source: ShellDialect::Bash5_2,
                target: ShellDialect::Posix,
                feature: SyntaxFeature::BashArrays,
                semantic_delta: None,
            },
        ];
        for t in transforms {
            let _ = format!("{:?}", t);

