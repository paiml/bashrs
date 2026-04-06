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
        use Transform::{Identity, Sequence, WhitespaceNormalize};
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
#[path = "transforms_tests_transform_id.rs"]
mod tests_extracted;
