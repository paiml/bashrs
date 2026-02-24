//! # bashrs REPL - Interactive Shell Analysis Environment
//!
//! The REPL (Read-Eval-Print Loop) module provides an interactive environment for
//! analyzing, transforming, and debugging bash scripts in real-time.
//!
//! ## Overview
//!
//! The bashrs REPL is a production-ready interactive shell that combines:
//!
//! - **5 Interactive Modes**: Normal, purify, lint, debug, and explain modes
//! - **Real-time Analysis**: Parse, lint, and purify bash code interactively
//! - **Script Management**: Load scripts, extract functions, reload after changes
//! - **Session State**: Persistent variables, history, and context
//! - **Tab Completion**: Intelligent completion for commands, modes, and files
//! - **Multi-line Input**: Natural support for functions, loops, and conditionals
//!
//! ## Architecture
//!
//! The REPL follows the **Debugger-as-REPL** pattern (matklad pattern):
//! - Debugger capabilities are exposed through REPL commands
//! - Symbiotic embedding enables deep integration with bash analysis tools
//!
//! ## Quick Start
//!
//! ```bash
//! $ bashrs repl
//! bashrs [normal]> echo "Hello, REPL!"
//! Hello, REPL!
//!
//! bashrs [normal]> :parse echo test
//! ✓ Parse successful!
//!
//! bashrs [normal]> :mode lint
//! Switched to lint mode
//!
//! bashrs [lint]> cat $FILE | grep pattern
//! Found 1 issue(s): SC2086 - Unquoted variable
//! ```
//!
//! ## Modules
//!
//! - [`ast_display`]: Format and display AST structures
//! - [`breakpoint`]: Debugger breakpoint management
//! - [`completion`]: Tab completion for commands and files
//! - [`config`]: REPL configuration and settings
//! - [`debugger`]: Interactive debugging capabilities
//! - [`determinism`]: Detect non-deterministic patterns
//! - [`errors`]: Error formatting and reporting
//! - [`executor`]: Execute bash commands safely
//! - [`explain`]: Explain bash constructs interactively
//! - [`highlighting`]: Syntax highlighting for bash code
//! - [`linter`]: Real-time linting and diagnostics
//! - [`loader`]: Script loading and function extraction
//! - [`modes`]: REPL mode management (normal, purify, lint, etc.)
//! - [`multiline`]: Multi-line input handling
//! - [`parser`]: Bash parsing integration
//! - [`purifier`]: Idempotency and determinism transformations
//! - [`state`]: Session state management
//! - [`variables`]: Variable storage and expansion
//!
//! ## Features
//!
//! ### Interactive Modes
//!
//! The REPL supports 5 specialized modes:
//!
//! | Mode | Purpose | Use Case |
//! |------|---------|----------|
//! | **normal** | Direct execution | Testing commands, learning bash |
//! | **purify** | Automatic purification | Fixing idempotency issues |
//! | **lint** | Automatic linting | Finding security problems |
//! | **debug** | Step-by-step execution | Understanding complex scripts |
//! | **explain** | Interactive explanations | Learning bash constructs |
//!
//! ### Commands
//!
//! Core commands for bash analysis:
//!
//! - `:parse <code>` - Parse bash code and show AST
//! - `:lint <code>` - Lint for security and quality issues
//! - `:purify <code>` - Transform to idempotent/deterministic code
//! - `:mode <name>` - Switch to a different mode
//! - `:load <file>` - Load and analyze a bash script
//! - `:reload` - Reload the most recently loaded script
//! - `:vars` - Show session variables
//! - `:history` - Show command history
//!
//! ### Script Loading
//!
//! Load complete scripts for analysis:
//!
//! ```bash
//! bashrs [normal]> :load deploy.sh
//! ✓ Loaded: deploy.sh (5 functions, 120 lines)
//!
//! bashrs [normal]> :functions
//! Available functions (5 total):
//!   1 validate_env
//!   2 build_app
//!   3 run_tests
//!   4 deploy_staging
//!   5 deploy_production
//! ```
//!
//! ### Variable Management
//!
//! Persistent variables across your session:
//!
//! ```bash
//! bashrs [normal]> app_name="myapp"
//! ✓ Variable set: app_name = myapp
//!
//! bashrs [normal]> version=1.0.0
//! ✓ Variable set: version = 1.0.0
//!
//! bashrs [normal]> echo $app_name v$version
//! myapp v1.0.0
//! ```
//!
//! ## Examples
//!
//! See the [`examples/repl/`](../../../examples/repl/) directory for 11 comprehensive
//! real-world examples covering:
//!
//! - Basic REPL workflow
//! - Security auditing
//! - Purification workflows
//! - CI/CD pipeline development
//! - Configuration management
//! - Multi-line editing
//! - Tab completion
//! - Variable management
//! - Troubleshooting
//!
//! ## API Usage
//!
//! ```rust,ignore
//! use bashrs::repl::{run_repl, ReplConfig};
//!
//! // Run the REPL with default configuration
//! let config = ReplConfig::default();
//! run_repl(config)?;
//! ```
//!
//! ## Implementation Status
//!
//! - ✅ **Phase 0** (REPL-001-002): Complete - Core infrastructure
//! - ✅ **Phase 1** (REPL-003-008): Complete - Basic REPL loop and modes
//! - ✅ **Phase 2** (REPL-009-016): Complete - Advanced features (variables, loading, completion)
//! - 🚧 **Phase 3** (REPL-017): In Progress - Documentation
//! - ⏳ **Phase 4** (REPL-018+): Planned - Testing and validation
//!
//! ## See Also
//!
//! - User Guide: [`book/src/repl/user-guide.md`](../../../book/src/repl/user-guide.md)
//! - Tutorial: [`book/src/repl/tutorial.md`](../../../book/src/repl/tutorial.md)
//! - Examples: [`examples/repl/`](../../../examples/repl/)
//!
//! Architecture: Debugger-as-REPL (matklad pattern)
//! Integration: Symbiotic embedding (RuchyRuchy pattern)
//! Sprint: REPL-003 (Basic REPL Loop)
//! Status: Phase 1 - RED-GREEN-REFACTOR-PROPERTY-MUTATION

pub mod ast_display;
pub mod breakpoint;
pub mod completion;
pub mod config;
pub mod debugger;
pub mod determinism;
pub mod diff;
pub mod errors;
pub mod executor;
pub mod explain;
pub mod help;
pub mod highlighting;
pub mod linter;
pub mod loader;
pub mod logic;
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
pub use errors::{
    format_command_error, format_error, format_lint_error, format_source_context, suggest_command,
    ErrorMessage, ErrorType, Severity, SourceContext, Suggestion,
};
pub use explain::{explain_bash, Explanation};
pub use highlighting::{highlight_bash, is_keyword, strip_ansi_codes, tokenize, Token, TokenType};
pub use linter::{format_lint_results, format_violations_with_context, lint_bash};
pub use modes::ReplMode;
pub use parser::{format_parse_error, parse_bash};
pub use purifier::{
    explain_purification_changes, explain_purification_changes_detailed,
    format_purified_lint_result, format_purified_lint_result_with_context,
    format_transformation_report, purify_and_lint, purify_and_validate, purify_bash, Alternative,
    PurificationError, PurifiedLintResult, SafetyRationale, SafetySeverity, TransformationCategory,
    TransformationExplanation,
};
pub use r#loop::run_repl;
pub use state::ReplState;

#[cfg(test)]
#[path = "help_coverage_tests.rs"]
mod help_coverage_tests;

#[cfg(test)]
#[path = "purifier_coverage_tests.rs"]
mod purifier_coverage_tests;
