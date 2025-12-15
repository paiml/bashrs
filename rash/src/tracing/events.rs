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

#[cfg(test)]
mod tests {
    use super::*;

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: TRACE-001-B-001 - Span creation
    #[test]
    fn test_trace_001_b_001_span_creation() {
        let span = Span::new(1, 5, 1, 10);
        assert_eq!(span.line_start, 1);
        assert_eq!(span.col_start, 5);
        assert_eq!(span.line_end, 1);
        assert_eq!(span.col_end, 10);
    }

    /// Test: TRACE-001-B-002 - Single-line span helper
    #[test]
    fn test_trace_001_b_002_single_line_span() {
        let span = Span::single_line(42, 10, 20);
        assert_eq!(span.line_start, 42);
        assert_eq!(span.line_end, 42);
        assert_eq!(span.col_start, 10);
        assert_eq!(span.col_end, 20);
    }

    /// Test: TRACE-001-B-003 - AstNodePatch variants
    #[test]
    fn test_trace_001_b_003_patch_variants() {
        let patch1 = AstNodePatch::AddedFlag {
            flag: "-p".to_string(),
        };
        let patch2 = AstNodePatch::RemovedRandomVar;
        assert!(matches!(patch1, AstNodePatch::AddedFlag { .. }));
        assert!(matches!(patch2, AstNodePatch::RemovedRandomVar));
    }

    /// Test: TRACE-001-B-004 - ParseEvent significance
    #[test]
    fn test_trace_001_b_004_parse_event_significance() {
        let start = TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        });
        assert_eq!(start.significance(), TraceSignificance::Trace);

        let error = TraceEvent::Parse(ParseEvent::ParseError {
            error: "Syntax error".to_string(),
            span: Span::single_line(1, 1, 10),
        });
        assert_eq!(error.significance(), TraceSignificance::High);
    }

    /// Test: TRACE-001-B-005 - PurifyEvent significance
    #[test]
    fn test_trace_001_b_005_purify_event_significance() {
        let applied = TraceEvent::Purify(PurifyEvent::TransformationApplied {
            id: 1,
            rule_id: "IDEM001".to_string(),
            node_id: 42,
            patch: AstNodePatch::AddedFlag {
                flag: "-p".to_string(),
            },
            reason: "Added -p for idempotency".to_string(),
            span: Span::single_line(5, 1, 10),
        });
        assert_eq!(applied.significance(), TraceSignificance::High);

        let conflict = TraceEvent::Purify(PurifyEvent::TransformationConflict {
            id1: 1,
            rule1: "IDEM001".to_string(),
            id2: 2,
            rule2: "SEC001".to_string(),
            node_id: 42,
            resolution: "IDEM001 takes precedence".to_string(),
            span: Span::single_line(5, 1, 10),
        });
        assert_eq!(conflict.significance(), TraceSignificance::Critical);
    }

    /// Test: TRACE-001-B-006 - Event descriptions
    #[test]
    fn test_trace_001_b_006_event_descriptions() {
        let event = TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        });
        assert!(event.description().contains("test.sh"));
    }

    /// Test: TRACE-001-B-007 - Violation with security flag
    #[test]
    fn test_trace_001_b_007_security_violation() {
        let violation = Violation {
            rule_id: "SEC001".to_string(),
            severity: Severity::Error,
            message: "Unquoted variable expansion".to_string(),
            span: Span::single_line(10, 5, 10),
            suggestion: Some("Quote the variable".to_string()),
            is_security: true,
        };

        let event = TraceEvent::Lint(LintEvent::RuleEvaluated {
            rule_id: "SEC001".to_string(),
            node_id: 42,
            passed: false,
            violation: Some(violation),
            span: Span::single_line(10, 5, 10),
        });

        assert_eq!(event.significance(), TraceSignificance::Critical);
    }

    /// Test: TRACE-001-B-008 - Serialization round-trip
    #[test]
    fn test_trace_001_b_008_serialization() {
        let event = TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        });

        let json = serde_json::to_string(&event).expect("Serialization failed");
        let deserialized: TraceEvent = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(event, deserialized);
    }

    // ===== COVERAGE IMPROVEMENT TESTS =====

    /// Test: ParseEvent descriptions for all variants
    #[test]
    fn test_parse_event_descriptions_all() {
        // ParseNode
        let node_event = ParseEvent::ParseNode {
            node_type: "Command".to_string(),
            span: Span::single_line(1, 1, 10),
        };
        assert!(node_event.description().contains("Command"));
        assert!(node_event.description().contains("1:1"));

        // ParseComplete
        let complete = ParseEvent::ParseComplete {
            node_count: 42,
            duration: Duration::from_millis(100),
        };
        assert!(complete.description().contains("42 nodes"));

        // ParseError
        let error = ParseEvent::ParseError {
            error: "unexpected token".to_string(),
            span: Span::single_line(5, 10, 15),
        };
        assert!(error.description().contains("unexpected token"));
        assert!(error.description().contains("5:10"));
    }

    /// Test: PurifyEvent descriptions for all variants
    #[test]
    fn test_purify_event_descriptions_all() {
        // PurifyStart
        let start = PurifyEvent::PurifyStart { node_count: 100 };
        assert!(start.description().contains("100 nodes"));

        // TransformationApplied
        let applied = PurifyEvent::TransformationApplied {
            id: 1,
            rule_id: "IDEM001".to_string(),
            node_id: 42,
            patch: AstNodePatch::AddedFlag {
                flag: "-p".to_string(),
            },
            reason: "idempotency".to_string(),
            span: Span::single_line(1, 1, 10),
        };
        assert!(applied.description().contains("IDEM001"));
        assert!(applied.description().contains("idempotency"));

        // TransformationSkipped
        let skipped = PurifyEvent::TransformationSkipped {
            rule_id: "DET003".to_string(),
            node_id: 10,
            reason: "already deterministic".to_string(),
            span: Span::single_line(2, 1, 5),
        };
        assert!(skipped.description().contains("DET003"));
        assert!(skipped.description().contains("already deterministic"));

        // TransformationConflict
        let conflict = PurifyEvent::TransformationConflict {
            id1: 1,
            rule1: "IDEM001".to_string(),
            id2: 2,
            rule2: "SEC001".to_string(),
            node_id: 5,
            resolution: "IDEM takes priority".to_string(),
            span: Span::single_line(3, 1, 10),
        };
        assert!(conflict.description().contains("IDEM001"));
        assert!(conflict.description().contains("SEC001"));
        assert!(conflict.description().contains("IDEM takes priority"));

        // PurifyComplete
        let complete = PurifyEvent::PurifyComplete {
            transformations_applied: 5,
            transformations_skipped: 2,
            conflicts: 1,
            duration: Duration::from_millis(50),
        };
        assert!(complete.description().contains("5 applied"));
        assert!(complete.description().contains("2 skipped"));
        assert!(complete.description().contains("1 conflicts"));
    }

    /// Test: LintEvent descriptions for all variants
    #[test]
    fn test_lint_event_descriptions_all() {
        // LintStart
        let start = LintEvent::LintStart { node_count: 50 };
        assert!(start.description().contains("50 nodes"));

        // RuleEvaluated - passed
        let passed = LintEvent::RuleEvaluated {
            rule_id: "SC2086".to_string(),
            node_id: 1,
            passed: true,
            violation: None,
            span: Span::single_line(1, 1, 10),
        };
        assert!(passed.description().contains("SC2086"));
        assert!(passed.description().contains("passed"));

        // RuleEvaluated - failed without violation details
        let failed = LintEvent::RuleEvaluated {
            rule_id: "SEC001".to_string(),
            node_id: 2,
            passed: false,
            violation: None,
            span: Span::single_line(2, 1, 10),
        };
        assert!(failed.description().contains("SEC001"));
        assert!(failed.description().contains("failed"));

        // RuleEvaluated - failed with violation
        let with_violation = LintEvent::RuleEvaluated {
            rule_id: "SC2034".to_string(),
            node_id: 3,
            passed: false,
            violation: Some(Violation {
                rule_id: "SC2034".to_string(),
                severity: Severity::Warning,
                message: "Unused variable".to_string(),
                span: Span::single_line(3, 1, 10),
                suggestion: None,
                is_security: false,
            }),
            span: Span::single_line(3, 1, 10),
        };
        assert!(with_violation.description().contains("Unused variable"));

        // LintComplete
        let complete = LintEvent::LintComplete {
            rules_evaluated: 100,
            violations: 3,
            duration: Duration::from_millis(25),
        };
        assert!(complete.description().contains("100 rules"));
        assert!(complete.description().contains("3 violations"));
    }

    /// Test: GenerateEvent descriptions for all variants
    #[test]
    fn test_generate_event_descriptions_all() {
        // GenerateStart
        let start = GenerateEvent::GenerateStart { node_count: 25 };
        assert!(start.description().contains("25 nodes"));

        // GenerateCode
        let code = GenerateEvent::GenerateCode {
            node_id: 1,
            bash_code: "mkdir -p foo".to_string(),
            span: Span::single_line(1, 1, 12),
        };
        assert!(code.description().contains("mkdir -p foo"));
        assert!(code.description().contains("1:1"));

        // GenerateComplete
        let complete = GenerateEvent::GenerateComplete {
            output_size: 1024,
            duration: Duration::from_millis(10),
        };
        assert!(complete.description().contains("1024 bytes"));

        // GenerateError
        let error = GenerateEvent::GenerateError {
            error: "Invalid node".to_string(),
            span: Span::single_line(5, 1, 10),
        };
        assert!(error.description().contains("Invalid node"));
        assert!(error.description().contains("5:1"));
    }

    /// Test: TraceEvent significance for all event types
    #[test]
    fn test_trace_event_significance_complete() {
        // Parse events
        assert_eq!(
            TraceEvent::Parse(ParseEvent::ParseNode {
                node_type: "Cmd".to_string(),
                span: Span::single_line(1, 1, 5),
            })
            .significance(),
            TraceSignificance::Low
        );
        assert_eq!(
            TraceEvent::Parse(ParseEvent::ParseComplete {
                node_count: 10,
                duration: Duration::from_millis(1),
            })
            .significance(),
            TraceSignificance::Trace
        );

        // Purify events
        assert_eq!(
            TraceEvent::Purify(PurifyEvent::PurifyStart { node_count: 10 }).significance(),
            TraceSignificance::Trace
        );
        assert_eq!(
            TraceEvent::Purify(PurifyEvent::PurifyComplete {
                transformations_applied: 1,
                transformations_skipped: 0,
                conflicts: 0,
                duration: Duration::from_millis(1),
            })
            .significance(),
            TraceSignificance::Trace
        );
        assert_eq!(
            TraceEvent::Purify(PurifyEvent::TransformationSkipped {
                rule_id: "R1".to_string(),
                node_id: 1,
                reason: "skip".to_string(),
                span: Span::single_line(1, 1, 5),
            })
            .significance(),
            TraceSignificance::Medium
        );

        // Lint events
        assert_eq!(
            TraceEvent::Lint(LintEvent::LintStart { node_count: 5 }).significance(),
            TraceSignificance::Trace
        );
        assert_eq!(
            TraceEvent::Lint(LintEvent::LintComplete {
                rules_evaluated: 10,
                violations: 0,
                duration: Duration::from_millis(1),
            })
            .significance(),
            TraceSignificance::Trace
        );
        // Non-security rule evaluation
        assert_eq!(
            TraceEvent::Lint(LintEvent::RuleEvaluated {
                rule_id: "SC2086".to_string(),
                node_id: 1,
                passed: false,
                violation: Some(Violation {
                    rule_id: "SC2086".to_string(),
                    severity: Severity::Warning,
                    message: "test".to_string(),
                    span: Span::single_line(1, 1, 5),
                    suggestion: None,
                    is_security: false,
                }),
                span: Span::single_line(1, 1, 5),
            })
            .significance(),
            TraceSignificance::Medium
        );

        // Generate events
        assert_eq!(
            TraceEvent::Generate(GenerateEvent::GenerateStart { node_count: 5 }).significance(),
            TraceSignificance::Trace
        );
        assert_eq!(
            TraceEvent::Generate(GenerateEvent::GenerateCode {
                node_id: 1,
                bash_code: "echo".to_string(),
                span: Span::single_line(1, 1, 4),
            })
            .significance(),
            TraceSignificance::Low
        );
        assert_eq!(
            TraceEvent::Generate(GenerateEvent::GenerateComplete {
                output_size: 100,
                duration: Duration::from_millis(1),
            })
            .significance(),
            TraceSignificance::Trace
        );
        assert_eq!(
            TraceEvent::Generate(GenerateEvent::GenerateError {
                error: "err".to_string(),
                span: Span::single_line(1, 1, 5),
            })
            .significance(),
            TraceSignificance::High
        );
    }

    /// Test: All AstNodePatch variants
    #[test]
    fn test_ast_node_patch_all_variants() {
        let patches = [
            AstNodePatch::AddedFlag {
                flag: "-p".to_string(),
            },
            AstNodePatch::RemovedFlag {
                flag: "-f".to_string(),
            },
            AstNodePatch::ReplacedArgument {
                index: 0,
                old: "a".to_string(),
                new: "b".to_string(),
            },
            AstNodePatch::ReplacedExpression {
                old_expr: "$x".to_string(),
                new_expr: "\"$x\"".to_string(),
            },
            AstNodePatch::AddedQuotes {
                variable: "foo".to_string(),
            },
            AstNodePatch::RemovedRandomVar,
            AstNodePatch::ReplacedTimestamp {
                old_pattern: "$(date)".to_string(),
                new_value: "2024-01-01".to_string(),
            },
            AstNodePatch::Generic {
                description: "custom".to_string(),
            },
        ];
        assert_eq!(patches.len(), 8); // All variants covered
    }

    /// Test: Severity enum
    #[test]
    fn test_severity_variants() {
        let severities = [
            Severity::Error,
            Severity::Warning,
            Severity::Info,
            Severity::Style,
        ];
        assert_eq!(severities.len(), 4);
    }
}
