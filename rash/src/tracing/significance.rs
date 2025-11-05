// Trace Significance Metric
//
// Task: TRACE-001-A - TraceSignificance ranking system
// Source: bashrs-tracing-architectural-refinements.md (Refinement 1)
//
// Purpose: Rank trace events by importance to prevent information overload
// Principle: Jidoka (Automation with Human Touch) - guide developer attention
//
// Design:
// - 5-level significance hierarchy (Critical → Trace)
// - Critical events always visible (transformation conflicts, security)
// - Default filter: HIGH + CRITICAL
// - Verbose filter: MEDIUM + HIGH + CRITICAL
// - Full filter: All levels including TRACE/LOW

use serde::{Deserialize, Serialize};

/// Trace event significance levels
///
/// Higher significance events bubble to the top in UI, preventing
/// information overload by filtering low-value noise.
///
/// Inspired by: Science of Computer Programming 2024 (circular buffer paper)
/// - Target: 80%+ bug coverage with 1024 events
/// - Approach: Rank by importance, not chronology
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TraceSignificance {
    /// Internal engine events (structural only)
    ///
    /// Examples:
    /// - ParseStart, PurifyStart, LintStart
    /// - Module initialization
    /// - Buffer management
    ///
    /// Visibility: Only in `--trace --verbose --all` mode
    Trace = 0,

    /// Low-importance events (fine-grained details)
    ///
    /// Examples:
    /// - ParseNode (individual AST nodes)
    /// - GenerateCode (code generation steps)
    /// - Buffer statistics
    ///
    /// Visibility: Only in `--trace --verbose --all` mode
    Low = 1,

    /// Medium-importance events (rule evaluations)
    ///
    /// Examples:
    /// - TransformationSkipped (rule didn't match)
    /// - RuleEvaluated (rule checked, passed)
    /// - Informational messages
    ///
    /// Visibility: `--trace --verbose` (MEDIUM + HIGH + CRITICAL)
    Medium = 2,

    /// High-importance events (actual transformations)
    ///
    /// Examples:
    /// - TransformationApplied (IDEM001 added -p flag)
    /// - ParseError (syntax error in input)
    /// - GenerationError (failed to generate output)
    ///
    /// Visibility: `--trace` (default: HIGH + CRITICAL)
    High = 3,

    /// Critical events (conflicts, security violations)
    ///
    /// Examples:
    /// - TransformationConflict (IDEM003 vs SEC001 conflict)
    /// - Security violations (SEC* rules fired)
    /// - Fatal errors (engine crashes, panics)
    ///
    /// Visibility: **Always visible**, cannot be filtered out
    Critical = 4,
}

impl TraceSignificance {
    /// Check if this significance level should be visible in default mode
    ///
    /// Default mode: Show HIGH + CRITICAL only
    ///
    /// # Example
    /// ```
    /// use rash::tracing::TraceSignificance;
    ///
    /// assert!(TraceSignificance::Critical.is_default_visible());
    /// assert!(TraceSignificance::High.is_default_visible());
    /// assert!(!TraceSignificance::Medium.is_default_visible());
    /// ```
    #[must_use]
    pub const fn is_default_visible(self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }

    /// Check if this significance level should be visible in verbose mode
    ///
    /// Verbose mode: Show MEDIUM + HIGH + CRITICAL
    ///
    /// # Example
    /// ```
    /// use rash::tracing::TraceSignificance;
    ///
    /// assert!(TraceSignificance::Medium.is_verbose_visible());
    /// assert!(!TraceSignificance::Low.is_verbose_visible());
    /// ```
    #[must_use]
    pub const fn is_verbose_visible(self) -> bool {
        matches!(self, Self::Medium | Self::High | Self::Critical)
    }

    /// Get a human-readable label for this significance level
    ///
    /// Used for UI display (e.g., `[CRITICAL]`, `[HIGH]`)
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }

    /// Get an ANSI color code for terminal display
    ///
    /// - CRITICAL: Red (bright)
    /// - HIGH: Yellow
    /// - MEDIUM: Cyan
    /// - LOW: White (dim)
    /// - TRACE: Gray
    #[must_use]
    pub const fn ansi_color(self) -> &'static str {
        match self {
            Self::Critical => "\x1b[1;31m", // Bright red
            Self::High => "\x1b[33m",       // Yellow
            Self::Medium => "\x1b[36m",     // Cyan
            Self::Low => "\x1b[2;37m",      // Dim white
            Self::Trace => "\x1b[90m",      // Gray
        }
    }

    /// ANSI reset code
    #[must_use]
    pub const fn ansi_reset() -> &'static str {
        "\x1b[0m"
    }
}

impl Default for TraceSignificance {
    /// Default significance is MEDIUM (balanced)
    fn default() -> Self {
        Self::Medium
    }
}

impl std::fmt::Display for TraceSignificance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: TRACE-001-A-001 - TraceSignificance ordering
    #[test]
    fn test_trace_001_a_001_significance_ordering() {
        assert!(TraceSignificance::Critical > TraceSignificance::High);
        assert!(TraceSignificance::High > TraceSignificance::Medium);
        assert!(TraceSignificance::Medium > TraceSignificance::Low);
        assert!(TraceSignificance::Low > TraceSignificance::Trace);
    }

    /// Test: TRACE-001-A-002 - Default visibility (HIGH + CRITICAL)
    #[test]
    fn test_trace_001_a_002_default_visibility() {
        assert!(TraceSignificance::Critical.is_default_visible());
        assert!(TraceSignificance::High.is_default_visible());
        assert!(!TraceSignificance::Medium.is_default_visible());
        assert!(!TraceSignificance::Low.is_default_visible());
        assert!(!TraceSignificance::Trace.is_default_visible());
    }

    /// Test: TRACE-001-A-003 - Verbose visibility (MEDIUM + HIGH + CRITICAL)
    #[test]
    fn test_trace_001_a_003_verbose_visibility() {
        assert!(TraceSignificance::Critical.is_verbose_visible());
        assert!(TraceSignificance::High.is_verbose_visible());
        assert!(TraceSignificance::Medium.is_verbose_visible());
        assert!(!TraceSignificance::Low.is_verbose_visible());
        assert!(!TraceSignificance::Trace.is_verbose_visible());
    }

    /// Test: TRACE-001-A-004 - Labels are correct
    #[test]
    fn test_trace_001_a_004_labels() {
        assert_eq!(TraceSignificance::Critical.label(), "CRITICAL");
        assert_eq!(TraceSignificance::High.label(), "HIGH");
        assert_eq!(TraceSignificance::Medium.label(), "MEDIUM");
        assert_eq!(TraceSignificance::Low.label(), "LOW");
        assert_eq!(TraceSignificance::Trace.label(), "TRACE");
    }

    /// Test: TRACE-001-A-005 - ANSI colors are defined
    #[test]
    fn test_trace_001_a_005_ansi_colors() {
        assert!(!TraceSignificance::Critical.ansi_color().is_empty());
        assert!(!TraceSignificance::High.ansi_color().is_empty());
        assert!(!TraceSignificance::ansi_reset().is_empty());
    }

    /// Test: TRACE-001-A-006 - Default is MEDIUM
    #[test]
    fn test_trace_001_a_006_default() {
        assert_eq!(TraceSignificance::default(), TraceSignificance::Medium);
    }

    /// Test: TRACE-001-A-007 - Display trait works
    #[test]
    fn test_trace_001_a_007_display() {
        assert_eq!(format!("{}", TraceSignificance::Critical), "CRITICAL");
        assert_eq!(format!("{}", TraceSignificance::High), "HIGH");
    }

    /// Test: TRACE-001-A-008 - Serialization works
    #[test]
    fn test_trace_001_a_008_serialization() {
        let sig = TraceSignificance::Critical;
        let json = serde_json::to_string(&sig).expect("Serialization failed");
        let deserialized: TraceSignificance =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(sig, deserialized);
    }

    // ===== PROPERTY TESTS =====

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        // Property: Higher significance is always more visible
        proptest! {
            #[test]
            fn prop_trace_001_a_higher_more_visible(
                sig_low in prop::sample::select(vec![
                    TraceSignificance::Trace,
                    TraceSignificance::Low,
                ]),
                sig_high in prop::sample::select(vec![
                    TraceSignificance::High,
                    TraceSignificance::Critical,
                ]),
            ) {
                // Property: High/Critical always more visible than Trace/Low
                assert!(sig_high.is_default_visible() || sig_high.is_verbose_visible());
                assert!(!sig_low.is_default_visible());
            }
        }

        // Property: Critical events always visible
        proptest! {
            #[test]
            fn prop_trace_001_a_critical_always_visible(_any in 0..100u32) {
                // Property: Critical events CANNOT be filtered out
                assert!(TraceSignificance::Critical.is_default_visible());
                assert!(TraceSignificance::Critical.is_verbose_visible());
            }
        }

        // Property: Ordering is transitive
        proptest! {
            #[test]
            fn prop_trace_001_a_ordering_transitive(
                a in prop::sample::select(vec![
                    TraceSignificance::Trace,
                    TraceSignificance::Low,
                    TraceSignificance::Medium,
                ]),
                b in prop::sample::select(vec![
                    TraceSignificance::Medium,
                    TraceSignificance::High,
                ]),
                c in prop::sample::select(vec![
                    TraceSignificance::High,
                    TraceSignificance::Critical,
                ]),
            ) {
                // Property: If a < b and b < c, then a < c (transitivity)
                if a < b && b < c {
                    prop_assert!(a < c);
                }
            }
        }
    }
}
