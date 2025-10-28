// REPL module for bashrs - Interactive REPL with integrated debugger
//
// Architecture: Debugger-as-REPL (matklad pattern)
// Integration: Symbiotic embedding (RuchyRuchy pattern)
//
// Sprint: REPL-003 (Basic REPL Loop)
// Status: Phase 1 - RED-GREEN-REFACTOR-PROPERTY-MUTATION

pub mod config;
pub mod state;
pub mod modes;
pub mod parser;
pub mod purifier;
pub mod linter;
mod r#loop;

pub use config::ReplConfig;
pub use state::ReplState;
pub use modes::ReplMode;
pub use parser::{parse_bash, format_parse_error};
pub use purifier::{purify_bash, format_purification_report};
pub use linter::{lint_bash, format_lint_results};
pub use r#loop::run_repl;
