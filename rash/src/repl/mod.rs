// REPL module for bashrs - Interactive REPL with integrated debugger
//
// Architecture: Debugger-as-REPL (matklad pattern)
// Integration: Symbiotic embedding (RuchyRuchy pattern)
//
// Sprint: REPL-003 (Basic REPL Loop)
// Status: Phase 1 - RED-GREEN-REFACTOR-PROPERTY-MUTATION

pub mod ast_display;
pub mod breakpoint;
pub mod completion;
pub mod config;
pub mod debugger;
pub mod determinism;
pub mod diff;
pub mod executor;
pub mod explain;
pub mod help;
pub mod linter;
pub mod loader;
mod r#loop;
pub mod modes;
pub mod multiline;
pub mod parser;
pub mod purifier;
pub mod state;
pub mod variables;

pub use ast_display::format_ast;
pub use breakpoint::{Breakpoint, BreakpointManager};
pub use config::ReplConfig;
pub use debugger::{ContinueResult, DebugSession, LineComparison, StackFrame};
pub use determinism::{
    format_idempotency_report, format_replay_diff, DeterminismChecker, DeterminismIssue,
    IdempotencyChecker, IdempotencyIssue, IdempotencyResult, IdempotencyVerifier,
    NonDeterministicPattern, NonIdempotentOperation, OutputDifference, ReplayResult,
    ReplayVerifier, RunOutput,
};
pub use diff::display_diff;
pub use explain::{explain_bash, Explanation};
pub use linter::{format_lint_results, lint_bash};
pub use modes::ReplMode;
pub use parser::{format_parse_error, parse_bash};
pub use purifier::{
    explain_purification_changes, explain_purification_changes_detailed, format_transformation_report,
    purify_bash, Alternative, SafetyRationale, SafetySeverity, TransformationCategory,
    TransformationExplanation,
};
pub use r#loop::run_repl;
pub use state::ReplState;
