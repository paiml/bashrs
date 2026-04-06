use super::*;

#[test]
fn test_transform_identity() {
    let t1 = Transform::Identity;
    let t2 = Transform::WhitespaceNormalize {
        context: WhitespaceContext::Command,
        preserved: IntervalSet::new(),
    };

    let composed = t1.compose(t2.clone());
    assert!(matches!(composed, Transform::WhitespaceNormalize { .. }));
}

#[test]
fn test_transform_sequence_flattening() {
    let t1 = Transform::Identity;
    let t2 = Transform::Identity;
    let seq1 = Transform::Sequence(vec![t1, t2]);

    let t3 = Transform::Identity;
    let t4 = Transform::Identity;
    let seq2 = Transform::Sequence(vec![t3, t4]);

    let composed = seq1.compose(seq2);
    if let Transform::Sequence(transforms) = composed {
        assert_eq!(transforms.len(), 4);
    } else {
        panic!("Expected sequence");
    }
}

#[test]
fn test_whitespace_normalization_merge() {
    let mut preserved1 = IntervalSet::new();
    preserved1.insert(BytePos(0)..BytePos(10));

    let mut preserved2 = IntervalSet::new();
    preserved2.insert(BytePos(5)..BytePos(15));

    let t1 = Transform::WhitespaceNormalize {
        context: WhitespaceContext::Command,
        preserved: preserved1,
    };

    let t2 = Transform::WhitespaceNormalize {
        context: WhitespaceContext::Command,
        preserved: preserved2,
    };

    let composed = t1.compose(t2);
    if let Transform::WhitespaceNormalize { preserved, .. } = composed {
        assert!(preserved.contains(BytePos(7))); // Should be in merged range
    } else {
        panic!("Expected whitespace normalize");
    }
}

#[test]
fn test_semantic_delta_composition() {
    let delta1 = SemanticDelta::None;
    let delta2 = SemanticDelta::ShortCircuitLost;

    let composed = delta1.compose(&delta2);
    assert_eq!(composed, SemanticDelta::ShortCircuitLost);

    let delta3 = SemanticDelta::ArithmeticPrecision(32);
    let delta4 = SemanticDelta::ArithmeticPrecision(16);
    let composed2 = delta3.compose(&delta4);
    assert_eq!(composed2, SemanticDelta::ArithmeticPrecision(16));
}

#[test]
fn test_interval_set_operations() {
    let mut set = IntervalSet::new();
    set.insert(BytePos(0)..BytePos(10));
    set.insert(BytePos(15)..BytePos(25));

    assert!(set.contains(BytePos(5)));
    assert!(set.contains(BytePos(20)));
    assert!(!set.contains(BytePos(12)));

    // Test overlapping merge
    set.insert(BytePos(8)..BytePos(18));
    assert!(set.contains(BytePos(12))); // Should now be covered
}

#[test]
fn test_interval_set_union() {
    let mut set1 = IntervalSet::new();
    set1.insert(BytePos(0)..BytePos(10));

    let mut set2 = IntervalSet::new();
    set2.insert(BytePos(20)..BytePos(30));

    let union = set1.union(&set2);
    assert!(union.contains(BytePos(5)));
    assert!(union.contains(BytePos(25)));
    assert!(!union.contains(BytePos(15)));
}

#[test]
fn test_transform_semantic_preserving() {
    assert!(Transform::Identity.is_semantic_preserving());
    assert!(Transform::WhitespaceNormalize {
        context: WhitespaceContext::Command,
        preserved: IntervalSet::new(),
    }
    .is_semantic_preserving());

    assert!(Transform::ArithToTest {
        preserve_short_circuit: true,
        overflow_behavior: OverflowSemantics::Wrap,
    }
    .is_semantic_preserving());

    assert!(!Transform::ArithToTest {
        preserve_short_circuit: false,
        overflow_behavior: OverflowSemantics::Wrap,
    }
    .is_semantic_preserving());
}

#[test]
fn test_transform_descriptions() {
    let transform = Transform::QuoteExpansion {
        kind: QuoteKind::Double,
        reason: QuoteReason::WordSplitting,
        proof: SexprProof::identity(),
    };

    let desc = transform.description();
    assert!(desc.contains("Double"));
    assert!(desc.contains("WordSplitting"));
}

#[test]
fn test_semantic_delta_descriptions() {
    assert_eq!(SemanticDelta::None.description(), "no semantic change");
    assert_eq!(
        SemanticDelta::ShortCircuitLost.description(),
        "short-circuit evaluation lost"
    );
    assert_eq!(
        SemanticDelta::ArraySemantics.description(),
        "array semantics differ"
    );
}

#[test]
fn test_transform_id_uniqueness() {
    let id1 = TransformId::new();
    let id2 = TransformId::new();
    assert_ne!(id1, id2);
}

#[test]
fn test_sexpr_proof() {
    let proof = SexprProof::new("(= (quote x) x)".to_string());
    assert!(proof.is_valid);
    assert_eq!(proof.to_smt2(), "(assert (= (quote x) x))");

    let identity_proof = SexprProof::identity();
    assert_eq!(identity_proof.to_smt2(), "(assert (= x x))");
}

// ===== WhitespaceContext Tests =====

#[test]
fn test_whitespace_context_variants() {
    // Test all variants exist and can be created
    let cmd = WhitespaceContext::Command;
    let heredoc = WhitespaceContext::HereDoc {
        delimiter: "EOF",
        strip_tabs: true,
    };
    let quoted = WhitespaceContext::QuotedString {
        quote_type: QuoteType::Double,
    };
    let arith = WhitespaceContext::Arithmetic;
    let case = WhitespaceContext::CasePattern;
    let assign = WhitespaceContext::AssignmentValue {
        array_element: false,
    };

    // Test Debug trait
    let _ = format!("{:?}", cmd);
    let _ = format!("{:?}", heredoc);
    let _ = format!("{:?}", quoted);
    let _ = format!("{:?}", arith);
    let _ = format!("{:?}", case);
    let _ = format!("{:?}", assign);
}

#[test]
fn test_whitespace_context_equality() {
    assert_eq!(WhitespaceContext::Command, WhitespaceContext::Command);
    assert_eq!(WhitespaceContext::Arithmetic, WhitespaceContext::Arithmetic);
    assert_ne!(WhitespaceContext::Command, WhitespaceContext::Arithmetic);
}

#[cfg(test)]
mod transforms_tests_extracted_whitespace {
    use super::*;
    include!("transforms_tests_extracted_whitespace.rs");
}
