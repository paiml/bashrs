pub mod args;
pub mod b2_logic;
pub mod bench;
pub mod color;
pub mod commands;
pub mod comply_logic;
pub mod config_logic;
pub mod corpus_analysis_logic;
pub mod corpus_convergence_logic;
pub mod corpus_display_logic;
pub mod corpus_entry_logic;
pub mod corpus_logic;
pub mod corpus_score_logic;
pub mod dockerfile_cmd_logic;
pub mod dockerfile_logic;
pub mod format_logic;
pub mod gate_logic;
pub mod installer_commands;
pub mod dockerfile_commands;
pub mod installer_golden_logic;
pub mod installer_logic;
pub mod installer_run_logic;
pub mod lint_logic;
pub mod logic;
pub mod make_logic;
pub mod score_logic;

#[cfg(test)]
mod tests;

pub use args::{Cli, Commands, ConfigCommands, ConfigOutputFormat};
pub use commands::execute_command;
