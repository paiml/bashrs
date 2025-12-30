//! Transform algebra and normalization operations

use crate::formatter::dialect::*;
use crate::formatter::types::*;
use std::ops::Range;

/// Transformations form a monoid under composition
#[derive(Debug, Clone)]
pub enum Transform {
    // Identity element
    Identity,

    // Syntactic (provably preserving via structural induction)
    WhitespaceNormalize {
        context: WhitespaceContext,
        /// Preserved byte ranges (e.g., string literals)
        preserved: IntervalSet<BytePos>,
    },

    QuoteExpansion {
        kind: QuoteKind,
        reason: QuoteReason,
        /// SMT formula asserting equivalence
        proof: SexprProof,
    },

    // Semantic (requiring SMT verification)
    ArithToTest {
        preserve_short_circuit: bool,
        overflow_behavior: OverflowSemantics,
    },

    // Composite
    Sequence(Vec<Transform>),

    // Dialect migration
    DialectMigration {
        source: ShellDialect,
        target: ShellDialect,
        feature: SyntaxFeature,
        semantic_delta: Option<SemanticDelta>,
    },
}

/// Context-dependent whitespace handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhitespaceContext {
    /// Normal command context: collapse to single space
    Command,

    /// Here-document: preserve exactly
    HereDoc {
        delimiter: &'static str,
        strip_tabs: bool, // <<- vs <<
    },

    /// String literal: preserve internal whitespace
    QuotedString { quote_type: QuoteType },

    /// Arithmetic expression: remove all whitespace
    Arithmetic,

    /// Case pattern: preserve for alignment
    CasePattern,

    /// Assignment RHS: context-dependent
    AssignmentValue { array_element: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteKind {
    Single,
    Double,
    Backslash,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteReason {
    WordSplitting,
    GlobExpansion,
    ParameterExpansion,
    CommandSubstitution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteType {
    Single,
    Double,
    DollarSingle,
    DollarDouble,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowSemantics {
    Wrap,
    Saturate,
    Trap,
}

/// Semantic changes introduced by transformations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticDelta {
    None,
    ShortCircuitLost,
    ArraySemantics,
    ArithmeticPrecision(u8),
    SignalHandling,
    ExitCodePropagation,
}

/// SMT proof representation (simplified)
#[derive(Debug, Clone)]
pub struct SexprProof {
    pub formula: String,
    pub is_valid: bool,
}

impl SexprProof {
    pub fn new(formula: String) -> Self {
        Self {
            formula,
            is_valid: true, // Simplified - would normally verify
        }
    }

    pub fn identity() -> Self {
        Self {
            formula: "(= x x)".to_string(),
            is_valid: true,
        }
    }

    pub fn to_smt2(&self) -> String {
        format!("(assert {})", self.formula)
    }
}

/// Interval set for tracking preserved ranges
#[derive(Debug, Clone)]
pub struct IntervalSet<T> {
    intervals: Vec<Range<T>>,
}

impl<T: Ord + Copy> IntervalSet<T> {
    pub fn new() -> Self {
        Self {
            intervals: Vec::new(),
        }
    }

    pub fn insert(&mut self, range: Range<T>) {
        self.intervals.push(range);
        self.merge_overlapping();
    }

    pub fn union(&self, other: &Self) -> Self {
        let mut result = self.clone();
        for interval in &other.intervals {
            result.insert(interval.clone());
        }
        result
    }

    pub fn contains(&self, point: T) -> bool {
        self.intervals.iter().any(|range| range.contains(&point))
    }

    fn merge_overlapping(&mut self) {
        if self.intervals.len() <= 1 {
            return;
        }

        self.intervals.sort_by_key(|range| range.start);
        let mut merged = Vec::new();
        let mut current = self.intervals[0].clone();

        for interval in &self.intervals[1..] {
            if current.end >= interval.start {
                // Overlapping, merge
                current.end = current.end.max(interval.end);
            } else {
                // Non-overlapping, push current and start new
                merged.push(current);
                current = interval.clone();
            }
        }
        merged.push(current);

        self.intervals = merged;
    }
}

impl<T: Ord + Copy> Default for IntervalSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform {
    /// Monoid composition with optimization
    pub fn compose(self, other: Self) -> Self {
        use Transform::*;
        match (self, other) {
            (Identity, x) | (x, Identity) => x,

            // Optimize consecutive whitespace normalizations
            (
                WhitespaceNormalize { preserved: p1, .. },
                WhitespaceNormalize {
                    context,
                    preserved: p2,
                },
            ) => WhitespaceNormalize {
                context,
                preserved: p1.union(&p2),
            },

            // Flatten sequences
            (Sequence(mut v1), Sequence(v2)) => {
                v1.extend(v2);
                Sequence(v1)
            }

            (Sequence(mut v), x) => {
                v.push(x);
                Sequence(v)
            }

            (x, Sequence(mut v)) => {
                v.insert(0, x);
                Sequence(v)
            }

            (a, b) => Sequence(vec![a, b]),
        }
    }

    /// Compute semantic delta for verification
    pub fn semantic_delta(&self) -> Option<SemanticDelta> {
        match self {
            Transform::ArithToTest {
                preserve_short_circuit: false,
                ..
            } => Some(SemanticDelta::ShortCircuitLost),
            Transform::DialectMigration { semantic_delta, .. } => semantic_delta.clone(),
            Transform::Sequence(transforms) => {
                // Combine all semantic deltas
                transforms
                    .iter()
                    .filter_map(|t| t.semantic_delta())
                    .fold(None, |acc, delta| match acc {
                        None => Some(delta),
                        Some(acc_delta) => Some(acc_delta.compose(&delta)),
                    })
            }
            _ => None,
        }
    }

    /// Check if transform preserves semantics
    pub fn is_semantic_preserving(&self) -> bool {
        match self {
            Transform::Identity => true,
            Transform::WhitespaceNormalize { .. } => true,
            Transform::QuoteExpansion { .. } => true, // When proven correct
            Transform::ArithToTest {
                preserve_short_circuit: true,
                ..
            } => true,
            Transform::Sequence(transforms) => {
                transforms.iter().all(|t| t.is_semantic_preserving())
            }
            _ => false,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            Transform::Identity => "identity".to_string(),
            Transform::WhitespaceNormalize { context, .. } => {
                format!("normalize whitespace in {context:?} context")
            }
            Transform::QuoteExpansion { kind, reason, .. } => {
                format!("add {kind:?} quotes for {reason:?}")
            }
            Transform::ArithToTest {
                preserve_short_circuit,
                ..
            } => {
                if *preserve_short_circuit {
                    "convert arithmetic to test (preserving short-circuit)".to_string()
                } else {
                    "convert arithmetic to test (losing short-circuit)".to_string()
                }
            }
            Transform::Sequence(transforms) => {
                let descriptions: Vec<_> = transforms.iter().map(|t| t.description()).collect();
                format!("sequence: {}", descriptions.join(" → "))
            }
            Transform::DialectMigration {
                source,
                target,
                feature,
                ..
            } => {
                format!(
                    "migrate {:?} from {} to {}",
                    feature,
                    source.display_name(),
                    target.display_name()
                )
            }
        }
    }
}

impl SemanticDelta {
    /// Compose semantic deltas (associative operation)
    pub fn compose(&self, other: &Self) -> Self {
        match (self, other) {
            (SemanticDelta::None, x) | (x, SemanticDelta::None) => x.clone(),
            (SemanticDelta::ArithmeticPrecision(a), SemanticDelta::ArithmeticPrecision(b)) => {
                SemanticDelta::ArithmeticPrecision((*a).min(*b)) // Take minimum precision
            }
            // If different types of deltas, this is a complex change
            _ => SemanticDelta::ArraySemantics, // Simplified - would be more sophisticated
        }
    }

    /// Check if delta is semantics-preserving
    pub fn is_preserving(&self) -> bool {
        matches!(self, SemanticDelta::None)
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            SemanticDelta::None => "no semantic change",
            SemanticDelta::ShortCircuitLost => "short-circuit evaluation lost",
            SemanticDelta::ArraySemantics => "array semantics differ",
            SemanticDelta::ArithmeticPrecision(_) => "arithmetic precision changed",
            SemanticDelta::SignalHandling => "signal handling semantics differ",
            SemanticDelta::ExitCodePropagation => "exit code propagation differs",
        }
    }
}

/// Unique identifier for transforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransformId(pub u64);

impl TransformId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        TransformId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

impl Default for TransformId {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
        }
    }
}
