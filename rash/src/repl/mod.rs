// REPL module for bashrs - Interactive REPL with integrated debugger
//
// Architecture: Debugger-as-REPL (matklad pattern)
// Integration: Symbiotic embedding (RuchyRuchy pattern)
//
// Sprint: REPL-003 (Basic REPL Loop)
// Status: Phase 1 - RED-GREEN-REFACTOR-PROPERTY-MUTATION

pub mod completion;
pub mod config;
pub mod explain;
pub mod linter;
mod r#loop;
pub mod modes;
pub mod parser;
pub mod purifier;
pub mod state;
pub mod variables;

pub use config::ReplConfig;
pub use explain::{explain_bash, Explanation};
pub use linter::{format_lint_results, lint_bash};
pub use modes::ReplMode;
pub use parser::{format_parse_error, parse_bash};
pub use purifier::{format_purification_report, purify_bash};
pub use r#loop::run_repl;
pub use state::ReplState;
