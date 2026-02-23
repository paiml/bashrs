// Tracing Module for bashrs
//
// Architecture: Deep semantic tracing during bash purification and linting
// Inspired by: Kishu (VLDB 2025), Jmvx (OOPSLA2 2024), WhyLine (CHI 2004)
//
// Task: TRACE-001 - Core TraceEvent data structure
// Phase: 1 - Core Tracing Infrastructure (Weeks 1-3)
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 20+ scenarios
// - Property tests: 5+ generators
// - Mutation score: ≥95%
// - Complexity: <10 per function
// - Performance: <10% overhead

#[allow(clippy::expect_used)] // Buffer uses expect() for internal invariants
pub mod buffer;
#[allow(clippy::expect_used)] // Events uses expect() for internal invariants
pub mod events;
pub mod manager;
#[allow(clippy::expect_used)] // Significance uses expect() for internal invariants
pub mod significance;

pub use buffer::CircularTraceBuffer;
pub use events::{GenerateEvent, LintEvent, ParseEvent, PurifyEvent, Span, TraceEvent};
pub use manager::{TraceManager, TraceStats};
pub use significance::TraceSignificance;
