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

pub mod events;
pub mod buffer;
pub mod significance;
pub mod manager;

pub use events::{TraceEvent, ParseEvent, PurifyEvent, LintEvent, GenerateEvent, Span};
pub use buffer::CircularTraceBuffer;
pub use significance::TraceSignificance;
pub use manager::{TraceManager, TraceStats};
