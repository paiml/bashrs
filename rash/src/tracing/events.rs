// Trace Events - Core data structures for bashrs tracing
//
// Task: TRACE-001-B - TraceEvent enum and sub-events
// Source: bashrs-tracing-while-purify-lint-spec.md (Section 3.1)
// Optimization: bashrs-tracing-architectural-refinements.md (Refinement 5 - Diff-based storage)
//
// Design Philosophy:
// - Memory-efficient: Store nodes by reference, diffs as patches (99.25% savings)
// - Semantic information: Full AST awareness (not text-based like shellcheck)
// - Significance-aware: Every event has importance ranking
//
// Performance Target: <10% overhead (OOPSLA2 2024 standard)

use crate::tracing::TraceSignificance;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Unique identifier for AST nodes (cheap to copy, 8 bytes)
///
/// Optimization: Instead of cloning full AST nodes (can be >10 KB),
/// we store a reference ID and reconstruct from registry on demand.
pub type AstNodeId = u64;

/// Unique identifier for transformations (for dependency tracking)
pub type TransformationId = u64;

/// Rule identifier (e.g., IDEM001, DET003, SEC002)
pub type RuleId = String;

/// Source code span (line:column range)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub line_start: usize,
    pub col_start: usize,
    pub line_end: usize,
    pub col_end: usize,
}

impl Span {
    #[must_use]
    pub const fn new(line_start: usize, col_start: usize, line_end: usize, col_end: usize) -> Self {
        Self {
            line_start,
            col_start,
            line_end,
            col_end,
        }
    }

    #[must_use]
    pub const fn single_line(line: usize, col_start: usize, col_end: usize) -> Self {
        Self::new(line, col_start, line, col_end)
    }
}

/// Minimal diff representation (not full clone)
///
/// Optimization: Instead of storing full node_before + node_after,
/// store only the diff. Typical size: <100 bytes vs 20 KB.
///
/// Memory savings: 99.25% (20 KB → 150 bytes per event)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AstNodePatch {
    /// Added a flag to command (e.g., mkdir → mkdir -p)
    AddedFlag { flag: String },

    /// Removed a flag from command
    RemovedFlag { flag: String },

    /// Replaced an argument at index
    ReplacedArgument {
        index: usize,
        old: String,
        new: String,
    },

    /// Replaced an expression
    ReplacedExpression { old_expr: String, new_expr: String },

    /// Added quotes around variable (e.g., $foo → "$foo")
    AddedQuotes { variable: String },

    /// Removed $RANDOM variable
    RemovedRandomVar,

    /// Replaced timestamp with fixed value
    ReplacedTimestamp {
        old_pattern: String,
        new_value: String,
    },

    /// Generic transformation (fallback)
    Generic { description: String },
}

/// Core trace event enum
///
/// Every event during bash parsing, purification, linting, and generation
/// is recorded as one of these events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraceEvent {
    // ===== PARSING EVENTS =====
    Parse(ParseEvent),

    // ===== PURIFICATION EVENTS =====
    Purify(PurifyEvent),

    // ===== LINTING EVENTS =====
    Lint(LintEvent),

    // ===== GENERATION EVENTS =====
    Generate(GenerateEvent),
}

impl TraceEvent {
    /// Calculate significance of this trace event
    ///
    /// Implements Refinement 1 (Trace Significance) from architectural spec.
    /// Higher significance events bubble to the top in UI.
    #[must_use]
    pub fn significance(&self) -> TraceSignificance {
        match self {
            // CRITICAL: Conflicts and security violations
            Self::Purify(PurifyEvent::TransformationConflict { .. }) => TraceSignificance::Critical,
            Self::Lint(LintEvent::RuleEvaluated {
                violation: Some(v), ..
            }) if v.is_security => TraceSignificance::Critical,

            // HIGH: Transformations applied, parse/generation errors
            Self::Purify(PurifyEvent::TransformationApplied { .. }) => TraceSignificance::High,
            Self::Parse(ParseEvent::ParseError { .. }) => TraceSignificance::High,
            Self::Generate(GenerateEvent::GenerateError { .. }) => TraceSignificance::High,

            // MEDIUM: Transformations skipped, rule evaluations
            Self::Purify(PurifyEvent::TransformationSkipped { .. }) => TraceSignificance::Medium,
            Self::Lint(LintEvent::RuleEvaluated { .. }) => TraceSignificance::Medium,

            // LOW: Parse nodes, generation steps
            Self::Parse(ParseEvent::ParseNode { .. }) => TraceSignificance::Low,
            Self::Generate(GenerateEvent::GenerateCode { .. }) => TraceSignificance::Low,

            // TRACE: Start/complete events (structural only)
            Self::Parse(ParseEvent::ParseStart { .. }) => TraceSignificance::Trace,
            Self::Parse(ParseEvent::ParseComplete { .. }) => TraceSignificance::Trace,
            Self::Purify(PurifyEvent::PurifyStart { .. }) => TraceSignificance::Trace,
            Self::Purify(PurifyEvent::PurifyComplete { .. }) => TraceSignificance::Trace,
            Self::Lint(LintEvent::LintStart { .. }) => TraceSignificance::Trace,
            Self::Lint(LintEvent::LintComplete { .. }) => TraceSignificance::Trace,
            Self::Generate(GenerateEvent::GenerateStart { .. }) => TraceSignificance::Trace,
            Self::Generate(GenerateEvent::GenerateComplete { .. }) => TraceSignificance::Trace,
        }
    }

    /// Get a human-readable description of this event
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::Parse(e) => e.description(),
            Self::Purify(e) => e.description(),
            Self::Lint(e) => e.description(),
            Self::Generate(e) => e.description(),
        }
    }
}

/// Parsing events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParseEvent {
    ParseStart {
        source: String,
        line: usize,
        col: usize,
    },
    ParseNode {
        node_type: String,
        span: Span,
    },
    ParseComplete {
        node_count: usize,
        duration: Duration,
    },
    ParseError {
        error: String,
        span: Span,
    },
}

impl ParseEvent {
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::ParseStart { source, .. } => format!("Parse started: {source}"),
            Self::ParseNode { node_type, span } => {
                format!(
                    "Parsed {node_type} at {}:{}",
                    span.line_start, span.col_start
                )
            }
            Self::ParseComplete {
                node_count,
                duration,
            } => format!("Parse complete: {node_count} nodes in {duration:?}"),
            Self::ParseError { error, span } => {
                format!(
                    "Parse error at {}:{}: {error}",
                    span.line_start, span.col_start
                )
            }
        }
    }
}

/// Purification events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PurifyEvent {
    PurifyStart {
        node_count: usize,
    },
    TransformationApplied {
        id: TransformationId,
        rule_id: RuleId,
        node_id: AstNodeId,
        patch: AstNodePatch,
        reason: String,
        span: Span,
    },
    TransformationSkipped {
        rule_id: RuleId,
        node_id: AstNodeId,
        reason: String,
        span: Span,
    },
    TransformationConflict {
        id1: TransformationId,
        rule1: RuleId,
        id2: TransformationId,
        rule2: RuleId,
        node_id: AstNodeId,
        resolution: String,
        span: Span,
    },
    PurifyComplete {
        transformations_applied: usize,
        transformations_skipped: usize,
        conflicts: usize,
        duration: Duration,
    },
}

impl PurifyEvent {
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::PurifyStart { node_count } => format!("Purify started: {node_count} nodes"),
            Self::TransformationApplied {
                rule_id, reason, ..
            } => format!("Applied {rule_id}: {reason}"),
            Self::TransformationSkipped {
                rule_id, reason, ..
            } => format!("Skipped {rule_id}: {reason}"),
            Self::TransformationConflict {
                rule1, rule2, resolution, ..
            } => format!("Conflict: {rule1} vs {rule2} → {resolution}"),
            Self::PurifyComplete {
                transformations_applied,
                transformations_skipped,
                conflicts,
                duration,
            } => format!(
                "Purify complete: {transformations_applied} applied, {transformations_skipped} skipped, {conflicts} conflicts in {duration:?}"
            ),
        }
    }
}

/// Linting events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LintEvent {
    LintStart {
        node_count: usize,
    },
    RuleEvaluated {
        rule_id: RuleId,
        node_id: AstNodeId,
        passed: bool,
        violation: Option<Violation>,
        span: Span,
    },
    LintComplete {
        rules_evaluated: usize,
        violations: usize,
        duration: Duration,
    },
}

impl LintEvent {
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::LintStart { node_count } => format!("Lint started: {node_count} nodes"),
            Self::RuleEvaluated {
                rule_id,
                passed,
                violation,
                ..
            } => {
                if *passed {
                    format!("{rule_id}: passed")
                } else if let Some(v) = violation {
                    format!("{rule_id}: {}", v.message)
                } else {
                    format!("{rule_id}: failed")
                }
            }
            Self::LintComplete {
                rules_evaluated,
                violations,
                duration,
            } => format!(
                "Lint complete: {rules_evaluated} rules, {violations} violations in {duration:?}"
            ),
        }
    }
}

/// Generation events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenerateEvent {
    GenerateStart {
        node_count: usize,
    },
    GenerateCode {
        node_id: AstNodeId,
        bash_code: String,
        span: Span,
    },
    GenerateComplete {
        output_size: usize,
        duration: Duration,
    },
    GenerateError {
        error: String,
        span: Span,
    },
}

impl GenerateEvent {
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::GenerateStart { node_count } => format!("Generate started: {node_count} nodes"),
            Self::GenerateCode {
                bash_code, span, ..
            } => format!(
                "Generated code at {}:{}: {bash_code}",
                span.line_start, span.col_start
            ),
            Self::GenerateComplete {
                output_size,
                duration,
            } => format!("Generate complete: {output_size} bytes in {duration:?}"),
            Self::GenerateError { error, span } => {
                format!(
                    "Generate error at {}:{}: {error}",
                    span.line_start, span.col_start
                )
            }
        }
    }
}

/// Linter violation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Violation {
    pub rule_id: RuleId,
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub suggestion: Option<String>,
    pub is_security: bool,
}

/// Violation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Style,
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "events_tests_trace_001.rs"]
// FIXME(PMAT-238): mod tests_extracted;
